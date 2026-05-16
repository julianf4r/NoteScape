<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ImageOff } from "lucide-vue-next";
import type { CanvasImage, NoteDecoration } from "../types";
import { imageFileUrl, resolveImagePath } from "../utils/storage";

const props = defineProps<{
  image: CanvasImage;
  selected: boolean;
  scale: number;
  libraryPath: string;
  panMode: boolean;
}>();

const emit = defineEmits<{
  select: [event: MouseEvent];
  update: [patch: Partial<CanvasImage> & { __before?: CanvasImage }, track?: boolean];
  live: [patch: Partial<CanvasImage>];
  delete: [];
  context: [event: MouseEvent];
}>();

const imageUrl = ref("");
const missing = ref(false);
const dragStart = ref<{ x: number; y: number; before: CanvasImage }>();
const resizeStart = ref<{ x: number; y: number; before: CanvasImage }>();

const decoration = computed<NoteDecoration>(() => props.image.decoration ?? "none");

const style = computed(() => ({
  left: `${props.image.x}px`,
  top: `${props.image.y}px`,
  width: `${props.image.width}px`,
  height: `${props.image.height}px`,
  zIndex: (props.image.pinned ? 100000 : 0) + props.image.zIndex,
  transform: `rotate(${props.image.rotation}deg)`,
}));

watch(
  () => [props.image.fileName, props.libraryPath],
  resolveImage,
  { immediate: true },
);

async function resolveImage() {
  const path = await resolveImagePath(props.image.fileName, props.libraryPath);
  missing.value = !path;
  imageUrl.value = path ? imageFileUrl(path) : "";
}

function startDrag(event: MouseEvent) {
  if (props.panMode || event.button === 1) {
    emit("select", event);
    return;
  }
  if ((event.target as HTMLElement).closest(".resize-handle")) return;
  emit("select", event);
  dragStart.value = { x: event.clientX, y: event.clientY, before: { ...props.image } };
  window.addEventListener("mousemove", drag);
  window.addEventListener("mouseup", endDrag, { once: true });
}

function drag(event: MouseEvent) {
  if (!dragStart.value) return;
  emit("live", {
    x: dragStart.value.before.x + (event.clientX - dragStart.value.x) / props.scale,
    y: dragStart.value.before.y + (event.clientY - dragStart.value.y) / props.scale,
  });
}

function endDrag() {
  window.removeEventListener("mousemove", drag);
  if (dragStart.value) emit("update", { x: props.image.x, y: props.image.y, __before: dragStart.value.before }, true);
  dragStart.value = undefined;
}

function startResize(event: MouseEvent) {
  event.preventDefault();
  event.stopPropagation();
  resizeStart.value = { x: event.clientX, y: event.clientY, before: { ...props.image } };
  window.addEventListener("mousemove", resize);
  window.addEventListener("mouseup", endResize, { once: true });
}

function resize(event: MouseEvent) {
  if (!resizeStart.value) return;
  emit("live", {
    width: Math.min(1000, Math.max(80, resizeStart.value.before.width + (event.clientX - resizeStart.value.x) / props.scale)),
    height: Math.min(1000, Math.max(80, resizeStart.value.before.height + (event.clientY - resizeStart.value.y) / props.scale)),
  });
}

function endResize() {
  window.removeEventListener("mousemove", resize);
  if (resizeStart.value) {
    emit("update", { width: props.image.width, height: props.image.height, __before: resizeStart.value.before }, true);
  }
  resizeStart.value = undefined;
}
</script>

<template>
  <article
    class="canvas-image"
    :class="{ selected, missing, [`decoration-${decoration}`]: true }"
    :style="style"
    @mousedown.left="startDrag"
    @contextmenu.prevent="emit('context', $event)"
  >
    <div v-if="decoration === 'tape'" class="tape"></div>
    <div v-if="decoration === 'double-tape'" class="double-tape"><i></i><i></i></div>
    <div v-if="decoration === 'corner-tape'" class="corner-tape"></div>
    <img v-if="!missing" :src="imageUrl" :alt="image.originalName || '图片'" draggable="false" @error="missing = true" />
    <div v-else class="missing-state">
      <ImageOff :size="28" />
      <span>找不到图片</span>
      <small>{{ image.fileName }}</small>
    </div>
    <span v-if="selected" class="resize-handle" @mousedown="startResize"></span>
  </article>
</template>

<style scoped>
.canvas-image {
  position: absolute;
  padding: 8px;
  background: #fff;
  border: 1px solid rgba(15, 23, 42, 0.08);
  border-radius: 4px;
  box-shadow: 0 10px 18px rgba(0, 0, 0, 0.14), 0 2px 4px rgba(0, 0, 0, 0.08);
  transform-origin: center center;
  cursor: var(--cursor-move);
  user-select: none;
}

.canvas-image.selected {
  outline: 2px solid rgba(59, 130, 246, 0.76);
  outline-offset: 3px;
}

.canvas-image img {
  width: 100%;
  height: 100%;
  display: block;
  object-fit: cover;
  border-radius: 2px;
  pointer-events: none;
}

.missing-state {
  width: 100%;
  height: 100%;
  display: grid;
  place-items: center;
  align-content: center;
  gap: 5px;
  color: #64748b;
  background: repeating-linear-gradient(45deg, #f8fafc, #f8fafc 10px, #eef2f7 10px, #eef2f7 20px);
  border: 1px dashed #cbd5e1;
  border-radius: 3px;
  text-align: center;
}

.missing-state span {
  font-size: 13px;
  font-weight: 700;
}

.missing-state small {
  max-width: calc(100% - 16px);
  overflow: hidden;
  color: #94a3b8;
  font-size: 11px;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.resize-handle {
  position: absolute;
  right: -7px;
  bottom: -7px;
  width: 14px;
  height: 14px;
  background: #fff;
  border: 2px solid #3b82f6;
  border-radius: 50%;
  cursor: var(--cursor-resize-nwse);
}

.tape,
.double-tape i,
.corner-tape {
  position: absolute;
  z-index: 1;
  pointer-events: none;
  background: rgba(255, 255, 255, 0.62);
  border: 1px solid rgba(148, 163, 184, 0.18);
  box-shadow: 0 2px 4px rgba(15, 23, 42, 0.08);
}

.tape {
  top: -13px;
  left: 50%;
  width: 74px;
  height: 24px;
  transform: translateX(-50%) rotate(-2deg);
}

.double-tape i {
  top: -10px;
  width: 54px;
  height: 22px;
}

.double-tape i:first-child {
  left: 22px;
  transform: rotate(-6deg);
}

.double-tape i:last-child {
  right: 22px;
  transform: rotate(5deg);
}

.corner-tape {
  top: -8px;
  right: -10px;
  width: 54px;
  height: 24px;
  transform: rotate(35deg);
}
</style>
