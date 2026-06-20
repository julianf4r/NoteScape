<script setup lang="ts">
import { X } from "lucide-vue-next";
import { useFeedbackStore } from "../stores/feedbackStore";

const feedback = useFeedbackStore();
</script>

<template>
  <div class="toast-host">
    <div v-for="toast in feedback.toasts" :key="toast.id" class="toast" :class="toast.type">
      <span>{{ toast.message }}</span>
      <button @click="feedback.dismiss(toast.id)"><X :size="14" /></button>
    </div>
  </div>
</template>

<style scoped>
.toast-host {
  position: fixed;
  right: 18px;
  bottom: 18px;
  z-index: 2000;
  display: flex;
  flex-direction: column;
  gap: 8px;
  pointer-events: none;
}

.toast {
  min-width: 220px;
  max-width: 360px;
  display: grid;
  grid-template-columns: 1fr 24px;
  align-items: center;
  gap: 8px;
  padding: 10px 10px 10px 12px;
  color: var(--text-primary);
  background: var(--surface-raised);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: var(--shadow-lg);
  pointer-events: auto;
}

.toast.success {
  color: #86efac;
  background: #183d2a;
  border-color: #286440;
}

.toast.error {
  color: var(--danger-text);
  background: var(--danger-bg);
  border-color: var(--danger-border);
}

:global(:root[data-theme="light"]) .toast.success {
  color: #166534;
  background: #dcfce7;
  border-color: #bbf7d0;
}

button {
  width: 24px;
  height: 24px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: inherit;
  background: transparent;
  border-radius: 5px;
}
</style>
