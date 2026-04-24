<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { AlignCenter, AlignLeft, Bold, ChevronsUp, Copy, Minus, Palette, Pin, Plus, Tags, Trash2 } from "lucide-vue-next";
import type { NoteColor, StickyNote, TagItem } from "../types";
import { noteColorList, noteColors } from "../utils/colors";

const props = defineProps<{
  note: StickyNote;
  selected: boolean;
  editing: boolean;
  shadow: boolean;
  scale: number;
  tags: TagItem[];
  searchQuery: string;
  highlighted: boolean;
}>();

const emit = defineEmits<{
  select: [event: MouseEvent];
  edit: [];
  update: [patch: Partial<StickyNote>, track?: boolean];
  live: [patch: Partial<StickyNote>];
  delete: [];
  duplicate: [];
  front: [];
  context: [event: MouseEvent];
  editingDone: [];
  toggleTag: [tagId: string];
}>();

const textarea = ref<HTMLTextAreaElement>();
const dragStart = ref<{ x: number; y: number; before: StickyNote }>();
const resizeStart = ref<{ x: number; y: number; before: StickyNote }>();
const draft = ref(props.note.content);
const showTags = ref(false);

const highlightedContent = computed(() => {
  const query = props.searchQuery.trim();
  if (!query) return [{ text: props.note.content, match: false }];
  const lowerContent = props.note.content.toLowerCase();
  const lowerQuery = query.toLowerCase();
  const parts: Array<{ text: string; match: boolean }> = [];
  let index = 0;
  let matchIndex = lowerContent.indexOf(lowerQuery);
  while (matchIndex >= 0) {
    if (matchIndex > index) parts.push({ text: props.note.content.slice(index, matchIndex), match: false });
    parts.push({ text: props.note.content.slice(matchIndex, matchIndex + query.length), match: true });
    index = matchIndex + query.length;
    matchIndex = lowerContent.indexOf(lowerQuery, index);
  }
  if (index < props.note.content.length) parts.push({ text: props.note.content.slice(index), match: false });
  return parts.length ? parts : [{ text: props.note.content, match: false }];
});

const style = computed(() => ({
  left: `${props.note.x}px`,
  top: `${props.note.y}px`,
  width: `${props.note.width}px`,
  height: `${props.note.height}px`,
  zIndex: props.selected ? props.note.zIndex + 1000 : props.note.zIndex,
  backgroundColor: noteColors[props.note.color],
  transform: `rotate(${props.note.rotation}deg)`,
  fontSize: `${props.note.fontSize}px`,
  fontWeight: props.note.fontWeight === "medium" ? 500 : props.note.fontWeight,
  textAlign: props.note.textAlign,
}));

watch(
  () => props.editing,
  async (editing) => {
    if (editing) {
      draft.value = props.note.content;
      await nextTick();
      textarea.value?.focus();
    }
  },
);

function startDrag(event: MouseEvent) {
  if (props.editing || (event.target as HTMLElement).closest(".note-actions, .resize-handle")) return;
  emit("select", event);
  dragStart.value = { x: event.clientX, y: event.clientY, before: { ...props.note } };
  window.addEventListener("mousemove", drag);
  window.addEventListener("mouseup", endDrag, { once: true });
}

function drag(event: MouseEvent) {
  if (!dragStart.value) return;
  emit("live", {
    x: dragStart.value.before.x + (event.clientX - dragStart.value.x) / props.scale,
    y: dragStart.value.before.y + (event.clientY - dragStart.value.y) / props.scale,
  });
}

function endDrag() {
  window.removeEventListener("mousemove", drag);
  if (dragStart.value) {
    emit("update", { x: props.note.x, y: props.note.y, __before: dragStart.value.before } as Partial<StickyNote>, true);
  }
  dragStart.value = undefined;
}

function startResize(event: MouseEvent) {
  event.stopPropagation();
  resizeStart.value = { x: event.clientX, y: event.clientY, before: { ...props.note } };
  window.addEventListener("mousemove", resize);
  window.addEventListener("mouseup", endResize, { once: true });
}

function resize(event: MouseEvent) {
  if (!resizeStart.value) return;
  emit("live", {
    width: Math.min(800, Math.max(120, resizeStart.value.before.width + (event.clientX - resizeStart.value.x) / props.scale)),
    height: Math.min(600, Math.max(100, resizeStart.value.before.height + (event.clientY - resizeStart.value.y) / props.scale)),
  });
}

