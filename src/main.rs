use git_starter_rust::{
    cat_file::cat_file, hash_object::hash_object, init::init, ls_tree::ls_tree,
    write_tree::write_tree,
};
use std::env;

fn main() {
    // Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();
    let rest_of_args = &args[2..];

    match args[1].as_str() {
        "init" => init(),
        "cat-file" => cat_file(rest_of_args),
        "hash-object" => {
            let checksum = hash_object(rest_of_args).unwrap();
            println!("{checksum}");
        }
        "ls-tree" => ls_tree(rest_of_args),
        "write-tree" => {
            let checksum = write_tree().unwrap();
            println!("{checksum}");
        }
        _ => println!("unknown command: {}", args[1]),
    }
}
