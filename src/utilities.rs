/// Collapses whitespace according to HTML rules (adapted from the DOM-based JavaScript implementation)
/// - Replaces sequences of spaces, tabs, newlines, and carriage returns with a single space
/// - Preserves blank lines (double newlines with optional whitespace between)
/// - Mimics the behavior: `/[ \r\n\t]+/g` → ` `
pub fn collapse_whitespace(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        if ch == '\n' {
            // Look ahead to detect blank lines: \n followed by optional whitespace then another \n
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() && chars[j] != '\n' {
                j += 1;
            }

            if j < chars.len() && chars[j] == '\n' {
                // Blank line detected - preserve both newlines
                result.push('\n');
                result.push('\n');
                i = j + 1;
            } else {
                // Single newline - collapse to space (following JavaScript behavior)
                if !result.ends_with(' ') && !result.is_empty() {
                    result.push(' ');
                }
                i += 1;
                // Skip any following whitespace except newlines
                while i < chars.len() && chars[i].is_whitespace() && chars[i] != '\n' {
                    i += 1;
                }
            }
        } else if ch.is_whitespace() {
            // Space, tab, or carriage return
            if !result.ends_with(' ') && !result.ends_with('\n') {
                result.push(' ');
            }
            i += 1;
            // Skip following whitespace
            while i < chars.len() && chars[i].is_whitespace() && chars[i] != '\n' {
                i += 1;
            }
        } else {
            result.push(ch);
            i += 1;
        }
    }

    if result.trim().is_empty() {
        String::new()
    } else {
        result
    }
}

/// Trims leading newlines from a string
pub fn trim_leading_newlines(s: &str) -> &str {
    s.trim_start_matches('\n')
}

/// Trims trailing newlines from a string
pub fn trim_trailing_newlines(s: &str) -> &str {
    let mut end = s.len();
    while end > 0 && s.as_bytes()[end - 1] == b'\n' {
        end -= 1;
    }
    &s[..end]
}

/// Trims both leading and trailing newlines
pub fn trim_newlines(s: &str) -> &str {
    trim_trailing_newlines(trim_leading_newlines(s))
}

/// Repeats a character n times
pub fn repeat(ch: char, count: usize) -> String {
    (0..count).map(|_| ch).collect()
}

