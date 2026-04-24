# 贴境 NoteScape

Windows 桌面端便签画布应用，基于 Tauri 2、Vue 3、TypeScript、Pinia 和 pnpm 开发。

## 开发命令

```bash
pnpm install
pnpm tauri dev
pnpm tauri build
```

当前版本使用 `localStorage` 持久化数据，键名为 `sticky-canvas-data`。
