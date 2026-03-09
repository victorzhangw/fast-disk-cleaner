<template>
  <div class="file-list">
    <!-- ── 欄位標頭（可點擊排序）────────────────────────────────────────── -->
    <div class="fl-header">
      <!-- 全選 checkbox -->
      <div class="col-check" @click.stop>
        <input
          type="checkbox"
          class="row-checkbox"
          :checked="isAllSelected"
          :indeterminate="isIndeterminate"
          @change="toggleAll"
          title="全選 / 取消全選"
        />
      </div>
      <div class="col-icon" />
      <!-- 名稱欄（靠左） -->
      <div class="col-name sort-col sort-col-left" @click="setSort('name')">
        {{ props.i18n?.name ?? 'Name' }}
        <span class="sort-arrow">{{ sortArrow('name') }}</span>
      </div>
      <!-- 大小欄（置中） -->
      <div class="col-size sort-col sort-col-center" @click="setSort('size')">
        <span class="sort-arrow">{{ sortArrow('size') }}</span>
        {{ props.i18n?.size ?? 'Size' }}
      </div>
      <!-- 檔案數欄（置中） -->
      <div class="col-files sort-col sort-col-center" @click="setSort('files')">
        <span class="sort-arrow">{{ sortArrow('files') }}</span>
        {{ props.i18n?.files ?? 'Files' }}
      </div>
      <!-- 修改時間欄（靠左） -->
      <div class="col-modified sort-col sort-col-left" @click="setSort('modified')">
        {{ props.i18n?.modified ?? 'Modified' }}
        <span class="sort-arrow">{{ sortArrow('modified') }}</span>
      </div>
      <div class="col-bar">Usage</div>
      <div class="col-actions" />
    </div>

    <!-- Loading skeleton -->
    <template v-if="isLoading">
      <div class="fl-row skeleton-row" v-for="i in 12" :key="i">
        <div class="col-check"><span class="skeleton" style="width:14px;height:14px;border-radius:3px" /></div>
        <div class="col-icon"><span class="skeleton icon-skel" /></div>
        <div class="col-name"><span class="skeleton" :style="`width:${60 + i * 7}px;height:12px`" /></div>
        <div class="col-size"><span class="skeleton" style="width:52px;height:12px" /></div>
        <div class="col-files"><span class="skeleton" style="width:36px;height:12px" /></div>
        <div class="col-modified"><span class="skeleton" style="width:100px;height:12px" /></div>
        <div class="col-bar"><span class="skeleton" style="width:100%;height:6px;border-radius:3px" /></div>
        <div class="col-actions" />
      </div>
    </template>

    <!-- 空目錄狀態 -->
    <div class="fl-empty" v-else-if="entries.length === 0">
      <span class="dim mono">{{ props.i18n?.noItems ?? '[ empty directory ]' }}</span>
    </div>

    <!-- File rows: Virtualized -->
    <template v-else>
      <div 
        ref="parentRef"
        class="virtual-container"
      >
        <div 
          :style="{
            height: `${virtualizer.getTotalSize()}px`,
            width: '100%',
            position: 'relative',
          }"
        >
          <div
            v-for="virtualRow in virtualizer.getVirtualItems()"
            :key="String(virtualRow.key)"
            class="fl-row"
            :class="{ selected: selected.has(sortedEntries[virtualRow.index].path) }"
            :style="{
              position: 'absolute',
              top: 0,
              left: 0,
              width: '100%',
              height: `${virtualRow.size}px`,
              transform: `translateY(${virtualRow.start}px)`,
            }"
            @dblclick="sortedEntries[virtualRow.index].is_dir && $emit('navigate', sortedEntries[virtualRow.index].path)"
          >
            <!-- Checkbox -->
            <div class="col-check" @click.stop>
              <input
                type="checkbox"
                class="row-checkbox"
                :checked="selected.has(sortedEntries[virtualRow.index].path)"
                @change="toggleSelect(sortedEntries[virtualRow.index])"
              />
            </div>

            <div class="col-icon">
              <span class="icon">{{ sortedEntries[virtualRow.index].is_dir ? "📁" : fileIcon(sortedEntries[virtualRow.index].name) }}</span>
            </div>

            <div class="col-name">
              <span
                class="entry-name"
                :class="{ 'dir-name': sortedEntries[virtualRow.index].is_dir }"
                @click.stop="sortedEntries[virtualRow.index].is_dir && $emit('navigate', sortedEntries[virtualRow.index].path)"
              >{{ sortedEntries[virtualRow.index].name }}</span>
            </div>

            <div class="col-size mono">
              <span v-if="sortedEntries[virtualRow.index].is_computing" class="computing-skel" />
              <span v-else :class="sizeClass(sortedEntries[virtualRow.index].size)">{{ formatBytes(sortedEntries[virtualRow.index].size) }}</span>
            </div>

            <div class="col-files mono dim">{{ sortedEntries[virtualRow.index].file_count.toLocaleString() }}</div>

            <div class="col-modified dim mono" style="font-size:11px">
              {{ formatDate(sortedEntries[virtualRow.index].modified) }}
            </div>

            <!-- Proportional size bar -->
            <div class="col-bar">
              <div class="size-bar-track">
                <div
                  class="size-bar-fill"
                  :style="{
                    width: barWidth(sortedEntries[virtualRow.index].size) + '%',
                    background: barColor(sortedEntries[virtualRow.index].size),
                  }"
                />
              </div>
            </div>

            <!-- Action buttons (appear on hover) -->
            <div class="col-actions" @click.stop>
              <button class="btn-icon" :title="props.i18n?.trashAction ?? 'Trash'" @click="$emit('trash', sortedEntries[virtualRow.index].path)">🗑</button>
              <button class="btn-icon btn-icon-danger" :title="props.i18n?.deleteAction ?? 'Delete'" @click="confirmDelete(sortedEntries[virtualRow.index])">✕</button>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- Batch action bar -->
    <Transition name="slide-up">
      <div class="batch-bar" v-if="selected.size > 0">
        <span class="dim">{{ selected.size }} selected · {{ formatBytes(selectedSize) }}</span>
        <div class="batch-actions">
          <button class="btn-ghost" @click="selected.clear()">Deselect</button>
          <button class="btn-ghost" @click="$emit('trash-batch', [...selected])">
            🗑 {{ props.i18n?.trashSelected ?? 'Trash selected' }}
          </button>
          <button class="btn-danger" @click="$emit('delete-batch', [...selected])">
            ✕ {{ props.i18n?.deleteSelected ?? 'Delete selected' }}
          </button>
        </div>
      </div>
    </Transition>

    <!-- Delete confirmation modal -->
    <Teleport to="body">
      <div class="modal-overlay" v-if="confirmEntry" @click="confirmEntry = null">
        <div class="modal" @click.stop>
          <h3 class="modal-title">⚠ {{ props.i18n?.deleteAction ?? 'Delete' }}</h3>
          <p class="dim" style="margin:12px 0">
            This cannot be undone. Permanently delete:
          </p>
          <p class="mono amber" style="word-break:break-all">{{ confirmEntry.path }}</p>
          <p class="dim" style="margin-top:6px;font-size:11px">{{ formatBytes(confirmEntry.size) }}</p>
          <div class="modal-actions">
            <button class="btn-ghost" @click="confirmEntry = null">Cancel</button>
            <button class="btn-danger" @click="doDelete">Delete Forever</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useVirtualizer } from '@tanstack/vue-virtual';
