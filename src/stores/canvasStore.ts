import { defineStore } from "pinia";
import { nanoid } from "nanoid";
import type { CanvasItem } from "../types";
import type { ViewportState } from "../types";
import { deleteCanvasData, removeCanvasForeverData, reportPersistenceError, restoreCanvasData, saveCanvasData } from "../utils/storage";

const now = () => new Date().toISOString();

export const useCanvasStore = defineStore("canvas", {
  state: () => ({
    canvases: [] as CanvasItem[],
    currentCanvasId: "",
    searchQuery: "",
  }),
  getters: {
    activeCanvases: (state) => state.canvases.filter((canvas) => !canvas.deletedAt).sort(compareCanvasOrder),
    deletedCanvases: (state) => state.canvases.filter((canvas) => canvas.deletedAt).sort(compareCanvasOrder),
    currentCanvas: (state) => state.canvases.find((canvas) => canvas.id === state.currentCanvasId),
  },
  actions: {
    setCanvases(canvases: CanvasItem[]) {
      this.canvases = normalizeCanvasOrder(canvases);
      this.currentCanvasId = this.activeCanvases[0]?.id ?? "";
    },
    createCanvas(name = "新画布") {
      const topOrder = Math.min(0, ...this.activeCanvases.map((canvas) => canvas.sortOrder)) - 1;
      const canvas: CanvasItem = {
        id: nanoid(),
        name,
        createdAt: now(),
        updatedAt: now(),
        sortOrder: topOrder,
        deletedAt: null,
      };
      this.canvases.unshift(canvas);
      this.currentCanvasId = canvas.id;
      void saveCanvasData(canvas).catch((error) => reportPersistenceError("保存画布", error));
      return canvas;
    },
    selectCanvas(id: string) {
      this.currentCanvasId = id;
    },
    renameCanvas(id: string, name: string) {
      const canvas = this.canvases.find((item) => item.id === id);
      if (canvas && name.trim()) {
        canvas.name = name.trim();
        canvas.updatedAt = now();
        void saveCanvasData(canvas).catch((error) => reportPersistenceError("保存画布", error));
      }
    },
    deleteCanvas(id: string) {
      const canvas = this.canvases.find((item) => item.id === id);
      if (!canvas) return;
      const deletedAt = now();
      canvas.deletedAt = deletedAt;
      canvas.updatedAt = deletedAt;
      void deleteCanvasData(id, deletedAt).catch((error) => reportPersistenceError("删除画布", error));
      if (this.currentCanvasId === id) {
        this.currentCanvasId = this.canvases.find((item) => !item.deletedAt)?.id ?? "";
      }
    },
    restoreCanvas(id: string) {
      const canvas = this.canvases.find((item) => item.id === id);
      if (canvas) {
        canvas.deletedAt = null;
        canvas.updatedAt = now();
        void restoreCanvasData(id, canvas.updatedAt).catch((error) => reportPersistenceError("恢复画布", error));
      }
    },
    removeForever(id: string) {
      this.canvases = this.canvases.filter((item) => item.id !== id);
      void removeCanvasForeverData(id).catch((error) => reportPersistenceError("永久删除画布", error));
    },
    updateViewport(id: string, viewport: ViewportState) {
      const canvas = this.canvases.find((item) => item.id === id);
      if (!canvas) return;
      canvas.viewport = { ...viewport };
      void saveCanvasData(canvas).catch((error) => reportPersistenceError("保存视口", error));
    },
    moveCanvas(id: string, direction: -1 | 1) {
      const active = this.activeCanvases;
      const index = active.findIndex((canvas) => canvas.id === id);
      const targetIndex = index + direction;
      if (index < 0 || targetIndex < 0 || targetIndex >= active.length) return;
      const reordered = [...active];
      const [canvas] = reordered.splice(index, 1);
      reordered.splice(targetIndex, 0, canvas);
      reordered.forEach((item, order) => {
        item.sortOrder = order;
      });
      this.canvases = [
        ...reordered,
        ...this.deletedCanvases,
      ];
      reordered.forEach((item) => {
        void saveCanvasData(item).catch((error) => reportPersistenceError("保存画布顺序", error));
      });
    },
  },
});

function compareCanvasOrder(a: CanvasItem, b: CanvasItem) {
  return canvasSortOrder(a) - canvasSortOrder(b) || a.createdAt.localeCompare(b.createdAt);
}

function normalizeCanvasOrder(canvases: CanvasItem[]) {
  const ordered = [...canvases].sort(compareCanvasOrder);
  ordered.forEach((canvas, index) => {
    if (!Number.isFinite(canvas.sortOrder)) canvas.sortOrder = index;
  });
  return ordered;
}

function canvasSortOrder(canvas: CanvasItem) {
  return Number.isFinite(canvas.sortOrder) ? canvas.sortOrder : Number.MAX_SAFE_INTEGER;
}
