use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::models::FileEntry;

// ── Public API ────────────────────────────────────────────────────────────────

/// 第一階段：快速列出 `path` 的直接子項，**不計算目錄大小**。
/// 目錄的 size = 0，is_computing = true。
/// 檔案立即取得實際大小。
/// 回傳順序：目錄優先，檔案次之，各自依名稱排序。
pub fn scan_directory_quick(path: &str) -> Result<Vec<FileEntry>, std::io::Error> {
    let dir_iter = std::fs::read_dir(path)?;

    let mut entries: Vec<FileEntry> = dir_iter
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            let meta = entry.metadata().ok()?;
            let path_obj = entry.path();
            let is_dir = meta.is_dir();

            Some(FileEntry {
                path: path_obj.display().to_string(),
                name: entry.file_name().to_string_lossy().into_owned(),
                size: if is_dir { 0 } else { meta.len() },
                is_dir,
                file_count: if is_dir { 0 } else { 1 },
                modified: system_time_to_rfc3339(meta.modified().ok()),
                is_computing: is_dir,  // 目錄尚未計算大小
            })
        })
        .collect();

    // 目錄優先，同類型依名稱排序
    entries.sort_unstable_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

/// 第二階段（背景執行）：對每個子目錄計算大小，逐一透過 callback 回報。
/// 使用 rayon 並行計算，但限制 thread 數量避免 IO 飽和。
pub fn compute_dir_sizes_with_progress(
    path: &str,
    cancel: Arc<AtomicBool>,
    on_update: impl Fn(String, u64, u64) + Send + Sync,
) {
    let Ok(dir_iter) = std::fs::read_dir(path) else { return };

    let dirs: Vec<std::fs::DirEntry> = dir_iter
        .filter_map(|e| e.ok())
        .filter(|e| e.metadata().map(|m| m.is_dir()).unwrap_or(false))
        .collect();

    // 並行計算各目錄大小，最多 6 個執行緒（避免 HDD IO 飽和）
    dirs.par_iter().for_each(|entry| {
        if cancel.load(Ordering::Relaxed) {
            return;
        }
        let entry_path = entry.path();
        let path_str = entry_path.display().to_string();
        let (size, file_count) = compute_dir_stats(&entry_path);
        on_update(path_str, size, file_count);
    });
}

/// 舊版同步掃描（保留供 deep scan 使用）。
pub fn scan_directory(path: &str) -> Result<Vec<FileEntry>, std::io::Error> {
    let dir_iter = std::fs::read_dir(path)?;

    let raw: Vec<std::fs::DirEntry> = dir_iter.filter_map(|e| e.ok()).collect();

    let mut entries: Vec<FileEntry> = raw
        .par_iter()
        .filter_map(|entry| {
            let meta = entry.metadata().ok()?;
            let path_obj = entry.path();

            let (size, file_count) = if meta.is_dir() {
                compute_dir_stats(&path_obj)
            } else {
                (meta.len(), 1)
            };

            Some(FileEntry {
                path: path_obj.display().to_string(),
                name: entry.file_name().to_string_lossy().into_owned(),
                size,
                is_dir: meta.is_dir(),
                file_count,
                modified: system_time_to_rfc3339(meta.modified().ok()),
                is_computing: false,
            })
        })
        .collect();

    entries.sort_unstable_by(|a, b| b.size.cmp(&a.size));
    Ok(entries)
}

/// 深層掃描，串流回報進度。
pub fn scan_deep_with_progress(
    path: &str,
    cancel: Arc<AtomicBool>,
    on_chunk: impl Fn(Vec<FileEntry>, u64) + Send + Sync,
) {
    let mut all: Vec<walkdir::DirEntry> = Vec::with_capacity(50_000);

    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .skip(1)
    {
        if cancel.load(Ordering::Relaxed) {
            return;
        }
        all.push(entry);
    }

    let chunk_size = 1_000;
    let mut processed = 0u64;

    for chunk in all.chunks(chunk_size) {
        if cancel.load(Ordering::Relaxed) {
            return;
        }

        let entries: Vec<FileEntry> = chunk
            .par_iter()
            .filter_map(|e| {
                let meta = e.metadata().ok()?;
                Some(FileEntry {
                    path: e.path().display().to_string(),
                    name: e.file_name().to_string_lossy().into_owned(),
                    size: if meta.is_file() { meta.len() } else { 0 },
                    is_dir: meta.is_dir(),
                    file_count: if meta.is_file() { 1 } else { 0 },
                    modified: system_time_to_rfc3339(meta.modified().ok()),
                    is_computing: false,
                })
            })
            .collect();

        processed += chunk.len() as u64;
        on_chunk(entries, processed);
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// 遞迴計算目錄的（總位元組, 總檔案數）。
fn compute_dir_stats(dir: &Path) -> (u64, u64) {
    WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .fold((0u64, 0u64), |(bytes, count), m| {
            (bytes + m.len(), count + 1)
        })
}

fn system_time_to_rfc3339(t: Option<std::time::SystemTime>) -> Option<String> {
    t.map(|st| {
        let dt: DateTime<Utc> = st.into();
        dt.to_rfc3339()
    })
}
