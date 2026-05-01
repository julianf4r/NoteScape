<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, watch } from "vue";
import { X } from "lucide-vue-next";
import CanvasToolbar from "./CanvasToolbar.vue";
import DrawingLayer from "./DrawingLayer.vue";
import MiniMap from "./MiniMap.vue";
import StickyNote from "./StickyNote.vue";
import { useAppStore } from "../stores/appStore";
import { useCanvasStore } from "../stores/canvasStore";
import { useNoteStore } from "../stores/noteStore";
import { useSettingsStore } from "../stores/settingsStore";
import { useTagStore } from "../stores/tagStore";
import { useFeedbackStore } from "../stores/feedbackStore";
import { useDrawingStore } from "../stores/drawingStore";
import type { DrawingItem, DrawingPoint, NoteColor, StickyNote as StickyNoteType, ViewportState } from "../types";
import { noteColorList, noteColors } from "../utils/colors";
import { clamp, screenToWorld } from "../utils/geometry";

const appStore = useAppStore();
const canvasStore = useCanvasStore();
const noteStore = useNoteStore();
const tagStore = useTagStore();
const settingsStore = useSettingsStore();
const feedback = useFeedbackStore();
const drawingStore = useDrawingStore();

const board = ref<HTMLElement>();
const viewport = reactive<ViewportState>({ offsetX: 0, offsetY: 0, scale: 1 });
const boardSize = reactive({ width: 0, height: 0 });
const handActive = ref(false);
const spaceDown = ref(false);
const panStart = ref<{ x: number; y: number; offsetX: number; offsetY: number }>();
const contextMenu = ref<{ x: number; y: number; noteId?: string; drawingId?: string; linkHref?: string; codeText?: string } | null>(null);
const contextWorld = ref<{ x: number; y: number }>({ x: 0, y: 0 });
const highlightedNoteId = ref("");
const boxSelect = ref<{ startX: number; startY: number; currentX: number; currentY: number } | null>(null);
const groupDrag = ref<{ startX: number; startY: number; before: StickyNoteType[] } | null>(null);
const mixedDrag = ref<{ startX: number; startY: number; beforeNotes: StickyNoteType[]; beforeDrawings: DrawingItem[] } | null>(null);
const activeDrawingId = ref("");
const clearSelectionAfterTinyDrawing = ref(false);
const drawingDrag = ref<{ id: string; startX: number; startY: number; before: DrawingItem } | null>(null);
let highlightTimer: number | undefined;

const visibleNotes = computed(() =>
  noteStore.notesForCanvas(canvasStore.currentCanvasId, tagStore.activeTagId, canvasStore.searchQuery),
);

const currentCanvasNotes = computed(() => noteStore.notesForCanvas(canvasStore.currentCanvasId));
const currentCanvasDrawings = computed(() => drawingStore.drawingsForCanvas(canvasStore.currentCanvasId));
const filterActive = computed(() => Boolean(tagStore.activeTagId || canvasStore.searchQuery.trim()));
const activeTag = computed(() => tagStore.activeTag);
const searchText = computed(() => canvasStore.searchQuery.trim());

const canvasClass = computed(() => ({
  "hide-grid": !settingsStore.settings.showGrid,
  "drawing-ready": drawingStore.tool !== "select",
  "pan-ready": handActive.value || spaceDown.value,
  panning: Boolean(panStart.value),
}));

function createNoteAt(clientX: number, clientY: number) {
  if (drawingStore.tool !== "select") return;
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
  return !target.closest(".sticky-note, .toolbar, .minimap, .menu-popover, .empty-board, .filter-status");
}

function isCanvasControlTarget(target: EventTarget | null) {
  return target instanceof HTMLElement && Boolean(target.closest(".toolbar, .minimap, .menu-popover, .filter-status"));
}

function clearFilters() {
  tagStore.activeTagId = "";
  canvasStore.searchQuery = "";
}

