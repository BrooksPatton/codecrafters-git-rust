use anyhow::Context;
use anyhow::Result;
use std::path::PathBuf;

use crate::{clone::GitObjects, hash::Hash};

pub fn checkout(_path: PathBuf, git_objects: &GitObjects, commit_hash: Hash) -> Result<()> {
    let commit = git_objects
        .get(&commit_hash)
        .expect("cannot find commit in git objects");

    let tree_start_index = commit
        .iter()
        .position(|&commit| commit == b'\0')
        .expect("could not find \\0 in commit");
    let tree_hash: Hash = commit[tree_start_index + 6..=tree_start_index + 45]
        .to_vec()
        .try_into()
        .context("converting commit bytes to hash")?;

    let tree = git_objects
        .get(&tree_hash)
        .expect("missing tree referenced in commit");

    dbg!(String::from_utf8_lossy(&tree));
    println!("{tree_hash}");

    Ok(())
}
