use crate::commonmark_rules;
use crate::node::{Node, NodeType};
use crate::parser;
use crate::rules::{Rule, RuleFilter, Rules};
use crate::utilities::{trim_leading_newlines, trim_trailing_newlines};
use regex::Regex;
use std::collections::HashMap;
use std::fmt;

/// Configuration options for Turndown
#[derive(Clone)]
pub struct Options {
    /// Conversion rules
    pub rules: HashMap<String, Rule>,
    /// Style for rendering headings: Setext or Atx (default: Atx)
    pub heading_style: HeadingStyle,
    /// Used to render horizontal rules (default: * * *)
    pub hr: String,
    /// Marker used for bullet lists (default: *)
    pub bullet_list_marker: String,
    /// Style for rendering code blocks: Indented or Fenced (default: Fenced)
    pub code_block_style: CodeBlockStyle,
    /// Delimiter used for fenced code blocks (default: ```)
    pub fence: String,
    /// Delimiter used for emphasis (default: _)
    pub em_delimiter: String,
    /// Delimiter used for strong emphasis (default: **)
    pub strong_delimiter: String,
    /// Style for rendering links: Inlined or Referenced (default: Inlined)
    pub link_style: LinkStyle,
    /// Style for link references: Full, Collapsed, or Shortcut (default: Full)
    pub link_reference_style: LinkReferenceStyle,
    /// String used for line breaks (default: two spaces)
    pub br: String,
    /// Options for stripping tracking images (default: false)
    pub strip_tracking_images: bool,
    /// Regex to identify tracking images, comes with a sensible default
    pub tracking_image_regex: Option<Regex>,
    /// Option to strip images without alt attributes (default: false)
    pub strip_images_without_alt: bool,
}

