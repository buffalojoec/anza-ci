//! Filesystem operations.

use std::path::PathBuf;

pub fn find_anza_toml() -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    let path = std::env::current_dir()?;

    let anza_toml = path.join("anza.toml");
    if anza_toml.exists() {
        return Ok(Some(anza_toml));
    }

    let anza_toml = path.join("Anza.toml");
    if anza_toml.exists() {
        return Ok(Some(anza_toml));
    }

    Ok(None)
}
