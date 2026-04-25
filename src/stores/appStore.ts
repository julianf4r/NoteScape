import { defineStore } from "pinia";
import type { AppData } from "../types";
import { createDatabase, loadData, parseData, resetDatabaseToDefault, saveData, switchDatabase } from "../utils/storage";
import { useCanvasStore } from "./canvasStore";
import { useNoteStore } from "./noteStore";
import { useSettingsStore } from "./settingsStore";
import { useTagStore } from "./tagStore";
import { useFeedbackStore } from "./feedbackStore";

export const useAppStore = defineStore("app", {
  state: () => ({
    loaded: false,
    databaseReady: false,
    databasePath: "",
    loadError: "",
    saveStatus: "idle" as "idle" | "saving" | "saved" | "error",
    statusMessage: "",
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
        this.loadError = "";
        this.databaseReady = true;
        this.applyData(parseData(result.data));
      } catch (error) {
        this.loadError = error instanceof Error ? error.message : String(error);
        this.databaseReady = false;
        useFeedbackStore().notify(`数据库加载失败：${this.loadError}`, "error");
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
    async persist(immediate = false): Promise<boolean> {
      if (!this.databaseReady) {
        this.saveStatus = "error";
        this.statusMessage = this.loadError || "数据库未加载";
        if (immediate) useFeedbackStore().notify(`保存失败：${this.statusMessage}`, "error");
        return false;
      }
      const settings = useSettingsStore().settings;
      if (!settings.autoSave && !immediate) return true;
      this.saveStatus = "saving";
      try {
        await saveData(this.snapshot());
        this.saveStatus = "saved";
        this.statusMessage = immediate ? "已手动保存" : "已保存";
        if (immediate) useFeedbackStore().notify(this.statusMessage, "success");
        return true;
      } catch (error) {
        this.saveStatus = "error";
        this.statusMessage = error instanceof Error ? error.message : String(error);
        useFeedbackStore().notify(`保存失败：${this.statusMessage}`, "error");
        return false;
      }
    },
    exportData() {
      return JSON.stringify(this.snapshot(), null, 2);
    },
    async importData(raw: string) {
      const data = JSON.parse(raw) as AppData;
      if (!Array.isArray(data.canvases) || !Array.isArray(data.notes)) throw new Error("数据格式不正确");
      this.applyData(data);
      const saved = await this.persist(true);
      if (!saved) throw new Error(this.statusMessage || "保存失败");
    },
    async changeDatabase(dbPath: string) {
      const result = await switchDatabase(dbPath);
      this.databasePath = result.db_path;
      this.loadError = "";
      this.databaseReady = true;
      this.applyData(parseData(result.data));
    },
    async createDatabase(dbPath: string) {
      const result = await createDatabase(dbPath);
      this.databasePath = result.db_path;
      this.loadError = "";
      this.databaseReady = true;
      this.applyData(parseData(result.data));
    },
    async useDefaultDatabase() {
      const result = await resetDatabaseToDefault();
      this.databasePath = result.db_path;
      this.loadError = "";
      this.databaseReady = true;
      this.applyData(parseData(result.data));
    },
  },
});
