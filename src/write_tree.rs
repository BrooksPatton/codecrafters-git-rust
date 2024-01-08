use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};
use ignore::{Walk, WalkBuilder};

use crate::hash_object::hash_object;

pub fn write_tree() -> Result<String> {
    // files and folders in the current directory
    let path = PathBuf::new().join(".");
    let checksum = write_tree_object(&path)?;

    Ok(checksum)
}

fn write_tree_object(path: &PathBuf) -> Result<String> {
    let dir = std::fs::read_dir(path).unwrap();
    let mut objects = vec![];
    for object in WalkBuilder::new(&path).build() {
        dbg!("walking without .gitignore: ", object?);
    }

    todo!();
    for dir_object_result in dir.into_iter() {
        if let Ok(dir_object) = dir_object_result {
            let file_path = dir_object.path();
            let metadata = dir_object.metadata().unwrap();
            let name = dir_object
                .file_name()
                .into_string()
                .map_err(|_error| anyhow!("Could not get file name from object"))?;

            let file_object = if metadata.is_file() {
                let checksum = hash_object(true, file_path)?;
                TreeObject::new(true, checksum, name)
            } else {
                let dir_path = path.clone().join(&name);
                if dir_path
                    .iter()
                    .filter(|component| *component == "target")
                    .count()
                    > 0
                {
                    // dbg!("found target, skipping...");
                    continue;
                }
                let checksum = write_tree_object(&dir_path)?;
                TreeObject::new(false, checksum, name)
            };

            objects.push(file_object);
        } else {
            bail!("could not load directory object");
        }
    }

    // dbg!(objects);

    todo!()
}

#[derive(Debug)]
struct TreeObject {
    mode: String,
    object_type: TreeObjectType,
    checksum: String,
    name: String,
}

impl TreeObject {
    pub fn new(is_file: bool, checksum: String, name: String) -> Self {
        let object_type = TreeObjectType::new(is_file);
        let mode = object_type.mode();

        Self {
            mode,
            object_type,
            checksum,
            name,
        }
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
            Self::Tree => "040000",
        }
        .to_owned()
    }
}
