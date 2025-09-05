use anyhow::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Mutex;
use walkdir::WalkDir;

use crate::perf::Timer;
use crate::utils;

/// Convert the repository to text files with optimized performance
pub fn convert_repository_to_text(
    repo_path: &Path,
    output_path: &str,
    threshold_mb: f64,
    include_all: bool,
    debug: bool,
    profile: bool,
) -> Result<()> {
    let total_timer = if profile {
        Some(Timer::new("Total conversion"))
    } else {
        None
    };

    // Use buffered writer for better I/O performance
    let file = fs::File::create(output_path)?;
    let output_file = Mutex::new(BufWriter::new(file));
    let threshold_bytes = (threshold_mb * 1024.0 * 1024.0) as u64;

    // Optimized file discovery with pre-filtering
    let discovery_timer = if profile {
        Some(Timer::new("File discovery"))
    } else {
        None
    };

    let files: Vec<_> = WalkDir::new(repo_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| !utils::should_exclude_file(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect();

    if let Some(timer) = discovery_timer {
        timer.print_elapsed();
    }

    if files.is_empty() {
        println!("No files found to process");
        return Ok(());
    }

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Counters for statistics
    let processed_files = Mutex::new(0u64);
    let skipped_files = Mutex::new(0u64);

    // Parallel processing for maximum performance
    let processing_timer = if profile {
        Some(Timer::new("File processing"))
    } else {
        None
    };

    files.par_iter().try_for_each(|file_path| -> Result<()> {
        let metadata = fs::metadata(file_path)?;
        let file_size = metadata.len();
        let relative_path = file_path.strip_prefix(repo_path)?;

        pb.set_message(format!("Processing: {}", relative_path.display()));

        // Check if file should be included
        let should_skip = if !include_all {
            // Skip binary files (optimized check)
            if utils::is_binary_file(file_path)? {
                if debug {
                    println!("Skipping binary file: {}", relative_path.display());
                }
                true
            }
            // Skip files larger than threshold
            else if file_size > threshold_bytes {
                if debug {
                    println!(
                        "Skipping large file: {} ({})",
                        relative_path.display(),
                        utils::format_file_size(file_size)
                    );
                }
                true
            } else {
                false
            }
        } else {
            false
        };

        if should_skip {
            *skipped_files.lock().unwrap() += 1;
            pb.inc(1);
            return Ok(());
        }

        // Process and write file content
        let mut content = String::new();

        // Build content string first
        content.push_str(&"=".repeat(80));
        content.push('\n');
        content.push_str(&format!("File: {}\n", relative_path.display()));
        content.push_str(&format!("Size: {}\n", utils::format_file_size(file_size)));
        content.push_str(&"=".repeat(80));
        content.push_str("\n\n");

        // Read file content efficiently
        match utils::read_file_content(file_path, file_size) {
            Ok(file_contents) => {
                content.push_str(&file_contents);
            }
            Err(_) => {
                content.push_str("[Binary file or read error]");
                if debug {
                    println!("Could not read file as text: {}", relative_path.display());
                }
            }
        }
        content.push_str("\n\n");

        // Write to output file (synchronized)
        {
            let mut writer = output_file.lock().unwrap();
            writer.write_all(content.as_bytes())?;
        }

        *processed_files.lock().unwrap() += 1;
        pb.inc(1);
        Ok(())
    })?;

    if let Some(timer) = processing_timer {
        timer.print_elapsed();
    }

    // Ensure all data is written
    let flush_timer = if profile {
        Some(Timer::new("File flush"))
    } else {
        None
    };

    {
        let mut writer = output_file.lock().unwrap();
        writer.flush()?;
    }

    if let Some(timer) = flush_timer {
        timer.print_elapsed();
    }

    pb.finish_with_message("Conversion complete!");

    let processed_count = *processed_files.lock().unwrap();
    let skipped_count = *skipped_files.lock().unwrap();

    println!(
        "📊 Processed {} files, skipped {} files",
        style(processed_count.to_string()).green(),
        style(skipped_count.to_string()).yellow()
    );

    if let Some(timer) = total_timer {
        timer.print_elapsed();
    }

    Ok(())
}
