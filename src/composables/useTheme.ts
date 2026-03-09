// composables/useTheme.ts
// 深色 / 淺色主題切換，持久化到 localStorage

import { ref, watch } from "vue";

export type Theme = "dark" | "light";

const STORAGE_KEY = "fdc-theme";

function getInitialTheme(): Theme {
    const saved = localStorage.getItem(STORAGE_KEY) as Theme | null;
    if (saved === "dark" || saved === "light") return saved;
    // 依照 OS 偏好設定
    return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
}

// 全域主題狀態（單例）
const theme = ref<Theme>(getInitialTheme());

// 立即套用初始主題
document.documentElement.setAttribute("data-theme", theme.value);

export function useTheme() {
    function toggleTheme() {
        theme.value = theme.value === "dark" ? "light" : "dark";
    }

    // 同步 DOM attribute + localStorage
    watch(theme, (val) => {
        document.documentElement.setAttribute("data-theme", val);
        localStorage.setItem(STORAGE_KEY, val);
    }, { immediate: true });

    return { theme, toggleTheme };
}
