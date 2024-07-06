use crate::cli::SemverVersion;
use crate::package;
use anyhow::{bail, Result};
use semver;
use std::env;
use std::io::Write;
use std::{
    collections, fs,
    io::{self, BufRead},
    path,
};

pub struct Nanpa {
    // only packages with versions
    packages: Vec<package::Package>,
}

impl Nanpa {
    pub fn new() -> Result<Self> {
        Ok(Self {
            packages: match find_root() {
                Some(path) => package::Package::get(path)?.flatten()?,
                None => {
                    bail!("could not find .nanparc file");
                }
            },
        })
    }

    pub fn get_version(&self) -> collections::HashMap<String, String> {
        let mut versions = collections::HashMap::new();

        for package in &self.packages {
            versions.insert(
                package.location.to_str().unwrap().to_string(),
                package.version.clone().unwrap(),
            );
        }

        versions
    }

    pub fn bump_semver(&self, version: &SemverVersion, package: Option<String>) -> Result<()> {
        if let Some(path) = package {
            todo!();
        } else if self.packages.len() == 1 {
            if let Ok(mut parsed) =
                semver::Version::parse(self.packages[0].version.clone().unwrap().as_str())
            {
                match version {
                    SemverVersion::Major => {
                        parsed.major += 1;
                        parsed.minor = 0;
                        parsed.patch = 0;
                        parsed.pre = semver::Prerelease::new("").unwrap();
                    }
                    SemverVersion::Minor => {
                        parsed.minor += 1;
                        parsed.patch = 0;
                        parsed.pre = semver::Prerelease::new("").unwrap();
                    }
                    SemverVersion::Patch => {
                        parsed.patch += 1;
                        parsed.pre = semver::Prerelease::new("").unwrap();
                    }
                    SemverVersion::Prerelease(x) => {
                        parsed.pre = semver::Prerelease::new(x.version.as_str()).unwrap()
                    }
                };

                println!(
                    "{} -> {}",
                    self.packages[0].version.clone().unwrap(),
                    parsed.to_string()
                );
                todo!();
            } else {
                bail!("package version is not a valid semver version");
            }
        } else {
            bail!("no package specified and more than one package in tree");
        }

        Ok(())
    }

    pub fn bump_custom(&self, version: String, package: Option<String>) -> Result<()> {
        if let Some(path) = package {
            let path = path::PathBuf::from(path);
            let path = fs::canonicalize(&path).unwrap();
            if self.get_version().contains_key(path.to_str().unwrap()) {
                write_custom(path, version.clone())?;
            } else {
                bail!("could not find package");
            }
        } else if self.packages.len() == 1 {
            write_custom(std::env::current_dir().unwrap(), version.clone())?;
        } else {
            bail!("no package specified and more than one package in tree");
        }

        println!(
            "{} -> {}",
            self.packages[0].version.clone().unwrap(),
            version,
        );

        Ok(())
    }
}

fn write_custom(path: path::PathBuf, version: String) -> Result<()> {
    let file = match fs::File::open(path.join(".nanparc")) {
        Ok(file) => io::BufReader::new(file),
        Err(e) => {
            bail!("{}", e.to_string());
        }
    };

    let mut lines = vec![];
    for line in file.lines() {
        if let Ok(line) = line {
            if line.starts_with("version") {
                lines.push("version ".to_string() + version.as_str())
            } else {
                lines.push(line.clone())
            }
        }
    }

    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path.join(".nanparc"))?;
    f.write_all(lines.join("\n").as_bytes())?;
    f.flush()?;

    Ok(())
}

pub fn new() -> Result<Nanpa> {
    Nanpa::new()
}

fn find_root() -> Option<path::PathBuf> {
    let mut dir = env::current_dir().unwrap();
    if dir.join(".nanparc").exists() {
        Some(dir)
    } else {
        eprintln!("current directory does not contain .nanparc, searching up");
        loop {
            if dir == path::PathBuf::from("/") {
                return None;
            }
            dir.pop();
            if dir.join(".nanparc").exists() {
                return Some(dir);
            }
        }
    }
}
