<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { AlignCenter, AlignLeft, Bold, ChevronsUp, Copy, FileText, Minus, Paperclip, Pin, Plus, Sparkles, Strikethrough, Tags, Trash2 } from "lucide-vue-next";
import { EditorContent, type JSONContent, useEditor } from "@tiptap/vue-3";
import { BubbleMenu } from "@tiptap/vue-3/menus";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import TextAlign from "@tiptap/extension-text-align";
import TaskList from "@tiptap/extension-task-list";
import TaskItem from "@tiptap/extension-task-item";
import Link from "@tiptap/extension-link";
import type { NoteDecoration, StickyNote, TagItem } from "../types";
import { useFeedbackStore } from "../stores/feedbackStore";
import { useSettingsStore } from "../stores/settingsStore";
import { noteColors } from "../utils/colors";

const feedback = useFeedbackStore();
const settingsStore = useSettingsStore();

const props = defineProps<{
  note: StickyNote;
  selected: boolean;
  editing: boolean;
  shadow: boolean;
  scale: number;
  tags: TagItem[];
  searchQuery: string;
  highlighted: boolean;
  panMode: boolean;
}>();

const emit = defineEmits<{
  select: [event: MouseEvent];
  edit: [];
  update: [patch: Partial<StickyNote>, track?: boolean];
  live: [patch: Partial<StickyNote>];
  delete: [];
  duplicate: [];
  copyText: [];
  front: [];
  context: [event: MouseEvent, payload?: { linkHref?: string; codeText?: string }];
  editingDone: [];
  toggleTag: [tagId: string];
}>();

const dragStart = ref<{ x: number; y: number; before: StickyNote }>();
const resizeStart = ref<{ x: number; y: number; before: StickyNote }>();
const draft = ref(props.note.content);
const showTags = ref(false);
const showDecorations = ref(false);
const heightLimited = ref(false);
const limitNoticeShown = ref(false);
const decoration = computed(() => props.note.decoration ?? "none");
const frontTitle = computed(() => (props.note.pinned ? "取消置顶" : "置顶"));
const maxNoteHeight = 1000;

const decorationOptions: Array<{ value: NoteDecoration; label: string }> = [
  { value: "none", label: "无" },
  { value: "tape", label: "胶带" },
  { value: "pin", label: "图钉" },
  { value: "double-tape", label: "双胶带" },
  { value: "paperclip", label: "回形针" },
  { value: "corner-tape", label: "角贴" },
];

function textToDoc(text: string): JSONContent {
  return {
    type: "doc",
    content: (text ? text.split("\n") : [""]).map((line) => ({
      type: "paragraph",
      content: line ? [{ type: "text", text: line }] : undefined,
    })),
  };
}

function buildFontFamily(...groups: string[]) {
  return groups.flatMap(parseFontList).join(", ");
}

function parseFontList(value: string) {
  return value
    .split(",")
    .map((font) => font.trim().replace(/^["']|["']$/g, ""))
    .filter(Boolean)
    .map(formatFontName);
}

function formatFontName(font: string) {
  const genericFamilies = new Set(["serif", "sans-serif", "monospace", "cursive", "fantasy", "system-ui"]);
  if (genericFamilies.has(font.toLowerCase())) return font;
  return `"${font.replace(/"/g, '\\"')}"`;
}

const editor = useEditor({
  content: (props.note.contentJson as JSONContent | undefined) ?? textToDoc(props.note.content),
  editable: props.editing,
  extensions: [
    StarterKit.configure({ link: false }),
    Placeholder.configure({ placeholder: "输入内容..." }),
    TextAlign.configure({ types: ["heading", "paragraph"] }),
    TaskList,
    TaskItem.configure({ nested: true }),
    Link.configure({ openOnClick: false, autolink: true, linkOnPaste: true }),
  ],
  editorProps: {
    handleTextInput: () => blockLimitedEditorInput(),
    handlePaste: () => blockLimitedEditorInput(),
    handleDOMEvents: {
      blur: () => {
        if (props.editing) saveEdit();
        return false;
      },
      keydown: (_view, event) => blockLimitedInput(event as KeyboardEvent),
      beforeinput: (_view, event) => blockLimitedBeforeInput(event as InputEvent),
      paste: (_view, event) => blockLimitedPaste(event as ClipboardEvent),
    },
  },
  onUpdate: ({ editor }) => {
    draft.value = editor.getText();
    scheduleAutoGrow();
  },
});

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
  zIndex: (props.note.pinned ? 100000 : 0) + props.note.zIndex,
  backgroundColor: noteColors[props.note.color],
  transform: props.editing ? "rotate(0deg)" : `rotate(${props.note.rotation}deg)`,
}));

