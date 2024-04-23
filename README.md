# DFS Auditing Tool

## Installation

You can either download the latest release from the realeases page, or you can build it from source (recommended).

***Note: For Windows***
See [RustLang's site](https://www.rust-lang.org/tools/install) for the windows exe
Other oses, see below:

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Requires Git
git clone https://github.com/HirschyKirkwood-Work/dfs_usage_audit.git
cd dfs_usage_audit 
# Requires installation of rust from https://www.rust-lang.org/tools/install
cargo build --release
# For building cross platform
# Install windows requirements on mac
brew install mingw-w64
# Unfortunately, Apple sucks so installing a mac compiler for Windows ain't happening.
# Build Both
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-pc-windows-gnu
```

## Usage
```
target/aarch64-apple-darwin/release/dfs_usage_audit --help
Usage: dfs_usage_audit [OPTIONS] --path <PATH>

Options:
  -o, --out-file <OUT_FILE>  [default: DFS_audit.csv]
  -p, --path <PATH>
  -t, --threads <THREADS>    [default: 50]
  -d, --days <DAYS>          [default: 1095]
  -h, --help                 Print help
  -V, --version              Print version
```
