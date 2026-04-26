import { defineStore } from "pinia";
import { nanoid } from "nanoid";

export interface ToastItem {
  id: string;
  type: "info" | "success" | "error";
  message: string;
}

export interface ConfirmRequest {
  id: string;
  message: string;
  resolve: (confirmed: boolean) => void;
}

export const useFeedbackStore = defineStore("feedback", {
  state: () => ({
    toasts: [] as ToastItem[],
    confirmRequest: null as ConfirmRequest | null,
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
      if (this.confirmRequest) this.confirmRequest.resolve(false);
      return new Promise<boolean>((resolve) => {
        this.confirmRequest = {
          id: nanoid(),
          message,
          resolve,
        };
      });
    },
    resolveConfirm(confirmed: boolean) {
      const request = this.confirmRequest;
      if (!request) return;
      this.confirmRequest = null;
      request.resolve(confirmed);
    },
  },
});
