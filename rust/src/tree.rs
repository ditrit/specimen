// This file implements the focustree::Tree trait for the Nodule struct and the NoduleRoot type.

mod structure;

use structure::*;

impl focustree::Tree<Nodule> for NoduleRoot {
    fn is_leaf(&self) -> bool {
        false
    }

    fn get_flag(&self) -> focustree::Flag {
        focustree::Flag::None
    }

    fn get_children(&self) -> Vec<&dyn focustree::Tree<Nodule>> {
        self
        // self.iter().map(|n| n as &dyn focustree::Tree<Nodule>).collect()
    }

    fn get_value(&self) -> Nodule {
        panic!("NoduleRoot cannot produce a value since it is not a leaf.");
    }

    fn warning(&self, message: &str) {
        println!("Warning: {}", message);
    }
}

impl focustree::Tree<Nodule> for Nodule {
    fn is_leaf(&self) -> bool {
        !self.children.has_content_key
    }

    fn get_flag(&self) -> focustree::Flag {
        self.flag
    }

    fn get_children(&self) -> Vec<&dyn focustree::Tree<Nodule>> {
        self.children
        // self.children.iter().map(|n| n as &dyn focustree::Tree<Nodule>).collect()
    }

    fn get_value(&self) -> Nodule {
        self.clone()
    }

    fn warning(&self, message: &str) {
        println!("Warning: {}", message);
    }
}
