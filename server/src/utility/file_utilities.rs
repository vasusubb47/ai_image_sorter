use ::serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn _get_file_type(file_path: &str) -> String {
    let path = PathBuf::from(file_path);
    let ext = path.extension().unwrap();
    ext.to_str().unwrap().to_owned()
}

pub fn get_file_type_from_mime(mime: &str) -> String {
    (mime.split('/').collect::<Vec<&str>>()[1]).to_owned()
}

pub fn create_file_write_all(file_path: &Path, content: &[u8]) {
    let mut file = fs::File::create(file_path).unwrap();
    file.write_all(content).unwrap();
}

pub fn object_to_byte_vec<T: Serialize>(object: &T) -> Vec<u8> {
    serde_json::to_string(object).unwrap().as_bytes().to_vec()
}
