# 贴境 NoteScape

Windows 桌面端便签画布应用，基于 Tauri 2、Vue 3、TypeScript、Pinia 和 pnpm 开发。

## 开发命令

```bash
pnpm install
pnpm tauri dev
pnpm tauri build
```

当前版本使用本地 SQLite 数据库文件持久化数据。首次启动会在应用用户数据目录创建 `notescape.sqlite3`，设置面板中可选择已有数据库或新建数据库；应用会记住上次指定的数据库文件，下次启动自动使用。

## 常用快捷键

- `Ctrl+F`：聚焦搜索
- `Ctrl+N`：新建便签
- `Ctrl+S`：手动保存
- `Ctrl+C` / `Ctrl+V`：复制/粘贴便签
- `Ctrl+D`：复制选中便签
- `Ctrl+Z` / `Ctrl+Y`：撤销/重做
- `Ctrl+Plus` / `Ctrl+Minus` / `Ctrl+0`：缩放
- `Delete`：删除选中便签
- `Space + Drag` / 中键拖拽：平移画布