function onBoardMouseDown(event: MouseEvent) {
  contextMenu.value = null;
  const blankTarget = isCanvasBlankTarget(event.target);
  const shouldPan = event.button === 1 || handActive.value || spaceDown.value;
  if (drawingStore.tool !== "select") {
    if (!isCanvasControlTarget(event.target) && shouldPan) {
      startPan(event);
      return;
    }
    if (!isCanvasControlTarget(event.target) && blankTarget && event.button === 0 && (event.shiftKey || event.ctrlKey)) {
      startBoxSelect(event);
      return;
    }
    if (!isCanvasControlTarget(event.target) && blankTarget) {
      clearSelectionAfterTinyDrawing.value = Boolean(noteStore.selectedIds.length || drawingStore.selectedIds.length);
      startDrawing(event);
    }
    noteStore.clearSelection();
    return;
  }
  if (noteStore.editingId && blankTarget) noteStore.stopEditing();
  if (blankTarget) {
    if (event.shiftKey || event.ctrlKey) {
      startBoxSelect(event);
    } else {
      noteStore.clearSelection();
      drawingStore.clearSelection();
    }
  }
  if (!isCanvasControlTarget(event.target) && (blankTarget || shouldPan)) startPan(event);
}

function onBoardDoubleClick(event: MouseEvent) {
  if (handActive.value || drawingStore.tool !== "select") return;
  if (isCanvasBlankTarget(event.target)) createNoteAt(event.clientX, event.clientY);
}

function drawingPointFromEvent(event: MouseEvent): DrawingPoint | null {
  const rect = board.value?.getBoundingClientRect();
  if (!rect) return null;
  return screenToWorld(event.clientX, event.clientY, viewport, rect);
}

function startDrawing(event: MouseEvent) {
  if (!canvasStore.currentCanvasId || event.button !== 0) return;
  const point = drawingPointFromEvent(event);
  if (!point) return;
  event.preventDefault();
  const base = drawingStore.tool === "pen"
    ? { points: [point] }
    : drawingStore.tool === "arrow" || drawingStore.tool === "line"
      ? { start: point, end: point }
      : { start: point, x: point.x, y: point.y, width: 0, height: 0 };
  const drawing = drawingStore.createDrawing(canvasStore.currentCanvasId, base);
  activeDrawingId.value = drawing.id;
  window.addEventListener("mousemove", updateDrawing);
  window.addEventListener("mouseup", finishDrawing, { once: true });
}

function updateDrawing(event: MouseEvent) {
  if (!activeDrawingId.value) return;
  const point = drawingPointFromEvent(event);
  const drawing = drawingStore.drawings.find((item) => item.id === activeDrawingId.value);
  if (!point || !drawing) return;
  if (drawing.type === "pen") {
    const last = drawing.points?.[drawing.points.length - 1];
    const minDistance = Math.max(2, drawing.strokeWidth * 0.22);
    if (!last || Math.hypot(point.x - last.x, point.y - last.y) >= minDistance) drawingStore.appendPoint(drawing.id, point);
    return;
  }
  if (drawing.type === "arrow" || drawing.type === "line") {
    drawingStore.updateDrawing(drawing.id, { end: point });
    return;
  }
  const start = drawing.start ?? point;
  drawingStore.updateDrawing(drawing.id, {
    x: Math.min(start.x, point.x),
    y: Math.min(start.y, point.y),
    width: Math.abs(point.x - start.x),
    height: Math.abs(point.y - start.y),
  });
}

function finishDrawing() {
  window.removeEventListener("mousemove", updateDrawing);
  const drawing = drawingStore.drawings.find((item) => item.id === activeDrawingId.value);
  if (drawing && isMeaningfulDrawing(drawing)) {
    drawingStore.finishDrawing(drawing.id);
  } else if (drawing) {
    drawingStore.deleteDrawing(drawing.id, false);
    if (clearSelectionAfterTinyDrawing.value) clearObjectSelection();
  }
  activeDrawingId.value = "";
  clearSelectionAfterTinyDrawing.value = false;
}

