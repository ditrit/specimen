/// Note:
/// This file was originally copy-pasted from yaml-rust
use std::collections::BTreeMap;
use std::f64;
use std::i64;
use std::mem;
use std::ops::Index;
use std::string;
use std::vec;
use yaml_rust::parser::*;
use yaml_rust::scanner::{Marker, ScanError, TScalarStyle, TokenType};

/// A YAML node is stored as this `Yaml` enumeration, which provides an easy way to
/// access your YAML document.
///
/// # Examples
///
/// ```
/// use yaml_rust::Yaml;
/// let foo = Yaml::from_str("-123"); // convert the string to the appropriate YAML type
/// assert_eq!(foo.as_i64().unwrap(), -123);
///
/// // iterate over an Array
/// let vec = Yaml::Array(vec![Yaml::Integer(1), Yaml::Integer(2)]);
/// for v in vec.as_vec().unwrap() {
///     assert!(v.as_i64().is_some());
/// }
/// ```
#[derive(Clone, Default, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub enum YamlData {
    /// Float types are stored as String and parsed on demand.
    /// Note that f64 does NOT implement Eq trait and can NOT be stored in BTreeMap.
    Real(string::String),
    /// YAML int is stored as i64.
    Integer(i64),
    /// YAML scalar.
    String(string::String),
    /// YAML bool, e.g. `true` or `false`.
    Boolean(bool),
    /// YAML sequence, can be accessed as a `Vec`.
    List(self::List),
    /// YAML map, can be accessed as a `Vec` of key-value pairs.
    ///
    /// Insertion order will match the order of insertion into the map.
    Mapping(self::Mapping),
    /// Alias, not fully supported yet.
    Alias(usize),
    /// YAML null, e.g. `null` or `~`.
    Null,
    /// Accessing a nonexistent node via the Index trait returns `BadValue`. This
    /// simplifies error handling in the calling code. Invalid type conversion also
    /// returns `BadValue`.
    #[default]
    BadValue,
}

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Default, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub struct Yaml {
    pub data: YamlData,
    pub position: Position,
}

pub type List = Vec<Yaml>;
pub type Mapping = Vec<(Yaml, Yaml)>;

impl Yaml {
    fn new(data: YamlData, position: Position) -> Yaml {
        Yaml { data, position }
    }
}

static BAD_VALUE: Yaml = Yaml {
    data: YamlData::BadValue,
    position: Position { line: 0, column: 0 },
};

// parse f64 as Core schema
// See: https://github.com/chyh1990/yaml-rust/issues/51
fn parse_f64(v: &str) -> Option<f64> {
    match v {
        ".inf" | ".Inf" | ".INF" | "+.inf" | "+.Inf" | "+.INF" => Some(f64::INFINITY),
        "-.inf" | "-.Inf" | "-.INF" => Some(f64::NEG_INFINITY),
        ".nan" | "NaN" | ".NAN" => Some(f64::NAN),
        _ => v.parse::<f64>().ok(),
    }
}

pub struct YamlLoader {
    docs: Vec<Yaml>,
    // states
    // (current node, anchor_id) tuple
    doc_stack: Vec<(Yaml, usize)>,
    key_stack: Vec<Yaml>,
    anchor_map: BTreeMap<usize, Yaml>,
}

