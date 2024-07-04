use crate::file;
use crate::flag;
use multistringmap::MultiStringMap;
use std::rc::Rc;
use yaml;

#[derive(Clone, Debug)]
pub struct Nodule<'a> {
    pub node: &'a yaml::Yaml,
    pub flag: focustree::Flag,
    pub is_leaf: bool,
    pub file_path: Rc<str>,
    pub data_matrix: MultiStringMap,
    pub children: Box<[Nodule<'a>]>,
}

impl<'a> Nodule<'a> {
    // Associated functions
    pub fn parse_file(file: &file::File, store: &'a mut Box<Vec<yaml::Yaml>>) -> Vec<Nodule<'a>> {
        let file_path = Rc::from(file.path.to_owned());

        let mut document_vec = match yaml::YamlLoader::load_from_str(&file.content) {
            Ok(v) => v,
            Err(e) => {
                println!("{}: {}", file.path, e);
                vec![]
            }
        };

        std::mem::swap(&mut document_vec, store);

        store
            .iter()
            .map(|node| {
                match node.data {
                    yaml::YamlData::Mapping(_) => {}
                    _ => panic!("The root node of the YAML test data file must be a mapping."),
                }

                let mut data_matrix = MultiStringMap::new();
                data_matrix.0.insert(
                    Box::from("file_path"),
                    Rc::new([Box::from(file.path.clone())]),
                );

                let mut n = Nodule {
                    node: &node,
                    flag: focustree::Flag::None,
                    is_leaf: true,
                    file_path: Rc::clone(&file_path),
                    data_matrix,
                    children: Box::new([]),
                };

                n.initalize_tree();

                n
            })
            .collect()
    }

    // Methods
    pub fn get_location(&self) -> Box<str> {
        format!(
            "{}:{}:{}",
            self.file_path, self.node.position.line, self.node.position.column
        )
        .into()
    }

    // The initialization creates all the nodules which correspond to the mapping nodes of the yaml tree, except for the PENDING nodes. It fills the fields `flag`, `has_content_key` and `children`. **It expects YamlNode and FilePath to be already set**, and it sets YamlNode and FilePath for its children.
    fn initalize_tree(&mut self) {
        match self.node.data {
            yaml::YamlData::Mapping(_) => {}
            _ => panic!("the content descendant nodes must be yaml mappings"),
        }

        let flag_node = &self.node.data["flag"];
        if *flag_node != yaml::BAD_VALUE {
            self.flag = flag::read_flag(&flag_node);
        }
        if self.flag == focustree::Flag::Skip {
            return;
        }

        let content_node = &self.node.data["content"];
        if *content_node != yaml::BAD_VALUE {
            self.is_leaf = false;
            if let yaml::YamlData::List(ref yaml_vec) = content_node.data {
                self.children = yaml_vec
                    .iter()
                    .map(|node| {
                        let mut n = Nodule {
                            node,
                            flag: focustree::Flag::None,
                            is_leaf: true,
                            file_path: Rc::clone(&self.file_path),
                            children: Box::new([]),
                            data_matrix: MultiStringMap::new(),
                        };
                        n.initalize_tree();
                        n
                    })
                    .collect();
            } else {
                panic!(
                    "the value associated with the content keyword must be a sequence of mappings"
                )
            }
        }
    }

    pub fn populate(
        &mut self,
        data_matrix: &MultiStringMap,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let data = match self.node.data {
            yaml::YamlData::Mapping(ref m) => m,
            _ => self.panic("the content descendant nodes must be yaml mappings"),
        };

        self.data_matrix = data_matrix.clone();

        for (key, value) in data.iter() {
            let key = match key.data {
                yaml::YamlData::String(ref s) => s,
                _ => self.panic("the keys of the mapping nodes must be strings"),
            };
            if key == "flag" || key == "content" || key == "about" {
                continue;
            }

            let value_vector: Vec<Box<str>> = match value.data {
                yaml::YamlData::String(ref s) => vec![Box::from(s.to_owned())],
                yaml::YamlData::List(ref a) => {
                    if a.len() == 0 {
                        self.panic("when the values of the mapping nodes is a sequence, it must not be empty.")
                    }
                    a.iter().map(|v| match v.data {
                        yaml::YamlData::String(ref s) => Box::from(s.to_owned()),
                        _ => self.panic("when the values of the mapping nodes is a sequence, it must be a sequence of strings only."),
                    }).collect()
                }
                _ => self.panic(&format!(
                    "the values of mapping nodes must be strings or sequences. (key: {:?})",
                    key
                )),
            };

            self.data_matrix
                .0
                .insert(key.to_owned().into_boxed_str(), Rc::from(value_vector));
        }

        for child in self.children.iter_mut() {
            child.populate(&self.data_matrix)?;
        }

        Ok(())
    }

    fn panic(&self, message: &str) -> ! {
        panic!(
            "{} ({}:{} with data: {:?})",
            message, self.file_path, self.node.position, self.node.data
        )
    }
}
