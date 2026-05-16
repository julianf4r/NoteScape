<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { X } from "lucide-vue-next";
import CanvasImage from "./CanvasImage.vue";
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
import { useImageStore } from "../stores/imageStore";
import type { CanvasImage as CanvasImageType, DrawingItem, DrawingPoint, NoteColor, StickyNote as StickyNoteType, ViewportState } from "../types";
import { noteColorList, noteColors } from "../utils/colors";
import { clamp, screenToWorld } from "../utils/geometry";
import { contentJsonToMarkdown } from "../utils/markdown";
import { imageFileUrl, importImageBytes, importImageFile } from "../utils/storage";

const appStore = useAppStore();
const canvasStore = useCanvasStore();
const noteStore = useNoteStore();
const tagStore = useTagStore();
const settingsStore = useSettingsStore();
const feedback = useFeedbackStore();
const drawingStore = useDrawingStore();
const imageStore = useImageStore();

const board = ref<HTMLElement>();
const viewport = reactive<ViewportState>({ offsetX: 0, offsetY: 0, scale: 1 });
const boardSize = reactive({ width: 0, height: 0 });
const handActive = ref(false);
const spaceDown = ref(false);
const panStart = ref<{ x: number; y: number; offsetX: number; offsetY: number }>();
const contextMenu = ref<{ x: number; y: number; noteId?: string; drawingId?: string; imageId?: string; linkHref?: string; codeText?: string } | null>(null);
const contextWorld = ref<{ x: number; y: number }>({ x: 0, y: 0 });
const highlightedNoteId = ref("");
const boxSelect = ref<{ startX: number; startY: number; currentX: number; currentY: number } | null>(null);
const groupDrag = ref<{ startX: number; startY: number; before: StickyNoteType[] } | null>(null);
const mixedDrag = ref<{ startX: number; startY: number; beforeNotes: StickyNoteType[]; beforeDrawings: DrawingItem[]; beforeImages: CanvasImageType[] } | null>(null);
const activeDrawingId = ref("");
const clearSelectionAfterTinyDrawing = ref(false);
const drawingDrag = ref<{ id: string; startX: number; startY: number; before: DrawingItem } | null>(null);
const drawingEdit = ref<{ id: string; handle: "start" | "end" | "resize"; before: DrawingItem } | null>(null);
let highlightTimer: number | undefined;
let unlistenImageDrop: UnlistenFn | undefined;
const minBoxSelectDistance = 4;
const imageExtensions = new Set(["png", "jpg", "jpeg", "webp", "gif", "bmp", "svg"]);
const visibleNotes = computed(() =>
  noteStore.notesForCanvas(canvasStore.currentCanvasId, tagStore.activeTagId, canvasStore.searchQuery),
);

const currentCanvasNotes = computed(() => noteStore.notesForCanvas(canvasStore.currentCanvasId));
const currentCanvasDrawings = computed(() => drawingStore.drawingsForCanvas(canvasStore.currentCanvasId));
const currentCanvasImages = computed(() => imageStore.imagesForCanvas(canvasStore.currentCanvasId));
const filterActive = computed(() => Boolean(tagStore.activeTagId || canvasStore.searchQuery.trim()));
const activeTag = computed(() => tagStore.activeTag);
const searchText = computed(() => canvasStore.searchQuery.trim());
type CanvasObjectType = "note" | "drawing" | "image";
type CanvasHistoryBatch = { notes: number; drawings: number; images: number };
type CanvasObjectRef =
  | { type: "note"; item: StickyNoteType }
  | { type: "drawing"; item: DrawingItem }
  | { type: "image"; item: CanvasImageType };
const canvasHistory = ref<CanvasHistoryBatch[]>([]);
const canvasFuture = ref<CanvasHistoryBatch[]>([]);
const pinnedZOffset = 100000;

