use crate::nanpa::find_root;
use anyhow::Result;
use glob::glob;
use std::{env, fs, io::Write, path};
use toml_edit::{value, DocumentMut};

pub fn bump(version: String, location: path::PathBuf) -> Result<()> {
    // update package version
    let toml = fs::read_to_string(location.join("Cargo.toml"))?;
    let mut doc = toml.parse::<DocumentMut>()?;
    doc["package"]["version"] = value(version.clone());
    let name = doc["package"]["name"].as_str().unwrap();
    let mut f = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(location.join("Cargo.toml"))?;
    f.write_all(doc.to_string().as_bytes())?;
    f.flush()?;

    // update dependent packages' manifests
    goto_highest_root()?;
    for file in glob("**/.nanparc")? {
        let file = file?;
        let mut needs_change = false;
        if let Ok(toml) = fs::read_to_string(file.parent().unwrap().join("Cargo.toml").clone()) {
            let mut doc = toml.parse::<DocumentMut>()?;
            if let Some(deps) = doc.get("dependencies") {
                if let Some(dep) = deps.get(name) {
                    if let Some(_) = dep.get("version") {
                        needs_change = true;
                        doc["dependencies"][name]["version"] = value(version.clone())
                    }
                }
            }
            if let Some(workspace) = doc.get("workspace") {
                if let Some(deps) = workspace.get("dependencies") {
                    if let Some(dep) = deps.get(name) {
                        if let Some(_) = dep.get("version") {
                            needs_change = true;
                            doc["workspace"]["dependencies"][name]["version"] =
                                value(version.clone())
                        }
                    }
                }
            }
            if needs_change {
                let mut f = fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(file.parent().unwrap().join("Cargo.toml"))?;
                f.write_all(doc.to_string().as_bytes())?;
                f.flush()?;
            }
        }
    }

    Ok(())
}

fn goto_highest_root() -> Result<()> {
    let cwd = env::current_dir()?;
    if let Some(parent) = env::current_dir()?.parent() {
        env::set_current_dir(parent)?;
    }
    if let Some(root) = find_root(false) {
        let root = fs::canonicalize(root).unwrap();
        env::set_current_dir(root)?;
        return goto_highest_root();
    }
    env::set_current_dir(cwd)?;
    Ok(())
}
