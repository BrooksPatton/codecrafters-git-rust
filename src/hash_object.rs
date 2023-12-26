use std::{fs::DirBuilder, io::Write};

use flate2::{write::ZlibEncoder, Compression};
use hex::ToHex;
use sha1::{Digest, Sha1};

pub fn hash_object(args: &[String]) {
    match args[0].as_str() {
        "-w" => {
            let filename = &args[1];
            let file = std::fs::read(filename).unwrap();
            let compressed_file = compress(&file);
            let sha = get_sha(&compressed_file);

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

fn save_file(file: &[u8], folder_path: &str, file_sha: &str) {
    let path = format!("{folder_path}/{file_sha}");
    std::fs::write(path, file).unwrap();
}

fn get_file_sha(sha: &str) -> &str {
    &sha[2..]
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
}
