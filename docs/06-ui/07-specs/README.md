---
id: 07-specs.README
title: Screen Specification (SSPEC) Standard — AI-Consumable Layout & Interaction Spec
status: draft
owner: presentation-architect
created: 2026-06-22
tags:
  - ui
  - screen-spec
  - ai-consumable
  - ssot
  - figma-replacement
  - widget
  - ascii-wireframe
  - flexbox
---

# Screen Specification (SSPEC) Standard — AI-Consumable Layout & Interaction Spec

> **职责**: @presentation-architect | **上游**: ADR-066 (Screen Spec 决策), ADR-055 (UI 架构), `docs/06-ui/` (运行时架构)
> **宪法依据**: 第九编 (UI 系统宪法 — Screen Spec 强制), 第十八编 (工程质量 — Screen Metrics)
> **执行计划**: `docs/11-refactor/ui-screen-spec-execution-plan.md`

---

## 1. 设计目的

### 1.1 为什么需要 Screen Spec

50 万行 SRPG 项目，UI 的复杂度在**组合逻辑**而非**视觉保真度**。当前 `06-ui/` 运行时架构（Projection / ViewModel / Widget Contract / Dirty<T>）定义的是 UI **如何运行**，但没有定义 UI **长什么样、怎么布局、交互区域在哪**。

Screen Specification (SSPEC) 填补这个空白：**在 AI 生成 UI 代码前，给 AI 一份完整的布局+交互规范**，使 AI 首次生成的 UI 代码正确率从 ~40% 提升到 ~80%。

### 1.2 两位一体约束

```
Screen = Layout（布局结构）  →  SSPEC 文件定义
Widget = Implementation（渲染实现） → 运行时 Widget Contract 定义

两个文件，一个不可见的屏幕：
- SSPEC 描述"布局长什么样"——给 AI 看
- Widget Contract 描述"组件怎么交互"——给代码用
- 二者必须一致：SSPEC 中的 widget_id 对应 Widget Contract 中的 UiBinding
```

**核心约束**:
- SSPEC 只描述布局结构，不描述渲染实现
- Widget 只实现渲染交互，不改变布局
- widget_id 是二者的连接点：SSPEC 标注位置，Widget 实现行为

### 1.3 零 Figma 策略

本项目**零 Figma**。所有 UI 布局设计通过纯文本工具链完成：

| 职能 | 替代工具 | 必需性 |
|------|---------|--------|
| 页面布局设计 | ASCII Wireframe (Markdown 代码块) | 每个 SSPEC 必有 |
| 组件视觉样式 | Theme Token (StyleToken / UiColor / UiSpacing) | 已有 `theme-localization.md` |
| 尺寸约束 | Flexbox YAML (width/height/flex_grow/intent) | 每个 SSPEC 必有 |
| 交互流程 | Widget Contract + Event Contract | 每个 SSPEC 必有 |
| 低保真草图 | Excalidraw / draw.io (可选) | 仅复杂布局探索阶段 |

---

## 2. 目录结构

```
07-specs/
├── README.md                         ← 本文件（总纲 + AI 规则 + DoD）
├── screen-spec-template.md           ← 完整模板（17 个字段）
├── screens/                          ← 每个 Screen 一个文件
│   ├── main_menu_screen.md           # P0-01: 最简单，先验证模板
│   ├── battle_screen.md              # P0-02: 最核心，最大工作量
│   ├── inventory_screen.md           # P1-01
│   ├── settings_screen.md            # P1-02
│   ├── shop_screen.md                # P1-03（Spec 先于代码）
│   └── save_load_screen.md           # P1-04（Spec 先于代码）
└── references/                       ← 跨 Screen 统一参考
    ├── widget-id-map.md              # Widget ID → UiBinding 映射总表
    ├── z-layer-spec.md               # Z-Layer 统一规范
    ├── layout-intent-library.md      # Layout Intent 跨 Screen 参考库
    └── screen-metrics-baseline.md    # 所有 Screen 的 metrics 基线
```

---

## 3. AI 生成规则（14 条）

> AI（Claude/ChatGPT/Gemini）在生成任何 UI 代码前，**必须逐条遵守以下规则**。这些规则的目的是确保 AI 输出的 UI 代码与 SSPEC 规范一致，不引入架构违规。

