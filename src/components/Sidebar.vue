<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from "vue";
import { Archive, ArrowDown, ArrowUp, Ellipsis, FileText, Menu, Pencil, Plus, RotateCcw, Settings, Trash2, X } from "lucide-vue-next";
import SearchBox from "./SearchBox.vue";
import { useAppStore } from "../stores/appStore";
import { useCanvasStore } from "../stores/canvasStore";
import { useNoteStore } from "../stores/noteStore";
import { useDrawingStore } from "../stores/drawingStore";
import { useSettingsStore } from "../stores/settingsStore";
import { useTagStore } from "../stores/tagStore";
import { useFeedbackStore } from "../stores/feedbackStore";
import packageInfo from "../../package.json";

const appStore = useAppStore();
const canvasStore = useCanvasStore();
const noteStore = useNoteStore();
const drawingStore = useDrawingStore();
const tagStore = useTagStore();
const settingsStore = useSettingsStore();
const feedback = useFeedbackStore();
const renamingId = ref("");
const renameDraft = ref("");
const showTrash = ref(false);
const renamingTagId = ref("");
const tagDraft = ref("");
const tagColorDraft = ref("#3b82f6");
const collapsed = ref(false);
const openCanvasMenuId = ref("");
const openTagMenuId = ref("");
const appVersion = packageInfo.version;
const hasSearchQuery = computed(() => Boolean(canvasStore.searchQuery.trim()));

const filteredCanvases = computed(() => {
  return canvasStore.activeCanvases;
});

function canvasIndex(id: string) {
  return filteredCanvases.value.findIndex((canvas) => canvas.id === id);
}

function canMoveCanvasUp(id: string) {
  return canvasIndex(id) > 0;
}

function canMoveCanvasDown(id: string) {
  const index = canvasIndex(id);
  return index >= 0 && index < filteredCanvases.value.length - 1;
}

const searchResults = computed(() => {
  const query = canvasStore.searchQuery.trim().toLowerCase();
  if (!query) return { canvases: [], notes: [], tags: [] };
  return {
    canvases: canvasStore.activeCanvases.filter((canvas) => canvas.name.toLowerCase().includes(query)),
    notes: noteStore.notes.filter((note) => note.content.toLowerCase().includes(query) || note.title?.toLowerCase().includes(query)).slice(0, 12),
    tags: tagStore.tags.filter((tag) => tag.name.toLowerCase().includes(query)),
  };
});

const hasSearchResults = computed(
  () => Boolean(canvasStore.searchQuery.trim()) && (searchResults.value.canvases.length > 0 || searchResults.value.notes.length > 0 || searchResults.value.tags.length > 0),
);

function tagIndex(id: string) {
  return tagStore.orderedTags.findIndex((tag) => tag.id === id);
}

function canMoveTagUp(id: string) {
  return tagIndex(id) > 0;
}

function canMoveTagDown(id: string) {
  const index = tagIndex(id);
  return index >= 0 && index < tagStore.orderedTags.length - 1;
}

function createCanvas() {
  if (!appStore.databaseReady) {
    feedback.notify(appStore.loadError || "数据库未加载", "error");
    return;
  }
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
  openCanvasMenuId.value = "";
  openTagMenuId.value = "";
  canvasStore.selectCanvas(id);
}

function toggleCanvasMenu(id: string) {
  openCanvasMenuId.value = openCanvasMenuId.value === id ? "" : id;
}

function moveCanvas(id: string, delta: -1 | 1) {
  canvasStore.moveCanvas(id, delta);
  openCanvasMenuId.value = "";
}

function renameCanvasFromMenu(id: string, name: string) {
  openCanvasMenuId.value = "";
  startRename(id, name);
}

async function deleteCanvasFromMenu(id: string, name: string) {
  openCanvasMenuId.value = "";
  await deleteCanvas(id, name);
}

function toggleTagMenu(id: string) {
  openTagMenuId.value = openTagMenuId.value === id ? "" : id;
}

