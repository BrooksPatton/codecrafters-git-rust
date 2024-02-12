use std::{fs::File, io::Read, path::Path};

use crate::utils::{decompress, get_object_directory_name, get_object_file_name};

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

    let mut bytes = decompress(&compressed_bytes).into_iter();
    let mut parser = TreeParser::Header;
    let mut filenames = vec![];

    'parse_filenames: loop {
        let Some(filename) = parser.step(&mut bytes) else {
            if matches!(parser, TreeParser::Done) {
                break 'parse_filenames;
            } else {
                continue;
            };
        };

        filenames.push(filename);
    }

    filenames.iter().for_each(|filename| println!("{filename}"));
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
    pub fn step<'a>(&mut self, mut bytes: impl Iterator<Item = u8>) -> Option<String> {
        match self {
            Self::Header => loop {
                let Some(next_byte) = bytes.next() else {
                    panic!("unable to get next byte in Header");
                };

                if next_byte == b'\0' {
                    *self = Self::Mode;
                    return None;
                }
            },
            TreeParser::Mode => loop {
                let Some(next_byte) = bytes.next() else {
                    panic!("unable to get next byte in Mode");
                };

                if next_byte == b' ' {
                    *self = Self::Filename;
                    return None;
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
