use linked_hash_map::LinkedHashMap;

use crate::file;
use crate::flag;
use crate::Dict;
use std::cell::RefCell;
use std::rc::Rc;
use yaml;

#[derive(Clone, Debug)]
pub struct Nodule<'a> {
    pub node: &'a yaml::Yaml,
    pub flag: focustree::Flag,
    pub is_leaf: bool,
    pub file_path: Rc<str>,
    pub data_matrix: LinkedHashMap<Box<str>, Rc<[Box<str>]>>,
    pub children: Box<[Nodule<'a>]>,
}

impl<'a> Nodule<'a> {
    // Associated functions
    pub fn parse_file(file: &file::File, store: &'a mut Box<Vec<yaml::Yaml>>) -> Vec<Nodule<'a>> {
        let file_path = Rc::from(file.path.to_owned());

        let mut document_vec = yaml::YamlLoader::load_from_str(&file.content)
            .expect(&format!("failed to parse YAML from file {}.", &file.path));

        std::mem::swap(&mut document_vec, store);

        store
            .iter()
            .map(|node| {
                match node.data {
                    yaml::YamlData::Mapping(_) => {}
                    _ => panic!("The root node of the YAML test data file must be a mapping."),
                }

                let mut data_matrix: LinkedHashMap<Box<str>, Rc<[Box<str>]>> = LinkedHashMap::new();
                data_matrix.insert(
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
                            data_matrix: LinkedHashMap::new(),
                        };
                        n.initalize_tree();
                        n
                    })
                    .collect();
            } else {
                panic!(
                    "the value associated with the content keyword must be a sequence of mappings."
                )
            }
        }
    }

    pub fn populate(
        &mut self,
        data_matrix: &LinkedHashMap<Box<str>, Rc<[Box<str>]>>,
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
            if key == "flag" || key == "content" {
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
                .insert(key.to_owned().into_boxed_str(), Rc::from(value_vector));
        }

        for child in self.children.iter_mut() {
            child.populate(&self.data_matrix)?;
        }

        Ok(())
    }

    pub fn into_resolved_data_matrix_iterator(self) -> DataMatrixIterator {
        let length = self.data_matrix.len();

        let reversed_key_array = self
            .data_matrix
            .keys()
            .rev()
            .map(|s| s.to_owned())
            .collect::<Box<[Box<str>]>>();

        let mut total_combinations = 1;
        let mut size_array = Vec::with_capacity(length);
        for key in reversed_key_array.iter() {
            let size = self.data_matrix[key].len();
            total_combinations *= size;
            size_array.push(total_combinations);
        }

        let index_array = vec![0; length].into_boxed_slice();

        let combination = RefCell::new(Dict::new());

        // Initialize the combination
        for key in reversed_key_array.iter() {
            combination
                .borrow_mut()
                .insert((*key).clone(), self.data_matrix[key][0].clone());
        }

        DataMatrixIterator {
            data_matrix: self.data_matrix,
            reversed_key_array: reversed_key_array,
            total_combinations,
            size_array: size_array.into_boxed_slice(),
            index_array,
            combination,
            index: 0,
        }
    }

    fn panic(&self, message: &str) -> ! {
        panic!(
            "{} ({}:{} with data: {:?})",
            message, self.file_path, self.node.position, self.node.data
        )
    }
}

pub struct DataMatrixIterator {
    data_matrix: LinkedHashMap<Box<str>, Rc<[Box<str>]>>,
    reversed_key_array: Box<[Box<str>]>,
    total_combinations: usize,
    size_array: Box<[usize]>,
    /// The index_array tracks the progress of values through every set
    index_array: Box<[usize]>,
    combination: RefCell<Dict>,
    index: usize,
}

impl Iterator for DataMatrixIterator {
    type Item = Dict;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == 0 {
            self.index += 1;
            return Some(self.combination.borrow().clone());
        } else if self.index >= self.total_combinations {
            return None;
        }

        for (k, key) in self.reversed_key_array.iter().enumerate() {
            if self.index % self.size_array[k] == 0 {
                self.index_array[k] += 1;
                self.index_array[k] %= self.size_array[k];
            } else {
                // bump the identified index
                self.index_array[k] += 1;
                self.index_array[k] %= self.size_array[k];

                // update the combination entry corresponding to the identified key
                self.combination.borrow_mut().insert(
                    key.clone(),
                    self.data_matrix[key][self.index_array[k]].clone(),
                );
                break;
            }
        }

        self.index += 1;
        Some(self.combination.borrow().clone())
    }
}
