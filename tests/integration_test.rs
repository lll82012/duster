use duster::{scan, ScanOptions};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_scan_empty_directory() {
    let dir = TempDir::new().unwrap();
    let options = ScanOptions::default();
    let result = scan(dir.path(), &options).unwrap();

    assert_eq!(result.total_files, 0);
    assert_eq!(result.total_dirs, 1); // root dir
    assert!(result.entries.is_empty());
}

#[test]
fn test_scan_files_in_flat_directory() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("a.txt"), b"hello world").unwrap();
    fs::write(dir.path().join("b.txt"), b"hello").unwrap();

    let options = ScanOptions {
        show_files: true,
        ..ScanOptions::default()
    };

    let result = scan(dir.path(), &options).unwrap();
    assert_eq!(result.total_files, 2);

    // Both files should be visible when show_files is true
    let names: Vec<&str> = result.entries.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"a.txt"));
    assert!(names.contains(&"b.txt"));
}

#[test]
fn test_scan_nested_directories() {
    let dir = TempDir::new().unwrap();
    let sub = dir.path().join("sub");
    fs::create_dir(&sub).unwrap();
    fs::write(sub.join("data.txt"), b"nested file content here").unwrap();

    let options = ScanOptions::default();
    let result = scan(dir.path(), &options).unwrap();

    assert_eq!(result.total_files, 1);
    assert_eq!(result.total_dirs, 2); // root + sub

    // Should see the "sub" directory as a child
    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].name, "sub");
    assert!(result.entries[0].is_dir);
    assert!(result.entries[0].size > 0);
}

#[test]
fn test_scan_with_max_depth() {
    let dir = TempDir::new().unwrap();
    let sub = dir.path().join("sub");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join("shallow.txt"), b"shallow file").unwrap();

    let options = ScanOptions {
        max_depth: Some(2),
        ..ScanOptions::default()
    };

    let result = scan(dir.path(), &options).unwrap();
    assert!(result.total_files > 0);

    // With max_depth=0, only direct children of root are traversed
    let options2 = ScanOptions {
        max_depth: Some(0),
        ..ScanOptions::default()
    };
    let result2 = scan(dir.path(), &options2).unwrap();
    // max_depth=0 means walk_depth=1, which visits root only, no files inside sub/
    assert_eq!(result2.total_files, 0);
}

#[test]
fn test_scan_with_min_size() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("tiny.txt"), b"x").unwrap();
    fs::write(dir.path().join("big.txt"), vec![0u8; 10_000]).unwrap();

    let options = ScanOptions {
        min_size: Some(1_000),
        show_files: true,
        ..ScanOptions::default()
    };

    let result = scan(dir.path(), &options).unwrap();
    assert_eq!(result.total_files, 2);

    // Both files exist in scan, but display filtering happens in main
    let names: Vec<&str> = result.entries.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"tiny.txt"));
    assert!(names.contains(&"big.txt"));
}

#[test]
fn test_scan_symlinks_are_handled() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("original.txt"), b"content").unwrap();

    // Windows symlink test - skip if not available
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(
            dir.path().join("original.txt"),
            dir.path().join("link.txt"),
        )
        .unwrap();

        let options = ScanOptions {
            show_files: true,
            ..ScanOptions::default()
        };
        let result = scan(dir.path(), &options).unwrap();
        // Should not infinite-loop on symlinks
        assert!(result.total_files >= 1);
    }
}

#[test]
fn test_root_size_is_sum_of_children() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("f1.txt"), b"12345").unwrap(); // 5 bytes
    fs::write(dir.path().join("f2.txt"), b"1234567890").unwrap(); // 10 bytes

    let options = ScanOptions::default();
    let result = scan(dir.path(), &options).unwrap();

    assert_eq!(result.total_files, 2);
    // Root size should account for both files
    assert!(result.root_size >= 15);
}

#[test]
fn test_non_existent_path() {
    let result = scan(Path::new("/this/path/definitely/does/not/exist_12345"), &ScanOptions::default());
    assert!(result.is_err());
}
