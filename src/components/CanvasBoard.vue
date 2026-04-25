<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, watch } from "vue";
import CanvasToolbar from "./CanvasToolbar.vue";
import MiniMap from "./MiniMap.vue";
import StickyNote from "./StickyNote.vue";
import { useCanvasStore } from "../stores/canvasStore";
import { useNoteStore } from "../stores/noteStore";
import { useSettingsStore } from "../stores/settingsStore";
import { useTagStore } from "../stores/tagStore";
import { useFeedbackStore } from "../stores/feedbackStore";
import type { NoteColor, StickyNote as StickyNoteType, ViewportState } from "../types";
import { noteColorList, noteColors } from "../utils/colors";
import { clamp, screenToWorld } from "../utils/geometry";

const canvasStore = useCanvasStore();
const noteStore = useNoteStore();
const tagStore = useTagStore();
const settingsStore = useSettingsStore();
const feedback = useFeedbackStore();

const board = ref<HTMLElement>();
const viewport = reactive<ViewportState>({ offsetX: 0, offsetY: 0, scale: 1 });
const boardSize = reactive({ width: 0, height: 0 });
const handActive = ref(false);
const spaceDown = ref(false);
const panStart = ref<{ x: number; y: number; offsetX: number; offsetY: number }>();
const contextMenu = ref<{ x: number; y: number; noteId?: string } | null>(null);
const contextWorld = ref<{ x: number; y: number }>({ x: 0, y: 0 });
const highlightedNoteId = ref("");
const boxSelect = ref<{ startX: number; startY: number; currentX: number; currentY: number } | null>(null);
const groupDrag = ref<{ startX: number; startY: number; before: StickyNoteType[] } | null>(null);
let highlightTimer: number | undefined;

const visibleNotes = computed(() =>
  noteStore.notesForCanvas(canvasStore.currentCanvasId, tagStore.activeTagId, canvasStore.searchQuery),
);

