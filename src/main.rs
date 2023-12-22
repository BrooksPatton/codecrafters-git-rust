use git_starter_rust::{cat_file::cat_file, init::init};
use std::env;

fn main() {
    // Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "init" => init(),
        "cat-file" => cat_file(&args[2..]),
        _ => println!("unknown command: {}", args[1]),
    }
}
