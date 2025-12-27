use crate::rules::{Rule, RuleFilter};
use crate::utilities::{is_tracking_image, repeat, trim_newlines};
use std::collections::HashMap;

pub fn get_rules() -> HashMap<String, Rule> {
    let mut rules = HashMap::new();

    rules.insert("paragraph".to_string(), paragraph_rule());
    rules.insert("lineBreak".to_string(), line_break_rule());
    rules.insert("heading".to_string(), heading_rule());
    rules.insert("blockquote".to_string(), blockquote_rule());
    rules.insert("list".to_string(), list_rule());
    rules.insert("listItem".to_string(), list_item_rule());
    rules.insert("indentedCodeBlock".to_string(), indented_code_block_rule());
    rules.insert("fencedCodeBlock".to_string(), fenced_code_block_rule());
    rules.insert("horizontalRule".to_string(), horizontal_rule_rule());
    rules.insert("inlineLink".to_string(), inline_link_rule());
    rules.insert("referenceLink".to_string(), reference_link_rule());
    rules.insert("emphasis".to_string(), emphasis_rule());
    rules.insert("strong".to_string(), strong_rule());
    rules.insert("code".to_string(), code_rule());
    rules.insert("image".to_string(), image_rule());
    rules.insert("comment".to_string(), comment_rule());
    rules.insert(
        "processingInstruction".to_string(),
        processing_instruction_rule(),
    );
    rules.insert("style".to_string(), style_rule());
    rules.insert("script".to_string(), script_rule());
    rules.insert("hiddenPreheader".to_string(), hidden_preheader_rule());
    rules.insert("superscript".to_string(), superscript_rule());
    rules.insert("subscript".to_string(), subscript_rule());

    rules
}

fn comment_rule() -> Rule {
    Rule {
        filter: RuleFilter::Function(|node, _| node.node_type == crate::node::NodeType::Comment),
        replacement: |_, _, _| String::new(),
    }
}

fn processing_instruction_rule() -> Rule {
    Rule {
        filter: RuleFilter::Function(|node, _| {
            node.node_type == crate::node::NodeType::ProcessingInstruction
        }),
        replacement: |_, _, _| String::new(),
    }
}

fn style_rule() -> Rule {
    Rule {
        filter: RuleFilter::String("style".to_string()),
        replacement: |_, _, _| String::new(),
    }
}

fn script_rule() -> Rule {
    Rule {
        filter: RuleFilter::String("script".to_string()),
        replacement: |_, _, _| String::new(),
    }
}

fn hidden_preheader_rule() -> Rule {
    Rule {
        filter: RuleFilter::Function(|node, _| {
            node.node_name == "DIV"
                && (node.get_attribute("data-email-preheader").is_some()
                    || (node.get_attribute("style")
                        .map(|s| s.contains("visibility:hidden") && s.contains("height:0"))
                        .unwrap_or(false)
                        && node.get_attribute("class")
                            .map(|c| c.contains("h-0") && c.contains("opacity-0"))
                            .unwrap_or(false)))
        }),
        replacement: |content, _, _| {
            // Keep the content inline - no block formatting
            // This allows preheader text and invisible characters to stay on same line
            content.trim().to_string()
        },
    }
}

fn paragraph_rule() -> Rule {
    Rule {
        filter: RuleFilter::String("p".to_string()),
        replacement: |content, node, _| {
            if node.get_attribute("data-list-type").is_some() {
                content.to_string()
            } else {
                format!("\n\n{}\n\n", content)
            }
        },
    }
}

fn line_break_rule() -> Rule {
    Rule {
        filter: RuleFilter::String("br".to_string()),
        replacement: |_, _, options| format!("{}\n", options.br),
    }
}

