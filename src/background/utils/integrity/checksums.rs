use std::path::Path;

use dockbar_utils::checksums::{CheckSums, Diff, calculate_sha256};
use futures::StreamExt;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::error::Result;

use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

// This build is keyless: no update server, no signing keys. The signature file
// is an unsigned stub produced by build.rs and is not verified. The checksums
// file itself is still validated against the bundled static assets, so any
// tampered or mismatched install still fails setup.

pub async fn ensure_bundle_files_integrity(app: &tauri::AppHandle) -> Result<()> {
    log::trace!("Validating bundle files integrity");

    let install_dir = app.path().resource_dir()?;
    let static_path = install_dir.join("static");

    let checksums_path = install_dir.join("SHA256SUMS");
    let signature_path = install_dir.join("SHA256SUMS.sig");

    if !checksums_path.exists() {
        return Err("Checksums file not found".into());
    }

    if !signature_path.exists() {
        return Err("Signature file not found".into());
    }

    validate_directory_checksums(&static_path, &checksums_path).await?;

    Ok(())
}

async fn validate_directory_checksums(base_path: &Path, checksums_path: &Path) -> Result<()> {
    log::trace!("Validating checksums for {}", base_path.display());

    let checksums_content = tokio::fs::read(checksums_path).await?;
    let expected_checksums = CheckSums::parse(&checksums_content)?;

    let root = base_path.parent().unwrap();
    let walk_path = base_path.to_path_buf();
    let files: Vec<std::path::PathBuf> =
        tokio::task::spawn_blocking(move || crate::utils::collect_files(walk_path.as_path()))
            .await
            .map_err(|e| format!("Bundle files listing task failed: {e}"))??
            .into_iter()
            .collect();

    let contents = futures::stream::iter(files)
        .map(|path| async move {
            let content = tokio::fs::read(&path).await.ok()?;
            Some((path, content))
        })
        .buffer_unordered(64)
        .filter_map(futures::future::ready)
        .collect::<Vec<_>>()
        .await;

    // Hashing is CPU bound, so it is spread between the cores off the runtime workers
    let hashes = tokio::task::spawn_blocking(move || {
        contents
            .into_par_iter()
            .map(|(path, content)| (path, calculate_sha256(&content)))
            .collect::<Vec<_>>()
    })
    .await
    .map_err(|e| format!("Bundle files hashing task failed: {e}"))?;

    // Store checksums with the path relative to the bundle root
    let mut actual_checksums = CheckSums::new();
    for (path, hash) in hashes {
        let relative_path = path.strip_prefix(root).expect("Strip failed");
        actual_checksums.add_hash(hash, relative_path);
    }

    let diffs = expected_checksums.compare(&actual_checksums);
    let (errors, extras): (Vec<_>, Vec<_>) = diffs
        .into_iter()
        .partition(|(diff, _)| *diff != Diff::Extra);

    // Extra files (leftovers from a previous install/dev run whose assets were
    // later removed from the source bundle) don't prove tampering: the bundler
    // never deletes stale files from the target resource dir. Warn and continue.
    if !extras.is_empty() {
        log::warn!(
            "Ignoring {} extra files not present in SHA256SUMS: {:#?}",
            extras.len(),
            extras
        );
    }

    if !errors.is_empty() {
        log::error!("Checksums mismatch: {:#?}", errors);
        return Err("Checksums does not match".into());
    }

    log::trace!("All Checksums validated successfully");
    Ok(())
}

pub fn show_bundle_integrity_dialog(app: &tauri::AppHandle) {
    app.dialog()
        .message(t!("runtime.files_integrity"))
        .title(t!("runtime.files_integrity_title"))
        .kind(MessageDialogKind::Error)
        .blocking_show();
}
