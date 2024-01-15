use std::{fmt::Display, path::PathBuf};

use anyhow::{Context, Result};
use hex::ToHex;
use ignore::WalkBuilder;

use crate::{hash_object::hash_object, utils::save_to_disk};

pub fn write_tree() -> Result<String> {
    // files and folders in the current directory
    let path = PathBuf::new().join(".");
    let checksum = write_tree_object(&path)?.expect("getting checksum after writing all trees");

    Ok(hex::encode(checksum))
}

fn write_tree_object(path: &PathBuf) -> Result<Option<Vec<u8>>> {
    println!("writing tree object: {:?}", &path);
    let mut objects = vec![];
    for object in WalkBuilder::new(&path)
        .hidden(false)
        .max_depth(Some(1))
        .build()
        .skip(1)
    {
        let dir_object = object?;
        let file_path = dir_object.path();
        let metadata = dir_object.metadata().unwrap();
        let name = dir_object
            .file_name()
            .to_str()
            .context("Could not get file name from object")?
            .to_owned();

        let file_object = if metadata.is_file() {
            let checksum = hash_object(true, file_path.to_path_buf())?;

            Some(TreeObject::new(true, checksum, name))
        } else {
            if let Some(checksum) = write_tree_object(&file_path.to_path_buf())? {
                println!("name: {} - {}", &name, checksum.encode_hex::<String>());
                // dbg!(&name, checksum.encode_hex::<String>());
                Some(TreeObject::new(false, checksum, name))
            } else {
                None
            }
        };

        objects.extend(file_object);
    }

    objects.sort_unstable_by_key(|object| object.name.clone());
    if objects.is_empty() {
        Ok(None)
    } else {
        let tree_file = create_tree_file(&objects);
        let hash = save_to_disk(&tree_file)?;

        Ok(Some(hash))
    }
}

#[derive(Debug)]
struct TreeObject {
    mode: String,
    checksum: Vec<u8>,
    name: String,
}

impl TreeObject {
    pub fn new(is_file: bool, checksum: Vec<u8>, name: String) -> Self {
        let object_type = TreeObjectType::new(is_file);
        let mode = object_type.mode();

        Self {
            mode,
            checksum,
            name,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        let mode = format!("{} ", &self.mode);
        bytes.extend(mode.as_bytes());

        let name = format!("{}\0", &self.name);
        bytes.extend(name.as_bytes());

        bytes.extend(&self.checksum);

        bytes
    }
}

impl Display for TreeObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}\0{:?}", &self.mode, &self.name, &self.checksum)
    }
}

#[derive(Debug)]
enum TreeObjectType {
    Blob,
    Tree,
}

impl TreeObjectType {
    pub fn new(is_file: bool) -> Self {
        if is_file {
            Self::Blob
        } else {
            Self::Tree
        }
    }

    pub fn mode(&self) -> String {
        match self {
            Self::Blob => "100644",
            Self::Tree => "40000",
        }
        .to_owned()
    }
}

impl Display for TreeObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Blob => "blob",
            Self::Tree => "tree",
        };

        write!(f, "{name}")
    }
}

fn create_tree_file(objects: &[TreeObject]) -> Vec<u8> {
    let mut tree_file = vec![];

    let objects = objects
        .iter()
        .map(|object| object.as_bytes())
        .collect::<Vec<Vec<u8>>>();
    let size = objects.iter().fold(0, |acc, object| acc + object.len());

    tree_file.extend(format!("tree {size}\0").as_bytes());
    for object in objects {
        tree_file.extend(object)
    }

    tree_file
}

// fn write_tree_to_file(content: &str, hash: &str) -> Result<()> {

// }
