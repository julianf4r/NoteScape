<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { AlertTriangle } from "lucide-vue-next";
import { useFeedbackStore } from "../stores/feedbackStore";

const feedback = useFeedbackStore();
const confirmButton = ref<HTMLButtonElement | null>(null);
const request = computed(() => feedback.confirmRequest);

watch(
  request,
  (value) => {
    if (!value) return;
    void nextTick(() => confirmButton.value?.focus());
  },
  { immediate: true },
);
</script>

<template>
  <Teleport to="body">
    <Transition name="confirm-fade">
      <div v-if="request" class="confirm-overlay" @click.self="feedback.resolveConfirm(false)" @keydown.esc.stop.prevent="feedback.resolveConfirm(false)">
        <div class="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="confirm-title" tabindex="-1">
          <div class="confirm-icon">
            <AlertTriangle :size="22" />
          </div>
          <div class="confirm-content">
            <h2 id="confirm-title">确认操作</h2>
            <p>{{ request.message }}</p>
          </div>
          <div class="confirm-actions">
            <button class="secondary" @click="feedback.resolveConfirm(false)">取消</button>
            <button ref="confirmButton" class="danger" @click="feedback.resolveConfirm(true)">确认</button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.confirm-overlay {
  position: fixed;
  inset: 0;
  z-index: 3000;
  display: grid;
  place-items: center;
  padding: 24px;
  background: rgba(31, 41, 55, 0.24);
  backdrop-filter: blur(2px);
}

.confirm-dialog {
  width: min(390px, calc(100vw - 48px));
  display: grid;
  grid-template-columns: 42px 1fr;
  gap: 14px;
  padding: 18px;
  color: #1f2937;
  background: #fffdf7;
  border: 1px solid rgba(229, 231, 235, 0.92);
  border-radius: 10px;
  box-shadow: 0 24px 58px rgba(15, 23, 42, 0.22), 0 4px 12px rgba(15, 23, 42, 0.1);
  outline: none;
}

.confirm-icon {
  width: 42px;
  height: 42px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: #b45309;
  background: #ffefbf;
  border: 1px solid rgba(245, 158, 11, 0.28);
  border-radius: 8px;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.68);
}

.confirm-content {
  min-width: 0;
}

.confirm-content h2 {
  margin: 1px 0 7px;
  font-size: 16px;
  font-weight: 800;
  letter-spacing: 0;
}

.confirm-content p {
  margin: 0;
  line-height: 1.6;
  color: #4b5563;
  font-size: 14px;
}

.confirm-actions {
  grid-column: 1 / -1;
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding-top: 6px;
}

.confirm-actions button {
  min-width: 76px;
  height: 34px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0 13px;
  border-radius: 7px;
  font-weight: 700;
}

.confirm-actions .secondary {
  color: #4b5563;
  background: #f3f4f6;
  border: 1px solid #e5e7eb;
}

.confirm-actions .secondary:hover {
  background: #e9edf2;
}

.confirm-actions .danger {
  color: #fff;
  background: #dc2626;
  box-shadow: 0 6px 14px rgba(220, 38, 38, 0.22);
}

.confirm-actions .danger:hover {
  background: #b91c1c;
}

.confirm-actions button:focus-visible {
  outline: 2px solid rgba(59, 130, 246, 0.48);
  outline-offset: 2px;
}

.confirm-fade-enter-active,
.confirm-fade-leave-active {
  transition: opacity 0.14s ease;
}

.confirm-fade-enter-active .confirm-dialog,
.confirm-fade-leave-active .confirm-dialog {
  transition: transform 0.14s ease;
}

.confirm-fade-enter-from,
.confirm-fade-leave-to {
  opacity: 0;
}

.confirm-fade-enter-from .confirm-dialog,
.confirm-fade-leave-to .confirm-dialog {
  transform: translateY(4px) scale(0.98);
}
</style>
