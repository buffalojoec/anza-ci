//! Anza Program CI.

mod file;
mod parser;
mod toml;

use {
    crate::{file::find_anza_toml, toml::Toml},
    clap::{Parser, Subcommand},
    std::process::Command,
};

#[derive(Parser)]
#[command(name = "anza-ci")]
#[command(about = "A CLI for program repository CI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Cargo {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command {
        Commands::Cargo { args } => {
            if args.is_empty() {
                return Err("Expected at least one argument".into());
            }

            if let Some(anza_toml_path) = find_anza_toml()? {
                let toml_contents = std::fs::read_to_string(anza_toml_path)?;
                let parsed_toml = Toml::parse(&toml_contents)?;

                if let Some(mut alias) = parsed_toml.compile_alias_command(&args[0]) {
                    alias.status()?;
                }
            } else {
                Command::new("cargo").args(args).status()?;
            }

            Ok(())
        }
    }
}