import type { FileEntry } from "../composables/useScanner";
import { formatBytes, formatDate } from "../composables/useScanner";

// ── 排序型別 ──────────────────────────────────────────────────────────────────

type SortKey = "name" | "size" | "files" | "modified";
type SortDir = "asc" | "desc";

const props = defineProps<{
  entries: FileEntry[];
  isLoading: boolean;
  i18n?: Record<string, string>;
}>();

const emit = defineEmits<{
  navigate:      [path: string];
  trash:         [path: string];
  "trash-batch": [paths: string[]];
  delete:        [path: string];
  "delete-batch":[paths: string[]];
}>();

// ── 排序狀態 ──────────────────────────────────────────────────────────────────

const sortKey = ref<SortKey>("size");
const sortDir = ref<SortDir>("desc");

function setSort(key: SortKey) {
  if (sortKey.value === key) {
    sortDir.value = sortDir.value === "asc" ? "desc" : "asc";
  } else {
    sortKey.value = key;
    sortDir.value = key === "name" ? "asc" : "desc";
  }
}

function sortArrow(key: SortKey): string {
  if (sortKey.value !== key) return " ↕";
  return sortDir.value === "asc" ? " ↑" : " ↓";
}

const sortedEntries = computed(() => {
  const dir = sortDir.value === "asc" ? 1 : -1;

  // 「名稱」排序：資料夾群組內排序 + 檔案群組內排序，不混合
  if (sortKey.value === "name") {
    const dirs  = props.entries.filter(e => e.is_dir);
    const files = props.entries.filter(e => !e.is_dir);
    const cmp = (a: FileEntry, b: FileEntry) =>
      dir * a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: "base" });
    return [...dirs.sort(cmp), ...files.sort(cmp)];
  }

  const arr = [...props.entries];
  return arr.sort((a, b) => {
    switch (sortKey.value) {
      case "size":
        // 計算中的目錄排在最後
        if (a.is_computing && !b.is_computing) return 1;
        if (!a.is_computing && b.is_computing) return -1;
        // 大小排序也保持「目錄優先」規則（同大小時目錄在前）
        if (a.is_dir !== b.is_dir && a.size === b.size)
          return a.is_dir ? -1 : 1;
        return dir * (a.size - b.size);
      case "files":
        return dir * (a.file_count - b.file_count);
      case "modified": {
        const ta = a.modified ? new Date(a.modified).getTime() : 0;
        const tb = b.modified ? new Date(b.modified).getTime() : 0;
        return dir * (ta - tb);
      }
      default:
        return 0;
    }
  });
});

