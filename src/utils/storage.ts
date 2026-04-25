import type { AppData, AppSettings, CanvasItem, NoteColor, StickyNote, TagItem } from "../types";
import { invoke } from "@tauri-apps/api/core";

export interface DatabaseLoadResult {
  data: string;
  db_path: string;
}

const now = () => new Date().toISOString();

const baseNote = (
  id: string,
  content: string,
  x: number,
  y: number,
  width: number,
  height: number,
  color: NoteColor,
  rotation: number,
  zIndex: number,
  tags: string[] = [],
  fontSize = 19,
): StickyNote => ({
  id,
  canvasId: "canvas-default",
  content,
  x,
  y,
  width,
  height,
  color,
  rotation,
  zIndex,
  pinned: false,
  tags,
  fontSize,
  fontWeight: "normal",
  textAlign: "left",
  decoration: "tape",
  createdAt: now(),
  updatedAt: now(),
});

export const defaultSettings: AppSettings = {
  theme: "light",
  defaultFontSize: 18,
  showGrid: true,
  randomRotation: true,
  noteShadow: true,
  autoSave: true,
};

export function createDefaultData(): AppData {
  const createdAt = now();
  return {
    version: 1,
    canvases: [
      {
        id: "canvas-default",
        name: "默认画布",
        createdAt,
        updatedAt: createdAt,
        deletedAt: null,
      },
    ],
    tags: [],
    settings: defaultSettings,
    notes: [baseNote("note-default", "", 360, 260, 260, 220, "yellow", -1.5, 1, [], defaultSettings.defaultFontSize)],
  };
}

export function parseData(raw: string): AppData {
  try {
    const parsed = JSON.parse(raw) as AppData;
    if (!parsed.version || !Array.isArray(parsed.canvases) || !Array.isArray(parsed.notes)) {
      throw new Error("Invalid app data");
    }
    return parsed;
  } catch {
    return createDefaultData();
  }
}

export async function loadData(): Promise<DatabaseLoadResult> {
  return invoke<DatabaseLoadResult>("load_app_data", {
    defaultData: JSON.stringify(createDefaultData()),
  });
}

export async function saveData(data: AppData) {
  await invoke("save_app_data", { data: JSON.stringify(data) });
}

export async function saveCanvasData(canvas: CanvasItem) {
  await invoke("save_canvas", { canvas: JSON.stringify(canvas) });
}

export async function deleteCanvasData(id: string, deletedAt: string) {
  await invoke("delete_canvas", { id, deletedAt });
}

export async function restoreCanvasData(id: string, updatedAt: string) {
  await invoke("restore_canvas", { id, updatedAt });
}

export async function removeCanvasForeverData(id: string) {
  await invoke("remove_canvas_forever", { id });
}

export async function saveNoteData(note: StickyNote) {
  await invoke("save_note", { note: JSON.stringify(note) });
}

export async function deleteNoteData(id: string) {
  await invoke("delete_note", { id });
}

export async function deleteNotesByCanvasData(canvasId: string) {
  await invoke("delete_notes_by_canvas", { canvasId });
}

export async function saveTagData(tag: TagItem) {
  await invoke("save_tag", { tag: JSON.stringify(tag) });
}

export async function deleteTagData(id: string) {
  await invoke("delete_tag", { id });
}

export async function saveSettingsData(settings: AppSettings) {
  await invoke("save_app_settings", { settings: JSON.stringify(settings) });
}

export async function backupDatabase(backupPath: string) {
  await invoke("backup_database", { backupPath });
}

export function reportPersistenceError(scope: string, error: unknown) {
  window.dispatchEvent(
    new CustomEvent("persistence-error", {
      detail: {
        message: `${scope}失败：${error instanceof Error ? error.message : String(error)}`,
      },
    }),
  );
}

export async function switchDatabase(dbPath: string): Promise<DatabaseLoadResult> {
  return invoke<DatabaseLoadResult>("set_database_path", {
    dbPath,
  });
}

export async function createDatabase(dbPath: string): Promise<DatabaseLoadResult> {
  return invoke<DatabaseLoadResult>("create_database", {
    dbPath,
    initialData: JSON.stringify(createDefaultData()),
  });
}

export function debounce<T extends (...args: never[]) => void>(fn: T, wait = 300) {
  let timer: number | undefined;
  return (...args: Parameters<T>) => {
    window.clearTimeout(timer);
    timer = window.setTimeout(() => fn(...args), wait);
  };
}