impl MarkedEventReceiver for YamlLoader {
    fn on_event(&mut self, ev: Event, marker: Marker) {
        let position = Position {
            line: marker.line(),
            column: marker.col(),
        };
        // println!("EV {:?}", ev);
        match ev {
            Event::DocumentStart => {
                // do nothing
            }
            Event::DocumentEnd => {
                match self.doc_stack.len() {
                    // empty document
                    0 => self.docs.push(Yaml::new(YamlData::BadValue, position)),
                    1 => self.docs.push(self.doc_stack.pop().unwrap().0),
                    _ => unreachable!(),
                }
            }
            Event::SequenceStart(aid) => {
                self.doc_stack
                    .push((Yaml::new(YamlData::List(Vec::new()), position), aid));
            }
            Event::SequenceEnd => {
                let node = self.doc_stack.pop().unwrap();
                self.insert_new_node(node);
            }
            Event::MappingStart(aid) => {
                self.doc_stack
                    .push((Yaml::new(YamlData::Mapping(Mapping::new()), position), aid));
                self.key_stack.push(BAD_VALUE.clone());
            }
            Event::MappingEnd => {
                self.key_stack.pop().unwrap();
                let node = self.doc_stack.pop().unwrap();
                self.insert_new_node(node);
            }
            Event::Scalar(v, style, aid, tag) => {
                let yaml_data = if style != TScalarStyle::Plain {
                    YamlData::String(v)
                } else if let Some(TokenType::Tag(ref handle, ref suffix)) = tag {
                    // XXX tag:yaml.org,2002:
                    if handle == "!!" {
                        match suffix.as_ref() {
                            "bool" => {
                                // "true" or "false"
                                match v.parse::<bool>() {
                                    Err(_) => YamlData::BadValue,
                                    Ok(v) => YamlData::Boolean(v),
                                }
                            }
                            "int" => match v.parse::<i64>() {
                                Err(_) => YamlData::BadValue,
                                Ok(v) => YamlData::Integer(v),
                            },
                            "float" => match parse_f64(&v) {
                                Some(_) => YamlData::Real(v),
                                None => YamlData::BadValue,
                            },
                            "null" => match v.as_ref() {
                                "~" | "null" => YamlData::Null,
                                _ => YamlData::BadValue,
                            },
                            _ => YamlData::String(v),
                        }
                    } else {
                        YamlData::String(v)
                    }
                } else {
                    // Datatype is not specified, or unrecognized
                    YamlData::from_str(&v)
                };

                self.insert_new_node((Yaml::new(yaml_data, position), aid));
            }
            Event::Alias(id) => {
                let n = match self.anchor_map.get(&id) {
                    Some(v) => v.clone(),
                    None => BAD_VALUE.clone(),
                };
                self.insert_new_node((n, 0));
            }
            _ => { /* ignore */ }
        }
        // println!("DOC {:?}", self.doc_stack);
    }
}

