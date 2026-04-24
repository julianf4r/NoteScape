# NoteScape 开发规范

## 1. 项目目标

开发一款 Windows 桌面端便签画布软件 NoteScape，使用 **Tauri 2 + Vue 3 + TypeScript + pnpm** 实现。

软件核心形态为：一个现代、简约、美观、轻微拟物的无限画布应用。左侧为画布管理和标签管理区域，右侧为大画布区域。用户可以在画布上自由创建、移动、缩放、编辑不同颜色和尺寸的便签，并支持搜索、缩放、设置、标签管理等基础扩展功能。

界面视觉必须高度接近参考图，目标相似度要求：**99% 接近参考 UI**。

---

## 2. 技术栈要求

### 2.1 基础技术

- 桌面框架：Tauri 2
- 前端框架：Vue 3
- 构建工具：Vite
- 包管理器：pnpm
- 语言：TypeScript
- 样式：CSS / SCSS / CSS Modules 均可，推荐使用原生 CSS 变量 + scoped style
- 状态管理：Pinia
- 本地数据存储：
  - 第一阶段可使用 `localStorage`
  - 推荐实现为 Tauri 文件系统持久化 JSON
  - 数据文件建议存储在 AppData 目录下

### 2.2 推荐依赖

```bash
pnpm add pinia nanoid lucide-vue-next
pnpm add -D sass
pnpm tauri add fs
pnpm tauri add dialog
````

---

## 3. 项目初始化要求

推荐项目结构：

```text
NoteScape/
├─ package.json
├─ pnpm-lock.yaml
├─ index.html
├─ vite.config.ts
├─ tsconfig.json
├─ src/
│  ├─ main.ts
│  ├─ App.vue
│  ├─ assets/
│  ├─ components/
│  │  ├─ AppShell.vue
│  │  ├─ Sidebar.vue
│  │  ├─ CanvasBoard.vue
│  │  ├─ StickyNote.vue
│  │  ├─ CanvasToolbar.vue
│  │  ├─ MiniMap.vue
│  │  ├─ SearchBox.vue
│  │  └─ SettingsPanel.vue
│  ├─ stores/
│  │  ├─ canvasStore.ts
│  │  ├─ noteStore.ts
│  │  ├─ tagStore.ts
│  │  └─ settingsStore.ts
│  ├─ types/
│  │  └─ index.ts
│  ├─ utils/
│  │  ├─ storage.ts
│  │  ├─ geometry.ts
│  │  └─ colors.ts
│  └─ styles/
│     ├─ variables.css
│     └─ global.css
└─ src-tauri/
   ├─ tauri.conf.json
   ├─ Cargo.toml
   └─ src/
      └─ main.rs
```

---

## 4. 视觉规范

### 4.1 总体风格

界面需要做到：

* 现代
* 简约
* 大气
* 轻微拟物
* 干净明亮
* 接近 Windows 原生桌面应用
* 不要过度卡通化
* 不要过度玻璃拟态
* 不要暗色赛博风
* 不要使用明显网页后台管理系统风格

整体视觉参考：

* Windows 11 应用窗口
* Notion / Miro / FigJam 的简洁感
* 真实便利贴的轻微纸张质感
* 浅灰背景
* 柔和阴影
* 圆角卡片
* 清晰但不刺眼的颜色

---

## 5. 窗口与布局

### 5.1 主窗口

默认窗口尺寸：

```ts
width: 1400
height: 1050
aspectRatio: 4 / 3
```

最低窗口尺寸：

```ts
minWidth: 1000
minHeight: 720
```

窗口应显示为 Windows 桌面软件截图风格：

* 顶部保留系统标题栏
* 应用名称：`贴境`
* 左上角显示简单黄色图标
* 右上角保留 Windows 最小化、最大化、关闭按钮
* 不强制自定义标题栏，除非能保证非常接近参考图

---

## 6. 主界面结构

### 6.1 总体布局

```text
┌────────────────────────────────────────────┐
│ Windows 标题栏                              │
├───────────────┬────────────────────────────┤
│ 左侧侧边栏     │ 右侧无限画布                 │
│               │                            │
│ 画布管理       │ 便签、工具栏、小地图          │
│ 标签管理       │                            │
│ 设置入口       │                            │
└───────────────┴────────────────────────────┘
```

### 6.2 左侧侧边栏

宽度：`280px`

背景色：`#f7f7f8`

