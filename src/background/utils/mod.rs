pub mod constants;
pub mod icon_extractor;
pub mod integrity;
pub mod lock_free;
pub mod winver;

pub use winver::*;

use std::path::{Path, PathBuf};

pub fn date_based_hex_id() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{:x}", now.as_millis())
}

pub fn spawn_named_thread<F>(name: &str, f: F) -> std::thread::JoinHandle<()>
where
    F: FnOnce() + Send + 'static,
{
    std::thread::Builder::new()
        .name(name.to_string())
        .spawn(f)
        .expect("Failed to spawn thread")
}

/// Splits a raw inline command into program and args (shell-like, naive split).
pub fn get_parts_of_inline_command(command: &str) -> (String, Vec<String>) {
    let mut parts = command.split_whitespace();
    let Some(program) = parts.next() else {
        return (String::new(), Vec::new());
    };
    (program.to_string(), parts.map(String::from).collect())
}

/// Collects all files (not directories) under `folder` recursively.
pub fn collect_files(folder: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    let stack = vec![folder.to_path_buf()];
    let mut dirs = stack;
    while let Some(dir) = dirs.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            Err(err) => return Err(err),
        };
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            } else {
                result.push(path);
            }
        }
    }
    Ok(result)
}

/// Writes the file content atomically: writes to a temp sibling first, then renames it.
pub fn atomic_write_file(path: &Path, content: &[u8]) -> std::io::Result<()> {
    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, content)?;
    match std::fs::rename(&temp_path, path) {
        Ok(()) => Ok(()),
        Err(err) => {
            let _ = std::fs::remove_file(&temp_path);
            Err(err)
        }
    }
}