/// Cleans an HTML attribute value
pub fn clean_attribute(attribute: Option<&str>) -> String {
    match attribute {
        Some(attr) => {
            // Replace multiple newlines and whitespace
            attr.replace("\n", " ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        }
        None => String::new(),
    }
}

/// List of block-level HTML elements
pub const BLOCK_ELEMENTS: &[&str] = &[
    "ADDRESS",
    "ARTICLE",
    "ASIDE",
    "AUDIO",
    "BLOCKQUOTE",
    "BODY",
    "CANVAS",
    "CENTER",
    "DD",
    "DIR",
    "DIV",
    "DL",
    "DT",
    "FIELDSET",
    "FIGCAPTION",
    "FIGURE",
    "FOOTER",
    "FORM",
    "FRAMESET",
    "H1",
    "H2",
    "H3",
    "H4",
    "H5",
    "H6",
    "HEADER",
    "HGROUP",
    "HR",
    "HTML",
    "ISINDEX",
    "LI",
    "MAIN",
    "MENU",
    "NAV",
    "NOFRAMES",
    "NOSCRIPT",
    "OL",
    "OUTPUT",
    "P",
    "PRE",
    "SECTION",
    "TABLE",
    "TBODY",
    "TD",
    "TFOOT",
    "TH",
    "THEAD",
    "TR",
    "UL",
];

/// List of void (self-closing) HTML elements
pub const VOID_ELEMENTS: &[&str] = &[
    "AREA", "BASE", "BR", "COL", "COMMAND", "EMBED", "HR", "IMG", "INPUT", "KEYGEN", "LINK",
    "META", "PARAM", "SOURCE", "TRACK", "WBR",
];

/// List of elements that are meaningful when blank
pub const MEANINGFUL_WHEN_BLANK_ELEMENTS: &[&str] = &[
    "A", "TABLE", "THEAD", "TBODY", "TFOOT", "TH", "TD", "IFRAME", "SCRIPT", "AUDIO", "VIDEO",
];

/// Checks if a node name is a block element
pub fn is_block(tag_name: &str) -> bool {
    is_in_list(tag_name, BLOCK_ELEMENTS)
}

/// Checks if a node name is a void element
pub fn is_void(tag_name: &str) -> bool {
    is_in_list(tag_name, VOID_ELEMENTS)
}

/// Checks if a node name is meaningful when blank
pub fn is_meaningful_when_blank(tag_name: &str) -> bool {
    is_in_list(tag_name, MEANINGFUL_WHEN_BLANK_ELEMENTS)
}

/// Helper function to check if a string is in a list
fn is_in_list(s: &str, list: &[&str]) -> bool {
    list.iter().any(|&item| item.eq_ignore_ascii_case(s))
}

/// Whitespace information for flanking
#[derive(Clone, Debug)]
pub struct FlankingWhitespace {
    pub leading: String,
    pub trailing: String,
}

impl FlankingWhitespace {
    pub fn new(leading: String, trailing: String) -> Self {
        FlankingWhitespace { leading, trailing }
    }
}

/// Checks if an image is likely a tracking pixel based on URL and attributes
pub fn is_tracking_image(
    src: &str,
    alt: &str,
    tracking_regex: Option<&regex::Regex>,
    strip_without_alt: bool,
) -> bool {
    // If stripping images without alt tags is enabled and alt is empty
    if strip_without_alt && alt.trim().is_empty() {
        return true;
    }

    // Check against tracking regex if provided
    if let Some(regex) = tracking_regex {
        if regex.is_match(src) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_leading_newlines() {
        assert_eq!(trim_leading_newlines("\n\ntest"), "test");
        assert_eq!(trim_leading_newlines("test"), "test");
    }

    #[test]
    fn test_trim_trailing_newlines() {
        assert_eq!(trim_trailing_newlines("test\n\n"), "test");
        assert_eq!(trim_trailing_newlines("test"), "test");
    }

    #[test]
    fn test_repeat() {
        assert_eq!(repeat('#', 3), "###");
        assert_eq!(repeat('=', 2), "==");
    }

    #[test]
    fn test_is_block() {
        assert!(is_block("div"));
        assert!(is_block("DIV"));
        assert!(!is_block("span"));
    }

    #[test]
    fn test_is_void() {
        assert!(is_void("br"));
        assert!(is_void("BR"));
        assert!(!is_void("div"));
    }

    #[test]
    fn test_clean_attribute() {
        assert_eq!(clean_attribute(Some("  hello  world  ")), "hello world");
        assert_eq!(clean_attribute(None), "");
    }

    #[test]
    fn test_collapse_whitespace_simple() {
        // Simple space should be preserved
        assert_eq!(collapse_whitespace(" and "), " and ");
    }

    #[test]
    fn test_collapse_whitespace_newline() {
        // Newline collapses to space
        assert_eq!(collapse_whitespace("text\nmore"), "text more");
    }

    #[test]
    fn test_collapse_whitespace_with_spaces() {
        // Multiple spaces and newlines collapse to single space
        assert_eq!(collapse_whitespace("text  \n  more"), "text more");
    }

    #[test]
    fn test_collapse_whitespace_double_newline() {
        // Blank lines (double newlines) are preserved
        assert_eq!(collapse_whitespace("para1\n\npara2"), "para1\n\npara2");
    }

    #[test]
    fn test_collapse_whitespace_leading_trailing() {
        // Leading/trailing spaces should be preserved for inline spacing
        assert_eq!(collapse_whitespace("  text  "), " text ");
    }
}
