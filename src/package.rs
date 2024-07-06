use anyhow::{bail, Result};
use glob::glob;
use std::{
    fs,
    io::{self, BufRead},
    path,
};

pub struct Package {
    // only used for package traversal, flattened
    subpackages: Vec<Package>,
    pub version: Option<String>,
    pub language: Option<String>,
    pub custom: Option<String>,
    pub location: path::PathBuf,
    pub name: Option<String>,
}

impl Package {
    pub fn get(path: path::PathBuf) -> Result<Self> {
        let config = match fs::File::open(path.join(".nanparc")) {
            Ok(file) => io::BufReader::new(file),
            Err(e) => {
                bail!(e);
            }
        };

        let mut subpackages = vec![];
        let mut version: Option<String> = None;
        let mut language: Option<String> = None;
        let mut custom: Option<String> = None;
        let mut name: Option<String> = None;

        for line in config.lines() {
            if let Ok(line) = line {
                if line.is_empty() || line.starts_with("#") {
                    continue;
                }

                let words = line.split_whitespace().collect::<Vec<&str>>();
                let (&keyword, rest) = words.split_first().unwrap();
                if rest.len() == 0 {
                    bail!("keyword {keyword} requires an argument");
                }
                match keyword {
                    "packages" => {
                        for &subpackage in rest {
                            for entry in glob(subpackage)? {
                                subpackages.push(Package::get(entry?)?)
                            }
                        }
                    }
                    "version" => {
                        version = Some(rest[0].to_string());
                    }
                    "language" => {
                        language = Some(rest[0].to_string());
                    }
                    "custom" => {
                        custom = Some(rest[0].to_string());
                    }
                    "name" => {
                        name = Some(rest[0].to_string());
                    }
                    unknown => {
                        bail!("unknown keyword {unknown}")
                    }
                }
            }
        }

        if !subpackages.is_empty() {
            if version.is_some() {
                bail!(
                    "{}: cannot specify version on superpackage",
                    path.to_str().unwrap().to_string()
                );
            }
            if language.is_some() {
                bail!(
                    "{}: cannot specify language on superpackage",
                    path.to_str().unwrap().to_string()
                );
            }
        }

        if subpackages.is_empty() && version.is_none() {
            bail!(
                "{}: config must have version or packages",
                path.to_str().unwrap().to_string()
            );
        }

        if version.is_some() && language.is_none() && name.is_none() {
            bail!(
                "{}: packages without a supported language must have a name",
                path.to_str().unwrap().to_string()
            );
        }

        Ok(Self {
            subpackages,
            version,
            language,
            custom,
            location: path,
            name,
        })
    }

    pub fn flatten(self) -> Result<Vec<Self>> {
        let mut packages = vec![];

        if self.version.is_some() {
            packages.push(self);
        } else {
            for package in self.subpackages {
                packages.append(&mut package.flatten()?);
            }
        }

        Ok(packages)
    }
}
