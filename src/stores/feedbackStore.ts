import { defineStore } from "pinia";
import { nanoid } from "nanoid";

export interface ToastItem {
  id: string;
  type: "info" | "success" | "error";
  message: string;
}

export const useFeedbackStore = defineStore("feedback", {
  state: () => ({
    toasts: [] as ToastItem[],
  }),
  actions: {
    notify(message: string, type: ToastItem["type"] = "info") {
      const toast = { id: nanoid(), type, message };
      this.toasts.push(toast);
      window.setTimeout(() => this.dismiss(toast.id), 3200);
    },
    dismiss(id: string) {
      this.toasts = this.toasts.filter((toast) => toast.id !== id);
    },
    confirm(message: string) {
      return window.confirm(message);
    },
  },
});
