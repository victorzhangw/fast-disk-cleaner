use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use walkdir::WalkDir;

use crate::models::{TrashEntry, TrashMeta};

// ── Trash directory ───────────────────────────────────────────────────────────

/// Returns the path to the application's private trash folder.
///
/// | Platform | Location                                              |
/// |----------|-------------------------------------------------------|
/// | Windows  | `%LOCALAPPDATA%\FastDiskCleaner\Trash`               |
/// | macOS    | `~/Library/Application Support/FastDiskCleaner/Trash`|
/// | Linux    | `~/.local/share/FastDiskCleaner/Trash`               |
fn trash_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("FastDiskCleaner")
        .join("Trash")
}

fn ensure_trash_dir() -> std::io::Result<PathBuf> {
    let dir = trash_dir();
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Move a file or directory into the custom trash.
///
/// Steps:
/// 1. Compute size before moving (so we can show it in the trash list).
/// 2. Generate a unique `trash_id` based on timestamp + original name.
/// 3. `fs::rename` the item into the trash folder (atomic, instant).
/// 4. Write a JSON sidecar `<trash_id>.meta.json` with restore metadata.
pub fn move_to_trash(path: &str) -> Result<TrashEntry, std::io::Error> {
    let src = Path::new(path);

    if !src.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Path not found: {path}"),
        ));
    }

    let trash = ensure_trash_dir()?;

    let name = src
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let now = Utc::now();
    let size = compute_size(src);

    // Unique ID: <unix_ms>_<name>
    let trash_id = format!("{}_{}", now.timestamp_millis(), name);
    let dest = trash.join(&trash_id);

    // Move item (atomic on same filesystem)
    fs::rename(src, &dest)?;

    // Write metadata sidecar
    let meta = TrashMeta {
        original_path: path.to_string(),
        name: name.clone(),
        size,
        deleted_at: now.to_rfc3339(),
    };
    let meta_path = trash.join(format!("{trash_id}.meta.json"));
    fs::write(&meta_path, serde_json::to_string_pretty(&meta)?)?;

    Ok(TrashEntry {
        trash_id,
        original_path: path.to_string(),
        name,
        size,
        deleted_at: now.to_rfc3339(),
    })
}

/// List all items currently in the custom trash.
pub fn list_trash() -> Result<Vec<TrashEntry>, std::io::Error> {
    let dir = trash_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut entries: Vec<TrashEntry> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .to_string_lossy()
                .ends_with(".meta.json")
        })
        .filter_map(|e| {
            let raw = fs::read_to_string(e.path()).ok()?;
            let meta: TrashMeta = serde_json::from_str(&raw).ok()?;

            let file_name = e.file_name().to_string_lossy().into_owned();
            let trash_id = file_name
                .strip_suffix(".meta.json")
                .unwrap_or(&file_name)
                .to_string();

            Some(TrashEntry {
                trash_id,
                original_path: meta.original_path,
                name: meta.name,
                size: meta.size,
                deleted_at: meta.deleted_at,
            })
        })
        .collect();

    // Newest first
    entries.sort_by(|a, b| b.deleted_at.cmp(&a.deleted_at));
    Ok(entries)
}

/// Restore a trashed item to its original location.
///
/// Fails with `AlreadyExists` if the destination path is already occupied.
pub fn restore_from_trash(trash_id: &str) -> Result<String, std::io::Error> {
    let dir = trash_dir();
    let item_path = dir.join(trash_id);
    let meta_path = dir.join(format!("{trash_id}.meta.json"));

    let raw = fs::read_to_string(&meta_path)?;
    let meta: TrashMeta =
        serde_json::from_str(&raw).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let dest = Path::new(&meta.original_path);

    if dest.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("Destination already exists: {}", meta.original_path),
        ));
    }

    // Re-create parent directories if they were deleted since trashing
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::rename(&item_path, dest)?;
    fs::remove_file(&meta_path)?;

    Ok(meta.original_path)
}

/// Permanently delete one item from the trash (no recovery possible).
pub fn purge_from_trash(trash_id: &str) -> Result<(), std::io::Error> {
    let dir = trash_dir();
    let item_path = dir.join(trash_id);
    let meta_path = dir.join(format!("{trash_id}.meta.json"));

    if item_path.is_dir() {
        fs::remove_dir_all(&item_path)?;
    } else if item_path.exists() {
        fs::remove_file(&item_path)?;
    }

    if meta_path.exists() {
        fs::remove_file(&meta_path)?;
    }

    Ok(())
}

/// Empty the entire trash, returning how many bytes were freed.
pub fn empty_trash() -> Result<u64, std::io::Error> {
    let dir = trash_dir();
    if !dir.exists() {
        return Ok(0);
    }

    let mut freed = 0u64;

    for entry in fs::read_dir(&dir)? {
        let path = entry?.path();
        freed += compute_size(&path);

        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }

    Ok(freed)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn compute_size(path: &Path) -> u64 {
    if path.is_file() {
        return path.metadata().map(|m| m.len()).unwrap_or(0);
    }

    WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}
