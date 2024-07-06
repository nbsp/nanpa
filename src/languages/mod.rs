use std::path;

use anyhow::{bail, Result};

mod cargo;
mod go;
mod node;

pub fn run_language(language: String, version: String, location: path::PathBuf) -> Result<()> {
    match language.as_str() {
        "cargo" | "rust" => cargo::bump(version, location)?,
        "node" | "javascript" | "js" | "typescript" | "ts" => node::bump(version)?,
        "go" => go::bump(version)?,
        unknown => bail!("unsupported language {unknown}. see `nanpa list-languages` for a list of supported languages"),
    }

    Ok(())
}
