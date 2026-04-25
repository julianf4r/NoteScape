<script setup lang="ts">
import { computed, ref } from "vue";
import { Download, Upload, X } from "lucide-vue-next";
import { open, save } from "@tauri-apps/plugin-dialog";
import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
import { useAppStore } from "../stores/appStore";
import { useFeedbackStore } from "../stores/feedbackStore";
import { useSettingsStore } from "../stores/settingsStore";
import { backupDatabase } from "../utils/storage";

const appStore = useAppStore();
const feedback = useFeedbackStore();
const settingsStore = useSettingsStore();
const importText = ref("");
const exported = ref("");
const dbMessage = ref("");
const dataMessage = ref("");

const settings = computed(() => settingsStore.settings);

async function exportData() {
  const selected = await save({
    defaultPath: "notescape-data.json",
    filters: [{ name: "JSON 数据", extensions: ["json"] }],
  });
  if (!selected) return;
  try {
    await writeTextFile(selected, appStore.exportData());
    dataMessage.value = "数据已导出";
    feedback.notify(dataMessage.value, "success");
  } catch (error) {
    dataMessage.value = error instanceof Error ? error.message : String(error);
    feedback.notify(`导出失败：${dataMessage.value}`, "error");
  }
}

function importData() {
  if (!importText.value.trim()) return;
  try {
    appStore.importData(importText.value);
    importText.value = "";
    dataMessage.value = "数据已导入";
    feedback.notify(dataMessage.value, "success");
  } catch (error) {
    dataMessage.value = error instanceof Error ? error.message : String(error);
    feedback.notify(`导入失败：${dataMessage.value}`, "error");
  }
}

async function importFromFile() {
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "JSON 数据", extensions: ["json"] }],
  });
  if (typeof selected !== "string") return;
  try {
    appStore.importData(await readTextFile(selected));
    dataMessage.value = "数据已导入";
    feedback.notify(dataMessage.value, "success");
  } catch (error) {
    dataMessage.value = error instanceof Error ? error.message : String(error);
    feedback.notify(`导入失败：${dataMessage.value}`, "error");
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
    dbMessage.value = "已切换数据库文件";
    feedback.notify(dbMessage.value, "success");
  } catch (error) {
    dbMessage.value = error instanceof Error ? error.message : String(error);
    feedback.notify(`切换数据库失败：${dbMessage.value}`, "error");
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
    dbMessage.value = "已创建并切换数据库文件";
    feedback.notify(dbMessage.value, "success");
  } catch (error) {
    dbMessage.value = error instanceof Error ? error.message : String(error);
    feedback.notify(`创建数据库失败：${dbMessage.value}`, "error");
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
    dbMessage.value = "数据库已备份";
    feedback.notify(dbMessage.value, "success");
  } catch (error) {
    dbMessage.value = error instanceof Error ? error.message : String(error);
    feedback.notify(`备份数据库失败：${dbMessage.value}`, "error");
  }
}

</script>

<template>
  <div class="overlay" @mousedown.self="settingsStore.togglePanel()">
    <aside class="panel">
      <header>
        <h2>设置</h2>
        <button class="icon-button" @click="settingsStore.togglePanel()"><X :size="18" /></button>
      </header>
      <p v-if="appStore.saveStatus !== 'idle'" class="save-status" :class="appStore.saveStatus">{{ appStore.statusMessage || appStore.saveStatus }}</p>

      <section>
        <h3>外观</h3>
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
        <div class="db-path">{{ appStore.databasePath || "未加载" }}</div>
        <div class="actions three">
          <button @click="chooseDatabase">切换</button>
          <button @click="createDatabase">新建</button>
          <button @click="backupCurrentDatabase">备份</button>
        </div>
        <p v-if="dbMessage" class="hint">{{ dbMessage }}</p>
      </section>

      <section>
        <h3>数据</h3>
        <div class="actions">
          <button @click="exportData"><Download :size="16" />导出</button>
          <button @click="importFromFile"><Upload :size="16" />导入文件</button>
        </div>
        <textarea v-model="importText" placeholder="粘贴 JSON 数据后点击下方按钮导入"></textarea>
        <button class="wide-action" @click="importData">导入粘贴的数据</button>
        <textarea v-if="exported" v-model="exported" readonly></textarea>
        <p v-if="dataMessage" class="hint">{{ dataMessage }}</p>
      </section>
    </aside>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  justify-content: flex-end;
  background: rgba(15, 23, 42, 0.14);
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
  background: #fff;
  border-left: 1px solid #e5e7eb;
  box-shadow: -12px 0 32px rgba(15, 23, 42, 0.12);
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
  border-top: 1px solid #eef1f4;
}

label {
  display: grid;
  grid-template-columns: 1fr 140px;
  align-items: center;
  gap: 12px;
  min-height: 42px;
  color: #374151;
}

select,
input[type="number"],
textarea {
  width: 100%;
  border: 1px solid #dfe3ea;
  border-radius: 7px;
  outline: 0;
  background: #fff;
}

select,
input[type="number"] {
  height: 34px;
  padding: 0 9px;
}

.switch input {
  justify-self: end;
  width: 18px;
  height: 18px;
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

.actions button {
  height: 34px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  color: #374151;
  background: #f8fafc;
  border: 1px solid #e5e7eb;
  border-radius: 7px;
}

textarea {
  min-height: 92px;
  margin-top: 8px;
  padding: 9px;
  resize: vertical;
  font-size: 12px;
}

.db-path {
  min-height: 38px;
  padding: 9px 10px;
  overflow-wrap: anywhere;
  color: #4b5563;
  background: #f8fafc;
  border: 1px solid #e5e7eb;
  border-radius: 7px;
  font-size: 12px;
  line-height: 1.45;
  margin-bottom: 10px;
}

.hint {
  margin: 0;
  color: #2563eb;
  font-size: 13px;
}

.save-status {
  margin: -6px 0 10px;
  padding: 8px 10px;
  border-radius: 7px;
  font-size: 13px;
}

.save-status.saved {
  color: #166534;
  background: #dcfce7;
}

.save-status.saving {
  color: #1d4ed8;
  background: #dbeafe;
}

.save-status.error {
  color: #991b1b;
  background: #fee2e2;
}

.wide-action {
  width: 100%;
  height: 34px;
  margin-top: 8px;
  color: #374151;
  background: #f8fafc;
  border: 1px solid #e5e7eb;
  border-radius: 7px;
}
</style>
