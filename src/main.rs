use anyhow::{Result, anyhow};
use clap::Parser;
use console::style;
use std::env;
use std::path::PathBuf;

mod core;
mod utils;

#[derive(Parser)]
#[command(
    name = "textify",
    about = "Convert local Git repository to text files",
    version
)]

struct Args {
    /// Path to the repository (defaults to current directory)
    #[arg(default_value = ".")]
    path: String,

    /// Output file path
    #[arg(short, long, default_value = "")]
    output: String,

    /// File size threshold in MB (files larger than this will be excluded)
    #[arg(short, long, default_value = "0.1")]
    threshold: f64,

    /// Include all files regardless of size or type
    #[arg(long)]
    include_all: bool,

    /// Enable debug mode with verbose logging
    #[arg(long)]
    debug: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.debug {
        println!("{}", style("Debug mode enabled").cyan());
    }

    let repo_path = PathBuf::from(&args.path);
    let repo_path = if repo_path.is_absolute() {
        repo_path
    } else {
        env::current_dir()?.join(repo_path)
    };

    if !repo_path.exists() {
        return Err(anyhow!("Path does not exist: {}", repo_path.display()));
    }

    if !repo_path.is_dir() {
        return Err(anyhow!("Path is not a directory: {}", repo_path.display()));
    }

    let repo_name = utils::get_repo_name(&repo_path)?;

    let output_path = if args.output.is_empty() {
        format!(".textify.txt")
    } else {
        args.output
    };

    if args.debug {
        println!("Repository path: {}", repo_path.display());
        println!("Repository name: {}", repo_name);
        println!("Output file: {}", output_path);
    }

    println!(
        "📂 {}",
        style(format!("Processing repository: {}", repo_name)).green()
    );

    core::convert_repository_to_text(
        &repo_path,
        &output_path,
        args.threshold,
        args.include_all,
        args.debug,
    )?;

    println!(
        "✅ {}",
        style(format!(
            "Repository converted successfully to: {}",
            output_path
        ))
        .green()
        .bold()
    );

    Ok(())
}
