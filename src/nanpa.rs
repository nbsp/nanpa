use crate::cli::SemverVersion;
use crate::package;
use anyhow::{bail, Result};
use semver;
use std::{
    collections, env, fs,
    io::{self, BufRead, Write},
    path, process,
};

pub struct Nanpa {
    // only packages with versions
    packages: Vec<package::Package>,
}

impl Nanpa {
    pub fn new() -> Result<Self> {
        Ok(Self {
            packages: match find_root(true) {
                Some(path) => package::Package::get(path)?.flatten()?,
                None => {
                    bail!("could not find .nanparc file");
                }
            },
        })
    }

    pub fn packages(&self) -> collections::HashMap<String, package::Package> {
        let mut packages = collections::HashMap::new();

        for package in self.packages.clone() {
            packages.insert(
                package.location.to_str().unwrap().to_string(),
                package.clone(),
            );
        }

        packages
    }

    pub fn bump_semver(&self, version: &SemverVersion, package: Option<String>) -> Result<()> {
        if let Some(path) = package {
            let path = path::PathBuf::from(path);
            let path = fs::canonicalize(&path).unwrap();
            if let Some(package) = self.packages().get(path.to_str().unwrap()).cloned() {
                write_semver(package, version)?;
            } else {
                bail!("could not find package");
            }
        } else if self.packages.len() == 1 && self.packages[0].location == find_root(false).unwrap()
        {
            write_semver(
                self.packages()
                    .get(find_root(false).unwrap().to_str().unwrap())
                    .unwrap()
                    .clone(),
                version,
            )?;
        } else {
            bail!("no package specified and more than one package in tree");
        }

        Ok(())
    }

    pub fn bump_custom(&self, version: String, package: Option<String>) -> Result<()> {
        if let Some(path) = package {
            let path = path::PathBuf::from(path);
            let path = fs::canonicalize(&path).unwrap();
            if let Some(package) = self.packages().get(path.to_str().unwrap()).cloned() {
                write_custom(package, version.clone())?;
            } else {
                bail!("could not find package");
            }
        } else if self.packages.len() == 1 && self.packages[0].location == find_root(false).unwrap()
        {
            write_custom(
                self.packages()
                    .get(find_root(false).unwrap().to_str().unwrap())
                    .unwrap()
                    .clone(),
                version.clone(),
            )?;
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

fn write_semver(package: package::Package, version: &SemverVersion) -> Result<()> {
    let file = match fs::File::open(package.location.join(".nanparc")) {
        Ok(file) => io::BufReader::new(file),
        Err(e) => {
            bail!("{}", e.to_string());
        }
    };

    if let Ok(mut parsed) = semver::Version::parse(package.version.clone().unwrap().as_str()) {
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

        let mut lines = vec![];
        for line in file.lines() {
            if let Ok(line) = line {
                if line.starts_with("version") {
                    lines.push("version ".to_string() + parsed.to_string().as_str())
                } else {
                    lines.push(line.clone())
                }
            }
        }

        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(package.location.join(".nanparc"))?;
        f.write_all((lines.join("\n") + "\n").as_bytes())?;
        f.flush()?;

        println!(
            "{} -> {}",
            package.version.clone().unwrap(),
            parsed.to_string()
        );

        run_custom(package)?;
    } else {
        bail!("package version is not a valid semver version");
    }

    Ok(())
}

fn write_custom(package: package::Package, version: String) -> Result<()> {
    let file = match fs::File::open(package.location.join(".nanparc")) {
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
        .open(package.location.join(".nanparc"))?;
    f.write_all((lines.join("\n") + "\n").as_bytes())?;
    f.flush()?;

    run_custom(package)?;
    Ok(())
}

pub fn new() -> Result<Nanpa> {
    Nanpa::new()
}

fn find_root(stdout: bool) -> Option<path::PathBuf> {
    let mut dir = env::current_dir().unwrap();
    if dir.join(".nanparc").exists() {
        Some(dir)
    } else {
        if stdout {
            eprintln!("current directory does not contain .nanparc, searching up");
        }
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

fn run_custom(package: package::Package) -> Result<()> {
    if let Some(custom) = package.custom {
        env::set_current_dir(package.location.clone())?;
        process::Command::new(package.location.join(custom)).spawn()?;
    }

    Ok(())
}