// ── 虛擬捲動 (Virtual Scrolling) ──────────────────────────────────────────────────
const parentRef = ref<HTMLElement | null>(null);

const virtualizer = useVirtualizer(
  computed(() => ({
    count: sortedEntries.value.length,
    getScrollElement: () => parentRef.value,
    estimateSize: () => 35, // .fl-row 的高度 (約35px)
    overscan: 10,
  }))
);

// ── Selection ─────────────────────────────────────────────────────────────────

const selected = ref(new Set<string>());

// 自動清理已不存在的項目 (防抖優化, 避免大量資料頻繁清理導致卡頓)
let cleanupTimer: number | null = null;
watch(() => props.entries, (newEntries) => {
  if (selected.value.size === 0) return;

  if (cleanupTimer) clearTimeout(cleanupTimer);
  cleanupTimer = setTimeout(() => {
    // 延遲清理以減少大目錄更新時的 CPU 開銷
    const nextSelected = new Set<string>();
    // 當前顯示中的有效路徑
    const currentPaths = new Set(newEntries.map(e => e.path));
    let changed = false;

    for (const p of selected.value) {
      if (currentPaths.has(p)) {
        nextSelected.add(p);
      } else {
        changed = true;
      }
    }
    
    if (changed) {
      selected.value = nextSelected;
    }
  }, 150) as unknown as number;
});

/** Toggle single item selection */
function toggleSelect(entry: FileEntry) {
  const s = new Set(selected.value);
  if (s.has(entry.path)) s.delete(entry.path);
  else s.add(entry.path);
  selected.value = s;
}

/** Select / Deselect all */
function toggleAll() {
  if (isAllSelected.value) {
    selected.value = new Set();
  } else {
    selected.value = new Set(sortedEntries.value.map(e => e.path));
  }
}

const isAllSelected = computed(() =>
  sortedEntries.value.length > 0 &&
  selected.value.size === sortedEntries.value.length // O(1) 優化
);

const isIndeterminate = computed(() =>
  selected.value.size > 0 && !isAllSelected.value
);

const selectedSize = computed(() => {
  if (selected.value.size === 0) return 0;
  // 優化：只對已選取的項目做疊加，減少完全走訪整包 entries 陣列的次數
  let total = 0;
  // 建立快速查詢辭典
  const entryMap = new Map();
  for (let i = 0; i < props.entries.length; i++) {
    const e = props.entries[i];
    if (selected.value.has(e.path)) {
      total += e.size;
    }
  }
  return total;
});

