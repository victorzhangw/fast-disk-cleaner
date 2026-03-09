<template>
  <div class="app">
    <!-- ── 標題列 ──────────────────────────────────────────────────────────── -->
    <header class="title-bar">
      <!-- 左側：Logo + 磁碟機選擇器 + 路徑輸入 -->
      <div class="tb-left-group">
        <span class="tb-logo">⚡</span>
        <span class="tb-name mono">{{ t.appName }}</span>

        <!-- 磁碟機下拉選擇器 -->
        <div class="drive-selector" v-if="drives.length > 0">
          <select class="drive-select" v-model="selectedDrive" @change="onDriveChange">
            <option v-for="d in drives" :key="d.path" :value="d.path">
              {{ d.label }} ({{ formatBytes(d.available) }} {{ t.free }})
            </option>
          </select>
        </div>
      </div>

      <!-- 中間：麵包屑 + 路徑輸入 -->
      <div class="tb-path">
        <button class="btn-icon-sm" :title="t.upBtn" @click="goUp" :disabled="breadcrumbs.length <= 1">↑</button>
        <div class="breadcrumbs">
          <template v-for="(crumb, i) in breadcrumbs" :key="crumb">
            <span class="crumb-sep" v-if="i > 0">›</span>
            <button
              class="crumb"
              :class="{ active: i === breadcrumbs.length - 1 }"
              @click="triggerNavigate(crumb)"
            >{{ crumbLabel(crumb, i) }}</button>
          </template>
        </div>
        <input
          class="path-input"
          v-model="pathInput"
          @keydown.enter="triggerNavigate(pathInput)"
          :placeholder="t.pathPlaceholder"
          spellcheck="false"
        />
        <button class="btn-primary" @click="triggerNavigate(pathInput)">{{ t.goBtn }}</button>
      </div>

      <!-- 右側：語言 / 主題 / 設定 -->
      <div class="tb-right">
        <button class="btn-icon-pill" @click="toggleLocale">🌐 {{ t.langToggle }}</button>
        <button class="btn-icon-pill" @click="toggleTheme">{{ theme === 'dark' ? '☀️' : '🌙' }}</button>
        <button class="btn-icon-pill" :class="{ active: showSettings }" @click="showSettings = !showSettings">⚙</button>
      </div>
    </header>

    <!-- ── 分頁列 ────────────────────────────────────────────────────────── -->
    <nav class="tab-bar">
      <button class="tab" :class="{ active: activeTab === 'browser' }" @click="activeTab = 'browser'">
        {{ t.tabBrowser }}
      </button>
      <button class="tab" :class="{ active: activeTab === 'deep' }" @click="activeTab = 'deep'">
        {{ t.tabDeepScan }}
      </button>
      <button class="tab" :class="{ active: activeTab === 'trash' }" @click="activeTab = 'trash'">
        {{ t.tabTrash }}
      </button>
      <div class="tab-spacer" />
      <!-- 版本號 -->
      <span class="version-badge mono">v{{ appVersion }}</span>
    </nav>

    <!-- ── 磁碟使用量 ─────────────────────────────────────────────────────── -->
    <DiskUsage
      :info="diskInfo"
      :i18n="t"
      :current-path="currentPath"
      @navigate="triggerNavigate"
      @go-home="goHome"
    />


    <!-- ── 主內容 ─────────────────────────────────────────────────────────── -->
    <div class="body-row">
      <main class="main-content">

        <!-- BROWSER 分頁 -->
        <template v-if="activeTab === 'browser'">
          <!-- 歡迎畫面（尚未掃描） -->
          <div class="welcome-screen" v-if="!hasScanned && !isLoading">
            <div class="welcome-icon">⚡</div>
            <h2 class="welcome-title">{{ t.welcome }}</h2>
            <p class="welcome-tip">{{ t.welcomeTip }}</p>
            <div class="welcome-drives" v-if="drives.length > 0">
              <button
                v-for="d in drives"
                :key="d.path"
                class="drive-card"
                @click="selectAndScan(d.path)"
              >
                <div class="drive-card-label">{{ d.label }}</div>
                <div class="drive-card-bar">
                  <div class="drive-card-fill" :style="usePctStyle(d)" />
                </div>
                <div class="drive-card-info">
                  <span class="amber">{{ formatBytes(d.total - d.available) }}</span>
                  <span class="dim"> / {{ formatBytes(d.total) }}</span>
                </div>
              </button>
            </div>
            <button class="btn-primary scan-big-btn" @click="selectAndScan(pathInput)">{{ t.scanBtn }}</button>
          </div>

          <!-- 正常瀏覽模式 -->
          <template v-else>
            <div class="content-toolbar">
              <span class="dim mono" style="font-size:11px">
                {{ entries.length.toLocaleString() }}{{ t.items }}
              </span>
              <div class="toolbar-actions">
                <!-- 第二階段：目錄大小計算中 -->
                <span class="dim mono" v-if="isComputingSizes" style="font-size:11px">
                  <span class="scanning-dot" /> 計算容量中…
                </span>
                <button class="btn-ghost" @click="triggerNavigate(currentPath)">{{ t.refresh }}</button>
              </div>
            </div>

            <FileList
              :entries="entries"
              :is-loading="isLoading"
              :i18n="t"
              @navigate="triggerNavigate"
              @trash="handleTrash"
              @trash-batch="handleTrashBatch"
              @delete="handleDelete"
              @delete-batch="handleDeleteBatch"
            />
          </template>
        </template>

        <!-- DEEP SCAN 分頁 -->
        <template v-else-if="activeTab === 'deep'">
          <div class="deep-toolbar">
            <div>
              <span class="dim">{{ t.scanningLabel }}</span>
              <span class="mono amber">{{ currentPath }}</span>
            </div>
            <div class="toolbar-actions">
              <span class="dim mono" v-if="isDeepScanning" style="font-size:11px">
                <span class="scanning-dot" />
                {{ deepProcessed.toLocaleString() }}{{ t.itemsFound }}
              </span>
              <span class="dim mono" v-else-if="sortedDeepEntries.length > 0" style="font-size:11px">
                {{ sortedDeepEntries.length.toLocaleString() }}{{ t.showingTop }}
              </span>
              <button class="btn-primary" v-if="!isDeepScanning" @click="startDeepScan">
                {{ t.startDeepScan }}
              </button>
              <button class="btn-ghost" v-else @click="cancelDeepScan">
                {{ t.cancelBtn }}
              </button>
            </div>
          </div>

          <FileList
            :entries="sortedDeepEntries"
            :is-loading="isDeepScanning && sortedDeepEntries.length === 0"
            :i18n="t"
            @navigate="navigateTo"
            @trash="handleTrash"
            @trash-batch="handleTrashBatch"
            @delete="handleDelete"
            @delete-batch="handleDeleteBatch"
          />
        </template>

        <!-- TRASH 分頁 -->
        <template v-else-if="activeTab === 'trash'">
          <TrashManager :i18n="t" />
        </template>
      </main>

      <!-- ── 設定面板（右側滑出） ─────────────────────────────────────────── -->
      <Transition name="slide-right">
        <SettingsPanel
          v-if="showSettings"
          :i18n="t"
          :app-version="appVersion"
          @close="showSettings = false"
        />
      </Transition>
    </div>

    <!-- ── 狀態列 ─────────────────────────────────────────────────────────── -->
    <footer class="status-bar">
      <span class="mono dim" style="font-size:11px">
        <span v-if="isLoading"><span class="scanning-dot" /> {{ t.loading }}</span>
        <span v-else-if="lastAction" class="green">✓ {{ lastAction }}</span>
        <span v-else>{{ t.ready }}</span>
      </span>
      <span class="mono dim" style="font-size:11px">{{ currentPath }}</span>
    </footer>

    <!-- ── 全域錯誤 Toast ──────────────────────────────────────────────────── -->
    <ErrorToast :error="error" @clear="error = null" />

  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";

