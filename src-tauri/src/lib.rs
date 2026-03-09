mod delete_engine;
mod error;
mod models;
mod scanner;
mod trash;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::{DeleteResult, DiskInfo, DriveInfo, FileEntry, TrashEntry};

// ── 共用狀態 ──────────────────────────────────────────────────────────────────

/// 持有取消旗標，可中止進行中的深層掃描。
pub struct ScanState {
    pub cancel_flag: Arc<AtomicBool>,
}

impl Default for ScanState {
    fn default() -> Self {
        Self {
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }
}

// ── Tauri commands 模組（避免 E0255 巨集命名衝突）────────────────────────────

pub mod cmd {
    use super::*;
    use tauri::{AppHandle, Emitter, State};

    // ── 掃描命令 ──────────────────────────────────────────────────────────────

    /// 回傳 `path` 的直接子項，依大小遞減排序。目錄大小以遞迴方式計算。
    #[tauri::command]
    pub fn scan_directory(path: String) -> Result<Vec<FileEntry>, AppError> {
        scanner::scan_directory(&path).map_err(AppError::from)
    }

    /// 啟動背景深層掃描，結果透過 `scan-chunk` 事件串流到前端。
    #[tauri::command]
    pub fn start_deep_scan(
        path: String,
        app: AppHandle,
        state: State<'_, ScanState>,
    ) -> Result<(), AppError> {
        state.cancel_flag.store(false, Ordering::Relaxed);
        let cancel = state.cancel_flag.clone();

        std::thread::spawn(move || {
            scanner::scan_deep_with_progress(&path, cancel, |entries, processed| {
                let _ = app.emit(
                    "scan-chunk",
                    serde_json::json!({ "entries": entries, "processed": processed }),
                );
            });

            let _ = app.emit("scan-complete", ());
        });

        Ok(())
    }

    /// 取消進行中的深層掃描。
    #[tauri::command]
    pub fn cancel_scan(state: State<'_, ScanState>) {
        state.cancel_flag.store(true, Ordering::Relaxed);
    }

    // ── 兩段式掃描（快速列項 + 背景大小計算）─────────────────────────

    /// 第一階段：快速列出 path 的直接子項，不計算目錄大小。
    /// 目錄的 size = 0、is_computing = true，檔案則有實際大小。
    #[tauri::command]
    pub fn scan_directory_quick(path: String) -> Result<Vec<FileEntry>, AppError> {
        scanner::scan_directory_quick(&path).map_err(AppError::from)
    }

    /// 第二階段：對指定路徑下的所有子目錄背景計算大小。
    /// 每個目錄計算完成後，發送 `size-update` 事件給前端。
    /// 全部完成後發送 `sizes-complete` 事件。
    #[tauri::command]
    pub fn compute_dir_sizes(
        path: String,
        app: AppHandle,
        state: State<'_, ScanState>,
    ) -> Result<(), AppError> {
        state.cancel_flag.store(false, Ordering::Relaxed);
        let cancel = state.cancel_flag.clone();

        std::thread::spawn(move || {
            scanner::compute_dir_sizes_with_progress(
                &path,
                cancel,
                |entry_path, size, file_count| {
                    let _ = app.emit(
                        "size-update",
                        serde_json::json!({ "path": entry_path, "size": size, "file_count": file_count }),
                    );
                },
            );
            let _ = app.emit("sizes-complete", ());
        });

        Ok(())
    }

    // ── 磁碟機列舉 ────────────────────────────────────────────────────────────

    /// 列出系統上所有可用的磁碟機。
    #[tauri::command]
    pub fn list_drives() -> Vec<DriveInfo> {
        list_drives_impl()
    }

    #[cfg(target_os = "windows")]
    fn list_drives_impl() -> Vec<DriveInfo> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let mut drives = Vec::new();

        // SAFETY: GetLogicalDrives 是標準 Windows API
        let bitmask = unsafe {
            windows_sys::Win32::Storage::FileSystem::GetLogicalDrives()
        };

        for i in 0u32..26 {
            if bitmask & (1 << i) == 0 {
                continue;
            }
            let letter = (b'A' + i as u8) as char;
            let root = format!("{}:\\", letter);

            let wide: Vec<u16> = OsStr::new(&root)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            let mut free_bytes: u64 = 0;
            let mut total_bytes: u64 = 0;
            let mut total_free: u64 = 0;

            let ok = unsafe {
                windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
                    wide.as_ptr(),
                    &mut free_bytes,
                    &mut total_bytes,
                    &mut total_free,
                )
            };

            // 跳過無法讀取的磁碟（例如沒有放入媒體的光碟機）
            if ok != 0 && total_bytes > 0 {
                drives.push(DriveInfo {
                    path: root,
                    label: format!("{}:", letter),
                    total: total_bytes,
                    available: free_bytes,
                });
            }
        }

