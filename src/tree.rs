use anyhow::{Context, Result};

use crate::hash::Hash;

pub struct Tree {
    tree_objects: Vec<TreeObject>,
}

impl From<Vec<u8>> for Tree {
    fn from(value: Vec<u8>) -> Self {
        let mut tree_objects = vec![];
        let mut parser = TreeParser::Header;
        let mut bytes = value.into_iter();

        // We need to rewrite the TreeParser step method AND
        // this loop so that we can create tree objects for every line in the bytes
        // currently each step is trying to get a filename from the line.
        'parse_tree_objects: loop {
            // don't like initializing here. But not sure right now the best place to put this
            let mut tree_object = TreeObject::default();
            if parser.step(&mut bytes, &mut tree_object)? {
            } else {
                if matches!(parser, TreeParser::Done) {
                    break 'parse_filenames;
                } else {
                    continue;
                };
            };

            filenames.push(filename);
        }

        filenames
    }
}

#[derive(Default)]
pub struct TreeObject {
    mode: u32,
    object_type: TreeObjectType,
    filename: String,
    checksum: Hash,
}

#[derive(Default)]
pub enum TreeObjectType {
    #[default]
    Blob,
    Tree,
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
                    return false;
                }
            },
            TreeParser::Mode => loop {
                // We've only worked on MODE, need to rework we think
                let mut mode = vec![];
                let Some(next_byte) = bytes.next() else {
                    panic!("unable to get next byte in Mode");
                };

                if next_byte == b' ' {
                    let mode = String::from_utf8(mode)?;
                    let mode = u32::from_str_radix(&mode, 8)?;

                    tree_object.mode = mode;
                    *self = Self::Filename;

                    return false;
                } else {
                    mode.push(next_byte);
                }
            },
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
                return Some(filename);
            }
            TreeParser::Checksum => {
                for _ in 0..21 {
                    if let None = bytes.next() {
                        *self = Self::Done;
                        return None;
                    }
                }

                *self = Self::Mode;
                return None;
            }
            TreeParser::Done => return None,
        }
    }
}
