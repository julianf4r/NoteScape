import type { AppData, AppSettings, CanvasImage, CanvasItem, DrawingItem, StickyNote, TagItem } from "../types";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";

export interface DatabaseLoadResult {
  data: string;
  db_path: string;
}

export const defaultSettings: AppSettings = {
  theme: "light",
  defaultFontSize: 18,
  showGrid: true,
  randomRotation: true,
  noteShadow: true,
  autoSave: true,
  chineseFontFamily: "Xiaolai, Microsoft YaHei",
  englishFontFamily: "Segoe Print, Comic Sans MS",
  monospaceFontFamily: "Consolas, Cascadia Mono, monospace",
  imageLibraryPath: "",
};

export function normalizeSettings(settings?: Partial<AppSettings>): AppSettings {
  return {
    ...defaultSettings,
    ...settings,
    theme: "light",
  };
}

export function parseData(raw: string): AppData {
  const parsed = JSON.parse(raw) as AppData;
  if (!parsed.version || !Array.isArray(parsed.canvases) || !Array.isArray(parsed.notes)) {
    throw new Error("数据格式不正确");
  }
  parsed.settings = normalizeSettings(parsed.settings);
  parsed.drawings = Array.isArray(parsed.drawings) ? parsed.drawings : [];
  parsed.images = Array.isArray(parsed.images) ? parsed.images : [];
  parsed.canvases = normalizeCanvases(parsed.canvases);
  parsed.tags = normalizeTags(Array.isArray(parsed.tags) ? parsed.tags : []);
  return parsed;
}

function normalizeCanvases(canvases: CanvasItem[]): CanvasItem[] {
  return canvases
    .map((canvas, index) => ({
      ...canvas,
      sortOrder: Number.isFinite(canvas.sortOrder) ? canvas.sortOrder : index,
    }))
    .sort((a, b) => a.sortOrder - b.sortOrder || a.createdAt.localeCompare(b.createdAt));
}

function normalizeTags(tags: TagItem[]): TagItem[] {
  return tags
    .map((tag, index) => ({
      ...tag,
      sortOrder: Number.isFinite(tag.sortOrder) ? tag.sortOrder : index,
    }))
    .sort((a, b) => a.sortOrder - b.sortOrder || a.createdAt.localeCompare(b.createdAt));
}

export async function loadData(): Promise<DatabaseLoadResult> {
  return invoke<DatabaseLoadResult>("load_app_data");
}

export async function resetDatabaseToDefault(): Promise<DatabaseLoadResult> {
  return invoke<DatabaseLoadResult>("reset_database_to_default");
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

export async function saveDrawingData(drawing: DrawingItem) {
  await invoke("save_drawing", { drawing: JSON.stringify(drawing) });
}

export async function deleteDrawingData(id: string) {
  await invoke("delete_drawing", { id });
}

export async function deleteDrawingsByCanvasData(canvasId: string) {
  await invoke("delete_drawings_by_canvas", { canvasId });
}

export async function saveImageData(image: CanvasImage) {
  await invoke("save_image", { image: JSON.stringify(image) });
}

export async function deleteImageData(id: string) {
  await invoke("delete_image", { id });
}

export async function deleteImagesByCanvasData(canvasId: string) {
  await invoke("delete_images_by_canvas", { canvasId });
}

export interface ImportedImageFile {
  fileName: string;
  originalName: string;
  path: string;
}

export async function importImageFile(sourcePath: string, libraryPath: string): Promise<ImportedImageFile> {
  return invoke<ImportedImageFile>("import_image_file", { sourcePath, libraryPath });
}

export async function resolveImagePath(fileName: string, libraryPath: string): Promise<string | null> {
  return invoke<string | null>("resolve_image_path", { fileName, libraryPath });
}

export async function defaultImageLibraryPath(): Promise<string> {
  return invoke<string>("default_image_library");
}

export function imageFileUrl(path: string) {
  return convertFileSrc(path);
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

export async function exportJsonFile(path: string, data: string) {
  await invoke("export_json_file", { path, data });
}

export async function importJsonFile(path: string): Promise<string> {
  return invoke<string>("import_json_file", { path });
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
  });
}

export function debounce<T extends (...args: never[]) => void>(fn: T, wait = 300) {
  let timer: number | undefined;
  return (...args: Parameters<T>) => {
    window.clearTimeout(timer);
    timer = window.setTimeout(() => fn(...args), wait);
  };
}
