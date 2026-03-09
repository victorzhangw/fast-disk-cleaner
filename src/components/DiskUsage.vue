<template>
  <div class="disk-usage" v-if="info">
    <!-- 路徑麵包屑（可點擊導覽）-->
    <div class="du-label">
      <div class="du-breadcrumbs">
        <!-- 家/磁碟機圖示按鈕 -->
        <button class="du-home" :title="i18n?.selectDrive ?? 'Drives'" @click="$emit('go-home')">🏠</button>
        <span class="du-sep">›</span>
        <!-- 路徑各段可點擊 -->
        <template v-for="(seg, idx) in pathSegments" :key="seg.path">
          <button
            class="du-crumb"
            :class="{ last: idx === pathSegments.length - 1 }"
            @click="$emit('navigate', seg.path)"
          >{{ seg.label }}</button>
          <span class="du-sep" v-if="idx < pathSegments.length - 1">›</span>
        </template>
      </div>
      <span class="du-usage mono">
        <span class="amber">{{ formatBytes(info.used) }}</span>
        <span class="dim"> / {{ formatBytes(info.total) }}</span>
      </span>
    </div>
    <div class="du-bar-track">
      <div
        class="du-bar-fill"
        :style="{ width: pct + '%', background: barColor }"
      />
    </div>
    <div class="du-sub">
      <span class="dim">{{ formatBytes(info.available) }} {{ i18n?.free ?? 'free' }}</span>
      <span class="dim">{{ pct.toFixed(1) }}% {{ i18n?.used ?? 'used' }}</span>
    </div>
  </div>

  <!-- Skeleton while loading -->
  <div class="disk-usage" v-else>
    <div class="du-label">
      <span class="skeleton" style="width:120px;height:12px" />
      <span class="skeleton" style="width:80px;height:12px" />
    </div>
    <div class="du-bar-track">
      <div class="du-bar-fill skeleton" style="width:60%;background:none" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { DiskInfo } from "../composables/useScanner";
import { formatBytes } from "../composables/useScanner";

const props = defineProps<{
  info: DiskInfo | null;
  i18n?: Record<string, string>;
  currentPath?: string;
}>();

defineEmits<{
  navigate: [path: string];
  "go-home": [];
}>();

const pct = computed(() => {
  if (!props.info || props.info.total === 0) return 0;
  return (props.info.used / props.info.total) * 100;
});

const barColor = computed(() => {
  const p = pct.value;
  if (p >= 90) return "var(--red)";
  if (p >= 70) return "var(--amber)";
  return "var(--green)";
});

/** 將當前路徑拆成可點擊的麵包屑段落 */
const pathSegments = computed(() => {
  const path = props.currentPath || props.info?.path || "";
  if (!path) return [];

  const isWin = /^[A-Za-z]:\\/.test(path);
  if (isWin) {
    const parts = path.split("\\").filter(Boolean);
    return parts.map((part, i) => ({
      label: i === 0 ? part + ":\\" : part,
      path: parts.slice(0, i + 1).join("\\") + (i === 0 ? "\\" : ""),
    }));
  } else {
    const parts = path.split("/").filter(Boolean);
    return [
      { label: "/", path: "/" },
      ...parts.map((part, i) => ({
        label: part,
        path: "/" + parts.slice(0, i + 1).join("/"),
      })),
    ];
  }
});
</script>

<style scoped>
.disk-usage {
  padding: 8px 16px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-1);
}
.du-label {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11px;
  margin-bottom: 5px;
}

/* 路徑麵包屑 */
.du-breadcrumbs {
  display: flex;
  align-items: center;
  gap: 2px;
  overflow: hidden;
  flex: 1;
  min-width: 0;
}

.du-home {
  background: none;
  border: none;
  font-size: 13px;
  cursor: pointer;
  padding: 1px 4px;
  border-radius: 3px;
  line-height: 1;
  transition: background 0.1s;
  flex-shrink: 0;
}
.du-home:hover { background: var(--bg-3); }

.du-crumb {
  background: none;
  border: none;
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-3);
  cursor: pointer;
  padding: 1px 3px;
  border-radius: 2px;
  white-space: nowrap;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 120px;
  transition: color 0.1s, background 0.1s;
}
.du-crumb:hover { color: var(--amber); background: var(--bg-3); }
.du-crumb.last { color: var(--text-2); }

.du-sep {
  color: var(--text-3);
  font-size: 10px;
  flex-shrink: 0;
  user-select: none;
}

.du-usage { font-size: 11px; flex-shrink: 0; margin-left: 8px; }

.du-bar-track {
  height: 4px;
  background: var(--bg-3);
  border-radius: 2px;
  overflow: hidden;
}
.du-bar-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.4s ease, background 0.3s;
}
.du-sub {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  margin-top: 4px;
}
</style>
