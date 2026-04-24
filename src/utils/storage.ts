import type { AppData, AppSettings, CanvasItem, NoteColor, StickyNote, TagItem } from "../types";
import { invoke } from "@tauri-apps/api/core";

export interface DatabaseLoadResult {
  data: string;
  db_path: string;
}

const now = () => new Date().toISOString();

const canvases: CanvasItem[] = ["工作规划", "产品设计", "学习笔记", "旅行计划", "生活清单"].map((name, index) => ({
  id: `canvas-${index + 1}`,
  name,
  createdAt: now(),
  updatedAt: now(),
  deletedAt: null,
}));

const baseNote = (
  id: string,
  content: string,
  x: number,
  y: number,
  width: number,
  height: number,
  color: NoteColor,
  rotation: number,
  zIndex: number,
  tags: string[] = [],
  fontSize = 19,
): StickyNote => ({
  id,
  canvasId: "canvas-1",
  content,
  x,
  y,
  width,
  height,
  color,
  rotation,
  zIndex,
  tags,
  fontSize,
  fontWeight: "normal",
  textAlign: "left",
  createdAt: now(),
  updatedAt: now(),
});

export const defaultSettings: AppSettings = {
  theme: "light",
  defaultNoteColor: "yellow",
  defaultFontSize: 18,
  showGrid: true,
  randomRotation: true,
  noteShadow: true,
  autoSave: true,
};

export function createDefaultData(): AppData {
  const tags: TagItem[] = [
    { id: "tag-work", name: "工作", color: "#3b82f6", count: 6, createdAt: now() },
    { id: "tag-study", name: "学习", color: "#22c55e", count: 4, createdAt: now() },
    { id: "tag-life", name: "生活", color: "#facc15", count: 3, createdAt: now() },
    { id: "tag-idea", name: "创意", color: "#8b5cf6", count: 2, createdAt: now() },
    { id: "tag-inspire", name: "灵感", color: "#ec4899", count: 5, createdAt: now() },
  ];

  return {
    version: 1,
    canvases,
    tags,
    settings: defaultSettings,
    notes: [
      baseNote("note-plan", "Q2 工作规划\n\n✓ 完成产品原型设计\n✓ 用户调研\n○ 团队协作优化\n○ 上线前测试", 120, 80, 260, 260, "yellow", -2.5, 10, ["tag-work"], 21),
      baseNote("note-feedback", "用户反馈：\n希望增加\n夜间模式\n和快捷键", 520, 90, 168, 164, "blue", 1.2, 12, ["tag-inspire"]),
      baseNote("note-review", "下周三\n项目评审会", 780, 150, 138, 142, "pink", -0.5, 8, ["tag-work"], 19),
      baseNote("note-competitor", "参考竞品：\n• Notion\n• Miro\n• Sketch", 1040, 280, 128, 118, "green", 0.6, 7, ["tag-work"], 14),
      baseNote("note-style", "设计风格\n简约 / 清晰\n现代 / 专业", 485, 360, 132, 130, "purple", 1, 9, ["tag-idea"], 16),
      baseNote("note-idea", "💡\n新功能\n创意", 250, 460, 124, 112, "blue", -0.4, 6, ["tag-idea"], 16),
      baseNote("note-todo", "待办事项\n\n□ 完善 PRD 文档\n□ 设计评审\n□ 技术方案确认", 260, 660, 210, 218, "grid-pink", -3, 11, ["tag-work"], 16),
      baseNote("note-timeline", "项目时间线\n\n5.20  需求分析\n5.30  原型设计\n6.10  开发阶段\n6.25  测试优化\n7.01  正式上线", 725, 420, 250, 260, "white", 2.2, 13, ["tag-work"], 17),
      baseNote("note-goal", "团队目标：\n提升用户体验\n提高产品质量\n\n  ☺", 560, 730, 155, 160, "yellow", -1, 5, ["tag-work"], 16),
      baseNote("note-coffee", "记得买咖啡\n\n  ☕", 920, 810, 145, 140, "grid-white", 1, 4, ["tag-life"], 15),
    ],
  };
}

export function parseData(raw: string): AppData {
  try {
    const parsed = JSON.parse(raw) as AppData;
    if (!parsed.version || !Array.isArray(parsed.canvases) || !Array.isArray(parsed.notes)) {
      throw new Error("Invalid app data");
    }
    return parsed;
  } catch {
    return createDefaultData();
  }
}

export async function loadData(): Promise<DatabaseLoadResult> {
  return invoke<DatabaseLoadResult>("load_app_data", {
    defaultData: JSON.stringify(createDefaultData()),
  });
}

export async function saveData(data: AppData) {
  await invoke("save_app_data", { data: JSON.stringify(data) });
}

export async function switchDatabase(dbPath: string, fallbackData: AppData): Promise<DatabaseLoadResult> {
  return invoke<DatabaseLoadResult>("set_database_path", {
    dbPath,
    fallbackData: JSON.stringify(fallbackData),
  });
}

export function debounce<T extends (...args: never[]) => void>(fn: T, wait = 300) {
  let timer: number | undefined;
  return (...args: Parameters<T>) => {
    window.clearTimeout(timer);
    timer = window.setTimeout(() => fn(...args), wait);
  };
}