const contentStyle = computed(() => ({
  fontSize: `${props.note.fontSize}px`,
  fontWeight: props.note.fontWeight === "medium" ? 500 : props.note.fontWeight,
  textAlign: props.note.textAlign,
  fontFamily: buildFontFamily(
    settingsStore.settings.englishFontFamily,
    settingsStore.settings.chineseFontFamily,
    settingsStore.settings.monospaceFontFamily,
    "cursive",
  ),
  "--note-monospace-font": buildFontFamily(settingsStore.settings.monospaceFontFamily, "monospace"),
}));

const editorStyle = computed(() => ({
  ...contentStyle.value,
  overflowY: "hidden",
}));

watch(
  () => props.editing,
  async (editing, wasEditing) => {
    if (!editing && wasEditing) {
      commitEditorContent(false);
    }
    editor.value?.setEditable(editing);
    if (editing) {
      draft.value = props.note.content;
      await nextTick();
      editor.value?.commands.focus("end");
      autoGrowToContent();
    }
  },
);

watch(
  () => [props.note.id, props.note.contentJson, props.note.content],
  () => {
    if (!props.editing) editor.value?.commands.setContent((props.note.contentJson as JSONContent | undefined) ?? textToDoc(props.note.content));
  },
);

function startDrag(event: MouseEvent) {
  if (props.panMode || event.button === 1) {
    emit("select", event);
    return;
  }
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
    height: Math.min(maxNoteHeight, Math.max(100, resizeStart.value.before.height + (event.clientY - resizeStart.value.y) / props.scale)),
  });
}

function endResize() {
  window.removeEventListener("mousemove", resize);
  if (resizeStart.value) {
    emit("update", { width: props.note.width, height: props.note.height, __before: resizeStart.value.before } as Partial<StickyNote>, true);
  }
  resizeStart.value = undefined;
}

function commitEditorContent(finish: boolean) {
  const richEditor = editor.value;
  emit("update", { content: richEditor?.getText() ?? draft.value, contentJson: richEditor?.getJSON() }, true);
  if (finish) emit("editingDone");
}

function saveEdit() {
  commitEditorContent(true);
}

function changeFontSize(delta: number) {
  emit("update", { fontSize: Math.min(32, Math.max(12, props.note.fontSize + delta)) }, true);
}

function setFontSize(event: Event) {
  const value = Number((event.target as HTMLInputElement).value);
  if (Number.isFinite(value)) emit("update", { fontSize: Math.min(32, Math.max(12, value)) }, true);
  scheduleAutoGrow();
}

function scheduleAutoGrow() {
  if (!props.editing) return;
  void nextTick(autoGrowToContent);
}

function autoGrowToContent() {
  const editorElement = editor.value?.view.dom;
  const editorWrapper = editorElement?.parentElement;
  if (!props.editing || !editorElement || !editorWrapper) return;
  const overflow = editorElement.scrollHeight - editorWrapper.clientHeight;
  heightLimited.value = measureHeightLimited();
  if (!heightLimited.value) limitNoticeShown.value = false;
  if (overflow <= 1) return;
  const desiredHeight = Math.min(maxNoteHeight, Math.ceil(props.note.height + overflow + 8));
  if (desiredHeight > props.note.height + 1) emit("live", { height: desiredHeight });
  if (desiredHeight >= maxNoteHeight) heightLimited.value = true;
}

function blockLimitedInput(event: KeyboardEvent) {
  if (!props.editing || event.isComposing || !isTextInputKey(event) || !isHeightLimited()) return false;
  event.preventDefault();
  notifyHeightLimited();
  return true;
}

function blockLimitedBeforeInput(event: InputEvent) {
  if (!props.editing || !isInsertInput(event) || !isHeightLimited()) return false;
  event.preventDefault();
  notifyHeightLimited();
  return true;
}

function blockLimitedPaste(event: ClipboardEvent) {
  if (!props.editing || !isHeightLimited()) return false;
  event.preventDefault();
  notifyHeightLimited();
  return true;
}

function blockLimitedEditorInput() {
  if (!props.editing || !isHeightLimited()) return false;
  notifyHeightLimited();
  return true;
}

function isTextInputKey(event: KeyboardEvent) {
  if (event.ctrlKey || event.metaKey || event.altKey) return false;
  return event.key.length === 1 || event.key === "Enter" || event.key === "Tab";
}