右侧边框：`1px solid #e5e7eb`

内容包括：

1. 顶部菜单按钮
2. 搜索框
3. 画布列表
4. 新建画布按钮
5. 回收站入口
6. 标签列表
7. 设置按钮

视觉要求：

* 选中画布使用浅蓝色背景
* 左侧有蓝色竖线强调
* 列表项高度约 42px
* 图标使用线性图标
* 文字颜色不要纯黑，使用 `#374151`
* 次要时间文字使用 `#9ca3af`

---

## 7. 画布区域规范

### 7.1 画布背景

右侧主区域为无限画布。

背景要求：

```css
background-color: #fbfaf8;
background-image: radial-gradient(#d8d5cf 1px, transparent 1px);
background-size: 18px 18px;
```

要求：

* 画布必须支持横向和纵向拖拽平移
* 鼠标滚轮默认上下滚动
* 按住 `Space + 鼠标拖动` 平移画布
* 鼠标中键拖动也可平移
* `Ctrl + 鼠标滚轮` 缩放画布
* 缩放范围：`25% ~ 300%`
* 默认缩放：`100%`
* 画布元素的位置使用世界坐标，不要直接绑定屏幕坐标

### 7.2 画布工具栏

右上角浮动工具栏，视觉接近参考图。

位置：

```css
position: absolute;
top: 16px;
right: 16px;
```

内容：

* 撤销
* 重做
* 拖拽/手型工具
* 缩放比例显示
* 缩放下拉
* 设置/调节按钮

样式：

* 白色背景
* 圆角 `10px`
* 阴影 `0 4px 14px rgba(0,0,0,0.08)`
* 每个按钮宽度约 `48px`
* 分隔线使用 `#eeeeee`

### 7.3 小地图

左下角浮动小地图。

位置：

```css
position: absolute;
left: 12px;
bottom: 20px;
```

功能：

* 显示当前画布整体缩略图
* 显示当前视口范围
* 支持点击小地图跳转视口
* 下方显示缩放控制：`- 100% +`
* 支持全屏/适应视图按钮

---

## 8. 便签功能规范

### 8.1 便签基础字段

```ts
export interface StickyNote {
  id: string
  canvasId: string
  title?: string
  content: string
  x: number
  y: number
  width: number
  height: number
  color: NoteColor
  rotation: number
  zIndex: number
  tags: string[]
  fontSize: number
  fontWeight: 'normal' | 'medium' | 'bold'
  textAlign: 'left' | 'center'
  checkedItems?: ChecklistItem[]
  createdAt: string
  updatedAt: string
}

export type NoteColor =
  | 'yellow'
  | 'blue'
  | 'pink'
  | 'green'
  | 'purple'
  | 'white'
  | 'grid-pink'
  | 'grid-white'

export interface ChecklistItem {
  id: string
  text: string
  checked: boolean
}
```

### 8.2 便签视觉

便签必须模拟真实便利贴，但不要过度写实。

通用样式：

```css
.sticky-note {
  position: absolute;
  border-radius: 2px 2px 8px 8px;
  box-shadow:
    0 10px 18px rgba(0, 0, 0, 0.14),
    0 2px 4px rgba(0, 0, 0, 0.08);
  padding: 20px 24px;
  color: #1f2937;
  transform-origin: center center;
}
```

颜色参考：

```ts
const noteColors = {
  yellow: '#ffe982',
  blue: '#9fd2f3',
  pink: '#f8aeb8',
  green: '#dcefa6',
  purple: '#c8b6e8',
  white: '#f8f5ec',
  gridPink: '#f6c8d4',
  gridWhite: '#f9f7f0'
}
```

要求：

