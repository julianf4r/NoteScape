<script setup lang="ts">
import type { DrawingItem, DrawingPoint } from "../types";
import { useDrawingStore } from "../stores/drawingStore";

defineProps<{
  canvasId: string;
  drawings: DrawingItem[];
  scale: number;
}>();

const emit = defineEmits<{
  dragDrawing: [event: MouseEvent, drawingId: string];
}>();

const drawingStore = useDrawingStore();

function selectDrawing(event: MouseEvent, id: string) {
  if (drawingStore.tool !== "select") return;
  event.preventDefault();
  event.stopPropagation();
  drawingStore.select(id);
  emit("dragDrawing", event, id);
}

function smoothPenPath(drawing: DrawingItem) {
  return pointsToSmoothPath(drawing.points ?? []);
}

function pointsToSmoothPath(points: DrawingPoint[]) {
  if (!points.length) return "";
  if (points.length === 1) return `M ${points[0].x} ${points[0].y}`;
  if (points.length === 2) return `M ${points[0].x} ${points[0].y} L ${points[1].x} ${points[1].y}`;

  const [first, second] = points;
  const firstMid = midpoint(first, second);
  const commands = [`M ${first.x} ${first.y}`, `L ${firstMid.x} ${firstMid.y}`];

  for (let index = 1; index < points.length - 1; index += 1) {
    const current = points[index];
    const next = points[index + 1];
    const mid = midpoint(current, next);
    commands.push(`Q ${current.x} ${current.y} ${mid.x} ${mid.y}`);
  }

  const last = points[points.length - 1];
  commands.push(`L ${last.x} ${last.y}`);
  return commands.join(" ");
}

function midpoint(a: DrawingPoint, b: DrawingPoint) {
  return {
    x: (a.x + b.x) / 2,
    y: (a.y + b.y) / 2,
  };
}

function arrowGeometry(drawing: DrawingItem) {
  const start = drawing.start ?? { x: 0, y: 0 };
  const end = drawing.end ?? start;
  const dx = end.x - start.x;
  const dy = end.y - start.y;
  const length = Math.hypot(dx, dy);

  if (length < 1) {
    return {
      main: `M ${start.x} ${start.y} L ${end.x} ${end.y}`,
      left: "",
      right: "",
    };
  }

  const seed = seededUnit(drawing.id);
  const unitX = dx / length;
  const unitY = dy / length;
  const normalX = -unitY;
  const normalY = unitX;
  const curveOffset = clamp((seed - 0.5) * length * 0.055, -18, 18);
  const control = {
    x: (start.x + end.x) / 2 + normalX * curveOffset,
    y: (start.y + end.y) / 2 + normalY * curveOffset,
  };
  const tangentX = end.x - control.x;
  const tangentY = end.y - control.y;
  const tangentLength = Math.hypot(tangentX, tangentY) || 1;
  const backX = -tangentX / tangentLength;
  const backY = -tangentY / tangentLength;
  const headNormalX = -backY;
  const headNormalY = backX;
  const headLength = clamp(Math.max(16, drawing.strokeWidth * 2.8), 16, 34);
  const spread = headLength * 0.58;
  const leftLength = headLength * (0.9 + seededUnit(`${drawing.id}:left`) * 0.18);
  const rightLength = headLength * (0.9 + seededUnit(`${drawing.id}:right`) * 0.18);
  const leftSpread = spread * (0.9 + seededUnit(`${drawing.id}:left-spread`) * 0.22);
  const rightSpread = spread * (0.9 + seededUnit(`${drawing.id}:right-spread`) * 0.22);
  const left = {
    x: end.x + backX * leftLength + headNormalX * leftSpread,
    y: end.y + backY * leftLength + headNormalY * leftSpread,
  };
  const right = {
    x: end.x + backX * rightLength - headNormalX * rightSpread,
    y: end.y + backY * rightLength - headNormalY * rightSpread,
  };

  return {
    main: `M ${start.x} ${start.y} Q ${control.x} ${control.y} ${end.x} ${end.y}`,
    left: `M ${end.x} ${end.y} L ${left.x} ${left.y}`,
    right: `M ${end.x} ${end.y} L ${right.x} ${right.y}`,
  };
}

