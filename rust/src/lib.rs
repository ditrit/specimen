pub mod file;
mod flag;
mod nodule;
mod tree;

pub use multistringmap::Dict;
use multistringmap::MultiStringMap;
pub use writable::Writable;

use std::io;
use std::io::Write;
use std::rc::Rc;
use std::time::Duration;
use std::time::SystemTime;

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
    tile_count: u32,
    tile_passed: u32,
    tile_failed: u32,
    tile_aborted: u32,
    tile_panicked: u32,
    failure_report: Vec<Box<str>>,

    status: FailStatus,
    fail_info: Box<str>,
}

pub fn run(
    test_box: &mut dyn FnMut(&Dict) -> Result<(), Box<str>>,
    file_slice: &[file::File],
) -> bool {
    let result = ioless_run(test_box, file_slice, &mut Writable::Out(io::stdout()));
    match result {
        Err(e) => {
            eprintln!("Error: {}", e);
            false
        }
        Ok(b) => b,
    }
}

pub fn ioless_run(
    test_box: &mut dyn FnMut(&Dict) -> Result<(), Box<str>>,
    file_slice: &[file::File],
    stdout: &mut Writable,
) -> io::Result<bool> {
    // Parse the data into a Root, which contains Nodule-s

    let mut document_store = Vec::from_iter(file_slice.iter().map(|_| Box::new(Vec::new())));

    let mut root_nodule_vec: Vec<nodule::Nodule> = file_slice
        .iter()
        .zip(document_store.iter_mut())
        .map(|(f, s)| nodule::Nodule::parse_file(f, s))
        .flatten()
        .collect();

    for nodule in root_nodule_vec.iter_mut() {
        let mut data_matrix = MultiStringMap::new();
        data_matrix.0.insert(
            Box::from("filepath"),
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
        data_matrix: MultiStringMap::new(),
        children: root_nodule_vec.into_boxed_slice(),
    };

    // Retrieving focused nodes, if any. This is done using a suffix tree-traversal: The presence of the FOCUS flag on a node is checked after all its children havec been checked. If a node which has FOCUS-ed children is FOCUS-ed itself, then its FOCUS flag is ignored and a warning is issued.
    let mut selected_leaves = Vec::new();
    let mut flag_stat = focustree::FlagStat::default();
    focustree::extract_focused_leaf_values(&root, &mut selected_leaves, &mut flag_stat, stdout);

    let start_time = SystemTime::now();

    // Run all the selected leaves
    let mut s = S::default();
    for slab in selected_leaves.into_iter() {
        let slab_location = slab.get_location();

        let mut index = 0;
        let mut iterator = slab.data_matrix.into_product_iterator();
        while let Some(tile) = iterator.next() {
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
            s.tile_count += 1;
            match s.status {
                FailStatus::Pristine => {
                    s.tile_passed += 1;
                }
                FailStatus::Failed => {
                    s.tile_failed += 1;
                }
                FailStatus::Aborted => {
                    s.tile_aborted += 1;
                }
                FailStatus::Panicked => {
                    s.tile_panicked += 1;
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
                let message = format!("{word}[{slab_location}][{index}]: {}", s.fail_info);

                s.failure_report.push(Box::from(message));
            }

            index += 1;
        }
    }

    let duration = SystemTime::now()
        .duration_since(start_time)
        .unwrap_or_else(|_| Duration::default());

    let outcome = if s.failure_report.len() == 0 {
        "SUCCESS"
    } else {
        writeln!(stdout, "{}", s.failure_report.join("\n"))?;
        "FAILURE"
    };

    if flag_stat.focus_count > 0 || flag_stat.skip_count > 0 {
        let mut message_vec = vec![];
        if flag_stat.focus_count > 0 {
            message_vec.push(format!("{} focused node(s)", flag_stat.focus_count));
        }
        if flag_stat.skip_count > 0 {
            message_vec.push(format!("{} pending node(s)", flag_stat.skip_count));
        }
        writeln!(stdout, "Encountered {}", message_vec.join(" and "))?;
    }

    writeln!(
        stdout,
        "Ran {} tiles in {}ms\n\
        {} -- {} Passed | {} Failed | {} Aborted | {} Panicked",
        s.tile_count,
        duration.as_millis(),
        outcome,
        s.tile_passed,
        s.tile_failed,
        s.tile_aborted,
        s.tile_panicked,
    )?;

    Ok(outcome == "SUCCESS")
}
