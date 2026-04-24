<script setup lang="ts">
import { Hand, Minus, Plus, RotateCcw, RotateCw, Settings2, SlidersHorizontal, StickyNote } from "lucide-vue-next";

defineProps<{
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
  toggleHand: [];
  settings: [];
}>();
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
    <button class="zoom" title="重置缩放" @click="emit('resetZoom')">{{ Math.round(scale * 100) }}%</button>
    <button title="放大" @click="emit('zoomIn')"><Plus :size="18" /></button>
    <span></span>
    <button title="设置" @click="emit('settings')"><SlidersHorizontal :size="20" /></button>
    <button title="显示设置" @click="emit('settings')"><Settings2 :size="19" /></button>
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
  overflow: hidden;
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

.zoom {
  width: 72px;
  font-weight: 600;
}
</style>
