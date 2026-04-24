import type { NoteColor } from "../types";

export const noteColors: Record<NoteColor, string> = {
  yellow: "#ffe982",
  blue: "#9fd2f3",
  pink: "#f8aeb8",
  green: "#dcefa6",
  purple: "#c8b6e8",
  white: "#f8f5ec",
  "grid-pink": "#f6c8d4",
  "grid-white": "#f9f7f0",
};

export const colorNames: Record<NoteColor, string> = {
  yellow: "黄色",
  blue: "蓝色",
  pink: "粉色",
  green: "绿色",
  purple: "紫色",
  white: "白色",
  "grid-pink": "粉格",
  "grid-white": "白格",
};

export const noteColorList = Object.keys(noteColors) as NoteColor[];