function moveTag(id: string, delta: -1 | 1) {
  tagStore.moveTag(id, delta);
  openTagMenuId.value = "";
}

function renameTagFromMenu(id: string, name: string, color: string) {
  openTagMenuId.value = "";
  startRenameTag(id, name, color);
}

async function deleteTagFromMenu(id: string, name: string) {
  openTagMenuId.value = "";
  await deleteTag(id, name);
}

async function deleteCanvas(id: string, name: string) {
  if (!(await feedback.confirm(`删除画布“${name}”？画布会先移入回收站。`))) return;
  canvasStore.deleteCanvas(id);
  noteStore.clearSelection();
  feedback.notify("画布已移入回收站", "success");
}

function restoreCanvas(id: string) {
  canvasStore.restoreCanvas(id);
  canvasStore.selectCanvas(id);
  showTrash.value = false;
}

async function removeForever(id: string, name: string) {
  if (!(await feedback.confirm(`永久删除画布“${name}”？此操作会删除其中所有便签，且无法撤销。`))) return;
  noteStore.removeNotesByCanvas(id);
  drawingStore.removeDrawingsByCanvas(id);
  canvasStore.removeForever(id);
  feedback.notify("画布已永久删除", "success");
}

function createTag() {
  if (!appStore.databaseReady) {
    feedback.notify(appStore.loadError || "数据库未加载", "error");
    return;
  }
  const tag = tagStore.createTag();
  startRenameTag(tag.id, tag.name, tag.color);
}

function startRenameTag(id: string, name: string, color: string) {
  renamingTagId.value = id;
  tagDraft.value = name;
  tagColorDraft.value = color;
  void nextTick(() => {
    const input = document.querySelector<HTMLInputElement>(`[data-tag-rename-id="${id}"]`);
    input?.focus();
    input?.select();
  });
}

function changeTagColor(id: string, color: string) {
  tagColorDraft.value = color;
  tagStore.updateTagColor(id, color);
}

function commitTag(id: string) {
  const value = tagDraft.value.trim();
  if (value) tagStore.renameTag(id, value);
  tagStore.updateTagColor(id, tagColorDraft.value);
  renamingTagId.value = "";
  tagDraft.value = "";
}

function cancelTagRename() {
  renamingTagId.value = "";
  tagDraft.value = "";
}

async function deleteTag(id: string, name: string) {
  if (!(await feedback.confirm(`删除标签“${name}”？该标签会从所有便签中移除。`))) return;
  noteStore.removeTagFromAll(id);
  tagStore.deleteTag(id);
  feedback.notify("标签已删除", "success");
}

function snippet(content: string) {
  const normalized = content.replace(/\s+/g, " ").trim();
  return normalized.length > 34 ? `${normalized.slice(0, 34)}...` : normalized || "空便签";
}

function canvasName(id: string) {
  return canvasStore.canvases.find((canvas) => canvas.id === id)?.name ?? "未知画布";
}

function selectSearchNote(noteId: string, canvasId: string) {
  showTrash.value = false;
  tagStore.activeTagId = "";
  canvasStore.selectCanvas(canvasId);
  noteStore.select(noteId);
  window.dispatchEvent(new CustomEvent("locate-note", { detail: { noteId } }));
}

function selectSearchTag(tagId: string) {
  openCanvasMenuId.value = "";
  openTagMenuId.value = "";
  tagStore.activeTagId = tagId;
  canvasStore.searchQuery = "";
}

function selectSearchCanvas(id: string) {
  selectCanvas(id);
  canvasStore.searchQuery = "";
}

function closeCanvasMenu() {
  openCanvasMenuId.value = "";
  openTagMenuId.value = "";
}

onMounted(() => {
  window.addEventListener("mousedown", closeCanvasMenu);
});

onUnmounted(() => {
  window.removeEventListener("mousedown", closeCanvasMenu);
});
</script>

