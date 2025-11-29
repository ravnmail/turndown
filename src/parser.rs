use crate::node::Node;
#[cfg(test)]
use crate::node::NodeType;
use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::default::Default;

/// Parses HTML string into a Node tree using html5ever
pub fn parse_html(html: &str) -> Node {
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .unwrap();

    convert_handle(&dom.document, false, false)
}

/// Converts an html5ever Handle to our Node structure
/// Tracks context: whether we're inside a CODE element and/or PRE block
fn convert_handle(handle: &Handle, in_code: bool, in_pre: bool) -> Node {
    let node = handle.as_ref();

    match &node.data {
        NodeData::Document => {
            let mut doc_node = Node::new_document();
            for child in node.children.borrow().iter() {
                doc_node.add_child(convert_handle(child, false, false));
            }
            doc_node
        }
        NodeData::Element { name, attrs, .. } => {
            let tag_name = name.local.to_string();
            let mut elem = Node::new_element(&tag_name);

            // Copy attributes from html5ever
            for attr in attrs.borrow().iter() {
                elem.set_attribute(&attr.name.local.to_string(), &attr.value.to_string());
            }

            // Update context for children
            let is_pre = tag_name.eq_ignore_ascii_case("PRE") || in_pre;
            let is_code = tag_name.eq_ignore_ascii_case("CODE") && !is_pre;

            // Process children with updated context
            for child in node.children.borrow().iter() {
                elem.add_child(convert_handle(child, is_code || in_code, is_pre));
            }

            elem
        }
        NodeData::Text { contents } => {
            let text = contents.borrow().to_string();
            // Only collapse whitespace if not in code/pre context
            let processed = if in_code || in_pre {
                text
            } else {
                crate::utilities::collapse_whitespace(&text)
            };
            let mut text_node = Node::new_text(&processed);
            text_node.is_code = in_code;
            text_node
        }
        NodeData::Comment { contents } => Node::new_comment(&contents.to_string()),
        NodeData::ProcessingInstruction { .. } | NodeData::Doctype { .. } => Node::new_document(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_html() {
        let html = "<p>Hello</p>";
        let doc = parse_html(html);
        assert!(!doc.children.is_empty());
    }

    #[test]
    fn test_parse_with_attributes() {
        let html = r#"<a href="https://example.com" title="Example">Link</a>"#;
        let doc = parse_html(html);
        assert!(!doc.children.is_empty());
    }

    #[test]
    fn test_parse_nested_elements() {
        let html = "<div><p>Hello <strong>World</strong></p></div>";
        let doc = parse_html(html);
        assert!(!doc.children.is_empty());
    }

    #[test]
    fn test_parse_multiple_elements() {
        let html = "<p>First</p><p>Second</p>";
        let doc = parse_html(html);
        assert!(!doc.children.is_empty());
    }

    #[test]
    fn test_code_element_marking() {
        let html = "<p>Use <code>console.log()</code> function.</p>";
        let doc = parse_html(html);

        fn find_code_text(node: &Node) -> Option<String> {
            for child in &node.children {
                if child.node_name == "CODE" {
                    for subchild in &child.children {
                        if subchild.node_type == NodeType::Text && subchild.is_code {
                            return Some(subchild.node_value.clone());
                        }
                    }
                }
                if let Some(text) = find_code_text(child) {
                    return Some(text);
                }
            }
            None
        }

        let code_text = find_code_text(&doc);
        assert_eq!(code_text, Some("console.log()".to_string()));
    }

    #[test]
    fn test_code_in_pre_not_marked() {
        let html = "<pre><code>function hello() {\n  console.log();\n}</code></pre>";
        let doc = parse_html(html);

        fn find_code_is_marked(node: &Node) -> Option<bool> {
            for child in &node.children {
                if child.node_name == "CODE" {
                    for subchild in &child.children {
                        if subchild.node_type == NodeType::Text {
                            return Some(subchild.is_code);
                        }
                    }
                }
                if let Some(marked) = find_code_is_marked(child) {
                    return Some(marked);
                }
            }
            None
        }

        let is_marked = find_code_is_marked(&doc);
        assert_eq!(is_marked, Some(false));
    }
}
