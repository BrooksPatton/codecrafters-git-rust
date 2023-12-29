use std::{fs::File, io::Read, path::Path};

use crate::utils::{decompress, get_object_directory_name, get_object_file_name, next_chunk};

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
    let mut bytes_slice = &bytes[..];
    // let mut trees_objects = vec![];

    let header_bytes = next_chunk(&bytes_slice, 0).unwrap();
    let header = bytes_to_string(&header_bytes);
    let null_index = bytes_slice.iter().position(|&byte| byte == b'\0').unwrap();
    bytes_slice = &bytes[null_index + 1..];

    let mode_bytes = next_chunk(&bytes_slice, 0).unwrap();
    let mode = bytes_to_string(&mode_bytes);
    let null_index = bytes_slice.iter().position(|&byte| byte == b'\0').unwrap();

    let null_index = null_index + 20;
    bytes_slice = &bytes[null_index + 1..];

    let header2_bytes = next_chunk(&bytes_slice, 0).unwrap();
    let header2 = bytes_to_string(&header2_bytes);

    dbg!(header, mode, header2);

    // more black box testing needed

    // for bytes in bytes.split(|&byte| byte == b'\n') {
    //     let mode_bytes = next_chunk(&bytes, 0).unwrap();

    //     trees_objects.push(TreeObject::new(header_bytes, mode_bytes));
    //     dbg!(&trees_objects);
    // }

    // let (tree_object, last_used_index) = TreeObject::new(&bytes);
    // let (tree_object_two, second_last_used_index) = TreeObject::new(&bytes[last_used_index + 2..]);

    // dbg!(tree_object);
    // dbg!(tree_object_two);

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
    pub fn new(mut header_bytes: &[u8], mut mode_bytes: &[u8]) -> Self {
        let mut header = String::new();

        header_bytes.read_to_string(&mut header).unwrap();

        let mut header = header.trim().split_whitespace();
        let object_type = header.next().unwrap().to_owned();
        let size = header.next().unwrap().parse().unwrap();

        let mut mode_and_filename = String::new();

        mode_bytes.read_to_string(&mut mode_and_filename).unwrap();

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

fn bytes_to_string(mut bytes: &[u8]) -> String {
    let mut result = String::new();

    bytes.read_to_string(&mut result).unwrap();

    result
}
