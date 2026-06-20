<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { Download, Upload, X } from "lucide-vue-next";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useAppStore } from "../stores/appStore";
import { useFeedbackStore } from "../stores/feedbackStore";
import { useSettingsStore } from "../stores/settingsStore";
import { backupDatabase, defaultImageLibraryPath, exportJsonFile, importJsonFile } from "../utils/storage";

const appStore = useAppStore();
const feedback = useFeedbackStore();
const settingsStore = useSettingsStore();
const defaultImagePath = ref("");

const settings = computed(() => settingsStore.settings);
const imageLibraryDisplayPath = computed(() => settings.value.imageLibraryPath || defaultImagePath.value || "默认图片目录");

onMounted(async () => {
  defaultImagePath.value = await defaultImageLibraryPath();
});

async function exportData() {
  if (!appStore.databaseReady) {
    const message = appStore.loadError || "数据库未加载";
    feedback.notify(`导出失败：${message}`, "error");
    return;
  }
  const selected = await save({
    defaultPath: "notescape-data.json",
    filters: [{ name: "JSON 数据", extensions: ["json"] }],
  });
  if (!selected) return;
  try {
    await exportJsonFile(selected, appStore.exportData());
    feedback.notify("数据已导出", "success");
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    feedback.notify(`导出失败：${message}`, "error");
  }
}

async function importFromFile() {
  if (!appStore.databaseReady) {
    const message = appStore.loadError || "数据库未加载";
    feedback.notify(`导入失败：${message}`, "error");
    return;
  }
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "JSON 数据", extensions: ["json"] }],
  });
  if (typeof selected !== "string") return;
  try {
    await appStore.importData(await importJsonFile(selected));
    feedback.notify("数据已导入", "success");
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    feedback.notify(`导入失败：${message}`, "error");
  }
}

async function chooseDatabase() {
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "SQLite 数据库", extensions: ["sqlite", "sqlite3", "db"] }],
  });
  if (typeof selected !== "string") return;
  try {
    await appStore.changeDatabase(selected);
    feedback.notify("已切换数据库文件", "success");
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    feedback.notify(`切换数据库失败：${message}`, "error");
  }
}

async function createDatabase() {
  const selected = await save({
    defaultPath: "notescape.sqlite3",
    filters: [{ name: "SQLite 数据库", extensions: ["sqlite", "sqlite3", "db"] }],
  });
  if (!selected) return;
  try {
    await appStore.createDatabase(selected);
    feedback.notify("已创建并切换数据库文件", "success");
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    feedback.notify(`创建数据库失败：${message}`, "error");
  }
}

async function useDefaultDatabase() {
  try {
    await appStore.useDefaultDatabase();
    feedback.notify("已回退到默认数据库", "success");
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    feedback.notify(`回退默认数据库失败：${message}`, "error");
  }
}

async function backupCurrentDatabase() {
  const selected = await save({
    defaultPath: "notescape-backup.sqlite3",
    filters: [{ name: "SQLite 数据库", extensions: ["sqlite", "sqlite3", "db"] }],
  });
  if (!selected) return;
  try {
    await backupDatabase(selected);
    feedback.notify("数据库已备份", "success");
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    feedback.notify(`备份数据库失败：${message}`, "error");
  }
}

async function chooseImageLibrary() {
  const selected = await open({
    multiple: false,
    directory: true,
  });
  if (typeof selected !== "string") return;
  settingsStore.updateSettings({ imageLibraryPath: selected });
}

function useDefaultImageLibrary() {
  settingsStore.updateSettings({ imageLibraryPath: "" });
}

</script>

<template>
  <div class="overlay" @mousedown.self="settingsStore.togglePanel()">
    <aside class="panel">
      <header>
        <h2>设置</h2>
        <button class="icon-button" @click="settingsStore.togglePanel()"><X :size="18" /></button>
      </header>

      <section>
        <h3>外观</h3>
        <div class="setting-row">
          <span>主题</span>
          <div class="theme-segments" role="group" aria-label="主题">
            <button :class="{ active: settings.theme === 'light' }" @click="settingsStore.updateSettings({ theme: 'light' })">浅色</button>
            <button :class="{ active: settings.theme === 'dark' }" @click="settingsStore.updateSettings({ theme: 'dark' })">深色</button>
            <button :class="{ active: settings.theme === 'system' }" @click="settingsStore.updateSettings({ theme: 'system' })">跟随系统</button>
          </div>
        </div>
        <label class="switch">
          <span>显示网格点</span>
          <input type="checkbox" :checked="settings.showGrid" @change="settingsStore.updateSettings({ showGrid: ($event.target as HTMLInputElement).checked })" />
        </label>
        <label class="switch">
          <span>便签阴影</span>
          <input type="checkbox" :checked="settings.noteShadow" @change="settingsStore.updateSettings({ noteShadow: ($event.target as HTMLInputElement).checked })" />
        </label>
      </section>

      <section>
        <h3>便签</h3>
        <label>
          <span>默认字号</span>
          <input type="number" min="12" max="32" :value="settings.defaultFontSize" @change="settingsStore.updateSettings({ defaultFontSize: Number(($event.target as HTMLInputElement).value) })" />
        </label>
        <label class="font-field">
          <span>中文字体</span>
          <input type="text" :value="settings.chineseFontFamily" @change="settingsStore.updateSettings({ chineseFontFamily: ($event.target as HTMLInputElement).value })" />
        </label>
        <label class="font-field">
          <span>英文字体</span>
          <input type="text" :value="settings.englishFontFamily" @change="settingsStore.updateSettings({ englishFontFamily: ($event.target as HTMLInputElement).value })" />
        </label>
        <label class="font-field">
          <span>等宽字体</span>
          <input type="text" :value="settings.monospaceFontFamily" @change="settingsStore.updateSettings({ monospaceFontFamily: ($event.target as HTMLInputElement).value })" />
        </label>
        <label class="switch">
          <span>随机旋转</span>
          <input type="checkbox" :checked="settings.randomRotation" @change="settingsStore.updateSettings({ randomRotation: ($event.target as HTMLInputElement).checked })" />
        </label>
        <label class="switch">
          <span>自动保存</span>
          <input type="checkbox" :checked="settings.autoSave" @change="settingsStore.updateSettings({ autoSave: ($event.target as HTMLInputElement).checked })" />
        </label>
      </section>

      <section>
        <h3>数据库</h3>
        <p v-if="appStore.loadError" class="load-error">{{ appStore.loadError }}</p>
        <div class="db-path">{{ appStore.databasePath || "未加载" }}</div>
        <div class="actions database-actions">
          <button @click="chooseDatabase">切换</button>
          <button @click="createDatabase">新建</button>
          <button @click="useDefaultDatabase">默认</button>
          <button :disabled="!appStore.databaseReady" @click="backupCurrentDatabase">备份</button>
        </div>
      </section>

      <section>
        <h3>数据</h3>
        <div class="actions">
          <button :disabled="!appStore.databaseReady" @click="exportData"><Download :size="16" />导出数据</button>
          <button :disabled="!appStore.databaseReady" @click="importFromFile"><Upload :size="16" />导入文件</button>
        </div>
      </section>

      <section>
        <h3>图片</h3>
        <div class="db-path">{{ imageLibraryDisplayPath }}</div>
        <div class="actions">
          <button @click="chooseImageLibrary">选择目录</button>
          <button @click="useDefaultImageLibrary">使用默认</button>
        </div>
      </section>
    </aside>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 1500;
  display: flex;
  justify-content: flex-end;
  background: var(--overlay);
}

