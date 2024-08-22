use specimen__focustree as focustree;
use specimen__yaml as yaml;

pub fn read_flag(node: &yaml::Yaml) -> focustree::Flag {
    let mut flag = focustree::Flag::default();
    // flag_name is used for printing warning(s) if needed
    let mut flag_name = "";
    let mut both = false;

    let text = match node.data {
        yaml::YamlData::String(ref s) => s,
        _ => panic!("The flag value must be a string."),
    };

    for word in text.split(" ") {
        match word {
            "FOCUS" => {
                if flag == focustree::Flag::Skip {
                    both = true
                }
                flag = focustree::Flag::Focus;
                flag_name = word;
            }
            "PENDING" => {
                if flag == focustree::Flag::Focus {
                    both = true
                }
                flag = focustree::Flag::Skip;
                flag_name = word;
            }
            _ => {
                if word == word.to_uppercase() && word != word.to_lowercase() {
                    eprintln!(
                        "Warning: Unrecognized all uppercase flag \"{}\". It has been ignored.",
                        word
                    )
                }
            }
        };
    }

    if both {
        eprintln!("Warning: Both FOCUS and PENDING flags have been found among the flags of a node. {} has been kept.", flag_name)
    }

    flag
}
