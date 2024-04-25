# DFS Auditing Tool

## Installation

You can either download the latest release from the realeases page, or you can build it from source (recommended).
## Dependancies
<details>
    <summary> <h3>Windows Dependancies</h3></summary>

See [RustLang's site](https://www.rust-lang.org/tools/install) for the windows install exe

**Install Winget**

```PowerShell
# Dep to install Chocolatey
$progressPreference = 'silentlyContinue'
Write-Information "Downloading WinGet and its dependencies..."
Invoke-WebRequest -Uri https://aka.ms/getwinget -OutFile Microsoft.DesktopAppInstaller_8wekyb3d8bbwe.msixbundle
Invoke-WebRequest -Uri https://aka.ms/Microsoft.VCLibs.x64.14.00.Desktop.appx -OutFile Microsoft.VCLibs.x64.14.00.Desktop.appx
Invoke-WebRequest -Uri https://github.com/microsoft/microsoft-ui-xaml/releases/download/v2.8.6/Microsoft.UI.Xaml.2.8.x64.appx -OutFile Microsoft.UI.Xaml.2.8.x64.appx
Add-AppxPackage Microsoft.VCLibs.x64.14.00.Desktop.appx
Add-AppxPackage Microsoft.UI.Xaml.2.8.x64.appx
Add-AppxPackage Microsoft.DesktopAppInstaller_8wekyb3d8bbwe.msixbundle
```

Open New terminal

**Install Chocolatey**


```PowerShell
# Dep to install mingw
winget install chocolatey
# Install git dependencies
winget install git
```

Open New Terminal


```PowerShell
# Installs ncessary build keychains etc.
choco install mingw
```

</details>

<details>
    <summary> <h3>Mac Dependancies</h3></summary>

**Installing Homebrew**


```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```
Restart terminal emulator.

**Install Deps**

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
brew install git
# If cross-compiling for Windows

# Install windows requirements on mac
brew install mingw-w64
```

**cross-compiling**

```bash
git clone https://github.com/CarnegieKirk/dfs_usage_audit.git
cd dfs_usage_audit 
# Build Both
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-pc-windows-gnu
# File system should look like this now
target
├── aarch64-apple-darwin
│  ├── CACHEDIR.TAG
│  └── release 
│     └── dfs_usage_audit <--- Your executable is here (Mac)
├── debug
│  ├── build
│  ├── deps
│  ├── examples
│  └── incremental
└── x86_64-pc-windows-gnu
   ├── CACHEDIR.TAG
   └── release
      └── dfs_usage_audit.exe <--- Your executable is here (Windows)
```

</details>

## Compilation

<details>
    <summary> <h3>Building</h3></summary>
Please see the dependancies section above for OS-specific requirements.

```bash
git clone https://github.com/CarnegieKirk/dfs_usage_audit.git
cd dfs_usage_audit 
cargo build --release
# File structure will look liek this:
target
├── release
│  ├── build
│  ├── deps
│  ├── dfs_usage_audit <--- This is your executable. For windows it will have the .exe extension
│  ├── dfs_usage_audit.d
│  ├── examples
│  └── incremental
```
</details>

## Usage

```bash
Usage: dfs_usage_audit [OPTIONS] --path <PATH>

Options:
  -o, --out-file <OUT_FILE>  Relative path to export audit results. [default: DFS_audit.csv]
  -p, --path <PATH>          The path to the directory/DFS space you wish to audit.
  -d, --directories          Whether to include only directories. Does not take a value
  -t, --threads <THREADS>    The amount of threads to use for performance.Higher is faster, but can be less accurate. I've found 50 to be the best tradeoff. Never inaccurate. [default: 50]
  -d, --days <DAYS>          Cut-off, in days, to include. e.g. 365 to show files not accessed in the last year. [default: 1095]
  -h, --help                 Print help
  -V, --version              Print version
```
