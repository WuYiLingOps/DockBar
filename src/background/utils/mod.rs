pub mod constants;
pub mod icon_extractor;
pub mod integrity;
pub mod lock_free;
pub mod winver;

pub use winver::*;

use std::{
    ffi::OsString,
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
};

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
///
/// Enumerates with `FindFirstFileW`/`FindNextFileW` instead of
/// [`std::fs::read_dir`]: on machines with damaged folder ACLs, the directory
/// handle `read_dir` opens (no `FILE_FLAG_BACKUP_SEMANTICS`) is rejected with
/// `PermissionDenied` while the shell-style enumeration still works.
///
/// Unreadable subdirectories are skipped with a warning instead of failing the
/// whole scan: a single locked folder (AV scan, index churn) must not empty
/// the entire start-menu listing.
pub fn collect_files(folder: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    let mut dirs = vec![folder.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        match find_dir_entries(&dir) {
            Ok(entries) => {
                for entry in entries {
                    let path = dir.join(&entry.file_name);
                    if entry.is_dir {
                        dirs.push(path);
                    } else {
                        result.push(path);
                    }
                }
            }
            // the folder may have vanished between listing and scanning
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            Err(err) => {
                log::warn!("Skipping unreadable folder {dir:?}: {err:?}");
            }
        }
    }
    Ok(result)
}

struct FindEntry {
    file_name: PathBuf,
    is_dir: bool,
}

/// Enumerates one directory via the Win32 find APIs, skipping `.` and `..`.
fn find_dir_entries(dir: &Path) -> std::io::Result<Vec<FindEntry>> {
    use windows::Win32::Storage::FileSystem::{
        FILE_ATTRIBUTE_DIRECTORY, FindClose, FindFirstFileW, FindNextFileW, WIN32_FIND_DATAW,
    };
    use windows::core::PCWSTR;

    let pattern: Vec<u16> = dir
        .as_os_str()
        .encode_wide()
        .chain(r"\*".encode_utf16())
        .chain(std::iter::once(0))
        .collect();

    let mut entries = Vec::new();
    let mut data = WIN32_FIND_DATAW::default();
    unsafe {
        let handle = FindFirstFileW(PCWSTR(pattern.as_ptr()), &mut data)?;
        loop {
            let name = &data.cFileName;
            let len = name.iter().position(|&c| c == 0).unwrap_or(name.len());
            let file_name = OsString::from_wide(&name[..len]);
            if file_name != "." && file_name != ".." {
                entries.push(FindEntry {
                    file_name: PathBuf::from(file_name),
                    is_dir: data.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY.0 != 0,
                });
            }
            if FindNextFileW(handle, &mut data).is_err() {
                break;
            }
        }
        let _ = FindClose(handle);
    }
    Ok(entries)
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
