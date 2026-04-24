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
  x: number;
  y: number;
  width: number;
  height: number;
  color: NoteColor;
  rotation: number;
  zIndex: number;
  tags: string[];
  fontSize: number;
  fontWeight: "normal" | "medium" | "bold";
  textAlign: "left" | "center";
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
}

export interface TagItem {
  id: string;
  name: string;
  color: string;
  count: number;
  createdAt: string;
}

export interface AppSettings {
  theme: "light" | "dark" | "system";
  defaultNoteColor: NoteColor;
  defaultFontSize: number;
  showGrid: boolean;
  randomRotation: boolean;
  noteShadow: boolean;
  autoSave: boolean;
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