:global(.settings-drawer-enter-active),
:global(.settings-drawer-leave-active) {
  transition: background-color 0.18s ease;
}

:global(.settings-drawer-enter-active .panel),
:global(.settings-drawer-leave-active .panel) {
  transition: transform 0.18s ease, opacity 0.18s ease;
}

:global(.settings-drawer-enter-from),
:global(.settings-drawer-leave-to) {
  background: rgba(15, 23, 42, 0);
}

:global(.settings-drawer-enter-from .panel),
:global(.settings-drawer-leave-to .panel) {
  opacity: 0;
  transform: translateX(100%);
}

.panel {
  width: 360px;
  height: 100%;
  padding: 20px;
  overflow: auto;
  color: var(--text-primary);
  background: var(--surface);
  border-left: 1px solid var(--border-color);
  box-shadow: var(--shadow-lg);
}

header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 18px;
}

h2,
h3 {
  margin: 0;
}

h2 {
  font-size: 20px;
}

h3 {
  margin-bottom: 12px;
  font-size: 15px;
}

section {
  padding: 16px 0;
  border-top: 1px solid var(--border-soft);
}

.setting-row,
label {
  display: grid;
  grid-template-columns: 1fr 140px;
  align-items: center;
  gap: 12px;
  min-height: 42px;
  color: var(--text-primary);
}

.setting-row {
  grid-template-columns: 1fr 184px;
}

select,
input[type="number"],
input[type="text"] {
  width: 100%;
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  border-radius: 7px;
  outline: 0;
  background: var(--surface-muted);
}

select,
input[type="number"],
input[type="text"] {
  height: 34px;
  padding: 0 9px;
}

.font-field {
  grid-template-columns: 78px 1fr;
}

.switch input {
  justify-self: end;
  width: 18px;
  height: 18px;
}

.theme-segments {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  padding: 3px;
  background: var(--surface-muted);
  border: 1px solid var(--border-color);
  border-radius: 8px;
}

.theme-segments button {
  min-width: 0;
  height: 28px;
  padding: 0 6px;
  color: var(--text-secondary);
  background: transparent;
  border-radius: 6px;
  font-size: 12px;
  white-space: nowrap;
}

.theme-segments button.active {
  color: var(--primary-text);
  background: var(--primary-soft);
  box-shadow: var(--shadow-sm);
}

.theme-segments button:focus-visible {
  outline: 2px solid var(--primary);
  outline-offset: 1px;
}

.actions {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
  margin-bottom: 10px;
}

.actions.two {
  grid-template-columns: repeat(2, 1fr);
}

.actions.three {
  grid-template-columns: repeat(3, 1fr);
}

.actions.database-actions {
  grid-template-columns: repeat(4, 1fr);
}

.actions button {
  height: 34px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  color: var(--text-primary);
  background: var(--surface-muted);
  border: 1px solid var(--border-color);
  border-radius: 7px;
}

.actions button:disabled {
  cursor: var(--cursor-not-allowed);
  color: var(--text-muted);
  background: var(--surface-muted);
}

.db-path {
  min-height: 38px;
  padding: 9px 10px;
  overflow-wrap: anywhere;
  color: var(--text-secondary);
  background: var(--surface-muted);
  border: 1px solid var(--border-color);
  border-radius: 7px;
  font-size: 12px;
  line-height: 1.45;
  margin-bottom: 10px;
}

.load-error {
  margin: 0 0 10px;
  padding: 9px 10px;
  color: var(--danger-text);
  background: var(--danger-bg);
  border: 1px solid var(--danger-border);
  border-radius: 7px;
  font-size: 13px;
  line-height: 1.5;
}

</style>
