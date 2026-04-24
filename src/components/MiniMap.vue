<script setup lang="ts">
import { computed } from "vue";
import { Maximize2, Minus, Plus } from "lucide-vue-next";
import type { StickyNote, ViewportState } from "../types";
import { noteColors } from "../utils/colors";

const props = defineProps<{
  notes: StickyNote[];
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
  if (!props.notes.length) return { minX: -400, minY: -300, maxX: 800, maxY: 600, width: 1200, height: 900 };
  const minX = Math.min(...props.notes.map((note) => note.x)) - padding;
  const minY = Math.min(...props.notes.map((note) => note.y)) - padding;
  const maxX = Math.max(...props.notes.map((note) => note.x + note.width)) + padding;
  const maxY = Math.max(...props.notes.map((note) => note.y + note.height)) + padding;
  return { minX, minY, maxX, maxY, width: Math.max(1, maxX - minX), height: Math.max(1, maxY - minY) };
});

const mapScale = computed(() => Math.min(mapWidth / bounds.value.width, mapHeight / bounds.value.height));
const offset = computed(() => ({
  x: (mapWidth - bounds.value.width * mapScale.value) / 2,
  y: (mapHeight - bounds.value.height * mapScale.value) / 2,
}));

function noteStyle(note: StickyNote) {
  return {
    left: `${offset.value.x + (note.x - bounds.value.minX) * mapScale.value}px`,
    top: `${offset.value.y + (note.y - bounds.value.minY) * mapScale.value}px`,
    width: `${Math.max(4, note.width * mapScale.value)}px`,
    height: `${Math.max(4, note.height * mapScale.value)}px`,
    backgroundColor: noteColors[note.color],
  };
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
      <div v-for="note in props.notes" :key="note.id" class="mini-note" :style="noteStyle(note)"></div>
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
  cursor: pointer;
}

.mini-note {
  position: absolute;
  border-radius: 1px;
  opacity: 0.72;
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