* 每个便签可有轻微旋转，范围 `-3deg ~ 3deg`
* 新建便签默认轻微随机旋转
* 便签阴影方向统一偏右下
* 部分便签可以带胶带装饰
* 部分便签可以带图钉装饰
* 便签可以大可小
* 便签不能全部对齐，要自然分布
* 便签内容默认使用类似手写风格，但必须清晰可读
* 推荐字体：

  * UI：`Inter`, `Segoe UI`, `Microsoft YaHei`
  * 便签内容：`Segoe Print`, `Comic Sans MS`, `Microsoft YaHei`

---

## 9. 便签交互

### 9.1 新建便签

支持方式：

1. 双击画布空白处新建便签
2. 右键画布打开菜单，新建便签
3. 工具栏按钮新建便签

新建默认值：

```ts
{
  width: 220,
  height: 180,
  color: 'yellow',
  content: '新便签',
  fontSize: 18,
  rotation: random(-2, 2)
}
```

### 9.2 编辑文字

交互要求：

* 双击便签进入编辑模式
* 编辑模式下显示 textarea 或 contenteditable
* 支持多行文本
* `Enter` 换行
* `Esc` 退出编辑
* 点击便签外部自动保存
* 编辑中不能触发画布拖拽
* 内容为空时显示 placeholder：`输入内容...`

### 9.3 移动便签

要求：

* 单击选中便签
* 按住便签拖动可移动
* 拖动过程中提高 z-index
* 松开鼠标后保存位置
* 支持多选后整体移动

### 9.4 调整大小

要求：

* 选中便签后显示 resize handles
* 至少支持右下角缩放
* 推荐支持四角缩放
* 最小尺寸：`120 x 100`
* 最大尺寸：`800 x 600`
* 调整尺寸后自动保存

### 9.5 便签样式调整

选中便签后显示轻量浮动样式面板。

功能包括：

* 修改颜色
* 修改字体大小
* 加粗
* 左对齐 / 居中
* 添加 / 移除标签
* 删除便签
* 置顶
* 复制便签

### 9.6 右键菜单

便签右键菜单：

* 编辑
* 复制
* 删除
* 置顶
* 更改颜色
* 添加标签

画布右键菜单：

* 新建便签
* 粘贴便签
* 适应视图
* 重置缩放

---

## 10. 画布管理

### 10.1 画布字段

```ts
export interface CanvasItem {
  id: string
  name: string
  description?: string
  createdAt: string
  updatedAt: string
  deletedAt?: string | null
}
```

### 10.2 功能要求

左侧画布列表支持：

* 新建画布
* 重命名画布
* 删除画布到回收站
* 恢复画布
* 永久删除画布
* 切换当前画布
* 显示更新时间，例如：

  * 刚刚
  * 2 小时前
  * 昨天
  * 3 天前
  * 上周

默认创建以下示例画布：

```text
工作规划
产品设计
学习笔记
旅行计划
生活清单
```

---

## 11. 标签管理

### 11.1 标签字段

```ts
export interface TagItem {
  id: string
  name: string
  color: string
  count: number
  createdAt: string
}
```

默认标签：

```text
工作 #3b82f6
学习 #22c55e
生活 #facc15
创意 #8b5cf6
灵感 #ec4899
```

### 11.2 功能要求

* 新建标签
* 重命名标签
* 删除标签
* 给便签添加标签
* 根据标签筛选便签
* 标签旁显示数量
* 标签颜色显示为圆点

---

## 12. 搜索功能

搜索框位于左侧侧边栏顶部。

功能要求：

* 搜索画布名称
* 搜索便签内容
* 搜索标签名称
* 输入时实时过滤
* 匹配内容高亮
* 快捷键：`Ctrl + F`
* 搜索结果点击后跳转到对应便签，并居中显示

搜索框视觉：

```text
搜索画布...        Ctrl+F
```

---

## 13. 设置功能

左下角设置入口。

设置面板可使用抽屉或弹窗。

设置项：

* 主题：

  * 浅色
  * 深色，可选，第一阶段可不完整实现
  * 跟随系统，可选
