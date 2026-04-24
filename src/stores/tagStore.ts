import { defineStore } from "pinia";
import { nanoid } from "nanoid";
import type { TagItem } from "../types";

const palette = ["#3b82f6", "#22c55e", "#facc15", "#8b5cf6", "#ec4899"];

export const useTagStore = defineStore("tag", {
  state: () => ({
    tags: [] as TagItem[],
    activeTagId: "",
  }),
  getters: {
    activeTag: (state) => state.tags.find((tag) => tag.id === state.activeTagId),
  },
  actions: {
    setTags(tags: TagItem[]) {
      this.tags = tags;
    },
    createTag(name = "新标签") {
      this.tags.push({
        id: nanoid(),
        name,
        color: palette[this.tags.length % palette.length],
        count: 0,
        createdAt: new Date().toISOString(),
      });
    },
    renameTag(id: string, name: string) {
      const tag = this.tags.find((item) => item.id === id);
      if (tag && name.trim()) tag.name = name.trim();
    },
    deleteTag(id: string) {
      this.tags = this.tags.filter((item) => item.id !== id);
      if (this.activeTagId === id) this.activeTagId = "";
    },
    toggleTag(id: string) {
      this.activeTagId = this.activeTagId === id ? "" : id;
    },
    recalculateCounts(noteTags: string[]) {
      this.tags.forEach((tag) => {
        tag.count = noteTags.filter((id) => id === tag.id).length;
      });
    },
  },
});
