use flate2::read::GzDecoder;
use flate2::read::ZlibDecoder;
use git_starter_rust::{cat_file::cat_file, init::init};
use std::env;
use std::fs;
use std::io::Read;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "init" => init(),
        "cat-file" => cat_file(&args[2..]),
        _ => println!("unknown command: {}", args[1]),
    }
}
