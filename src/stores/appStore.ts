import { defineStore } from "pinia";
import type { AppData } from "../types";
import { createDefaultData, debounce, loadData, saveData } from "../utils/storage";
import { useCanvasStore } from "./canvasStore";
import { useNoteStore } from "./noteStore";
import { useSettingsStore } from "./settingsStore";
import { useTagStore } from "./tagStore";

const debouncedSave = debounce((data: AppData) => saveData(data), 300);

export const useAppStore = defineStore("app", {
  state: () => ({
    loaded: false,
  }),
  actions: {
    load() {
      const data = loadData();
      useCanvasStore().setCanvases(data.canvases);
      useNoteStore().setNotes(data.notes);
      useTagStore().setTags(data.tags);
      useSettingsStore().setSettings(data.settings);
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
      if (immediate) saveData(data);
      else debouncedSave(data);
    },
    resetSampleData() {
      const data = createDefaultData();
      useCanvasStore().setCanvases(data.canvases);
      useNoteStore().setNotes(data.notes);
      useTagStore().setTags(data.tags);
      useSettingsStore().setSettings(data.settings);
      saveData(data);
    },
    exportData() {
      return JSON.stringify(this.snapshot(), null, 2);
    },
    importData(raw: string) {
      const data = JSON.parse(raw) as AppData;
      if (!Array.isArray(data.canvases) || !Array.isArray(data.notes)) throw new Error("数据格式不正确");
      useCanvasStore().setCanvases(data.canvases);
      useNoteStore().setNotes(data.notes);
      useTagStore().setTags(data.tags);
      useSettingsStore().setSettings(data.settings);
      saveData(data);
    },
  },
});
