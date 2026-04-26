<script setup lang="ts">
import type { DrawingItem } from "../types";
import { useDrawingStore } from "../stores/drawingStore";

defineProps<{
  canvasId: string;
  drawings: DrawingItem[];
  scale: number;
}>();

const drawingStore = useDrawingStore();

function selectDrawing(event: MouseEvent, id: string) {
  if (drawingStore.tool !== "select") return;
  event.preventDefault();
  event.stopPropagation();
  drawingStore.select(id);
}

function penPoints(drawing: DrawingItem) {
  return (drawing.points ?? []).map((point) => `${point.x},${point.y}`).join(" ");
}

function rectFor(drawing: DrawingItem) {
  return {
    x: drawing.x ?? 0,
    y: drawing.y ?? 0,
    width: drawing.width ?? 0,
    height: drawing.height ?? 0,
  };
}
</script>

<template>
  <svg
    data-drawing-layer="true"
    class="drawing-layer"
    width="2800"
    height="2200"
    viewBox="0 0 2800 2200"
  >
    <defs>
      <marker id="drawing-arrowhead" markerWidth="10" markerHeight="10" refX="8" refY="3" orient="auto" markerUnits="strokeWidth">
        <path d="M0,0 L0,6 L8,3 z" fill="context-stroke" />
      </marker>
    </defs>

    <g v-for="drawing in drawings" :key="drawing.id" :class="{ selected: drawingStore.selectedId === drawing.id }">
      <polyline
        v-if="drawing.type === 'pen'"
        :points="penPoints(drawing)"
        :stroke="drawing.color"
        :stroke-width="drawing.strokeWidth"
        fill="none"
        stroke-linecap="round"
        stroke-linejoin="round"
        @mousedown="selectDrawing($event, drawing.id)"
      />
      <line
        v-else-if="drawing.type === 'arrow'"
        :x1="drawing.start?.x ?? 0"
        :y1="drawing.start?.y ?? 0"
        :x2="drawing.end?.x ?? 0"
        :y2="drawing.end?.y ?? 0"
        :stroke="drawing.color"
        :stroke-width="drawing.strokeWidth"
        stroke-linecap="round"
        marker-end="url(#drawing-arrowhead)"
        @mousedown="selectDrawing($event, drawing.id)"
      />
      <rect
        v-else-if="drawing.type === 'rect'"
        v-bind="rectFor(drawing)"
        :stroke="drawing.color"
        :stroke-width="drawing.strokeWidth"
        fill="transparent"
        rx="4"
        @mousedown="selectDrawing($event, drawing.id)"
      />
      <ellipse
        v-else-if="drawing.type === 'ellipse'"
        :cx="(drawing.x ?? 0) + (drawing.width ?? 0) / 2"
        :cy="(drawing.y ?? 0) + (drawing.height ?? 0) / 2"
        :rx="(drawing.width ?? 0) / 2"
        :ry="(drawing.height ?? 0) / 2"
        :stroke="drawing.color"
        :stroke-width="drawing.strokeWidth"
        fill="transparent"
        @mousedown="selectDrawing($event, drawing.id)"
      />
    </g>
  </svg>
</template>

<style scoped>
.drawing-layer {
  position: absolute;
  inset: 0;
  z-index: 200001;
  pointer-events: none;
  overflow: visible;
}

.drawing-layer :deep(polyline),
.drawing-layer :deep(line),
.drawing-layer :deep(rect),
.drawing-layer :deep(ellipse) {
  vector-effect: non-scaling-stroke;
  pointer-events: stroke;
}

.drawing-layer :deep(rect),
.drawing-layer :deep(ellipse) {
  pointer-events: visiblePainted;
}

.drawing-layer .selected :deep(polyline),
.drawing-layer .selected :deep(line),
.drawing-layer .selected :deep(rect),
.drawing-layer .selected :deep(ellipse) {
  filter: drop-shadow(0 0 4px rgba(59, 130, 246, 0.9));
}
</style>