<template>
  <aside class="sidebar" :class="{ collapsed }">
    <div class="brand-row">
      <button class="menu icon-button" title="收起/展开侧边栏" @click="collapsed = !collapsed">
        <Menu :size="22" />
      </button>
      <div v-if="!collapsed" class="brand">
        <strong>贴境</strong>
        <span>v{{ appVersion }}</span>
      </div>
    </div>

    <template v-if="!collapsed">
      <SearchBox v-model="canvasStore.searchQuery" class="sidebar-search" />

      <div class="navigation-scroll">
        <section v-if="hasSearchQuery" class="search-results">
        <template v-if="hasSearchResults">
          <div v-if="searchResults.canvases.length" class="result-group">
            <h3>画布</h3>
            <button v-for="canvas in searchResults.canvases" :key="canvas.id" @click="selectSearchCanvas(canvas.id)">
              <FileText :size="14" />
              <span>{{ canvas.name }}</span>
            </button>
          </div>
          <div v-if="searchResults.notes.length" class="result-group">
            <h3>便签</h3>
            <button v-for="note in searchResults.notes" :key="note.id" @click="selectSearchNote(note.id, note.canvasId)">
              <FileText :size="14" />
              <span>{{ snippet(note.content) }}</span>
              <small>{{ canvasName(note.canvasId) }}</small>
            </button>
          </div>
          <div v-if="searchResults.tags.length" class="result-group">
            <h3>标签</h3>
            <button v-for="tag in searchResults.tags" :key="tag.id" @click="selectSearchTag(tag.id)">
              <i :style="{ backgroundColor: tag.color }"></i>
              <span>{{ tag.name }}</span>
            </button>
          </div>
        </template>
        <div v-else class="empty small">没有匹配结果</div>
        </section>

        <section v-if="!hasSearchQuery" class="section">
        <div class="section-title">
          <span>画布</span>
          <button class="new-button" :disabled="!appStore.databaseReady" @click="createCanvas"><Plus :size="15" />新建</button>
        </div>

        <div v-if="!filteredCanvases.length && !showTrash" class="empty">还没有画布<br />点击“新建”开始整理你的想法</div>

        <div v-for="canvas in filteredCanvases" :key="canvas.id" class="canvas-row-wrap">
          <div
            class="canvas-row"
            :class="{ active: canvas.id === canvasStore.currentCanvasId && !showTrash }"
            role="button"
            tabindex="0"
            @click="selectCanvas(canvas.id)"
            @dblclick="startRename(canvas.id, canvas.name)"
            @contextmenu.prevent="renameCanvasFromMenu(canvas.id, canvas.name)"
            @keydown.enter.prevent="selectCanvas(canvas.id)"
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
            <span class="row-actions">
              <button title="更多" @mousedown.stop @click.stop="toggleCanvasMenu(canvas.id)"><Ellipsis :size="16" /></button>
            </span>
          </div>
          <div v-if="openCanvasMenuId === canvas.id" class="row-action-menu" @mousedown.stop @click.stop>
            <button :disabled="!canMoveCanvasUp(canvas.id)" @click="moveCanvas(canvas.id, -1)"><ArrowUp :size="14" />上移</button>
            <button :disabled="!canMoveCanvasDown(canvas.id)" @click="moveCanvas(canvas.id, 1)"><ArrowDown :size="14" />下移</button>
            <button @click="renameCanvasFromMenu(canvas.id, canvas.name)"><Pencil :size="14" />重命名</button>
            <button class="danger" @click="deleteCanvasFromMenu(canvas.id, canvas.name)"><Trash2 :size="14" />删除</button>
          </div>
        </div>

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

        <section class="section tags" @mousedown="openCanvasMenuId = ''">
        <div class="section-title">
          <span>标签</span>
          <button class="small-add" :disabled="!appStore.databaseReady" @click="createTag"><Plus :size="18" /></button>
        </div>
        <div v-for="tag in tagStore.orderedTags" :key="tag.id" class="tag-row-wrap">
          <div
            class="tag-row"
            :class="{ active: tag.id === tagStore.activeTagId }"
            role="button"
            tabindex="0"
            @click="tagStore.toggleTag(tag.id)"
            @dblclick="startRenameTag(tag.id, tag.name, tag.color)"
            @contextmenu.prevent="renameTagFromMenu(tag.id, tag.name, tag.color)"
            @keydown.enter.prevent="tagStore.toggleTag(tag.id)"
          >
            <label class="tag-color-control" title="修改颜色" @click.stop @dblclick.stop @mousedown.stop>
              <i :style="{ backgroundColor: tag.color }"></i>
              <input
                type="color"
                :value="tag.color"
                @input="changeTagColor(tag.id, ($event.target as HTMLInputElement).value)"
                @change="changeTagColor(tag.id, ($event.target as HTMLInputElement).value)"
              />
            </label>
            <input
              v-if="renamingTagId === tag.id"
              v-model="tagDraft"
              :data-tag-rename-id="tag.id"
              class="tag-name-input"
              @click.stop
              @keydown.enter.stop.prevent="commitTag(tag.id)"
              @keydown.esc.stop.prevent="cancelTagRename"
              @blur="commitTag(tag.id)"
            />
            <span v-else>{{ tag.name }}</span>
            <span class="tag-meta">
              <b>{{ tag.count }}</b>
              <button title="更多" @mousedown.stop @click.stop="toggleTagMenu(tag.id)"><Ellipsis :size="16" /></button>
            </span>
          </div>
          <div v-if="openTagMenuId === tag.id" class="row-action-menu" @mousedown.stop @click.stop>
            <button :disabled="!canMoveTagUp(tag.id)" @click="moveTag(tag.id, -1)"><ArrowUp :size="14" />上移</button>
            <button :disabled="!canMoveTagDown(tag.id)" @click="moveTag(tag.id, 1)"><ArrowDown :size="14" />下移</button>
            <button @click="renameTagFromMenu(tag.id, tag.name, tag.color)"><Pencil :size="14" />重命名</button>
            <button class="danger" @click="deleteTagFromMenu(tag.id, tag.name)"><Trash2 :size="14" />删除</button>
          </div>
        </div>
        </section>
      </div>
    </template>

    <button class="settings" :class="{ compact: collapsed }" @click="settingsStore.togglePanel()">
      <Settings :size="17" />
      <span v-if="!collapsed">设置</span>
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
  transition: width 0.18s ease, padding 0.18s ease;
}

