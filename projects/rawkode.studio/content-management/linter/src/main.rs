use crate::cli::Cli;
use clap::Parser;
use format::{
    Episodes, MinimalEpisodes, MinimalPeople, MinimalShows, MinimalTechnologies, People, Shows,
    Technologies,
};
use glob::glob;
use hcl::from_str;
use miette::{miette, IntoDiagnostic, Result};

use std::fs;
use std::path::PathBuf;

mod cli;
mod format;

fn get_paths(path: PathBuf) -> Vec<PathBuf> {
    if path.extension().map(|ext| ext == "hcl").unwrap_or(false) {
        vec![path]
    } else {
        glob(&format!("{}/**/*.hcl", path.display()))
            .into_diagnostic()
            .unwrap()
            .flatten()
            .collect::<Vec<PathBuf>>()
    }
}

fn try_check(content: &str) -> Result<()> {
    if from_str::<MinimalEpisodes>(content).is_ok() {
        hcl::from_str::<Episodes>(content)
            .into_diagnostic()
            .map(|_| ())
    } else if from_str::<MinimalShows>(content).is_ok() {
        hcl::from_str::<Shows>(content)
            .into_diagnostic()
            .map(|_| ())
    } else if from_str::<MinimalTechnologies>(content).is_ok() {
        hcl::from_str::<Technologies>(content)
            .into_diagnostic()
            .map(|_| ())
    } else if from_str::<MinimalPeople>(content).is_ok() {
        hcl::from_str::<People>(content)
            .into_diagnostic()
            .map(|_| ())
    } else {
        Err(miette!("Format of file is not supported"))
    }
}

fn try_parse(content: &str) -> Result<String> {
    if let Ok(content) = from_str::<Episodes>(content) {
        hcl::to_string(&content).into_diagnostic()
    } else if let Ok(content) = from_str::<Shows>(content) {
        hcl::to_string(&content).into_diagnostic()
    } else if let Ok(content) = from_str::<Technologies>(content) {
        hcl::to_string(&content).into_diagnostic()
    } else if let Ok(content) = from_str::<People>(content) {
        hcl::to_string(&content).into_diagnostic()
    } else {
        Err(miette!("Format of cone file is not supported"))
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.subcommand {
        cli::SubCommand::Format(cli::Format { path, apply }) => {
            let files = get_paths(path);

            println!("Formatting {} files", files.len());
            println!();

            for file in files {
                if let Ok(content) = fs::read_to_string(&file) {
                    if let Ok(formatted_content) = try_parse(&content) {
                        if apply {
                            fs::write(&file, formatted_content).into_diagnostic()?;
                            println!("{} - OK", file.display())
                        } else {
                            println!("{} - DRY RUN", file.display())
                        }
                    } else {
                        eprintln!("{} - NOT OK", file.display());
                    }
                }
            }
        }
        cli::SubCommand::Lint(cli::Lint { path }) => {
            let files = get_paths(path);

            println!("Linting {} files", files.len());
            println!();

            let mut errors = false;

            for file in files {
                if let Ok(content) = fs::read_to_string(&file) {
                    match try_check(&content) {
                        Ok(_) => println!("{} - OK", file.display()),
                        Err(err) => {
                            // only parsing errors count towards exit code = 1
                            errors = true;

                            eprintln!("{} - NOT OK\n {:?}", file.display(), err)
                        }
                    }
                } else {
                    eprintln!("{} - NOT OK", file.display());
                }
            }

            if errors {
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
