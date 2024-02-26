use core::panic;

use anyhow::{bail, Result};

use crate::hash::Hash;

pub struct Tree {
    tree_objects: Vec<TreeObject>,
}

impl Tree {
    pub fn filenames(&self) -> Vec<&str> {
        self.tree_objects
            .iter()
            .map(|tree_object| tree_object.filename.as_str())
            .collect()
    }
}

impl From<Vec<u8>> for Tree {
    fn from(value: Vec<u8>) -> Self {
        let mut tree_objects = vec![];
        let mut parser = TreeParser::Header;
        let mut lines = value.split(|&byte| byte == b'\0');
        let mut tree_object = TreeObject::default();

        tree_object
            .parse_header(lines.next())
            .expect("error parseing header");

        tree_object
            .parse_mode_and_filename(lines.next())
            .expect("error parseing mode and filename");

        dbg!(tree_object);
        panic!();
        // parser.parse(line);

        // // We need to rewrite the TreeParser step method AND
        // this loop so that we can create tree objects for every line in the bytes
        // // currently each step is trying to get a filename from the line.
        // let mut tree_object = TreeObject::default();
        // loop {
        //     // don't like initializing here. But not sure right now the best place to put this
        //     if parser
        //         .step(&mut bytes, &mut tree_object)
        //         .expect("error stepping through tree")
        //     {
        //         break;
        //     };

        //     tree_objects.push(tree_object);
        //     tree_object = TreeObject::default();
        // }

        // dbg!(&tree_objects);

        Self { tree_objects }
    }
}

#[derive(Default, Debug)]
pub struct TreeObject {
    mode: u32,
    object_type: TreeObjectType,
    filename: String,
    checksum: Hash,
}

impl TreeObject {
    pub fn parse_header(&mut self, bytes: Option<&[u8]>) -> Result<()> {
        let Some(bytes) = bytes else {
            bail!("missing bytes")
        };
        let mut split_bytes = bytes.split(|&byte| byte == b' ');
        let type_as_bytes = split_bytes
            .next()
            .expect("missing type when parseing type and type");
        let object_type = TreeObjectType::from(type_as_bytes);

        self.object_type = object_type;

        Ok(())
    }

    pub fn parse_mode_and_filename(&mut self, bytes: Option<&[u8]>) -> Result<()> {
        let Some(bytes) = bytes else {
            bail!("missing bytes")
        };
        let mut split_bytes = bytes.split(|&byte| byte == b' ');
        let mode_as_bytes = split_bytes
            .next()
            .expect("missing mode when parseing mode and filename");
        let mode = std::str::from_utf8(mode_as_bytes)?.parse()?;
        let filename_as_bytes = split_bytes
            .next()
            .expect("missing filename when parseing mode and filename");
        let filename = std::str::from_utf8(filename_as_bytes)?.to_owned();

        self.mode = mode;
        self.filename = filename;

        Ok(())
    }
}

#[derive(Default, Debug)]
pub enum TreeObjectType {
    #[default]
    Blob,
    Tree,
}

impl From<&[u8]> for TreeObjectType {
    fn from(value: &[u8]) -> Self {
        let stringified = std::str::from_utf8(value)
            .expect("Error: unable to extract tree object type string from bytes");

        match stringified.to_lowercase().as_str() {
            "blob" => Self::Blob,
            "tree" => Self::Tree,
            _ => unreachable!("attempting to extract tree object type, but not one of the types"),
        }
    }
}

#[derive(Debug)]
enum TreeParser {
    Header,
    Mode,
    Filename,
    Checksum,
    Done,
}

impl TreeParser {
    pub fn parse(&mut self, bytes: Option<&[u8]>) {
        dbg!(bytes);
    }

    pub fn step<'a>(
        &mut self,
        mut bytes: impl Iterator<Item = u8>,
        tree_object: &mut TreeObject,
        // probably need a different return value, current thinking is to have it return "I am done or not, but don't like it."
    ) -> Result<bool> {
        match self {
            Self::Header => loop {
                let Some(next_byte) = bytes.next() else {
                    panic!("unable to get next byte in Header");
                };

                if next_byte == b'\0' {
                    *self = Self::Mode;
                    return Ok(false);
                }
            },
            TreeParser::Mode => {
                let mut mode = vec![];
                loop {
                    // We've only worked on MODE, need to rework we think
                    let Some(next_byte) = bytes.next() else {
                        panic!("unable to get next byte in Mode");
                    };

                    if next_byte == b' ' {
                        let mode = String::from_utf8(mode)?;
                        let mode = mode.parse()?;
                        dbg!(mode);

                        tree_object.mode = mode;
                        *self = Self::Filename;

                        return Ok(false);
                    } else {
                        mode.push(next_byte);
                    }
                }
            }
            TreeParser::Filename => {
                let mut filename = String::new();
                loop {
                    let Some(next_byte) = bytes.next() else {
                        panic!("unable to get next byte in Filename");
                    };

                    if next_byte == b'\0' {
                        break;
                    }

                    let byte = [next_byte];

                    filename.push_str(std::str::from_utf8(&byte).expect("error pushing filename"));
                }

                *self = Self::Checksum;
                return Ok(false);
            }
            TreeParser::Checksum => {
                for _ in 0..21 {
                    if let None = bytes.next() {
                        *self = Self::Done;
                        return Ok(false);
                    }
                }

                *self = Self::Mode;
                return Ok(false);
            }
            TreeParser::Done => return Ok(true),
        }
    }
}