function isInsertInput(event: InputEvent) {
  return (
    event.inputType === "insertText" ||
    event.inputType === "insertCompositionText" ||
    event.inputType === "insertParagraph" ||
    event.inputType === "insertLineBreak" ||
    event.inputType === "insertFromPaste"
  );
}

function isHeightLimited() {
  heightLimited.value = measureHeightLimited();
  if (!heightLimited.value) limitNoticeShown.value = false;
  return heightLimited.value;
}

function measureHeightLimited() {
  const editorElement = editor.value?.view.dom;
  const editorWrapper = editorElement?.parentElement;
  if (!editorElement || !editorWrapper || props.note.height < maxNoteHeight) return false;
  const contentBottom = Array.from(editorElement.children).reduce((bottom, child) => {
    const element = child as HTMLElement;
    return Math.max(bottom, element.offsetTop + element.offsetHeight);
  }, 0);
  return editorWrapper.clientHeight - contentBottom <= 8;
}

function notifyHeightLimited() {
  if (limitNoticeShown.value) return;
  limitNoticeShown.value = true;
  feedback.notify("便签已达到最大高度");
}

async function copySelectedText() {
  const richEditor = editor.value;
  const text = richEditor ? richEditor.state.doc.textBetween(richEditor.state.selection.from, richEditor.state.selection.to, "\n") : "";
  if (!text) return;
  await navigator.clipboard.writeText(text);
  feedback.notify("文字已复制", "success");
}

function stopLinkNavigation(event: MouseEvent) {
  const target = event.target;
  if (!(target instanceof HTMLElement)) return;
  if (!target.closest("a")) return;
  event.preventDefault();
  event.stopPropagation();
}

function onContextMenu(event: MouseEvent) {
  if (props.editing && (event.target as HTMLElement).closest(".tiptap")) {
    event.stopPropagation();
    return;
  }
  event.preventDefault();
  event.stopPropagation();
  const target = event.target as HTMLElement;
  const link = props.editing ? null : target.closest("a");
  const code = props.editing ? null : target.closest("pre, code");
  const codeText = code?.textContent?.trim();
  emit("context", event, {
    linkHref: link?.getAttribute("href") || link?.href,
    codeText: codeText || undefined,
  });
}

function shouldShowTextMenu({ editor: currentEditor }: { editor: { isEditable: boolean; state: { selection: { empty: boolean } } } }) {
  return props.editing && currentEditor.isEditable && !currentEditor.state.selection.empty;
}

function changeDecoration(value: NoteDecoration) {
  emit("update", { decoration: value }, true);
  showDecorations.value = false;
}

function toggleTagsPanel() {
  showTags.value = !showTags.value;
  if (showTags.value) showDecorations.value = false;
}

function toggleDecorationPanel() {
  showDecorations.value = !showDecorations.value;
  if (showDecorations.value) showTags.value = false;
}
</script>