impl fmt::Debug for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Options")
            .field("rules", &self.rules)
            .field("heading_style", &self.heading_style)
            .field("hr", &self.hr)
            .field("bullet_list_marker", &self.bullet_list_marker)
            .field("code_block_style", &self.code_block_style)
            .field("fence", &self.fence)
            .field("em_delimiter", &self.em_delimiter)
            .field("strong_delimiter", &self.strong_delimiter)
            .field("link_style", &self.link_style)
            .field("link_reference_style", &self.link_reference_style)
            .field("br", &self.br)
            .field("strip_tracking_images", &self.strip_tracking_images)
            .field(
                "tracking_image_regex",
                &self.tracking_image_regex.as_ref().map(|_| "<regex>"),
            )
            .field("strip_images_without_alt", &self.strip_images_without_alt)
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum HeadingStyle {
    Setext,
    Atx,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CodeBlockStyle {
    Indented,
    Fenced,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LinkStyle {
    Inlined,
    Referenced,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LinkReferenceStyle {
    Full,
    Collapsed,
    Shortcut,
}

impl Default for Options {
    fn default() -> Self {
        // Create default tracking image regex with common tracking indicators
        // This regex targets specific patterns that are almost certainly tracking pixels
        let tracking_regex = Regex::new(
            r"(?i)(pixel|beacon|\.com/ts|splash.tools/o/|tr/op|track|klclick.com/o/|ho\.gif|transp|msg_del_|analytics|spacer|tagpixel|emimp/ip_|utm_|/open\?|\.gif\?|1x1|/tr/|/track\.)",
        )
        .ok();

        Options {
            rules: commonmark_rules::get_rules(),
            heading_style: HeadingStyle::Atx,
            hr: "* * *".to_string(),
            bullet_list_marker: "*".to_string(),
            code_block_style: CodeBlockStyle::Fenced,
            fence: "```".to_string(),
            em_delimiter: "_".to_string(),
            strong_delimiter: "**".to_string(),
            link_style: LinkStyle::Inlined,
            link_reference_style: LinkReferenceStyle::Full,
            br: "  ".to_string(),
            strip_tracking_images: false,
            tracking_image_regex: tracking_regex,
            strip_images_without_alt: false,
        }
    }
}

pub type TurndownOptions = Options;

/// Main turndown for converting HTML to Markdown
pub struct Turndown {
    pub options: TurndownOptions,
    pub rules: Rules,
    escape_patterns: Vec<(Regex, String)>,
}

/// Context for list processing
#[derive(Clone, Debug)]
struct ListContext {
    pub list_type: String, // "OL" or "UL"
    pub item_index: usize, // 1-based index for items
}

impl Turndown {
    /// Creates a new Turndown with default options
    pub fn new() -> Self {
        Self::with_options(TurndownOptions::default())
    }

    /// Creates a Turndown with custom options
    pub fn with_options(options: TurndownOptions) -> Self {
        let rules = Rules::new(options.clone());

        let escape_patterns = vec![
            (Regex::new(r"\\").unwrap(), "\\\\".to_string()),
            (Regex::new(r"\*").unwrap(), "\\*".to_string()),
            (Regex::new(r"^-").unwrap(), "\\-".to_string()),
            (Regex::new(r"^\+ ").unwrap(), "\\+ ".to_string()),
            (Regex::new(r"^(=+)").unwrap(), "\\$1".to_string()),
            (Regex::new(r"^(#{1,6}) ").unwrap(), "\\$1 ".to_string()),
            (Regex::new(r"`").unwrap(), "\\`".to_string()),
            (Regex::new(r"^~~~").unwrap(), "\\~~~".to_string()),
            (Regex::new(r"\[").unwrap(), "\\[".to_string()),
            (Regex::new(r"\]").unwrap(), "\\]".to_string()),
            (Regex::new(r"^>").unwrap(), "\\>".to_string()),
            (Regex::new(r"_").unwrap(), "\\_".to_string()),
            (Regex::new(r"^(\d+)\. ").unwrap(), "$1\\. ".to_string()),
        ];

        Turndown {
            options,
            rules,
            escape_patterns,
        }
    }

    /// Converts HTML to Markdown
    pub fn convert(&self, html: &str) -> String {
        if html.is_empty() {
            return String::new();
        }

        let root = parser::parse_html(html);
        let output = self.process_with_context(&root, None);
        self.post_process(&output)
    }

    /// Processes a node and its children recursively with optional list context
    fn process_with_context(&self, node: &Node, list_context: Option<ListContext>) -> String {
        self.process_with_full_context(node, list_context, false)
    }

    /// Processes a node and its children recursively with full context
    fn process_with_full_context(
        &self,
        node: &Node,
        list_context: Option<ListContext>,
        in_pre: bool,
    ) -> String {
        let mut output = String::new();
        let mut item_index = 0;

        // Determine if this is a list element
        let is_list = matches!(node.node_name.as_str(), "OL" | "UL");
        let new_list_context = if is_list {
            Some(ListContext {
                list_type: node.node_name.clone(),
                item_index: 0,
            })
        } else {
            list_context.clone()
        };

        // Determine if we're entering a PRE block
        let new_in_pre = in_pre || node.node_name == "PRE";

        for child in &node.children {
            let replacement = if child.node_type == NodeType::Text {
                if child.is_code {
                    child.node_value.clone()
                } else {
                    self.escape(&child.node_value)
                }
            } else if child.node_type == NodeType::Element {
                // Increment item index for LI elements
                if child.node_name == "LI" && new_list_context.is_some() {
                    item_index += 1;
                    let mut context_with_index = new_list_context.clone().unwrap();
                    context_with_index.item_index = item_index;
                    self.replacement_for_node_with_full_context(
                        child,
                        Some(context_with_index),
                        new_in_pre,
                    )
                } else {
                    self.replacement_for_node_with_full_context(
                        child,
                        new_list_context.clone(),
                        new_in_pre,
                    )
                }
            } else {
                String::new()
            };

            output = self.join(&output, &replacement);
        }

        output
    }

    /// Gets replacement for an element node with full context
    fn replacement_for_node_with_full_context(
        &self,
        node: &Node,
        list_context: Option<ListContext>,
        in_pre: bool,
    ) -> String {
        let new_in_pre = in_pre || node.node_name == "PRE";
        let mut content = self.process_with_full_context(node, list_context.clone(), new_in_pre);

        let whitespace = node.flanking_whitespace();

        let is_table_cell = matches!(node.node_name.as_str(), "TD" | "TH");

        if node.is_block() {
            content = content.trim_start().to_string();
        }

        let (use_leading, use_trailing) = if is_table_cell || node.is_block() {
            (String::new(), String::new())
        } else {
            (whitespace.leading.clone(), whitespace.trailing.clone())
        };

        if !whitespace.leading.is_empty() || !whitespace.trailing.is_empty() {
            content = content.trim().to_string();
        }

        let mut node_with_context = node.clone();
        if let Some(ctx) = list_context {
            node_with_context.set_attribute("data-list-type", &ctx.list_type);
            node_with_context.set_attribute("data-list-index", &ctx.item_index.to_string());
        }
        if new_in_pre {
            node_with_context.set_attribute("data-in-pre", "true");
        }

        let rule = self.rules.for_node(&node_with_context);

        format!(
            "{}{}{}",
            use_leading,
            (rule.replacement)(&content, &node_with_context, &self.options),
            use_trailing
        )
    }

    /// Post-processes the output
    fn post_process(&self, output: &str) -> String {
        let collapsed = self.collapse_excessive_newlines(output);
        let trimmed = collapsed
            .trim_start_matches(|c| c == '\t' || c == '\r' || c == '\n')
            .trim_end_matches(|c| c == '\t' || c == '\r' || c == '\n' || c == ' ');

        trimmed.to_string()
    }

    /// Collapses sequences of 3+ newlines down to 2 newlines (representing 1 blank line)
    fn collapse_excessive_newlines(&self, s: &str) -> String {
        let mut result = String::new();
        let mut consecutive_newlines = 0;

        for ch in s.chars() {
            if ch == '\n' {
                consecutive_newlines += 1;
                if consecutive_newlines <= 2 {
                    result.push(ch);
                }
            } else if ch == ' ' || ch == '\t' || ch == '\r' {
                result.push(ch);
            } else {
                consecutive_newlines = 0;
                result.push(ch);
            }
        }

        result
    }

    /// Escapes Markdown special characters
    pub fn escape(&self, string: &str) -> String {
        let mut result = string.to_string();
        for (pattern, replacement) in &self.escape_patterns {
            result = pattern
                .replace_all(&result, replacement.as_str())
                .to_string();
        }
        result
    }

    /// Joins two strings with appropriate newlines
    fn join(&self, output: &str, replacement: &str) -> String {
        let s1 = trim_trailing_newlines(output);
        let s2 = trim_leading_newlines(replacement);

        let output_newlines = output.len() - s1.len();
        let replacement_newlines = replacement.len() - s2.len();
        let nls = output_newlines.max(replacement_newlines);

        let separator = if nls >= 2 {
            "\n\n"
        } else if nls == 1 {
            "\n"
        } else {
            ""
        };

        format!("{}{}{}", s1, separator, s2)
    }

    /// Adds a custom rule
    pub fn add_rule(&mut self, key: String, rule: Rule) {
        self.rules.add(key, rule);
    }

    /// Keeps nodes matching a filter as HTML
    pub fn keep(&mut self, filter: RuleFilter) {
        self.rules.keep(filter);
    }

    /// Removes nodes matching a filter
    pub fn remove(&mut self, filter: RuleFilter) {
        self.rules.remove(filter);
    }
}

impl Default for Turndown {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let turndown = Turndown::new();
        assert_eq!(turndown.options.heading_style, HeadingStyle::Atx);
    }

    #[test]
    fn test_escape() {
        let turndown = Turndown::new();
        let text = "Test [brackets] and *asterisks*";
        let escaped = turndown.escape(text);
        assert!(escaped.contains("\\["));
        assert!(escaped.contains("\\*"));
    }

    #[test]
    fn test_empty_input() {
        let turndown = Turndown::new();
        let result = turndown.convert("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_simple_paragraph() {
        let turndown = Turndown::new();
        let html = "<p>Hello World</p>";
        let result = turndown.convert(html);
        assert!(!result.is_empty());
    }
}