import DiskUsage    from "./components/DiskUsage.vue";
import FileList     from "./components/FileList.vue";
import TrashManager from "./components/TrashManager.vue";
import SettingsPanel from "./components/SettingsPanel.vue";
import ErrorToast   from "./components/ErrorToast.vue";

import { useScanner, formatBytes } from "./composables/useScanner";
import { useTheme }    from "./composables/useTheme";
import { useI18n }     from "./composables/useI18n";

// ── Tauri 錯誤解析 helper ────────────────────────────────────────
// Tauri invoke 失敗時，e 可能是 string 或 { message, code } 等不同格式
function parseInvokeError(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  if (e && typeof e === "object") {
    const obj = e as Record<string, unknown>;
    // Tauri v2 形式： { message: string } 或直接是 error string
    if (typeof obj.message === "string") return obj.message;
    if (typeof obj.error === "string") return obj.error;
    try { return JSON.stringify(e); } catch { /* ignore */ }
  }
  return String(e);
}

// ── 主題 & 語言 ───────────────────────────────────────────────────────────────

const { theme, toggleTheme } = useTheme();
const { t, toggleLocale }    = useI18n();

// ── 掃描器 ────────────────────────────────────────────────────────────────────

const {
  currentPath,
  entries,
  breadcrumbs,
  isLoading,
  isComputingSizes,
  isDeepScanning,
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
} = useScanner();

