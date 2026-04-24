<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref } from "vue";
import CanvasToolbar from "./CanvasToolbar.vue";
import MiniMap from "./MiniMap.vue";
import StickyNote from "./StickyNote.vue";
import { useCanvasStore } from "../stores/canvasStore";
import { useNoteStore } from "../stores/noteStore";
import { useSettingsStore } from "../stores/settingsStore";
import { useTagStore } from "../stores/tagStore";
import type { StickyNote as StickyNoteType, ViewportState } from "../types";
import { clamp, screenToWorld } from "../utils/geometry";

const canvasStore = useCanvasStore();
const noteStore = useNoteStore();
const tagStore = useTagStore();
const settingsStore = useSettingsStore();

const board = ref<HTMLElement>();
const viewport = reactive<ViewportState>({ offsetX: 0, offsetY: 0, scale: 1 });
const handActive = ref(false);
const spaceDown = ref(false);
const panStart = ref<{ x: number; y: number; offsetX: number; offsetY: number }>();
const contextMenu = ref<{ x: number; y: number; noteId?: string } | null>(null);
const highlightedNoteId = ref("");
let highlightTimer: number | undefined;

const visibleNotes = computed(() =>
  noteStore.notesForCanvas(canvasStore.currentCanvasId, tagStore.activeTagId, canvasStore.searchQuery),
);

const canvasClass = computed(() => ({
  "hide-grid": !settingsStore.settings.showGrid,
  panning: Boolean(panStart.value),
}));

function createNoteAt(clientX: number, clientY: number) {
  if (!canvasStore.currentCanvasId || !board.value) return;
  const rect = board.value.getBoundingClientRect();
  const point = screenToWorld(clientX, clientY, viewport, rect);
  noteStore.createNote(
    canvasStore.currentCanvasId,
    point.x - 110,
    point.y - 90,
    settingsStore.settings.defaultNoteColor,
    settingsStore.settings.defaultFontSize,
    settingsStore.settings.randomRotation,
  );
}

function createNoteCenter() {
  const rect = board.value?.getBoundingClientRect();
  if (!rect) return;
  createNoteAt(rect.left + rect.width / 2, rect.top + rect.height / 2);
}

function zoomBy(delta: number, originX?: number, originY?: number) {
  const rect = board.value?.getBoundingClientRect();
  const oldScale = viewport.scale;
  const newScale = clamp(oldScale + delta, 0.25, 3);
  if (!rect) {
    viewport.scale = newScale;
    return;
  }
  const cx = originX ?? rect.left + rect.width / 2;
  const cy = originY ?? rect.top + rect.height / 2;
  const world = screenToWorld(cx, cy, viewport, rect);
  viewport.scale = newScale;
  viewport.offsetX = cx - rect.left - world.x * newScale;
  viewport.offsetY = cy - rect.top - world.y * newScale;
}

function resetZoom() {
  viewport.scale = 1;
  viewport.offsetX = 0;
  viewport.offsetY = 0;
}

function fitView() {
  viewport.scale = 0.82;
  viewport.offsetX = 60;
  viewport.offsetY = 20;
}

function onWheel(event: WheelEvent) {
  if (event.ctrlKey) {
    event.preventDefault();
    zoomBy(event.deltaY > 0 ? -0.08 : 0.08, event.clientX, event.clientY);
  }
}

function startPan(event: MouseEvent) {
  if (event.button === 1 || handActive.value || spaceDown.value) {
    event.preventDefault();
    panStart.value = { x: event.clientX, y: event.clientY, offsetX: viewport.offsetX, offsetY: viewport.offsetY };
    window.addEventListener("mousemove", pan);
    window.addEventListener("mouseup", stopPan, { once: true });
  }
}

function pan(event: MouseEvent) {
  if (!panStart.value) return;
  viewport.offsetX = panStart.value.offsetX + event.clientX - panStart.value.x;
  viewport.offsetY = panStart.value.offsetY + event.clientY - panStart.value.y;
}

function stopPan() {
  window.removeEventListener("mousemove", pan);
  panStart.value = undefined;
}

function onBoardMouseDown(event: MouseEvent) {
  contextMenu.value = null;
  if (noteStore.editingId && (event.target as HTMLElement).classList.contains("canvas-board")) noteStore.stopEditing();
  if ((event.target as HTMLElement).classList.contains("canvas-board")) noteStore.clearSelection();
  startPan(event);
}

function openCanvasMenu(event: MouseEvent) {
  contextMenu.value = { x: event.clientX, y: event.clientY };
}

function openNoteMenu(event: MouseEvent, id: string) {
  noteStore.select(id);
  contextMenu.value = { x: event.clientX, y: event.clientY, noteId: id };
}

function updateNote(note: StickyNoteType, patch: Partial<StickyNoteType> & { __before?: StickyNoteType }, track = true) {
  const { __before, ...cleanPatch } = patch;
  if (__before) noteStore.commitNoteChange(__before, cleanPatch);
  else noteStore.updateNote(note.id, cleanPatch, track);
}

function jumpMiniMap(x: number, y: number) {
  const rect = board.value?.getBoundingClientRect();
  if (!rect) return;
  viewport.offsetX = rect.width / 2 - x * viewport.scale;
  viewport.offsetY = rect.height / 2 - y * viewport.scale;
}

function centerNote(noteId: string) {
  const note = noteStore.notes.find((item) => item.id === noteId);
  const rect = board.value?.getBoundingClientRect();
  if (!note || !rect) return;
  viewport.offsetX = rect.width / 2 - (note.x + note.width / 2) * viewport.scale;
  viewport.offsetY = rect.height / 2 - (note.y + note.height / 2) * viewport.scale;
  noteStore.select(noteId);
  highlightedNoteId.value = noteId;
  window.clearTimeout(highlightTimer);
  highlightTimer = window.setTimeout(() => {
    highlightedNoteId.value = "";
  }, 1400);
}