.sidebar.collapsed {
  width: 64px;
  align-items: center;
  padding: 18px 10px 16px;
}

.brand-row {
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  gap: 10px;
  min-height: 38px;
  margin: 0 0 10px 0;
}

.brand {
  min-width: 0;
  display: grid;
  grid-template-columns: auto auto;
  align-items: baseline;
  gap: 8px;
}

.brand strong {
  color: #1f2937;
  font-size: 17px;
  font-weight: 800;
}

.brand span {
  color: #9ca3af;
  font-size: 12px;
  font-weight: 600;
}

.sidebar-search {
  flex: 0 0 auto;
}

.navigation-scroll {
  flex: 1 1 auto;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 0;
  margin-right: -8px;
  padding-right: 8px;
  overflow-x: hidden;
  overflow-y: auto;
  scrollbar-gutter: stable;
  scrollbar-width: thin;
  scrollbar-color: rgba(148, 163, 184, 0.58) transparent;
}

.navigation-scroll::-webkit-scrollbar {
  width: 8px;
}

.navigation-scroll::-webkit-scrollbar-track {
  background: transparent;
}

.navigation-scroll::-webkit-scrollbar-thumb {
  background: rgba(148, 163, 184, 0.42);
  border: 2px solid transparent;
  border-radius: 999px;
  background-clip: content-box;
}

