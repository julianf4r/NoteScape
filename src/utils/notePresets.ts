import type { NoteColor, NoteDecoration } from "../types";

export interface RandomNoteStyle {
  color: NoteColor;
  width: number;
  height: number;
  decoration: NoteDecoration;
}

const noteColors: NoteColor[] = ["yellow", "blue", "pink", "green", "purple", "white", "grid-pink", "grid-white"];
const noteDecorations: NoteDecoration[] = ["none", "pin", "tape", "double-tape", "paperclip", "corner-tape"];

function randomItem<T>(items: T[]) {
  return items[Math.floor(Math.random() * items.length)];
}

function randomStep(min: number, max: number, step: number) {
  const steps = Math.floor((max - min) / step);
  return min + Math.floor(Math.random() * (steps + 1)) * step;
}

export function randomNoteStyle(): RandomNoteStyle {
  return {
    color: randomItem(noteColors),
    width: randomStep(225, 360, 5),
    height: randomStep(205, 370, 5),
    decoration: randomItem(noteDecorations),
  };
}
