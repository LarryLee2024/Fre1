use std::fs;
use std::path::Path;

use crate::content::loading::discover_ron_files;

#[test]
fn discover_empty_dir() {
    let dir = std::env::temp_dir().join("fre_test_discover_empty");
    let _ = fs::create_dir_all(&dir);
    let files = discover_ron_files(&dir);
    assert!(files.is_empty());
}

#[test]
fn discover_nonexistent_dir() {
    let dir = Path::new("/nonexistent/path/config");
    let files = discover_ron_files(dir);
    assert!(files.is_empty());
}
