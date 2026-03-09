<template>
  <aside class="settings-panel">
    <div class="sp-header">
      <span class="sp-title">{{ i18n.settingsTitle }}</span>
      <button class="btn-icon-sm" @click="$emit('close')">✕</button>
    </div>

    <!-- 1. 介面與控制項 -->
    <div class="sp-section">
      <label class="sp-label">介面字型與大小</label>
      <select class="sp-select" v-model="settings.uiFontFamily">
        <option v-for="opt in FONT_OPTIONS" :value="opt.value" :key="opt.value">{{ opt.label }}</option>
      </select>
      <input
        type="range" min="10" max="20" step="1"
        :value="settings.uiFontSize"
        @input="settings.uiFontSize = Number(($event.target as HTMLInputElement).value)"
        class="sp-slider"
      />
      <div class="sp-slider-marks"><span>10</span><span class="amber">{{settings.uiFontSize}}px</span><span>20</span></div>
    </div>

    <!-- 2. 檔案名稱 -->
    <div class="sp-section">
      <label class="sp-label">檔案名稱字型與大小</label>
      <select class="sp-select" v-model="settings.nameFontFamily">
        <option v-for="opt in FONT_OPTIONS" :value="opt.value" :key="opt.value">{{ opt.label }}</option>
      </select>
      <input
        type="range" min="10" max="24" step="1"
        :value="settings.nameFontSize"
        @input="settings.nameFontSize = Number(($event.target as HTMLInputElement).value)"
        class="sp-slider"
      />
      <div class="sp-slider-marks"><span>10</span><span class="amber">{{settings.nameFontSize}}px</span><span>24</span></div>
    </div>

    <!-- 3. 副資訊(大小/時間) -->
    <div class="sp-section">
      <label class="sp-label">數據資訊字型與大小</label>
      <select class="sp-select" v-model="settings.monoFontFamily">
        <option v-for="opt in FONT_OPTIONS" :value="opt.value" :key="opt.value">{{ opt.label }}</option>
      </select>
      <input
        type="range" min="9" max="20" step="1"
        :value="settings.monoFontSize"
        @input="settings.monoFontSize = Number(($event.target as HTMLInputElement).value)"
        class="sp-slider"
      />
      <div class="sp-slider-marks"><span>9</span><span class="amber">{{settings.monoFontSize}}px</span><span>20</span></div>
    </div>

    <!-- 版本號 -->
    <div class="sp-version">
      <span class="dim">{{ i18n.version }}</span>
      <span class="mono amber">v{{ appVersion }}</span>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { useSettings } from "../composables/useSettings";

defineProps<{
  i18n: Record<string, string>;
  appVersion: string;
}>();

defineEmits<{
  close: [];
}>();

const { settings, FONT_OPTIONS } = useSettings();
</script>

<style scoped>
.settings-panel {
  width: 280px;
  flex-shrink: 0;
  background: var(--bg-1);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}
.sp-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
}
.sp-title { font-size: 13px; font-weight: 600; color: var(--text-1); }

.sp-section { padding: 14px 16px; border-bottom: 1px solid var(--border); }
.sp-label { display: block; font-size: 11px; color: var(--text-2); margin-bottom: 8px; text-transform: uppercase; letter-spacing: .05em; }

.sp-select {
  width: 100%;
  padding: 6px 8px;
  background: var(--bg-2);
  color: var(--text-1);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-family: inherit;
  font-size: 12px;
  outline: none;
  cursor: pointer;
}
.sp-select:focus { border-color: var(--amber); }

.sp-slider {
  width: 100%;
  accent-color: var(--amber);
  height: 4px;
  cursor: pointer;
}
.sp-slider-marks {
  display: flex;
  justify-content: space-between;
  font-size: 10px;
  color: var(--text-3);
  margin-top: 4px;
  font-family: var(--font-mono);
}

.sp-font-list { display: flex; flex-direction: column; gap: 4px; }
.sp-font-btn {
  background: var(--bg-2);
  border: 1px solid var(--border);
  border-radius: 4px;
  color: var(--text-2);
  text-align: left;
  padding: 7px 12px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.12s;
}
.sp-font-btn:hover { border-color: var(--border-hi); color: var(--text-1); }
.sp-font-btn.active { background: var(--amber-glow); border-color: var(--amber); color: var(--amber); }

.sp-version {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  font-size: 11px;
  margin-top: auto;
}
</style>
