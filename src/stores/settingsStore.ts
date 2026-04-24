import { defineStore } from "pinia";
import type { AppSettings } from "../types";
import { defaultSettings } from "../utils/storage";
import { reportPersistenceError, saveSettingsData } from "../utils/storage";

export const useSettingsStore = defineStore("settings", {
  state: () => ({
    settings: { ...defaultSettings } as AppSettings,
    panelOpen: false,
  }),
  actions: {
    setSettings(settings: AppSettings) {
      this.settings = { ...settings, theme: "light" };
    },
    updateSettings(patch: Partial<AppSettings>) {
      this.settings = { ...this.settings, ...patch, theme: "light" };
      void saveSettingsData(this.settings).catch((error) => reportPersistenceError("保存设置", error));
    },
    togglePanel() {
      this.panelOpen = !this.panelOpen;
    },
  },
});