<template>
  <article
    class="sticky-note"
    :class="[`color-${props.note.color}`, { selected: props.selected, editing: props.editing, noShadow: !props.shadow, highlighted: props.highlighted }]"
    :style="style"
    @mousedown.left="startDrag"
    @dblclick.stop="emit('edit')"
    @contextmenu="onContextMenu"
  >
    <div v-if="decoration === 'pin'" class="pin"><Pin :size="40" fill="currentColor"/></div>
    <div v-if="decoration === 'tape'" class="tape"></div>
    <div v-if="decoration === 'double-tape'" class="double-tape">
      <i></i>
      <i></i>
    </div>
    <div v-if="decoration === 'paperclip'" class="paperclip"><Paperclip :size="40" /></div>
    <div v-if="decoration === 'corner-tape'" class="corner-tape"></div>

    <template v-if="props.editing">
      <BubbleMenu
        v-if="editor"
        :editor="editor"
        :should-show="shouldShowTextMenu"
        :options="{ placement: 'top', offset: 8 }"
        class="text-menu"
        @mousedown.prevent.stop
        @dblclick.prevent.stop
      >
        <button title="加粗" :class="{ active: editor.isActive('bold') }" @click.prevent.stop="editor.chain().focus().toggleBold().run()"><Bold :size="15" /></button>
        <button title="删除线" :class="{ active: editor.isActive('strike') }" @click.prevent.stop="editor.chain().focus().toggleStrike().run()"><Strikethrough :size="15" /></button>
        <span></span>
        <button title="左对齐" :class="{ active: editor.isActive({ textAlign: 'left' }) }" @click.prevent.stop="editor.chain().focus().setTextAlign('left').run()"><AlignLeft :size="15" /></button>
        <button title="居中" :class="{ active: editor.isActive({ textAlign: 'center' }) }" @click.prevent.stop="editor.chain().focus().setTextAlign('center').run()"><AlignCenter :size="15" /></button>
        <span></span>
        <button title="复制选中文字" @click.prevent.stop="copySelectedText"><Copy :size="15" /></button>
      </BubbleMenu>
      <EditorContent class="editor-content" :editor="editor" :style="editorStyle" @mousedown.stop @click.capture="stopLinkNavigation" @keydown.esc.capture.prevent.stop="saveEdit" />
    </template>
    <div v-else class="content" :style="contentStyle">
      <template v-if="props.searchQuery.trim()" v-for="(part, index) in highlightedContent" :key="index">
        <mark v-if="part.match">{{ part.text }}</mark>
        <template v-else>{{ part.text }}</template>
      </template>
      <EditorContent v-else class="editor-content readonly" :editor="editor" @click.capture="stopLinkNavigation" />
    </div>

    <div v-if="props.selected && !props.editing" class="note-actions" @mousedown.stop @dblclick.prevent.stop>
      <button title="减小字号" @click="changeFontSize(-1)"><Minus :size="14" /></button>
      <input class="font-input" type="number" min="12" max="32" :value="props.note.fontSize" @change="setFontSize" />
      <button title="增大字号" @click="changeFontSize(1)"><Plus :size="14" /></button>
      <button :title="frontTitle" :class="{ active: props.note.pinned }" @click="emit('front')"><ChevronsUp :size="15" /></button>
      <button title="标签" :class="{ active: showTags }" @click="toggleTagsPanel"><Tags :size="15" /></button>
      <button title="装饰" :class="{ active: showDecorations }" @click="toggleDecorationPanel"><Sparkles :size="15" /></button>
      <button title="复制文字" @click="emit('copyText')"><FileText :size="15" /></button>
      <button title="复制便签" @click="emit('duplicate')"><Copy :size="15" /></button>
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

    <div v-if="props.selected && !props.editing && showDecorations" class="decoration-panel">
      <button
        v-for="option in decorationOptions"
        :key="option.value"
        :class="{ active: decoration === option.value }"
        @click="changeDecoration(option.value)"
      >
        <span :class="['decoration-preview', `preview-${option.value}`]"></span>
        <b>{{ option.label }}</b>
      </button>
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
  transition: transform 0.12s ease;
  cursor: move;
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

.sticky-note.editing {
  cursor: text;
  user-select: text;
}

.sticky-note:active {
  cursor: move;
}

.sticky-note.editing:active {
  cursor: text;
}

