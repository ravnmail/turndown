pub mod commonmark_rules;
pub mod node;
pub mod parser;
pub mod rules;
pub mod turndown;
pub mod utilities;

pub use node::{Node, NodeType};
pub use rules::{Rule, RuleFilter, Rules};
pub use turndown::{
    CodeBlockStyle, HeadingStyle, LinkReferenceStyle, LinkStyle, Turndown, TurndownOptions,
};
pub use utilities::{
    clean_attribute, is_block, is_meaningful_when_blank, is_tracking_image, is_void, repeat,
    trim_leading_newlines, trim_newlines, trim_trailing_newlines, FlankingWhitespace,
    BLOCK_ELEMENTS, MEANINGFUL_WHEN_BLANK_ELEMENTS, VOID_ELEMENTS,
};
