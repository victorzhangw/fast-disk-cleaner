<template>
  <div class="trash-manager">
    <div class="tm-toolbar">
      <h3 class="tm-title">
        <span class="amber">🗑</span> {{ i18n?.trashTitle ?? 'Trash' }}
        <span class="dim" style="font-size:11px;font-weight:400;margin-left:8px">
          ({{ items.length }} · {{ formatBytes(totalSize) }})
        </span>
      </h3>
      <div class="tm-actions">
        <button class="btn-ghost" @click="load">↻ {{ i18n?.refresh ?? 'Refresh' }}</button>
        <button
          class="btn-danger"
          :disabled="items.length === 0"
          @click="confirmEmpty = true"
        >
          {{ i18n?.emptyTrash ?? 'Empty Trash' }}
        </button>
      </div>
    </div>

    <!-- 載入中 -->
    <div class="tm-loading" v-if="isLoading">
      <span class="scanning-dot" />
      <span class="dim" style="margin-left:8px">{{ i18n?.loading ?? 'Loading…' }}</span>
    </div>

    <!-- 空狀態 -->
    <div class="tm-empty" v-else-if="items.length === 0">
      <div class="empty-icon">🗑</div>
      <p class="dim">{{ i18n?.trashEmpty ?? 'Trash is empty' }}</p>
    </div>

    <!-- Items -->
    <div class="tm-list" v-else>
      <div class="tm-header">
        <div class="tc-icon" />
        <div class="tc-name">{{ i18n?.name ?? 'Name' }}</div>
        <div class="tc-size">{{ i18n?.size ?? 'Size' }}</div>
        <div class="tc-date">{{ i18n?.deletedAt ?? 'Deleted' }}</div>
        <div class="tc-orig">{{ i18n?.originalPath ?? 'Original Path' }}</div>
        <div class="tc-actions" />
      </div>

      <div class="tm-row" v-for="item in items" :key="item.trash_id">
        <div class="tc-icon">📦</div>
        <div class="tc-name truncate">{{ item.name }}</div>
        <div class="tc-size mono amber">{{ formatBytes(item.size) }}</div>
        <div class="tc-date dim mono" style="font-size:11px">{{ formatDate(item.deleted_at) }}</div>
        <div class="tc-orig dim truncate" style="font-size:11px">{{ item.original_path }}</div>
        <div class="tc-actions">
          <button class="btn-ghost" style="font-size:11px" @click="restoreItem(item.trash_id)">
            ↩ {{ i18n?.restore ?? 'Restore' }}
          </button>
          <button class="btn-danger" style="font-size:11px" @click="purgeItem(item.trash_id)">
            ✕ {{ i18n?.delete ?? 'Delete' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Error -->
    <div class="tm-error" v-if="error">
      <span class="red">⚠ {{ error }}</span>
    </div>

    <!-- Empty confirmation modal -->
    <Teleport to="body">
      <div class="modal-overlay" v-if="confirmEmpty" @click="confirmEmpty = false">
        <div class="modal" @click.stop>
          <h3 style="color:var(--red);font-size:15px;font-weight:600">⚠ {{ i18n?.emptyTrash ?? 'Empty Trash' }}?</h3>
          <p class="dim" style="margin:12px 0">
            {{ i18n?.emptyTrashConfirm ?? 'Permanently delete all items? This cannot be undone.' }}
            ({{ items.length }} items · {{ formatBytes(totalSize) }})
          </p>
          <div style="display:flex;gap:10px;justify-content:flex-end;margin-top:16px">
            <button class="btn-ghost" @click="confirmEmpty = false">{{ i18n?.cancelBtn ?? 'Cancel' }}</button>
            <button class="btn-danger" @click="doEmptyTrash">{{ i18n?.emptyTrash ?? 'Empty Trash' }}</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from "vue";
import { useTrash, formatBytes, formatDate } from "../composables/useScanner";

const props = defineProps<{ i18n?: Record<string, string> }>();
const { i18n } = props;

const { items, isLoading, error, load, restore, purge, emptyAll } = useTrash();

const confirmEmpty = ref(false);

const totalSize = computed(() =>
  items.value.reduce((s, i) => s + i.size, 0)
);

async function restoreItem(trash_id: string) {
  try {
    await restore(trash_id);
  } catch (e) {
    alert(String(e));
  }
}

async function purgeItem(trash_id: string) {
  try {
    await purge(trash_id);
  } catch (e) {
    alert(String(e));
  }
}

async function doEmptyTrash() {
  confirmEmpty.value = false;
  try {
    await emptyAll();
  } catch (e) {
    alert(String(e));
  }
}

onMounted(() => load());
</script>

<style scoped>
.trash-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.tm-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-1);
  flex-shrink: 0;
}
.tm-title { font-size: 13px; font-weight: 600; }
.tm-actions { display: flex; gap: 8px; }

.tm-loading, .tm-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 60px;
  color: var(--text-2);
}
.empty-icon { font-size: 48px; opacity: .3; }

.tm-error {
  padding: 10px 16px;
  font-size: 12px;
}

/* ── List ────────────────────────────────────────────────────────────────── */
.tm-list { flex: 1; overflow-y: auto; }

.tm-header, .tm-row {
  display: grid;
  grid-template-columns: 28px 1fr 90px 150px 200px 160px;
  padding: 6px 12px;
  align-items: center;
}

.tm-header {
  font-size: 11px;
  color: var(--text-3);
  text-transform: uppercase;
  letter-spacing: .05em;
  border-bottom: 1px solid var(--border);
  background: var(--bg-1);
  position: sticky;
  top: 0;
}

.tm-row {
  border-bottom: 1px solid var(--border);
  transition: background 0.1s;
}
.tm-row:hover { background: var(--bg-2); }

.tc-actions { display: flex; gap: 6px; justify-content: flex-end; }
.tc-icon { font-size: 14px; }

/* modal (shared styles defined globally) */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}
.modal {
  background: var(--bg-2);
  border: 1px solid var(--border-hi);
  border-radius: 6px;
  padding: 24px;
  max-width: 440px;
  width: 90%;
  box-shadow: 0 20px 60px rgba(0,0,0,.8);
}
</style>
