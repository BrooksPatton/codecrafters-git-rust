use std::env;
use std::fs;
use std::io::Read;

use flate2::read::GzDecoder;
use flate2::read::ZlibDecoder;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();
    if args[1] == "init" {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
        println!("Initialized git directory");
    } else if args[1] == "cat-file" {
        if args[2] == "-p" {
            let hash = args[3].clone();
            // the first two characters of the hash are the folder name, and the rest are the file name
            let folder_name = &hash[0..2];
            let file_name = &hash[2..];
            let path = format!(".git/objects/{folder_name}/{file_name}");
            let mut object = fs::File::open(&path).unwrap();
            let mut content: Vec<u8> = vec![];
            let mut extracted_content = String::new();

            object.read_to_end(&mut content).unwrap();

            let mut decoder = ZlibDecoder::new(content.as_slice());

            decoder.read_to_string(&mut extracted_content).unwrap();

            println!("{extracted_content}");
        }
    } else {
        println!("unknown command: {}", args[1]);
    }
}
