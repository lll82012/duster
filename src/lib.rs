use anyhow::Context;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct ScanResult {
    pub root_name: String,
    pub root_size: u64,
    pub entries: Vec<Entry>,
    pub total_files: u64,
    pub total_dirs: u64,
}

pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub children: Vec<Entry>,
    pub depth: usize,
}

pub struct ScanOptions {
    pub max_depth: Option<usize>,
    pub min_size: Option<u64>,
    pub show_files: bool,
    pub top_n: Option<usize>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            max_depth: None,
            min_size: None,
            show_files: false,
            top_n: Some(20),
        }
    }
}

pub fn scan(path: &Path, options: &ScanOptions) -> anyhow::Result<ScanResult> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("Cannot access path: {}", path.display()))?;

    let root_name = canonical
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| canonical.to_string_lossy().to_string());

    let mut size_map: HashMap<PathBuf, u64> = HashMap::new();
    let mut children_map: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    let mut is_dir_map: HashMap<PathBuf, bool> = HashMap::new();
    let mut total_files = 0u64;
    let mut total_dirs = 0u64;

    let walk_depth = options.max_depth.map(|d| d + 1).unwrap_or(usize::MAX);

    for entry in WalkDir::new(&canonical)
        .max_depth(walk_depth)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path().to_path_buf();
        let metadata = entry.metadata().ok();

        let is_dir = entry.file_type().is_dir();
        is_dir_map.insert(path.clone(), is_dir);

        if is_dir {
            total_dirs += 1;
            children_map.entry(path.clone()).or_default();
        } else {
            total_files += 1;
            let size = metadata.map(|m| m.len()).unwrap_or(0);
            *size_map.entry(path.clone()).or_insert(0) += size;
        }

        if let Some(parent) = path.parent() {
            let parent = parent.to_path_buf();
            children_map
                .entry(parent)
                .or_default()
                .push(path.clone());
        }
    }

    // Bubble up sizes from children to parents (bottom-up)
    let mut sorted_paths: Vec<_> = size_map.keys().cloned().collect();
    sorted_paths.sort_by_key(|p| {
        let depth = p.components().count();
        std::cmp::Reverse(depth)
    });

    for path in &sorted_paths {
        let size = *size_map.get(path).unwrap_or(&0);
        if let Some(parent) = path.parent() {
            if parent.starts_with(&canonical) || parent == &canonical {
                *size_map.entry(parent.to_path_buf()).or_insert(0) += size;
            }
        }
    }

    // Assign directory sizes to size_map
    for (path, children) in &children_map {
        if *is_dir_map.get(path).unwrap_or(&false) {
            let total: u64 = children.iter().map(|c| size_map.get(c).unwrap_or(&0)).sum();
            size_map.insert(path.clone(), total);
        }
    }

    let root_size = *size_map.get(&canonical).unwrap_or(&0);

    fn build_tree(
        path: &Path,
        size_map: &HashMap<PathBuf, u64>,
        children_map: &HashMap<PathBuf, Vec<PathBuf>>,
        is_dir_map: &HashMap<PathBuf, bool>,
        root: &Path,
        depth: usize,
        options: &ScanOptions,
    ) -> Entry {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let is_dir = *is_dir_map.get(path).unwrap_or(&false);
        let size = *size_map.get(path).unwrap_or(&0);

        let children = if is_dir {
            let mut kids: Vec<Entry> = children_map
                .get(path)
                .cloned()
                .unwrap_or_default()
                .iter()
                .map(|child_path| {
                    build_tree(
                        child_path, size_map, children_map, is_dir_map, root, depth + 1, options,
                    )
                })
                .collect();
            kids.sort_by(|a, b| b.size.cmp(&a.size));
            if let Some(n) = options.top_n {
                kids.truncate(n);
            }
            kids
        } else {
            Vec::new()
        };

        Entry {
            name,
            path: path.to_path_buf(),
            size,
            is_dir,
            children,
            depth: path.components().count().saturating_sub(root.components().count()),
        }
    }

    let root_entry = build_tree(
        &canonical,
        &size_map,
        &children_map,
        &is_dir_map,
        &canonical,
        0,
        options,
    );

    Ok(ScanResult {
        root_name,
        root_size,
        entries: root_entry.children,
        total_files,
        total_dirs,
    })
}
