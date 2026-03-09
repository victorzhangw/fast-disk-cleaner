use std::fs;
use std::path::Path;
use std::thread;

use rayon::prelude::*;

use crate::models::DeleteResult;

// ── Single delete ─────────────────────────────────────────────────────────────

/// **刪除策略（依序嘗試）：**
///
/// 1. 對目錄：rename → 背景 remove_dir_all（原子 rename 立即回傳）
/// 2. 對檔案：直接 remove_file（最快）
/// 3. rename 失敗時（跨磁碟、權限不足等）：直接就地刪除
pub fn ultra_delete(path: &str) -> Result<(), std::io::Error> {
    let src = Path::new(path);

    if !src.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("路徑不存在: {path}"),
        ));
    }

    // 單一檔案直接刪除，最快
    if src.is_file() {
        return fs::remove_file(src).map_err(|e| {
            std::io::Error::new(
                e.kind(),
                format!("無法刪除檔案 '{path}': {e}"),
            )
        });
    }

    // 目錄：嘗試 rename → 背景刪除
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    // 在相同父目錄下建立暫存名稱（確保同一磁碟）
    let staging = format!("{}.deleting_{}", path, ts);

    match fs::rename(src, &staging) {
        Ok(()) => {
            // Rename 成功 → 背景刪除
            thread::spawn(move || {
                if let Err(e) = fs::remove_dir_all(&staging) {
                    eprintln!("[delete_engine] background remove_dir_all failed for '{staging}': {e}");
                    let _ = remove_recursive_fallback(Path::new(&staging));
                }
            });
            Ok(())
        }
        Err(rename_err) => {
            // Rename 失敗（跨磁碟、目錄保護等）→ 直接就地刪除
            eprintln!("[delete_engine] rename failed for '{path}': {rename_err}, falling back to direct removal");
            fs::remove_dir_all(src).map_err(|e| {
                std::io::Error::new(
                    e.kind(),
                    format!("無法刪除目錄 '{path}': {e}（rename 也失敗: {rename_err}）"),
                )
            })
        }
    }
}

/// 刪除單一路徑（自動判斷檔案或目錄）。
pub fn delete_file(path: &str) -> Result<(), std::io::Error> {
    ultra_delete(path)
}

// ── Batch delete ──────────────────────────────────────────────────────────────

/// 並行刪除多個路徑，回傳每個路徑的成功/失敗結果（含詳細錯誤訊息）。
pub fn batch_delete(paths: Vec<String>) -> Vec<DeleteResult> {
    paths
        .par_iter()
        .map(|path| match ultra_delete(path) {
            Ok(_) => DeleteResult {
                path: path.clone(),
                success: true,
                error: None,
            },
            Err(e) => DeleteResult {
                path: path.clone(),
                success: false,
                error: Some(format!("{}: {}", path, e)),
            },
        })
        .collect()
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// 最後手段：遞迴刪除（用於 remove_dir_all 失敗的情況）。
fn remove_recursive_fallback(path: &Path) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            remove_recursive_fallback(&entry?.path())?;
        }
        fs::remove_dir(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}