const globalMaxZ = computed(() =>
  Math.max(
    0,
    ...noteStore.notes.map((note) => objectLayerZ({ type: "note", item: note })),
    ...drawingStore.drawings.map((drawing) => drawing.zIndex),
    ...imageStore.images.map((image) => objectLayerZ({ type: "image", item: image })),
  ),
);

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
  captureCanvasHistory(() => {
    noteStore.createNote(
      canvasStore.currentCanvasId,
      point.x - 110,
      point.y - 90,
      settingsStore.settings.defaultFontSize,
      settingsStore.settings.randomRotation,
      "",
      globalMaxZ.value + 1,
    );
  });
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
  return !target.closest(".sticky-note, .canvas-image, .toolbar, .minimap, .menu-popover, .empty-board, .filter-status");
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
      clearSelectionAfterTinyDrawing.value = Boolean(noteStore.selectedIds.length || drawingStore.selectedIds.length || imageStore.selectedIds.length);
      startDrawing(event);
    }
    noteStore.clearSelection();
    imageStore.clearSelection();
    return;
  }
  if (noteStore.editingId && blankTarget) noteStore.stopEditing();
  if (!isCanvasControlTarget(event.target) && shouldPan) {
    startPan(event);
    return;
  }
  if (blankTarget) {
    if (event.button === 0) {
      noteStore.clearSelection();
      drawingStore.clearSelection();
      imageStore.clearSelection();
      startBoxSelect(event);
      return;
    }
    noteStore.clearSelection();
    drawingStore.clearSelection();
    imageStore.clearSelection();
  }
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
  const drawing = drawingStore.createDrawing(canvasStore.currentCanvasId, { ...base, zIndex: globalMaxZ.value + 1 });
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
    captureCanvasHistory(() => drawingStore.finishDrawing(drawing.id));
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
  ensureContextSelection("note", id);
  if (board.value) {
    const rect = board.value.getBoundingClientRect();
    contextWorld.value = screenToWorld(event.clientX, event.clientY, viewport, rect);
  }
  contextMenu.value = { x: event.clientX, y: event.clientY, noteId: id, ...payload };
}

function openDrawingMenu(event: MouseEvent, id: string) {
  ensureContextSelection("drawing", id);
  if (board.value) {
    const rect = board.value.getBoundingClientRect();
    contextWorld.value = screenToWorld(event.clientX, event.clientY, viewport, rect);
  }
  contextMenu.value = { x: event.clientX, y: event.clientY, drawingId: id };
}

function openImageMenu(event: MouseEvent, id: string) {
  ensureContextSelection("image", id);
  contextMenu.value = { x: event.clientX, y: event.clientY, imageId: id };
}

function isObjectSelected(type: CanvasObjectType, id: string) {
  if (type === "note") return noteStore.selectedIds.includes(id);
  if (type === "drawing") return drawingStore.selectedIds.includes(id);
  return imageStore.selectedIds.includes(id);
}

function selectSingleObject(type: CanvasObjectType, id: string) {
  clearObjectSelection();
  if (type === "note") noteStore.select(id);
  else if (type === "drawing") drawingStore.select(id);
  else imageStore.select(id);
}

function ensureContextSelection(type: CanvasObjectType, id: string) {
  if (isObjectSelected(type, id)) return;
  selectSingleObject(type, id);
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
  if (!additive && !alreadySelected) {
    drawingStore.clearSelection();
    imageStore.clearSelection();
  }
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
  return noteStore.selectedIds.length + drawingStore.selectedIds.length + imageStore.selectedIds.length;
}

function selectedObjectRefs() {
  const refs: CanvasObjectRef[] = [
    ...noteStore.notes
      .filter((note) => noteStore.selectedIds.includes(note.id))
      .map((item) => ({ type: "note" as const, item })),
    ...drawingStore.drawings
      .filter((drawing) => drawingStore.selectedIds.includes(drawing.id))
      .map((item) => ({ type: "drawing" as const, item })),
    ...imageStore.images
      .filter((image) => imageStore.selectedIds.includes(image.id))
      .map((item) => ({ type: "image" as const, item })),
  ];
  return refs.sort((a, b) => objectLayerZ(a) - objectLayerZ(b));
}

function objectLayerZ(ref: CanvasObjectRef) {
  if (ref.type === "note") return (ref.item.pinned ? pinnedZOffset : 0) + ref.item.zIndex;
  if (ref.type === "image") return (ref.item.pinned ? pinnedZOffset : 0) + ref.item.zIndex;
  return ref.item.zIndex;
}

function drawingLayerStyle(drawing: DrawingItem) {
  return { zIndex: drawing.zIndex };
}

function historyCounts(): CanvasHistoryBatch {
  return {
    notes: noteStore.history.length,
    drawings: drawingStore.history.length,
    images: imageStore.history.length,
  };
}

function pushCanvasHistory(batch: CanvasHistoryBatch) {
  if (!batch.notes && !batch.drawings && !batch.images) return;
  canvasHistory.value.push(batch);
  if (canvasHistory.value.length > 50) canvasHistory.value.shift();
  canvasFuture.value = [];
}

function recordCanvasHistoryChange(before: CanvasHistoryBatch) {
  const after = historyCounts();
  pushCanvasHistory({
    notes: Math.max(0, after.notes - before.notes),
    drawings: Math.max(0, after.drawings - before.drawings),
    images: Math.max(0, after.images - before.images),
  });
}

