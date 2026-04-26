export type NoteColor =
  | "yellow"
  | "blue"
  | "pink"
  | "green"
  | "purple"
  | "white"
  | "grid-pink"
  | "grid-white";

export interface ChecklistItem {
  id: string;
  text: string;
  checked: boolean;
}

export interface StickyNote {
  id: string;
  canvasId: string;
  title?: string;
  content: string;
  contentJson?: unknown;
  x: number;
  y: number;
  width: number;
  height: number;
  color: NoteColor;
  rotation: number;
  zIndex: number;
  pinned?: boolean;
  tags: string[];
  fontSize: number;
  fontWeight: "normal" | "medium" | "bold";
  textAlign: "left" | "center";
  decoration?: "none" | "pin" | "tape";
  checkedItems?: ChecklistItem[];
  createdAt: string;
  updatedAt: string;
}

export interface CanvasItem {
  id: string;
  name: string;
  description?: string;
  createdAt: string;
  updatedAt: string;
  deletedAt?: string | null;
  viewport?: ViewportState;
}

export interface TagItem {
  id: string;
  name: string;
  color: string;
  count: number;
  createdAt: string;
}

export interface AppSettings {
  theme: "light";
  defaultFontSize: number;
  showGrid: boolean;
  randomRotation: boolean;
  noteShadow: boolean;
  autoSave: boolean;
  chineseFontFamily: string;
  englishFontFamily: string;
  monospaceFontFamily: string;
}

export interface AppData {
  version: number;
  canvases: CanvasItem[];
  notes: StickyNote[];
  tags: TagItem[];
  settings: AppSettings;
}

export interface ViewportState {
  offsetX: number;
  offsetY: number;
  scale: number;
}

export interface HistoryEntry {
  id: string;
  type: "create" | "update" | "delete";
  before?: StickyNote;
  after?: StickyNote;
  timestamp: string;
}
