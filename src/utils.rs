use anyhow::{Result, anyhow};
use std::fs;
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
            // @DEV: (just in case) Git command failed (not a git repo or other error)
            // noop is fine for now
        }
        Err(_) => {
            // @DEV: (just in case) Git command not found or other execution error
            // noop is fine for now
        }
    }

    // why?: sometimes git --show-toplevel fails, so we fallback to the directory name
    return repo_path
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

/// Check if a file not empty or does not exist
/// Returns true if the file is empty or does not exis
pub fn is_file_empty_or_nonexistent(path: &Path) -> bool {
    if !path.exists() {
        return true; // File does not exist
    }

    match fs::metadata(path) {
        Ok(metadata) => metadata.len() == 0, // Check if file size is 0
        Err(_) => true,                      // If we can't read metadata, consider it empty
    }
}

/// Check if a file should be excluded based on its path
pub fn should_exclude_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();

    // Exclude common directories that shouldn't be processed
    let excluded_dirs = [
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
    ];

    // Check if any part of the path contains excluded directories
    for excluded in &excluded_dirs {
        if path_str.contains(&format!("/{}/", excluded))
            || path_str.starts_with(&format!("{}/", excluded))
            || path_str.contains(&format!("\\{}\\", excluded))
            || path_str.starts_with(&format!("{}\\", excluded))
        {
            return true;
        }
    }

    // Exclude specific file patterns
    let excluded_patterns = [
        ".DS_Store",
        "Thumbs.db",
        "Desktop.ini",
        ".gitignore",
        ".gitkeep",
        ".dockerignore",
        "Dockerfile",
        "Cargo.lock",
        "package-lock.json",
        "yarn.lock",
        "pnpm-lock.yaml",
        "composer.lock",
        "go.sum",
    ];

    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    for pattern in &excluded_patterns {
        // @DEV: rm
        // println!("Skipping skip: {}, filename: {}, pattern: {},", filename == *pattern, filename, pattern);

        if filename == *pattern {
            return true;
        }
    }

    if is_file_empty_or_nonexistent(path) {
        return true; // Exclude empty or non-existent files
    }

    return false;
}

/// Check if a file is likely binary by reading its first few bytes
pub fn is_binary_file(path: &Path) -> Result<bool> {
    // First check by file extension
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        let binary_extensions = [
            // Images
            "jpg", "jpeg", "png", "gif", "bmp", "tiff", "ico", "svg", "webp", // Videos
            "mp4", "avi", "mov", "wmv", "flv", "webm", "mkv", // Audio
            "mp3", "wav", "flac", "aac", "ogg", "wma", // Documents
            "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", // Archives
            "zip", "tar", "gz", "bz2", "7z", "rar", "dmg", // Executables
            "exe", "dll", "so", "dylib", "bin", // Fonts
            "ttf", "otf", "woff", "woff2", "eot", // Other binary formats
            "db", "sqlite", "sqlite3",
        ];

        let ext_lower = extension.to_lowercase();
        if binary_extensions.contains(&ext_lower.as_str()) {
            return Ok(true);
        }
    }

    // Check file contents for null bytes (common in binary files)
    const SAMPLE_SIZE: usize = 8192; // Read first 8KB
    match fs::read(path) {
        Ok(bytes) => {
            let sample_size = std::cmp::min(bytes.len(), SAMPLE_SIZE);
            let sample = &bytes[..sample_size];

            // Check for null bytes
            if sample.contains(&0) {
                return Ok(true);
            }

            // Check for high percentage of non-printable characters
            let non_printable_count = sample
                .iter()
                .filter(|&&b| b < 32 && b != 9 && b != 10 && b != 13) // Exclude tab, LF, CR
                .count();

            let non_printable_ratio = non_printable_count as f64 / sample.len() as f64;
            Ok(non_printable_ratio > 0.3) // If more than 30% non-printable, consider binary
        }
        Err(_) => Ok(false), // If we can't read it, assume it's not binary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
        assert_eq!(format_file_size(1073741824), "1.0 GB");
    }

    #[test]
    fn test_should_exclude_file() {
        assert!(should_exclude_file(&PathBuf::from(
            "node_modules/package/file.js"
        )));
        assert!(should_exclude_file(&PathBuf::from(".git/config")));
        assert!(should_exclude_file(&PathBuf::from("src/target/debug/main")));
        assert!(should_exclude_file(&PathBuf::from(".DS_Store")));
        assert!(!should_exclude_file(&PathBuf::from("src/main.rs")));
        assert!(!should_exclude_file(&PathBuf::from("README.md")));
    }
}