.content {
  width: 100%;
  height: 100%;
  white-space: pre-wrap;
  overflow: hidden;
  line-height: 1.58;
  font-family: inherit;
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

.editor-content {
  width: 100%;
  height: 100%;
  color: inherit;
  font-family: inherit;
  font-size: inherit;
  font-synthesis: weight;
}

.editor-content :deep(.tiptap) {
  min-height: 100%;
  outline: 0;
  white-space: pre-wrap;
  line-height: 1.55;
}

.sticky-note.editing .editor-content,
.sticky-note.editing .editor-content :deep(.tiptap) {
  cursor: text;
  user-select: text;
}

.editor-content :deep(p) {
  margin: 0 0 0.35em;
}

.editor-content :deep(code),
.editor-content :deep(pre) {
  font-family: var(--note-monospace-font), monospace;
  font-size: 0.92em;
  font-synthesis: none;
}

.editor-content :deep(pre) {
  margin: 0.45em 0;
  white-space: pre-wrap;
}

.editor-content :deep(ul),
.editor-content :deep(ol) {
  margin: 0.2em 0;
  padding-left: 1.25em;
}

.editor-content :deep(ul[data-type="taskList"]) {
  list-style: none;
  padding-left: 0;
}

.editor-content :deep(li[data-type="taskItem"]) {
  display: flex;
  gap: 0.45em;
}

.editor-content :deep(a) {
  color: #1d4ed8;
  text-decoration: underline;
}

.editor-content :deep(.is-editor-empty:first-child::before) {
  content: attr(data-placeholder);
  float: left;
  height: 0;
  color: rgba(31, 41, 55, 0.48);
  pointer-events: none;
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
  width: 300px;
  padding: 5px;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 9px;
  box-shadow: var(--shadow-md);
  transform: translateX(-50%);
  font-size: 13px;
  line-height: 1;
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
  font-size: 13px;
}

.note-actions button:hover {
  background: #f1f5f9;
}

.note-actions button.active,
.text-menu button.active {
  color: #1d4ed8;
  background: #e8f1ff;
}

.font-input {
  width: 42px;
  height: 26px;
  padding: 0 3px;
  color: #374151;
  font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  font-size: 13px;
  font-weight: 500;
  line-height: 1;
  text-align: center;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  outline: 0;
  background: #fff;
}

.text-menu {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 5px;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 9px;
  box-shadow: var(--shadow-md);
  font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  font-size: 13px;
  line-height: 1;
}

.text-menu button {
  width: 28px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: #374151;
  background: transparent;
  border-radius: 6px;
}

.text-menu button:hover {
  background: #f1f5f9;
}

.text-menu span {
  width: 1px;
  height: 18px;
  background: #e5e7eb;
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
  font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  font-size: 13px;
  font-weight: 400;
  line-height: 1.3;
  text-align: left;
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

.decoration-panel {
  position: absolute;
  left: 50%;
  top: calc(100% + 12px);
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 4px;
  width: 190px;
  padding: 6px;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 9px;
  box-shadow: var(--shadow-md);
  transform: translateX(-50%);
  font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  font-size: 13px;
  line-height: 1.2;
}

.decoration-panel button {
  min-height: 32px;
  display: grid;
  grid-template-columns: 26px 1fr;
  align-items: center;
  gap: 6px;
  padding: 0 7px;
  color: #374151;
  background: transparent;
  border-radius: 7px;
  text-align: left;
}

.decoration-panel button:hover,
.decoration-panel button.active {
  background: #eef2f7;
}

.decoration-panel b {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  font-weight: 500;
}

.decoration-preview {
  position: relative;
  display: block;
  width: 24px;
  height: 20px;
}

.preview-none::before {
  content: "";
  position: absolute;
  left: 5px;
  top: 9px;
  width: 14px;
  height: 2px;
  background: #cbd5e1;
}

.preview-tape::before,
.preview-double-tape::before,
.preview-double-tape::after,
.preview-corner-tape::before {
  content: "";
  position: absolute;
  background: rgba(201, 173, 128, 0.64);
}

.preview-tape::before {
  left: 4px;
  top: 6px;
  width: 16px;
  height: 8px;
  transform: rotate(4deg);
}

.preview-double-tape::before,
.preview-double-tape::after {
  top: 5px;
  width: 10px;
  height: 7px;
}

.preview-double-tape::before {
  left: 1px;
  transform: rotate(-10deg);
}

.preview-double-tape::after {
  right: 1px;
  transform: rotate(10deg);
}

.preview-pin::before {
  content: "";
  position: absolute;
  left: 7px;
  top: 3px;
  width: 10px;
  height: 10px;
  background: #0ea5e9;
  border-radius: 50%;
  box-shadow: 0 7px 0 -4px #075985;
}

.preview-paperclip::before {
  content: "";
  position: absolute;
  left: 6px;
  top: 2px;
  width: 10px;
  height: 16px;
  border: 2px solid #64748b;
  border-left-color: transparent;
  border-radius: 8px;
  transform: rotate(18deg);
}

.preview-corner-tape::before {
  right: 2px;
  top: 2px;
  width: 14px;
  height: 8px;
  transform: rotate(42deg);
}

.tape {
  position: absolute;
  top: -10px;
  left: 50%;
  width: 66px;
  height: 28px;
  background: rgba(226, 202, 165, 0.55);
  transform: translateX(-50%) rotate(2deg);
  pointer-events: none;
}

.pin {
  position: absolute;
  top: -24px;
  left: 50%;
  display: grid;
  place-items: center;
  width: 30px;
  height: 30px;
  color: #0369a1;
  transform: translateX(-50%) rotate(45deg);
  pointer-events: none;
}

.double-tape {
  pointer-events: none;
}

.double-tape i,
.corner-tape {
  position: absolute;
  width: 54px;
  height: 24px;
  background: rgba(226, 202, 165, 0.52);
}

.double-tape i:first-child {
  top: -9px;
  left: 28px;
  transform: rotate(-7deg);
}

.double-tape i:last-child {
  top: -8px;
  right: 28px;
  transform: rotate(6deg);
}

.paperclip {
  position: absolute;
  top: -18px;
  right: 24px;
  color: #64748b;
  transform: rotate(18deg);
  filter: drop-shadow(0 1px 0 rgba(255, 255, 255, 0.62));
  pointer-events: none;
}

.corner-tape {
  top: -4px;
  right: -14px;
  transform: rotate(42deg);
  pointer-events: none;
}
</style>
