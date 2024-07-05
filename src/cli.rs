use clap::{Parser, Subcommand, Args};
use crate::nanpa;

#[derive(Parser, Debug)]
#[command(about, version, infer_subcommands = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Bump package version
    #[command(args_conflicts_with_subcommands = true)]
    Bump {
        #[command(subcommand)]
        semver_version: Option<SemverVersion>,

        #[command(flatten)]
        custom_version: Option<CustomVersion>,
    },
    /// Show current package version
    Version,
}


#[derive(Subcommand, Debug)]
pub enum SemverVersion {
    #[command(alias = "x")]
    Major,
    #[command(alias = "y")]
    Minor,
    #[command(alias = "z")]
    Patch,
    Prerelease(Prerelease)
}

#[derive(Args, Debug)]
struct Prerelease {
    version: String,
}

#[derive(Args, Debug)]
struct CustomVersion {
    version: String,
}

pub fn command() {
    let cli = Cli::parse();
    let nanpa = nanpa::new();

    match &cli.command {
        Commands::Bump { semver_version, custom_version } => {
            if let Some(version) = semver_version {
                nanpa.bump_semver(&version);
            } else {
                nanpa.bump_custom(custom_version.unwrap());
            }
        },
        Commands::Version => {
            nanpa.get_version();
        }
    }
}