function captureCanvasHistory<T>(operation: () => T): T {
  const before = historyCounts();
  const result = operation();
  recordCanvasHistoryChange(before);
  return result;
}

function undoStoreEntries(undoFn: () => boolean | void, count: number) {
  for (let index = 0; index < count; index += 1) undoFn();
}

function redoStoreEntries(redoFn: () => boolean | void, count: number) {
  for (let index = 0; index < count; index += 1) redoFn();
}

function startMixedDrag(event: MouseEvent) {
  mixedDrag.value = {
    startX: event.clientX,
    startY: event.clientY,
    beforeNotes: noteStore.notes.filter((item) => noteStore.selectedIds.includes(item.id)).map((item) => ({ ...item, tags: [...item.tags] })),
    beforeDrawings: drawingStore.drawings.filter((item) => drawingStore.selectedIds.includes(item.id)).map(cloneDrawing),
    beforeImages: imageStore.images.filter((item) => imageStore.selectedIds.includes(item.id)).map(cloneImage),
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
  imageStore.moveSelectedBy(deltaX, deltaY, mixedDrag.value.beforeImages);
}

function endMixedDrag() {
  window.removeEventListener("mousemove", dragMixed);
  if (mixedDrag.value) {
    captureCanvasHistory(() => {
      noteStore.commitSelectedMove(mixedDrag.value!.beforeNotes);
      drawingStore.commitSelectedMove(mixedDrag.value!.beforeDrawings);
      imageStore.commitSelectedMove(mixedDrag.value!.beforeImages);
    });
  }
  mixedDrag.value = null;
}

function dragGroup(event: MouseEvent) {
  if (!groupDrag.value) return;
  noteStore.moveSelectedBy((event.clientX - groupDrag.value.startX) / viewport.scale, (event.clientY - groupDrag.value.startY) / viewport.scale, groupDrag.value.before);
}

function endGroupDrag() {
  window.removeEventListener("mousemove", dragGroup);
  if (groupDrag.value) captureCanvasHistory(() => noteStore.commitSelectedMove(groupDrag.value!.before));
  groupDrag.value = null;
}

function deleteObjectForContext(type: CanvasObjectType, id: string) {
  if (isObjectSelected(type, id)) {
    deleteSelectedObjects();
    return;
  }
  captureCanvasHistory(() => {
    if (type === "note") noteStore.deleteNote(id);
    else if (type === "drawing") drawingStore.deleteDrawing(id);
    else imageStore.deleteImage(id);
  });
}

function deleteSelectedDrawing() {
  if (drawingStore.selectedIds.length) captureCanvasHistory(() => drawingStore.deleteSelected());
}

function deleteSelectedObjects() {
  captureCanvasHistory(() => {
    if (drawingStore.selectedIds.length) drawingStore.deleteSelected();
    if (noteStore.selectedIds.length) noteStore.deleteSelected();
    if (imageStore.selectedIds.length) imageStore.deleteSelected();
  });
}

function clearObjectSelection() {
  noteStore.clearSelection();
  drawingStore.clearSelection();
  imageStore.clearSelection();
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
    if (!additive) {
      noteStore.clearSelection();
      imageStore.clearSelection();
    }
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

function startDrawingEdit(event: MouseEvent, drawingId: string, handle: "start" | "end" | "resize") {
  if (event.button !== 0 || handActive.value || spaceDown.value) return;
  const drawing = drawingStore.drawings.find((item) => item.id === drawingId);
  if (!drawing) return;
  event.preventDefault();
  event.stopPropagation();
  drawingStore.select(drawingId);
  drawingEdit.value = {
    id: drawingId,
    handle,
    before: cloneDrawing(drawing),
  };
  window.addEventListener("mousemove", editDrawing);
  window.addEventListener("mouseup", endDrawingEdit, { once: true });
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

function editDrawing(event: MouseEvent) {
  if (!drawingEdit.value) return;
  const point = drawingPointFromEvent(event);
  if (!point) return;
  const { id, handle, before } = drawingEdit.value;
  if ((before.type === "arrow" || before.type === "line") && (handle === "start" || handle === "end")) {
    drawingStore.updateDrawing(id, { [handle]: point });
    return;
  }
  if ((before.type === "rect" || before.type === "ellipse") && handle === "resize") {
    const x = before.x ?? 0;
    const y = before.y ?? 0;
    drawingStore.updateDrawing(id, {
      width: Math.max(8, point.x - x),
      height: Math.max(8, point.y - y),
    });
  }
}

function endDrawingDrag() {
  window.removeEventListener("mousemove", dragDrawing);
  if (drawingDrag.value) captureCanvasHistory(() => drawingStore.commitDrawingMove(drawingDrag.value!.before));
  drawingDrag.value = null;
}

function endDrawingEdit() {
  window.removeEventListener("mousemove", editDrawing);
  if (drawingEdit.value) captureCanvasHistory(() => drawingStore.commitDrawingMove(drawingEdit.value!.before));
  drawingEdit.value = null;
}

function onImagePointerDown(event: MouseEvent, image: CanvasImageType) {
  if (drawingStore.tool !== "select") return;
  if (handActive.value || spaceDown.value || event.button === 1) return;
  const additive = event.shiftKey || event.ctrlKey;
  const alreadySelected = imageStore.selectedIds.includes(image.id);
  if (!additive && !alreadySelected) {
    noteStore.clearSelection();
    drawingStore.clearSelection();
  }
  if (!alreadySelected || additive) imageStore.select(image.id, additive);
  if (additive && alreadySelected) {
    event.preventDefault();
    return;
  }
  if (selectedObjectCount() > 1 && imageStore.selectedIds.includes(image.id)) {
    event.preventDefault();
    startMixedDrag(event);
  }
}

function updateImage(image: CanvasImageType, patch: Partial<CanvasImageType> & { __before?: CanvasImageType }, track = true) {
  const { __before, ...cleanPatch } = patch;
  captureCanvasHistory(() => {
    if (__before) imageStore.commitImageChange(__before, cleanPatch);
    else imageStore.updateImage(image.id, cleanPatch, track);
  });
}

async function addImageAt(clientX?: number, clientY?: number) {
  if (!canvasStore.currentCanvasId || !board.value) return;
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "图片", extensions: [...imageExtensions] }],
  });
  if (typeof selected !== "string") return;
  await addImageFileAt(selected, clientX, clientY);
}

