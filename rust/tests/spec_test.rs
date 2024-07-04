use regex::Regex;
use specimen::Writable;
use std::collections::HashMap;

fn indent(text: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    return text
        .lines()
        .map(|line| format!("{}{}", indent, line))
        .collect::<Vec<String>>()
        .join("\n");
}

fn to_hashmap(data: &yaml_rust::Yaml) -> specimen::Dict {
    return match data {
        yaml_rust::Yaml::Hash(key_value_vec) => {
            let mut hashmap = HashMap::new();
            for (yaml_key, yaml_value) in key_value_vec {
                let key = match yaml_key {
                    yaml_rust::Yaml::String(key) => key,
                    _ => panic!("Expected the key to be a YAML string"),
                };
                let value = match yaml_value {
                    yaml_rust::Yaml::String(value) => value,
                    _ => panic!("Expected the value to be a YAML string"),
                };
                hashmap.insert(key.clone().into(), value.clone().into());
            }
            hashmap
        }
        _ => panic!("Expected a YAML mapping"),
    };
}

fn call_logger(tile: &specimen::Dict) -> Result<(), Box<str>> {
    let call_string = &tile["calls"];
    let call_yaml = &yaml_rust::YamlLoader::load_from_str(call_string).unwrap()[0];
    let call_vec = match call_yaml {
        yaml_rust::Yaml::Array(vec) => vec,
        _ => panic!("Expected a YAML array"),
    };
    let mut index = 0;
    let mut error = "".to_string();
    let result = specimen::ioless_run(
        &mut |spec_tile: &specimen::Dict| -> Result<(), Box<str>> {
            let mut expected_tile = to_hashmap(&call_vec[index]);
            expected_tile.insert("filepath".into(), spec_tile["filepath"].clone());

            if error.len() == 0 && expected_tile != *spec_tile {
                error = format!(
                    "[Call {}]\nExpected: {:?}\nActual__: {:?}",
                    index, expected_tile, spec_tile,
                );
            }
            index += 1;
            Ok(())
        },
        &[specimen::file::File {
            path: tile["filepath"].clone(),
            content: tile["spec"].clone(),
        }],
        &mut Writable::Vec(Vec::new()),
    );

    if let Err(err) = result {
        return Err(err.to_string().into());
    }

    if index != call_vec.len() {
        return Err(format!("Expected {} calls, but got {}", call_vec.len(), index).into());
    }

    return if error != "" {
        return Err(error.into());
    } else {
        Ok(())
    };
}

fn report(tile: &specimen::Dict) -> Result<(), Box<str>> {
    let behavior_yaml = &yaml_rust::YamlLoader::load_from_str(&tile["behavior"]).unwrap()[0];
    let behavior = to_hashmap(behavior_yaml);
    let mut writable_buffer = Writable::Vec(Vec::new());

    let result = specimen::ioless_run(
        &mut |spec_tile: &specimen::Dict| -> Result<(), Box<str>> {
            let outcome = &behavior[&spec_tile["letter"]];
            return match &**outcome {
                "pass" => Ok(()),
                "fail" => Err("failure".into()),
                "abort" => Err("ABORTaborted".into()),
                other => Err(format!("ABORT: unknown outcome: {}", other).into()),
            };
        },
        &[specimen::file::File {
            path: tile["filepath"].clone(),
            content: tile["spec"].clone(),
        }],
        &mut writable_buffer,
    );

    if let Err(error) = result {
        return Err(error.to_string().into());
    }

    let buffer = match writable_buffer {
        Writable::Vec(vec) => vec,
        _ => panic!("Expected a Vec"),
    };
    let output = String::from_utf8(buffer).unwrap();
    let report_regex = Regex::new(&tile["report"]).unwrap();
    if !report_regex.is_match(&output) {
        Err(format!(
            "Expected:\n{}\nActual:\n{}",
            indent(&*tile["report"], 4),
            indent(&*output, 4)
        )
        .into())
    } else {
        Ok(())
    }
}

#[test]
fn test_spec() {
    let test_passed = specimen::run(
        &mut |tile: &specimen::Dict| -> Result<(), Box<str>> {
            return match tile.get("box") {
                Some(box_name) if **box_name == *"call-logger" => call_logger(tile),
                Some(box_name) if **box_name == *"report" => report(tile),
                Some(box_name) => Err(format!("Unknown box: {}", box_name).into()),
                None => Err("No box specified".into()),
            };
        },
        &[
            specimen::file::File::read_local_file("../spec/about.yaml"),
            specimen::file::File::read_local_file("../spec/flag.yaml"),
            specimen::file::File::read_local_file("../spec/matrix.yaml"),
            specimen::file::File::read_local_file("../spec/report.yaml"),
        ],
    );

    assert!(test_passed);
}
