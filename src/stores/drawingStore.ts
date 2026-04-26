import { defineStore } from "pinia";
import { nanoid } from "nanoid";
import type { DrawingItem, DrawingPoint, DrawingTool } from "../types";
import { deleteDrawingData, deleteDrawingsByCanvasData, reportPersistenceError, saveDrawingData } from "../utils/storage";

const now = () => new Date().toISOString();

const saveDrawingSafely = (drawing: DrawingItem) => {
  void saveDrawingData(drawing).catch((error) => reportPersistenceError("保存绘图", error));
};

const deleteDrawingSafely = (id: string) => {
  void deleteDrawingData(id).catch((error) => reportPersistenceError("删除绘图", error));
};

export const useDrawingStore = defineStore("drawing", {
  state: () => ({
    drawings: [] as DrawingItem[],
    selectedId: "",
    tool: "select" as DrawingTool,
    color: "#1f2937",
    strokeWidth: 3,
  }),
  getters: {
    maxZ: (state) => Math.max(0, ...state.drawings.map((drawing) => drawing.zIndex)),
  },
  actions: {
    setDrawings(drawings: DrawingItem[]) {
      this.drawings = drawings;
      this.selectedId = "";
    },
    drawingsForCanvas(canvasId: string) {
      return this.drawings.filter((drawing) => drawing.canvasId === canvasId).sort((a, b) => a.zIndex - b.zIndex);
    },
    setTool(tool: DrawingTool) {
      this.tool = tool;
      if (tool !== "select") this.selectedId = "";
    },
    createDrawing(canvasId: string, patch: Partial<DrawingItem>) {
      const drawing: DrawingItem = {
        id: nanoid(),
        canvasId,
        type: this.tool,
        color: this.color,
        strokeWidth: this.strokeWidth,
        zIndex: this.maxZ + 1,
        createdAt: now(),
        updatedAt: now(),
        ...patch,
      };
      this.drawings.push(drawing);
      return drawing;
    },
    updateDrawing(id: string, patch: Partial<DrawingItem>, persist = false) {
      const drawing = this.drawings.find((item) => item.id === id);
      if (!drawing) return;
      Object.assign(drawing, patch, { updatedAt: now() });
      if (persist) saveDrawingSafely(drawing);
    },
    finishDrawing(id: string) {
      const drawing = this.drawings.find((item) => item.id === id);
      if (drawing) saveDrawingSafely(drawing);
    },
    select(id: string) {
      this.selectedId = id;
    },
    clearSelection() {
      this.selectedId = "";
    },
    deleteDrawing(id: string) {
      this.drawings = this.drawings.filter((drawing) => drawing.id !== id);
      if (this.selectedId === id) this.selectedId = "";
      deleteDrawingSafely(id);
    },
    removeDrawingsByCanvas(canvasId: string) {
      this.drawings = this.drawings.filter((drawing) => drawing.canvasId !== canvasId);
      if (this.selectedId && !this.drawings.some((drawing) => drawing.id === this.selectedId)) this.selectedId = "";
      void deleteDrawingsByCanvasData(canvasId).catch((error) => reportPersistenceError("删除画布绘图", error));
    },
    appendPoint(id: string, point: DrawingPoint) {
      const drawing = this.drawings.find((item) => item.id === id);
      if (!drawing) return;
      drawing.points = [...(drawing.points ?? []), point];
      drawing.updatedAt = now();
    },
  },
});