const canvasClass = computed(() => ({
  "hide-grid": !settingsStore.settings.showGrid,
  "pan-ready": handActive.value || spaceDown.value,
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
  saveViewport();
}

function setZoom(scale: number) {
  const rect = board.value?.getBoundingClientRect();
  if (!rect) {
    viewport.scale = clamp(scale, 0.25, 3);
    saveViewport();
    return;
  }
  const world = screenToWorld(rect.left + rect.width / 2, rect.top + rect.height / 2, viewport, rect);
  viewport.scale = clamp(scale, 0.25, 3);
  viewport.offsetX = rect.width / 2 - world.x * viewport.scale;
  viewport.offsetY = rect.height / 2 - world.y * viewport.scale;
  saveViewport();
}

function resetZoom() {
  viewport.scale = 1;
  viewport.offsetX = 0;
  viewport.offsetY = 0;
  saveViewport();
}

function fitView() {
  const rect = board.value?.getBoundingClientRect();
  if (!rect || !visibleNotes.value.length) {
    resetZoom();
    return;
  }
  const padding = 140;
  const minX = Math.min(...visibleNotes.value.map((note) => note.x));
  const minY = Math.min(...visibleNotes.value.map((note) => note.y));
  const maxX = Math.max(...visibleNotes.value.map((note) => note.x + note.width));
  const maxY = Math.max(...visibleNotes.value.map((note) => note.y + note.height));
  const width = Math.max(1, maxX - minX);
  const height = Math.max(1, maxY - minY);
  viewport.scale = clamp(Math.min((rect.width - padding) / width, (rect.height - padding) / height), 0.25, 3);
  viewport.offsetX = rect.width / 2 - (minX + width / 2) * viewport.scale;
  viewport.offsetY = rect.height / 2 - (minY + height / 2) * viewport.scale;
  saveViewport();
}

function onWheel(event: WheelEvent) {
  if (event.ctrlKey) {
    event.preventDefault();
    zoomBy(event.deltaY > 0 ? -0.08 : 0.08, event.clientX, event.clientY);
    return;
  }
  event.preventDefault();
  if (event.shiftKey) viewport.offsetX -= event.deltaY;
  else {
    viewport.offsetX -= event.deltaX;
    viewport.offsetY -= event.deltaY;
  }
  saveViewport();
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
  saveViewport();
}

function isTextInputTarget(target: EventTarget | null) {
  if (!(target instanceof HTMLElement)) return false;
  return Boolean(target.closest(".tiptap, input, textarea, select, [contenteditable='true']"));
}

function isCanvasBlankTarget(target: EventTarget | null) {
  if (!(target instanceof HTMLElement)) return false;
  return !target.closest(".sticky-note, .toolbar, .minimap, .menu-popover, .empty-board");
}

function isCanvasControlTarget(target: EventTarget | null) {
  return target instanceof HTMLElement && Boolean(target.closest(".toolbar, .minimap, .menu-popover"));
}

function onBoardMouseDown(event: MouseEvent) {
  contextMenu.value = null;
  const blankTarget = isCanvasBlankTarget(event.target);
  if (noteStore.editingId && blankTarget) noteStore.stopEditing();
  if (blankTarget) {
    if (event.shiftKey || event.ctrlKey) {
      startBoxSelect(event);
    } else {
      noteStore.clearSelection();
    }
  }
  if (!isCanvasControlTarget(event.target) && (blankTarget || event.button === 1 || handActive.value || spaceDown.value)) startPan(event);
}

function onBoardDoubleClick(event: MouseEvent) {
  if (isCanvasBlankTarget(event.target)) createNoteAt(event.clientX, event.clientY);
}

function openCanvasMenu(event: MouseEvent) {
  if (board.value) {
    const rect = board.value.getBoundingClientRect();
    contextWorld.value = screenToWorld(event.clientX, event.clientY, viewport, rect);
  }
  contextMenu.value = { x: event.clientX, y: event.clientY };
}

function openNoteMenu(event: MouseEvent, id: string) {
  noteStore.select(id);
  if (board.value) {
    const rect = board.value.getBoundingClientRect();
    contextWorld.value = screenToWorld(event.clientX, event.clientY, viewport, rect);
  }
  contextMenu.value = { x: event.clientX, y: event.clientY, noteId: id };
}

function updateNote(note: StickyNoteType, patch: Partial<StickyNoteType> & { __before?: StickyNoteType }, track = true) {
  const { __before, ...cleanPatch } = patch;
  if (__before) noteStore.commitNoteChange(__before, cleanPatch);
  else noteStore.updateNote(note.id, cleanPatch, track);
}

function onNotePointerDown(event: MouseEvent, note: StickyNoteType) {
  if (handActive.value || spaceDown.value || event.button === 1) return;
  const additive = event.shiftKey || event.ctrlKey;
  if (!noteStore.selectedIds.includes(note.id)) noteStore.select(note.id, additive);
  if (noteStore.selectedIds.length > 1 && noteStore.selectedIds.includes(note.id)) {
    groupDrag.value = {
      startX: event.clientX,
      startY: event.clientY,
      before: noteStore.notes.filter((item) => noteStore.selectedIds.includes(item.id)).map((item) => ({ ...item, tags: [...item.tags] })),
    };
    window.addEventListener("mousemove", dragGroup);
    window.addEventListener("mouseup", endGroupDrag, { once: true });
  }
}

function dragGroup(event: MouseEvent) {
  if (!groupDrag.value) return;
  noteStore.moveSelectedBy((event.clientX - groupDrag.value.startX) / viewport.scale, (event.clientY - groupDrag.value.startY) / viewport.scale, groupDrag.value.before);
}

function endGroupDrag() {
  window.removeEventListener("mousemove", dragGroup);
  if (groupDrag.value) noteStore.commitSelectedMove(groupDrag.value.before);
  groupDrag.value = null;
}

function deleteSelection(noteId: string) {
  if (noteStore.selectedIds.length > 1 && noteStore.selectedIds.includes(noteId)) noteStore.deleteSelected();
  else noteStore.deleteNote(noteId);
}

function duplicateSelection(noteId: string) {
  if (noteStore.selectedIds.length > 1 && noteStore.selectedIds.includes(noteId)) noteStore.duplicateSelected();
  else noteStore.duplicateNote(noteId);
}

function jsonNodeText(node: unknown): string {
  if (!node || typeof node !== "object") return "";
  const value = node as { text?: unknown; content?: unknown };
  if (typeof value.text === "string") return value.text;
  return Array.isArray(value.content) ? value.content.map(jsonNodeText).join("") : "";
}

function plainTextFromContentJson(value: unknown): string {
  if (!value || typeof value !== "object") return "";
  const node = value as { type?: unknown; content?: unknown };
  if (Array.isArray(node.content)) {
    if (node.type === "paragraph" || node.type === "heading" || node.type === "listItem" || node.type === "taskItem") {
      return jsonNodeText(node);
    }
    return node.content.map(plainTextFromContentJson).filter(Boolean).join("\n");
  }
  return jsonNodeText(node);
}

async function copyNoteText(noteId: string) {
  const note = noteStore.notes.find((item) => item.id === noteId);
  const text = (note?.content?.trim() || plainTextFromContentJson(note?.contentJson).trim()) ?? "";
  if (!text) {
    feedback.notify("便签没有可复制的文字");
    return;
  }
  await navigator.clipboard.writeText(text);
  feedback.notify("文字已复制", "success");
}

function bringSelectionToFront(noteId: string) {
  if (noteStore.selectedIds.length > 1 && noteStore.selectedIds.includes(noteId)) noteStore.bringSelectedToFront();
  else noteStore.bringToFront(noteId);
}

function updateSelection(note: StickyNoteType, patch: Partial<StickyNoteType> & { __before?: StickyNoteType }, track = true) {
  if (noteStore.selectedIds.length > 1 && noteStore.selectedIds.includes(note.id) && !patch.__before) {
    noteStore.updateSelected(patch);
    return;
  }
  updateNote(note, patch, track);
}

function toggleTagForSelection(noteId: string, tagId: string) {
  if (noteStore.selectedIds.length > 1 && noteStore.selectedIds.includes(noteId)) noteStore.toggleTagForSelected(tagId);
  else noteStore.toggleTagForNote(noteId, tagId);
}

function changeColorForContext(noteId: string, color: NoteColor) {
  if (noteStore.selectedIds.length > 1 && noteStore.selectedIds.includes(noteId)) noteStore.updateSelected({ color });
  else noteStore.updateNote(noteId, { color });
  contextMenu.value = null;
}

function pasteAtContext() {
  noteStore.pasteClipboard(canvasStore.currentCanvasId, contextWorld.value.x, contextWorld.value.y);
  contextMenu.value = null;
}

function startBoxSelect(event: MouseEvent) {
  if (!board.value) return;
  const rect = board.value.getBoundingClientRect();
  boxSelect.value = {
    startX: event.clientX - rect.left,
    startY: event.clientY - rect.top,
    currentX: event.clientX - rect.left,
    currentY: event.clientY - rect.top,
  };
  window.addEventListener("mousemove", updateBoxSelect);
  window.addEventListener("mouseup", finishBoxSelect, { once: true });
}

function updateBoxSelect(event: MouseEvent) {
  if (!boxSelect.value || !board.value) return;
  const rect = board.value.getBoundingClientRect();
  boxSelect.value.currentX = event.clientX - rect.left;
  boxSelect.value.currentY = event.clientY - rect.top;
}

function finishBoxSelect() {
  window.removeEventListener("mousemove", updateBoxSelect);
  if (!boxSelect.value) return;
  const left = Math.min(boxSelect.value.startX, boxSelect.value.currentX);
  const top = Math.min(boxSelect.value.startY, boxSelect.value.currentY);
  const right = Math.max(boxSelect.value.startX, boxSelect.value.currentX);
  const bottom = Math.max(boxSelect.value.startY, boxSelect.value.currentY);
  const selected = visibleNotes.value.filter((note) => {
    const noteLeft = note.x * viewport.scale + viewport.offsetX;
    const noteTop = note.y * viewport.scale + viewport.offsetY;
    const noteRight = noteLeft + note.width * viewport.scale;
    const noteBottom = noteTop + note.height * viewport.scale;
    return noteRight >= left && noteLeft <= right && noteBottom >= top && noteTop <= bottom;
  });
  noteStore.setSelection(selected.map((note) => note.id));
  boxSelect.value = null;
}

const boxSelectStyle = computed(() => {
  if (!boxSelect.value) return {};
  const left = Math.min(boxSelect.value.startX, boxSelect.value.currentX);
  const top = Math.min(boxSelect.value.startY, boxSelect.value.currentY);
  return {
    left: `${left}px`,
    top: `${top}px`,
    width: `${Math.abs(boxSelect.value.currentX - boxSelect.value.startX)}px`,
    height: `${Math.abs(boxSelect.value.currentY - boxSelect.value.startY)}px`,
  };
});

function jumpMiniMap(x: number, y: number) {
  const rect = board.value?.getBoundingClientRect();
  if (!rect) return;
  viewport.offsetX = rect.width / 2 - x * viewport.scale;
  viewport.offsetY = rect.height / 2 - y * viewport.scale;
  saveViewport();
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
  if (isTextInputTarget(event.target)) return;
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
  if (event.ctrlKey && event.key.toLowerCase() === "v" && canvasStore.currentCanvasId && !noteStore.editingId) {
    event.preventDefault();
    const rect = board.value?.getBoundingClientRect();
    if (!rect) return;
    const point = screenToWorld(rect.left + rect.width / 2, rect.top + rect.height / 2, viewport, rect);
    noteStore.pasteClipboard(canvasStore.currentCanvasId, point.x, point.y);
  }
}

function onKeyup(event: KeyboardEvent) {
  if (event.code === "Space") spaceDown.value = false;
}

onMounted(() => {
  updateBoardSize();
  window.addEventListener("keydown", onKeydown);
  window.addEventListener("keyup", onKeyup);
  window.addEventListener("locate-note", onLocateNote);
  window.addEventListener("resize", updateBoardSize);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
  window.removeEventListener("keyup", onKeyup);
  window.removeEventListener("locate-note", onLocateNote);
  window.removeEventListener("resize", updateBoardSize);
  window.clearTimeout(highlightTimer);
});

function updateBoardSize() {
  const rect = board.value?.getBoundingClientRect();
  if (!rect) return;
  boardSize.width = rect.width;
  boardSize.height = rect.height;
}

function saveViewport() {
  if (!canvasStore.currentCanvasId) return;
  canvasStore.updateViewport(canvasStore.currentCanvasId, { ...viewport });
}

watch(
  () => canvasStore.currentCanvasId,
  () => {
    const saved = canvasStore.currentCanvas?.viewport;
    viewport.offsetX = saved?.offsetX ?? 0;
    viewport.offsetY = saved?.offsetY ?? 0;
    viewport.scale = saved?.scale ?? 1;
    requestAnimationFrame(updateBoardSize);
  },
);
</script>

<template>
  <section
    ref="board"
    class="canvas-board"
    :class="canvasClass"
    @mousedown="onBoardMouseDown"
    @dblclick="onBoardDoubleClick"
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
        :pan-mode="handActive || spaceDown"
        @select="onNotePointerDown($event, note)"
        @edit="noteStore.editingId = note.id"
        @update="(patch, track) => updateSelection(note, patch, track)"
        @live="(patch) => noteStore.patchNoteLive(note.id, patch)"
        @delete="deleteSelection(note.id)"
        @duplicate="duplicateSelection(note.id)"
        @copy-text="copyNoteText(note.id)"
        @front="bringSelectionToFront(note.id)"
        @context="(event) => openNoteMenu(event, note.id)"
        @editing-done="noteStore.stopEditing()"
        @toggle-tag="(tagId) => toggleTagForSelection(note.id, tagId)"
      />
    </div>

    <div v-if="boxSelect" class="selection-box" :style="boxSelectStyle"></div>

    <CanvasToolbar
      :scale="viewport.scale"
      :hand-active="handActive"
      @add="createNoteCenter"
      @undo="noteStore.undo"
      @redo="noteStore.redo"
      @zoom-in="zoomBy(0.1)"
      @zoom-out="zoomBy(-0.1)"
      @set-zoom="setZoom"
      @reset-zoom="resetZoom"
      @toggle-hand="handActive = !handActive"
      @settings="settingsStore.togglePanel()"
    />

    <MiniMap
      :notes="visibleNotes"
      :viewport="viewport"
      :board-width="boardSize.width"
      :board-height="boardSize.height"
      @zoom-in="zoomBy(0.1)"
      @zoom-out="zoomBy(-0.1)"
      @fit="fitView"
      @jump="jumpMiniMap"
    />

    <div
      v-if="contextMenu"
      class="menu-popover"
      :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
      @mousedown.stop
      @click.stop
      @contextmenu.prevent.stop
    >
      <template v-if="contextMenu.noteId">
        <button @click="noteStore.editingId = contextMenu!.noteId!; contextMenu = null">编辑</button>
        <button @click="copyNoteText(contextMenu!.noteId!); contextMenu = null">复制文字</button>
        <button @click="duplicateSelection(contextMenu!.noteId!); contextMenu = null">复制便签</button>
        <button @click="bringSelectionToFront(contextMenu!.noteId!); contextMenu = null">
          {{ noteStore.notes.find((note) => note.id === contextMenu!.noteId)?.pinned ? "取消置顶" : "置顶" }}
        </button>
        <button @click="deleteSelection(contextMenu!.noteId!); contextMenu = null">删除</button>
        <div class="context-section">
          <span>颜色</span>
          <div class="context-swatches">
            <button
              v-for="color in noteColorList"
              :key="color"
              :style="{ backgroundColor: noteColors[color] }"
              @click="changeColorForContext(contextMenu!.noteId!, color)"
            ></button>
          </div>
        </div>
      </template>
      <template v-else>
        <button @click="createNoteAt(contextMenu!.x, contextMenu!.y); contextMenu = null">新建便签</button>
        <button :disabled="!noteStore.clipboard.length" @click="pasteAtContext">粘贴便签</button>
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

.canvas-board.pan-ready {
  cursor: grab;
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

.selection-box {
  position: absolute;
  z-index: 40;
  pointer-events: none;
  border: 1px solid rgba(59, 130, 246, 0.85);
  background: rgba(59, 130, 246, 0.12);
}

.context-section {
  padding: 6px 4px 4px;
  border-top: 1px solid #eef1f4;
}

.context-section > span {
  display: block;
  padding: 2px 4px 5px;
  color: #9ca3af;
  font-size: 12px;
  font-weight: 700;
}

.context-swatches {
  display: grid;
  grid-template-columns: repeat(4, 24px);
  gap: 5px;
  padding: 0 4px 4px;
}

.context-swatches button {
  width: 22px;
  height: 22px;
  min-height: 22px;
  padding: 0;
  border: 1px solid rgba(0, 0, 0, 0.12);
  border-radius: 50%;
}

.menu-popover button:disabled {
  cursor: default;
  opacity: 0.45;
}
</style>
