<script setup lang="ts">
import { computed } from "vue";
import { Maximize2, Minus, Plus } from "lucide-vue-next";
import type { CanvasImage, DrawingItem, StickyNote, ViewportState } from "../types";
import { noteColors } from "../utils/colors";

const props = defineProps<{
  notes: StickyNote[];
  drawings: DrawingItem[];
  images: CanvasImage[];
  viewport: ViewportState;
  boardWidth: number;
  boardHeight: number;
}>();

const emit = defineEmits<{
  zoomIn: [];
  zoomOut: [];
  fit: [];
  jump: [x: number, y: number];
}>();

const mapWidth = 154;
const mapHeight = 122;
const padding = 60;

const bounds = computed(() => {
  const itemBounds = canvasItemBounds.value;
  if (!itemBounds.length) return { minX: -400, minY: -300, maxX: 800, maxY: 600, width: 1200, height: 900 };
  const minX = Math.min(...itemBounds.map((item) => item.minX)) - padding;
  const minY = Math.min(...itemBounds.map((item) => item.minY)) - padding;
  const maxX = Math.max(...itemBounds.map((item) => item.maxX)) + padding;
  const maxY = Math.max(...itemBounds.map((item) => item.maxY)) + padding;
  return { minX, minY, maxX, maxY, width: Math.max(1, maxX - minX), height: Math.max(1, maxY - minY) };
});

const canvasItemBounds = computed(() => [
  ...props.notes.map((note) => ({
    id: note.id,
    type: "note" as const,
    color: note.color,
    minX: note.x,
    minY: note.y,
    maxX: note.x + note.width,
    maxY: note.y + note.height,
  })),
  ...props.drawings.flatMap((drawing) => {
    const bounds = drawingBounds(drawing);
    return bounds ? [{ id: drawing.id, type: "drawing" as const, color: drawing.color, ...bounds }] : [];
  }),
  ...props.images.map((image) => ({
    id: image.id,
    type: "image" as const,
    minX: image.x,
    minY: image.y,
    maxX: image.x + image.width,
    maxY: image.y + image.height,
  })),
]);

const mapScale = computed(() => Math.min(mapWidth / bounds.value.width, mapHeight / bounds.value.height));
const offset = computed(() => ({
  x: (mapWidth - bounds.value.width * mapScale.value) / 2,
  y: (mapHeight - bounds.value.height * mapScale.value) / 2,
}));

function itemStyle(item: (typeof canvasItemBounds.value)[number]) {
  return {
    left: `${offset.value.x + (item.minX - bounds.value.minX) * mapScale.value}px`,
    top: `${offset.value.y + (item.minY - bounds.value.minY) * mapScale.value}px`,
    width: `${Math.max(4, (item.maxX - item.minX) * mapScale.value)}px`,
    height: `${Math.max(4, (item.maxY - item.minY) * mapScale.value)}px`,
    backgroundColor: item.type === "note" ? noteColors[item.color] : item.type === "drawing" ? item.color : "#94a3b8",
  };
}

function drawingBounds(drawing: DrawingItem) {
  if (drawing.type === "pen") {
    const points = drawing.points ?? [];
    if (!points.length) return null;
    const xs = points.map((point) => point.x);
    const ys = points.map((point) => point.y);
    return { minX: Math.min(...xs), minY: Math.min(...ys), maxX: Math.max(...xs), maxY: Math.max(...ys) };
  }
  if (drawing.type === "arrow" || drawing.type === "line") {
    if (!drawing.start || !drawing.end) return null;
    return {
      minX: Math.min(drawing.start.x, drawing.end.x),
      minY: Math.min(drawing.start.y, drawing.end.y),
      maxX: Math.max(drawing.start.x, drawing.end.x),
      maxY: Math.max(drawing.start.y, drawing.end.y),
    };
  }
  const x = drawing.x ?? 0;
  const y = drawing.y ?? 0;
  return { minX: x, minY: y, maxX: x + (drawing.width ?? 0), maxY: y + (drawing.height ?? 0) };
}

const viewportStyle = computed(() => {
  const worldLeft = -props.viewport.offsetX / props.viewport.scale;
  const worldTop = -props.viewport.offsetY / props.viewport.scale;
  const worldWidth = props.boardWidth / props.viewport.scale;
  const worldHeight = props.boardHeight / props.viewport.scale;
  return {
    left: `${offset.value.x + (worldLeft - bounds.value.minX) * mapScale.value}px`,
    top: `${offset.value.y + (worldTop - bounds.value.minY) * mapScale.value}px`,
    width: `${Math.max(8, worldWidth * mapScale.value)}px`,
    height: `${Math.max(8, worldHeight * mapScale.value)}px`,
  };
});

function clickMap(event: MouseEvent) {
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
  const x = bounds.value.minX + ((event.clientX - rect.left - offset.value.x) / mapScale.value);
  const y = bounds.value.minY + ((event.clientY - rect.top - offset.value.y) / mapScale.value);
  emit("jump", x, y);
}
</script>

<template>
  <div class="minimap">
    <div class="map" @click="clickMap">
      <div v-for="item in canvasItemBounds" :key="`${item.type}-${item.id}`" class="mini-item" :class="item.type" :style="itemStyle(item)"></div>
      <div class="viewport" :style="viewportStyle"></div>
    </div>
    <div class="mini-controls">
      <button @click="emit('zoomOut')"><Minus :size="16" /></button>
      <b>{{ Math.round(props.viewport.scale * 100) }}%</b>
      <button @click="emit('zoomIn')"><Plus :size="16" /></button>
      <button @click="emit('fit')"><Maximize2 :size="15" /></button>
    </div>
  </div>
</template>

<style scoped>
.minimap {
  position: absolute;
  left: 12px;
  bottom: 20px;
  z-index: 25;
  width: 176px;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.94);
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  box-shadow: var(--shadow-md);
}

.map {
  position: relative;
  height: 122px;
  margin: 11px;
  background: #f2f2f1;
  border: 1px solid #dedede;
  cursor: var(--cursor-pointer);
}

.mini-item {
  position: absolute;
  border-radius: 1px;
  opacity: 0.72;
}

.mini-item.drawing {
  min-width: 5px;
  min-height: 3px;
  border-radius: 999px;
}

.mini-item.image {
  opacity: 0.62;
  outline: 1px solid rgba(51, 65, 85, 0.35);
}

.viewport {
  position: absolute;
  border: 2px solid #3b82f6;
  background: rgba(59, 130, 246, 0.06);
}

.mini-controls {
  height: 40px;
  display: grid;
  grid-template-columns: 34px 1fr 34px 34px;
  align-items: center;
  border-top: 1px solid #e5e7eb;
}

button {
  height: 40px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: #374151;
  background: transparent;
}

b {
  text-align: center;
  font-size: 13px;
  color: #4b5563;
}
</style>