| # | 规则 | 等级 | 违反后果 |
|---|------|------|---------|
| R01 | **必须先读 SSPEC 再生成代码** — 每个 Screen 的 UI 代码生成前，必须读取对应的 `07-specs/screens/{name}.md` 文件 | P0 | 布局错误率 +300% |
| R02 | **不得新增/删除/修改 widget_id** — SSPEC 中定义的 region 数量、widget_id 名称是最终权威。AI 不得擅自增减区域或修改 ID | P0 | 布局与 Widget Contract 失联 |
| R03 | **必须使用 Flexbox Layout 定义的尺寸** — width/height/flex_grow/intent 必须精确实现，不得自行推断或调整 | P0 | 尺寸溢出/挤压/不对齐 |
| R04 | **不得硬编码视觉样式** — 所有颜色/字体/间距必须引用 StyleToken / UiColor / UiSpacing / UiTypography，不得使用 `Color::srgb()` 或 `TextStyle` 直接值 | P0 | 主题不可切换 |
| R05 | **不得硬编码用户可见文本** — 所有用户可见字符串必须使用 `UiTextKey` / `LocalizationKey`，不得使用 `Text::new("...")` | P0 | 多语言失效 |
| R06 | **必须实现所有 State Mapping 状态** — SSPEC 中每个 region 定义的 Loading / Empty / Normal / Error 状态必须全部实现 | P0 | 状态遗漏导致空指针/白屏 |
| R07 | **必须实现所有 Interaction Zones** — SSPEC 中定义的 Click / Hover / Drag / Drop 区域必须全部实现交互响应 | P0 | 交互遗漏 |
| R08 | **不得在 Screen 中写业务逻辑** — Screen 负责 Widget 组合和布局，不得包含条件分支/数值计算/状态机等业务逻辑 | P0 | 架构违规 (INV-UI-005) |
| R09 | **必须遵守 Screen Lifecycle** — OnEnter / OnReady / Active / OnExit 四个阶段的行为必须精确实现 | P0 | 资源泄漏/状态错误 |
| R10 | **必须实现 Focus Navigation** — SSPEC 中定义的 Tab 导航路径必须完整实现，不得遗漏焦点区域 | P1 | 键盘/手柄不可操作 |
| R11 | **不得破坏单向数据流** — UI 只能消费 ViewModel，不得直接 Query Domain 数据；交互必须通过 UiCommand / UiAction 传递 | P0 | 架构违规 (宪法第九编) |
| R12 | **必须遵守 Widget Contract** — 每个 Widget 的 Inputs/Outputs/States 必须与 widget-atoms.md / widget-composites.md 定义的契约一致 | P0 | Widget 行为不可预测 |
| R13 | **Widget 嵌套深度不得超过 6 层** — 从 Screen root 到最深叶子 Widget，嵌套层数 > 6 时必须重构 | P1 | 性能下降/难以维护 |
| R14 | **不得在 Overlay 中嵌套 Screen 逻辑** — Tooltip / Modal / Notification / Popup 是独立层级，不得在其中放置页面级导航或业务逻辑 | P0 | Overlay 误用 |

### 3.1 遵守优先级

当多条规则冲突时，按以下优先级裁决：
1. R04 > R03 > R02 — 视觉一致性 > 布局精确性 > ID 稳定性
2. R08 > R11 > R12 — 架构隔离 > 数据流 > 组件契约
3. R01 **始终最高** — 先读 Spec 是其他所有规则的前提

---

## 4. SSPEC 范围约束

### 4.1 SSPEC 必须描述

- 信息架构（每个区域展示什么信息）
- Widget 结构（区域间的组合关系）
- Flexbox 布局（尺寸/方向/弹性约束）
- 交互区域（点击/悬停/拖拽范围）
- 状态映射（每个区域的 Loading/Empty/Normal/Error）
- 数据流（UI 消费哪些 ViewModel，发射哪些 UiCommand）

### 4.2 SSPEC 禁止描述

- **视觉样式**（颜色、渐变、阴影、圆角、边框）→ 归于 Theme Token 系统
- **字体**（字号、字重、行高）→ 归于 Theme Token 系统
- **间距**（内边距、外边距）→ 归于 Theme Token 系统
- **动画**（过渡、缓动）→ 归于 Animation Ownership 规范
- **具体 Widget 实现细节**（Button 的 hover 颜色变化逻辑）→ 归于 Widget Contract

---

## 5. Definition of Done — 14 项验证清单

每个 Screen Spec 文件完成时必须通过以下 14 项检查。AI 生成 SSPEC 后，**必须逐项确认通过**才能标记为 `status: active`。