function isMeaningfulDrawing(drawing: DrawingItem) {
  if (drawing.type === "pen") return (drawing.points?.length ?? 0) > 1;
  if (drawing.type === "arrow" || drawing.type === "line") return Boolean(drawing.start && drawing.end && Math.hypot(drawing.end.x - drawing.start.x, drawing.end.y - drawing.start.y) > 4);
  return Math.max(drawing.width ?? 0, drawing.height ?? 0) > 4;
}

function setHandActive(value: boolean) {
  handActive.value = value;
}

function openCanvasMenu(event: MouseEvent) {
  if (board.value) {
    const rect = board.value.getBoundingClientRect();
    contextWorld.value = screenToWorld(event.clientX, event.clientY, viewport, rect);
  }
  contextMenu.value = { x: event.clientX, y: event.clientY };
}

function openNoteMenu(event: MouseEvent, id: string, payload?: { linkHref?: string; codeText?: string }) {
  drawingStore.clearSelection();
  noteStore.select(id);
  if (board.value) {
    const rect = board.value.getBoundingClientRect();
    contextWorld.value = screenToWorld(event.clientX, event.clientY, viewport, rect);
  }
  contextMenu.value = { x: event.clientX, y: event.clientY, noteId: id, ...payload };
}

function openDrawingMenu(event: MouseEvent, id: string) {
  noteStore.clearSelection();
  if (!drawingStore.selectedIds.includes(id)) drawingStore.select(id);
  if (board.value) {
    const rect = board.value.getBoundingClientRect();
    contextWorld.value = screenToWorld(event.clientX, event.clientY, viewport, rect);
  }
  contextMenu.value = { x: event.clientX, y: event.clientY, drawingId: id };
}

function updateNote(note: StickyNoteType, patch: Partial<StickyNoteType> & { __before?: StickyNoteType }, track = true) {
  const { __before, ...cleanPatch } = patch;
  if (__before) noteStore.commitNoteChange(__before, cleanPatch);
  else noteStore.updateNote(note.id, cleanPatch, track);
}

