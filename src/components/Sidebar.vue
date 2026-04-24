<script setup lang="ts">
import { computed, ref } from "vue";
import { Archive, FileText, Menu, Plus, Settings } from "lucide-vue-next";
import SearchBox from "./SearchBox.vue";
import { useCanvasStore } from "../stores/canvasStore";
import { useNoteStore } from "../stores/noteStore";
import { useSettingsStore } from "../stores/settingsStore";
import { useTagStore } from "../stores/tagStore";

const canvasStore = useCanvasStore();
const noteStore = useNoteStore();
const tagStore = useTagStore();
const settingsStore = useSettingsStore();
const renamingId = ref("");

const filteredCanvases = computed(() => {
  const query = canvasStore.searchQuery.trim().toLowerCase();
  if (!query) return canvasStore.activeCanvases;
  return canvasStore.activeCanvases.filter((canvas) => canvas.name.toLowerCase().includes(query));
});

function relativeTime(value: string) {
  const diff = Date.now() - new Date(value).getTime();
  const hour = 3600 * 1000;
  if (diff < 60 * 1000) return "刚刚";
  if (diff < 24 * hour) return `${Math.max(1, Math.floor(diff / hour))} 小时前`;
  if (diff < 48 * hour) return "昨天";
  if (diff < 7 * 24 * hour) return `${Math.floor(diff / (24 * hour))} 天前`;
  return "上周";
}

function createCanvas() {
  canvasStore.createCanvas();
  noteStore.clearSelection();
}
</script>

<template>
  <aside class="sidebar">
    <button class="menu icon-button" title="菜单">
      <Menu :size="22" />
    </button>

    <SearchBox v-model="canvasStore.searchQuery" />

    <section class="section">
      <div class="section-title">
        <span>画布</span>
        <button class="new-button" @click="createCanvas"><Plus :size="15" />新建</button>
      </div>

      <div v-if="!filteredCanvases.length" class="empty">还没有画布<br />点击“新建”开始整理你的想法</div>

      <button
        v-for="canvas in filteredCanvases"
        :key="canvas.id"
        class="canvas-row"
        :class="{ active: canvas.id === canvasStore.currentCanvasId }"
        @click="canvasStore.selectCanvas(canvas.id)"
        @dblclick="renamingId = canvas.id"
      >
        <FileText :size="16" />
        <input
          v-if="renamingId === canvas.id"
          :value="canvas.name"
          @click.stop
          @keydown.enter="canvasStore.renameCanvas(canvas.id, ($event.target as HTMLInputElement).value); renamingId = ''"
          @blur="canvasStore.renameCanvas(canvas.id, ($event.target as HTMLInputElement).value); renamingId = ''"
        />
        <span v-else class="name">{{ canvas.name }}</span>
        <span class="time">{{ relativeTime(canvas.updatedAt) }}</span>
      </button>

      <button class="canvas-row trash">
        <Archive :size="16" />
        <span class="name">回收站</span>
        <span class="time">{{ canvasStore.deletedCanvases.length || "" }}</span>
      </button>
    </section>

    <section class="section tags">
      <div class="section-title">
        <span>标签</span>
        <button class="small-add" @click="tagStore.createTag()"><Plus :size="18" /></button>
      </div>
      <button
        v-for="tag in tagStore.tags"
        :key="tag.id"
        class="tag-row"
        :class="{ active: tag.id === tagStore.activeTagId }"
        @click="tagStore.toggleTag(tag.id)"
      >
        <i :style="{ backgroundColor: tag.color }"></i>
        <span>{{ tag.name }}</span>
        <b>{{ tag.count }}</b>
      </button>
    </section>

    <button class="settings" @click="settingsStore.togglePanel()">
      <Settings :size="17" />
      <span>设置</span>
    </button>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 280px;
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 18px 12px 16px;
  background: var(--sidebar-bg);
  border-right: 1px solid var(--border-color);
}

.menu {
  margin: 0 0 10px 0;
}

.section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.section-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 0 6px;
  font-weight: 700;
  color: #1f2937;
}

.new-button,
.small-add {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  height: 34px;
  padding: 0 10px;
  color: #374151;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 7px;
  box-shadow: var(--shadow-sm);
}

.small-add {
  width: 34px;
  padding: 0;
}

.canvas-row,
.tag-row,
.settings {
  position: relative;
  display: grid;
  grid-template-columns: 22px 1fr auto;
  align-items: center;
  gap: 8px;
  width: 100%;
  height: 42px;
  padding: 0 10px;
  color: #374151;
  background: transparent;
  border-radius: 7px;
  text-align: left;
}

.canvas-row.active {
  color: #1d4ed8;
  background: var(--primary-soft);
}

.canvas-row.active::before {
  content: "";
  position: absolute;
  left: 0;
  width: 3px;
  height: 24px;
  border-radius: 3px;
  background: var(--primary);
}

.canvas-row:hover,
.tag-row:hover,
.settings:hover,
.tag-row.active {
  background: #eef2f7;
}

.name {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.time,
.tag-row b {
  font-weight: 500;
  font-size: 13px;
  color: var(--text-muted);
}

.canvas-row input {
  min-width: 0;
  border: 0;
  outline: 0;
  color: #374151;
  background: transparent;
}

.trash {
  margin-top: 16px;
  border-top: 1px solid #edf0f3;
  border-radius: 0;
}

.tags {
  margin-top: 14px;
}

.tag-row i {
  width: 12px;
  height: 12px;
  margin-left: 2px;
  border-radius: 50%;
}

.tag-row b {
  justify-self: end;
}

.settings {
  margin-top: auto;
  grid-template-columns: 22px 1fr;
}

.empty {
  padding: 22px 10px;
  line-height: 1.7;
  font-size: 14px;
  color: var(--text-muted);
}
</style>