| # | 检查项 | 验证方式 | P0/P1 |
|---|--------|---------|-------|
| D01 | **ASCII Wireframe 存在** — 纯文本线框图，所有区域已命名（region_id） | 人工审查 | P0 |
| D02 | **无匿名面板** — 线框图中每个矩形区域都有 widget_id 标注 | 人工审查 | P0 |
| D03 | **Widget Tree 完整** — 从 Screen root 到每个叶子 Widget，树结构完整无隐藏节点 | 人工审查 | P0 |
| D04 | **每个 widget_id 在 Widget Tree 中存在** — Flexbox Layout / Region Responsibility / State Mapping 中引用的所有 widget_id 都能在 Widget Tree 中找到 | 交叉校验 | P0 |
| D05 | **Flexbox Layout 完整** — 每个 widget_id 都有 direction / width / height / flex_grow / intent | 人工审查 | P0 |
| D06 | **Responsive Rules 已定义** — 至少包含 "strategy: none" | 人工审查 | P0 |
| D07 | **Region Responsibility 已定义** — 每个 region 3-8 条职责，不含空 region | 人工审查 | P0 |
| D08 | **Widget Contract 已定义** — Inputs / Outputs / Selection Model 完整 | 人工审查 | P0 |
| D09 | **State Mapping 完整** — 每个 region 的 Loading / Empty / Normal / Error 状态都已定义 | 交叉校验 | P0 |
| D10 | **Focus Navigation 已定义** — Tab 导航路径完整，无遗漏焦点区域 | 人工审查 | P1 |
| D11 | **Interaction Zones 已定义** — Click / Hover / Drag / Drop 区域已标注 | 人工审查 | P0 |
| D12 | **Overlay Definition 已定义** — Overlay 列表 + Z-Layer 已标注 | 人工审查 | P0 |
| D13 | **Lifecycle 已定义** — OnEnter / OnReady / OnExit 行为完整 | 人工审查 | P0 |
| D14 | **Data Ownership 已定义** — Owns / Uses 区分正确 | 人工审查 | P0 |
| D15 | **Layout Intent 已定义** — 每个关键尺寸有理由说明 | 人工审查 | P1 |
| D16 | **Scroll & Overflow Policy 已定义** — 每个滚动区域有 policy | 人工审查 | P1 |
| D17 | **Event Contract 已定义** — UI -> Domain 事件 + Domain -> UI 事件完整 | 人工审查 | P1 |
| D18 | **Screen Metrics 已定义** — widget_count / container_count / interactive_count / overlay_count / max_depth | 人工审查 | P1 |

### 5.1 通过标准

- **P0 字段** (D01-D14): 全部通过 → SSPEC 可标记 `status: active`
- **P1 字段** (D15-D18): 标记为 `wip:` 备注，不阻塞 `status: active`

---

## 6. SSPEC 文件生命周期

```
draft → review → active → deprecated
  │        │        │         │
  │        │        │         └── 不再维护，标记 deprecated 日期
  │        │        │
  │        │        └── 已完成 DoD 14 项检查，可被 AI 消费
  │        │
  │        └── 提交审查中，pending DoD 检查
  │
  └── 初始创建，未完成
```

| 阶段 | 前置条件 | 操作 |
|------|---------|------|
| `draft` | 文件已创建 | 所有 P0 字段完成后可进入 review |
| `review` | P0 字段全部完成 | 提交架构委员会审查（至少 @presentation-architect 审批） |
| `active` | DoD 14 项全通过，review 批准 | AI 可消费 |
| `deprecated` | Screen 被删除或重构 | 保留文件，frontmatter 标注 deprecated + deactivated_date |

---

## 7. 维护规则

- **SSPEC 是 SSOT**（Single Source of Truth）: Screen 的真实布局、交互、数据流以 SSPEC 为准，**不依赖 Figma / GUI 设计工具**
- **widget_id 一旦分配，永久有效**: 重构时标记 deprecated，不重新分配同一 ID 给不同的 UI 元素
- **SSPEC 先于代码**: 新增 Screen 必须先写 SSPEC（status: draft），通过 DoD 检查后再实现代码
- **SSPEC 与代码同步**: Screen 重构时，必须先更新 SSPEC，再修改代码
- **SSPEC 不替代现有文档**: `06-ui/03-screens/screens.md` 和其他运行时架构文档继续有效，SSPEC 是对它们的补充而非替代

---

*本文档由 @presentation-architect 维护。07-specs/ 目录所有文件的初始 status 为 draft，完成 DoD 14 项检查后改为 active。*
