use anyhow::Result;
use serde_json::Value;
use std::{fs, io::Write, path};

pub fn bump(version: String, location: path::PathBuf) -> Result<()> {
    let json = fs::read_to_string(location.join("package.json"))?;
    let mut doc: Value = serde_json::from_str(json.as_str())?;

    if let Some(v) = doc.get_mut("version") {
        *v = Value::from(version);
    }

    let mut f = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(location.join("package.json"))?;
    f.write_all(serde_json::to_string_pretty(&doc)?.as_bytes())?;
    f.flush()?;

    Ok(())
}
