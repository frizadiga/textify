use anyhow::{Result, anyhow};
use memmap2::Mmap;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::Command;

/// Get the name of the repository using the git command
pub fn get_repo_name(repo_path: &Path) -> Result<String> {
    match Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(repo_path)
        .output()
    {
        Ok(output) if output.status.success() => {
            let git_root = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let git_root_path = Path::new(&git_root);

            if let Some(name) = git_root_path.file_name().and_then(|n| n.to_str()) {
                return Ok(name.to_string());
            }
        }
        Ok(_) => {
            // Git command failed (not a git repo or other error)
        }
        Err(_) => {
            // Git command not found or other execution error
        }
    }

    // Fallback to directory name
    repo_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Could not determine repository directory name"))
}

/// Format file size in human-readable format
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Check if a file is empty or does not exist
// pub fn is_file_empty_or_nonexistent(path: &Path) -> bool {
//     if !path.exists() {
//         return true;
//     }
//
//     match fs::metadata(path) {
//         Ok(metadata) => metadata.len() == 0,
//         Err(_) => true,
//     }
// }

/// Optimized binary file detection - checks only first 512 bytes
pub fn is_binary_file(path: &Path) -> Result<bool> {
    let mut file = fs::File::open(path)?;
    let mut buffer = [0; 512];
    let bytes_read = file.read(&mut buffer)?;

    // Check for null bytes (common indicator of binary files)
    Ok(buffer[..bytes_read].contains(&0))
}

/// Efficiently read file content based on size
pub fn read_file_content(path: &Path, file_size: u64) -> Result<String> {
    // For small files, use regular read
    if file_size < 1024 * 1024 {
        // < 1MB
        return Ok(fs::read_to_string(path)?);
    }

    // For larger files, use memory mapping for better performance
    let file = fs::File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };

    // Convert to string, handling potential UTF-8 errors
    match std::str::from_utf8(&mmap) {
        Ok(content) => Ok(content.to_string()),
        Err(_) => Ok(String::from_utf8_lossy(&mmap).to_string()),
    }
}

/// Check if a file should be excluded based on its path
pub fn should_exclude_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();

    // Exclude common directories that shouldn't be processed
    const EXCLUDED_DIRS: &[&str] = &[
        "node_modules",
        ".git",
        ".svn",
        ".hg",
        "target",
        "build",
        "dist",
        ".vscode",
        ".idea",
        "__pycache__",
        ".pytest_cache",
        "coverage",
        ".nyc_output",
        "vendor",
        "deps",
        "cmake-build-debug",
        "cmake-build-release",
    ];

    // Check if any part of the path contains excluded directories
    for excluded in EXCLUDED_DIRS {
        if path_str.contains(&format!("/{}/", excluded))
            || path_str.starts_with(&format!("{}/", excluded))
            || path_str.contains(&format!("\\{}\\", excluded))
            || path_str.starts_with(&format!("{}\\", excluded))
        {
            return true;
        }
    }

    // Exclude common binary file extensions
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        const BINARY_EXTENSIONS: &[&str] = &[
            "exe", "dll", "so", "dylib", "bin", "obj", "o", "a", "lib", "jpg", "jpeg", "png",
            "gif", "bmp", "ico", "svg", "mp3", "mp4", "avi", "mov", "wav", "flac", "zip", "tar",
            "gz", "rar", "7z", "bz2", "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
        ];

        if BINARY_EXTENSIONS.contains(&extension.to_lowercase().as_str()) {
            return true;
        }
    }

    false
}
