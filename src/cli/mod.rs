use crate::core::archive::inspect_archive;
use crate::core::import::import_archive;
use crate::core::models::{ImportRequest, PackageRequest};
use crate::core::pack::package_repository;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(name = "lfs_bundle")]
#[command(about = "Package and import git bundle/LFS archives")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Gui,
    Pack {
        repo: PathBuf,
        from: String,
        to: String,
        output: PathBuf,
        #[arg(long)]
        direct: bool,
    },
    Import {
        repo: PathBuf,
        branch: String,
        archive: PathBuf,
        #[arg(long)]
        direct: bool,
    },
    Inspect {
        archive: PathBuf,
    },
}

pub fn run() -> ExitCode {
    match try_run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err:#}");
            ExitCode::FAILURE
        }
    }
}

fn try_run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Gui) | None => crate::gui::launch(),
        Some(Commands::Pack { repo, from, to, output, direct }) => {
            let summary = package_repository(&PackageRequest {
                repo_path: repo,
                start_commit: from,
                end_commit: to,
                output_archive: output,
                safe_mode: !direct,
            })?;
            println!(
                "packaged {} commits and {} lfs objects",
                summary.commit_count, summary.lfs_object_count
            );
            Ok(())
        }
        Some(Commands::Import { repo, branch, archive, direct }) => {
            import_archive(&ImportRequest {
                repo_path: repo,
                branch,
                archive_path: archive,
                safe_mode: !direct,
            })?;
            println!("import completed successfully");
            Ok(())
        }
        Some(Commands::Inspect { archive }) => {
            let manifest = inspect_archive(&archive)?;
            println!("start_commit: {}", manifest.start_commit);
            println!("end_commit: {}", manifest.end_commit);
            println!("target_commit: {}", manifest.target_commit);
            Ok(())
        }
    }
}
