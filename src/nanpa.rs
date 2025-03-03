use crate::cli::SemverVersionAdd;
use crate::package;
use crate::{cli::SemverVersion, languages};
use anyhow::{bail, Result};
use chrono;
use glob::glob;
use kdl::{KdlDocument, KdlNode};
use rand::prelude::IndexedRandom;
use semver;
use std::{
    collections, env, fs,
    io::{self, BufRead, Read, Write},
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
            "{}: {} -> {}",
            self.packages[0].location.to_str().unwrap(),
            self.packages[0].version.clone().unwrap(),
            version,
        );

        Ok(())
    }

    pub fn changesets(
        &self,
        package: Option<String>,
        pre: Option<String>,
        yes: bool,
    ) -> Result<()> {
        if let Some(path) = package {
            let path = path::PathBuf::from(path);
            let path = fs::canonicalize(&path).unwrap();
            if let Some(package) = self.packages().get(path.to_str().unwrap()).cloned() {
                changesets(package, find_root(false).unwrap(), pre, yes)?;
            } else {
                bail!("could not find package");
            }
        } else if self.packages.len() == 1 && self.packages[0].location == find_root(false).unwrap()
        {
            changesets(
                self.packages()
                    .get(find_root(false).unwrap().to_str().unwrap())
                    .unwrap()
                    .clone(),
                find_root(false).unwrap(),
                pre,
                yes,
            )?
        } else {
            let root = fs::canonicalize(find_root(false).unwrap()).unwrap();
            for package in self.packages().values() {
                changesets(package.clone(), root.clone(), pre.clone(), yes)?;
            }
        }

        Ok(())
    }

    pub fn add(
        &self,
        package: Option<String>,
        bump: SemverVersionAdd,
        change_type: Option<String>,
        message: Option<String>,
    ) -> Result<()> {
        add_changeset(
            package,
            find_root(false).unwrap(),
            bump,
            change_type,
            message,
        )?;

        Ok(())
    }
}

fn write_semver(package: package::Package, version: &SemverVersion) -> Result<String> {
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

        let mut f = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(package.location.join(".nanparc"))?;
        f.write_all((lines.join("\n") + "\n").as_bytes())?;
        f.flush()?;

        println!(
            "{}: {} -> {}",
            package.location.to_str().unwrap(),
            package.version.clone().unwrap(),
            parsed.to_string()
        );

        run_language(package.clone(), parsed.to_string())?;
        run_custom(package, parsed.to_string())?;

        Ok(parsed.to_string())
    } else {
        bail!("package version is not a valid semver version");
    }
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

    let mut f = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(package.location.join(".nanparc"))?;
    f.write_all((lines.join("\n") + "\n").as_bytes())?;
    f.flush()?;

    run_language(package.clone(), version.clone())?;
    run_custom(package, version)?;
    Ok(())
}

pub fn new() -> Result<Nanpa> {
    Nanpa::new()
}

