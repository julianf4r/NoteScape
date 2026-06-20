<script setup lang="ts">
import { computed, onMounted, onUnmounted, watch } from "vue";
import CanvasBoard from "./CanvasBoard.vue";
import ConfirmHost from "./ConfirmHost.vue";
import SettingsPanel from "./SettingsPanel.vue";
import Sidebar from "./Sidebar.vue";
import ToastHost from "./ToastHost.vue";
import { useAppStore } from "../stores/appStore";
import { useCanvasStore } from "../stores/canvasStore";
import { useNoteStore } from "../stores/noteStore";
import { useSettingsStore } from "../stores/settingsStore";
import { useTagStore } from "../stores/tagStore";
import { useFeedbackStore } from "../stores/feedbackStore";
import { useDrawingStore } from "../stores/drawingStore";
import { useImageStore } from "../stores/imageStore";

const appStore = useAppStore();
const canvasStore = useCanvasStore();
const noteStore = useNoteStore();
const tagStore = useTagStore();
const settingsStore = useSettingsStore();
const feedbackStore = useFeedbackStore();
const drawingStore = useDrawingStore();
const imageStore = useImageStore();
let systemThemeQuery: MediaQueryList | undefined;

const noteTags = computed(() => noteStore.notes.flatMap((note) => note.tags));
const globalMaxZ = computed(() =>
  Math.max(
    0,
    ...noteStore.notes.map((note) => note.zIndex),
    ...drawingStore.drawings.map((drawing) => drawing.zIndex),
    ...imageStore.images.map((image) => image.zIndex),
  ),
);

function isTextInputTarget(target: EventTarget | null) {
  if (!(target instanceof HTMLElement)) return false;
  return Boolean(target.closest(".tiptap, input, textarea, select, [contenteditable='true']"));
}

function onKeydown(event: KeyboardEvent) {
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
  if (event.ctrlKey && event.key.toLowerCase() === "n" && canvasStore.currentCanvasId) {
    event.preventDefault();
    noteStore.createNote(canvasStore.currentCanvasId, 360, 260, settingsStore.settings.defaultFontSize, settingsStore.settings.randomRotation, "", globalMaxZ.value + 1);
  }
}

function applyTheme() {
  const requestedTheme = settingsStore.settings.theme;
  const resolvedTheme = requestedTheme === "system"
    ? (systemThemeQuery?.matches ? "dark" : "light")
    : requestedTheme;
  document.documentElement.dataset.theme = resolvedTheme;
  document.documentElement.style.colorScheme = resolvedTheme;
}

onMounted(() => {
  systemThemeQuery = window.matchMedia("(prefers-color-scheme: dark)");
  systemThemeQuery.addEventListener("change", applyTheme);
  applyTheme();
  appStore.load();
  window.addEventListener("keydown", onKeydown);
  window.addEventListener("persistence-error", onPersistenceError);
});

onUnmounted(() => {
  systemThemeQuery?.removeEventListener("change", applyTheme);
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
    if (appStore.databaseReady) tagStore.recalculateCounts(tags);
  },
  { immediate: true },
);

watch(
  () => settingsStore.settings.theme,
  applyTheme,
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
    <ConfirmHost />
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
