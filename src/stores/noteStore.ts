import { defineStore } from "pinia";
import { nanoid } from "nanoid";
import type { HistoryEntry, NoteColor, StickyNote } from "../types";

const now = () => new Date().toISOString();
const randomRotation = () => Math.round((Math.random() * 4 - 2) * 10) / 10;
const hasMeaningfulChange = (before: StickyNote, after: StickyNote) =>
  JSON.stringify({ ...before, updatedAt: undefined }) !== JSON.stringify({ ...after, updatedAt: undefined });

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
    notesForCanvas(canvasId: string, tagId = "", query = "") {
      const normalized = query.trim().toLowerCase();
      return this.notes.filter((note) => {
        const canvasMatch = note.canvasId === canvasId;
        const tagMatch = !tagId || note.tags.includes(tagId);
        const queryMatch = !normalized || note.content.toLowerCase().includes(normalized);
        return canvasMatch && tagMatch && queryMatch;
      });
    },
    addHistory(entry: Omit<HistoryEntry, "id" | "timestamp">) {
      this.history.push({ ...entry, id: nanoid(), timestamp: now() });
      if (this.history.length > 50) this.history.shift();
      this.future = [];
    },
    createNote(canvasId: string, x: number, y: number, color: NoteColor, fontSize: number, rotationEnabled = true, content = "新便签") {
      const note: StickyNote = {
        id: nanoid(),
        canvasId,
        content,
        x,
        y,
        width: 220,
        height: 180,
        color,
        rotation: rotationEnabled ? randomRotation() : 0,
        zIndex: this.maxZ + 1,
        tags: [],
        fontSize,
        fontWeight: "normal",
        textAlign: "left",
        createdAt: now(),
        updatedAt: now(),
      };
      this.notes.push(note);
      this.selectedIds = [note.id];
      this.addHistory({ type: "create", after: { ...note } });
      return note;
    },
    updateNote(id: string, patch: Partial<StickyNote>, track = true) {
      const note = this.notes.find((item) => item.id === id);
      if (!note) return;
      const before = { ...note };
      Object.assign(note, patch, { updatedAt: now() });
      if (track && hasMeaningfulChange(before, note)) this.addHistory({ type: "update", before, after: { ...note } });
    },
    commitNoteChange(before: StickyNote, patch: Partial<StickyNote>) {
      const note = this.notes.find((item) => item.id === before.id);
      if (!note) return;
      Object.assign(note, patch, { updatedAt: now() });
      if (hasMeaningfulChange(before, note)) this.addHistory({ type: "update", before, after: { ...note } });
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
    },
    removeNotesByCanvas(canvasId: string) {
      this.notes = this.notes.filter((note) => note.canvasId !== canvasId);
      this.selectedIds = this.selectedIds.filter((id) => this.notes.some((note) => note.id === id));
      if (this.editingId && !this.notes.some((note) => note.id === this.editingId)) this.editingId = "";
    },
    duplicateNote(id: string) {
      const note = this.notes.find((item) => item.id === id);
      if (!note) return;
      const copy = {
        ...note,
        id: nanoid(),
        x: note.x + 28,
        y: note.y + 28,
        zIndex: this.maxZ + 1,
        createdAt: now(),
        updatedAt: now(),
      };
      this.notes.push(copy);
      this.selectedIds = [copy.id];
      this.addHistory({ type: "create", after: { ...copy } });
    },
    bringToFront(id: string) {
      this.updateNote(id, { zIndex: this.maxZ + 1 });
    },
    select(id: string, additive = false) {
      this.selectedIds = additive ? Array.from(new Set([...this.selectedIds, id])) : [id];
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
      if (entry.type === "create" && entry.after) this.notes = this.notes.filter((note) => note.id !== entry.after?.id);
      if (entry.type === "delete" && entry.before) this.notes.push(entry.before);
      if (entry.type === "update" && entry.before) {
        const index = this.notes.findIndex((note) => note.id === entry.before?.id);
        if (index >= 0) this.notes[index] = entry.before;
      }
      this.future.push(entry);
    },
    redo() {
      const entry = this.future.pop();
      if (!entry) return;
      if (entry.type === "create" && entry.after) this.notes.push(entry.after);
      if (entry.type === "delete" && entry.before) this.notes = this.notes.filter((note) => note.id !== entry.before?.id);
      if (entry.type === "update" && entry.after) {
        const index = this.notes.findIndex((note) => note.id === entry.after?.id);
        if (index >= 0) this.notes[index] = entry.after;
      }
      this.history.push(entry);
    },
  },
});
