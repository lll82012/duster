use anyhow::Context;
use clap::Parser;
use colored::*;
use duster::{scan, Entry, ScanOptions};
use std::path::PathBuf;

/// ⚡ A blazingly fast disk usage analyzer with beautiful terminal output.
#[derive(Parser)]
#[command(name = "duster", version, about, long_about = None)]
struct Cli {
    /// Directory to scan (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Maximum display depth
    #[arg(short = 'd', long = "depth", value_name = "N")]
    max_depth: Option<usize>,

    /// Number of top items to show per directory
    #[arg(short = 'n', long = "count", default_value = "20", value_name = "N")]
    top_n: Option<usize>,

    /// Show individual files, not just directories
    #[arg(short = 'f', long = "files", default_value = "false")]
    show_files: bool,

    /// Minimum size threshold (e.g., "10MB", "1GB")
    #[arg(short = 't', long = "threshold", value_name = "SIZE")]
    min_size: Option<String>,

    /// Disable colored output
    #[arg(long = "no-color")]
    no_color: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.no_color {
        colored::control::set_override(false);
    }

    let min_size = match &cli.min_size {
        Some(s) => Some(parse_size(s)?),
        None => None,
    };

    let options = ScanOptions {
        max_depth: cli.max_depth,
        min_size,
        show_files: cli.show_files,
        top_n: cli.top_n,
    };

    let result = scan(&cli.path, &options)?;

    print_summary(&result);
    println!();
    print_entries(&result.entries, &options, 0, None);

    Ok(())
}

fn parse_size(s: &str) -> anyhow::Result<u64> {
    let s = s.trim().to_uppercase().replace(' ', "");
    let (num_str, unit) = if let Some(rest) = s.strip_suffix("GB") {
        (rest.trim(), 1_000_000_000u64)
    } else if let Some(rest) = s.strip_suffix("MB") {
        (rest.trim(), 1_000_000u64)
    } else if let Some(rest) = s.strip_suffix("KB") {
        (rest.trim(), 1_000u64)
    } else if let Some(rest) = s.strip_suffix('B') {
        (rest.trim(), 1u64)
    } else if let Some(rest) = s.strip_suffix("GIB") {
        (rest.trim(), 1_073_741_824u64)
    } else if let Some(rest) = s.strip_suffix("MIB") {
        (rest.trim(), 1_048_576u64)
    } else if let Some(rest) = s.strip_suffix("KIB") {
        (rest.trim(), 1_024u64)
    } else {
        return Err(anyhow::anyhow!(
            "Invalid size format: {s}. Use e.g. 10MB, 1GB, 500KB"
        ));
    };

    let num: f64 = num_str
        .parse()
        .with_context(|| format!("Invalid number: {num_str}"))?;
    Ok((num * unit as f64) as u64)
}

fn print_summary(result: &duster::ScanResult) {
    println!();
    println!(
        " {}  {}",
        "📂".bold(),
        result.root_name.bold().bright_white()
    );
    println!(
        " ├─ Total size:  {}",
        format_size(result.root_size).bright_yellow().bold()
    );
    println!(
        " ├─ Files:       {}",
        result.total_files.to_string().bright_green()
    );
    println!(
        " └─ Directories: {}",
        result.total_dirs.to_string().bright_cyan()
    );
}

fn print_entries(entries: &[Entry], options: &ScanOptions, _parent_depth: usize, parent_size: Option<u64>) {
    print_tree(entries, options, &[], parent_size);
}

fn print_tree(entries: &[Entry], options: &ScanOptions, ancestors_last: &[bool], parent_size: Option<u64>) {
    let visible: Vec<&Entry> = entries
        .iter()
        .filter(|e| {
            if let Some(min) = options.min_size {
                e.size >= min
            } else {
                true
            }
        })
        .filter(|e| e.is_dir || options.show_files)
        .filter(|e| e.size > 0 || e.is_dir) // skip empty files
        .take(options.top_n.unwrap_or(usize::MAX))
        .collect();

    let count = visible.len();

    // Use largest sibling as bar reference, fall back to parent size
    let reference = visible.iter().map(|e| e.size).max().unwrap_or(1);

    for (i, entry) in visible.iter().enumerate() {
        let is_last = i == count - 1;

        // Draw tree branch prefix
        for &last in ancestors_last {
            if last {
                print!("    ");
            } else {
                print!(" │  ");
            }
        }

        let connector = if is_last { "└── " } else { "├── " };
        print!("{}", connector);

        // Icon
        let icon = if entry.is_dir { "📁" } else { "📄" };
        print!("{} ", icon);

        // Name
        print!("{} ", entry.name.bright_white().bold());

        // Size
        print!("{}", format_size(entry.size).bright_yellow().bold());

        // Bar (relative to largest sibling)
        if reference > 0 {
            let ratio = entry.size as f64 / reference as f64;
            let bar_width = (ratio * 24.0).ceil().max(1.0) as usize;
            let bar = "█".repeat(bar_width);
            print!("  {}", bar.truecolor(80, 80, 80));
        }

        // Percentage
        if let Some(ps) = parent_size {
            if ps > 0 {
                let pct = entry.size as f64 / ps as f64 * 100.0;
                if pct >= 0.1 {
                    print!("  {:.1}%", pct);
                }
            }
        }

        println!();

        // Recurse into children
        if entry.is_dir && !entry.children.is_empty() {
            let mut next_ancestors = ancestors_last.to_vec();
            next_ancestors.push(is_last);
            print_tree(&entry.children, options, &next_ancestors, Some(entry.size));
        }
    }
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[(&str, f64)] = &[
        ("TB", 1_000_000_000_000.0),
        ("GB", 1_000_000_000.0),
        ("MB", 1_000_000.0),
        ("KB", 1_000.0),
        ("B", 1.0),
    ];

    for (unit, threshold) in UNITS {
        let value = bytes as f64 / threshold;
        if value >= 1.0 {
            if value >= 100.0 {
                return format!("{:.0} {}", value, unit);
            } else if value >= 10.0 {
                return format!("{:.1} {}", value, unit);
            } else {
                return format!("{:.2} {}", value, unit);
            }
        }
    }
    "0 B".to_string()
}