function onLocateNote(event: Event) {
  const noteId = (event as CustomEvent<{ noteId: string }>).detail?.noteId;
  if (!noteId) return;
  requestAnimationFrame(() => centerNote(noteId));
}

function onKeydown(event: KeyboardEvent) {
  if (event.code === "Space" && !noteStore.editingId) {
    event.preventDefault();
    spaceDown.value = true;
  }
  if (event.ctrlKey && (event.key === "+" || event.key === "=")) {
    event.preventDefault();
    zoomBy(0.1);
  }
  if (event.ctrlKey && event.key === "-") {
    event.preventDefault();
    zoomBy(-0.1);
  }
  if (event.ctrlKey && event.key === "0") {
    event.preventDefault();
    resetZoom();
  }
}

function onKeyup(event: KeyboardEvent) {
  if (event.code === "Space") spaceDown.value = false;
}

onMounted(() => {
  window.addEventListener("keydown", onKeydown);
  window.addEventListener("keyup", onKeyup);
  window.addEventListener("locate-note", onLocateNote);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
  window.removeEventListener("keyup", onKeyup);
  window.removeEventListener("locate-note", onLocateNote);
  window.clearTimeout(highlightTimer);
});
</script>

<template>
  <section
    ref="board"
    class="canvas-board"
    :class="canvasClass"
    @mousedown="onBoardMouseDown"
    @dblclick.self="createNoteAt($event.clientX, $event.clientY)"
    @wheel="onWheel"
    @contextmenu.prevent="openCanvasMenu"
  >
    <div v-if="!canvasStore.currentCanvasId" class="empty-board">还没有画布<br />点击“新建”开始整理你的想法</div>
    <div v-else-if="!visibleNotes.length" class="empty-board">双击画布空白处创建第一张便签</div>

    <div class="canvas-content" :style="{ transform: `translate(${viewport.offsetX}px, ${viewport.offsetY}px) scale(${viewport.scale})` }">
      <StickyNote
        v-for="note in visibleNotes"
        :key="note.id"
        :note="note"
        :selected="noteStore.selectedIds.includes(note.id)"
        :editing="noteStore.editingId === note.id"
        :shadow="settingsStore.settings.noteShadow"
        :scale="viewport.scale"
        :tags="tagStore.tags"
        :search-query="canvasStore.searchQuery"
        :highlighted="highlightedNoteId === note.id"
        @select="noteStore.select(note.id, $event.shiftKey)"
        @edit="noteStore.editingId = note.id"
        @update="(patch, track) => updateNote(note, patch, track)"
        @live="(patch) => noteStore.patchNoteLive(note.id, patch)"
        @delete="noteStore.deleteNote(note.id)"
        @duplicate="noteStore.duplicateNote(note.id)"
        @front="noteStore.bringToFront(note.id)"
        @context="(event) => openNoteMenu(event, note.id)"
        @editing-done="noteStore.stopEditing()"
        @toggle-tag="(tagId) => noteStore.toggleTagForNote(note.id, tagId)"
      />
    </div>

    <CanvasToolbar
      :scale="viewport.scale"
      :hand-active="handActive"
      @add="createNoteCenter"
      @undo="noteStore.undo"
      @redo="noteStore.redo"
      @zoom-in="zoomBy(0.1)"
      @zoom-out="zoomBy(-0.1)"
      @reset-zoom="resetZoom"
      @toggle-hand="handActive = !handActive"
      @settings="settingsStore.togglePanel()"
    />

    <MiniMap :notes="visibleNotes" :viewport="viewport" @zoom-in="zoomBy(0.1)" @zoom-out="zoomBy(-0.1)" @fit="fitView" @jump="jumpMiniMap" />

    <div v-if="contextMenu" class="menu-popover" :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }">
      <template v-if="contextMenu.noteId">
        <button @click="noteStore.editingId = contextMenu!.noteId!; contextMenu = null">编辑</button>
        <button @click="noteStore.duplicateNote(contextMenu!.noteId!); contextMenu = null">复制</button>
        <button @click="noteStore.bringToFront(contextMenu!.noteId!); contextMenu = null">置顶</button>
        <button @click="noteStore.deleteNote(contextMenu!.noteId!); contextMenu = null">删除</button>
      </template>
      <template v-else>
        <button @click="createNoteAt(contextMenu!.x, contextMenu!.y); contextMenu = null">新建便签</button>
        <button @click="fitView(); contextMenu = null">适应视图</button>
        <button @click="resetZoom(); contextMenu = null">重置缩放</button>
      </template>
    </div>
  </section>
</template>

<style scoped>
.canvas-board {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
  background-color: #fbfaf8;
  background-image: radial-gradient(#d8d5cf 1px, transparent 1px);
  background-size: 18px 18px;
}

.canvas-board.hide-grid {
  background-image: none;
}

.canvas-board.panning {
  cursor: grabbing;
}

.canvas-content {
  position: absolute;
  inset: 0;
  width: 2800px;
  height: 2200px;
  transform-origin: 0 0;
}

.empty-board {
  position: absolute;
  top: 50%;
  left: 50%;
  z-index: 1;
  padding: 18px 22px;
  line-height: 1.8;
  color: var(--text-muted);
  background: rgba(255, 255, 255, 0.72);
  border: 1px solid #eceff3;
  border-radius: 8px;
  transform: translate(-50%, -50%);
}
</style>