.navigation-scroll:hover::-webkit-scrollbar-thumb {
  background: rgba(107, 114, 128, 0.5);
  border: 2px solid transparent;
  background-clip: content-box;
}

.section {
  flex: 0 0 auto;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.search-results {
  flex: 0 0 auto;
  padding: 8px;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  box-shadow: var(--shadow-sm);
}

.result-group + .result-group {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid #eef1f4;
}

.result-group h3 {
  margin: 0 0 5px;
  padding: 0 4px;
  color: #9ca3af;
  font-size: 12px;
  font-weight: 700;
}

.result-group button {
  width: 100%;
  min-height: 32px;
  display: grid;
  grid-template-columns: 18px 1fr auto;
  align-items: center;
  gap: 6px;
  padding: 0 7px;
  color: #374151;
  background: transparent;
  border-radius: 7px;
  text-align: left;
  cursor: var(--cursor-pointer);
}

.result-group button:hover {
  background: #eef2f7;
}

.result-group button span {
  min-width: 0;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.result-group button small {
  max-width: 76px;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  color: #9ca3af;
  font-size: 12px;
}

.result-group button i {
  width: 10px;
  height: 10px;
  border-radius: 50%;
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

.new-button:disabled,
.small-add:disabled {
  cursor: var(--cursor-not-allowed);
  color: #9ca3af;
  background: #f3f4f6;
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

.canvas-row-wrap,
.tag-row-wrap {
  position: relative;
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

.tag-meta {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.tag-meta button {
  width: 26px;
  height: 26px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: #6b7280;
  background: transparent;
  border-radius: 6px;
}

.tag-meta button:hover {
  color: #1f2937;
  background: #fff;
}

.canvas-row input {
  min-width: 0;
  border: 0;
  outline: 0;
  color: #374151;
  background: transparent;
}

.row-actions {
  display: inline-flex;
  align-items: center;
  gap: 2px;
}

.canvas-row:hover .time,
.tag-row:hover b {
  display: inline;
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

.row-actions button:disabled {
  cursor: var(--cursor-default);
  opacity: 0.34;
}

.row-action-menu {
  position: absolute;
  top: calc(100% - 2px);
  right: 8px;
  z-index: 20;
  width: 112px;
  padding: 5px;
  background: rgba(255, 255, 255, 0.98);
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  box-shadow: 0 10px 24px rgba(15, 23, 42, 0.14);
}

.row-action-menu button {
  width: 100%;
  min-height: 30px;
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 0 8px;
  color: #374151;
  background: transparent;
  border-radius: 6px;
  font-size: 13px;
  text-align: left;
}

.row-action-menu button:hover {
  background: #f3f6fb;
}

.row-action-menu button:disabled {
  cursor: var(--cursor-default);
  color: #aeb6c2;
  background: transparent;
}

.row-action-menu button.danger {
  color: #b91c1c;
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

.tag-color-control {
  position: relative;
  display: inline-grid;
  place-items: center;
  width: 22px;
  height: 22px;
  cursor: var(--cursor-pointer);
}

.tag-color-control i {
  width: 12px;
  height: 12px;
  margin-left: 2px;
  border-radius: 50%;
}

.tag-color-control:hover i {
  outline: 2px solid rgba(59, 130, 246, 0.24);
  outline-offset: 3px;
}

.tag-row b {
  justify-self: end;
}

.tag-row {
  grid-template-columns: 22px 1fr auto;
}

.tag-color-control input {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  opacity: 0;
  cursor: var(--cursor-pointer);
}

.tag-name-input {
  min-width: 0;
  border: 0;
  outline: 0;
  color: #374151;
  background: transparent;
}

.settings {
  flex: 0 0 auto;
  margin-top: auto;
  grid-template-columns: 22px 1fr;
  justify-items: start;
}

.settings.compact {
  width: 42px;
  height: 42px;
  grid-template-columns: 1fr;
  padding: 0;
  justify-items: center;
  justify-self: center;
}

.settings.compact svg {
  justify-self: center;
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
