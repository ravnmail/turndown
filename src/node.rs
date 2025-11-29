use crate::utilities::{is_block, is_meaningful_when_blank, is_void, FlankingWhitespace};
use std::collections::HashMap;

/// Represents different types of DOM nodes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeType {
    Element,
    Text,
    Comment,
    Document,
    ProcessingInstruction,
}

/// Represents an HTML/DOM node with minimal stored state
/// Computed properties are derived from node_name on-demand
#[derive(Clone, Debug)]
pub struct Node {
    pub node_type: NodeType,
    pub node_name: String,
    pub node_value: String,
    pub children: Vec<Node>,
    pub attributes: HashMap<String, String>,
    pub is_code: bool, // Only meaningful state derived from context
}

impl Node {
    /// Creates a new element node
    pub fn new_element(name: &str) -> Self {
        Node {
            node_type: NodeType::Element,
            node_name: name.to_uppercase(),
            node_value: String::new(),
            children: Vec::new(),
            attributes: HashMap::new(),
            is_code: false,
        }
    }

    /// Creates a new text node
    pub fn new_text(value: &str) -> Self {
        Node {
            node_type: NodeType::Text,
            node_name: "#text".to_string(),
            node_value: value.to_string(),
            children: Vec::new(),
            attributes: HashMap::new(),
            is_code: false,
        }
    }

    /// Creates a new document node
    pub fn new_document() -> Self {
        Node {
            node_type: NodeType::Document,
            node_name: "#document".to_string(),
            node_value: String::new(),
            children: Vec::new(),
            attributes: HashMap::new(),
            is_code: false,
        }
    }

    /// Creates a new comment node
    pub fn new_comment(value: &str) -> Self {
        Node {
            node_type: NodeType::Comment,
            node_name: "#comment".to_string(),
            node_value: value.to_string(),
            children: Vec::new(),
            attributes: HashMap::new(),
            is_code: false,
        }
    }

    // Computed property methods (lazy evaluation)

    /// Checks if this element is a block-level element
    pub fn is_block(&self) -> bool {
        is_block(&self.node_name)
    }

    /// Checks if this element is a void (self-closing) element
    pub fn is_void(&self) -> bool {
        is_void(&self.node_name)
    }

    /// Checks if this element is meaningful when blank
    pub fn is_meaningful_when_blank(&self) -> bool {
        is_meaningful_when_blank(&self.node_name)
    }

    /// Calculates if node is blank (empty or only whitespace/void elements)
    pub fn is_blank(&self) -> bool {
        if self.is_meaningful_when_blank() {
            return false;
        }

        // Void elements with meaningful attributes are never blank
        if self.is_void() {
            if self.get_attribute("src").is_some()
                || self.get_attribute("data").is_some()
                || matches!(self.node_name.as_str(), "BR" | "HR")
            {
                return false;
            }
        }

        let has_text_content = self
            .children
            .iter()
            .any(|child| child.node_type == NodeType::Text && !child.node_value.trim().is_empty());

        // Check if there are any void elements with meaningful attributes
        let has_meaningful_void_children = self.children.iter().any(|child| {
            child.node_type == NodeType::Element
                && child.is_void()
                && (child.get_attribute("src").is_some()
                    || child.get_attribute("data").is_some()
                    || matches!(child.node_name.as_str(), "BR" | "HR"))
        });

        // Element is blank if it has no text content AND no meaningful void children
        let has_only_empty_void_children = self.children.iter().all(|child| {
            if child.node_type == NodeType::Element && child.is_void() {
                child.get_attribute("src").is_none()
                    && child.get_attribute("data").is_none()
                    && !matches!(child.node_name.as_str(), "BR" | "HR")
            } else {
                false
            }
        });

        !has_text_content
            && !has_meaningful_void_children
            && (has_only_empty_void_children || self.children.is_empty())
    }

