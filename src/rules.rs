use crate::node::Node;
use crate::TurndownOptions;

/// A replacement function for converting HTML to Markdown
pub type ReplacementFn = fn(&str, &Node, &TurndownOptions) -> String;

/// A filter function to match nodes
pub type FilterFn = fn(&Node, &TurndownOptions) -> bool;

/// Represents a conversion rule
#[derive(Clone)]
pub struct Rule {
    pub filter: RuleFilter,
    pub replacement: ReplacementFn,
}

impl std::fmt::Debug for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rule")
            .field("filter", &self.filter)
            .field("replacement", &"<fn>")
            .finish()
    }
}

/// Different types of filters for rules
#[derive(Clone, Debug)]
pub enum RuleFilter {
    String(String),
    Array(Vec<String>),
    Function(FilterFn),
}

impl RuleFilter {
    /// Checks if a node matches this filter
    pub fn matches(&self, node: &Node, options: &TurndownOptions) -> bool {
        match self {
            RuleFilter::String(s) => node.node_name.to_uppercase() == s.to_uppercase(),
            RuleFilter::Array(arr) => {
                let upper = node.node_name.to_uppercase();
                arr.iter().any(|s| s.to_uppercase() == upper)
            }
            RuleFilter::Function(f) => f(node, options),
        }
    }
}

/// Manages a collection of conversion rules
pub struct Rules {
    pub array: Vec<Rule>,
    pub keep: Vec<Rule>,
    pub remove: Vec<Rule>,
    pub options: TurndownOptions,
}

impl Rules {
    /// Creates a new Rules collection with default rules
    pub fn new(options: TurndownOptions) -> Self {
        let mut rules = Rules {
            array: Vec::new(),
            keep: Vec::new(),
            remove: Vec::new(),
            options: options.clone(),
        };

        // Initialize with default rules from options
        for (_, rule) in &options.rules {
            rules.array.push(rule.clone());
        }

        rules
    }

    /// Adds a new rule to the beginning of the rules list
    pub fn add(&mut self, _key: String, rule: Rule) {
        self.array.insert(0, rule);
    }

    /// Marks a filter to keep nodes as HTML
    pub fn keep(&mut self, filter: RuleFilter) {
        self.keep.push(Rule {
            filter,
            replacement: |_, node, _| format!("\n\n{}\n\n", node.to_outer_html()),
        });
    }

    /// Marks a filter to remove nodes
    pub fn remove(&mut self, filter: RuleFilter) {
        self.remove.push(Rule {
            filter,
            replacement: |_, _, _| String::new(),
        });
    }

    /// Gets the appropriate rule for a node
    pub fn for_node(&self, node: &Node) -> Rule {
        // Check if node is blank
        if node.is_blank() {
            return Rule {
                filter: RuleFilter::String("blank".to_string()),
                replacement: |_, node, _| {
                    if node.is_block() {
                        "\n\n".to_string()
                    } else {
                        String::new()
                    }
                },
            };
        }

        // Check regular rules
        if let Some(rule) = self.find_rule(&self.array, node) {
            return rule.clone();
        }

        // Check keep rules
        if let Some(rule) = self.find_rule(&self.keep, node) {
            return rule.clone();
        }

        // Check remove rules
        if let Some(rule) = self.find_rule(&self.remove, node) {
            return rule.clone();
        }

        // Return default rule
        Rule {
            filter: RuleFilter::String("default".to_string()),
            replacement: |content, node, _| {
                if node.is_block() {
                    format!("\n\n{}\n\n", content)
                } else {
                    content.to_string()
                }
            },
        }
    }

    /// Finds a rule that matches a node
    fn find_rule(&self, rules: &[Rule], node: &Node) -> Option<Rule> {
        for rule in rules {
            if rule.filter.matches(node, &self.options) {
                return Some(rule.clone());
            }
        }
        None
    }

    /// Iterates over all rules
    pub fn for_each<F: FnMut(&Rule, usize)>(&self, mut f: F) {
        for (i, rule) in self.array.iter().enumerate() {
            f(rule, i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_filter_string() {
        let filter = RuleFilter::String("div".to_string());
        let node = Node::new_element("div");
        assert!(filter.matches(&node, &TurndownOptions::default()));
    }

    #[test]
    fn test_rule_filter_array() {
        let filter = RuleFilter::Array(vec!["p".to_string(), "div".to_string()]);
        let node = Node::new_element("p");
        assert!(filter.matches(&node, &TurndownOptions::default()));
    }
}
