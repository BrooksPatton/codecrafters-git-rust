use std::path::Path;

use anyhow::Result;

use crate::utils::create_directory;

pub fn clone(_uri: &str, target_dir: &str) -> Result<()> {
    // create directory
    let target_directory = Path::new(".").join(target_dir);

    create_directory(&target_directory)?;

    Ok(())
}