fn heading_rule() -> Rule {
    Rule {
        filter: RuleFilter::Array(vec![
            "h1".to_string(),
            "h2".to_string(),
            "h3".to_string(),
            "h4".to_string(),
            "h5".to_string(),
            "h6".to_string(),
        ]),
        replacement: |content, node, options| {
            let h_level = node
                .node_name
                .chars()
                .nth(1)
                .and_then(|c| c.to_digit(10))
                .unwrap_or(1) as usize;

            if options.heading_style == crate::HeadingStyle::Setext && h_level < 3 {
                let underline = repeat(if h_level == 1 { '=' } else { '-' }, content.len());
                format!("\n\n{}\n{}\n\n", content, underline)
            } else {
                format!("\n\n{} {}\n\n", repeat('#', h_level), content)
            }
        },
    }
}

fn blockquote_rule() -> Rule {
    Rule {
        filter: RuleFilter::String("blockquote".to_string()),
        replacement: |content, _, _| {
            let trimmed = trim_newlines(content);
            let quoted = trimmed
                .lines()
                .map(|line| format!("> {}", line))
                .collect::<Vec<_>>()
                .join("\n");
            format!("\n\n{}\n\n", quoted)
        },
    }
}

fn list_rule() -> Rule {
    Rule {
        filter: RuleFilter::Array(vec!["ul".to_string(), "ol".to_string()]),
        replacement: |content, _node, _| format!("\n\n{}\n\n", content),
    }
}

fn list_item_rule() -> Rule {
    Rule {
        filter: RuleFilter::String("li".to_string()),
        replacement: |content, node, options| {
            // Check if this is in an ordered list via data attributes
            let list_type = node.get_attribute("data-list-type");
            let list_index = node.get_attribute("data-list-index");

            if let (Some(list_type), Some(list_index_str)) = (list_type, list_index) {
                if list_type == "OL" {
                    if let Ok(index) = list_index_str.parse::<usize>() {
                        let prefix = format!("{}.  ", index);
                        return format!("{}{}\n", prefix, content.trim_end());
                    }
                }
            }

            // Default to bullet list (bullet + 1 space)
            let prefix = format!("{} ", options.bullet_list_marker);
            format!("{}{}\n", prefix, content.trim_end())
        },
    }
}

fn indented_code_block_rule() -> Rule {
    Rule {
        filter: RuleFilter::Function(|node, options| {
            options.code_block_style == crate::CodeBlockStyle::Indented && node.node_name == "PRE"
        }),
        replacement: |content, _node, _| format!("\n\n{}\n\n", content),
    }
}

fn fenced_code_block_rule() -> Rule {
    Rule {
        filter: RuleFilter::Function(|node, options| {
            options.code_block_style == crate::CodeBlockStyle::Fenced && node.node_name == "PRE"
        }),
        replacement: |content, _node, options| {
            let fence_char = options.fence.chars().next().unwrap_or('`');
            let fence = repeat(fence_char, 3);
            format!("\n\n{}{}\n{}\n{}\n\n", fence, "", content.trim_end(), fence)
        },
    }
}

fn horizontal_rule_rule() -> Rule {
    Rule {
        filter: RuleFilter::String("hr".to_string()),
        replacement: |_, _, options| format!("\n\n{}\n\n", options.hr),
    }
}

