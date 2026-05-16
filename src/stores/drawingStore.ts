import { defineStore } from "pinia";
import { nanoid } from "nanoid";
import type { DrawingHistoryEntry, DrawingItem, DrawingPoint, DrawingTool } from "../types";
import { deleteDrawingData, deleteDrawingsByCanvasData, reportPersistenceError, saveDrawingData } from "../utils/storage";

const now = () => new Date().toISOString();

const saveDrawingSafely = (drawing: DrawingItem) => {
  void saveDrawingData(drawing).catch((error) => reportPersistenceError("保存绘图", error));
};

const deleteDrawingSafely = (id: string) => {
  void deleteDrawingData(id).catch((error) => reportPersistenceError("删除绘图", error));
};

function cloneDrawing(drawing: DrawingItem): DrawingItem {
  return {
    ...drawing,
    points: drawing.points?.map((point) => ({ ...point })),
    start: drawing.start ? { ...drawing.start } : undefined,
    end: drawing.end ? { ...drawing.end } : undefined,
  };
}

function moveDrawingFrom(before: DrawingItem, deltaX: number, deltaY: number): Partial<DrawingItem> {
  return {
    points: before.points?.map((point) => ({ x: point.x + deltaX, y: point.y + deltaY })),
    start: before.start ? { x: before.start.x + deltaX, y: before.start.y + deltaY } : undefined,
    end: before.end ? { x: before.end.x + deltaX, y: before.end.y + deltaY } : undefined,
    x: before.x === undefined ? undefined : before.x + deltaX,
    y: before.y === undefined ? undefined : before.y + deltaY,
  };
}

const hasMeaningfulChange = (before: DrawingItem, after: DrawingItem) =>
  JSON.stringify({ ...before, updatedAt: undefined }) !== JSON.stringify({ ...after, updatedAt: undefined });