        drives
    }

    #[cfg(not(target_os = "windows"))]
    fn list_drives_impl() -> Vec<DriveInfo> {
        vec![DriveInfo {
            path: "/".to_string(),
            label: "Root (/)".to_string(),
            total: 0,
            available: 0,
        }]
    }

    // ── 版本號命令 ────────────────────────────────────────────────────────────

    /// 回傳應用程式版本號（來自 Cargo.toml）。
    #[tauri::command]
    pub fn app_version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    // ── 刪除命令 ──────────────────────────────────────────────────────────────

    /// 快速刪除：重命名後背景移除，立即回傳。
    #[tauri::command]
    pub fn delete_fast(path: String) -> Result<(), AppError> {
        delete_engine::ultra_delete(&path).map_err(AppError::from)
    }

    /// 刪除單一檔案。
    #[tauri::command]
    pub fn delete_file(path: String) -> Result<(), AppError> {
        delete_engine::delete_file(&path).map_err(AppError::from)
    }

    /// 並行刪除多個路徑。
    #[tauri::command]
    pub fn batch_delete(paths: Vec<String>) -> Vec<DeleteResult> {
        delete_engine::batch_delete(paths)
    }

    // ── 垃圾桶命令 ────────────────────────────────────────────────────────────

    #[tauri::command]
    pub fn trash_item(path: String) -> Result<TrashEntry, AppError> {
        trash::move_to_trash(&path).map_err(AppError::from)
    }

    #[tauri::command]
    pub fn list_trash() -> Result<Vec<TrashEntry>, AppError> {
        trash::list_trash().map_err(AppError::from)
    }

    #[tauri::command]
    pub fn restore_trash(trash_id: String) -> Result<String, AppError> {
        trash::restore_from_trash(&trash_id).map_err(AppError::from)
    }

    #[tauri::command]
    pub fn purge_trash(trash_id: String) -> Result<(), AppError> {
        trash::purge_from_trash(&trash_id).map_err(AppError::from)
    }

    #[tauri::command]
    pub fn empty_trash() -> Result<u64, AppError> {
        trash::empty_trash().map_err(AppError::from)
    }

    // ── 磁碟資訊命令 ──────────────────────────────────────────────────────────

    #[tauri::command]
    pub fn get_disk_info(path: String) -> Result<DiskInfo, AppError> {
        disk_info_impl(&path)
    }

    #[cfg(target_os = "windows")]
    fn disk_info_impl(path: &str) -> Result<DiskInfo, AppError> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let wide: Vec<u16> = OsStr::new(path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut free_bytes: u64 = 0;
        let mut total_bytes: u64 = 0;
        let mut total_free: u64 = 0;

        let ok = unsafe {
            windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
                wide.as_ptr(),
                &mut free_bytes,
                &mut total_bytes,
                &mut total_free,
            )
        };

        if ok == 0 {
            return Err(AppError::Custom("GetDiskFreeSpaceExW failed".into()));
        }

        Ok(DiskInfo {
            path: path.to_string(),
            total: total_bytes,
            available: free_bytes,
            used: total_bytes.saturating_sub(free_bytes),
        })
    }

    #[cfg(not(target_os = "windows"))]
    fn disk_info_impl(path: &str) -> Result<DiskInfo, AppError> {
        use std::ffi::CString;
        use std::mem::MaybeUninit;

        let c_path = CString::new(path).map_err(|e| AppError::Custom(e.to_string()))?;

        let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
        let ret = unsafe { libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) };

        if ret != 0 {
            return Err(AppError::Io(std::io::Error::last_os_error()));
        }

        let stat = unsafe { stat.assume_init() };
        let block = stat.f_frsize as u64;
        let total = stat.f_blocks as u64 * block;
        let available = stat.f_bavail as u64 * block;

        Ok(DiskInfo {
            path: path.to_string(),
            total,
            available,
            used: total.saturating_sub(available),
        })
    }
}

// ── 程式進入點 ────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ScanState::default())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // 掃描
            cmd::scan_directory,
            cmd::scan_directory_quick,
            cmd::compute_dir_sizes,
            cmd::start_deep_scan,
            cmd::cancel_scan,
            // 磁碟機 & 版本
            cmd::list_drives,
            cmd::app_version,
            // 刪除
            cmd::delete_fast,
            cmd::delete_file,
            cmd::batch_delete,
            // 垃圾桶
            cmd::trash_item,
            cmd::list_trash,
            cmd::restore_trash,
            cmd::purge_trash,
            cmd::empty_trash,
            // 磁碟資訊
            cmd::get_disk_info,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 應用程式啟動失敗");
}