async function addImageFileAt(sourcePath: string, clientX?: number, clientY?: number, index = 0) {
  if (!canvasStore.currentCanvasId || !board.value || !isImagePath(sourcePath)) return;
  try {
    const imported = await importImageFile(sourcePath, settingsStore.settings.imageLibraryPath);
    await createImportedImage(imported, clientX, clientY, index);
  } catch (error) {
    feedback.notify(`添加图片失败：${error instanceof Error ? error.message : String(error)}`, "error");
  }
}

async function addImageBlobAt(blob: Blob, clientX?: number, clientY?: number, index = 0) {
  if (!canvasStore.currentCanvasId || !board.value || !blob.type.startsWith("image/")) return false;
  try {
    const bytes = Array.from(new Uint8Array(await blob.arrayBuffer()));
    const imported = await importImageBytes(bytes, imageNameFromBlob(blob), blob.type, settingsStore.settings.imageLibraryPath);
    await createImportedImage(imported, clientX, clientY, index);
    return true;
  } catch (error) {
    feedback.notify(`粘贴图片失败：${error instanceof Error ? error.message : String(error)}`, "error");
    return false;
  }
}

async function createImportedImage(imported: Awaited<ReturnType<typeof importImageFile>>, clientX?: number, clientY?: number, index = 0) {
  if (!canvasStore.currentCanvasId || !board.value) return;
  const size = await measureImageSize(imageFileUrl(imported.path));
  const rect = board.value.getBoundingClientRect();
  const point = clientX !== undefined && clientY !== undefined
    ? screenToWorld(clientX, clientY, viewport, rect)
    : screenToWorld(rect.left + rect.width / 2, rect.top + rect.height / 2, viewport, rect);
  captureCanvasHistory(() => {
    imageStore.createImage(canvasStore.currentCanvasId, {
      fileName: imported.fileName,
      originalName: imported.originalName,
      contentHash: imported.contentHash,
      x: point.x - size.width / 2 + index * 18,
      y: point.y - size.height / 2 + index * 18,
      width: size.width,
      height: size.height,
      zIndex: globalMaxZ.value + 1 + index,
      rotationEnabled: settingsStore.settings.randomRotation,
    });
  });
  noteStore.clearSelection();
  drawingStore.clearSelection();
}

function imageNameFromBlob(blob: Blob) {
  const extension = blob.type.split("/")[1]?.replace("jpeg", "jpg").replace("svg+xml", "svg") || "png";
  return `clipboard-image.${extension}`;
}

function isImagePath(path: string) {
  const extension = path.split(/[\\/]/).pop()?.split(".").pop()?.toLowerCase();
  return Boolean(extension && imageExtensions.has(extension));
}