function endResize() {
  window.removeEventListener("mousemove", resize);
  if (resizeStart.value) {
    emit("update", { width: props.note.width, height: props.note.height, __before: resizeStart.value.before } as Partial<StickyNote>, true);
  }
  resizeStart.value = undefined;
}

function saveEdit() {
  emit("update", { content: draft.value }, true);
  emit("editingDone");
}

function changeColor(color: NoteColor) {
  emit("update", { color }, true);
}

function changeFontSize(delta: number) {
  emit("update", { fontSize: Math.min(32, Math.max(12, props.note.fontSize + delta)) }, true);
}

function setFontSize(event: Event) {
  const value = Number((event.target as HTMLInputElement).value);
  if (Number.isFinite(value)) emit("update", { fontSize: Math.min(32, Math.max(12, value)) }, true);
}
</script>

<template>
  <article
    class="sticky-note"
    :class="[`color-${props.note.color}`, { selected: props.selected, editing: props.editing, noShadow: !props.shadow, highlighted: props.highlighted }]"
    :style="style"
    @mousedown.left="startDrag"
    @dblclick.stop="emit('edit')"
    @contextmenu.prevent.stop="emit('context', $event)"
  >
    <div v-if="props.note.id === 'note-feedback'" class="pin"><Pin :size="22" /></div>
    <div v-if="['note-todo', 'note-timeline', 'note-coffee'].includes(props.note.id)" class="tape"></div>

    <textarea
      v-if="props.editing"
      ref="textarea"
      v-model="draft"
      placeholder="输入内容..."
      @mousedown.stop
      @keydown.esc.prevent.stop="saveEdit"
      @blur="saveEdit"
    ></textarea>
    <div v-else class="content">
      <template v-for="(part, index) in highlightedContent" :key="index">
        <mark v-if="part.match">{{ part.text }}</mark>
        <template v-else>{{ part.text }}</template>
      </template>
    </div>

    <div v-if="props.selected && !props.editing" class="note-actions">
      <button title="颜色"><Palette :size="15" /></button>
      <span class="swatches">
        <button v-for="color in noteColorList" :key="color" class="swatch" :style="{ backgroundColor: noteColors[color] }" @click="changeColor(color)"></button>
      </span>
      <button title="减小字号" @click="changeFontSize(-1)"><Minus :size="14" /></button>
      <input class="font-input" type="number" min="12" max="32" :value="props.note.fontSize" @change="setFontSize" />
      <button title="增大字号" @click="changeFontSize(1)"><Plus :size="14" /></button>
      <button title="加粗" @click="emit('update', { fontWeight: props.note.fontWeight === 'bold' ? 'normal' : 'bold' }, true)"><Bold :size="15" /></button>
      <button title="左对齐" :class="{ active: props.note.textAlign === 'left' }" @click="emit('update', { textAlign: 'left' }, true)"><AlignLeft :size="15" /></button>
      <button title="居中" :class="{ active: props.note.textAlign === 'center' }" @click="emit('update', { textAlign: 'center' }, true)"><AlignCenter :size="15" /></button>
      <button title="置顶" @click="emit('front')"><ChevronsUp :size="15" /></button>
      <button title="标签" :class="{ active: showTags }" @click="showTags = !showTags"><Tags :size="15" /></button>
      <button title="复制" @click="emit('duplicate')"><Copy :size="15" /></button>
      <button title="删除" @click="emit('delete')"><Trash2 :size="15" /></button>
    </div>

    <div v-if="props.selected && !props.editing && showTags" class="tag-panel">
      <button
        v-for="tag in props.tags"
        :key="tag.id"
        :class="{ active: props.note.tags.includes(tag.id) }"
        @click="emit('toggleTag', tag.id)"
      >
        <i :style="{ backgroundColor: tag.color }"></i>
        <span>{{ tag.name }}</span>
      </button>
      <p v-if="!props.tags.length">暂无标签</p>
    </div>

    <span v-if="props.selected && !props.editing" class="resize-handle" @mousedown="startResize"></span>
  </article>
</template>

