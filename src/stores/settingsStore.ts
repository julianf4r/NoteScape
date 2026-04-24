import { defineStore } from "pinia";
import type { AppSettings } from "../types";
import { defaultSettings } from "../utils/storage";
import { reportPersistenceError, saveSettingsData } from "../utils/storage";

function applyTheme(theme: AppSettings["theme"]) {
  const dark = theme === "dark" || (theme === "system" && window.matchMedia?.("(prefers-color-scheme: dark)").matches);
  document.documentElement.dataset.theme = dark ? "dark" : "light";
}

export const useSettingsStore = defineStore("settings", {
  state: () => ({
    settings: { ...defaultSettings } as AppSettings,
    panelOpen: false,
  }),
  actions: {
    setSettings(settings: AppSettings) {
      this.settings = { ...settings };
      applyTheme(this.settings.theme);
    },
    updateSettings(patch: Partial<AppSettings>) {
      this.settings = { ...this.settings, ...patch };
      if (patch.theme) applyTheme(this.settings.theme);
      void saveSettingsData(this.settings).catch((error) => reportPersistenceError("保存设置", error));
    },
    togglePanel() {
      this.panelOpen = !this.panelOpen;
    },
  },
});