function clientPointFromDropPosition(position: { x: number; y: number }) {
  const ratio = window.devicePixelRatio || 1;
  const x = position.x > window.innerWidth + 2 ? position.x / ratio : position.x;
  const y = position.y > window.innerHeight + 2 ? position.y / ratio : position.y;
  return { x, y };
}

async function addDroppedImages(paths: string[], clientX: number, clientY: number) {
  const imagePaths = paths.filter(isImagePath);
  if (!imagePaths.length) return;
  for (const [index, path] of imagePaths.entries()) {
    await addImageFileAt(path, clientX, clientY, index);
  }
}

async function onNativeDrop(event: DragEvent) {
  event.preventDefault();
  event.stopPropagation();
  const files = Array.from(event.dataTransfer?.files ?? []);
  const paths = files
    .map((file) => (file as File & { path?: string }).path)
    .filter((path): path is string => Boolean(path));
  if (!paths.length) return;
  await addDroppedImages(paths, event.clientX, event.clientY);
}

async function onPaste(event: ClipboardEvent) {
  if (isTextInputTarget(event.target) || noteStore.editingId || !canvasStore.currentCanvasId) return;
  const clipboard = event.clipboardData;
  const imageFiles = Array.from(clipboard?.files ?? []).filter((file) => file.type.startsWith("image/"));
  if (!imageFiles.length && clipboard?.items) {
    imageFiles.push(
      ...Array.from(clipboard.items)
        .filter((item) => item.kind === "file" && item.type.startsWith("image/"))
        .map((item) => item.getAsFile())
        .filter((file): file is File => Boolean(file)),
    );
  }
  if (imageFiles.length) {
    event.preventDefault();
    for (const [index, file] of imageFiles.entries()) {
      await addImageBlobAt(file, undefined, undefined, index);
    }
    return;
  }
  if (noteStore.clipboard.length || drawingStore.clipboard.length || imageStore.clipboard.length) {
    event.preventDefault();
    pasteAtCenter();
  }
}

async function startImageDropListener() {
  unlistenImageDrop = await getCurrentWebview().onDragDropEvent(async (event) => {
    if (event.payload.type !== "drop" || !board.value) return;
    const point = clientPointFromDropPosition(event.payload.position);
    const rect = board.value.getBoundingClientRect();
    if (point.x < rect.left || point.x > rect.right || point.y < rect.top || point.y > rect.bottom) return;
    await addDroppedImages(event.payload.paths, point.x, point.y);
  });
}

function measureImageSize(src: string): Promise<{ width: number; height: number }> {
  return new Promise((resolve) => {
    if (!src) {
      resolve({ width: 320, height: 240 });
      return;
    }
    const image = new Image();
    image.onload = () => {
      const maxSize = 420;
      const ratio = Math.min(1, maxSize / Math.max(image.naturalWidth || maxSize, image.naturalHeight || maxSize));
      resolve({
        width: Math.max(120, Math.round((image.naturalWidth || 280) * ratio)),
        height: Math.max(120, Math.round((image.naturalHeight || 220) * ratio)),
      });
    };
    image.onerror = () => resolve({ width: 320, height: 240 });
    image.src = src;
  });
}

function cloneDrawing(drawing: DrawingItem): DrawingItem {
  return {
    ...drawing,
    points: drawing.points?.map((point) => ({ ...point })),
    start: drawing.start ? { ...drawing.start } : undefined,
    end: drawing.end ? { ...drawing.end } : undefined,
  };
}

function cloneImage(image: CanvasImageType): CanvasImageType {
  return { ...image };
}

function undo() {
  const batch = canvasHistory.value.pop();
  if (batch) {
    undoStoreEntries(imageStore.undo, batch.images);
    undoStoreEntries(drawingStore.undo, batch.drawings);
    undoStoreEntries(noteStore.undo, batch.notes);
    canvasFuture.value.push(batch);
    return;
  }
  if (imageStore.selectedIds.length) imageStore.undo();
  else if (drawingStore.selectedIds.length || drawingStore.tool !== "select") drawingStore.undo();
  else if (noteStore.history.length) noteStore.undo();
  else if (!drawingStore.undo()) imageStore.undo();
}

function redo() {
  const batch = canvasFuture.value.pop();
  if (batch) {
    redoStoreEntries(noteStore.redo, batch.notes);
    redoStoreEntries(drawingStore.redo, batch.drawings);
    redoStoreEntries(imageStore.redo, batch.images);
    canvasHistory.value.push(batch);
    return;
  }
  if (imageStore.selectedIds.length) imageStore.redo();
  else if (drawingStore.selectedIds.length || drawingStore.tool !== "select" || (drawingStore.future.length && !noteStore.future.length)) drawingStore.redo();
  else if (noteStore.future.length) noteStore.redo();
  else if (!drawingStore.redo()) imageStore.redo();
}