pub fn find_root(stdout: bool) -> Option<path::PathBuf> {
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

fn run_custom(package: package::Package, parsed: String) -> Result<()> {
    if let Some(custom) = package.custom {
        env::set_var("VERSION", parsed);
        env::set_current_dir(package.location.clone())?;
        process::Command::new(package.location.join(custom)).spawn()?;
    }

    Ok(())
}

fn changesets(
    package: package::Package,
    root: path::PathBuf,
    pre: Option<String>,
    yes: bool,
) -> Result<()> {
    let mut bump = 0;
    let mut changelog = Changelog::new();
    let mut to_delete: Vec<path::PathBuf> = vec![];

    env::set_current_dir(root)?;
    for file in glob(".nanpa/*.kdl")? {
        let file = file?;
        let mut changeset: KdlDocument = fs::read_to_string(file.clone())?.parse()?;
        let nodes = changeset.nodes_mut();
        let nodes_cloned = nodes.clone();
        let package_nodes: Vec<&KdlNode> = nodes_cloned
            .iter()
            .filter(|change| {
                change.get("package").is_some_and(|path| {
                    package.location == fs::canonicalize(path.value().as_string().unwrap()).unwrap()
                })
            })
            .collect();
        for node in package_nodes {
            if node.get(0).cloned().is_some() {
                match node.name().to_string().as_str() {
                    "major" => {
                        bump = 3;
                        changelog.push(node.clone())?;
                    }
                    "minor" => {
                        if bump < 2 {
                            bump = 2;
                        }
                        changelog.push(node.clone())?;
                    }
                    "patch" => {
                        if bump == 0 {
                            bump = 1;
                        }
                        changelog.push(node.clone())?;
                    }
                    unknown => bail!("{}: unknown keyword {unknown}", file.to_str().unwrap()),
                }
            }

            nodes.remove(nodes.iter().position(|x| *x == *node).unwrap());
        }
        if changeset.nodes().is_empty() {
            to_delete.push(fs::canonicalize(file.to_str().unwrap().to_string()).unwrap())
        } else {
            let mut f = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(file)?;
            f.write_all(changeset.to_string().as_bytes())?;
            f.flush()?;
        }
    }

    env::set_current_dir(package.location.clone())?;
    for file in glob(".nanpa/*.kdl")? {
        let file = file?;
        let changeset: KdlDocument = fs::read_to_string(file.clone())?.parse()?;
        for node in changeset.nodes() {
            if node.get(0).cloned().is_some() {
                match node.name().to_string().as_str() {
                    "major" => {
                        bump = 3;
                        changelog.push(node.clone())?;
                    }
                    "minor" => {
                        if bump < 2 {
                            bump = 2;
                        }
                        changelog.push(node.clone())?;
                    }
                    "patch" => {
                        if bump == 0 {
                            bump = 1;
                        }
                        changelog.push(node.clone())?;
                    }
                    unknown => bail!("unknown keyword {unknown}"),
                }
            }
        }
        to_delete.push(fs::canonicalize(file.to_str().unwrap().to_string()).unwrap());
    }

    let semver = semver::Version::parse(package.version.clone().unwrap().as_str());
    if semver.is_err() {
        bail!(
            "{}: package version is not a valid semver version",
            package.location.to_str().unwrap()
        );
    }
    let mut version = semver.unwrap();
    match bump {
        0 => {
            if !version.pre.is_empty() {
                if let Some(pre) = pre {
                    let head = version.pre.split_once(".").unwrap().0;
                    if head != pre {
                        version.pre = semver::Prerelease::new(format!("{pre}.0").as_str()).unwrap();
                    } else {
                        println!(
                            "{}: no changesets found",
                            package.location.to_str().unwrap()
                        );
                        return Ok(());
                    }
                } else {
                    version.pre = semver::Prerelease::EMPTY;
                }
            } else {
                println!(
                    "{}: no changesets found",
                    package.location.to_str().unwrap()
                );
                return Ok(());
            }
        }
        3 => {
            if version.minor == 0 && version.patch == 0 && !version.pre.is_empty() && pre.is_some()
            {
                // XXX: assumes prerelease matches `^[\w\d]+\.\d+$`
                let (head, tail) = version.pre.split_once(".").unwrap();
                let pre = pre.unwrap();
                if pre == head {
                    let tail = tail.parse::<u8>().unwrap() + 1;
                    version.pre =
                        semver::Prerelease::new(format!("{head}.{tail}").as_str()).unwrap();
                } else {
                    version.pre = semver::Prerelease::new(format!("{pre}.0").as_str()).unwrap();
                }
            } else {
                version.major += 1;
                version.minor = 0;
                version.patch = 0;
                if let Some(pre) = pre {
                    version.pre = semver::Prerelease::new(format!("{pre}.0").as_str()).unwrap();
                } else {
                    version.pre = semver::Prerelease::new("").unwrap();
                }
            }
        }
        2 => {
            if version.patch == 0 && !version.pre.is_empty() && pre.is_some() {
                // XXX: assumes prerelease matches `^[\w\d]+\.\d+$`
                let (head, tail) = version.pre.split_once(".").unwrap();
                let pre = pre.unwrap();
                if pre == head {
                    let tail = tail.parse::<u8>().unwrap() + 1;
                    version.pre =
                        semver::Prerelease::new(format!("{head}.{tail}").as_str()).unwrap();
                } else {
                    version.pre = semver::Prerelease::new(format!("{pre}.0").as_str()).unwrap();
                }
            } else {
                version.minor += 1;
                version.patch = 0;
                if let Some(pre) = pre {
                    version.pre = semver::Prerelease::new(format!("{pre}.0").as_str()).unwrap();
                } else {
                    version.pre = semver::Prerelease::new("").unwrap();
                }
            }
        }
        1 => {
            if !version.pre.is_empty() && pre.is_some() {
                // XXX: assumes prerelease matches `^[\w\d]+\.\d+$`
                let (head, tail) = version.pre.split_once(".").unwrap();
                let pre = pre.unwrap();
                if pre == head {
                    let tail = tail.parse::<u8>().unwrap() + 1;
                    version.pre =
                        semver::Prerelease::new(format!("{head}.{tail}").as_str()).unwrap();
                } else {
                    version.pre = semver::Prerelease::new(format!("{pre}.0").as_str()).unwrap();
                }
            } else {
                version.patch += 1;
                if let Some(pre) = pre {
                    version.pre = semver::Prerelease::new(format!("{pre}.0").as_str()).unwrap();
                } else {
                    version.pre = semver::Prerelease::new("").unwrap();
                }
            }
        }
        _ => bail!("something has gone horribly wrong"),
    };
    let version = version.to_string();

    let mut markdown = changelog.markdown(version.clone());
    if !yes {
        if let Ok(editor) = env::var("EDITOR") {
            let mut tmpfile = env::temp_dir();
            tmpfile.push("CHANGESET_EDITMSG.md");
            let mut buffer = fs::File::create(&tmpfile)?;
            writeln!(buffer, "{}", markdown.trim())?;
            let status = process::Command::new(editor).arg(&tmpfile).status()?;
            markdown = "".to_string();
            fs::File::open(tmpfile)?.read_to_string(&mut markdown)?;

            if markdown.trim().is_empty() || !status.success() {
                println!(
                    "{}: no changelog found, aborting",
                    package.location.to_str().unwrap()
                );
                return Ok(());
            }
        } else {
            bail!("EDITOR must be set");
        }
    }

    let changelog = fs::read_to_string(path::PathBuf::from("CHANGELOG.md"))
        .unwrap_or("# Changelog\n\n".to_string());
    let (prologue, changelog) = changelog
        .split_once("##")
        .unwrap_or((changelog.as_str(), ""));
    let changelog = if changelog.is_empty() {
        changelog.to_string()
    } else {
        "\n## ".to_string() + changelog.trim() + "\n"
    };
    let mut f = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("CHANGELOG.md")?;
    f.write_all(prologue.as_bytes())?;
    f.write_all((markdown.trim().to_string() + "\n").as_bytes())?;
    f.write_all(changelog.as_bytes())?;
    f.flush()?;

    println!(
        "{}: {} -> {}",
        package.location.to_str().unwrap(),
        package.version.clone().unwrap(),
        version,
    );

    write_custom(package, version)?;

    for file in to_delete {
        fs::remove_file(file)?;
    }
    Ok(())
}

fn add_changeset(
    package: Option<String>,
    mut fpath: path::PathBuf,
    bump: SemverVersionAdd,
    change_type: Option<String>,
    message: Option<String>,
) -> Result<()> {
    if let Ok(editor) = env::var("EDITOR") {
        fpath.push(".nanpa");
        _ = fs::create_dir(fpath.clone());
        fpath.push(format!("{}.kdl", gen_changeset_name()));
        let mut buffer = fs::File::create(&fpath)?;

        let mut initial = match bump {
            SemverVersionAdd::Major => "major ",
            SemverVersionAdd::Minor => "minor ",
            SemverVersionAdd::Patch => "patch ",
        }
        .to_string();
        if let Some(package) = package {
            initial += format!("package=\"{}\" ", package).as_str();
        }
        initial += format!(
            "type=\"{}\" \"{}\"",
            change_type.unwrap_or_default(),
            message.unwrap_or_default()
        )
        .as_str();

        writeln!(buffer, "{}", initial)?;
        let status = process::Command::new(editor).arg(&fpath).status()?;
        let mut done = "".to_string();
        fs::File::open(fpath.clone())?.read_to_string(&mut done)?;

        if done.trim().is_empty() || !status.success() {
            println!("empty changeset, aborting",);
            fs::remove_file(fpath)?;
            return Ok(());
        }
    } else {
        bail!("EDITOR must be set");
    }

    Ok(())
}

struct Changelog {
    pub added: Vec<String>,
    pub changed: Vec<String>,
    pub deprecated: Vec<String>,
    pub removed: Vec<String>,
    pub fixed: Vec<String>,
    pub security: Vec<String>,
}

impl Changelog {
    pub fn new() -> Self {
        Self {
            added: vec![],
            changed: vec![],
            deprecated: vec![],
            removed: vec![],
            fixed: vec![],
            security: vec![],
        }
    }

    pub fn push(&mut self, change: KdlNode) -> Result<()> {
        if let Some(change_type) = change.get("type") {
            let change = change
                .get(0)
                .unwrap()
                .value()
                .as_string()
                .unwrap()
                .to_string();
            match change_type.value().as_string().unwrap() {
                "added" => self.added.push(change),
                "changed" => self.changed.push(change),
                "deprecated" => self.deprecated.push(change),
                "removed" => self.removed.push(change),
                "fixed" => self.fixed.push(change),
                "security" => self.security.push(change),
                unknown => bail!("unknown change type {unknown}"),
            };
        } else {
            bail!("change type (added, changed, etc.) must be specified")
        }

        Ok(())
    }

    pub fn markdown(&mut self, version: String) -> String {
        let mut ret = format!(
            "## [{}] - {}\n\n",
            version,
            chrono::Utc::now().format("%Y-%m-%d")
        );

        if !self.added.is_empty() {
            ret += "### Added\n\n";
            for item in self.added.clone() {
                ret += format!("- {item}\n").as_str();
            }
            ret += "\n";
        }
        if !self.changed.is_empty() {
            ret += "### Changed\n\n";
            for item in self.changed.clone() {
                ret += format!("- {item}\n").as_str();
            }
            ret += "\n";
        }
        if !self.deprecated.is_empty() {
            ret += "### Deprecated\n\n";
            for item in self.deprecated.clone() {
                ret += format!("- {item}\n").as_str();
            }
            ret += "\n";
        }
        if !self.removed.is_empty() {
            ret += "### Removed\n\n";
            for item in self.removed.clone() {
                ret += format!("- {item}\n").as_str();
            }
            ret += "\n";
        }
        if !self.fixed.is_empty() {
            ret += "### Fixed\n\n";
            for item in self.fixed.clone() {
                ret += format!("- {item}\n").as_str();
            }
            ret += "\n";
        }
        if !self.security.is_empty() {
            ret += "### Security\n\n";
            for item in self.security.clone() {
                ret += format!("- {item}\n").as_str();
            }
            ret += "\n";
        }

        ret
    }
}

fn run_language(package: package::Package, version: String) -> Result<()> {
    if let Some(language) = package.language {
        languages::run_language(language, version, package.location.clone())?;
    }

    Ok(())
}

fn gen_changeset_name() -> String {
    let words: Vec<&str> = include_str!("eff_short_wordlist.txt").lines().collect();

    let mut rng = rand::rng();
    let random_words: Vec<&str> = words.choose_multiple(&mut rng, 3).copied().collect();

    random_words.join("-")
}
