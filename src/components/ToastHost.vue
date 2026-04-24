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
  color: #374151;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  box-shadow: 0 16px 34px rgba(15, 23, 42, 0.16);
  pointer-events: auto;
}

.toast.success {
  color: #166534;
  background: #dcfce7;
  border-color: #bbf7d0;
}

.toast.error {
  color: #991b1b;
  background: #fee2e2;
  border-color: #fecaca;
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
