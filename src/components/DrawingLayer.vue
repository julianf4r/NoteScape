<script setup lang="ts">
import { computed } from "vue";
import type { DrawingItem, DrawingPoint } from "../types";
import { useDrawingStore } from "../stores/drawingStore";

const props = defineProps<{
  canvasId: string;
  drawings: DrawingItem[];
  scale: number;
  panMode: boolean;
  fontFamily: string;
}>();

const emit = defineEmits<{
  dragDrawing: [event: MouseEvent, drawingId: string];
  editDrawing: [event: MouseEvent, drawingId: string, handle: "start" | "end" | "resize"];
  contextDrawing: [event: MouseEvent, drawingId: string];
}>();

const drawingStore = useDrawingStore();

const layerBounds = computed(() => {
  const bounds = props.drawings.map(drawingBounds).filter((item): item is DrawingBounds => Boolean(item));
  const maxStroke = Math.max(1, ...props.drawings.map((drawing) => drawing.strokeWidth));
  const padding = maxStroke + 28;
  if (!bounds.length) return { minX: -padding, minY: -padding, width: padding * 2, height: padding * 2 };
  const minX = Math.min(...bounds.map((item) => item.minX)) - padding;
  const minY = Math.min(...bounds.map((item) => item.minY)) - padding;
  const maxX = Math.max(...bounds.map((item) => item.maxX)) + padding;
  const maxY = Math.max(...bounds.map((item) => item.maxY)) + padding;
  return {
    minX,
    minY,
    width: Math.max(1, maxX - minX),
    height: Math.max(1, maxY - minY),
  };
});

const layerStyle = computed(() => ({
  left: `${layerBounds.value.minX}px`,
  top: `${layerBounds.value.minY}px`,
  width: `${layerBounds.value.width}px`,
  height: `${layerBounds.value.height}px`,
}));

type DrawingBounds = { minX: number; minY: number; maxX: number; maxY: number };

function drawingBounds(drawing: DrawingItem): DrawingBounds | null {
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
  if (drawing.type === "text") {
    const rect = textRectFor(drawing);
    return {
      minX: rect.x,
      minY: rect.y,
      maxX: rect.x + rect.width,
      maxY: rect.y + rect.height,
    };
  }
  const x = drawing.x ?? 0;
  const y = drawing.y ?? 0;
  const width = drawing.width ?? 0;
  const height = drawing.height ?? 0;
  return {
    minX: Math.min(x, x + width),
    minY: Math.min(y, y + height),
    maxX: Math.max(x, x + width),
    maxY: Math.max(y, y + height),
  };
}

function selectDrawing(event: MouseEvent, id: string) {
  if (event.button === 1 || props.panMode) return;
  event.preventDefault();
  event.stopPropagation();
  emit("dragDrawing", event, id);
}

function editDrawing(event: MouseEvent, id: string, handle: "start" | "end" | "resize") {
  if (event.button !== 0 || props.panMode) return;
  event.preventDefault();
  event.stopPropagation();
  emit("editDrawing", event, id, handle);
}

