export type NoteColor =
  | "yellow"
  | "blue"
  | "pink"
  | "green"
  | "purple"
  | "white"
  | "grid-pink"
  | "grid-white";

export type NoteDecoration = "none" | "pin" | "tape" | "double-tape" | "paperclip" | "corner-tape";

export type DrawingTool = "select" | "pen" | "arrow" | "line" | "rect" | "ellipse";

export interface DrawingPoint {
  x: number;
  y: number;
}

export interface DrawingItem {
  id: string;
  canvasId: string;
  type: DrawingTool;
  points?: DrawingPoint[];
  start?: DrawingPoint;
  end?: DrawingPoint;
  x?: number;
  y?: number;
  width?: number;
  height?: number;
  color: string;
  strokeWidth: number;
  zIndex: number;
  createdAt: string;
  updatedAt: string;
}

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
  decoration?: NoteDecoration;
  checkedItems?: ChecklistItem[];
  createdAt: string;
  updatedAt: string;
}

export interface CanvasImage {
  id: string;
  canvasId: string;
  fileName: string;
  originalName?: string;
  contentHash?: string;
  x: number;
  y: number;
  width: number;
  height: number;
  rotation: number;
  zIndex: number;
  pinned?: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface CanvasItem {
  id: string;
  name: string;
  description?: string;
  createdAt: string;
  updatedAt: string;
  sortOrder: number;
  deletedAt?: string | null;
  viewport?: ViewportState;
}

export interface TagItem {
  id: string;
  name: string;
  color: string;
  count: number;
  sortOrder: number;
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
  imageLibraryPath: string;
}

export interface AppData {
  version: number;
  canvases: CanvasItem[];
  notes: StickyNote[];
  drawings: DrawingItem[];
  images: CanvasImage[];
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

export interface DrawingHistoryEntry {
  id: string;
  type: "create" | "update" | "delete";
  before?: DrawingItem;
  after?: DrawingItem;
  timestamp: string;
}

export interface ImageHistoryEntry {
  id: string;
  type: "create" | "update" | "delete";
  before?: CanvasImage;
  after?: CanvasImage;
  timestamp: string;
}
