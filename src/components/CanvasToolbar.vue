<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { ArrowUpRight, ChevronDown, Circle, Hand, ImagePlus, Minus, MousePointer2, Pencil, Plus, RotateCcw, RotateCw, SlidersHorizontal, Square, Trash2 } from "lucide-vue-next";
import type { DrawingTool } from "../types";

const props = defineProps<{
  scale: number;
  handActive: boolean;
  drawingTool: DrawingTool;
  drawingColor: string;
  drawingStrokeWidth: number;
  objectSelected: boolean;
}>();

const emit = defineEmits<{
  undo: [];
  redo: [];
  zoomIn: [];
  zoomOut: [];
  resetZoom: [];
  setZoom: [scale: number];
  setHandActive: [active: boolean];
  setDrawingTool: [tool: DrawingTool];
  setDrawingColor: [color: string];
  setDrawingStrokeWidth: [width: number];
  deleteSelected: [];
  addImage: [];
  settings: [];
}>();

const zoomOptions = [0.25, 0.5, 0.75, 1, 1.5, 2, 3];
const zoomOpen = ref(false);
const zoomMenu = ref<HTMLElement>();
const zoomLabel = computed(() => `${Math.round(props.scale * 100)}%`);
const currentMode = computed(() => (props.handActive ? "hand" : props.drawingTool === "select" ? "select" : "draw"));
const lastDrawingTool = ref<DrawingTool>(props.drawingTool === "select" ? "pen" : props.drawingTool);
const drawingTools: Array<{ tool: DrawingTool; title: string; icon: typeof MousePointer2 }> = [
  { tool: "pen", title: "自由绘制", icon: Pencil },
  { tool: "arrow", title: "箭头", icon: ArrowUpRight },
  { tool: "line", title: "直线", icon: Minus },
  { tool: "rect", title: "矩形", icon: Square },
  { tool: "ellipse", title: "椭圆", icon: Circle },
];

watch(
  () => props.drawingTool,
  (tool) => {
    if (tool !== "select") lastDrawingTool.value = tool;
  },
);

function setMode(mode: "select" | "hand" | "draw") {
  if (mode === "select") {
    emit("setHandActive", false);
    emit("setDrawingTool", "select");
    return;
  }
  if (mode === "hand") {
    emit("setHandActive", true);
    emit("setDrawingTool", "select");
    return;
  }
  emit("setHandActive", false);
  emit("setDrawingTool", lastDrawingTool.value === "select" ? "pen" : lastDrawingTool.value);
}

function selectDrawingTool(tool: DrawingTool) {
  lastDrawingTool.value = tool;
  emit("setHandActive", false);
  emit("setDrawingTool", tool);
}

function selectZoom(scale: number) {
  emit("setZoom", scale);
  zoomOpen.value = false;
}

function toggleZoom() {
  zoomOpen.value = !zoomOpen.value;
  if (!zoomOpen.value) return;
  window.addEventListener("mousedown", closeZoomOnOutside);
  window.addEventListener("keydown", closeZoomOnEscape);
}

function closeZoom() {
  zoomOpen.value = false;
  window.removeEventListener("mousedown", closeZoomOnOutside);
  window.removeEventListener("keydown", closeZoomOnEscape);
}

function closeZoomOnOutside(event: MouseEvent) {
  if (zoomMenu.value?.contains(event.target as Node)) return;
  closeZoom();
}

function closeZoomOnEscape(event: KeyboardEvent) {
  if (event.key === "Escape") closeZoom();
}

onBeforeUnmount(closeZoom);
</script>

