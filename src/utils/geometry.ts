import type { ViewportState } from "../types";

export function screenToWorld(clientX: number, clientY: number, viewport: ViewportState, rect?: DOMRect) {
  const left = rect?.left ?? 0;
  const top = rect?.top ?? 0;
  return {
    x: (clientX - left - viewport.offsetX) / viewport.scale,
    y: (clientY - top - viewport.offsetY) / viewport.scale,
  };
}

export function worldToScreen(x: number, y: number, viewport: ViewportState) {
  return {
    x: x * viewport.scale + viewport.offsetX,
    y: y * viewport.scale + viewport.offsetY,
  };
}

export function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}
