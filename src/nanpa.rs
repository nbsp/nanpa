use crate::cli::SemverVersion;
use crate::package;
use crate::util;
use semver;
use std::{collections, process};

pub struct Nanpa {
    // only packages with versions
    packages: Vec<package::Package>,
}

impl Nanpa {
    pub fn new() -> Self {
        Self {
            packages: match util::find_root() {
                Some(path) => package::Package::get(path).flatten(),
                None => {
                    util::error("could not find .nanparc file, refer to nanparc(5)");
                    process::exit(1);
                }
            },
        }
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

    pub fn bump_semver(&self, version: &SemverVersion, package: Option<String>) {
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
                util::error("package version is not a valid semver version");
                util::error("refer to nanpa(1) for more information");
                process::exit(1);
            }
        } else {
            util::error("no package specified and more than one package in tree");
            util::error("refer to nanparc(5) for more information");
            process::exit(1);
        }
    }

    pub fn bump_custom(&self, version: String, package: Option<String>) {
        if let Some(path) = package {
            todo!();
        } else if self.packages.len() == 1 {
            println!(
                "{} -> {}",
                self.packages[0].version.clone().unwrap(),
                version,
            );
            todo!();
        } else {
            util::error("no package specified and more than one package in tree");
            util::error("refer to nanparc(5) for more information");
            process::exit(1);
        }
    }
}

pub fn new() -> Nanpa {
    Nanpa::new()
}
