use std::{env, path::PathBuf};
use colored::Colorize;

pub fn find_root() -> Option<PathBuf> {
    let mut dir = env::current_dir().unwrap();
    if dir.join(".nanparc").exists() {
        Some(dir)
    } else {
        eprintln!("current directory does not contain .nanparc, searching up");
        loop {
            if dir == PathBuf::from("/") {
                return None;
            }
            dir.pop();
            if dir.join(".nanparc").exists() {
                return Some(dir);
            }
        }
    }
}

pub fn error(s: &str) {
    eprintln!("{} {}", "error:".red().bold(), s);
}
