import type { NoteColor } from "../types";

export interface NoteStylePreset {
  color: NoteColor;
  width: number;
  height: number;
  decoration: "none" | "pin" | "tape";
}

export const noteStylePresets: NoteStylePreset[] = [
  { color: "yellow", width: 300, height: 260, decoration: "none" },
  { color: "blue", width: 240, height: 220, decoration: "pin" },
  { color: "pink", width: 230, height: 210, decoration: "none" },
  { color: "green", width: 220, height: 200, decoration: "pin" },
  { color: "purple", width: 225, height: 205, decoration: "pin" },
  { color: "white", width: 305, height: 320, decoration: "tape" },
  { color: "grid-pink", width: 290, height: 270, decoration: "tape" },
  { color: "grid-white", width: 230, height: 210, decoration: "tape" },
  { color: "yellow", width: 280, height: 240, decoration: "pin" },
  { color: "blue", width: 265, height: 235, decoration: "none" },
  { color: "pink", width: 275, height: 250, decoration: "tape" },
  { color: "white", width: 255, height: 225, decoration: "none" },
];

export function randomNotePreset() {
  return noteStylePresets[Math.floor(Math.random() * noteStylePresets.length)];
}
