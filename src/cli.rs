use crate::nanpa;
use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use colored::Colorize;

#[derive(Parser)]
#[command(about, version, infer_subcommands = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show current package version
    Version,
    /// Bump package version from changesets and generate changelog entry
    Changeset {
        package: Option<String>,

        /// Tag version as a prerelease
        #[arg(long, value_name = "TYPE")]
        pre: Option<String>,

        /// Accept changeset without passing through editor
        #[arg(short)]
        yes: bool,
    },
    /// Add a changeset
    Add {
        #[arg(value_enum)]
        bump: SemverVersionAdd,

        #[arg(long, short)]
        package: Option<String>,

        #[arg(id = "type", long, short, value_name = "TYPE")]
        change_type: Option<String>,

        #[arg(long, short)]
        message: Option<String>,
    },
    /// Manually bump package version
    #[command(args_conflicts_with_subcommands = true)]
    Bump {
        #[command(subcommand)]
        semver_version: Option<SemverVersion>,

        #[command(flatten)]
        custom_version: Option<CustomVersion>,

        package: Option<String>,
    },
    /// List supported languages
    ListLanguages,
}

#[derive(Subcommand)]
pub enum SemverVersion {
    #[command(alias = "x")]
    Major,
    #[command(alias = "y")]
    Minor,
    #[command(alias = "z")]
    Patch,
    Prerelease(Prerelease),
}

#[derive(ValueEnum, Clone)]
pub enum SemverVersionAdd {
    #[value(alias = "x")]
    Major,
    #[value(alias = "y")]
    Minor,
    #[value(alias = "z")]
    Patch,
}

#[derive(Args)]
pub struct Prerelease {
    pub version: String,
}

#[derive(Args)]
struct CustomVersion {
    pub version: String,
}

pub fn command() -> Result<()> {
    let cli = Cli::parse();
    let nanpa = nanpa::new()?;

    match &cli.command {
        Commands::Bump {
            semver_version,
            custom_version,
            package,
        } => {
            if let Some(version) = semver_version {
                nanpa.bump_semver(version, package.clone())?;
            } else {
                nanpa.bump_custom(
                    custom_version.as_ref().unwrap().version.clone(),
                    package.clone(),
                )?;
            }
        }
        Commands::Version => {
            let versions = nanpa.packages();
            for (location, package) in versions {
                println!("{}: {}", location, package.version.unwrap());
            }
        }
        Commands::Changeset { package, pre, yes } => {
            nanpa.changesets(package.clone(), pre.clone(), yes.clone())?
        }
        Commands::Add {
            bump,
            package,
            change_type,
            message,
        } => {
            nanpa.add(
                package.clone(),
                bump.clone(),
                change_type.clone(),
                message.clone(),
            )?;
        }
        Commands::ListLanguages => {
            println!("{}", "Supported languages:".bold().underline());
            println!(
                "Rust Cargo\t {} (or {})",
                "cargo".yellow().bold(),
                "rust".yellow()
            );
            println!(
                "Node.js\t\t {} (or {}, {}, {}, {})",
                "node".yellow().bold(),
                "javascript".yellow(),
                "js".yellow(),
                "typescript".yellow(),
                "ts".yellow(),
            );
        }
    }

    Ok(())
}