impl YamlLoader {
    fn insert_new_node(&mut self, node: (Yaml, usize)) {
        // valid anchor id starts from 1
        if node.1 > 0 {
            self.anchor_map.insert(node.1, node.0.clone());
        }
        if self.doc_stack.is_empty() {
            self.doc_stack.push(node);
        } else {
            let parent = self.doc_stack.last_mut().unwrap();

            match parent.0.data {
                YamlData::List(ref mut v) => v.push(node.0),
                YamlData::Mapping(ref mut h) => {
                    let cur_key = self.key_stack.last_mut().unwrap();
                    // current node is a key
                    if cur_key.data.is_badvalue() {
                        *cur_key = node.0;
                    // current node is a value
                    } else {
                        let mut newkey = BAD_VALUE.clone();
                        mem::swap(&mut newkey, cur_key);
                        h.push((newkey, node.0));
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    pub fn load_from_str(source: &str) -> Result<Vec<Yaml>, ScanError> {
        let mut loader = YamlLoader {
            docs: Vec::new(),
            doc_stack: Vec::new(),
            key_stack: Vec::new(),
            anchor_map: BTreeMap::new(),
        };
        let mut parser = Parser::new(source.chars());
        parser.load(&mut loader, true)?;
        Ok(loader.docs)
    }
}

macro_rules! define_as (
    ($name:ident, $t:ident, $yt:ident) => (
pub fn $name(&self) -> Option<$t> {
    match *self {
        YamlData::$yt(v) => Some(v),
        _ => None
    }
}
    );
);

macro_rules! define_as_ref (
    ($name:ident, $t:ty, $yt:ident) => (
pub fn $name(&self) -> Option<$t> {
    match *self {
        YamlData::$yt(ref v) => Some(v),
        _ => None
    }
}
    );
);

macro_rules! define_into (
    ($name:ident, $t:ty, $yt:ident) => (
pub fn $name(self) -> Option<$t> {
    match self {
        YamlData::$yt(v) => Some(v),
        _ => None
    }
}
    );
);

impl YamlData {
    define_as!(as_bool, bool, Boolean);
    define_as!(as_i64, i64, Integer);

    define_as_ref!(as_str, &str, String);
    define_as_ref!(as_vec, &List, List);

    define_into!(into_bool, bool, Boolean);
    define_into!(into_i64, i64, Integer);
    define_into!(into_string, String, String);
    define_into!(into_vec, List, List);

    pub fn is_null(&self) -> bool {
        match *self {
            YamlData::Null => true,
            _ => false,
        }
    }

    pub fn is_badvalue(&self) -> bool {
        match *self {
            YamlData::BadValue => true,
            _ => false,
        }
    }

    pub fn is_array(&self) -> bool {
        match *self {
            YamlData::List(_) => true,
            _ => false,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            YamlData::Real(ref v) => parse_f64(v),
            _ => None,
        }
    }

    pub fn into_f64(self) -> Option<f64> {
        match self {
            YamlData::Real(ref v) => parse_f64(v),
            _ => None,
        }
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(should_implement_trait))]
impl YamlData {
    // Not implementing FromStr because there is no possibility of Error.
    // This function falls back to Yaml::String if nothing else matches.
    pub fn from_str(v: &str) -> YamlData {
        if v.starts_with("0x") {
            if let Ok(i) = i64::from_str_radix(&v[2..], 16) {
                return YamlData::Integer(i);
            }
        }
        if v.starts_with("0o") {
            if let Ok(i) = i64::from_str_radix(&v[2..], 8) {
                return YamlData::Integer(i);
            }
        }
        if v.starts_with('+') {
            if let Ok(i) = v[1..].parse::<i64>() {
                return YamlData::Integer(i);
            }
        }
        match v {
            "~" | "null" => YamlData::Null,
            "true" => YamlData::Boolean(true),
            "false" => YamlData::Boolean(false),
            _ if v.parse::<i64>().is_ok() => YamlData::Integer(v.parse::<i64>().unwrap()),
            // try parsing as f64
            _ if parse_f64(v).is_some() => YamlData::Real(v.to_owned()),
            _ => YamlData::String(v.to_owned()),
        }
    }
}

impl<'a> Index<&'a str> for YamlData {
    type Output = Yaml;

    fn index(&self, idx: &'a str) -> &Yaml {
        let key = YamlData::String(idx.to_owned());

        match *self {
            YamlData::Mapping(ref v) => v
                .iter()
                .find(|e| e.0.data == key)
                .map(|e| &e.1)
                .unwrap_or(&BAD_VALUE),
            _ => &BAD_VALUE,
        }
    }
}

impl Index<usize> for YamlData {
    type Output = Yaml;

    fn index(&self, idx: usize) -> &Yaml {
        if let YamlData::List(ref v) = *self {
            v.get(idx).unwrap_or(&BAD_VALUE)
        } else if let YamlData::Mapping(ref v) = *self {
            let key = YamlData::Integer(idx as i64);
            v.iter()
                .find(|e| e.0.data == key)
                .map(|e| &e.1)
                .unwrap_or(&BAD_VALUE)
        } else {
            &BAD_VALUE
        }
    }
}

impl IntoIterator for YamlData {
    type Item = Yaml;
    type IntoIter = YamlIter;

    fn into_iter(self) -> Self::IntoIter {
        YamlIter {
            yaml: self.into_vec().unwrap_or_else(Vec::new).into_iter(),
        }
    }
}

pub struct YamlIter {
    yaml: vec::IntoIter<Yaml>,
}

impl Iterator for YamlIter {
    type Item = Yaml;

    fn next(&mut self) -> Option<Yaml> {
        self.yaml.next()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::f64;
    #[test]
    fn test_coerce() {
        let s = "---
a: 1
b: 2.2
c: [1, 2]
";
        let out = YamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];
        assert_eq!(doc.data["a"].data.as_i64().unwrap(), 1i64);
        assert_eq!(doc.data["b"].data.as_f64().unwrap(), 2.2f64);
        assert_eq!(doc.data["c"].data[1].data.as_i64().unwrap(), 2i64);
        assert!(doc.data["d"].data[0].data.is_badvalue());
    }

    #[test]
    fn test_empty_doc() {
        let s: String = "".to_owned();
        YamlLoader::load_from_str(&s).unwrap();
        let s: String = "---".to_owned();
        assert_eq!(
            YamlLoader::load_from_str(&s).unwrap()[0].data,
            YamlData::Null
        );
    }

    #[test]
    fn test_parser() {
        let s: String = "
# comment
a0 bb: val
a1:
    b1: 4
    b2: d
a2: 4 # i'm comment
a3: [1, 2, 3]
a4:
    - - a1
      - a2
    - 2
a5: 'single_quoted'
a6: \"double_quoted\"
a7: 你好
"
        .to_owned();
        let out = YamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];
        assert_eq!(doc.data["a7"].data.as_str().unwrap(), "你好");
    }

    #[test]
    fn test_multi_doc() {
        let s = "
'a scalar'
---
'a scalar'
---
'a scalar'
";
        let out = YamlLoader::load_from_str(&s).unwrap();
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn test_anchor() {
        let s = "
a1: &DEFAULT
    b1: 4
    b2: d
a2: *DEFAULT
";
        let out = YamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];
        assert_eq!(doc.data["a2"].data["b1"].data.as_i64().unwrap(), 4);
    }

    #[test]
    fn test_bad_anchor() {
        let s = "
a1: &DEFAULT
    b1: 4
    b2: *DEFAULT
";
        let out = YamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];
        assert_eq!(doc.data["a1"].data["b2"].data, YamlData::BadValue);
    }

    #[test]
    fn test_github_27() {
        // https://github.com/chyh1990/yaml-rust/issues/27
        let s = "&a";
        let out = YamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];
        assert_eq!(doc.data.as_str().unwrap(), "");
    }

    #[test]
    fn test_plain_datatype() {
        let s = "
- 'string'
- \"string\"
- string
- 123
- -321
- 1.23
- -1e4
- ~
- null
- true
- false
- !!str 0
- !!int 100
- !!float 2
- !!null ~
- !!bool true
- !!bool false
- 0xFF
# bad values
- !!int string
- !!float string
- !!bool null
- !!null val
- 0o77
- [ 0xF, 0xF ]
- +12345
- [ true, false ]
";
        let out = YamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];

        assert_eq!(doc.data[0].data.as_str().unwrap(), "string");
        assert_eq!(doc.data[1].data.as_str().unwrap(), "string");
        assert_eq!(doc.data[2].data.as_str().unwrap(), "string");
        assert_eq!(doc.data[3].data.as_i64().unwrap(), 123);
        assert_eq!(doc.data[4].data.as_i64().unwrap(), -321);
        assert_eq!(doc.data[5].data.as_f64().unwrap(), 1.23);
        assert_eq!(doc.data[6].data.as_f64().unwrap(), -1e4);
        assert!(doc.data[7].data.is_null());
        assert!(doc.data[8].data.is_null());
        assert_eq!(doc.data[9].data.as_bool().unwrap(), true);
        assert_eq!(doc.data[10].data.as_bool().unwrap(), false);
        assert_eq!(doc.data[11].data.as_str().unwrap(), "0");
        assert_eq!(doc.data[12].data.as_i64().unwrap(), 100);
        assert_eq!(doc.data[13].data.as_f64().unwrap(), 2.0);
        assert!(doc.data[14].data.is_null());
        assert_eq!(doc.data[15].data.as_bool().unwrap(), true);
        assert_eq!(doc.data[16].data.as_bool().unwrap(), false);
        assert_eq!(doc.data[17].data.as_i64().unwrap(), 255);
        assert!(doc.data[18].data.is_badvalue());
        assert!(doc.data[19].data.is_badvalue());
        assert!(doc.data[20].data.is_badvalue());
        assert!(doc.data[21].data.is_badvalue());
        assert_eq!(doc.data[22].data.as_i64().unwrap(), 63);
        assert_eq!(doc.data[23].data[0].data.as_i64().unwrap(), 15);
        assert_eq!(doc.data[23].data[1].data.as_i64().unwrap(), 15);
        assert_eq!(doc.data[24].data.as_i64().unwrap(), 12345);
        assert!(doc.data[25].data[0].data.as_bool().unwrap());
        assert!(!doc.data[25].data[1].data.as_bool().unwrap());
    }

    #[test]
    fn test_bad_hyphen() {
        // See: https://github.com/chyh1990/yaml-rust/issues/23
        let s = "{-";
        assert!(YamlLoader::load_from_str(&s).is_err());
    }

    #[test]
    fn test_issue_65() {
        // See: https://github.com/chyh1990/yaml-rust/issues/65
        let b = "\n\"ll\\\"ll\\\r\n\"ll\\\"ll\\\r\r\r\rU\r\r\rU";
        assert!(YamlLoader::load_from_str(&b).is_err());
    }

    #[test]
    fn test_bad_docstart() {
        assert!(YamlLoader::load_from_str("---This used to cause an infinite loop").is_ok());
        assert_eq!(
            YamlLoader::load_from_str("----"),
            Ok(vec![Yaml::new(
                YamlData::String(String::from("----")),
                Position { line: 1, column: 0 }
            )]),
            "----"
        );
        assert_eq!(
            YamlLoader::load_from_str("--- #here goes a comment"),
            Ok(vec![Yaml::new(
                YamlData::Null,
                Position { line: 2, column: 0 }
            )]),
            "--- #"
        );
        assert_eq!(
            YamlLoader::load_from_str("---- #here goes a comment"),
            Ok(vec![Yaml::new(
                YamlData::String(String::from("----")),
                Position { line: 1, column: 0 }
            )]),
            "---- #"
        );
    }

    #[test]
    fn test_plain_datatype_with_into_methods() {
        let s = "
- 'string'
- \"string\"
- string
- 123
- -321
- 1.23
- -1e4
- true
- false
- !!str 0
- !!int 100
- !!float 2
- !!bool true
- !!bool false
- 0xFF
- 0o77
- +12345
- -.INF
- .NAN
- !!float .INF
";
        let mut out = YamlLoader::load_from_str(&s).unwrap().into_iter();
        let mut doc = out.next().unwrap().data.into_iter();

        assert_eq!(doc.next().unwrap().data.into_string().unwrap(), "string");
        assert_eq!(doc.next().unwrap().data.into_string().unwrap(), "string");
        assert_eq!(doc.next().unwrap().data.into_string().unwrap(), "string");
        assert_eq!(doc.next().unwrap().data.into_i64().unwrap(), 123);
        assert_eq!(doc.next().unwrap().data.into_i64().unwrap(), -321);
        assert_eq!(doc.next().unwrap().data.into_f64().unwrap(), 1.23);
        assert_eq!(doc.next().unwrap().data.into_f64().unwrap(), -1e4);
        assert_eq!(doc.next().unwrap().data.into_bool().unwrap(), true);
        assert_eq!(doc.next().unwrap().data.into_bool().unwrap(), false);
        assert_eq!(doc.next().unwrap().data.into_string().unwrap(), "0");
        assert_eq!(doc.next().unwrap().data.into_i64().unwrap(), 100);
        assert_eq!(doc.next().unwrap().data.into_f64().unwrap(), 2.0);
        assert_eq!(doc.next().unwrap().data.into_bool().unwrap(), true);
        assert_eq!(doc.next().unwrap().data.into_bool().unwrap(), false);
        assert_eq!(doc.next().unwrap().data.into_i64().unwrap(), 255);
        assert_eq!(doc.next().unwrap().data.into_i64().unwrap(), 63);
        assert_eq!(doc.next().unwrap().data.into_i64().unwrap(), 12345);
        assert_eq!(
            doc.next().unwrap().data.into_f64().unwrap(),
            f64::NEG_INFINITY
        );
        assert!(doc.next().unwrap().data.into_f64().is_some());
        assert_eq!(doc.next().unwrap().data.into_f64().unwrap(), f64::INFINITY);
    }

    #[test]
    fn test_hash_order() {
        let s = "---
b: ~
a: ~
c: ~
";
        let out = YamlLoader::load_from_str(&s).unwrap();
        let first = out.into_iter().next().unwrap();
        let v = match first.data {
            YamlData::Mapping(v) => v,
            _ => panic!(),
        };
        let mut iter = v.into_iter().map(|(x, y)| (x.data, y.data));
        assert_eq!(
            Some((YamlData::String("b".to_owned()), YamlData::Null)),
            iter.next()
        );
        assert_eq!(
            Some((YamlData::String("a".to_owned()), YamlData::Null)),
            iter.next()
        );
        assert_eq!(
            Some((YamlData::String("c".to_owned()), YamlData::Null)),
            iter.next()
        );
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_integer_key() {
        let s = "
0:
    important: true
1:
    important: false
";
        let out = YamlLoader::load_from_str(&s).unwrap();
        let first = out.iter().next().unwrap();
        assert_eq!(
            first.data[0].data["important"].data.as_bool().unwrap(),
            true
        );
    }

    #[test]
    fn test_two_space_indentations() {
        // https://github.com/kbknapp/clap-rs/issues/965

        let s = r#"
subcommands:
  - server:
    about: server related commands
subcommands2:
  - server:
      about: server related commands
subcommands3:
 - server:
    about: server related commands
            "#;

        let out = YamlLoader::load_from_str(&s).unwrap();
        let doc = &out.into_iter().next().unwrap();

        println!("{:#?}", doc);
        assert_eq!(
            doc.data["subcommands"].data[0].data["server"].data,
            YamlData::Null
        );
    }

    #[test]
    fn test_recursion_depth_check_objects() {
        let s = "{a:".repeat(10_000) + &"}".repeat(10_000);
        assert!(YamlLoader::load_from_str(&s).is_err());
    }

    #[test]
    fn test_recursion_depth_check_arrays() {
        let s = "[".repeat(10_000) + &"]".repeat(10_000);
        assert!(YamlLoader::load_from_str(&s).is_err());
    }
}
