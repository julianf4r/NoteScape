<script setup lang="ts">
import { Maximize2, Minus, Plus } from "lucide-vue-next";
import type { StickyNote, ViewportState } from "../types";
import { noteColors } from "../utils/colors";

const props = defineProps<{
  notes: StickyNote[];
  viewport: ViewportState;
}>();

const emit = defineEmits<{
  zoomIn: [];
  zoomOut: [];
  fit: [];
  jump: [x: number, y: number];
}>();

function clickMap(event: MouseEvent) {
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
  const x = ((event.clientX - rect.left) / rect.width) * 1200;
  const y = ((event.clientY - rect.top) / rect.height) * 900;
  emit("jump", x, y);
}
</script>

<template>
  <div class="minimap">
    <div class="map" @click="clickMap">
      <div
        v-for="note in props.notes"
        :key="note.id"
        class="mini-note"
        :style="{
          left: `${note.x / 10}px`,
          top: `${note.y / 10}px`,
          width: `${Math.max(10, note.width / 10)}px`,
          height: `${Math.max(8, note.height / 10)}px`,
          backgroundColor: noteColors[note.color],
        }"
      ></div>
      <div
        class="viewport"
        :style="{
          left: `${Math.max(0, -props.viewport.offsetX / props.viewport.scale / 10)}px`,
          top: `${Math.max(0, -props.viewport.offsetY / props.viewport.scale / 10)}px`,
        }"
      ></div>
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
  width: 72px;
  height: 42px;
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