function duplicateObjectsForContext(type: CanvasObjectType, id: string) {
  ensureContextSelection(type, id);
  duplicateSelectedObjects();
}

function copySelectedObjects() {
  const hasNotes = Boolean(noteStore.selectedIds.length);
  const hasDrawings = Boolean(drawingStore.selectedIds.length);
  const hasImages = Boolean(imageStore.selectedIds.length);
  noteStore.clipboard = [];
  drawingStore.clipboard = [];
  imageStore.clipboard = [];
  if (!hasNotes && !hasDrawings && !hasImages) return;
  if (noteStore.selectedIds.length) noteStore.copySelected();
  if (drawingStore.selectedIds.length) drawingStore.copySelected();
  if (imageStore.selectedIds.length) imageStore.copySelected();
}

function duplicateSelectedObjects() {
  captureCanvasHistory(() => {
    const baseZ = globalMaxZ.value + 1;
    const noteCount = noteStore.selectedIds.length;
    const drawingCount = drawingStore.selectedIds.length;
    if (noteStore.selectedIds.length) {
      if (noteStore.selectedIds.length > 1) noteStore.duplicateSelected(baseZ);
      else noteStore.duplicateNote(noteStore.selectedIds[0], baseZ);
    }
    if (drawingStore.selectedIds.length) drawingStore.duplicateSelected(baseZ + noteCount);
    if (imageStore.selectedIds.length) imageStore.duplicateSelected(baseZ + noteCount + drawingCount);
  });
}

function bringSelectedObjectsToFront() {
  captureCanvasHistory(() => {
    const refs = selectedObjectRefs();
    const selectedNotes = refs.filter((ref): ref is { type: "note"; item: StickyNoteType } => ref.type === "note");
    const selectedImages = refs.filter((ref): ref is { type: "image"; item: CanvasImageType } => ref.type === "image");
    const shouldTogglePinned = Boolean(selectedNotes.length || selectedImages.length);
    const targetPinned = shouldTogglePinned
      ? ![...selectedNotes.map((ref) => ref.item), ...selectedImages.map((ref) => ref.item)].every((item) => item.pinned === true)
      : false;
    const baseZ = globalMaxZ.value + 1;
    refs.forEach((ref, index) => {
      const zIndex = baseZ + index;
      if (ref.type === "note") {
        noteStore.updateNote(ref.item.id, { zIndex, pinned: targetPinned });
      } else if (ref.type === "image") {
        imageStore.updateImage(ref.item.id, { zIndex, pinned: targetPinned });
      } else {
        const before = cloneDrawing(ref.item);
        drawingStore.updateDrawing(ref.item.id, { zIndex }, true);
        const after = drawingStore.drawings.find((drawing) => drawing.id === ref.item.id);
        if (after) drawingStore.addHistory({ type: "update", before, after: cloneDrawing(after) });
      }
    });
  });
}

async function copyNoteText(noteId: string) {
  const note = noteStore.notes.find((item) => item.id === noteId);
  const text = (contentJsonToMarkdown(note?.contentJson) || note?.content?.trim()) ?? "";
  if (!text) {
    feedback.notify("便签没有可复制的文字");
    return;
  }
  await navigator.clipboard.writeText(text);
  feedback.notify("Markdown 已复制", "success");
}

async function copyLink(href: string) {
  await navigator.clipboard.writeText(href);
  feedback.notify("链接已复制", "success");
}

async function copyCode(code: string) {
  await navigator.clipboard.writeText(code);
  feedback.notify("代码已复制", "success");
}

function bringObjectForContext(type: CanvasObjectType, id: string) {
  ensureContextSelection(type, id);
  bringSelectedObjectsToFront();
}

function updateSelection(note: StickyNoteType, patch: Partial<StickyNoteType> & { __before?: StickyNoteType }, track = true) {
  captureCanvasHistory(() => {
    if (noteStore.selectedIds.length > 1 && noteStore.selectedIds.includes(note.id) && !patch.__before) {
      noteStore.updateSelected(patch);
      return;
    }
    updateNote(note, patch, track);
  });
}

function toggleTagForSelection(noteId: string, tagId: string) {
  captureCanvasHistory(() => {
    if (noteStore.selectedIds.length > 1 && noteStore.selectedIds.includes(noteId)) noteStore.toggleTagForSelected(tagId);
    else noteStore.toggleTagForNote(noteId, tagId);
  });
}