function onNotePointerDown(event: MouseEvent, note: StickyNoteType) {
  if (drawingStore.tool !== "select") return;
  if (handActive.value || spaceDown.value || event.button === 1) return;
  const additive = event.shiftKey || event.ctrlKey;
  const alreadySelected = noteStore.selectedIds.includes(note.id);
  if (!additive && !alreadySelected) drawingStore.clearSelection();
  if (!alreadySelected || additive) noteStore.select(note.id, additive);
  if (selectedObjectCount() > 1 && noteStore.selectedIds.includes(note.id)) {
    startMixedDrag(event);
    return;
  }
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

function selectedObjectCount() {
  return noteStore.selectedIds.length + drawingStore.selectedIds.length;
}

function startMixedDrag(event: MouseEvent) {
  mixedDrag.value = {
    startX: event.clientX,
    startY: event.clientY,
    beforeNotes: noteStore.notes.filter((item) => noteStore.selectedIds.includes(item.id)).map((item) => ({ ...item, tags: [...item.tags] })),
    beforeDrawings: drawingStore.drawings.filter((item) => drawingStore.selectedIds.includes(item.id)).map(cloneDrawing),
  };
  window.addEventListener("mousemove", dragMixed);
  window.addEventListener("mouseup", endMixedDrag, { once: true });
}

function dragMixed(event: MouseEvent) {
  if (!mixedDrag.value) return;
  const deltaX = (event.clientX - mixedDrag.value.startX) / viewport.scale;
  const deltaY = (event.clientY - mixedDrag.value.startY) / viewport.scale;
  noteStore.moveSelectedBy(deltaX, deltaY, mixedDrag.value.beforeNotes);
  drawingStore.moveSelectedBy(deltaX, deltaY, mixedDrag.value.beforeDrawings);
}

function endMixedDrag() {
  window.removeEventListener("mousemove", dragMixed);
  if (mixedDrag.value) {
    noteStore.commitSelectedMove(mixedDrag.value.beforeNotes);
    drawingStore.commitSelectedMove(mixedDrag.value.beforeDrawings);
  }
  mixedDrag.value = null;
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
  drawingStore.clearSelection();
  if (noteStore.selectedIds.length > 1 && noteStore.selectedIds.includes(noteId)) noteStore.deleteSelected();
  else noteStore.deleteNote(noteId);
}

function deleteSelectedDrawing() {
  if (drawingStore.selectedIds.length) drawingStore.deleteSelected();
}

function deleteSelectedObjects() {
  if (drawingStore.selectedIds.length) drawingStore.deleteSelected();
  if (noteStore.selectedIds.length) noteStore.deleteSelected();
}

function clearObjectSelection() {
  noteStore.clearSelection();
  drawingStore.clearSelection();
}

function startDrawingDrag(event: MouseEvent, drawingId: string) {
  if (event.button !== 0 || handActive.value || spaceDown.value) return;
  const drawing = drawingStore.drawings.find((item) => item.id === drawingId);
  if (!drawing) return;
  event.preventDefault();
  event.stopPropagation();
  const alreadySelected = drawingStore.selectedIds.includes(drawingId);
  const additive = event.shiftKey || event.ctrlKey;
  if (!alreadySelected) {
    if (!additive) noteStore.clearSelection();
    drawingStore.select(drawingId, additive);
  }
  if (selectedObjectCount() > 1 && drawingStore.selectedIds.includes(drawingId)) {
    startMixedDrag(event);
    return;
  }
  drawingDrag.value = {
    id: drawingId,
    startX: event.clientX,
    startY: event.clientY,
    before: cloneDrawing(drawing),
  };
  window.addEventListener("mousemove", dragDrawing);
  window.addEventListener("mouseup", endDrawingDrag, { once: true });
}

function dragDrawing(event: MouseEvent) {
  if (!drawingDrag.value) return;
  drawingStore.moveDrawingLive(
    drawingDrag.value.id,
    drawingDrag.value.before,
    (event.clientX - drawingDrag.value.startX) / viewport.scale,
    (event.clientY - drawingDrag.value.startY) / viewport.scale,
  );
}

function endDrawingDrag() {
  window.removeEventListener("mousemove", dragDrawing);
  if (drawingDrag.value) drawingStore.commitDrawingMove(drawingDrag.value.before);
  drawingDrag.value = null;
}

function cloneDrawing(drawing: DrawingItem): DrawingItem {
  return {
    ...drawing,
    points: drawing.points?.map((point) => ({ ...point })),
    start: drawing.start ? { ...drawing.start } : undefined,
    end: drawing.end ? { ...drawing.end } : undefined,
  };
}

function undo() {
  const preferDrawing = Boolean(drawingStore.selectedIds.length || drawingStore.tool !== "select");
  if (preferDrawing) {
    if (!drawingStore.undo() && noteStore.history.length) noteStore.undo();
    return;
  }
  if (noteStore.history.length) noteStore.undo();
  else drawingStore.undo();
}

function redo() {
  const preferDrawing = Boolean(drawingStore.selectedIds.length || drawingStore.tool !== "select" || (drawingStore.future.length && !noteStore.future.length));
  if (preferDrawing) {
    if (!drawingStore.redo() && noteStore.future.length) noteStore.redo();
    return;
  }
  if (noteStore.future.length) noteStore.redo();
  else drawingStore.redo();
}

function duplicateSelection(noteId: string) {
  if (noteStore.selectedIds.length > 1 && noteStore.selectedIds.includes(noteId)) noteStore.duplicateSelected();
  else noteStore.duplicateNote(noteId);
}

function copySelectedObjects() {
  if (noteStore.selectedIds.length) noteStore.copySelected();
  if (drawingStore.selectedIds.length) drawingStore.copySelected();
}

function duplicateSelectedObjects() {
  if (noteStore.selectedIds.length) {
    if (noteStore.selectedIds.length > 1) noteStore.duplicateSelected();
    else noteStore.duplicateNote(noteStore.selectedIds[0]);
  }
  if (drawingStore.selectedIds.length) drawingStore.duplicateSelected();
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

async function copyLink(href: string) {
  await navigator.clipboard.writeText(href);
  feedback.notify("链接已复制", "success");
}

async function copyCode(code: string) {
  await navigator.clipboard.writeText(code);
  feedback.notify("代码已复制", "success");
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
  drawingStore.pasteClipboard(canvasStore.currentCanvasId, contextWorld.value.x, contextWorld.value.y);
  contextMenu.value = null;
}

function changeDrawingColorForContext(color: string) {
  drawingStore.color = color;
  if (drawingStore.selectedIds.length) drawingStore.updateSelected({ color });
  contextMenu.value = null;
}

function setDrawingColor(color: string) {
  drawingStore.color = color;
  if (drawingStore.selectedIds.length) drawingStore.updateSelected({ color });
}

function setDrawingStrokeWidth(width: number) {
  const strokeWidth = Math.min(16, Math.max(1, width || 1));
  drawingStore.strokeWidth = strokeWidth;
  if (drawingStore.selectedIds.length) drawingStore.updateSelected({ strokeWidth });
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

function drawingBounds(drawing: DrawingItem) {
  if (drawing.type === "pen") {
    const points = drawing.points ?? [];
    if (!points.length) return null;
    const xs = points.map((point) => point.x);
    const ys = points.map((point) => point.y);
    return {
      minX: Math.min(...xs),
      minY: Math.min(...ys),
      maxX: Math.max(...xs),
      maxY: Math.max(...ys),
    };
  }
  if (drawing.type === "arrow" || drawing.type === "line") {
    if (!drawing.start || !drawing.end) return null;
    return {
      minX: Math.min(drawing.start.x, drawing.end.x),
      minY: Math.min(drawing.start.y, drawing.end.y),
      maxX: Math.max(drawing.start.x, drawing.end.x),
      maxY: Math.max(drawing.start.y, drawing.end.y),
    };
  }
  const x = drawing.x ?? 0;
  const y = drawing.y ?? 0;
  const width = drawing.width ?? 0;
  const height = drawing.height ?? 0;
  return {
    minX: x,
    minY: y,
    maxX: x + width,
    maxY: y + height,
  };
}

function finishBoxSelect() {
  window.removeEventListener("mousemove", updateBoxSelect);
  if (!boxSelect.value) return;
  const left = Math.min(boxSelect.value.startX, boxSelect.value.currentX);
  const top = Math.min(boxSelect.value.startY, boxSelect.value.currentY);
  const right = Math.max(boxSelect.value.startX, boxSelect.value.currentX);
  const bottom = Math.max(boxSelect.value.startY, boxSelect.value.currentY);
  const selectedNotes = visibleNotes.value.filter((note) => {
    const noteLeft = note.x * viewport.scale + viewport.offsetX;
    const noteTop = note.y * viewport.scale + viewport.offsetY;
    const noteRight = noteLeft + note.width * viewport.scale;
    const noteBottom = noteTop + note.height * viewport.scale;
    return noteRight >= left && noteLeft <= right && noteBottom >= top && noteTop <= bottom;
  });
  const selectedDrawings = currentCanvasDrawings.value.filter((drawing) => {
    const bounds = drawingBounds(drawing);
    if (!bounds) return false;
    const drawingLeft = bounds.minX * viewport.scale + viewport.offsetX;
    const drawingTop = bounds.minY * viewport.scale + viewport.offsetY;
    const drawingRight = bounds.maxX * viewport.scale + viewport.offsetX;
    const drawingBottom = bounds.maxY * viewport.scale + viewport.offsetY;
    return drawingRight >= left && drawingLeft <= right && drawingBottom >= top && drawingTop <= bottom;
  });
  noteStore.setSelection(selectedNotes.map((note) => note.id));
  drawingStore.setSelection(selectedDrawings.map((drawing) => drawing.id));
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
  if (event.ctrlKey && event.key.toLowerCase() === "z") {
    event.preventDefault();
    undo();
    return;
  }
  if (event.ctrlKey && event.key.toLowerCase() === "y") {
    event.preventDefault();
    redo();
    return;
  }
  if (event.ctrlKey && event.key.toLowerCase() === "c" && !noteStore.editingId) {
    if (noteStore.selectedIds.length || drawingStore.selectedIds.length) {
      event.preventDefault();
      copySelectedObjects();
    }
    return;
  }
  if (event.ctrlKey && event.key.toLowerCase() === "d" && !noteStore.editingId) {
    if (noteStore.selectedIds.length || drawingStore.selectedIds.length) {
      event.preventDefault();
      duplicateSelectedObjects();
    }
    return;
  }
  if ((event.key === "Delete" || event.key === "Backspace") && (drawingStore.selectedIds.length || noteStore.selectedIds.length) && !noteStore.editingId) {
    event.preventDefault();
    deleteSelectedObjects();
    return;
  }
  if (event.key === "Escape") {
    clearObjectSelection();
    return;
  }
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
    drawingStore.pasteClipboard(canvasStore.currentCanvasId, point.x, point.y);
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
  window.removeEventListener("mousemove", updateDrawing);
  window.removeEventListener("mousemove", dragDrawing);
  window.removeEventListener("mousemove", dragMixed);
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
    <div v-if="appStore.loaded && !appStore.databaseReady" class="empty-board database-error">
      数据库未加载<br />
      <span>{{ appStore.loadError }}</span>
      <button @click.stop="settingsStore.togglePanel()">处理数据库</button>
    </div>
    <div v-else-if="!canvasStore.currentCanvasId" class="empty-board">还没有画布<br />点击“新建”开始整理你的想法</div>
    <div v-else-if="!visibleNotes.length && filterActive" class="empty-board filtered-empty">
      当前筛选下没有便签<br />
      <button @click.stop="clearFilters">清除筛选</button>
    </div>
    <div v-else-if="!visibleNotes.length && !currentCanvasDrawings.length" class="empty-board">双击画布空白处创建第一张便签</div>

    <div v-if="appStore.databaseReady && filterActive" class="filter-status" @mousedown.stop @dblclick.stop>
      <span v-if="activeTag" class="tag-filter">
        <i :style="{ backgroundColor: activeTag.color }"></i>
        {{ activeTag.name }}
      </span>
      <span v-if="searchText" class="search-filter">搜索：{{ searchText }}</span>
      <b>{{ visibleNotes.length }} / {{ currentCanvasNotes.length }}</b>
      <button title="清除筛选" @click.stop="clearFilters"><X :size="14" /></button>
    </div>

    <div v-if="appStore.databaseReady" class="canvas-content" :style="{ transform: `translate(${viewport.offsetX}px, ${viewport.offsetY}px) scale(${viewport.scale})` }">
      <DrawingLayer
        :canvas-id="canvasStore.currentCanvasId"
        :drawings="currentCanvasDrawings"
        :scale="viewport.scale"
        @drag-drawing="startDrawingDrag"
        @context-drawing="openDrawingMenu"
      />
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
        :pan-mode="handActive || spaceDown || drawingStore.tool !== 'select'"
        @select="onNotePointerDown($event, note)"
        @edit="!handActive && drawingStore.tool === 'select' && (noteStore.editingId = note.id)"
        @update="(patch, track) => updateSelection(note, patch, track)"
        @live="(patch) => noteStore.patchNoteLive(note.id, patch)"
        @delete="deleteSelection(note.id)"
        @duplicate="duplicateSelection(note.id)"
        @copy-text="copyNoteText(note.id)"
        @front="bringSelectionToFront(note.id)"
        @context="(event, payload) => openNoteMenu(event, note.id, payload)"
        @editing-done="noteStore.stopEditing()"
        @toggle-tag="(tagId) => toggleTagForSelection(note.id, tagId)"
      />
    </div>

    <div v-if="boxSelect" class="selection-box" :style="boxSelectStyle"></div>

    <CanvasToolbar
      :scale="viewport.scale"
      :hand-active="handActive"
      :drawing-tool="drawingStore.tool"
      :drawing-color="drawingStore.color"
      :drawing-stroke-width="drawingStore.strokeWidth"
      :drawing-selected="Boolean(drawingStore.selectedIds.length)"
      @undo="undo"
      @redo="redo"
      @zoom-in="zoomBy(0.1)"
      @zoom-out="zoomBy(-0.1)"
      @set-zoom="setZoom"
      @reset-zoom="resetZoom"
      @set-hand-active="setHandActive"
      @set-drawing-tool="drawingStore.setTool"
      @set-drawing-color="setDrawingColor"
      @set-drawing-stroke-width="setDrawingStrokeWidth"
      @delete-drawing="deleteSelectedDrawing"
      @settings="settingsStore.togglePanel()"
    />

    <MiniMap
      v-if="appStore.databaseReady"
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
        <button v-if="contextMenu.linkHref" @click="copyLink(contextMenu!.linkHref!); contextMenu = null">复制链接</button>
        <button v-if="contextMenu.codeText" @click="copyCode(contextMenu!.codeText!); contextMenu = null">复制代码</button>
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
      <template v-else-if="contextMenu.drawingId">
        <button @click="copySelectedObjects(); contextMenu = null">复制</button>
        <button @click="duplicateSelectedObjects(); contextMenu = null">复制一份</button>
        <button @click="deleteSelectedDrawing(); contextMenu = null">删除</button>
        <div class="context-section">
          <span>颜色</span>
          <div class="context-swatches">
            <button
              v-for="color in ['#ff0000', '#1f2937', '#2563eb', '#16a34a', '#f59e0b', '#9333ea', '#ec4899', '#64748b']"
              :key="color"
              :style="{ backgroundColor: color }"
              @click="changeDrawingColorForContext(color)"
            ></button>
          </div>
        </div>
      </template>
      <template v-else>
        <button @click="createNoteAt(contextMenu!.x, contextMenu!.y); contextMenu = null">新建便签</button>
        <button :disabled="!noteStore.clipboard.length && !drawingStore.clipboard.length" @click="pasteAtContext">粘贴</button>
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

.canvas-board.drawing-ready {
  cursor: var(--cursor-crosshair);
}

.canvas-board.pan-ready {
  cursor: var(--cursor-grab);
}

.canvas-board.panning {
  cursor: var(--cursor-grabbing);
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

.empty-board.database-error {
  width: min(520px, calc(100% - 48px));
  color: #991b1b;
  background: rgba(255, 255, 255, 0.92);
  border-color: #fecaca;
}

.empty-board.database-error span {
  display: block;
  margin-top: 4px;
  overflow-wrap: anywhere;
  color: #6b7280;
  font-size: 13px;
}

.empty-board.database-error button {
  height: 34px;
  margin-top: 12px;
  padding: 0 12px;
  color: #374151;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 7px;
}

.empty-board.filtered-empty button {
  height: 34px;
  margin-top: 10px;
  padding: 0 12px;
  color: #374151;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 7px;
}

.filter-status {
  position: absolute;
  top: 16px;
  left: 16px;
  z-index: 32;
  min-height: 38px;
  max-width: min(520px, calc(100% - 340px));
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px 6px 10px;
  color: #374151;
  background: rgba(255, 255, 255, 0.96);
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  box-shadow: var(--shadow-sm);
  font-size: 13px;
}

.filter-status span {
  min-width: 0;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.filter-status .tag-filter {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-weight: 700;
}

.filter-status .tag-filter i {
  width: 10px;
  height: 10px;
  flex: 0 0 auto;
  border-radius: 50%;
}

.filter-status .search-filter {
  color: #4b5563;
}

.filter-status b {
  flex: 0 0 auto;
  color: #6b7280;
  font-size: 12px;
}

.filter-status button {
  width: 26px;
  height: 26px;
  flex: 0 0 auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: #6b7280;
  background: #f8fafc;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
}

.filter-status button:hover {
  color: #111827;
  background: #eef2f7;
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
  cursor: var(--cursor-default);
  opacity: 0.45;
}
</style>
