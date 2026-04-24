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
    activeCanvases: (state) => state.canvases.filter((canvas) => !canvas.deletedAt),
    deletedCanvases: (state) => state.canvases.filter((canvas) => canvas.deletedAt),
    currentCanvas: (state) => state.canvases.find((canvas) => canvas.id === state.currentCanvasId),
  },
  actions: {
    setCanvases(canvases: CanvasItem[]) {
      this.canvases = canvases;
      this.currentCanvasId = canvases.find((canvas) => !canvas.deletedAt)?.id ?? "";
    },
    createCanvas(name = "新画布") {
      const canvas: CanvasItem = {
        id: nanoid(),
        name,
        createdAt: now(),
        updatedAt: now(),
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
      canvas.updatedAt = now();
      void saveCanvasData(canvas).catch((error) => reportPersistenceError("保存视口", error));
    },
  },
});
