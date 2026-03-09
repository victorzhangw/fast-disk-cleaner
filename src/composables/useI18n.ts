// composables/useI18n.ts
// 繁體中文 / 英文 雙語支援

import { ref, computed } from "vue";

export type Locale = "zh-TW" | "en";

const messages = {
  "zh-TW": {
    // 標題列
    appName: "快速磁碟清理",
    pathPlaceholder: "輸入路徑…",
    goBtn: "前往",
    upBtn: "上層",
    // 分頁
    tabBrowser: "📂 瀏覽器",
    tabDeepScan: "🔬 深層掃描",
    tabTrash: "🗑 垃圾桶",
    // 工具列
    items: "個項目",
    refresh: "↻ 重新整理",
    scanningLabel: "掃描中：",
    itemsFound: "個項目…",
    showingTop: "個項目（顯示前 500 筆）",
    startDeepScan: "▶ 開始深層掃描",
    cancelBtn: "■ 取消",
    // 狀態列
    loading: "載入中…",
    ready: "就緒",
    // toast / 訊息
    movedToTrash: "已移至垃圾桶：",
    movedNToTrash: "已移動 {n} 個項目至垃圾桶",
    deleted: "已刪除：",
    deletedN: "已刪除 {n} 個項目",
    deleteFailedN: "{n} 個刪除失敗",
    // 垃圾桶
    trashTitle: "垃圾桶",
    trashEmpty: "垃圾桶是空的",
    restore: "還原",
    delete: "永久刪除",
    emptyTrash: "清空垃圾桶",
    emptyTrashConfirm: "確定要清空垃圾桶嗎？",
    originalPath: "原始路徑",
    deletedAt: "刪除時間",
    // 設定面板
    settingsTitle: "顯示設定",
    fontSize: "字型大小",
    fontFamily: "字型",
    version: "版本",
    // 磁碟機切換
    selectDrive: "切換磁碟機",
    driveInfo: "容量",
    // 歡迎畫面
    welcome: "歡迎使用 FastDiskCleaner",
    welcomeTip: "請選擇磁碟機或輸入路徑，然後點擊「掃描」開始。",
    scanBtn: "🔍 開始掃描",
    // 主題 / 語言切換
    themeToggle: "切換主題",
    langToggle: "English",
    // 磁碟使用
    used: "已使用",
    free: "可用",
    total: "總容量",
    // FileList
    name: "名稱",
    size: "大小",
    files: "檔案數",
    modified: "修改時間",
    trashAction: "移至垃圾桶",
    deleteAction: "刪除",
    trashSelected: "移至垃圾桶（已選）",
    deleteSelected: "刪除（已選）",
    noItems: "此目錄為空",
  },
  "en": {
    appName: "FastDiskCleaner",
    pathPlaceholder: "Enter path…",
    goBtn: "Go",
    upBtn: "Up",
    tabBrowser: "📂 Browser",
    tabDeepScan: "🔬 Deep Scan",
    tabTrash: "🗑 Trash",
    items: " items",
    refresh: "↻ Refresh",
    scanningLabel: "Scanning: ",
    itemsFound: " items found…",
    showingTop: " items (showing top 500 by size)",
    startDeepScan: "▶ Start Deep Scan",
    cancelBtn: "■ Cancel",
    loading: "Loading…",
    ready: "Ready",
    movedToTrash: "Moved to trash: ",
    movedNToTrash: "Moved {n} items to trash",
    deleted: "Deleted: ",
    deletedN: "Deleted {n} items",
    deleteFailedN: "{n} delete(s) failed",
    trashTitle: "Trash",
    trashEmpty: "Trash is empty",
    restore: "Restore",
    delete: "Delete permanently",
    emptyTrash: "Empty Trash",
    emptyTrashConfirm: "Are you sure you want to empty the trash?",
    originalPath: "Original Path",
    deletedAt: "Deleted At",
    // 設定面板
    settingsTitle: "Display Settings",
    fontSize: "Font Size",
    fontFamily: "Font Family",
    version: "Version",
    // 磁碟機切換
    selectDrive: "Switch Drive",
    driveInfo: "Capacity",
    // 歡迎畫面
    welcome: "Welcome to FastDiskCleaner",
    welcomeTip: "Select a drive or enter a path, then click Scan.",
    scanBtn: "🔍 Start Scan",
    // 主題 / 語言切換
    themeToggle: "Toggle Theme",
    langToggle: "中文",
    // 磁碟使用
    used: "Used",
    free: "Free",
    total: "Total",
    // FileList
    name: "Name",
    size: "Size",
    files: "Files",
    modified: "Modified",
    trashAction: "Trash",
    deleteAction: "Delete",
    trashSelected: "Trash selected",
    deleteSelected: "Delete selected",
    noItems: "This directory is empty",
  },
} as const;

// 全域語言狀態（單例，跨元件共用）
const locale = ref<Locale>("zh-TW");

export function useI18n() {
  const t = computed(() => messages[locale.value]);

  function toggleLocale() {
    locale.value = locale.value === "zh-TW" ? "en" : "zh-TW";
  }

  function fmt(key: keyof typeof messages["en"], vars?: Record<string, string | number>): string {
    let str = (t.value as Record<string, string>)[key as string] ?? key;
    if (vars) {
      for (const [k, v] of Object.entries(vars)) {
        str = str.replace(`{${k}}`, String(v));
      }
    }
    return str;
  }

  return { t, locale, toggleLocale, fmt };
}