// ── Delete confirmation ───────────────────────────────────────────────────────

const confirmEntry = ref<FileEntry | null>(null);

function confirmDelete(entry: FileEntry) {
  confirmEntry.value = entry;
}

function doDelete() {
  if (confirmEntry.value) {
    emit("delete", confirmEntry.value.path);
    confirmEntry.value = null;
  }
}

// ── Size bar helpers ──────────────────────────────────────────────────────────

const maxSize = computed(() =>
  props.entries.reduce((m, e) => Math.max(m, e.size), 1)
);

function barWidth(size: number): number {
  return (size / maxSize.value) * 100;
}

function barColor(size: number): string {
  const pct = size / maxSize.value;
  if (pct > 0.6) return "var(--amber)";
  if (pct > 0.3) return "var(--blue)";
  return "var(--text-3)";
}

function sizeClass(size: number): string {
  const pct = size / maxSize.value;
  if (pct > 0.6) return "amber";
  return "";
}

// ── File type icons ───────────────────────────────────────────────────────────

function fileIcon(name: string): string {
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  const map: Record<string, string> = {
    rs: "🦀", ts: "📘", js: "📒", vue: "💚",
    zip: "📦", tar: "📦", gz: "📦", rar: "📦", "7z": "📦",
    mp4: "🎬", mkv: "🎬", avi: "🎬", mov: "🎬",
    mp3: "🎵", flac: "🎵", wav: "🎵",
    png: "🖼", jpg: "🖼", jpeg: "🖼", gif: "🖼", webp: "🖼",
    pdf: "📕", doc: "📄", docx: "📄", txt: "📄",
    exe: "⚙", dll: "🔧", so: "🔧",
  };
  return map[ext] ?? "📄";
}
</script>

<style scoped>
.file-list {
  flex: 1;
  overflow-y: auto;
  position: relative;
}

/* ── Header ─────────────────────────────────────────────────────────────── */
.fl-header {
  display: grid;
  grid-template-columns: var(--col-widths);
  padding: 6px 12px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-1);
  position: sticky;
  top: 0;
  z-index: 2;
  font-size: 11px;
  color: var(--text-3);
  text-transform: uppercase;
  letter-spacing: .05em;
  user-select: none;
}


/* ── Row ────────────────────────────────────────────────────────────────── */
.virtual-container {
  height: calc(100vh - 120px);
  overflow-y: auto;
  contain: strict;
}

.fl-row {
  display: grid;
  grid-template-columns: var(--col-widths);
  padding: 0 12px;
  border-bottom: 1px solid var(--border);
  align-items: center;
  cursor: default;
  transition: background 0.1s;
  user-select: none;
}
.fl-row:hover { background: var(--row-hover); }
.fl-row.selected { background: var(--row-selected); }

/* grid column sizing
   check | icon | name | size | files | modified | bar | actions */
.fl-header, .fl-row {
  --col-widths: 36px 28px minmax(120px, 0.6fr) 96px 80px 168px 100px 68px;
}

/* 欄位預設就是靠左，要靠右的另指定 */
.col-check    { display: flex; align-items: center; justify-content: center; overflow: hidden; }
.col-icon     { font-size: var(--name-font-size, 14px); overflow: hidden; }
.col-name     { overflow: hidden; padding-right: 6px; min-width: 0; }
.col-size     { text-align: center; font-family: var(--mono-font-family); font-size: var(--mono-font-size); overflow: hidden; min-width: 0; }
.col-files    { text-align: center; font-family: var(--mono-font-family); font-size: calc(var(--mono-font-size) - 1px); overflow: hidden; min-width: 0; white-space: nowrap; }
.col-modified { font-family: var(--mono-font-family); font-size: calc(var(--mono-font-size) - 1px); overflow: hidden; min-width: 0; white-space: nowrap; padding-left: 6px; }
.col-bar      { padding: 0 6px; overflow: hidden; min-width: 0; }
.col-actions  {
  display: flex;
  gap: 4px;
  justify-content: flex-end;
  opacity: 0;
  transition: opacity 0.1s;
  overflow: hidden;
}
.fl-row:hover .col-actions { opacity: 1; }

