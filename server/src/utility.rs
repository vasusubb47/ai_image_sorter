use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};

pub mod file_utilities;
pub mod jwt_token;

pub fn genarate_salt(salt_len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(salt_len)
        .map(char::from)
        .collect()
}

pub fn hash_password(password: &str) -> String {
    let mut sha = Sha256::new();
    let salt = genarate_salt(64);

    sha.update(password.to_owned() + &salt.to_owned());
    let passcode_hash = sha.finalize();

    let passcode_hash = format!("{:X}", passcode_hash) + ":" + &salt;
    passcode_hash
}

pub fn verify_password(password: &str, passcode_hash: &str) -> bool {
    let passcode_parts = passcode_hash.split(':').collect::<Vec<&str>>();
    let passcode_hash = passcode_parts[0];
    let passcode_salt = passcode_parts[1];

    let mut sha = Sha256::new();
    sha.update(password.to_owned() + passcode_salt);
    let user_passcode_hash = sha.finalize();

    let user_passcode_hash = format!("{:X}", user_passcode_hash);

    passcode_hash == user_passcode_hash
}

// pub fn get_current_working_dir() -> std::io::Result<PathBuf> {
//     env::current_dir()
// }
