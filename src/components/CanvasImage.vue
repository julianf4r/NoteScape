<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ImageOff } from "lucide-vue-next";
import type { CanvasImage } from "../types";
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
  const before = resizeStart.value.before;
  const aspectRatio = before.width / Math.max(1, before.height);
  const deltaX = (event.clientX - resizeStart.value.x) / props.scale;
  const deltaY = (event.clientY - resizeStart.value.y) / props.scale;
  const diagonalLength = Math.hypot(before.width, before.height) || 1;
  const projectedDelta = (deltaX * before.width + deltaY * before.height) / diagonalLength;
  const scale = Math.min(1000 / before.width, Math.max(80 / before.width, (diagonalLength + projectedDelta) / diagonalLength));
  const width = before.width * scale;
  emit("live", {
    width,
    height: width / aspectRatio,
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
    :class="{ selected, missing }"
    :style="style"
    @mousedown.left="startDrag"
    @contextmenu.prevent="emit('context', $event)"
  >
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

</style>