/* Checkbox 樣式 */
.row-checkbox {
  width: 15px;
  height: 15px;
  cursor: pointer;
  accent-color: var(--amber);
  flex-shrink: 0;
  border-radius: 3px;
}

/* 欄位標頭排列方式
   sort-col-left  : 靠左（名稱、修改時間）
   sort-col-right : 靠右（大小、檔案數）——和資料欄相同 */
.sort-col {
  cursor: pointer;
  display: flex;
  align-items: center;
  transition: color 0.1s;
  user-select: none;
  gap: 2px;
}
.sort-col-left   { justify-content: flex-start; }
.sort-col-center { justify-content: center; }      /* 置中（大小、檔案數） */
.sort-col-right  { justify-content: flex-end; }
.sort-col:hover { color: var(--text-1); }
.sort-arrow { font-size: 10px; opacity: 0.6; flex-shrink: 0; }

/* ── Entry name ────────────────────────────────────────────────────────── */
.entry-name {
  font-family: var(--name-font-family);
  font-size: var(--name-font-size);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: block;
}
.dir-name {
  cursor: pointer;
  color: var(--text-1);
}
.dir-name:hover { color: var(--amber); text-decoration: underline; }

/* ── Size bar ───────────────────────────────────────────────────────────── */
.size-bar-track {
  height: 4px;
  background: var(--bg-3);
  border-radius: 2px;
  overflow: hidden;
}
.size-bar-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.3s ease;
}

/* ── Computing skeleton ─────────────────────────────────────────────────── */
.computing-skel {
  display: inline-block;
  width: 52px;
  height: 11px;
  border-radius: 3px;
  background: linear-gradient(90deg, var(--bg-3) 25%, var(--border-hi) 50%, var(--bg-3) 75%);
  background-size: 200% 100%;
  animation: shimmer 1.2s infinite;
}
@keyframes shimmer {
  0%   { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

/* ── Action buttons ─────────────────────────────────────────────────────── */
.btn-icon {
  background: transparent;
  border: 1px solid var(--border);
  border-radius: 3px;
  padding: 2px 6px;
  font-size: 12px;
  color: var(--text-2);
  cursor: pointer;
  transition: all 0.1s;
}
.btn-icon:hover { background: var(--bg-3); border-color: var(--border-hi); }
.btn-icon-danger:hover { background: rgba(255,77,109,.15); border-color: var(--red); color: var(--red); }

/* ── Empty / skeleton ───────────────────────────────────────────────────── */
.fl-empty {
  display: flex;
  justify-content: center;
  padding: 60px;
  font-size: 12px;
}
.skeleton-row { pointer-events: none; }
.icon-skel { display: inline-block; width: 16px; height: 16px; border-radius: 2px; }

/* ── Batch bar ──────────────────────────────────────────────────────────── */
.batch-bar {
  position: sticky;
  bottom: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background: var(--bg-2);
  border-top: 1px solid var(--amber-dim);
  box-shadow: 0 -4px 20px rgba(0,0,0,.5);
  z-index: 10;
  font-size: 12px;
}
.batch-actions { display: flex; gap: 8px; }

/* transition */
.slide-up-enter-active, .slide-up-leave-active { transition: transform 0.2s, opacity 0.2s; }
.slide-up-enter-from, .slide-up-leave-to { transform: translateY(100%); opacity: 0; }

/* ── Modal ──────────────────────────────────────────────────────────────── */
.modal-overlay {
  position: fixed; inset: 0;
  background: rgba(0,0,0,.7);
  display: flex; align-items: center; justify-content: center;
  z-index: 100;
}
.modal {
  background: var(--bg-2);
  border: 1px solid var(--border-hi);
  border-radius: 6px;
  padding: 24px;
  max-width: 480px; width: 90%;
  box-shadow: 0 20px 60px rgba(0,0,0,.8);
}
.modal-title { color: var(--red); font-size: 15px; font-weight: 600; }
.modal-actions {
  display: flex; gap: 10px;
  justify-content: flex-end; margin-top: 20px;
}
</style>