* 默认便签颜色
* 默认字体大小
* 是否显示网格点
* 是否启用便签随机旋转
* 是否启用便签阴影
* 自动保存开关
* 数据导出
* 数据导入
* 重置示例数据

---

## 14. 数据持久化

### 14.1 数据结构

```ts
export interface AppData {
  version: number
  canvases: CanvasItem[]
  notes: StickyNote[]
  tags: TagItem[]
  settings: AppSettings
}

export interface AppSettings {
  theme: 'light' | 'dark' | 'system'
  defaultNoteColor: NoteColor
  defaultFontSize: number
  showGrid: boolean
  randomRotation: boolean
  noteShadow: boolean
  autoSave: boolean
}
```

### 14.2 存储方式

第一阶段允许使用：

```ts
localStorage.setItem('sticky-canvas-data', JSON.stringify(data))
```

正式实现推荐：

```text
AppData/sticky-canvas/data.json
```

要求：

* 启动时读取数据
* 无数据时初始化示例数据
* 每次编辑后自动保存
* 保存操作应 debounce，建议 300ms
* 数据损坏时自动备份损坏文件，并重新初始化

---

## 15. 示例数据要求

首次启动时，应生成与参考图类似的示例画布。

当前画布：`工作规划`

示例便签：

1. 黄色大便签

```text
Q2 工作规划

✓ 完成产品原型设计
✓ 用户调研
○ 团队协作优化
○ 上线前测试
```

2. 蓝色便签

```text
用户反馈：
希望增加
夜间模式
和快捷键
```

3. 粉色便签

```text
下周三
项目评审会
```

4. 白色大便签

```text
项目时间线

5.20  需求分析
5.30  原型设计
6.10  开发阶段
6.25  测试优化
7.01  正式上线
```

5. 绿色便签

```text
参考竞品：
• Notion
• Miro
• Sketch
```

6. 紫色便签

```text
设计风格
简约 / 清晰
现代 / 专业
```

7. 粉色网格便签

```text
待办事项

□ 完善 PRD 文档
□ 设计评审
□ 技术方案确认
```

8. 黄色小便签

```text
团队目标：
提升用户体验
提高产品质量
```

9. 白色网格便签

```text
记得买咖啡
```

---

## 16. 组件职责

### 16.1 AppShell.vue

负责整体布局：

* 标题栏下方主结构
* 左侧 Sidebar
* 右侧 CanvasBoard
* 全局快捷键注册
* 数据加载状态

### 16.2 Sidebar.vue

负责：

* 搜索框
* 画布列表
* 标签列表
* 回收站入口
* 设置入口

### 16.3 CanvasBoard.vue

负责：

* 无限画布渲染
* 缩放
* 平移
* 便签容器
* 右键菜单
* 画布坐标转换
* 便签选中状态

### 16.4 StickyNote.vue

负责：

* 单个便签渲染
* 编辑文字
* 拖拽
* 缩放
* 样式面板
* 右键菜单

### 16.5 CanvasToolbar.vue

负责：

* 撤销
* 重做
* 手型工具
* 缩放显示
* 缩放调整
* 设置入口

### 16.6 MiniMap.vue

负责：

* 画布缩略图
* 当前视口显示
* 点击定位
* 缩放按钮

### 16.7 SettingsPanel.vue

负责：

* 设置项展示
* 数据导入导出
* 主题配置

---

## 17. 状态管理

### 17.1 canvasStore

负责：

* 当前画布 ID
* 画布列表
* 新建画布
* 删除画布
* 重命名画布
* 恢复画布

### 17.2 noteStore

负责：

* 当前画布便签列表
* 新建便签
* 更新便签
* 删除便签
* 复制便签
* 修改样式
* 修改位置
* 修改尺寸
* 搜索便签

### 17.3 tagStore

负责：

* 标签列表
* 新建标签
* 删除标签
* 标签计数
* 标签筛选

### 17.4 settingsStore

负责：

