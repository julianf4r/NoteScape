import { defineStore } from "pinia";
import type { AppSettings } from "../types";
import { defaultSettings, normalizeSettings } from "../utils/storage";
import { reportPersistenceError, saveSettingsData } from "../utils/storage";

export const useSettingsStore = defineStore("settings", {
  state: () => ({
    settings: { ...defaultSettings } as AppSettings,
    panelOpen: false,
  }),
  actions: {
    setSettings(settings: AppSettings) {
      this.settings = normalizeSettings(settings);
    },
    updateSettings(patch: Partial<AppSettings>) {
      this.settings = normalizeSettings({ ...this.settings, ...patch });
      void saveSettingsData(this.settings).catch((error) => reportPersistenceError("保存设置", error));
    },
    togglePanel() {
      this.panelOpen = !this.panelOpen;
    },
  },
});
