use std::{fs, path::PathBuf};

pub fn init(mut path: PathBuf) {
    path.push(".git");
    fs::create_dir(&path).unwrap();

    let mut objects = path.clone();
    objects.push("objects");
    fs::create_dir(&objects).unwrap();

    let mut refs = path.clone();
    refs.push("refs");
    fs::create_dir(&refs).unwrap();

    let mut head = path.clone();
    head.push("HEAD");
    fs::write(&head, "ref: refs/heads/master\n").unwrap();
    println!("Initialized git directory");
}
