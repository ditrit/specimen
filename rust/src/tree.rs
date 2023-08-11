use crate::nodule::Nodule;

// This file implements the focustree::Tree trait for the Nodule struct and the NoduleRoot type.

impl<'a> focustree::Tree<Nodule<'a>> for Nodule<'a> {
    fn is_leaf(&self) -> bool {
        self.is_leaf
    }

    fn get_flag(&self) -> focustree::Flag {
        self.flag
    }

    fn get_children(&self) -> Vec<&dyn focustree::Tree<Nodule<'a>>> {
        self.children
            .iter()
            .map(|n| n as &dyn focustree::Tree<Nodule>)
            .collect()
    }

    fn get_value(&self) -> Nodule<'a> {
        self.clone()
    }

    fn warning(&self, message: &str) {
        println!("Warning: {}", message);
    }
}
