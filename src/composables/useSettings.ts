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
    uiFontSize: number;
    uiFontFamily: FontFamily;
    nameFontSize: number;
    nameFontFamily: FontFamily;
    monoFontSize: number;
    monoFontFamily: FontFamily;
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
    return {
        uiFontSize: 13, uiFontFamily: "IBM Plex Sans",
        nameFontSize: 13, nameFontFamily: "IBM Plex Sans",
        monoFontSize: 12, monoFontFamily: "IBM Plex Mono",
    };
}

function load(): AppSettings {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (raw) {
            const parsed = JSON.parse(raw);
            // 遷移舊的單一設定
            if ('fontSize' in parsed && !('uiFontSize' in parsed)) {
                return {
                    uiFontSize: parsed.fontSize, uiFontFamily: parsed.fontFamily || "IBM Plex Sans",
                    nameFontSize: parsed.fontSize, nameFontFamily: parsed.fontFamily || "IBM Plex Sans",
                    monoFontSize: Math.max(10, parsed.fontSize - 1), monoFontFamily: "IBM Plex Mono",
                };
            }
            return { ...defaults(), ...parsed };
        }
    } catch { }
    return defaults();
}

// ── 套用到 DOM ────────────────────────────────────────────────────────────────

function applySettings(s: AppSettings) {
    const html = document.documentElement;
    // UI (全域預設)
    html.style.setProperty("--app-font-size", `${s.uiFontSize}px`);
    html.style.setProperty("--app-font-family", `"${s.uiFontFamily}", system-ui, sans-serif`);
    html.style.fontSize = `${s.uiFontSize}px`;
    html.style.fontFamily = `"${s.uiFontFamily}", system-ui, sans-serif`;

    // 檔案名稱
    html.style.setProperty("--name-font-size", `${s.nameFontSize}px`);
    html.style.setProperty("--name-font-family", `"${s.nameFontFamily}", system-ui, sans-serif`);

    // 數據 (檔案大小 / 數量 / 日期)
    html.style.setProperty("--mono-font-size", `${s.monoFontSize}px`);
    html.style.setProperty("--mono-font-family", `"${s.monoFontFamily}", monospace`);
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
