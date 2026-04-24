import { defineStore } from "pinia";
import type { AppSettings } from "../types";
import { defaultSettings } from "../utils/storage";
import { saveSettingsData } from "../utils/storage";

export const useSettingsStore = defineStore("settings", {
  state: () => ({
    settings: { ...defaultSettings } as AppSettings,
    panelOpen: false,
  }),
  actions: {
    setSettings(settings: AppSettings) {
      this.settings = { ...settings };
    },
    updateSettings(patch: Partial<AppSettings>) {
      this.settings = { ...this.settings, ...patch };
      void saveSettingsData(this.settings);
    },
    togglePanel() {
      this.panelOpen = !this.panelOpen;
    },
  },
});
