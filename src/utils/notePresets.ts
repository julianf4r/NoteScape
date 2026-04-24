import type { NoteColor } from "../types";

export interface NoteStylePreset {
  color: NoteColor;
  width: number;
  height: number;
  decoration: "none" | "pin" | "tape";
}

export const noteStylePresets: NoteStylePreset[] = [
  { color: "yellow", width: 240, height: 220, decoration: "none" },
  { color: "blue", width: 180, height: 170, decoration: "pin" },
  { color: "pink", width: 160, height: 150, decoration: "none" },
  { color: "green", width: 145, height: 130, decoration: "none" },
  { color: "purple", width: 150, height: 140, decoration: "none" },
  { color: "white", width: 245, height: 260, decoration: "tape" },
  { color: "grid-pink", width: 230, height: 220, decoration: "tape" },
  { color: "grid-white", width: 150, height: 145, decoration: "tape" },
];

export function randomNotePreset() {
  return noteStylePresets[Math.floor(Math.random() * noteStylePresets.length)];
}
