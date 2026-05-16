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
  const itemBounds = [...miniNotes.value, ...miniDrawings.value, ...miniImages.value];
  if (!itemBounds.length) return { minX: -400, minY: -300, maxX: 800, maxY: 600, width: 1200, height: 900 };
  const minX = Math.min(...itemBounds.map((item) => item.minX)) - padding;
  const minY = Math.min(...itemBounds.map((item) => item.minY)) - padding;
  const maxX = Math.max(...itemBounds.map((item) => item.maxX)) + padding;
  const maxY = Math.max(...itemBounds.map((item) => item.maxY)) + padding;
  return { minX, minY, maxX, maxY, width: Math.max(1, maxX - minX), height: Math.max(1, maxY - minY) };
});

const miniNotes = computed(() =>
  props.notes.map((note) => ({
    id: note.id,
    color: note.color,
    minX: note.x,
    minY: note.y,
    maxX: note.x + note.width,
    maxY: note.y + note.height,
  })),
);

const miniDrawings = computed(() =>
  props.drawings.flatMap((drawing) => {
    const bounds = drawingBounds(drawing);
    return bounds ? [{ drawing, ...bounds }] : [];
  }),
);

const miniImages = computed(() =>
  props.images.map((image) => ({
    id: image.id,
    minX: image.x,
    minY: image.y,
    maxX: image.x + image.width,
    maxY: image.y + image.height,
  })),
);

const mapScale = computed(() => Math.min(mapWidth / bounds.value.width, mapHeight / bounds.value.height));
const offset = computed(() => ({
  x: (mapWidth - bounds.value.width * mapScale.value) / 2,
  y: (mapHeight - bounds.value.height * mapScale.value) / 2,
}));

function boxStyle(item: { minX: number; minY: number; maxX: number; maxY: number }) {
  return {
    left: `${offset.value.x + (item.minX - bounds.value.minX) * mapScale.value}px`,
    top: `${offset.value.y + (item.minY - bounds.value.minY) * mapScale.value}px`,
    width: `${Math.max(4, (item.maxX - item.minX) * mapScale.value)}px`,
    height: `${Math.max(4, (item.maxY - item.minY) * mapScale.value)}px`,
  };
}

function noteStyle(note: (typeof miniNotes.value)[number]) {
  return {
    ...boxStyle(note),
    backgroundColor: noteColors[note.color],
  };
}

function imageStyle(image: (typeof miniImages.value)[number]) {
  return {
    ...boxStyle(image),
    backgroundColor: "#94a3b8",
  };
}

function drawingSvgStyle() {
  return {
    left: `${offset.value.x}px`,
    top: `${offset.value.y}px`,
    width: `${bounds.value.width * mapScale.value}px`,
    height: `${bounds.value.height * mapScale.value}px`,
  };
}

function miniPoint(point: { x: number; y: number }) {
  return {
    x: (point.x - bounds.value.minX) * mapScale.value,
    y: (point.y - bounds.value.minY) * mapScale.value,
  };
}

function miniDrawingPath(drawing: DrawingItem) {
  if (drawing.type === "pen") {
    const points = drawing.points ?? [];
    if (!points.length) return "";
    return points
      .map((point, index) => {
        const mini = miniPoint(point);
        return `${index === 0 ? "M" : "L"} ${mini.x} ${mini.y}`;
      })
      .join(" ");
  }
  if (drawing.type === "arrow" || drawing.type === "line") {
    if (!drawing.start || !drawing.end) return "";
    const start = miniPoint(drawing.start);
    const end = miniPoint(drawing.end);
    return `M ${start.x} ${start.y} L ${end.x} ${end.y}`;
  }
  const x = ((drawing.x ?? 0) - bounds.value.minX) * mapScale.value;
  const y = ((drawing.y ?? 0) - bounds.value.minY) * mapScale.value;
  const width = (drawing.width ?? 0) * mapScale.value;
  const height = (drawing.height ?? 0) * mapScale.value;
  if (drawing.type === "ellipse") {
    const rx = Math.abs(width) / 2;
    const ry = Math.abs(height) / 2;
    const cx = x + width / 2;
    const cy = y + height / 2;
    return `M ${cx - rx} ${cy} A ${rx} ${ry} 0 1 0 ${cx + rx} ${cy} A ${rx} ${ry} 0 1 0 ${cx - rx} ${cy}`;
  }
  return `M ${x} ${y} L ${x + width} ${y} L ${x + width} ${y + height} L ${x} ${y + height} Z`;
}

function miniStrokeWidth(drawing: DrawingItem) {
  return Math.max(1, Math.min(2.4, drawing.strokeWidth * mapScale.value * 0.45));
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
      <div v-for="note in miniNotes" :key="`note-${note.id}`" class="mini-item note" :style="noteStyle(note)"></div>
      <div v-for="image in miniImages" :key="`image-${image.id}`" class="mini-item image" :style="imageStyle(image)"></div>
      <svg class="mini-drawings" :style="drawingSvgStyle()" :viewBox="`0 0 ${bounds.width * mapScale} ${bounds.height * mapScale}`">
        <path
          v-for="item in miniDrawings"
          :key="`drawing-${item.drawing.id}`"
          :d="miniDrawingPath(item.drawing)"
          :stroke="item.drawing.color"
          :stroke-width="miniStrokeWidth(item.drawing)"
          fill="none"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
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

.mini-item.image {
  opacity: 0.62;
  outline: 1px solid rgba(51, 65, 85, 0.35);
}

.mini-drawings {
  position: absolute;
  pointer-events: none;
  opacity: 0.62;
  overflow: visible;
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