* 主题
* 默认便签颜色
* 默认字体大小
* 网格显示
* 数据导入导出

---

## 18. 坐标系统

画布必须使用世界坐标系统。

```ts
interface ViewportState {
  offsetX: number
  offsetY: number
  scale: number
}
```

屏幕坐标转世界坐标：

```ts
function screenToWorld(clientX: number, clientY: number, viewport: ViewportState) {
  return {
    x: (clientX - viewport.offsetX) / viewport.scale,
    y: (clientY - viewport.offsetY) / viewport.scale
  }
}
```

世界坐标转屏幕坐标：

```ts
function worldToScreen(x: number, y: number, viewport: ViewportState) {
  return {
    x: x * viewport.scale + viewport.offsetX,
    y: y * viewport.scale + viewport.offsetY
  }
}
```

便签容器可使用：

```css
.canvas-content {
  transform-origin: 0 0;
  transform: translate(var(--offset-x), var(--offset-y)) scale(var(--scale));
}
```

---

## 19. 快捷键

必须支持：

| 快捷键          | 功能          |
| ------------ | ----------- |
| Ctrl + F     | 聚焦搜索        |
| Ctrl + N     | 新建便签        |
| Ctrl + Plus  | 放大          |
| Ctrl + Minus | 缩小          |
| Ctrl + 0     | 重置缩放        |
| Delete       | 删除选中便签      |
| Esc          | 退出编辑 / 取消选择 |
| Space + Drag | 平移画布        |
| Ctrl + D     | 复制选中便签      |
| Ctrl + S     | 手动保存        |

---

## 20. 撤销 / 重做

第一阶段至少支持：

* 新建便签
* 删除便签
* 移动便签
* 修改文字
* 修改颜色
* 调整大小

实现方式：

```ts
interface HistoryEntry {
  id: string
  type: 'create' | 'update' | 'delete'
  before?: unknown
  after?: unknown
  timestamp: string
}
```

最多保存最近 50 步。

---

## 21. 交互细节要求

### 21.1 选择状态

选中便签时：

* 显示蓝色细边框
* 显示 resize handle
* z-index 提高
* 不要破坏便签拟物阴影

### 21.2 拖拽状态

拖拽时：

* 鼠标变为 grabbing
* 便签轻微放大 `scale(1.01)`
* 阴影加深
* 禁止选中文字

### 21.3 编辑状态

编辑时：

* 便签内部显示文本输入光标
* 不显示 resize handle
* 不触发拖拽
* 自动聚焦
* 保留原始换行

### 21.4 空状态

无画布时显示：

```text
还没有画布
点击“新建”开始整理你的想法
```

无便签时显示：

```text
双击画布空白处创建第一张便签
```

---

## 22. 性能要求

* 100 张便签内操作流畅
* 300 张便签内可接受
* 拖拽时不得频繁写入磁盘
* 拖拽过程中只更新内存状态
* mouseup 后保存数据
* 搜索使用 computed，不要每次重新构建复杂数据
* 大量便签时可后续加入虚拟渲染

---

## 23. UI 精度要求

开发时必须严格靠近参考图。

### 23.1 必须接近的部分

* 左侧栏宽度、层级、颜色
* 画布浅米色点阵背景
* 便签颜色、阴影、大小、随机摆放感
* 右上角浮动工具栏
* 左下角小地图
* Windows 应用窗口感
* 便签上的中文内容
* 便签轻微旋转
* 胶带、图钉、手绘图标等轻微真实感装饰

### 23.2 不允许出现的问题

* 不要做成网页后台管理系统
* 不要用过重的渐变
* 不要大面积高饱和色
* 不要玻璃拟态过度
* 不要卡通化
* 不要让便签排列过于整齐
* 不要让文字太小或太淡
* 不要让画布背景纯白
* 不要让阴影过黑

---

## 24. 推荐 CSS 变量