<template>
  <div class="toolbar">
    <button title="撤销" @click="emit('undo')"><RotateCcw :size="20" /></button>
    <button title="重做" @click="emit('redo')"><RotateCw :size="20" /></button>
    <span></span>
    <div class="mode-group" aria-label="操作模式">
      <button title="选择模式" :class="{ active: currentMode === 'select' }" @click="setMode('select')"><MousePointer2 :size="19" /></button>
      <button title="手型模式" :class="{ active: currentMode === 'hand' }" @click="setMode('hand')"><Hand :size="20" /></button>
      <button title="绘制模式" :class="{ active: currentMode === 'draw' }" @click="setMode('draw')"><Pencil :size="19" /></button>
    </div>
    <span></span>
    <div class="draw-group" :class="{ muted: currentMode !== 'draw' }" aria-label="绘制工具">
      <button
        v-for="item in drawingTools"
        :key="item.tool"
        :title="item.title"
        :class="{ active: drawingTool === item.tool && currentMode === 'draw' }"
        @click="selectDrawingTool(item.tool)"
      >
        <component :is="item.icon" :size="19" />
      </button>
      <label class="drawing-color" title="绘图颜色" :style="{ '--drawing-color': drawingColor }">
        <input type="color" :value="drawingColor" @input="emit('setDrawingColor', ($event.target as HTMLInputElement).value)" />
      </label>
      <input
        class="stroke-input"
        title="线宽"
        type="number"
        min="1"
        max="16"
        :value="drawingStrokeWidth"
        @change="emit('setDrawingStrokeWidth', Number(($event.target as HTMLInputElement).value))"
      />
    </div>
    <button title="删除选中对象" :disabled="!objectSelected" @click="emit('deleteSelected')"><Trash2 :size="18" /></button>
    <button title="添加图片" @click="emit('addImage')"><ImagePlus :size="18" /></button>
    <span></span>
    <button title="缩小" @click="emit('zoomOut')"><Minus :size="18" /></button>
    <div ref="zoomMenu" class="zoom-menu" @mousedown.stop>
      <button class="zoom-trigger" title="缩放比例" @click="toggleZoom">
        <span>{{ zoomLabel }}</span>
        <ChevronDown :size="15" />
      </button>
      <div v-if="zoomOpen" class="zoom-options">
        <button
          v-for="option in zoomOptions"
          :key="option"
          :class="{ active: Math.round(scale * 100) === Math.round(option * 100) }"
          @click="selectZoom(option)"
        >
          {{ Math.round(option * 100) }}%
        </button>
      </div>
    </div>
    <button title="放大" @click="emit('zoomIn')"><Plus :size="18" /></button>
    <span></span>
    <button title="设置" @click="emit('settings')"><SlidersHorizontal :size="20" /></button>
  </div>
</template>

<style scoped>
.toolbar {
  position: absolute;
  top: 16px;
  right: 16px;
  z-index: 30;
  display: flex;
  align-items: center;
  height: 50px;
  overflow: visible;
  background: rgba(255, 255, 255, 0.96);
  border: 1px solid #eceff3;
  border-radius: 10px;
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.08);
}

button {
  width: 46px;
  height: 50px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: #374151;
  background: transparent;
}

button:hover,
button.active {
  background: #f3f6fb;
}

button:disabled {
  cursor: var(--cursor-default);
  opacity: 0.4;
}

.mode-group,
.draw-group {
  display: flex;
  align-items: center;
  height: 50px;
}

.mode-group {
  padding: 0 4px;
  gap: 3px;
}

.mode-group button {
  width: 40px;
  height: 38px;
  border-radius: 8px;
}

.draw-group.muted {
  background: rgba(248, 250, 252, 0.62);
}

.draw-group.muted button:not(.active),
.draw-group.muted .drawing-color,
.draw-group.muted .stroke-input {
  opacity: 0.68;
}

span {
  width: 1px;
  height: 50px;
  background: #eeeeee;
}

.zoom-menu {
  position: relative;
}

.zoom-trigger {
  width: 78px;
  gap: 5px;
  font-weight: 600;
}

.zoom-trigger span {
  width: auto;
  height: auto;
  background: transparent;
}

.zoom-options {
  position: absolute;
  top: calc(100% + 6px);
  left: 50%;
  z-index: 50;
  width: 88px;
  padding: 5px;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  box-shadow: var(--shadow-md);
  transform: translateX(-50%);
}

.zoom-options button {
  width: 100%;
  height: 30px;
  justify-content: center;
  border-radius: 6px;
  font-size: 13px;
}

.zoom-options button.active {
  color: #1d4ed8;
  background: #e8f1ff;
}

.drawing-color {
  position: relative;
  width: 46px;
  height: 50px;
  display: inline-grid;
  place-items: center;
}

.drawing-color::before {
  content: "";
  width: 20px;
  height: 20px;
  border: 1px solid #d1d5db;
  border-radius: 50%;
  background: var(--drawing-color);
}

.drawing-color input {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  opacity: 0;
  cursor: var(--cursor-pointer);
}

.stroke-input {
  width: 42px;
  height: 28px;
  margin: 0 2px;
  color: #374151;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  text-align: center;
  outline: 0;
}
</style>
