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
    fail_info: Box<str>,
}

pub type Dict = HashMap<Box<str>, Box<str>>;

pub fn run(test_box: &mut dyn FnMut(&Dict) -> Result<(), Box<str>>, file_slice: &[file::File]) {
    // Parse the data into a Root, which contains Nodule-s

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
        let slab_location = slab.get_location();

        for (index, tile) in slab.into_resolved_data_matrix_iterator().enumerate() {
            // Pass the slab data to the testbox
            // - Manage the context (s, test start and test end)
            // - Recover from any panic that might arise during the testbox call

            // Tile Start
            s.status = FailStatus::Pristine;
            s.fail_info = "".into();

            // Tile Run
            match test_box(&tile) {
                Ok(()) => {}
                Err(message) => {
                    if message.starts_with("ABORT") {
                        s.status = FailStatus::Aborted;
                        s.fail_info = message["ABORT".len()..].into();
                    } else {
                        s.status = FailStatus::Failed;
                        s.fail_info = message;
                    }
                }
            }

            // Tile End
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
                let word = match s.status {
                    FailStatus::Failed => "FAIL",
                    FailStatus::Aborted => "ABORT",
                    FailStatus::Panicked => "PANIC",
                    _ => "",
                };
                let message = format!("{word}[slab: {slab_location}][{index}]: {}", s.fail_info);

                s.failure_report.push(Box::from(message));
            }
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
