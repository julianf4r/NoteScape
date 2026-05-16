import { defineStore } from "pinia";
import { nanoid } from "nanoid";
import type { CanvasImage, ImageHistoryEntry } from "../types";
import { deleteImageData, deleteImagesByCanvasData, reportPersistenceError, saveImageData } from "../utils/storage";

const now = () => new Date().toISOString();
const randomRotation = () => Math.round((Math.random() * 4 - 2) * 10) / 10;

const saveImageSafely = (image: CanvasImage) => {
  void saveImageData(image).catch((error) => reportPersistenceError("保存图片", error));
};

const deleteImageSafely = (id: string) => {
  void deleteImageData(id).catch((error) => reportPersistenceError("删除图片", error));
};

function cloneImage(image: CanvasImage): CanvasImage {
  return { ...image };
}

const hasMeaningfulChange = (before: CanvasImage, after: CanvasImage) =>
  JSON.stringify({ ...before, updatedAt: undefined }) !== JSON.stringify({ ...after, updatedAt: undefined });

export const useImageStore = defineStore("image", {
  state: () => ({
    images: [] as CanvasImage[],
    selectedIds: [] as string[],
    history: [] as ImageHistoryEntry[],
    future: [] as ImageHistoryEntry[],
    clipboard: [] as CanvasImage[],
  }),
  getters: {
    maxZ: (state) => Math.max(0, ...state.images.map((image) => image.zIndex)),
  },
  actions: {
    setImages(images: CanvasImage[]) {
      this.images = images;
      this.selectedIds = [];
      this.history = [];
      this.future = [];
    },
    imagesForCanvas(canvasId: string) {
      return this.images.filter((image) => image.canvasId === canvasId).sort((a, b) => a.zIndex - b.zIndex);
    },
    addHistory(entry: Omit<ImageHistoryEntry, "id" | "timestamp">) {
      this.history.push({ ...entry, id: nanoid(), timestamp: now() });
      if (this.history.length > 50) this.history.shift();
      this.future = [];
    },
    createImage(canvasId: string, patch: {
      fileName: string;
      originalName?: string;
      contentHash?: string;
      x: number;
      y: number;
      width: number;
      height: number;
      zIndex?: number;
      rotationEnabled?: boolean;
    }) {
      const image: CanvasImage = {
        id: nanoid(),
        canvasId,
        fileName: patch.fileName,
        originalName: patch.originalName,
        contentHash: patch.contentHash,
        x: patch.x,
        y: patch.y,
        width: patch.width,
        height: patch.height,
        rotation: patch.rotationEnabled === false ? 0 : randomRotation(),
        zIndex: patch.zIndex ?? this.maxZ + 1,
        pinned: false,
        createdAt: now(),
        updatedAt: now(),
      };
      this.images.push(image);
      this.selectedIds = [image.id];
      this.addHistory({ type: "create", after: cloneImage(image) });
      saveImageSafely(image);
      return image;
    },
    cloneImageToCanvas(image: CanvasImage, canvasId: string, x: number, y: number, index = 0, zIndex?: number) {
      const copy: CanvasImage = {
        ...cloneImage(image),
        id: nanoid(),
        canvasId,
        x,
        y,
        zIndex: zIndex ?? this.maxZ + index + 1,
        createdAt: now(),
        updatedAt: now(),
      };
      return copy;
    },
    updateImage(id: string, patch: Partial<CanvasImage>, track = true) {
      const image = this.images.find((item) => item.id === id);
      if (!image) return;
      const before = cloneImage(image);
      Object.assign(image, patch, { updatedAt: now() });
      if (track && hasMeaningfulChange(before, image)) this.addHistory({ type: "update", before, after: cloneImage(image) });
      saveImageSafely(image);
    },
    patchImageLive(id: string, patch: Partial<CanvasImage>) {
      const image = this.images.find((item) => item.id === id);
      if (image) Object.assign(image, patch);
    },
    commitImageChange(before: CanvasImage, patch: Partial<CanvasImage>) {
      const image = this.images.find((item) => item.id === before.id);
      if (!image) return;
      Object.assign(image, patch, { updatedAt: now() });
      if (hasMeaningfulChange(before, image)) this.addHistory({ type: "update", before, after: cloneImage(image) });
      saveImageSafely(image);
    },
    deleteImage(id: string) {
      const image = this.images.find((item) => item.id === id);
      if (!image) return;
      this.images = this.images.filter((item) => item.id !== id);
      this.selectedIds = this.selectedIds.filter((item) => item !== id);
      this.addHistory({ type: "delete", before: cloneImage(image) });
      deleteImageSafely(id);
    },
    deleteSelected() {
      [...this.selectedIds].forEach((id) => this.deleteImage(id));
    },
    copySelected() {
      this.clipboard = this.images
        .filter((image) => this.selectedIds.includes(image.id))
        .map(cloneImage);
    },
    duplicateSelected(baseZ?: number) {
      const selected = this.images.filter((image) => this.selectedIds.includes(image.id));
      if (!selected.length) return;
      const copies = selected.map((image, index) => this.cloneImageToCanvas(
        image,
        image.canvasId,
        image.x + 28 + index * 8,
        image.y + 28 + index * 8,
        index,
        baseZ === undefined ? undefined : baseZ + index,
      ));
      this.images.push(...copies);
      this.selectedIds = copies.map((image) => image.id);
      copies.forEach((image) => {
        this.addHistory({ type: "create", after: cloneImage(image) });
        saveImageSafely(image);
      });
    },
    pasteClipboard(canvasId: string, x: number, y: number, baseZ?: number) {
      if (!this.clipboard.length) return;
      const minX = Math.min(...this.clipboard.map((image) => image.x));
      const minY = Math.min(...this.clipboard.map((image) => image.y));
      const copies = this.clipboard.map((image, index) => this.cloneImageToCanvas(
        image,
        canvasId,
        x + (image.x - minX) + index * 8,
        y + (image.y - minY) + index * 8,
        index,
        baseZ === undefined ? undefined : baseZ + index,
      ));
      this.images.push(...copies);
      this.selectedIds = copies.map((image) => image.id);
      copies.forEach((image) => {
        this.addHistory({ type: "create", after: cloneImage(image) });
        saveImageSafely(image);
      });
    },
    moveSelectedBy(deltaX: number, deltaY: number, beforeImages: CanvasImage[]) {
      beforeImages.forEach((before) => {
        const image = this.images.find((item) => item.id === before.id);
        if (!image) return;
        image.x = before.x + deltaX;
        image.y = before.y + deltaY;
        image.updatedAt = now();
      });
    },
    commitSelectedMove(beforeImages: CanvasImage[]) {
      beforeImages.forEach((before) => {
        const image = this.images.find((item) => item.id === before.id);
        if (!image) return;
        if (hasMeaningfulChange(before, image)) this.addHistory({ type: "update", before: cloneImage(before), after: cloneImage(image) });
        saveImageSafely(image);
      });
    },
    bringSelectedToFront() {
      const selected = this.images.filter((image) => this.selectedIds.includes(image.id));
      if (!selected.length) return;
      const targetPinned = !selected.every((image) => image.pinned === true);
      const baseZ = this.maxZ;
      selected.forEach((image, index) => {
        const before = cloneImage(image);
        image.pinned = targetPinned;
        image.zIndex = baseZ + index + 1;
        image.updatedAt = now();
        if (hasMeaningfulChange(before, image)) this.addHistory({ type: "update", before, after: cloneImage(image) });
        saveImageSafely(image);
      });
    },
    bringToFront(id: string) {
      const image = this.images.find((item) => item.id === id);
      if (!image) return;
      const before = cloneImage(image);
      image.pinned = !image.pinned;
      image.zIndex = this.maxZ + 1;
      image.updatedAt = now();
      if (hasMeaningfulChange(before, image)) this.addHistory({ type: "update", before, after: cloneImage(image) });
      saveImageSafely(image);
    },
    removeImagesByCanvas(canvasId: string) {
      this.images = this.images.filter((image) => image.canvasId !== canvasId);
      this.selectedIds = this.selectedIds.filter((id) => this.images.some((image) => image.id === id));
      void deleteImagesByCanvasData(canvasId).catch((error) => reportPersistenceError("删除画布图片", error));
    },
    select(id: string, additive = false) {
      if (!additive) {
        this.selectedIds = [id];
        return;
      }
      this.selectedIds = this.selectedIds.includes(id)
        ? this.selectedIds.filter((item) => item !== id)
        : [...this.selectedIds, id];
    },
    setSelection(ids: string[]) {
      this.selectedIds = Array.from(new Set(ids)).filter((id) => this.images.some((image) => image.id === id));
    },
    clearSelection() {
      this.selectedIds = [];
    },
    undo() {
      const entry = this.history.pop();
      if (!entry) return false;
      if (entry.type === "create" && entry.after) {
        this.images = this.images.filter((image) => image.id !== entry.after?.id);
        this.selectedIds = this.selectedIds.filter((id) => id !== entry.after?.id);
        deleteImageSafely(entry.after.id);
      }
      if (entry.type === "delete" && entry.before) {
        this.images.push(cloneImage(entry.before));
        this.selectedIds = [entry.before.id];
        saveImageSafely(entry.before);
      }
      if (entry.type === "update" && entry.before) {
        const index = this.images.findIndex((image) => image.id === entry.before?.id);
        if (index >= 0) {
          this.images[index] = cloneImage(entry.before);
          this.selectedIds = [entry.before.id];
          saveImageSafely(entry.before);
        }
      }
      this.future.push(entry);
      return true;
    },
    redo() {
      const entry = this.future.pop();
      if (!entry) return false;
      if (entry.type === "create" && entry.after) {
        this.images.push(cloneImage(entry.after));
        this.selectedIds = [entry.after.id];
        saveImageSafely(entry.after);
      }
      if (entry.type === "delete" && entry.before) {
        this.images = this.images.filter((image) => image.id !== entry.before?.id);
        this.selectedIds = this.selectedIds.filter((id) => id !== entry.before?.id);
        deleteImageSafely(entry.before.id);
      }
      if (entry.type === "update" && entry.after) {
        const index = this.images.findIndex((image) => image.id === entry.after?.id);
        if (index >= 0) {
          this.images[index] = cloneImage(entry.after);
          this.selectedIds = [entry.after.id];
          saveImageSafely(entry.after);
        }
      }
      this.history.push(entry);
      return true;
    },
  },
});