function openDrawingMenu(event: MouseEvent, id: string) {
  event.preventDefault();
  event.stopPropagation();
  emit("contextDrawing", event, id);
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

function roughLinePath(drawing: DrawingItem) {
  const start = drawing.start ?? { x: 0, y: 0 };
  const end = drawing.end ?? start;
  return curvedLinePath(start, end, drawing.id, 0.035, 12);
}

function curvedLinePath(start: DrawingPoint, end: DrawingPoint, seedKey: string, ratio: number, maxOffset: number) {
  const dx = end.x - start.x;
  const dy = end.y - start.y;
  const length = Math.hypot(dx, dy);
  if (length < 1) return `M ${start.x} ${start.y} L ${end.x} ${end.y}`;

  const normalX = -dy / length;
  const normalY = dx / length;
  const curveOffset = clamp(seededSigned(seedKey) * length * ratio, -maxOffset, maxOffset);
  const control = {
    x: (start.x + end.x) / 2 + normalX * curveOffset,
    y: (start.y + end.y) / 2 + normalY * curveOffset,
  };
  return `M ${start.x} ${start.y} Q ${control.x} ${control.y} ${end.x} ${end.y}`;
}

function roughRectPath(drawing: DrawingItem) {
  const rect = rectFor(drawing);
  const x = rect.x;
  const y = rect.y;
  const width = Math.max(0, rect.width);
  const height = Math.max(0, rect.height);
  if (width < 1 || height < 1) return "";

  const radius = Math.min(18, width * 0.12, height * 0.12);
  const wobble = Math.min(8, Math.max(2, Math.min(width, height) * 0.025));
  const p = (name: string, px: number, py: number) => ({
    x: px + seededSigned(`${drawing.id}:${name}:x`) * wobble,
    y: py + seededSigned(`${drawing.id}:${name}:y`) * wobble,
  });

  const topLeftStart = p("tls", x + radius, y);
  const topRightStart = p("trs", x + width - radius, y);
  const topRightEnd = p("tre", x + width, y + radius);
  const bottomRightStart = p("brs", x + width, y + height - radius);
  const bottomRightEnd = p("bre", x + width - radius, y + height);
  const bottomLeftStart = p("bls", x + radius, y + height);
  const bottomLeftEnd = p("ble", x, y + height - radius);
  const topLeftEnd = p("tle", x, y + radius);

  return [
    `M ${topLeftStart.x} ${topLeftStart.y}`,
    `Q ${(topLeftStart.x + topRightStart.x) / 2} ${y + seededSigned(`${drawing.id}:top`) * wobble} ${topRightStart.x} ${topRightStart.y}`,
    `Q ${x + width + seededSigned(`${drawing.id}:tr`) * wobble} ${y + seededSigned(`${drawing.id}:tr2`) * wobble} ${topRightEnd.x} ${topRightEnd.y}`,
    `Q ${x + width + seededSigned(`${drawing.id}:right`) * wobble} ${(topRightEnd.y + bottomRightStart.y) / 2} ${bottomRightStart.x} ${bottomRightStart.y}`,
    `Q ${x + width + seededSigned(`${drawing.id}:br`) * wobble} ${y + height + seededSigned(`${drawing.id}:br2`) * wobble} ${bottomRightEnd.x} ${bottomRightEnd.y}`,
    `Q ${(bottomRightEnd.x + bottomLeftStart.x) / 2} ${y + height + seededSigned(`${drawing.id}:bottom`) * wobble} ${bottomLeftStart.x} ${bottomLeftStart.y}`,
    `Q ${x + seededSigned(`${drawing.id}:bl`) * wobble} ${y + height + seededSigned(`${drawing.id}:bl2`) * wobble} ${bottomLeftEnd.x} ${bottomLeftEnd.y}`,
    `Q ${x + seededSigned(`${drawing.id}:left`) * wobble} ${(bottomLeftEnd.y + topLeftEnd.y) / 2} ${topLeftEnd.x} ${topLeftEnd.y}`,
    `Q ${x + seededSigned(`${drawing.id}:tl`) * wobble} ${y + seededSigned(`${drawing.id}:tl2`) * wobble} ${topLeftStart.x} ${topLeftStart.y}`,
  ].join(" ");
}

function roughEllipsePath(drawing: DrawingItem) {
  const rect = rectFor(drawing);
  const cx = rect.x + rect.width / 2;
  const cy = rect.y + rect.height / 2;
  const rx = Math.abs(rect.width) / 2;
  const ry = Math.abs(rect.height) / 2;
  if (rx < 1 || ry < 1) return "";

  const wobble = Math.min(10, Math.max(2, Math.min(rx, ry) * 0.035));
  const k = 0.5522847498;
  const p = (name: string, px: number, py: number) => ({
    x: px + seededSigned(`${drawing.id}:${name}:x`) * wobble,
    y: py + seededSigned(`${drawing.id}:${name}:y`) * wobble,
  });
  const top = p("top", cx, cy - ry);
  const right = p("right", cx + rx, cy);
  const bottom = p("bottom", cx, cy + ry);
  const left = p("left", cx - rx, cy);

  return [
    `M ${top.x} ${top.y}`,
    `C ${cx + rx * k} ${cy - ry + seededSigned(`${drawing.id}:c1`) * wobble} ${cx + rx + seededSigned(`${drawing.id}:c2`) * wobble} ${cy - ry * k} ${right.x} ${right.y}`,
    `C ${cx + rx + seededSigned(`${drawing.id}:c3`) * wobble} ${cy + ry * k} ${cx + rx * k} ${cy + ry + seededSigned(`${drawing.id}:c4`) * wobble} ${bottom.x} ${bottom.y}`,
    `C ${cx - rx * k} ${cy + ry + seededSigned(`${drawing.id}:c5`) * wobble} ${cx - rx + seededSigned(`${drawing.id}:c6`) * wobble} ${cy + ry * k} ${left.x} ${left.y}`,
    `C ${cx - rx + seededSigned(`${drawing.id}:c7`) * wobble} ${cy - ry * k} ${cx - rx * k} ${cy - ry + seededSigned(`${drawing.id}:c8`) * wobble} ${top.x} ${top.y}`,
  ].join(" ");
}

function seededUnit(value: string) {
  let hash = 2166136261;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }
  return (hash >>> 0) / 4294967295;
}

