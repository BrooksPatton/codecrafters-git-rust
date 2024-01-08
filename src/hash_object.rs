use std::{
    fs::DirBuilder,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{bail, Result};
use flate2::{write::ZlibEncoder, Compression};
use hex::ToHex;
use sha1::{Digest, Sha1};

// Update hash object to take in a path, and go from there.
// That way we can call this from write-tree function
pub fn hash_object(write_flag: bool, mut path: PathBuf) -> Result<String> {
    if write_flag {
        let file = std::fs::read(path).unwrap();
        let header = get_header(&file);
        let mut content = header.into_bytes();

        content.extend(file);

        let sha = get_sha(&content);
        let compressed_file = compress(&content);
        let folder_path = create_folder(&sha);
        save_file(&compressed_file, folder_path, get_file_sha(&sha));
        Ok(sha)
    } else {
        unimplemented!();
    }
}

fn get_sha(file: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(file);
    hasher.finalize().encode_hex::<String>()
}

fn compress(file: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(file).unwrap();
    encoder.finish().unwrap()
}

fn create_folder(sha: &str) -> PathBuf {
    // normally we should find the .git folder in case we are nested in. Let's yolo because why not? :)
    // let path = format!(".git/objects/{}", &sha[0..2]);
    let path = Path::new(".git").join("objects").join(&sha[0..2]);
    // we need to handle the case that the folder already exists. If that is so, we don't want to crash.
    DirBuilder::new().recursive(true).create(&path).unwrap();

    path
}

fn print_sha(sha: &str) {
    println!("{sha}");
}

/// Save the file to disk, if it already exists don't do anything
fn save_file(file: &[u8], mut path: PathBuf, file_sha: &str) {
    path.push(file_sha);

    if path.exists() {
        return;
    }

    std::fs::write(path, file).unwrap();
}

fn get_file_sha(sha: &str) -> &str {
    &sha[2..]
}

fn get_header(content: &[u8]) -> String {
    let object_type = "blob";
    let size = content.len();

    format!("{object_type} {size}\0")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_provide_file_sha() {
        let sha = "2qy39147jhrspetndhrsiutljgdwf897";
        let expected_file_sha = "y39147jhrspetndhrsiutljgdwf897";
        let result = get_file_sha(sha);

        assert_eq!(result, expected_file_sha);
    }

    #[test]
    fn should_create_blob_header() {
        let content = "what is up, doc?";
        let expected_result = "blob 16\0";
        let result = get_header(content.as_bytes());

        assert_eq!(result, expected_result);
    }
}
