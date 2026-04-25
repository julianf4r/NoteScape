<script setup lang="ts">
import { computed, ref } from "vue";
import { ChevronDown, Hand, Minus, Plus, RotateCcw, RotateCw, SlidersHorizontal, StickyNote } from "lucide-vue-next";

const props = defineProps<{
  scale: number;
  handActive: boolean;
}>();

const emit = defineEmits<{
  add: [];
  undo: [];
  redo: [];
  zoomIn: [];
  zoomOut: [];
  resetZoom: [];
  setZoom: [scale: number];
  toggleHand: [];
  settings: [];
}>();

const zoomOptions = [0.25, 0.5, 0.75, 1, 1.5, 2, 3];
const zoomOpen = ref(false);
const zoomLabel = computed(() => `${Math.round(props.scale * 100)}%`);

function selectZoom(scale: number) {
  emit("setZoom", scale);
  zoomOpen.value = false;
}
</script>

<template>
  <div class="toolbar">
    <button title="撤销" @click="emit('undo')"><RotateCcw :size="20" /></button>
    <button title="重做" @click="emit('redo')"><RotateCw :size="20" /></button>
    <span></span>
    <button title="手型工具" :class="{ active: handActive }" @click="emit('toggleHand')"><Hand :size="20" /></button>
    <button title="新建便签" @click="emit('add')"><StickyNote :size="20" /></button>
    <span></span>
    <button title="缩小" @click="emit('zoomOut')"><Minus :size="18" /></button>
    <div class="zoom-menu" @mousedown.stop @mouseleave="zoomOpen = false">
      <button class="zoom-trigger" title="缩放比例" @click="zoomOpen = !zoomOpen">
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
</style>
