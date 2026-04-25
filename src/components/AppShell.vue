<script setup lang="ts">
import { computed, onMounted, onUnmounted, watch } from "vue";
import CanvasBoard from "./CanvasBoard.vue";
import SettingsPanel from "./SettingsPanel.vue";
import Sidebar from "./Sidebar.vue";
import ToastHost from "./ToastHost.vue";
import { useAppStore } from "../stores/appStore";
import { useCanvasStore } from "../stores/canvasStore";
import { useNoteStore } from "../stores/noteStore";
import { useSettingsStore } from "../stores/settingsStore";
import { useTagStore } from "../stores/tagStore";
import { useFeedbackStore } from "../stores/feedbackStore";

const appStore = useAppStore();
const canvasStore = useCanvasStore();
const noteStore = useNoteStore();
const tagStore = useTagStore();
const settingsStore = useSettingsStore();
const feedbackStore = useFeedbackStore();

const noteTags = computed(() => noteStore.notes.flatMap((note) => note.tags));

function isTextInputTarget(target: EventTarget | null) {
  if (!(target instanceof HTMLElement)) return false;
  return Boolean(target.closest(".tiptap, input, textarea, select, [contenteditable='true']"));
}

function onKeydown(event: KeyboardEvent) {
  const selected = noteStore.selectedIds[0];
  const isEditing = Boolean(noteStore.editingId);
  const textInputTarget = isTextInputTarget(event.target);
  if (textInputTarget && !(event.ctrlKey && event.key.toLowerCase() === "s") && event.key !== "Escape") return;
  if (event.ctrlKey && event.key.toLowerCase() === "f") {
    event.preventDefault();
    window.dispatchEvent(new CustomEvent("focus-search"));
  }
  if (event.ctrlKey && event.key.toLowerCase() === "s") {
    event.preventDefault();
    appStore.persist(true);
  }
  if (event.ctrlKey && event.key.toLowerCase() === "z") {
    event.preventDefault();
    noteStore.undo();
  }
  if (event.ctrlKey && event.key.toLowerCase() === "y") {
    event.preventDefault();
    noteStore.redo();
  }
  if (event.ctrlKey && event.key.toLowerCase() === "d" && selected) {
    event.preventDefault();
    if (noteStore.selectedIds.length > 1) noteStore.duplicateSelected();
    else noteStore.duplicateNote(selected);
  }
  if (event.ctrlKey && event.key.toLowerCase() === "c" && selected && !isEditing) {
    event.preventDefault();
    noteStore.copySelected();
  }
  if (event.ctrlKey && event.key.toLowerCase() === "n" && canvasStore.currentCanvasId) {
    event.preventDefault();
    noteStore.createNote(canvasStore.currentCanvasId, 360, 260, settingsStore.settings.defaultFontSize, settingsStore.settings.randomRotation);
  }
  if (event.key === "Delete" && selected && !isEditing) {
    if (noteStore.selectedIds.length > 1) noteStore.deleteSelected();
    else noteStore.deleteNote(selected);
  }
  if (event.key === "Escape") {
    noteStore.clearSelection();
  }
}

onMounted(() => {
  appStore.load();
  window.addEventListener("keydown", onKeydown);
  window.addEventListener("persistence-error", onPersistenceError);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
  window.removeEventListener("persistence-error", onPersistenceError);
});

function onPersistenceError(event: Event) {
  const message = (event as CustomEvent<{ message: string }>).detail?.message ?? "保存失败";
  feedbackStore.notify(message, "error");
}

watch(
  noteTags,
  (tags) => {
    if (appStore.loaded) tagStore.recalculateCounts(tags);
  },
  { immediate: true },
);
</script>

<template>
  <div class="app-shell">
    <Sidebar />
    <main class="workspace">
      <CanvasBoard />
    </main>
    <Transition name="settings-drawer">
      <SettingsPanel v-if="settingsStore.panelOpen" />
    </Transition>
    <ToastHost />
  </div>
</template>

<style scoped>
.app-shell {
  width: 100vw;
  height: 100vh;
  display: flex;
  overflow: hidden;
  background: var(--app-bg);
}

.workspace {
  position: relative;
  flex: 1;
  min-width: 0;
  overflow: hidden;
}
</style>
