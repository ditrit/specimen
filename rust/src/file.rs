use std::fs;

pub struct File {
    pub path: Box<str>,
    pub content: Box<str>,
}

/// Note: To create a "virtual file", just construct a `File {}` instance and specify both the file path and its content.
impl File {
    /// This function reads a file from the local file system and returns a File struct.
    pub fn read_local_file(path: &str) -> File {
        File {
            path: path.into(),
            content: fs::read_to_string(path)
                .expect("The path should point to an existing file.")
                .into(),
        }
    }
}
