```markdown
# textify

## Overview
**textify** is a command-line tool designed to convert a local Git repository into a text file that contains the contents of the files within the repository. This utility allows developers to easily view file content without manually browsing through the repository structure.

## Features
- Convert local Git repositories to a structured text file.
- Exclude large, binary, or specified files from the output.
- Customizable file size threshold to exclude large files.
- Progress tracking during the conversion process.

## Installation
To install the necessary dependencies, run:
```bash
cargo build --release
```
To install the tool globally, use:
```bash
cargo install --path .
```

## Usage
You can run the application from the command line as follows:

```bash
textify -- [OPTIONS]
```

### Options
- `--path <PATH>`: Path to the repository (defaults to the current directory).
- `--output <OUTPUT>`: Specify the output file path.
- `--threshold <THRESHOLD>`: Set the file size threshold in MB (default is 0.1 MB).
- `--include-all`: Include all files regardless of size or type.
- `--debug`: Enable debug mode with verbose logging.

### Example
```bash
cargo run -- --path /path/to/repo --output output.txt --threshold 1.0
```

## Contributing
Contributions are welcome! Please feel free to submit a pull request or raise an issue.

