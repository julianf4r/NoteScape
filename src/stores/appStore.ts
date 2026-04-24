import { defineStore } from "pinia";
import type { AppData } from "../types";
import { createDefaultData, debounce, loadData, parseData, saveData, switchDatabase } from "../utils/storage";
import { useCanvasStore } from "./canvasStore";
import { useNoteStore } from "./noteStore";
import { useSettingsStore } from "./settingsStore";
import { useTagStore } from "./tagStore";

const debouncedSave = debounce((data: AppData) => {
  void saveData(data);
}, 300);

export const useAppStore = defineStore("app", {
  state: () => ({
    loaded: false,
    databasePath: "",
    loadError: "",
  }),
  actions: {
    applyData(data: AppData) {
      useCanvasStore().setCanvases(data.canvases);
      useNoteStore().setNotes(data.notes);
      useTagStore().setTags(data.tags);
      useSettingsStore().setSettings(data.settings);
    },
    async load() {
      try {
        const result = await loadData();
        this.databasePath = result.db_path;
        this.applyData(parseData(result.data));
      } catch (error) {
        this.loadError = error instanceof Error ? error.message : String(error);
        this.applyData(createDefaultData());
      }
      this.loaded = true;
    },
    snapshot(): AppData {
      return {
        version: 1,
        canvases: useCanvasStore().canvases,
        notes: useNoteStore().notes,
        tags: useTagStore().tags,
        settings: useSettingsStore().settings,
      };
    },
    persist(immediate = false) {
      const settings = useSettingsStore().settings;
      if (!settings.autoSave && !immediate) return;
      const data = this.snapshot();
      if (immediate) void saveData(data);
      else debouncedSave(data);
    },
    resetSampleData() {
      const data = createDefaultData();
      this.applyData(data);
      void saveData(data);
    },
    exportData() {
      return JSON.stringify(this.snapshot(), null, 2);
    },
    importData(raw: string) {
      const data = JSON.parse(raw) as AppData;
      if (!Array.isArray(data.canvases) || !Array.isArray(data.notes)) throw new Error("数据格式不正确");
      this.applyData(data);
      void saveData(data);
    },
    async changeDatabase(dbPath: string) {
      const result = await switchDatabase(dbPath, this.snapshot());
      this.databasePath = result.db_path;
      this.applyData(parseData(result.data));
    },
  },
});
