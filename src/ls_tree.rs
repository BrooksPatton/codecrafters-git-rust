use std::{fs::File, io::Read, path::Path};

use crate::{
    tree::parse_tree_from_bytes,
    utils::{decompress, get_object_directory_name, get_object_file_name},
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
    let mut file = File::open(path).expect("error opening file");
    let mut compressed_bytes = vec![];

    file.read_to_end(&mut compressed_bytes)
        .expect("error reading file to end");

    let bytes = decompress(&compressed_bytes);
    let filenames = parse_tree_from_bytes(bytes);

    filenames.iter().for_each(|filename| println!("{filename}"));
}