// ── UI 狀態 ───────────────────────────────────────────────────────────────────

const activeTab   = ref<"browser" | "deep" | "trash">("browser");
const pathInput   = ref("C:\\");
const lastAction  = ref("");
const showSettings = ref(false);
const hasScanned  = ref(false);
const appVersion  = ref("0.1.0");

// 磁碟機列表
interface DriveInfo { path: string; label: string; total: number; available: number; }
const drives = ref<DriveInfo[]>([]);
const selectedDrive = ref("C:\\");

// 路徑輸入跟對當前路徑同步
watch(currentPath, (p) => { pathInput.value = p; });

// 有操作後 3 秒自動清除訊息
watch(lastAction, (v) => {
  if (v) setTimeout(() => { if (lastAction.value === v) lastAction.value = ""; }, 3000);
});

// ── 初始化 ────────────────────────────────────────────────────────────────────

onMounted(async () => {
  // 取得版本號
  try {
    appVersion.value = await invoke<string>("app_version");
  } catch {}

  // 列印磁碟機清單
  try {
    drives.value = await invoke<DriveInfo[]>("list_drives");
    if (drives.value.length > 0) {
      selectedDrive.value = drives.value[0].path;
      pathInput.value = drives.value[0].path;
    }
  } catch {}

  // ★ 不自動掃描，等使用者點擊
});

// ── 磁碟機切換 ────────────────────────────────────────────────────────────────

function onDriveChange() {
  pathInput.value = selectedDrive.value;
  hasScanned.value = false;
}

async function selectAndScan(path: string) {
  hasScanned.value = true;   // ★ 立即設定，避免閃回歡迎畫面
  pathInput.value = path;
  selectedDrive.value = path;
  await navigateTo(path);
}

// ── 磁碟使用率 helper ─────────────────────────────────────────────────────────

function usePctStyle(d: DriveInfo): { width: string; background: string } {
  if (d.total === 0) return { width: "0%", background: "var(--text-3)" };
  const pct = ((d.total - d.available) / d.total) * 100;
  return {
    width: `${pct}%`,
    background: pct > 90 ? "var(--red)" : pct > 70 ? "var(--amber)" : "var(--green)",
  };
}

// ── 事件處理 ──────────────────────────────────────────────────────────────────

// 指定路徑手動掃描
async function triggerNavigate(path: string) {
  hasScanned.value = true;   // ★ 先設定，避免閃回歡迎畫面
  await navigateTo(path);
}

