<script setup lang="ts">
import { computed, ref } from "vue";
import { Download, RotateCcw, Upload, X } from "lucide-vue-next";
import { useAppStore } from "../stores/appStore";
import { useSettingsStore } from "../stores/settingsStore";
import { noteColorList, noteColors } from "../utils/colors";
import type { NoteColor } from "../types";

const appStore = useAppStore();
const settingsStore = useSettingsStore();
const importText = ref("");
const exported = ref("");

const settings = computed(() => settingsStore.settings);

function setColor(color: NoteColor) {
  settingsStore.updateSettings({ defaultNoteColor: color });
}

function exportData() {
  exported.value = appStore.exportData();
}

function importData() {
  if (!importText.value.trim()) return;
  appStore.importData(importText.value);
  importText.value = "";
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
        <label>
          <span>主题</span>
          <select :value="settings.theme" @change="settingsStore.updateSettings({ theme: ($event.target as HTMLSelectElement).value as any })">
            <option value="light">浅色</option>
            <option value="dark">深色</option>
            <option value="system">跟随系统</option>
          </select>
        </label>
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
        <div class="color-grid">
          <button
            v-for="color in noteColorList"
            :key="color"
            :class="{ active: color === settings.defaultNoteColor }"
            :style="{ backgroundColor: noteColors[color] }"
            @click="setColor(color)"
          ></button>
        </div>
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
        <h3>数据</h3>
        <div class="actions">
          <button @click="exportData"><Download :size="16" />导出</button>
          <button @click="importData"><Upload :size="16" />导入</button>
          <button @click="appStore.resetSampleData()"><RotateCcw :size="16" />重置示例</button>
        </div>
        <textarea v-model="importText" placeholder="粘贴 JSON 数据后点击导入"></textarea>
        <textarea v-if="exported" v-model="exported" readonly></textarea>
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

.color-grid {
  display: grid;
  grid-template-columns: repeat(8, 1fr);
  gap: 8px;
  margin-bottom: 10px;
}

.color-grid button {
  aspect-ratio: 1;
  border: 1px solid rgba(0, 0, 0, 0.12);
  border-radius: 50%;
}

.color-grid button.active {
  outline: 2px solid #3b82f6;
  outline-offset: 2px;
}

.actions {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
  margin-bottom: 10px;
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
</style>
