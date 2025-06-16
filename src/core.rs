use utils::{format_use anyhow::{Result, anyhow};
use clap::Parser;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;file_size, get_repo_name, is_binary_file, should_exclude_file};

/// Convert the repository to text files
pub fn convert_repository_to_text(
    repo_path: &Path,
    output_path: &str,
    threshold_mb: f64,
    include_all: bool,
    debug: bool,
) -> Result<()> {
    let mut output_file = fs::File::create(output_path)?;
    let threshold_bytes = (threshold_mb * 1024.0 * 1024.0) as u64;

    // Collect all files first to show progress
    let mut files = Vec::new();
    for entry in WalkDir::new(repo_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        // Skip excluded directories and files
        if should_exclude_file(path) {
            if debug {
                println!("Skipping excluded file: {}", path.display());
            }
            continue;
        }

        files.push(path.to_path_buf());
    }

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut processed_files = 0;
    let mut skipped_files = 0;

    for file_path in files {
        let metadata = fs::metadata(&file_path)?;
        let file_size = metadata.len();
        let relative_path = file_path.strip_prefix(repo_path)?;

        pb.set_message(format!("Processing: {}", relative_path.display()));

        // Check if file should be included
        if !include_all {
            // Skip binary files
            if is_binary_file(&file_path)? {
                if debug {
                    println!("Skipping binary file: {}", relative_path.display());
                }
                skipped_files += 1;
                pb.inc(1);
                continue;
            }

            // Skip files larger than threshold
            if file_size > threshold_bytes {
                if debug {
                    println!(
                        "Skipping large file: {} ({})",
                        relative_path.display(),
                        format_file_size(file_size)
                    );
                }
                skipped_files += 1;
                pb.inc(1);
                continue;
            }
        }

        // Write file header section
        writeln!(output_file, "{}", "=".repeat(80))?;
        writeln!(output_file, "File: {}", relative_path.display())?;
        writeln!(output_file, "Size: {}", format_file_size(file_size))?;
        writeln!(output_file, "{}", "=".repeat(80))?;
        writeln!(output_file)?;

        // Write file actual contents
        match fs::read_to_string(&file_path) {
            Ok(contents) => {
                writeln!(output_file, "{}", contents)?;
            }
            Err(_) => {
                writeln!(output_file, "[Binary file or read error]")?;
                if debug {
                    println!("Could not read file as text: {}", relative_path.display());
                }
            }
        }

        writeln!(output_file)?;
        processed_files += 1;
        pb.inc(1);
    }

    pb.finish_with_message("Conversion complete!");

    println!(
        "📊 Processed {} files, skipped {} files",
        style(processed_files.to_string()).green(),
        style(skipped_files.to_string()).yellow()
    );

    Ok(())
}