// 返回磁碟機選擇（歡迎畫面）
function goHome() {
  hasScanned.value = false;
  activeTab.value = "browser";
}


async function handleTrash(path: string) {
  try {
    await trashItem(path);
    lastAction.value = t.value.movedToTrash + (path.split("\\").pop() ?? path);
  } catch (e) {
    error.value = `移動失敗：${parseInvokeError(e)}`;
  }
}

async function handleTrashBatch(paths: string[]) {
  try {
    for (const p of paths) await trashItem(p);
    lastAction.value = t.value.movedNToTrash.replace("{n}", String(paths.length));
  } catch (e) { error.value = String(e); }
}

async function handleDelete(path: string) {
  try {
    await deleteFast(path);
    lastAction.value = t.value.deleted + (path.split("\\").pop() ?? path);
  } catch (e) {
    error.value = `刪除失敗：${parseInvokeError(e)}`;
  }
}

async function handleDeleteBatch(paths: string[]) {
  try {
    const results = await invoke<{ path: string; success: boolean; error: string | null }[]>(
      "batch_delete", { paths }
    );
    
    // 更新清單：過濾掉成功刪除的項目
    const succeededPaths = new Set(results.filter((r) => r.success).map((r) => r.path));
    if (succeededPaths.size > 0) {
      entries.value = entries.value.filter((e) => !succeededPaths.has(e.path));
    }

    const failed = results.filter((r) => !r.success);
    if (failed.length > 0) {
      // 顯示第一個失敗的具體原因
      const firstErr = failed[0].error ?? "原因不明";
      error.value = `刪除失敗 (${failed.length}/${results.length})：${firstErr}`;
    } else {
      lastAction.value = t.value.deletedN.replace("{n}", String(results.length));
    }
  } catch (e) {
    error.value = `刪除命令失敗：${parseInvokeError(e)}`;
  }
}

// ── 麵包屑標籤 ───────────────────────────────────────────────────────────────

function crumbLabel(crumb: string, idx: number): string {
  if (idx === 0) return crumb.endsWith("\\") ? crumb.slice(0, -1) : crumb || "/";
  return crumb.split(/[\\\/]/).filter(Boolean).pop() ?? crumb;
}
</script>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  background: var(--bg-0);
}

/* ── 標題列 ──────────────────────────────────────────────────────────────── */
.title-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 10px;
  height: 44px;
  background: var(--bg-1);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.tb-left-group {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}
.tb-logo { font-size: 16px; }
.tb-name { font-size: 13px; font-weight: 600; color: var(--amber); white-space: nowrap; }

/* 磁碟機選擇器 */
.drive-selector { margin-left: 4px; }
.drive-select {
  background: var(--bg-2);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  color: var(--text-1);
  font-family: var(--font-mono);
  font-size: 11px;
  padding: 4px 8px;
  cursor: pointer;
  outline: none;
  transition: border-color 0.15s;
  max-width: 160px;
}
.drive-select:focus { border-color: var(--amber); }

/* 路徑區塊 */
.tb-path {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
}
.breadcrumbs {
  display: flex;
  align-items: center;
  gap: 2px;
  overflow: hidden;
  flex: 1;
}
.crumb {
  background: none; border: none;
  padding: 2px 4px;
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-2);
  cursor: pointer;
  border-radius: 2px;
  white-space: nowrap;
}
.crumb:hover { color: var(--amber); }
.crumb.active { color: var(--text-1); }
.crumb-sep { color: var(--text-3); font-size: 10px; }

.path-input { width: 180px; flex-shrink: 0; }

/* 右側按鈕群組 */
.tb-right {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}
.btn-icon-sm {
  background: none; border: 1px solid var(--border);
  color: var(--text-2); padding: 3px 8px;
  border-radius: 3px; font-size: 12px; cursor: pointer;
}
.btn-icon-sm:hover:not(:disabled) { border-color: var(--border-hi); color: var(--text-1); }
.btn-icon-sm:disabled { opacity: 0.3; cursor: not-allowed; }