    /// Gets the flanking whitespace (leading/trailing whitespace)
    pub fn flanking_whitespace(&self) -> FlankingWhitespace {
        if self.node_type != NodeType::Element {
            return FlankingWhitespace::new(String::new(), String::new());
        }

        let mut leading = String::new();
        let mut trailing = String::new();

        // Check for leading whitespace in first text child
        if let Some(first_child) = self.children.first() {
            if first_child.node_type == NodeType::Text {
                let text = &first_child.node_value;
                let trimmed = text.trim_start();
                if trimmed.len() < text.len() {
                    leading = text[..(text.len() - trimmed.len())].to_string();
                }
            }
        }

        // Check for trailing whitespace in last text child
        if let Some(last_child) = self.children.last() {
            if last_child.node_type == NodeType::Text {
                let text = &last_child.node_value;
                let trimmed = text.trim_end();
                if trimmed.len() < text.len() {
                    trailing = text[trimmed.len()..].to_string();
                }
            }
        }

        FlankingWhitespace::new(leading, trailing)
    }

    /// Gets an attribute value
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        self.attributes.get(name).cloned()
    }

    /// Sets an attribute value
    pub fn set_attribute(&mut self, name: &str, value: &str) {
        self.attributes.insert(name.to_string(), value.to_string());
    }

    /// Adds a child node
    pub fn add_child(&mut self, child: Node) {
        self.children.push(child);
    }

    /// Checks if this node has any children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Gets the text content recursively
    pub fn text_content(&self) -> String {
        match self.node_type {
            NodeType::Text => self.node_value.clone(),
            NodeType::Element | NodeType::Document => self
                .children
                .iter()
                .map(|c| c.text_content())
                .collect::<Vec<_>>()
                .join(""),
            NodeType::Comment | NodeType::ProcessingInstruction => String::new(),
        }
    }

    /// Converts node to outer HTML representation
    pub fn to_outer_html(&self) -> String {
        match self.node_type {
            NodeType::Element => {
                let mut html = format!("<{}", self.node_name.to_lowercase());
                for (key, value) in &self.attributes {
                    html.push_str(&format!(r#" {}="{}""#, key, value));
                }
                html.push('>');

                for child in &self.children {
                    html.push_str(&child.to_outer_html());
                }

                if !self.is_void() {
                    html.push_str(&format!("</{}>", self.node_name.to_lowercase()));
                }
                html
            }
            NodeType::Text => self.node_value.clone(),
            NodeType::Comment => format!("<!--{}-->", self.node_value),
            NodeType::Document => self.children.iter().map(|c| c.to_outer_html()).collect(),
            NodeType::ProcessingInstruction => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_element() {
        let node = Node::new_element("div");
        assert_eq!(node.node_type, NodeType::Element);
        assert_eq!(node.node_name, "DIV");
        assert!(node.is_block());
    }

    #[test]
    fn test_new_text() {
        let node = Node::new_text("Hello");
        assert_eq!(node.node_type, NodeType::Text);
        assert_eq!(node.node_value, "Hello");
        assert!(!node.is_block());
    }

    #[test]
    fn test_void_element() {
        let node = Node::new_element("br");
        assert!(node.is_void());
    }

    #[test]
    fn test_add_child() {
        let mut parent = Node::new_element("div");
        let child = Node::new_text("child");
        parent.add_child(child);
        assert_eq!(parent.children.len(), 1);
    }

    #[test]
    fn test_get_set_attribute() {
        let mut node = Node::new_element("a");
        node.set_attribute("href", "http://example.com");
        assert_eq!(
            node.get_attribute("href"),
            Some("http://example.com".to_string())
        );
    }

    #[test]
    fn test_text_content() {
        let mut parent = Node::new_element("p");
        parent.add_child(Node::new_text("Hello "));
        parent.add_child(Node::new_text("World"));
        assert_eq!(parent.text_content(), "Hello World");
    }

    #[test]
    fn test_is_blank() {
        let node = Node::new_element("div");
        assert!(node.is_blank());

        let mut node_with_text = Node::new_element("p");
        node_with_text.add_child(Node::new_text("content"));
        assert!(!node_with_text.is_blank());
    }
}
