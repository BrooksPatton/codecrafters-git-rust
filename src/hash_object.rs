use std::{fs::DirBuilder, io::Write, path::Path};

use flate2::{write::ZlibEncoder, Compression};
use hex::ToHex;
use sha1::{Digest, Sha1};

pub fn hash_object(args: &[String]) {
    match args[0].as_str() {
        "-w" => {
            let filename = &args[1];
            let file = std::fs::read(filename).unwrap();
            let header = get_header(&file);
            let mut content = header.into_bytes();

            content.extend(file);

            let sha = get_sha(&content);
            let compressed_file = compress(&content);
            let folder_path = create_folder(&sha);
            print_sha(&sha);
            save_file(&compressed_file, &folder_path, get_file_sha(&sha));
        }
        _ => eprintln!("Unknown option"),
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

fn create_folder(sha: &str) -> String {
    // normally we should find the .git folder in case we are nested in. Let's yolo because why not? :)
    let path = format!(".git/objects/{}", &sha[0..2]);
    // we need to handle the case that the folder already exists. If that is so, we don't want to crash.
    DirBuilder::new().recursive(true).create(&path).unwrap();

    path
}

fn print_sha(sha: &str) {
    println!("{sha}");
}

/// Save the file to disk, if it already exists don't do anything
fn save_file(file: &[u8], folder_path: &str, file_sha: &str) {
    let path = format!("{folder_path}/{file_sha}");

    let path = Path::new(&path);
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