<style scoped>
.sticky-note {
  position: absolute;
  padding: 22px 24px;
  color: #1f2937;
  border-radius: 2px 2px 8px 8px;
  box-shadow: 0 10px 18px rgba(0, 0, 0, 0.14), 0 2px 4px rgba(0, 0, 0, 0.08);
  transform-origin: center center;
  cursor: grab;
  user-select: none;
}

.sticky-note.noShadow {
  box-shadow: none;
}

.sticky-note.selected {
  outline: 2px solid rgba(59, 130, 246, 0.76);
  outline-offset: 3px;
}

.sticky-note.highlighted {
  animation: pulse-note 1.4s ease;
}

.sticky-note:active {
  cursor: grabbing;
}

.content {
  width: 100%;
  height: 100%;
  white-space: pre-wrap;
  overflow: hidden;
  line-height: 1.58;
  font-family: "Segoe Print", "Comic Sans MS", "Microsoft YaHei", cursive;
}

mark {
  padding: 0 2px;
  background: rgba(250, 204, 21, 0.55);
  border-radius: 3px;
  color: inherit;
}

@keyframes pulse-note {
  0%,
  100% {
    outline-color: rgba(59, 130, 246, 0.76);
  }
  35% {
    outline-color: rgba(236, 72, 153, 0.9);
    box-shadow: 0 16px 30px rgba(59, 130, 246, 0.2), 0 2px 5px rgba(0, 0, 0, 0.1);
  }
}

textarea {
  width: 100%;
  height: 100%;
  padding: 0;
  resize: none;
  border: 0;
  outline: 0;
  color: inherit;
  background: transparent;
  line-height: 1.55;
  font-family: "Segoe Print", "Comic Sans MS", "Microsoft YaHei", cursive;
  font-size: inherit;
}

.color-grid-pink,
.color-grid-white {
  background-image: linear-gradient(rgba(255,255,255,0.24) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.24) 1px, transparent 1px);
  background-size: 18px 18px;
}

.note-actions {
  position: absolute;
  left: 50%;
  bottom: calc(100% + 12px);
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px;
  max-width: 360px;
  padding: 5px;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 9px;
  box-shadow: var(--shadow-md);
  transform: translateX(-50%);
}

.note-actions button {
  width: 26px;
  height: 26px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: #374151;
  background: transparent;
  border-radius: 6px;
}

.note-actions button:hover {
  background: #f1f5f9;
}

.note-actions button.active {
  color: #1d4ed8;
  background: #e8f1ff;
}

.note-actions .swatch {
  width: 18px;
  height: 18px;
  border: 1px solid rgba(0, 0, 0, 0.12);
  border-radius: 50%;
}

.swatches {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding-right: 3px;
  border-right: 1px solid #eef1f4;
}

.font-input {
  width: 42px;
  height: 26px;
  padding: 0 3px;
  color: #374151;
  text-align: center;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  outline: 0;
  background: #fff;
}

.resize-handle {
  position: absolute;
  right: -7px;
  bottom: -7px;
  width: 14px;
  height: 14px;
  background: #3b82f6;
  border: 2px solid #fff;
  border-radius: 50%;
  cursor: nwse-resize;
}

.tag-panel {
  position: absolute;
  left: 50%;
  top: calc(100% + 12px);
  min-width: 150px;
  max-width: 210px;
  padding: 6px;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 9px;
  box-shadow: var(--shadow-md);
  transform: translateX(-50%);
}

.tag-panel button {
  width: 100%;
  min-height: 30px;
  display: grid;
  grid-template-columns: 16px 1fr;
  align-items: center;
  gap: 7px;
  padding: 0 8px;
  color: #374151;
  background: transparent;
  border-radius: 7px;
  text-align: left;
}

.tag-panel button:hover,
.tag-panel button.active {
  background: #eef2f7;
}

.tag-panel i {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.tag-panel span {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.tag-panel p {
  margin: 6px;
  color: #9ca3af;
  font-size: 13px;
}

.tape {
  position: absolute;
  top: -10px;
  left: 50%;
  width: 66px;
  height: 28px;
  background: rgba(226, 202, 165, 0.55);
  transform: translateX(-50%) rotate(2deg);
}

.pin {
  position: absolute;
  top: -18px;
  left: 50%;
  display: grid;
  place-items: center;
  width: 22px;
  height: 22px;
  color: #0369a1;
  transform: translateX(-50%) rotate(45deg);
}
</style>
