use crate::util;
use std::{
    fs,
    io::{self, BufRead},
    path, process,
};

pub struct Package {
    // only used for package traversal, flattened
    subpackages: Vec<Package>,
    pub version: Option<String>,
    pub language: Option<String>,
    pub custom: Option<String>,
    pub location: path::PathBuf,
}

impl Package {
    pub fn get(path: path::PathBuf) -> Self {
        let config = match fs::File::open(path.join(".nanparc")) {
            Ok(file) => io::BufReader::new(file),
            Err(e) => {
                util::error(e.to_string().as_str());
                util::error("refer to nanpa(1) for more information");
                process::exit(1);
            }
        };

        let mut subpackages = vec![];
        let mut version: Option<String> = None;
        let mut language: Option<String> = None;
        let mut custom: Option<String> = None;

        for line in config.lines() {
            if let Ok(line) = line {
                if line.is_empty() || line.starts_with("#") {
                    continue;
                }

                let words = line.split_whitespace().collect::<Vec<&str>>();
                let (&keyword, rest) = words.split_first().unwrap();
                if rest.len() == 0 {
                    util::error("configuration error, refer to nanparc(5) for more information");
                    process::exit(1);
                }
                match keyword {
                    "packages" => {
                        for &subpackage in rest {
                            subpackages.push(Package::get(path.join(subpackage)))
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
                    unknown => {
                        util::error(
                            ("unknown configuration keyword ".to_string()
                                + unknown
                                + ", refer to nanparc(5)")
                                .as_str(),
                        );
                        process::exit(1);
                    }
                }
            }
        }

        if !subpackages.is_empty() {
            if version.is_some() {
                util::error(
                    (path.to_str().unwrap().to_string()
                        + ": cannot specify version on superpackage")
                        .as_str(),
                );
                util::error("refer to nanparc(5) for more information");
                process::exit(1);
            }
            if language.is_some() {
                util::error(
                    (path.to_str().unwrap().to_string()
                        + ": cannot specify language on superpackage")
                        .as_str(),
                );
                util::error("refer to nanparc(5) for more information");
                process::exit(1);
            }
        }

        if subpackages.is_empty() && version.is_none() {
            util::error(
                (path.to_str().unwrap().to_string() + ": config must have version or packages")
                    .as_str(),
            );
            util::error("refer to nanparc(5) for more information");
            process::exit(1);
        }

        return Self {
            subpackages,
            version,
            language,
            custom,
            location: path,
        };
    }

    pub fn flatten(self) -> Vec<Self> {
        let mut packages = vec![];

        if self.version.is_some() {
            packages.push(self);
        } else {
            for package in self.subpackages {
                packages.append(&mut package.flatten());
            }
        }

        packages
    }
}
