# FastDiskCleaner

A high-performance disk analyser and cleaner built with **Rust + Tauri v2 + Vue 3**.

## Features

| Feature | Detail |
|---|---|
| 📂 **Tree Browser** | Drill into any folder, sizes computed recursively in parallel |
| 🔬 **Deep Scan** | Full subtree walk with streaming progress (handles 500 k+ files) |
| ⚡ **Ultra Delete** | Rename → background `remove_dir_all` → UI returns instantly |
| 🗑 **Custom Trash** | Recoverable delete with JSON metadata sidecars |
| 📊 **Size Bars** | Proportional visualisation per row |
| 🖥 **Native Window** | Custom title bar, frameless window |

## Tech Stack

- **Rust** backend: `walkdir` + `rayon` + `chrono` + `thiserror`
- **Tauri v2** IPC bridge
- **Vue 3** + Vite frontend (single `.exe` output)

## Quickstart

### Prerequisites

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node ≥ 20
https://nodejs.org

# Tauri system deps (Windows: nothing extra)
# Ubuntu: sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev
```

### Run (dev)

```bash
npm install
npm run tauri dev
```

### Build (release EXE)

```bash
npm run tauri build
# Output: src-tauri/target/release/bundle/
```

## Architecture

```
fast-disk-cleaner/
├─ src-tauri/src/
│   ├─ main.rs          — thin entry point
│   ├─ lib.rs           — all Tauri commands
│   ├─ models.rs        — shared types (FileEntry, TrashEntry, …)
│   ├─ scanner.rs       — walkdir + rayon scan engine
│   ├─ delete_engine.rs — ultra-fast rename-then-delete
│   ├─ trash.rs         — custom recoverable trash
│   └─ error.rs         — unified AppError type
└─ src/
    ├─ App.vue                    — main shell (tabs, title bar, toolbar)
    ├─ components/
    │   ├─ FileList.vue           — sortable file table with size bars
    │   ├─ DiskUsage.vue          — volume usage bar
    │   └─ TrashManager.vue       — trash list + restore / purge
    └─ composables/
        └─ useScanner.ts          — all invoke() calls + state
```

## Key design decisions

### Why not `par_bridge()` on WalkDir?

`WalkDir` is an iterator that must run on a single thread (directory ordering).
`par_bridge()` wraps it in a work-stealing adapter but adds heavy synchronisation
overhead.  The correct approach used here:

1. **Single-threaded walk** → collect `Vec<DirEntry>`
2. **Parallel metadata reads** → `.par_iter()` on chunks

### Ultra-delete

```
fs::rename(path, path + ".deleting_<ts>")   ← atomic, instant
thread::spawn(|| fs::remove_dir_all(...))    ← background
```

The rename is atomic on the same filesystem, so the UI sees success immediately
while the actual I/O happens off the main thread.

### Streaming deep scan

Rather than serialising 500 k `FileEntry` objects into one giant JSON response,
the backend emits `scan-chunk` events every 1 000 entries.  The frontend
accumulates them and re-renders progressively.

## Upgrade path

| Goal | Approach |
|---|---|
| NTFS MFT scan (WizTree speed) | `ntfs` crate or Windows `DeviceIoControl` |
| Treemap visualisation | D3 / ECharts in a new Vue component |
| Smart clean (node_modules etc.) | Pattern matcher on entry names |
| Cloud backup before delete | `rclone` shell-out via `tauri-plugin-shell` |
