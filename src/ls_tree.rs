use core::panic;
use std::{fs::File, io::Read, path::Path};

use serde::{Deserialize, Serialize};

use crate::utils::{
    decompress, get_object_directory_name, get_object_file_name, index_of_next_null,
};

pub fn ls_tree(args: &[String]) {
    let _option = &args[0];
    let hash = &args[1];
    let directory_name = get_object_directory_name(hash);
    let file_name = get_object_file_name(hash);
    let path = Path::new(".git")
        .join("objects")
        .join(directory_name)
        .join(file_name);
    let mut file = File::open(path).unwrap();
    let mut compressed_bytes = vec![];

    file.read_to_end(&mut compressed_bytes).unwrap();

    let bytes = decompress(&compressed_bytes);
    let tree_object = TreeObject::new(&bytes);

    // figure out where one object begins and ends and then loop throug everything
    // loop {}
}

#[derive(Debug)]
struct TreeObject {
    object_type: String,
    size: u32,
    mode: String,
    filename: String,
}

impl TreeObject {
    // potentially we can return Self and the index that we last used for the \0.
    // Then we can use that to generate the next object???????
    pub fn new(bytes: &[u8]) -> Self {
        let header_null_index = index_of_next_null(&bytes, 0).expect("don't have a header");
        let mut header_bytes = &bytes[0..header_null_index];
        let mut header = String::new();

        header_bytes.read_to_string(&mut header).unwrap();

        let mut header = header.trim().split_whitespace();
        let object_type = header.next().unwrap().to_owned();
        let size = header.next().unwrap().parse().unwrap();

        let mode_null_index =
            index_of_next_null(&bytes, header_null_index + 1).expect("doesn't have a mode");
        let mut mode_and_filename_bytes = &bytes[header_null_index + 1..mode_null_index];
        let mut mode_and_filename = String::new();

        mode_and_filename_bytes
            .read_to_string(&mut mode_and_filename)
            .unwrap();

        let mut mode_and_filename = mode_and_filename.trim().split_whitespace();
        let mode = mode_and_filename.next().unwrap().to_owned();
        let filename = mode_and_filename.next().unwrap().to_owned();

        Self {
            object_type,
            size,
            mode,
            filename,
        }
    }
}
