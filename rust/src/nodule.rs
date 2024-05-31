use linked_hash_map::LinkedHashMap;

use crate::file;
use crate::flag;
use yaml;

pub type Root<'a> = Vec<Nodule<'a>>;

#[derive(Clone, Debug)]
pub struct Nodule<'a> {
    pub node: &'a yaml::Yaml,
    pub flag: focustree::Flag,
    pub has_content_key: bool,
    pub file_path: Box<str>,
    pub children: Box<[Nodule<'a>]>,
    pub data_matrix: LinkedHashMap<Box<str>, Box<[Box<str>]>>,
}

impl<'a> Nodule<'a> {
    // Associated functions
    pub fn from_file(file: &'a file::File) -> Nodule<'a> {
        let document_vec = yaml::YamlLoader::load_from_str(&file.content)
            .expect(&format!("Failed to parse YAML from file {}.", &file.path));

        let node = &document_vec[0];

        match node.data {
            yaml::YamlData::Mapping(_) => {}
            _ => panic!("The root node of the YAML test data file must be a map."),
        }

        let mut n = Nodule {
            node,
            flag: focustree::Flag::None,
            has_content_key: false,
            file_path: file.path.clone(),
            children: Box::new([]),
            data_matrix: LinkedHashMap::new(),
        };

        n.initalize_tree();

        n
    }

    // Methods
    pub fn get_location(&self) -> Box<str> {
        format!(
            "{}:{}:{}",
            self.file_path, self.node.position.line, self.node.position.column
        )
        .into()
    }

    // The initialization creates all the nodules corresponding to the mapping nodes of the yaml tree, except for the PENDING nodes. It fills the fields `flag`, `has_content_key` and `children`. **It expects YamlNode and FilePath to be already set**, and it sets YamlNode and FilePath for its children.
    fn initalize_tree(&mut self) {
        if let yaml::YamlData::Mapping(ref map) = self.node.data {
            for (key, value) in map {
                if let yaml::YamlData::String(ref key_name) = key.data {
                    if key_name == "content" {
                        self.has_content_key = true;
                        if let yaml::YamlData::List(ref yaml_vec) = value.data {
                            self.children = yaml_vec
                                .iter()
                                .map(|node| {
                                    let mut n = Nodule {
                                        node,
                                        flag: focustree::Flag::None,
                                        has_content_key: false,
                                        file_path: self.file_path.clone(),
                                        children: Box::new([]),
                                        data_matrix: LinkedHashMap::new(),
                                    };
                                    n.initalize_tree();
                                    n
                                })
                                .collect();
                        } else {
                            panic!(
                                "The value associated with the content keyword must be a sequence of mappings."
                            )
                        }
                    }
                    if key_name == "flag" {
                        self.flag = flag::read_flag(&value);
                    }
                } else {
                    panic!("using non-string keys is not supported")
                }
            }
        } else {
            panic!("the content descendant nodes must by yaml mappings")
        }
    }
}
