//! This library is used to go through a user-defined tree and extract value
//! from the leaves of the tree, while supporting the option for the tree
//! to skip or focus certain branches.

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Flag {
    #[default]
    None,
    Skip,
    Focus,
}

pub trait Tree<TValue> {
    /// returns true if the current node is a leaf.
    fn is_leaf(&self) -> bool;
    /// returns the flag of the current node, if any.
    fn get_flag(&self) -> Flag;
    /// returns the children of the current node.
    fn get_children(&self) -> Vec<&dyn Tree<TValue>>;
    /// returns the value of the current node.
    fn get_value(&self) -> TValue;
    /// a function that is called when some unexpected situation occurs
    /// during the traversal of the current node.
    fn warning(&self, message: &str);
}

// This function calls find_focused_nodes. If no node is focused, it
// considers that the root of the tree is focused. It then calls
// extract_leaf_value on all the focused nodes, filling the destination vec.
pub fn extract_focused_leaf_values<TValue>(tree: &dyn Tree<TValue>, destination: &mut Vec<TValue>) {
    let mut focused_node_vec = Vec::new();
    extract_focused_nodes(tree, &mut focused_node_vec);
    if focused_node_vec.is_empty() {
        focused_node_vec.push(tree);
    }
    for focused_node in focused_node_vec {
        get_leaf_values(focused_node, destination);
    }
}

/// This function is used to find the nodes that are focused. If a node
/// has focused descendants and is itself focused, then it is considered
/// not focused and a warning is issued.
fn extract_focused_nodes<'vec, 'node: 'vec, TValue>(
    tree: &'node dyn Tree<TValue>,
    focused_node_vec: &'vec mut Vec<&'node dyn Tree<TValue>>,
) {
    if tree.get_flag() == Flag::Skip {
        return;
    }
    let initial_length = focused_node_vec.len();
    for child in tree.get_children() {
        extract_focused_nodes(child, focused_node_vec);
    }
    if tree.get_flag() == Flag::Focus {
        if focused_node_vec.len() <= initial_length {
            focused_node_vec.push(tree);
        } else {
            tree.warning(
                "A node with focused descendants is itself focused. \
                It has been considered not focused in favor of its \
                focused descendants",
            );
        }
    }
}

/// This function produces the synthesis of the values of the leaves of
/// the node it receives.
fn get_leaf_values<TValue>(tree: &dyn Tree<TValue>, value_vec: &mut Vec<TValue>) {
    if tree.get_flag() == Flag::Skip {
        return;
    }
    if tree.is_leaf() {
        value_vec.push(tree.get_value());
    }
    for child in tree.get_children() {
        get_leaf_values(child, value_vec);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_supports_trees_with_focused_relatives() {
        // This struct is used to represent the tree.
        #[derive(Debug, PartialEq)]
        struct Node {
            value: i32,
            flag: Flag,
            children: Vec<Node>,
        }

        // The implementation of the trait is as follows:
        impl Tree<i32> for Node {
            fn is_leaf(&self) -> bool {
                self.children.is_empty()
            }
            fn get_flag(&self) -> Flag {
                self.flag
            }
            fn get_children(&self) -> Vec<&dyn Tree<i32>> {
                self.children.iter().map(|x| x as &dyn Tree<i32>).collect()
            }
            fn get_value(&self) -> i32 {
                self.value
            }
            fn warning(&self, _message: &str) {}
        }

        // This tree is used to test the library.
        // The tree is as follows:
        //      1
        //     / \
        //    2   3
        //   / \   \
        //  4   5   6
        //     / \
        //    7   8
        //   / \
        //  9   10
        // The leaves are 4, 9, 10, 8 and 6.
        // The focused nodes are 2, 7 and 6.
        // The selected leaves should thus be in order: 9, 10, 6.

        let tree = Node {
            value: 1,
            flag: Flag::None,
            children: vec![
                Node {
                    value: 2,
                    flag: Flag::Focus,
                    children: vec![
                        Node {
                            value: 4,
                            flag: Flag::None,
                            children: vec![],
                        },
                        Node {
                            value: 5,
                            flag: Flag::None,
                            children: vec![
                                Node {
                                    value: 7,
                                    flag: Flag::Focus,
                                    children: vec![
                                        Node {
                                            value: 9,
                                            flag: Flag::None,
                                            children: vec![],
                                        },
                                        Node {
                                            value: 10,
                                            flag: Flag::None,
                                            children: vec![],
                                        },
                                    ],
                                },
                                Node {
                                    value: 8,
                                    flag: Flag::None,
                                    children: vec![],
                                },
                            ],
                        },
                    ],
                },
                Node {
                    value: 3,
                    flag: Flag::None,
                    children: vec![Node {
                        value: 6,
                        flag: Flag::Focus,
                        children: vec![],
                    }],
                },
            ],
        };

        let mut actual_result = Vec::new();
        extract_focused_leaf_values(&tree, &mut actual_result);

        assert_eq!(actual_result, vec![9, 10, 6]);
    }
}