function seededSigned(value: string) {
  return seededUnit(value) * 2 - 1;
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

function textLines(drawing: DrawingItem) {
  return (drawing.text || "").split(/\r?\n/);
}

function textFontSize(drawing: DrawingItem) {
  return drawing.fontSize ?? 18;
}

function textLineHeight(drawing: DrawingItem) {
  return textFontSize(drawing) * 1.35;
}

function textRectFor(drawing: DrawingItem) {
  const fontSize = textFontSize(drawing);
  const lines = textLines(drawing);
  return {
    x: drawing.x ?? 0,
    y: drawing.y ?? 0,
    width: Math.max(drawing.width ?? 0, 24),
    height: Math.max(drawing.height ?? 0, Math.max(1, lines.length) * fontSize * 1.35),
  };
}

function resizeHandlePoint(drawing: DrawingItem) {
  const rect = drawing.type === "text" ? textRectFor(drawing) : rectFor(drawing);
  return {
    x: rect.x + rect.width,
    y: rect.y + rect.height,
  };
}
</script>

<template>
  <svg
    data-drawing-layer="true"
    class="drawing-layer"
    :width="layerBounds.width"
    :height="layerBounds.height"
    :viewBox="`${layerBounds.minX} ${layerBounds.minY} ${layerBounds.width} ${layerBounds.height}`"
    :style="layerStyle"
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

    <g
      v-for="drawing in drawings"
      :key="drawing.id"
      :class="{ selected: drawingStore.selectedIds.includes(drawing.id), 'arrow-drawing': drawing.type === 'arrow' }"
      @contextmenu="openDrawingMenu($event, drawing.id)"
    >
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
        <template v-if="drawingStore.selectedIds.includes(drawing.id)">
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
      <path
        v-else-if="drawing.type === 'line'"
        class="drawing-stroke"
        :d="roughLinePath(drawing)"
        :stroke="drawing.color"
        :stroke-width="drawing.strokeWidth"
        fill="none"
        stroke-linecap="round"
        stroke-linejoin="round"
        @mousedown="selectDrawing($event, drawing.id)"
      />
      <path
        v-else-if="drawing.type === 'rect'"
        class="drawing-stroke drawing-shape"
        :d="roughRectPath(drawing)"
        :stroke="drawing.color"
        :stroke-width="drawing.strokeWidth"
        fill="none"
        stroke-linecap="round"
        stroke-linejoin="round"
        @mousedown="selectDrawing($event, drawing.id)"
      />
      <path
        v-else-if="drawing.type === 'ellipse'"
        class="drawing-stroke drawing-shape"
        :d="roughEllipsePath(drawing)"
        :stroke="drawing.color"
        :stroke-width="drawing.strokeWidth"
        fill="none"
        stroke-linecap="round"
        stroke-linejoin="round"
        @mousedown="selectDrawing($event, drawing.id)"
      />
      <g
        v-else-if="drawing.type === 'text'"
        class="drawing-text"
        @mousedown="selectDrawing($event, drawing.id)"
      >
        <rect
          class="drawing-text-hit"
          :x="textRectFor(drawing).x"
          :y="textRectFor(drawing).y"
          :width="textRectFor(drawing).width"
          :height="textRectFor(drawing).height"
        />
        <rect
          v-if="drawingStore.selectedIds.includes(drawing.id)"
          class="drawing-text-bounds"
          :x="textRectFor(drawing).x"
          :y="textRectFor(drawing).y"
          :width="textRectFor(drawing).width"
          :height="textRectFor(drawing).height"
        />
        <text
          class="drawing-text-content"
          :x="textRectFor(drawing).x"
          :y="textRectFor(drawing).y + textFontSize(drawing)"
          :fill="drawing.color"
          :font-size="textFontSize(drawing)"
          :font-family="fontFamily"
        >
          <tspan
            v-for="(line, index) in textLines(drawing)"
            :key="index"
            :x="textRectFor(drawing).x"
            :dy="index === 0 ? 0 : textLineHeight(drawing)"
          >{{ line || " " }}</tspan>
        </text>
      </g>
      <template v-if="drawingStore.selectedIds.includes(drawing.id) && (drawing.type === 'arrow' || drawing.type === 'line')">
        <g
          v-if="drawing.start"
          class="edit-handle"
          @mousedown="editDrawing($event, drawing.id, 'start')"
        >
          <circle class="edit-handle-hit" :cx="drawing.start.x" :cy="drawing.start.y" :r="11 / scale" />
          <circle class="edit-handle-dot" :cx="drawing.start.x" :cy="drawing.start.y" :r="5 / scale" />
        </g>
        <g
          v-if="drawing.end"
          class="edit-handle"
          @mousedown="editDrawing($event, drawing.id, 'end')"
        >
          <circle class="edit-handle-hit" :cx="drawing.end.x" :cy="drawing.end.y" :r="11 / scale" />
          <circle class="edit-handle-dot" :cx="drawing.end.x" :cy="drawing.end.y" :r="5 / scale" />
        </g>
      </template>
      <g
        v-if="drawingStore.selectedIds.includes(drawing.id) && (drawing.type === 'rect' || drawing.type === 'ellipse' || drawing.type === 'text')"
        class="edit-handle"
        @mousedown="editDrawing($event, drawing.id, 'resize')"
      >
        <circle class="edit-handle-hit" :cx="resizeHandlePoint(drawing).x" :cy="resizeHandlePoint(drawing).y" :r="11 / scale" />
        <circle class="edit-handle-dot" :cx="resizeHandlePoint(drawing).x" :cy="resizeHandlePoint(drawing).y" :r="5 / scale" />
      </g>
    </g>
  </svg>
</template>

<style scoped>
.drawing-layer {
  position: absolute;
  pointer-events: none;
  overflow: visible;
  contain: layout style;
}

.drawing-layer :deep(.drawing-stroke) {
  vector-effect: non-scaling-stroke;
  pointer-events: stroke;
  cursor: var(--cursor-move);
}

.drawing-layer :deep(.drawing-shape) {
  pointer-events: stroke;
}

.drawing-text {
  cursor: var(--cursor-move);
}

.drawing-text-hit {
  fill: transparent;
  pointer-events: all;
}

.drawing-text-bounds {
  fill: rgba(59, 130, 246, 0.04);
  stroke: rgba(59, 130, 246, 0.78);
  stroke-width: 1.4;
  stroke-dasharray: 4 3;
  vector-effect: non-scaling-stroke;
  pointer-events: none;
}

.drawing-text-content {
  font-weight: 500;
  white-space: pre;
  pointer-events: none;
  user-select: none;
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

.edit-handle {
  pointer-events: all;
  cursor: var(--cursor-pointer);
}

.edit-handle-hit {
  fill: transparent;
  stroke: transparent;
}

.edit-handle-dot {
  fill: #ffffff;
  stroke: #2563eb;
  stroke-width: 2;
  vector-effect: non-scaling-stroke;
}
</style>
