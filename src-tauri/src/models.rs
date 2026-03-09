use serde::{Deserialize, Serialize};

/// 磁碟機資訊（供磁碟機選擇器使用）。
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DriveInfo {
    /// 根路徑，例如 "C:\\"
    pub path: String,
    /// 顯示標籤，例如 "C:"
    pub label: String,
    /// 總容量（bytes）
    pub total: u64,
    /// 可用空間（bytes）
    pub available: u64,
}

/// A file or directory entry returned from a scan.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileEntry {
    /// Absolute path on disk.
    pub path: String,
    /// File / folder name (last component of path).
    pub name: String,
    /// Total size in bytes. For files: raw size. For directories: recursive sum.
    /// If `is_computing` is true, this value is 0 (placeholder).
    pub size: u64,
    /// True when this entry is a directory.
    pub is_dir: bool,
    /// Number of files inside (recursive).  1 for plain files.
    pub file_count: u64,
    /// RFC-3339 last-modified timestamp, if available.
    pub modified: Option<String>,
    /// True when the directory size is still being computed in the background.
    pub is_computing: bool,
}

/// Disk usage information for a volume / mount point.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiskInfo {
    pub path: String,
    pub total: u64,
    pub available: u64,
    pub used: u64,
}

/// Result of a delete operation.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeleteResult {
    pub path: String,
    pub success: bool,
    pub error: Option<String>,
}

// ── Trash ─────────────────────────────────────────────────────────────────────

/// Metadata stored alongside each trashed item (as a `.meta.json` sidecar).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrashMeta {
    pub original_path: String,
    pub name: String,
    pub size: u64,
    pub deleted_at: String,
}

/// A trash entry returned to the frontend.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrashEntry {
    /// Unique ID (used as the folder name inside the trash directory).
    pub trash_id: String,
    pub original_path: String,
    pub name: String,
    pub size: u64,
    pub deleted_at: String,
}
