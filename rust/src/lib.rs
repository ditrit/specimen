pub mod file;
mod flag;
mod nodule;
mod yaml;

use std::collections::HashMap;

pub enum FailStatus {
    Pristine,
    Failed,
    Aborted,
    Panicked,
}

struct S {
    slab_count: u32,
    slab_passed: u32,
    slab_failed: u32,
    slab_aborted: u32,
    slab_panicked: u32,
    failure_report: Vec<Box<str>>,

    status: FailStatus,
    fail_info: Vec<Box<str>>,
}

pub type Dict = HashMap<Box<str>, Box<str>>;

pub trait Evaluator {
    fn evaluate(&self, tile: &Dict);
}

pub fn run(evaluator: Box<dyn Evaluator>, file_slice: &[file::File]) {
    let root: nodule::Root = file_slice
        .iter()
        .map(|f| nodule::Nodule::from_file(f))
        .collect();
}
