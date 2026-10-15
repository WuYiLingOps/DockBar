use std::{fs::create_dir, path::PathBuf};

use dockbar_utils::checksums::CheckSums;

// This build is keyless: no update server, no signing keys. The checksums file
// still lets the runtime verify bundled static assets, but no minisign
// signature is produced (a stub is written instead).

const UNSIGNED_STUB: &str = "NOT SIGNED (KEYLESS BUILD)";

fn main() {
    let _ = create_dir("gen");

    let mut checksums = CheckSums::new();
    read_folder_recursive(PathBuf::from("static"), &mut |path| {
        checksums.add(&path).unwrap();
    });

    let target_dir = target_dir();
    let sums_path = target_dir.join("SHA256SUMS");
    checksums.write(&sums_path).unwrap();
    std::fs::write(sums_path.with_extension("sig"), UNSIGNED_STUB).unwrap();

    tauri_build::build();
}

fn target_dir() -> PathBuf {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    // see <https://github.com/rust-lang/cargo/issues/5457>
    out_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn read_folder_recursive<F>(path: PathBuf, cb: &mut F)
where
    F: FnMut(PathBuf),
{
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            read_folder_recursive(path, cb);
        } else {
            cb(path);
        }
    }
}