.btn-icon-pill {
  background: var(--bg-2); border: 1px solid var(--border);
  color: var(--text-2); padding: 4px 10px;
  border-radius: 20px; font-size: 11px; cursor: pointer;
  transition: background 0.15s, border-color 0.15s, color 0.15s;
}
.btn-icon-pill:hover { background: var(--bg-3); border-color: var(--border-hi); color: var(--text-1); }
.btn-icon-pill.active { background: var(--amber-glow); border-color: var(--amber); color: var(--amber); }

/* ── 分頁列 ──────────────────────────────────────────────────────────────── */
.tab-bar {
  display: flex;
  align-items: center;
  padding: 0 12px;
  background: var(--bg-1);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}
.tab {
  background: none; border: none;
  border-bottom: 2px solid transparent;
  padding: 8px 14px; font-size: 12px; color: var(--text-2); cursor: pointer;
  transition: color 0.15s, border-color 0.15s;
}
.tab:hover { color: var(--text-1); }
.tab.active { color: var(--amber); border-bottom-color: var(--amber); }
.tab-spacer { flex: 1; }
.version-badge {
  font-size: 10px; color: var(--text-3);
  padding: 2px 8px; border-radius: 10px;
  background: var(--bg-2); border: 1px solid var(--border);
}

/* ── Body row (main + settings panel) ────────────────────────────────────── */
.body-row {
  flex: 1;
  display: flex;
  overflow: hidden;
}

/* ── 主內容 ──────────────────────────────────────────────────────────────── */
.main-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background: var(--bg-0);
}
.content-toolbar, .deep-toolbar {
  display: flex; align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-1); flex-shrink: 0;
}
.toolbar-actions { display: flex; align-items: center; gap: 8px; }

/* ── 歡迎畫面 ────────────────────────────────────────────────────────────── */
.welcome-screen {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 40px 24px;
  overflow-y: auto;
}
.welcome-icon { font-size: 56px; line-height: 1; filter: drop-shadow(0 0 20px rgba(245,166,35,.4)); }
.welcome-title { font-size: 22px; font-weight: 700; color: var(--text-1); }
.welcome-tip { color: var(--text-2); font-size: 13px; text-align: center; }

.welcome-drives {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  justify-content: center;
  margin: 8px 0;
}
.drive-card {
  background: var(--bg-2);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 14px 20px;
  min-width: 140px;
  cursor: pointer;
  text-align: left;
  transition: border-color 0.15s, background 0.15s, transform 0.1s;
}
.drive-card:hover {
  border-color: var(--amber);
  background: var(--bg-3);
  transform: translateY(-2px);
}
.drive-card-label { font-size: 18px; font-weight: 700; font-family: var(--font-mono); color: var(--amber); margin-bottom: 8px; }
.drive-card-bar { height: 4px; background: var(--bg-3); border-radius: 2px; overflow: hidden; margin-bottom: 6px; }
.drive-card-fill { height: 100%; border-radius: 2px; transition: width 0.3s; }
.drive-card-info { font-size: 11px; font-family: var(--font-mono); }

.scan-big-btn { padding: 10px 32px; font-size: 15px; margin-top: 4px; }

/* ── 列表與主畫面樣式 ──────────────────────────────────────────────────────── */

/* ── 狀態列 ──────────────────────────────────────────────────────────────── */
.status-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 5px 12px;
  background: var(--bg-1);
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

/* ── Transitions ──────────────────────────────────────────────────────────── */
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s, transform 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; transform: translateX(-50%) translateY(10px); }

.slide-right-enter-active, .slide-right-leave-active { transition: width 0.25s ease, opacity 0.2s; overflow: hidden; }
.slide-right-enter-from, .slide-right-leave-to { width: 0 !important; opacity: 0; }
</style>