export const useDrawingStore = defineStore("drawing", {
  state: () => ({
    drawings: [] as DrawingItem[],
    selectedId: "",
    selectedIds: [] as string[],
    tool: "select" as DrawingTool,
    color: "#ff0000",
    strokeWidth: 8,
    history: [] as DrawingHistoryEntry[],
    future: [] as DrawingHistoryEntry[],
  }),
  getters: {
    maxZ: (state) => Math.max(0, ...state.drawings.map((drawing) => drawing.zIndex)),
  },
  actions: {
    setDrawings(drawings: DrawingItem[]) {
      this.drawings = drawings;
      this.selectedId = "";
      this.selectedIds = [];
      this.history = [];
      this.future = [];
    },
    drawingsForCanvas(canvasId: string) {
      return this.drawings.filter((drawing) => drawing.canvasId === canvasId).sort((a, b) => a.zIndex - b.zIndex);
    },
    setTool(tool: DrawingTool) {
      this.tool = tool;
      if (tool !== "select") this.clearSelection();
    },
    createDrawing(canvasId: string, patch: Partial<DrawingItem>) {
      const drawing: DrawingItem = {
        id: nanoid(),
        canvasId,
        type: this.tool,
        color: this.color,
        strokeWidth: this.strokeWidth,
        zIndex: this.maxZ + 1,
        pinned: false,
        createdAt: now(),
        updatedAt: now(),
        ...patch,
      };
      this.drawings.push(drawing);
      return drawing;
    },
    addHistory(entry: Omit<DrawingHistoryEntry, "id" | "timestamp">) {
      this.history.push({ ...entry, id: nanoid(), timestamp: now() });
      if (this.history.length > 50) this.history.shift();
      this.future = [];
    },
    updateDrawing(id: string, patch: Partial<DrawingItem>, persist = false) {
      const drawing = this.drawings.find((item) => item.id === id);
      if (!drawing) return;
      Object.assign(drawing, patch, { updatedAt: now() });
      if (persist) saveDrawingSafely(drawing);
    },
    updateSelected(patch: Partial<DrawingItem>) {
      this.selectedIds.forEach((id) => {
        const drawing = this.drawings.find((item) => item.id === id);
        if (!drawing) return;
        const before = cloneDrawing(drawing);
        Object.assign(drawing, patch, { updatedAt: now() });
        if (hasMeaningfulChange(before, drawing)) this.addHistory({ type: "update", before, after: cloneDrawing(drawing) });
        saveDrawingSafely(drawing);
      });
    },
    finishDrawing(id: string) {
      const drawing = this.drawings.find((item) => item.id === id);
      if (!drawing) return;
      this.addHistory({ type: "create", after: cloneDrawing(drawing) });
      this.setSelection([drawing.id]);
      saveDrawingSafely(drawing);
    },
    select(id: string, additive = false) {
      if (!additive) {
        this.setSelection([id]);
        return;
      }
      this.setSelection(this.selectedIds.includes(id)
        ? this.selectedIds.filter((item) => item !== id)
        : [...this.selectedIds, id]);
    },
    setSelection(ids: string[]) {
      this.selectedIds = Array.from(new Set(ids)).filter((id) => this.drawings.some((drawing) => drawing.id === id));
      this.selectedId = this.selectedIds[0] ?? "";
    },
    clearSelection() {
      this.selectedId = "";
      this.selectedIds = [];
    },
    deleteDrawing(id: string, track = true) {
      const drawing = this.drawings.find((item) => item.id === id);
      if (!drawing) return;
      this.drawings = this.drawings.filter((drawing) => drawing.id !== id);
      if (this.selectedIds.includes(id)) this.setSelection(this.selectedIds.filter((item) => item !== id));
      if (track) this.addHistory({ type: "delete", before: cloneDrawing(drawing) });
      deleteDrawingSafely(id);
    },
    deleteSelected() {
      [...this.selectedIds].forEach((id) => this.deleteDrawing(id));
    },
    duplicateSelected(baseZ?: number) {
      if (!this.selectedIds.length) return;
      const copies = this.drawings
        .filter((drawing) => this.selectedIds.includes(drawing.id))
        .map((drawing, index) => ({
          ...cloneDrawing(drawing),
          ...moveDrawingFrom(drawing, 28 + index * 8, 28 + index * 8),
          id: nanoid(),
          zIndex: baseZ === undefined ? this.maxZ + index + 1 : baseZ + index,
          createdAt: now(),
          updatedAt: now(),
        }));
      this.drawings.push(...copies);
      this.setSelection(copies.map((drawing) => drawing.id));
      copies.forEach((drawing) => {
        this.addHistory({ type: "create", after: cloneDrawing(drawing) });
        saveDrawingSafely(drawing);
      });
    },
    bringSelectedToFront() {
      const selected = this.drawings.filter((drawing) => this.selectedIds.includes(drawing.id));
      if (!selected.length) return;
      const targetPinned = !selected.every((drawing) => drawing.pinned === true);
      const baseZ = this.maxZ;
      selected.forEach((drawing, index) => {
        const before = cloneDrawing(drawing);
        drawing.pinned = targetPinned;
        drawing.zIndex = baseZ + index + 1;
        drawing.updatedAt = now();
        if (hasMeaningfulChange(before, drawing)) this.addHistory({ type: "update", before, after: cloneDrawing(drawing) });
        saveDrawingSafely(drawing);
      });
    },
    removeDrawingsByCanvas(canvasId: string) {
      this.drawings = this.drawings.filter((drawing) => drawing.canvasId !== canvasId);
      this.setSelection(this.selectedIds);
      void deleteDrawingsByCanvasData(canvasId).catch((error) => reportPersistenceError("删除画布绘图", error));
    },
    appendPoint(id: string, point: DrawingPoint) {
      const drawing = this.drawings.find((item) => item.id === id);
      if (!drawing) return;
      drawing.points = [...(drawing.points ?? []), point];
      drawing.updatedAt = now();
    },
    moveDrawingLive(id: string, before: DrawingItem, deltaX: number, deltaY: number) {
      const drawing = this.drawings.find((item) => item.id === id);
      if (!drawing) return;
      Object.assign(drawing, moveDrawingFrom(before, deltaX, deltaY), { updatedAt: now() });
    },
    moveSelectedBy(deltaX: number, deltaY: number, beforeDrawings: DrawingItem[]) {
      beforeDrawings.forEach((before) => this.moveDrawingLive(before.id, before, deltaX, deltaY));
    },
    commitDrawingMove(before: DrawingItem) {
      const drawing = this.drawings.find((item) => item.id === before.id);
      if (!drawing) return;
      if (hasMeaningfulChange(before, drawing)) this.addHistory({ type: "update", before: cloneDrawing(before), after: cloneDrawing(drawing) });
      saveDrawingSafely(drawing);
    },
    commitSelectedMove(beforeDrawings: DrawingItem[]) {
      beforeDrawings.forEach((before) => this.commitDrawingMove(before));
    },
    undo() {
      const entry = this.history.pop();
      if (!entry) return false;
      if (entry.type === "create" && entry.after) {
        this.drawings = this.drawings.filter((drawing) => drawing.id !== entry.after?.id);
        if (this.selectedIds.includes(entry.after.id)) this.setSelection(this.selectedIds.filter((id) => id !== entry.after?.id));
        deleteDrawingSafely(entry.after.id);
      }
      if (entry.type === "delete" && entry.before) {
        this.drawings.push(cloneDrawing(entry.before));
        this.setSelection([entry.before.id]);
        saveDrawingSafely(entry.before);
      }
      if (entry.type === "update" && entry.before) {
        const index = this.drawings.findIndex((drawing) => drawing.id === entry.before?.id);
        if (index >= 0) {
          this.drawings[index] = cloneDrawing(entry.before);
          this.setSelection([entry.before.id]);
          saveDrawingSafely(entry.before);
        }
      }
      this.future.push(entry);
      return true;
    },
    redo() {
      const entry = this.future.pop();
      if (!entry) return false;
      if (entry.type === "create" && entry.after) {
        this.drawings.push(cloneDrawing(entry.after));
        this.setSelection([entry.after.id]);
        saveDrawingSafely(entry.after);
      }
      if (entry.type === "delete" && entry.before) {
        this.drawings = this.drawings.filter((drawing) => drawing.id !== entry.before?.id);
        if (this.selectedIds.includes(entry.before.id)) this.setSelection(this.selectedIds.filter((id) => id !== entry.before?.id));
        deleteDrawingSafely(entry.before.id);
      }
      if (entry.type === "update" && entry.after) {
        const index = this.drawings.findIndex((drawing) => drawing.id === entry.after?.id);
        if (index >= 0) {
          this.drawings[index] = cloneDrawing(entry.after);
          this.setSelection([entry.after.id]);
          saveDrawingSafely(entry.after);
        }
      }
      this.history.push(entry);
      return true;
    },
  },
});