function seededUnit(value: string) {
  let hash = 2166136261;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }
  return (hash >>> 0) / 4294967295;
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
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
      <filter id="drawing-selection-glow" x="-80%" y="-80%" width="260%" height="260%">
        <feGaussianBlur in="SourceGraphic" stdDeviation="4" result="blur" />
        <feColorMatrix
          in="blur"
          type="matrix"
          values="0 0 0 0 0.231 0 0 0 0 0.510 0 0 0 0 0.965 0 0 0 0.95 0"
          result="blueGlow"
        />
        <feMerge>
          <feMergeNode in="blueGlow" />
        </feMerge>
      </filter>
    </defs>

    <g v-for="drawing in drawings" :key="drawing.id" :class="{ selected: drawingStore.selectedId === drawing.id, 'arrow-drawing': drawing.type === 'arrow' }">
      <path
        v-if="drawing.type === 'pen'"
        class="drawing-stroke"
        :d="smoothPenPath(drawing)"
        :stroke="drawing.color"
        :stroke-width="drawing.strokeWidth"
        fill="none"
        stroke-linecap="round"
        stroke-linejoin="round"
        @mousedown="selectDrawing($event, drawing.id)"
      />
      <g
        v-else-if="drawing.type === 'arrow'"
        @mousedown="selectDrawing($event, drawing.id)"
      >
        <template v-if="drawingStore.selectedId === drawing.id">
          <path
            class="selection-glow"
            :d="arrowGeometry(drawing).main"
            :stroke-width="drawing.strokeWidth + 8"
            fill="none"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <path
            class="selection-glow"
            :d="arrowGeometry(drawing).left"
            :stroke-width="drawing.strokeWidth + 8"
            fill="none"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <path
            class="selection-glow"
            :d="arrowGeometry(drawing).right"
            :stroke-width="drawing.strokeWidth + 8"
            fill="none"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </template>
        <path
          class="drawing-stroke"
          :d="arrowGeometry(drawing).main"
          :stroke="drawing.color"
          :stroke-width="drawing.strokeWidth"
          fill="none"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
        <path
          class="drawing-stroke"
          :d="arrowGeometry(drawing).left"
          :stroke="drawing.color"
          :stroke-width="drawing.strokeWidth"
          fill="none"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
        <path
          class="drawing-stroke"
          :d="arrowGeometry(drawing).right"
          :stroke="drawing.color"
          :stroke-width="drawing.strokeWidth"
          fill="none"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </g>
      <line
        v-else-if="drawing.type === 'line'"
        class="drawing-stroke"
        :x1="drawing.start?.x ?? 0"
        :y1="drawing.start?.y ?? 0"
        :x2="drawing.end?.x ?? 0"
        :y2="drawing.end?.y ?? 0"
        :stroke="drawing.color"
        :stroke-width="drawing.strokeWidth"
        stroke-linecap="round"
        @mousedown="selectDrawing($event, drawing.id)"
      />
      <rect
        v-else-if="drawing.type === 'rect'"
        class="drawing-stroke drawing-shape"
        v-bind="rectFor(drawing)"
        :stroke="drawing.color"
        :stroke-width="drawing.strokeWidth"
        fill="transparent"
        rx="4"
        @mousedown="selectDrawing($event, drawing.id)"
      />
      <ellipse
        v-else-if="drawing.type === 'ellipse'"
        class="drawing-stroke drawing-shape"
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

.drawing-layer :deep(.drawing-stroke) {
  vector-effect: non-scaling-stroke;
  pointer-events: stroke;
  cursor: move;
}

.drawing-layer :deep(.drawing-shape) {
  pointer-events: visiblePainted;
}

.drawing-layer :deep(.selection-glow) {
  stroke: rgba(59, 130, 246, 0.72);
  filter: url("#drawing-selection-glow");
  vector-effect: non-scaling-stroke;
  pointer-events: none;
}

.drawing-layer .selected:not(.arrow-drawing) :deep(.drawing-stroke) {
  filter: drop-shadow(0 0 4px rgba(59, 130, 246, 0.9));
}
</style>
