# egit
![GitHub Release](https://img.shields.io/github/release/EdwardJoke/egit.svg) ![License](https://img.shields.io/badge/license-MIT%20OR%20Apache%202.0-blue.svg) ![Static Badge](https://img.shields.io/badge/Build-Passing-green) 
A fast and efficient command-line tool for downloading GitHub releases and source code with support for parallel downloads.

## Features

- **Fast Downloads**: Optimized streaming algorithm with support for parallel downloads using multiple threads
- **Parallel Downloads**: Enable multithreaded downloads with configurable thread count
- **Real-time Progress**: Individual progress bars for each download thread with detailed statistics
- **Cross-platform**: Works seamlessly on Windows, macOS, and Linux
- **Easy to Use**: Simple command-line interface with intuitive syntax
- **Memory Efficient**: Streaming downloads reduce memory usage for large files
- **Automatic Format Selection**: Downloads appropriate file format based on your operating system

## Installation

### Prerequisites

- Rust 1.70 or higher

### Build from Source

```bash
git clone https://github.com/yourusername/egit.git
cd egit
cargo build --release
```

The binary will be available at `target/release/egit`.

## Usage

### Basic Usage

Download the latest release from a GitHub repository:

```bash
egit download owner/repo
```

Download a specific version:

```bash
egit download owner/repo@v1.0.0
```

### Download Source Code

Download source code instead of binary releases:

```bash
egit download owner/repo --source
```

### Parallel Downloads

Enable multithreaded downloads with default thread count (4):

```bash
egit download owner/repo --multithread
```

Specify custom thread count:

```bash
egit download owner/repo --multithread --threads 8
```

## Command Reference

### `download` Command

```
egit download [OPTIONS] <PACKAGE>
```

**Arguments**:
- `<PACKAGE>`: GitHub repository in format `owner/repo` or `owner/repo@version`

**Options**:
- `-s, --source`: Download source code instead of binary
- `--multithread`: Enable multithreaded parallel downloads
- `--threads <THREADS>`: Number of threads to use for parallel downloads [default: 4]
- `-h, --help`: Print help information

## How It Works

- **Single-threaded Mode**: Uses streaming downloads to efficiently download files without loading them entirely into memory
- **Parallel Mode**: Splits files into chunks and downloads them concurrently using multiple threads, each with its own progress bar
- **Progress Tracking**: Provides real-time statistics including download speed, elapsed time, and estimated time remaining
- **Format Detection**: Automatically downloads .zip files for Windows and .tar.gz files for Unix-based systems

## Performance

- **Memory Usage**: Uses constant memory regardless of file size due to streaming downloads
- **Download Speed**: Parallel downloads can significantly improve speed, especially for large files and high-latency connections
- **Reliability**: Built with robust error handling and retry mechanisms

## License

MIT or APACHE 2.0