function changeColorForContext(noteId: string, color: NoteColor) {
  captureCanvasHistory(() => {
    if (noteStore.selectedIds.length > 1 && noteStore.selectedIds.includes(noteId)) noteStore.updateSelected({ color });
    else noteStore.updateNote(noteId, { color });
  });
  contextMenu.value = null;
}

function pasteAtContext() {
  captureCanvasHistory(() => {
    const baseZ = globalMaxZ.value + 1;
    noteStore.pasteClipboard(canvasStore.currentCanvasId, contextWorld.value.x, contextWorld.value.y, baseZ);
    drawingStore.pasteClipboard(canvasStore.currentCanvasId, contextWorld.value.x, contextWorld.value.y, baseZ + noteStore.clipboard.length);
    imageStore.pasteClipboard(canvasStore.currentCanvasId, contextWorld.value.x, contextWorld.value.y, baseZ + noteStore.clipboard.length + drawingStore.clipboard.length);
  });
  contextMenu.value = null;
}

function pasteAtCenter() {
  const rect = board.value?.getBoundingClientRect();
  if (!rect || !canvasStore.currentCanvasId) return;
  const point = screenToWorld(rect.left + rect.width / 2, rect.top + rect.height / 2, viewport, rect);
  captureCanvasHistory(() => {
    const baseZ = globalMaxZ.value + 1;
    noteStore.pasteClipboard(canvasStore.currentCanvasId, point.x, point.y, baseZ);
    drawingStore.pasteClipboard(canvasStore.currentCanvasId, point.x, point.y, baseZ + noteStore.clipboard.length);
    imageStore.pasteClipboard(canvasStore.currentCanvasId, point.x, point.y, baseZ + noteStore.clipboard.length + drawingStore.clipboard.length);
  });
}

function changeDrawingColorForContext(color: string) {
  drawingStore.color = color;
  if (drawingStore.selectedIds.length) captureCanvasHistory(() => drawingStore.updateSelected({ color }));
  contextMenu.value = null;
}

function setDrawingColor(color: string) {
  drawingStore.color = color;
  if (drawingStore.selectedIds.length) captureCanvasHistory(() => drawingStore.updateSelected({ color }));
}

function setDrawingStrokeWidth(width: number) {
  const strokeWidth = Math.min(16, Math.max(1, width || 1));
  drawingStore.strokeWidth = strokeWidth;
  if (drawingStore.selectedIds.length) captureCanvasHistory(() => drawingStore.updateSelected({ strokeWidth }));
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
  if (right - left < minBoxSelectDistance && bottom - top < minBoxSelectDistance) {
    boxSelect.value = null;
    return;
  }
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
  const selectedImages = currentCanvasImages.value.filter((image) => {
    const imageLeft = image.x * viewport.scale + viewport.offsetX;
    const imageTop = image.y * viewport.scale + viewport.offsetY;
    const imageRight = imageLeft + image.width * viewport.scale;
    const imageBottom = imageTop + image.height * viewport.scale;
    return imageRight >= left && imageLeft <= right && imageBottom >= top && imageTop <= bottom;
  });
  noteStore.setSelection(selectedNotes.map((note) => note.id));
  drawingStore.setSelection(selectedDrawings.map((drawing) => drawing.id));
  imageStore.selectedIds = selectedImages.map((image) => image.id);
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
    if (noteStore.selectedIds.length || drawingStore.selectedIds.length || imageStore.selectedIds.length) {
      event.preventDefault();
      copySelectedObjects();
    }
    return;
  }
  if (event.ctrlKey && event.key.toLowerCase() === "d" && !noteStore.editingId) {
    if (noteStore.selectedIds.length || drawingStore.selectedIds.length || imageStore.selectedIds.length) {
      event.preventDefault();
      duplicateSelectedObjects();
    }
    return;
  }
  if ((event.key === "Delete" || event.key === "Backspace") && (drawingStore.selectedIds.length || noteStore.selectedIds.length || imageStore.selectedIds.length) && !noteStore.editingId) {
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
}

function onKeyup(event: KeyboardEvent) {
  if (event.code === "Space") spaceDown.value = false;
}

onMounted(() => {
  updateBoardSize();
  void startImageDropListener().catch((error) => feedback.notify(`监听图片拖拽失败：${error instanceof Error ? error.message : String(error)}`, "error"));
  window.addEventListener("keydown", onKeydown);
  window.addEventListener("keyup", onKeyup);
  window.addEventListener("paste", onPaste);
  window.addEventListener("locate-note", onLocateNote);
  window.addEventListener("resize", updateBoardSize);
});

