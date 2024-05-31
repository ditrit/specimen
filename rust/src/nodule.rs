use linked_hash_map::LinkedHashMap;

use crate::file;
use crate::flag;
use crate::yaml;

pub type Root = Vec<Nodule>;

#[derive(Clone, Debug, Default)]
pub struct Nodule {
    yaml_node: yaml::Node,
    flag: focustree::Flag,
    has_content_key: bool,
    file_path: Box<str>,
    children: Box<[Nodule]>,
    data_matrix: LinkedHashMap<Box<str>, Box<[Box<str>]>>,
}

impl Nodule {
    // Associated functions
    pub fn from_file(file: &file::File) -> Nodule {
        let yaml_node = yaml::Node::from(file.content);

        if *yaml_node.kind != *"map" {
            panic!("The root node of the YAML test data file must be a map.");
        }
        Nodule {
            yaml_node,
            flag: flag::read_flag(&yaml_node),
            has_content_key: false,
            file_path: file.path.clone(),
            children: Box::new([]),
            data_matrix: LinkedHashMap::new(),
        }
    }

    // Methods
    pub fn get_location(&self) -> Box<str> {
        format!(
            "{}:{}:{}",
            self.file_path, self.yaml_node.line, self.yaml_node.column
        )
        .into()
    }
}
