#![allow(dead_code)]
use std::fs;
use anyhow::anyhow;
use thiserror::Error;
use clap::{Parser, Subcommand};

mod catfile;

#[derive(Error, Debug, PartialEq, Eq)]
enum Error {
    #[error("unknown command: {0}")]
    UnknownCommand(String),
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    CatFile {
        #[arg(short = 'p')]
        pretty: Option<bool>,
        hash: String,
    },
    HashObject {
        #[arg(short, long)]
        write: bool,
        path: String,
    },
}


fn init() -> anyhow::Result<()> {
    fs::create_dir(".git")?;
    fs::create_dir(".git/objects")?;
    fs::create_dir(".git/refs")?;
    fs::write(".git/HEAD", "ref: refs/heads/main\n")?;
    println!("Initialized git directory");
    Ok(())
}

fn unknown_command(command: &str) -> anyhow::Result<()> {
    Err(anyhow!(Error::UnknownCommand(command.to_string())))
}

fn main() {
    let cli = Cli::parse();

    if let Err(err) = match &cli.command {
        Commands::Init => init(),
        Commands::CatFile{ hash, .. } => catfile::cat_file(hash),
        Commands::HashObject{ path, .. } => catfile::hash_object(path)
    } {
        println!("{err}");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_unknown_command() {
        let command = "unknown-command";
        let expected_error = Error::UnknownCommand(command.to_string());

        let actual_result = unknown_command(command);
        if let Err(actual_error) = actual_result {
            if let Ok(err) = actual_error.downcast::<Error>() {
                assert_eq!(err, expected_error);
                return;
            }
        }
        panic!();
    }
}

