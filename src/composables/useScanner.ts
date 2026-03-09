import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// ── Types (mirroring Rust models) ─────────────────────────────────────────────

export interface FileEntry {
  path: string;
  name: string;
  size: number;
  is_dir: boolean;
  file_count: number;
  modified: string | null;
  is_computing: boolean;  // 目錄大小是否仍在計算中
}

export interface TrashEntry {
  trash_id: string;
  original_path: string;
  name: string;
  size: number;
  deleted_at: string;
}

export interface DiskInfo {
  path: string;
  total: number;
  available: number;
  used: number;
}

export interface DeleteResult {
  path: string;
  success: boolean;
  error: string | null;
}

// ── Helpers ───────────────────────────────────────────────────────────────────

export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const exp = Math.min(Math.floor(Math.log2(bytes) / 10), units.length - 1);
  const val = bytes / Math.pow(1024, exp);
  return `${val.toFixed(exp === 0 ? 0 : 1)} ${units[exp]}`;
}

export function formatDate(iso: string | null): string {
  if (!iso) return "—";
  const d = new Date(iso);
  return d.toLocaleDateString() + " " + d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

// ── Composable ────────────────────────────────────────────────────────────────

export function useScanner() {
  // ── State ──────────────────────────────────────────────────────────────────
  const currentPath = ref<string>("");
  const entries = ref<FileEntry[]>([]);
  const breadcrumbs = ref<string[]>([]);
  const isLoading = ref(false);        // 第一階段：列出項目
  const isComputingSizes = ref(false);      // 第二階段：計算容量
  const isDeepScanning = ref(false);
  const deepEntries = ref<FileEntry[]>([]);
  const deepProcessed = ref(0);
  const diskInfo = ref<DiskInfo | null>(null);
  const error = ref<string | null>(null);

  let unlistenChunk: UnlistenFn | null = null;
  let unlistenComplete: UnlistenFn | null = null;
  let unlistenSizeUpdate: UnlistenFn | null = null;
  let unlistenSizesComplete: UnlistenFn | null = null;

  // ── Computed ───────────────────────────────────────────────────────────────
  const sortedDeepEntries = computed(() =>
    [...deepEntries.value].sort((a, b) => b.size - a.size).slice(0, 500)
  );

  // ── Actions ────────────────────────────────────────────────────────────────

  /**
   * 兩段式導航：
   * 第一段：快速列出子項（名稱先出現），目錄 is_computing=true
   * 第二段：背景計算目錄大小，透過 size-update 事件逐一更新
   */
  async function navigateTo(path: string) {
    if (!path.trim()) return;

    // 取消上一次的大小計算（如果還在跑）
    cleanupSizeListeners();

    isLoading.value = true;
    error.value = null;

    try {
      // ── 第一階段：快速列出（毫秒級） ─────────────────────────────────────
      const quickEntries = await invoke<FileEntry[]>("scan_directory_quick", { path });
      entries.value = quickEntries;
      currentPath.value = path;
      rebuildBreadcrumbs(path);
      isLoading.value = false;  // 名稱已顯示，關閉 loading

      // 磁碟資訊非同步載入
      loadDiskInfo(path);

      // ── 第二階段：背景計算目錄大小 ───────────────────────────────────────
      const hasDirs = quickEntries.some(e => e.is_dir);
      if (!hasDirs) return;  // 沒有目錄，不需要計算

      isComputingSizes.value = true;

      // 建立 debounce 佇列與 rAF 機制以避免高頻更新癱瘓畫面
      let pendingSizeUpdates = new Map<string, { size: number, count: number }>();
      let sizeAnimationFrame: number | null = null;

      unlistenSizeUpdate = await listen<{ path: string; size: number; file_count: number }>(
        "size-update",
        ({ payload }) => {
          // 先存入佇列
          pendingSizeUpdates.set(payload.path, { size: payload.size, count: payload.file_count });

          // 透過 rAF 批次處理
          if (sizeAnimationFrame === null) {
            sizeAnimationFrame = requestAnimationFrame(() => {
              // 遍歷所有收到的大小的更新並一次性套用到 entries (Vue proxy object)
              pendingSizeUpdates.forEach(({ size, count }, path) => {
                const entry = entries.value.find(e => e.path === path);
                if (entry) {
                  entry.size = size;
                  entry.file_count = count;
                  entry.is_computing = false;
                }
              });
              // 清空佇列並釋放 frame id
              pendingSizeUpdates.clear();
              sizeAnimationFrame = null;
            });
          }
        }
      );

      // 監聽全部完成
      unlistenSizesComplete = await listen("sizes-complete", () => {
        if (sizeAnimationFrame !== null) {
          cancelAnimationFrame(sizeAnimationFrame);
          pendingSizeUpdates.forEach(({ size, count }, path) => {
            const entry = entries.value.find(e => e.path === path);
            if (entry) {
              entry.size = size;
              entry.file_count = count;
              entry.is_computing = false;
            }
          });
          pendingSizeUpdates.clear();
          sizeAnimationFrame = null;
        }

        isComputingSizes.value = false;
        // 計算完成後依大小重新排序
        entries.value = [...entries.value].sort((a, b) => b.size - a.size);
        cleanupSizeListeners();
      });

      // 啟動背景計算
      await invoke("compute_dir_sizes", { path });

    } catch (e) {
      error.value = String(e);
      isLoading.value = false;
      isComputingSizes.value = false;
    }
  }

  /** 清理大小計算的事件監聽器 */
  function cleanupSizeListeners() {
    unlistenSizeUpdate?.();
    unlistenSizeUpdate = null;
    unlistenSizesComplete?.();
    unlistenSizesComplete = null;
    isComputingSizes.value = false;
    // 清除所有 is_computing 旗標（防止殘留）
    entries.value.forEach(e => { if (e.is_computing) e.is_computing = false; });
  }

  /** Go up one directory level. */
  function goUp() {
    const sep = currentPath.value.includes("/") ? "/" : "\\";
    const parts = currentPath.value.split(sep).filter(Boolean);
    if (parts.length <= 1) return;
    parts.pop();

    let parent: string;
    if (currentPath.value.startsWith("/")) {
      parent = "/" + parts.join("/");
    } else {
      parent = parts.join("\\") + (parts.length === 1 ? "\\" : "");
    }

    navigateTo(parent);
  }

  /** Start a deep scan and stream results via events. */
  async function startDeepScan() {
    deepEntries.value = [];
    deepProcessed.value = 0;
    isDeepScanning.value = true;
    error.value = null;

    unlistenChunk?.();
    unlistenComplete?.();

    let pendingEntries: FileEntry[] = [];
    let pendingProcessed = 0;
    let frameId: number | null = null;

    unlistenChunk = await listen<{ entries: FileEntry[]; processed: number }>(
      "scan-chunk",
      ({ payload }) => {
        pendingEntries.push(...payload.entries);
        pendingProcessed = payload.processed;

        if (frameId === null) {
          frameId = requestAnimationFrame(() => {
            deepEntries.value.push(...pendingEntries);
            deepProcessed.value = pendingProcessed;
            pendingEntries = [];
            frameId = null;
          });
        }
      }
    );

    unlistenComplete = await listen("scan-complete", () => {
      isDeepScanning.value = false;
      if (frameId !== null) {
        cancelAnimationFrame(frameId);
        deepEntries.value.push(...pendingEntries);
        deepProcessed.value = pendingProcessed;
        pendingEntries = [];
        frameId = null;
      }
    });

    try {
      await invoke("start_deep_scan", { path: currentPath.value });
    } catch (e) {
      error.value = String(e);
      isDeepScanning.value = false;
    }
  }

  /** Cancel an in-progress deep scan. */
  async function cancelDeepScan() {
    await invoke("cancel_scan");
    isDeepScanning.value = false;
  }

  /** 删除一個路徑（rename + 背景移除）。失敗時 re-throw 含完整路徑的錯誤。 */
  async function deleteFast(path: string) {
    try {
      await invoke("delete_fast", { path });
      entries.value = entries.value.filter((e) => e.path !== path);
    } catch (e) {
      // 重新包裝錯誤訊息，確保上層可以看到完整內容
      const msg = typeof e === "string" ? e
        : (e instanceof Error ? e.message : JSON.stringify(e));
      throw new Error(`删除 '${path}' 失敗：${msg}`);
    }
  }

  /** 將路徑移入自訂垃圾桶。 */
  async function trashItem(path: string): Promise<TrashEntry> {
    try {
      const entry = await invoke<TrashEntry>("trash_item", { path });
      entries.value = entries.value.filter((e) => e.path !== path);
      return entry;
    } catch (e) {
      const msg = typeof e === "string" ? e
        : (e instanceof Error ? e.message : JSON.stringify(e));
      throw new Error(`移到垃圾桶 '${path}' 失敗：${msg}`);
    }
  }

  // ── Disk info ──────────────────────────────────────────────────────────────

  async function loadDiskInfo(path: string) {
    try {
      diskInfo.value = await invoke<DiskInfo>("get_disk_info", { path });
    } catch {
      diskInfo.value = null;
    }
  }

  // ── Breadcrumbs ────────────────────────────────────────────────────────────

  function rebuildBreadcrumbs(path: string) {
    const isWin = path.match(/^[A-Za-z]:\\/);
    if (isWin) {
      const parts = path.split("\\").filter(Boolean);
      breadcrumbs.value = parts.map((_, i) =>
        parts.slice(0, i + 1).join("\\") + (i === 0 ? "\\" : "")
      );
    } else {
      const parts = path.split("/").filter(Boolean);
      breadcrumbs.value = ["", ...parts].map((_, i) =>
        "/" + parts.slice(0, i).join("/")
      );
    }
  }

  return {
    currentPath,
    entries,
    breadcrumbs,
    isLoading,
    isComputingSizes,
    isDeepScanning,
    deepEntries,
    sortedDeepEntries,
    deepProcessed,
    diskInfo,
    error,
    navigateTo,
    goUp,
    startDeepScan,
    cancelDeepScan,
    deleteFast,
    trashItem,
  };
}

// ── Trash composable ──────────────────────────────────────────────────────────

export function useTrash() {
  const items = ref<TrashEntry[]>([]);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  async function load() {
    isLoading.value = true;
    try {
      items.value = await invoke<TrashEntry[]>("list_trash");
    } catch (e) {
      error.value = String(e);
    } finally {
      isLoading.value = false;
    }
  }

  async function restore(trash_id: string) {
    await invoke("restore_trash", { trash_id });
    items.value = items.value.filter((i) => i.trash_id !== trash_id);
  }

  async function purge(trash_id: string) {
    await invoke("purge_trash", { trash_id });
    items.value = items.value.filter((i) => i.trash_id !== trash_id);
  }

  async function emptyAll(): Promise<number> {
    const freed = await invoke<number>("empty_trash");
    items.value = [];
    return freed;
  }

  return { items, isLoading, error, load, restore, purge, emptyAll };
}
