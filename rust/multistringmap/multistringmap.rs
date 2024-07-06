use linked_hash_map::LinkedHashMap;
use std::collections::HashMap;
use std::rc::Rc;

pub type Dict = HashMap<Box<str>, Box<str>>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MultiStringMap(pub LinkedHashMap<Box<str>, Rc<[Box<str>]>>);

impl MultiStringMap {
    pub fn new() -> Self {
        MultiStringMap(LinkedHashMap::new())
    }
    pub fn into_product_iterator(self) -> MultiStringMapProductIterator {
        let reversed_key_array = self.0.keys().rev().cloned().collect::<Box<[Box<str>]>>();
        let reversed_size_array = reversed_key_array
            .iter()
            .map(|key| self.0[key].len())
            .collect::<Box<[usize]>>();
        let reversed_index_array = vec![0; reversed_size_array.len()].into_boxed_slice();

        // Initialize the combination
        let mut combination = Dict::new();
        for key in reversed_key_array.iter() {
            combination.insert((*key).clone(), self.0[key][0].clone());
        }

        MultiStringMapProductIterator {
            map: self,
            reversed_key_array,
            reversed_size_array,
            reversed_index_array,
            first: true,
            stopped: false,
            combination,
        }
    }
}

pub struct MultiStringMapProductIterator {
    map: MultiStringMap,
    reversed_key_array: Box<[Box<str>]>,
    reversed_size_array: Box<[usize]>,
    /// The index_array tracks the progress of values through every set
    reversed_index_array: Box<[usize]>,
    first: bool,
    stopped: bool,
    combination: Dict,
}

impl MultiStringMapProductIterator {
    pub fn next(&mut self) -> Option<&Dict> {
        if self.stopped {
            return None;
        }
        if self.first {
            self.first = false;
            return Some(&self.combination);
        }
        let mut n = -1i32;
        while n < 0 || self.reversed_index_array[n as usize] == 0 {
            n += 1;
            let m = n as usize;
            if m >= self.reversed_index_array.len() {
                self.stopped = true;
                return None;
            }
            self.reversed_index_array[m] += 1;
            self.reversed_index_array[m] %= self.reversed_size_array[m];
            self.combination.insert(
                self.reversed_key_array[m].clone(),
                self.map.0[&self.reversed_key_array[m]][self.reversed_index_array[m]].clone(),
            );
        }

        Some(&self.combination)
    }
}