onUnmounted(() => {
  window.removeEventListener("mousemove", updateDrawing);
  window.removeEventListener("mousemove", dragDrawing);
  window.removeEventListener("mousemove", editDrawing);
  window.removeEventListener("mousemove", dragMixed);
  window.removeEventListener("keydown", onKeydown);
  window.removeEventListener("keyup", onKeyup);
  window.removeEventListener("paste", onPaste);
  window.removeEventListener("locate-note", onLocateNote);
  window.removeEventListener("resize", updateBoardSize);
  unlistenImageDrop?.();
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
    @dragover.prevent
    @drop.prevent="onNativeDrop"
  >
    <div v-if="appStore.loaded && !appStore.databaseReady" class="empty-board database-error">
      数据库未加载<br />
      <span>{{ appStore.loadError }}</span>
      <button @click.stop="settingsStore.togglePanel()">处理数据库</button>
    </div>
    <div v-else-if="!canvasStore.currentCanvasId" class="empty-board">还没有画布<br />点击“+”开始整理你的想法</div>
    <div v-else-if="!visibleNotes.length && filterActive" class="empty-board filtered-empty">
      当前筛选下没有便签<br />
      <button @click.stop="clearFilters">清除筛选</button>
    </div>
    <div v-else-if="!visibleNotes.length && !currentCanvasDrawings.length && !currentCanvasImages.length" class="empty-board">双击画布空白处创建第一张便签</div>

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
        v-for="drawing in currentCanvasDrawings"
        :key="drawing.id"
        :canvas-id="canvasStore.currentCanvasId"
        :drawings="[drawing]"
        :scale="viewport.scale"
        :pan-mode="handActive || spaceDown"
        :style="drawingLayerStyle(drawing)"
        @drag-drawing="startDrawingDrag"
        @edit-drawing="startDrawingEdit"
        @context-drawing="openDrawingMenu"
      />
      <CanvasImage
        v-for="image in currentCanvasImages"
        :key="image.id"
        :image="image"
        :selected="imageStore.selectedIds.includes(image.id)"
        :scale="viewport.scale"
        :library-path="settingsStore.settings.imageLibraryPath"
        :pan-mode="handActive || spaceDown || drawingStore.tool !== 'select'"
        @select="onImagePointerDown($event, image)"
        @update="(patch, track) => updateImage(image, patch, track)"
        @live="(patch) => imageStore.patchImageLive(image.id, patch)"
        @delete="imageStore.deleteImage(image.id)"
        @context="(event) => openImageMenu(event, image.id)"
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
        @delete="deleteObjectForContext('note', note.id)"
        @duplicate="duplicateObjectsForContext('note', note.id)"
        @copy-text="copyNoteText(note.id)"
        @front="bringObjectForContext('note', note.id)"
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
      @add-image="addImageAt()"
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
        <button @click="duplicateObjectsForContext('note', contextMenu!.noteId!); contextMenu = null">复制一份</button>
        <button @click="bringObjectForContext('note', contextMenu!.noteId!); contextMenu = null">
          {{ noteStore.notes.find((note) => note.id === contextMenu!.noteId)?.pinned ? "取消置顶" : "置顶" }}
        </button>
        <button @click="deleteObjectForContext('note', contextMenu!.noteId!); contextMenu = null">删除</button>
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
        <button @click="bringObjectForContext('drawing', contextMenu!.drawingId!); contextMenu = null">置顶</button>
        <button @click="deleteObjectForContext('drawing', contextMenu!.drawingId!); contextMenu = null">删除</button>
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
      <template v-else-if="contextMenu.imageId">
        <button @click="copySelectedObjects(); contextMenu = null">复制</button>
        <button @click="duplicateSelectedObjects(); contextMenu = null">复制一份</button>
        <button @click="bringObjectForContext('image', contextMenu!.imageId!); contextMenu = null">
          {{ imageStore.images.find((image) => image.id === contextMenu!.imageId)?.pinned ? "取消置顶" : "置顶" }}
        </button>
        <button @click="deleteObjectForContext('image', contextMenu!.imageId!); contextMenu = null">删除</button>
      </template>
      <template v-else>
        <button @click="createNoteAt(contextMenu!.x, contextMenu!.y); contextMenu = null">新建便签</button>
        <button @click="addImageAt(contextMenu!.x, contextMenu!.y); contextMenu = null">添加图片</button>
        <button :disabled="!noteStore.clipboard.length && !drawingStore.clipboard.length && !imageStore.clipboard.length" @click="pasteAtContext">粘贴</button>
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
