<script setup lang="ts">
import { computed, nextTick, ref } from "vue";
import { Archive, FileText, Menu, Pencil, Plus, RotateCcw, Settings, Trash2, X } from "lucide-vue-next";
import SearchBox from "./SearchBox.vue";
import { useCanvasStore } from "../stores/canvasStore";
import { useNoteStore } from "../stores/noteStore";
import { useSettingsStore } from "../stores/settingsStore";
import { useTagStore } from "../stores/tagStore";

const canvasStore = useCanvasStore();
const noteStore = useNoteStore();
const tagStore = useTagStore();
const settingsStore = useSettingsStore();
const renamingId = ref("");
const renameDraft = ref("");
const showTrash = ref(false);

const filteredCanvases = computed(() => {
  const query = canvasStore.searchQuery.trim().toLowerCase();
  if (!query) return canvasStore.activeCanvases;
  return canvasStore.activeCanvases.filter((canvas) => canvas.name.toLowerCase().includes(query));
});

function relativeTime(value: string) {
  const diff = Date.now() - new Date(value).getTime();
  const hour = 3600 * 1000;
  if (diff < 60 * 1000) return "刚刚";
  if (diff < 24 * hour) return `${Math.max(1, Math.floor(diff / hour))} 小时前`;
  if (diff < 48 * hour) return "昨天";
  if (diff < 7 * 24 * hour) return `${Math.floor(diff / (24 * hour))} 天前`;
  return "上周";
}

function createCanvas() {
  const canvas = canvasStore.createCanvas();
  noteStore.clearSelection();
  showTrash.value = false;
  startRename(canvas.id, canvas.name);
}

function startRename(id: string, name: string) {
  renamingId.value = id;
  renameDraft.value = name;
  void nextTick(() => {
    const input = document.querySelector<HTMLInputElement>(`[data-rename-id="${id}"]`);
    input?.focus();
    input?.select();
  });
}

function commitRename(id: string) {
  const value = renameDraft.value.trim();
  if (value) canvasStore.renameCanvas(id, value);
  renamingId.value = "";
  renameDraft.value = "";
}

function cancelRename() {
  renamingId.value = "";
  renameDraft.value = "";
}

function selectCanvas(id: string) {
  showTrash.value = false;
  canvasStore.selectCanvas(id);
}

function deleteCanvas(id: string, name: string) {
  if (!window.confirm(`删除画布“${name}”？画布会先移入回收站。`)) return;
  canvasStore.deleteCanvas(id);
  noteStore.clearSelection();
}

function restoreCanvas(id: string) {
  canvasStore.restoreCanvas(id);
  canvasStore.selectCanvas(id);
  showTrash.value = false;
}

function removeForever(id: string, name: string) {
  if (!window.confirm(`永久删除画布“${name}”？此操作会删除其中所有便签，且无法撤销。`)) return;
  noteStore.removeNotesByCanvas(id);
  canvasStore.removeForever(id);
}
</script>

