<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref } from "vue";
import CanvasToolbar from "./CanvasToolbar.vue";
import MiniMap from "./MiniMap.vue";
import StickyNote from "./StickyNote.vue";
import { useCanvasStore } from "../stores/canvasStore";
import { useNoteStore } from "../stores/noteStore";
import { useSettingsStore } from "../stores/settingsStore";
import { useTagStore } from "../stores/tagStore";
import type { NoteColor, StickyNote as StickyNoteType, ViewportState } from "../types";
import { noteColorList, noteColors } from "../utils/colors";
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
  if ((event.target as HTMLElement).classList.contains("canvas-board")) {
    if (event.shiftKey || event.ctrlKey) {
      startBoxSelect(event);
    } else {
      noteStore.clearSelection();
    }
  }
  startPan(event);
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
        @select="onNotePointerDown($event, note)"
        @edit="noteStore.editingId = note.id"
        @update="(patch, track) => updateSelection(note, patch, track)"
        @live="(patch) => noteStore.patchNoteLive(note.id, patch)"
        @delete="deleteSelection(note.id)"
        @duplicate="duplicateSelection(note.id)"
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
      @reset-zoom="resetZoom"
      @toggle-hand="handActive = !handActive"
      @settings="settingsStore.togglePanel()"
    />

    <MiniMap :notes="visibleNotes" :viewport="viewport" @zoom-in="zoomBy(0.1)" @zoom-out="zoomBy(-0.1)" @fit="fitView" @jump="jumpMiniMap" />

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
        <button @click="duplicateSelection(contextMenu!.noteId!); contextMenu = null">复制</button>
        <button @click="bringSelectionToFront(contextMenu!.noteId!); contextMenu = null">置顶</button>
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
        <div class="context-section">
          <span>标签</span>
          <button v-for="tag in tagStore.tags" :key="tag.id" @click="toggleTagForSelection(contextMenu!.noteId!, tag.id); contextMenu = null">
            {{ tag.name }}
          </button>
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
