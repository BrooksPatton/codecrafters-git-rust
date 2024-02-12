use std::{fs, path::PathBuf};

pub fn init(mut path: PathBuf) {
    path.push(".git");
    fs::create_dir(&path).expect("error creating .git directory");

    let mut objects = path.clone();
    objects.push("objects");
    fs::create_dir(&objects).expect("error creating objects directory");

    let mut refs = path.clone();
    refs.push("refs");
    fs::create_dir(&refs).expect("error creating refs directory");

    let mut head = path.clone();
    head.push("HEAD");
    fs::write(&head, "ref: refs/heads/master\n").expect("error writing head info to refs");
    println!("Initialized git directory");
}
