# 🧹 Duster

<p align="center">
  <img src="https://img.shields.io/github/license/lll82012/duster" alt="License">
  <img src="https://img.shields.io/github/actions/workflow/status/lll82012/duster/ci.yml" alt="CI">
  <img src="https://img.shields.io/crates/v/duster" alt="Crates.io">
</p>

<p align="center"><b>A blazingly fast disk usage analyzer with beautiful terminal output.</b></p>

Duster scans your directories and instantly shows you what's taking up space — with colored bars, tree views, and percentages. Think `du` + `tree`, but prettier and faster.

## ✨ Features

- **⚡ Blazingly fast** — powered by `walkdir`, scans thousands of files per second
- **🎨 Beautiful output** — colored bars, tree view, sizes, and percentages at a glance
- **🌳 Tree display** — see exactly which subdirectories are eating your disk
- **📊 Visual bars** — relative bar chart makes the big stuff immediately obvious
- **🔍 Smart defaults** — shows top 20 items per directory, skips noise
- **🛠  Flexible** — control depth, threshold, file visibility, count, and more
- **🖥  Cross-platform** — Windows, macOS, Linux — works everywhere

## 📦 Installation

### Cargo (recommended)

```bash
cargo install duster
```

### From source

```bash
git clone https://github.com/lll82012/duster.git
cd duster
cargo build --release
./target/release/duster --help
```

## 🚀 Usage

### Basic: scan current directory

```bash
duster
```

### Scan a specific directory

```bash
duster ~/projects
```

### Limit depth

```bash
duster -d 2 ~/Downloads
```

### Show files (not just directories)

```bash
duster -f -n 10 .
```

### Filter by minimum size

```bash
duster -t 100MB /data
```

### All options

```
Usage: duster [OPTIONS] [PATH]

Arguments:
  [PATH]  Directory to scan (defaults to current directory)

Options:
  -d, --depth <N>       Maximum display depth
  -n, --count <N>       Number of top items to show per directory [default: 20]
  -f, --files           Show individual files, not just directories
  -t, --threshold <SIZE>  Minimum size threshold (e.g., "10MB", "1GB")
      --no-color        Disable colored output
  -h, --help            Print help
  -V, --version         Print version
```

## 📸 Screenshot

```
 📂  duster
 ├─ Total size:  32.5 KB
 ├─ Files:       14
 └─ Directories: 12

├── 📁 target 20.6 MB  ████████████████████████
 │  ├── 📁 debug 20.6 MB  ████████████████████████  100.0%
 │   │  ├── 📄 duster.pdb 15.2 MB  ██████████████████  74.1%
 │   │  ├── 📄 libduster.rlib 3.21 MB  ████  15.6%
 │   │  └── 📄 duster.exe 2.12 MB  ███  10.3%
 │  └── 📄 .rustc_info.json 1.11 KB
├── 📄 Cargo.lock 18.4 KB  ████████████████████████
├── 📁 src 11.3 KB  ███████████████
 │  ├── 📄 main.rs 6.25 KB  ████████████████████████  55.3%
 │  └── 📄 lib.rs 5.05 KB  ████████████████████  44.7%
├── 📄 LICENSE 1.06 KB  ██
└── 📄 Cargo.toml 604 B  █
```

## 🔧 Why Duster?

| Tool | Speed | Visual | Tree View | Size Bars | Cross-Platform |
|------|-------|--------|-----------|-----------|----------------|
| `du` | ✅ | ❌ | ❌ | ❌ | ✅ |
| `ncdu` | ✅ | ✅ | ✅ | ❌ | ✅ |
| `dust` | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Duster** | ✅ | ✅ | ✅ | ✅ | ✅ |

Duster combines the best of all worlds: the speed of Rust, the beauty of tree views, and visual bars that make it instantly obvious what's consuming your disk.

## 🤝 Contributing

Contributions are welcome! Feel free to open issues or PRs.

```bash
git clone https://github.com/lll82012/duster.git
cd duster
cargo test
cargo clippy
```

## 📄 License

MIT © [lll82012](https://github.com/lll82012)
