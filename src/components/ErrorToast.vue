<template>
  <Transition name="fade">
    <div class="error-toast" v-if="error">
      <div class="error-content">
        <span class="red">⚠</span>
        <span class="error-msg" style="user-select:text">{{ error }}</span>
      </div>
      <div class="error-actions">
        <button class="btn-icon-sm" title="複製錯誤訊息" @click="copyError">📋</button>
        <button class="btn-icon-sm" @click="$emit('clear')">✕</button>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
const props = defineProps<{
  error: string | null;
}>();

defineEmits<{
  clear: [];
}>();

function copyError() {
  if (props.error) navigator.clipboard.writeText(props.error).catch(() => {});
}
</script>

<style scoped>
.error-toast {
  position: fixed; bottom: 36px; left: 50%;
  transform: translateX(-50%);
  background: var(--bg-2); border: 1px solid var(--red);
  border-radius: 6px; padding: 12px 16px;
  display: flex; align-items: flex-start; gap: 12px;
  font-size: 12px; box-shadow: var(--shadow-card); z-index: 200;
  max-width: min(600px, 90vw);
}
.error-content {
  display: flex; gap: 8px; align-items: flex-start; flex: 1; min-width: 0;
}
.error-msg {
  word-break: break-all;
  white-space: pre-wrap;
  line-height: 1.5;
  color: var(--text-1);
}
.error-actions { display: flex; gap: 4px; flex-shrink: 0; }

.btn-icon-sm {
  background: none; border: 1px solid var(--border);
  color: var(--text-2); padding: 3px 8px;
  border-radius: 3px; font-size: 12px; cursor: pointer;
}
.btn-icon-sm:hover:not(:disabled) { border-color: var(--border-hi); color: var(--text-1); }
.btn-icon-sm:disabled { opacity: 0.3; cursor: not-allowed; }

.fade-enter-active, .fade-leave-active { transition: opacity 0.2s, transform 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; transform: translateX(-50%) translateY(10px); }
</style>
