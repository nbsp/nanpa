use anyhow::Result;
use std::{fs, io::Write, path};
use toml_edit::{value, DocumentMut};

pub fn bump(version: String, location: path::PathBuf) -> Result<()> {
    let toml = fs::read_to_string(location.join("Cargo.toml"))?;
    let mut doc = toml.parse::<DocumentMut>()?;
    doc["package"]["version"] = value(version);
    let mut f = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(location.join("Cargo.toml"))?;
    f.write_all(doc.to_string().as_bytes())?;
    f.flush()?;

    Ok(())
}
