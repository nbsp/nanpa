mod cli;
mod languages;
mod nanpa;
mod package;

use colored::Colorize;

fn main() {
    if let Err(e) = cli::command() {
        eprintln!("{} {}", "error:".red().bold(), e);
    };
}