<template>
  <aside class="sidebar">
    <button class="menu icon-button" title="菜单">
      <Menu :size="22" />
    </button>

    <SearchBox v-model="canvasStore.searchQuery" />

    <section class="section">
      <div class="section-title">
        <span>画布</span>
        <button class="new-button" @click="createCanvas"><Plus :size="15" />新建</button>
      </div>

      <div v-if="!filteredCanvases.length && !showTrash" class="empty">还没有画布<br />点击“新建”开始整理你的想法</div>

      <button
        v-for="canvas in filteredCanvases"
        :key="canvas.id"
        class="canvas-row"
        :class="{ active: canvas.id === canvasStore.currentCanvasId && !showTrash }"
        @click="selectCanvas(canvas.id)"
        @dblclick="startRename(canvas.id, canvas.name)"
        @contextmenu.prevent="startRename(canvas.id, canvas.name)"
      >
        <FileText :size="16" />
        <input
          v-if="renamingId === canvas.id"
          v-model="renameDraft"
          :data-rename-id="canvas.id"
          @click.stop
          @keydown.enter.stop.prevent="commitRename(canvas.id)"
          @keydown.esc.stop.prevent="cancelRename"
          @blur="commitRename(canvas.id)"
        />
        <span v-else class="name">{{ canvas.name }}</span>
        <span class="time">{{ relativeTime(canvas.updatedAt) }}</span>
        <span class="row-actions">
          <button title="重命名" @click.stop="startRename(canvas.id, canvas.name)"><Pencil :size="14" /></button>
          <button title="删除" @click.stop="deleteCanvas(canvas.id, canvas.name)"><Trash2 :size="14" /></button>
        </span>
      </button>

      <button class="canvas-row trash" :class="{ active: showTrash }" @click="showTrash = !showTrash">
        <Archive :size="16" />
        <span class="name">回收站</span>
        <span class="time">{{ canvasStore.deletedCanvases.length || "" }}</span>
      </button>

      <div v-if="showTrash" class="trash-panel">
        <div v-if="!canvasStore.deletedCanvases.length" class="empty small">回收站为空</div>
        <div v-for="canvas in canvasStore.deletedCanvases" :key="canvas.id" class="trash-row">
          <FileText :size="15" />
          <span>{{ canvas.name }}</span>
          <button title="恢复" @click="restoreCanvas(canvas.id)"><RotateCcw :size="14" /></button>
          <button title="永久删除" @click="removeForever(canvas.id, canvas.name)"><X :size="14" /></button>
        </div>
      </div>
    </section>

    <section class="section tags">
      <div class="section-title">
        <span>标签</span>
        <button class="small-add" @click="tagStore.createTag()"><Plus :size="18" /></button>
      </div>
      <button
        v-for="tag in tagStore.tags"
        :key="tag.id"
        class="tag-row"
        :class="{ active: tag.id === tagStore.activeTagId }"
        @click="tagStore.toggleTag(tag.id)"
      >
        <i :style="{ backgroundColor: tag.color }"></i>
        <span>{{ tag.name }}</span>
        <b>{{ tag.count }}</b>
      </button>
    </section>

    <button class="settings" @click="settingsStore.togglePanel()">
      <Settings :size="17" />
      <span>设置</span>
    </button>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 280px;
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 18px 12px 16px;
  background: var(--sidebar-bg);
  border-right: 1px solid var(--border-color);
}

.menu {
  margin: 0 0 10px 0;
}

.section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.section-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 0 6px;
  font-weight: 700;
  color: #1f2937;
}

.new-button,
.small-add {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  height: 34px;
  padding: 0 10px;
  color: #374151;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 7px;
  box-shadow: var(--shadow-sm);
}

.small-add {
  width: 34px;
  padding: 0;
}

.canvas-row,
.tag-row,
.settings {
  position: relative;
  display: grid;
  grid-template-columns: 22px 1fr auto;
  align-items: center;
  gap: 8px;
  width: 100%;
  height: 42px;
  padding: 0 10px;
  color: #374151;
  background: transparent;
  border-radius: 7px;
  text-align: left;
}

.canvas-row {
  padding-right: 8px;
}

.canvas-row.active {
  color: #1d4ed8;
  background: var(--primary-soft);
}

.canvas-row.active::before {
  content: "";
  position: absolute;
  left: 0;
  width: 3px;
  height: 24px;
  border-radius: 3px;
  background: var(--primary);
}

.canvas-row:hover,
.tag-row:hover,
.settings:hover,
.tag-row.active {
  background: #eef2f7;
}

.name {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.time,
.tag-row b {
  font-weight: 500;
  font-size: 13px;
  color: var(--text-muted);
}

.canvas-row input {
  min-width: 0;
  border: 0;
  outline: 0;
  color: #374151;
  background: transparent;
}

.row-actions {
  display: none;
  align-items: center;
  gap: 2px;
}

.canvas-row:hover .row-actions {
  display: inline-flex;
}

.canvas-row:hover .time {
  display: none;
}

.row-actions button,
.trash-row button {
  width: 26px;
  height: 26px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: #6b7280;
  background: transparent;
  border-radius: 6px;
}

.row-actions button:hover,
.trash-row button:hover {
  color: #1f2937;
  background: #fff;
}

.trash {
  margin-top: 16px;
  border-top: 1px solid #edf0f3;
  border-radius: 0;
}

.trash-panel {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 2px 0 4px 14px;
}

.trash-row {
  display: grid;
  grid-template-columns: 20px 1fr 28px 28px;
  align-items: center;
  gap: 6px;
  min-height: 36px;
  padding: 0 4px 0 8px;
  color: #4b5563;
  border-radius: 7px;
}

.trash-row:hover {
  background: #eef2f7;
}

.trash-row span {
  min-width: 0;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.tags {
  margin-top: 14px;
}

.tag-row i {
  width: 12px;
  height: 12px;
  margin-left: 2px;
  border-radius: 50%;
}

.tag-row b {
  justify-self: end;
}

.settings {
  margin-top: auto;
  grid-template-columns: 22px 1fr;
}

.empty {
  padding: 22px 10px;
  line-height: 1.7;
  font-size: 14px;
  color: var(--text-muted);
}

.empty.small {
  padding: 10px 8px;
}
</style>
