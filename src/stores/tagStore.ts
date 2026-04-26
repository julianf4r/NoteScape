import { defineStore } from "pinia";
import { nanoid } from "nanoid";
import type { TagItem } from "../types";
import { deleteTagData, reportPersistenceError, saveTagData } from "../utils/storage";

const palette = ["#3b82f6", "#22c55e", "#facc15", "#8b5cf6", "#ec4899"];

export const useTagStore = defineStore("tag", {
  state: () => ({
    tags: [] as TagItem[],
    activeTagId: "",
  }),
  getters: {
    activeTag: (state) => state.tags.find((tag) => tag.id === state.activeTagId),
    orderedTags: (state) => [...state.tags].sort(compareTagOrder),
  },
  actions: {
    setTags(tags: TagItem[]) {
      this.tags = normalizeTagOrder(tags);
    },
    createTag(name = "新标签") {
      const bottomOrder = Math.max(-1, ...this.orderedTags.map((tag) => tag.sortOrder)) + 1;
      const tag = {
        id: nanoid(),
        name,
        color: palette[this.tags.length % palette.length],
        count: 0,
        sortOrder: bottomOrder,
        createdAt: new Date().toISOString(),
      };
      this.tags.push(tag);
      void saveTagData(tag).catch((error) => reportPersistenceError("保存标签", error));
      return tag;
    },
    renameTag(id: string, name: string) {
      const tag = this.tags.find((item) => item.id === id);
      if (tag && name.trim()) {
        tag.name = name.trim();
        void saveTagData(tag).catch((error) => reportPersistenceError("保存标签", error));
      }
    },
    deleteTag(id: string) {
      this.tags = this.tags.filter((item) => item.id !== id);
      if (this.activeTagId === id) this.activeTagId = "";
      void deleteTagData(id).catch((error) => reportPersistenceError("删除标签", error));
    },
    updateTagColor(id: string, color: string) {
      const tag = this.tags.find((item) => item.id === id);
      if (tag) {
        tag.color = color;
        void saveTagData(tag).catch((error) => reportPersistenceError("保存标签", error));
      }
    },
    toggleTag(id: string) {
      this.activeTagId = this.activeTagId === id ? "" : id;
    },
    recalculateCounts(noteTags: string[]) {
      this.tags.forEach((tag) => {
        tag.count = noteTags.filter((id) => id === tag.id).length;
      });
    },
    moveTag(id: string, direction: -1 | 1) {
      const ordered = this.orderedTags;
      const index = ordered.findIndex((tag) => tag.id === id);
      const targetIndex = index + direction;
      if (index < 0 || targetIndex < 0 || targetIndex >= ordered.length) return;
      const reordered = [...ordered];
      const [tag] = reordered.splice(index, 1);
      reordered.splice(targetIndex, 0, tag);
      reordered.forEach((item, order) => {
        item.sortOrder = order;
      });
      this.tags = reordered;
      reordered.forEach((item) => {
        void saveTagData(item).catch((error) => reportPersistenceError("保存标签顺序", error));
      });
    },
  },
});

function compareTagOrder(a: TagItem, b: TagItem) {
  return tagSortOrder(a) - tagSortOrder(b) || a.createdAt.localeCompare(b.createdAt);
}

function normalizeTagOrder(tags: TagItem[]) {
  const ordered = [...tags].sort(compareTagOrder);
  ordered.forEach((tag, index) => {
    if (!Number.isFinite(tag.sortOrder)) tag.sortOrder = index;
  });
  return ordered;
}

function tagSortOrder(tag: TagItem) {
  return Number.isFinite(tag.sortOrder) ? tag.sortOrder : Number.MAX_SAFE_INTEGER;
}
