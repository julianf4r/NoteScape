import { defineStore } from "pinia";
import { nanoid } from "nanoid";
import type { HistoryEntry, StickyNote } from "../types";
import { deleteNoteData, deleteNotesByCanvasData, reportPersistenceError, saveNoteData } from "../utils/storage";
import { randomNoteStyle } from "../utils/notePresets";

const now = () => new Date().toISOString();
const randomRotation = () => Math.round((Math.random() * 4 - 2) * 10) / 10;
const hasMeaningfulChange = (before: StickyNote, after: StickyNote) =>
  JSON.stringify({ ...before, updatedAt: undefined }) !== JSON.stringify({ ...after, updatedAt: undefined });
const pinnedValue = (note: StickyNote) => note.pinned === true;
const saveNoteSafely = (note: StickyNote) => {
  void saveNoteData(note).catch((error) => reportPersistenceError("保存便签", error));
};
const deleteNoteSafely = (id: string) => {
  void deleteNoteData(id).catch((error) => reportPersistenceError("删除便签", error));
};

export const useNoteStore = defineStore("note", {
  state: () => ({
    notes: [] as StickyNote[],
    selectedIds: [] as string[],
    editingId: "",
    history: [] as HistoryEntry[],
    future: [] as HistoryEntry[],
  }),
  getters: {
    selectedNote: (state) => state.notes.find((note) => note.id === state.selectedIds[0]),
    maxZ: (state) => Math.max(1, ...state.notes.map((note) => note.zIndex)),
  },
  actions: {
    setNotes(notes: StickyNote[]) {
      this.notes = notes;
      this.selectedIds = [];
      this.editingId = "";
    },
    maxZForPinned(pinned: boolean) {
      return Math.max(0, ...this.notes.filter((note) => pinnedValue(note) === pinned).map((note) => note.zIndex));
    },
    normalizeZIndexes() {
      const ordered = [...this.notes].sort((a, b) => {
        if (pinnedValue(a) !== pinnedValue(b)) return Number(pinnedValue(a)) - Number(pinnedValue(b));
        if (a.zIndex !== b.zIndex) return a.zIndex - b.zIndex;
        return a.createdAt.localeCompare(b.createdAt);
      });
      const changed: StickyNote[] = [];
      ordered.forEach((note, index) => {
        const nextZ = index + 1;
        if (note.zIndex !== nextZ) {
          note.zIndex = nextZ;
          note.updatedAt = now();
          changed.push(note);
        }
      });
      return changed;
    },
    notesForCanvas(canvasId: string, tagId = "", query = "") {
      const normalized = query.trim().toLowerCase();
      return this.notes.filter((note) => {
        const canvasMatch = note.canvasId === canvasId;
        const tagMatch = !tagId || note.tags.includes(tagId);
        const queryMatch = !normalized || note.content.toLowerCase().includes(normalized) || note.title?.toLowerCase().includes(normalized);
        return canvasMatch && tagMatch && queryMatch;
      });
    },
    addHistory(entry: Omit<HistoryEntry, "id" | "timestamp">) {
      this.history.push({ ...entry, id: nanoid(), timestamp: now() });
      if (this.history.length > 50) this.history.shift();
      this.future = [];
    },
    createNote(canvasId: string, x: number, y: number, fontSize: number, rotationEnabled = true, content = "", zIndex?: number) {
      const style = randomNoteStyle();
      const note: StickyNote = {
        id: nanoid(),
        canvasId,
        content,
        x,
        y,
        width: style.width,
        height: style.height,
        color: style.color,
        rotation: rotationEnabled ? randomRotation() : 0,
        zIndex: zIndex ?? this.maxZForPinned(false) + 1,
        pinned: false,
        tags: [],
        fontSize,
        fontWeight: "normal",
        textAlign: "left",
        decoration: style.decoration,
        createdAt: now(),
        updatedAt: now(),
      };
      this.notes.push(note);
      if (zIndex === undefined) this.normalizeZIndexes().forEach(saveNoteSafely);
      this.selectedIds = [note.id];
      this.addHistory({ type: "create", after: { ...note } });
      saveNoteSafely(note);
      return note;
    },
    updateNote(id: string, patch: Partial<StickyNote>, track = true) {
      const note = this.notes.find((item) => item.id === id);
      if (!note) return;
      const before = { ...note };
      Object.assign(note, patch, { updatedAt: now() });
      if (track && hasMeaningfulChange(before, note)) this.addHistory({ type: "update", before, after: { ...note } });
      saveNoteSafely(note);
    },
    commitNoteChange(before: StickyNote, patch: Partial<StickyNote>) {
      const note = this.notes.find((item) => item.id === before.id);
      if (!note) return;
      Object.assign(note, patch, { updatedAt: now() });
      if (hasMeaningfulChange(before, note)) this.addHistory({ type: "update", before, after: { ...note } });
      saveNoteSafely(note);
    },
    patchNoteLive(id: string, patch: Partial<StickyNote>) {
      const note = this.notes.find((item) => item.id === id);
      if (note) Object.assign(note, patch);
    },
    deleteNote(id: string) {
      const note = this.notes.find((item) => item.id === id);
      if (!note) return;
      this.notes = this.notes.filter((item) => item.id !== id);
      this.selectedIds = this.selectedIds.filter((item) => item !== id);
      this.addHistory({ type: "delete", before: { ...note } });
      deleteNoteSafely(id);
    },
    removeNotesByCanvas(canvasId: string) {
      this.notes = this.notes.filter((note) => note.canvasId !== canvasId);
      this.selectedIds = this.selectedIds.filter((id) => this.notes.some((note) => note.id === id));
      if (this.editingId && !this.notes.some((note) => note.id === this.editingId)) this.editingId = "";
      void deleteNotesByCanvasData(canvasId).catch((error) => reportPersistenceError("删除画布便签", error));
    },
    removeTagFromAll(tagId: string) {
      this.notes.forEach((note) => {
        note.tags = note.tags.filter((id) => id !== tagId);
        note.updatedAt = now();
        saveNoteSafely(note);
      });
    },
    toggleTagForNote(noteId: string, tagId: string) {
      const note = this.notes.find((item) => item.id === noteId);
      if (!note) return;
      const before = { ...note, tags: [...note.tags] };
      const exists = note.tags.includes(tagId);
      note.tags = exists ? note.tags.filter((id) => id !== tagId) : [...note.tags, tagId];
      note.updatedAt = now();
      this.addHistory({ type: "update", before, after: { ...note, tags: [...note.tags] } });
      saveNoteSafely(note);
    },
    duplicateNote(id: string, zIndex?: number) {
      const note = this.notes.find((item) => item.id === id);
      if (!note) return;
      const copy = {
        ...note,
        id: nanoid(),
        x: note.x + 28,
        y: note.y + 28,
        zIndex: zIndex ?? this.maxZForPinned(pinnedValue(note)) + 1,
        createdAt: now(),
        updatedAt: now(),
      };
      this.notes.push(copy);
      if (zIndex === undefined) this.normalizeZIndexes().forEach(saveNoteSafely);
      this.selectedIds = [copy.id];
      this.addHistory({ type: "create", after: { ...copy } });
      saveNoteSafely(copy);
    },
    duplicateSelected(baseZ?: number) {
      const selected = this.notes.filter((note) => this.selectedIds.includes(note.id));
      if (!selected.length) return;
      const copies = selected.map((note, index) => ({
        ...note,
        id: nanoid(),
        x: note.x + 28 + index * 8,
        y: note.y + 28 + index * 8,
        zIndex: baseZ === undefined ? this.maxZForPinned(pinnedValue(note)) + index + 1 : baseZ + index,
        createdAt: now(),
        updatedAt: now(),
      }));
      this.notes.push(...copies);
      if (baseZ === undefined) this.normalizeZIndexes().forEach(saveNoteSafely);
      this.selectedIds = copies.map((note) => note.id);
      copies.forEach((note) => {
        this.addHistory({ type: "create", after: { ...note } });
        saveNoteSafely(note);
      });
    },
    deleteSelected() {
      const ids = [...this.selectedIds];
      ids.forEach((id) => this.deleteNote(id));
    },
    bringSelectedToFront() {
      const selected = this.notes.filter((note) => this.selectedIds.includes(note.id));
      if (!selected.length) return;
      const targetPinned = !selected.every((note) => pinnedValue(note));
      selected.forEach((note, index) => {
        const before = { ...note };
        note.pinned = targetPinned;
        note.zIndex = this.maxZForPinned(targetPinned) + index + 1;
        note.updatedAt = now();
        if (hasMeaningfulChange(before, note)) this.addHistory({ type: "update", before, after: { ...note } });
      });
      this.normalizeZIndexes().forEach(saveNoteSafely);
      selected.forEach(saveNoteSafely);
    },
    updateSelected(patch: Partial<StickyNote>) {
      this.selectedIds.forEach((id) => this.updateNote(id, patch));
    },
    toggleTagForSelected(tagId: string) {
      this.selectedIds.forEach((id) => this.toggleTagForNote(id, tagId));
    },
    moveSelectedBy(deltaX: number, deltaY: number, beforeNotes: StickyNote[]) {
      beforeNotes.forEach((before) => {
        const note = this.notes.find((item) => item.id === before.id);
        if (!note) return;
        note.x = before.x + deltaX;
        note.y = before.y + deltaY;
        note.updatedAt = now();
      });
    },
    commitSelectedMove(beforeNotes: StickyNote[]) {
      beforeNotes.forEach((before) => {
        const note = this.notes.find((item) => item.id === before.id);
        if (!note) return;
        if (hasMeaningfulChange(before, note)) this.addHistory({ type: "update", before, after: { ...note } });
        saveNoteSafely(note);
      });
    },
    bringToFront(id: string) {
      const note = this.notes.find((item) => item.id === id);
      if (!note) return;
      const before = { ...note };
      note.pinned = !pinnedValue(note);
      note.zIndex = this.maxZForPinned(note.pinned) + 1;
      note.updatedAt = now();
      if (hasMeaningfulChange(before, note)) this.addHistory({ type: "update", before, after: { ...note } });
      this.normalizeZIndexes().forEach(saveNoteSafely);
      saveNoteSafely(note);
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
      this.selectedIds = Array.from(new Set(ids));
    },
    clearSelection() {
      this.selectedIds = [];
      this.editingId = "";
    },
    stopEditing() {
      this.editingId = "";
    },
    undo() {
      const entry = this.history.pop();
      if (!entry) return;
      if (entry.type === "create" && entry.after) {
        this.notes = this.notes.filter((note) => note.id !== entry.after?.id);
        deleteNoteSafely(entry.after.id);
      }
      if (entry.type === "delete" && entry.before) {
        this.notes.push(entry.before);
        saveNoteSafely(entry.before);
      }
      if (entry.type === "update" && entry.before) {
        const index = this.notes.findIndex((note) => note.id === entry.before?.id);
        if (index >= 0) {
          this.notes[index] = entry.before;
          saveNoteSafely(entry.before);
        }
      }
      this.future.push(entry);
    },
    redo() {
      const entry = this.future.pop();
      if (!entry) return;
      if (entry.type === "create" && entry.after) {
        this.notes.push(entry.after);
        saveNoteSafely(entry.after);
      }
      if (entry.type === "delete" && entry.before) {
        this.notes = this.notes.filter((note) => note.id !== entry.before?.id);
        deleteNoteSafely(entry.before.id);
      }
      if (entry.type === "update" && entry.after) {
        const index = this.notes.findIndex((note) => note.id === entry.after?.id);
        if (index >= 0) {
          this.notes[index] = entry.after;
          saveNoteSafely(entry.after);
        }
      }
      this.history.push(entry);
    },
  },
});
