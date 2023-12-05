use rand::{distributions::Alphanumeric, Rng};
use std::fmt::Debug;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub mod jwt_token;

pub fn genarate_salt(salt_len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(salt_len)
        .map(char::from)
        .collect()
}

pub fn get_file_type(file_path: &str) -> String {
    let path = PathBuf::from(file_path);
    let ext = path.extension().unwrap();
    ext.to_str().unwrap().to_owned()
}

pub fn create_file_write_all(file_path: &Path, content: &[u8]) {
    let mut file = fs::File::create(file_path).unwrap();
    file.write_all(content).unwrap();
}

pub fn get_vec_to_sql_str<T>(vec_data: &Vec<T>) -> String
where
    T: Debug,
{
    let mut vec_data_str = Vec::<String>::new();

    for ele in vec_data {
        vec_data_str.push(format!("'{:?}'", ele));
    }

    let sql_vec_str = vec_data_str.join(",");

    format!("({})", sql_vec_str)
}

// pub fn get_current_working_dir() -> std::io::Result<PathBuf> {
//     env::current_dir()
// }
