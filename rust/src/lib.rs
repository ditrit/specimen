pub mod file;
mod flag;
mod nodule;
mod tree;

use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;
use std::time::SystemTime;

use linked_hash_map::LinkedHashMap;

#[derive(Default, PartialEq, Eq)]
pub enum FailStatus {
    #[default]
    Pristine,
    Failed,
    Aborted,
    Panicked,
}

#[derive(Default)]
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
    fn evaluate(&mut self, tile: &Dict) -> bool;
}

pub fn run(mut evaluator: Box<dyn Evaluator>, file_slice: &[file::File]) {
    // Parse the data into a Root, which contains Nodule-s

    // let mut root: nodule::Root = Vec::with_capacity(file_slice.len());
    // let mut document_store = Vec::from_iter((0..file_slice.len()).map(|_| Box::new(Vec::new())));
    // for (f, s) in file_slice.iter().zip(document_store.iter_mut()) {
    //     root.push(nodule::Nodule::from_file(f, s));
    // }

    let mut document_store = Vec::from_iter(file_slice.iter().map(|_| Box::new(Vec::new())));

    let mut root_nodule_vec: Vec<nodule::Nodule> = file_slice
        .iter()
        .zip(document_store.iter_mut())
        .map(|(f, s)| nodule::Nodule::parse_file(f, s))
        .flatten()
        .collect();

    for nodule in root_nodule_vec.iter_mut() {
        let mut data_matrix: LinkedHashMap<Box<str>, Rc<[Box<str>]>> = LinkedHashMap::new();
        data_matrix.insert(
            Box::from("file_path"),
            Rc::new([Box::from((*nodule.file_path).to_owned())]),
        );

        if let Err(e) = nodule.populate(&data_matrix) {
            panic!(
                "Failed to populate nodule data matrix for file {} because: {}",
                nodule.file_path, e
            );
        }
    }

    let root = nodule::Nodule {
        node: &yaml::BAD_VALUE,
        flag: focustree::Flag::None,
        is_leaf: false,
        file_path: Rc::from("".to_owned()),
        data_matrix: LinkedHashMap::new(),
        children: root_nodule_vec.into_boxed_slice(),
    };

    // Retrieving focused nodes, if any. This is done using a suffix tree-traversal: The presence of the FOCUS flag on a node is checked after all its children havec been checked. If a node which has FOCUS-ed children is FOCUS-ed itself, then its FOCUS flag is ignored and a warning is issued.
    let mut selected_leaves = Vec::new();
    focustree::extract_focused_leaf_values(&root, &mut selected_leaves);

    let start_time = std::time::SystemTime::now();

    // Run all the selected leaves
    let mut s = S::default();
    for slab in selected_leaves.into_iter() {
        // Pass the slab data to the testbox
        // - Manage the context (s, test start and test end)
        // - Recover from any panic that might arise during the testbox call

        // Nodule Start
        s.status = FailStatus::Pristine;
        s.fail_info = Vec::new();
        let slab_location = slab.get_location();

        // Nodule Run
        for tile in slab.into_resolved_data_matrix_iterator() {
            run_tile(&mut s, &*tile.borrow(), Box::new(&mut *evaluator))
        }

        // Nodule End
        s.slab_count += 1;
        match s.status {
            FailStatus::Pristine => {
                s.slab_passed += 1;
            }
            FailStatus::Failed => {
                s.slab_failed += 1;
            }
            FailStatus::Aborted => {
                s.slab_aborted += 1;
            }
            FailStatus::Panicked => {
                s.slab_panicked += 1;
            }
        }
        // summarize the failures
        if s.status != FailStatus::Pristine {
            let slab_info = format!("({})", slab_location);

            let info = s.fail_info.join("; ");

            let message = match s.status {
                FailStatus::Failed => "FAIL",
                FailStatus::Aborted => "ABORT",
                FailStatus::Panicked => "PANIC",
                _ => "",
            };
            let message = format!("{}[slab: {}]: {}", message, slab_info, info);

            s.failure_report.push(Box::from(message));
        }
    }

    let duration = SystemTime::now()
        .duration_since(start_time)
        .unwrap_or_else(|_| Duration::default());

    let outcome = if s.failure_report.len() == 0 {
        "SUCCESS"
    } else {
        eprintln!("{}", s.failure_report.join("\n"));
        "FAILURE"
    };

    eprintln!(
        "Ran {} tiles in {}\n \
        {} -- {} Passed | {} Failed | {} Aborted | {} Panicked",
        s.slab_count,
        duration.as_secs(),
        outcome,
        s.slab_passed,
        s.slab_failed,
        s.slab_aborted,
        s.slab_panicked,
    )
}

fn run_tile(s: &mut S, tile: &Dict, evaluator: Box<&mut dyn Evaluator>) {
    match evaluator.evaluate(tile) {
        true => {}
        false => {
            if s.status == FailStatus::Aborted {
                return;
            }
            s.status = FailStatus::Failed;
        }
    }
}
