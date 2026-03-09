// composables/useSettings.ts
// 字型大小 / 字型系列設定，持久化到 localStorage

import { ref, watch } from "vue";

export type FontFamily =
    | "IBM Plex Sans"
    | "IBM Plex Mono"
    | "Inter"
    | "Noto Sans TC"
    | "system-ui";

export interface AppSettings {
    fontSize: number;      // 10–20 px
    fontFamily: FontFamily;
}

const STORAGE_KEY = "fdc-settings";

export const FONT_OPTIONS: { label: string; value: FontFamily }[] = [
    { label: "IBM Plex Sans（預設）", value: "IBM Plex Sans" },
    { label: "IBM Plex Mono", value: "IBM Plex Mono" },
    { label: "Inter", value: "Inter" },
    { label: "Noto Sans TC（中文）", value: "Noto Sans TC" },
    { label: "系統預設", value: "system-ui" },
];

// ── Google Fonts 預載 ─────────────────────────────────────────────────────────

const GFONTS_FAMILIES = [
    "IBM+Plex+Sans:wght@400;500;600",
    "IBM+Plex+Mono:wght@400;500",
    "Inter:wght@400;500;600",
    "Noto+Sans+TC:wght@400;500;700",
].join("&family=");

function ensureGFonts() {
    if (document.getElementById("fdc-gfonts")) return;
    const link = document.createElement("link");
    link.id = "fdc-gfonts";
    link.rel = "stylesheet";
    link.href = `https://fonts.googleapis.com/css2?family=${GFONTS_FAMILIES}&display=swap`;
    document.head.appendChild(link);
}

// ── 讀取 / 預設 ───────────────────────────────────────────────────────────────

function defaults(): AppSettings {
    return { fontSize: 13, fontFamily: "IBM Plex Sans" };
}

function load(): AppSettings {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (raw) {
            const parsed = JSON.parse(raw);
            return { ...defaults(), ...parsed };
        }
    } catch { }
    return defaults();
}

// ── 套用到 DOM ────────────────────────────────────────────────────────────────

function applySettings(s: AppSettings) {
    const html = document.documentElement;
    // 直接設在 html element 的 style，優先級最高
    html.style.setProperty("--app-font-size", `${s.fontSize}px`);
    html.style.setProperty("--app-font-family", `"${s.fontFamily}", system-ui, sans-serif`);
    // 同時直接設 fontSize / fontFamily（確保覆蓋）
    html.style.fontSize = `${s.fontSize}px`;
    html.style.fontFamily = `"${s.fontFamily}", system-ui, sans-serif`;
}

// ── 全域單例 ──────────────────────────────────────────────────────────────────

const settings = ref<AppSettings>(load());

// 立即套用（模組載入時執行）
ensureGFonts();
applySettings(settings.value);

// ── Composable ────────────────────────────────────────────────────────────────

export function useSettings() {
    // 監聽設定變化，即時套用並儲存
    watch(
        settings,
        (s) => {
            applySettings(s);
            localStorage.setItem(STORAGE_KEY, JSON.stringify(s));
        },
        { deep: true, immediate: true }
    );

    return { settings, FONT_OPTIONS };
}