```css
:root {
  --app-bg: #f3f4f6;
  --sidebar-bg: #f7f7f8;
  --canvas-bg: #fbfaf8;
  --border-color: #e5e7eb;

  --text-primary: #1f2937;
  --text-secondary: #6b7280;
  --text-muted: #9ca3af;

  --primary: #3b82f6;
  --primary-soft: #e8f1ff;

  --note-yellow: #ffe982;
  --note-blue: #9fd2f3;
  --note-pink: #f8aeb8;
  --note-green: #dcefa6;
  --note-purple: #c8b6e8;
  --note-white: #f8f5ec;

  --shadow-sm: 0 2px 6px rgba(0, 0, 0, 0.08);
  --shadow-md: 0 8px 18px rgba(0, 0, 0, 0.12);
  --shadow-lg: 0 14px 28px rgba(0, 0, 0, 0.16);
}
```

---

## 25. 验收标准

### 25.1 基础功能验收

必须完成：

* 应用可在 Windows 上运行
* 左侧栏显示画布列表
* 左侧栏显示标签列表
* 可创建画布
* 可切换画布
* 可创建便签
* 可编辑便签文字
* 可拖拽便签
* 可调整便签大小
* 可修改便签颜色
* 可删除便签
* 可搜索便签
* 可缩放画布
* 可平移画布
* 数据可持久化
* 重启应用后数据仍存在

### 25.2 视觉验收

必须达到：

* 整体布局与参考图高度一致
* 画布颜色和点阵接近参考图
* 便签质感接近参考图
* 左侧栏接近参考图
* 右上角工具栏接近参考图
* 左下角小地图接近参考图
* 默认示例数据打开后应和参考图构图接近

### 25.3 交互验收

必须达到：

* 拖拽流畅
* 编辑不误触拖拽
* 缩放以鼠标位置为中心
* 搜索结果可定位到便签
* 设置项修改后立即生效
* 删除操作有确认或可撤销机制

---

## 26. 开发顺序建议

### 阶段 1：基础框架

1. 创建 Tauri 2 + Vue 3 + TypeScript 项目
2. 配置 pnpm
3. 搭建 AppShell
4. 搭建 Sidebar
5. 搭建 CanvasBoard 静态界面
6. 加入示例便签静态渲染

### 阶段 2：核心交互

1. 实现画布平移
2. 实现画布缩放
3. 实现便签拖拽
4. 实现便签编辑
5. 实现便签 resize
6. 实现便签颜色修改

### 阶段 3：数据管理

1. 接入 Pinia
2. 实现画布管理
3. 实现便签 CRUD
4. 实现标签管理
5. 实现本地持久化
6. 实现启动加载

### 阶段 4：增强功能

1. 搜索
2. 小地图
3. 设置面板
4. 撤销 / 重做
5. 导入 / 导出
6. 视觉细节打磨

---

## 27. Codex 执行要求

Codex 开发时必须遵守：

1. 不要改变技术栈
2. 不要把项目改成 Electron
3. 不要把界面做成普通网页
4. 不要省略核心交互
5. 不要只做静态 UI
6. 不要使用远程服务
7. 不要引入登录系统
8. 不要引入云同步
9. 优先完成本地可运行版本
10. 所有数据默认保存在本地

---

## 28. 最终交付物

最终项目必须包含：

* 可运行的 Tauri 2 桌面应用
* Vue 3 前端源码
* TypeScript 类型定义
* Pinia 状态管理
* 本地数据持久化
* 示例数据
* README
* 基础构建命令

README 至少包含：

```bash
pnpm install
pnpm tauri dev
pnpm tauri build
```

---

## 29. 应用名称建议

中文名：

```text
贴境
```

英文内部名：

```text
NoteScape
```

窗口标题：

```text
贴境
```

---

## 30. 结论

本项目不是普通便签列表软件，而是一个以无限画布为核心的视觉化便签管理工具。

开发优先级：

1. 高度还原参考 UI
2. 完成便签核心编辑能力
3. 完成画布平移缩放
4. 完成本地数据持久化
5. 增加搜索、设置、小地图、撤销重做等增强功能

最终效果应当像一款成熟的 Windows 桌面应用，而不是一个简单网页 Demo。