fn inline_link_rule() -> Rule {
    Rule {
        filter: RuleFilter::Function(|node, options| {
            options.link_style == crate::LinkStyle::Inlined
                && node.node_name == "A"
                && node.get_attribute("href").is_some()
        }),
        replacement: |content, node, _| {
            let normalized_content = content
                .trim()
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join(" ");

            if normalized_content.starts_with('[') && normalized_content.contains("](") {
                return normalized_content;
            }

            let href = node.get_attribute("href").unwrap_or_default();
            let href_escaped = href.replace("(", "\\(").replace(")", "\\)");
            let title = node.get_attribute("title").unwrap_or_default();
            let title_part = if !title.is_empty() {
                format!(r#" "{}""#, title.replace("\"", "\\\""))
            } else {
                String::new()
            };
            format!("[{}]({}{})", normalized_content, href_escaped, title_part)
        },
    }
}

fn reference_link_rule() -> Rule {
    Rule {
        filter: RuleFilter::Function(|node, options| {
            options.link_style == crate::LinkStyle::Referenced
                && node.node_name == "A"
                && node.get_attribute("href").is_some()
        }),
        replacement: |content, _node, options| match options.link_reference_style {
            crate::LinkReferenceStyle::Collapsed => format!("{}[]", content),
            crate::LinkReferenceStyle::Shortcut => format!("[{}]", content),
            crate::LinkReferenceStyle::Full => format!("[{}][1]", content),
        },
    }
}

fn emphasis_rule() -> Rule {
    Rule {
        filter: RuleFilter::Array(vec!["em".to_string(), "i".to_string()]),
        replacement: |content, _, options| {
            if content.trim().is_empty() {
                String::new()
            } else {
                format!(
                    "{}{}{}",
                    options.em_delimiter, content, options.em_delimiter
                )
            }
        },
    }
}

fn strong_rule() -> Rule {
    Rule {
        filter: RuleFilter::Array(vec!["strong".to_string(), "b".to_string()]),
        replacement: |content, _, options| {
            if content.trim().is_empty() {
                String::new()
            } else {
                format!(
                    "{}{}{}",
                    options.strong_delimiter, content, options.strong_delimiter
                )
            }
        },
    }
}

fn code_rule() -> Rule {
    Rule {
        filter: RuleFilter::Function(|node, _| {
            if node.node_name.to_uppercase() != "CODE" {
                return false;
            }

            node.get_attribute("data-in-pre").is_none()
        }),
        replacement: |content, _, _| {
            if content.is_empty() {
                return String::new();
            }
            let normalized = content.replace("\r\n", " ").replace("\r", " ");

            if normalized.contains('`') {
                format!("`` {} ``", normalized)
            } else {
                format!("`{}`", normalized)
            }
        },
    }
}

fn image_rule() -> Rule {
    Rule {
        filter: RuleFilter::String("img".to_string()),
        replacement: |_, node, options| {
            let alt = node.get_attribute("alt").unwrap_or_default();
            let src = node.get_attribute("src").unwrap_or_default();
            let title = node.get_attribute("title").unwrap_or_default();
            let width = node.get_attribute("width").unwrap_or_default();
            let height = node.get_attribute("height").unwrap_or_default();

            // Always strip 1x1 pixel images without alt text (common tracking pixels)
            if alt.trim().is_empty() && width == "1" && height == "1" {
                return String::new();
            }

            if options.strip_tracking_images
                && is_tracking_image(
                    &src,
                    &alt,
                    options.tracking_image_regex.as_ref(),
                    options.strip_images_without_alt,
                )
            {
                return String::new();
            }

            let title_part = if !title.is_empty() {
                format!(r#" "{}""#, title)
            } else {
                String::new()
            };

            if !src.is_empty() {
                format!("![{}]({}{})", alt, src, title_part)
            } else {
                String::new()
            }
        },
    }
}

fn superscript_rule() -> Rule {
    Rule {
        filter: RuleFilter::String("sup".to_string()),
        replacement: |content, _node, _| {
            let trimmed = content.trim();
            if trimmed.is_empty() {
                "<sup></sup>".to_string()
            } else {
                format!("<sup>{}</sup> ", trimmed)
            }
        },
    }
}

fn subscript_rule() -> Rule {
    Rule {
        filter: RuleFilter::String("sub".to_string()),
        replacement: |content, _node, _| {
            let trimmed = content.trim();
            if trimmed.is_empty() {
                "<sub></sub>".to_string()
            } else {
                format!("<sub>{}</sub> ", trimmed)
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_rules() {
        let rules = get_rules();
        assert!(rules.contains_key("paragraph"));
        assert!(rules.contains_key("heading"));
    }
}
