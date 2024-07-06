use crate::nanpa;
use anyhow::Result;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(about, version, infer_subcommands = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Bump package version
    #[command(args_conflicts_with_subcommands = true)]
    Bump {
        #[command(subcommand)]
        semver_version: Option<SemverVersion>,

        #[command(flatten)]
        custom_version: Option<CustomVersion>,

        package: Option<String>,
    },
    /// Show current package version
    Version,
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
    }

    Ok(())
}
