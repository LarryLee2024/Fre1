---
id: UI-CONSTITUTION
title: UI 系统宪法
status: accepted
stability: stable
layer: ui
related:
  - ai-constitution-complete.md
tags:
  - ui
  - screen
  - widget
  - factory
  - bsn
---

> **原文来源**：`ai-constitution-complete.md` 第九编（L978-L1067）
> **锚定总宪法**：第九编

## 第九编 UI 系统宪法
- 🟩 **技术选型**：产品 UI 优先使用 bevy_ui，开发工具 UI 优先使用 bevy_egui
- 🟩 **状态单向流动**
  - UI 只能展示业务状态，🟥 绝对禁止保存业务真相
  - 🟥 绝对禁止业务逻辑直接操作 UI 组件
  - 🟥 绝对禁止在 UI 系统中修改业务状态
  - UI 必须通过监听状态变化自动刷新
- 🟩 **模块化**：UI 必须作为独立的 Feature 模块存在
- 🟩 **临时状态隔离**
  - 选中单位、悬停格子、技能预览等 UI 交互状态，属于表现层临时状态
  - 绝对禁止混入业务事实状态，不参与存档、不进入回放
- 🟩 **BSN 作用域限制**（依据 ADR-054 DR-003 + ADR-056 DR-008）
  - 🟩 BSN 仅允许用于：`src/app/scenes/`（Composition Root）、Editor Prototype、Debug UI
  - 🟩 BSN 使用范围中的内容必须保持无状态、无逻辑、无业务语义
  - 🟥 `src/ui/screens/` 禁止直接使用 BSN 构建 Screen
  - 🟥 `src/ui/widgets/` 禁止直接使用 BSN 构建 Widget
- 🟩 **Widget/Screen Factory 方案**
  - 🟩 所有 Screen/Widget 必须通过 Factory 构建：`spawn_xxx(commands, props)` 或 `XxxFactory`
  - 🟩 Factory 是 UI 的唯一构建入口
  - 🟩 Factory 输入仅限：Props、ViewModel、Theme；禁止直接读取 Domain
  - 🟩 Factory 必须满足：可测试、可复用、可审查、可被 AI 独立生成

### 第 X 条：Primitives 隔离

UI 代码必须遵守三层依赖方向：

```
primitives/ → theme/     （允许）
widgets/   → primitives/ （允许，禁止直接访问 Bevy UI 类型）
screens/   → widgets/ + primitives/ （允许，通过 Factory）
```

- 🟩 `primitives/` 是 UI 层唯一允许直接操作 Bevy UI 底层类型（Node、Button、Interaction、BackgroundColor）的模块
- 🟩 `widgets/` 和 `screens/` 必须通过 Primitives 的工厂函数和组件索引使用底层 UI 能力
- 🟥 违反此规则的代码应在 code review 中被拒绝

### 第 X 条：Screen Specification 强制（P0）

依据 ADR-066（UI Screen Specification 标准），本节在现有 UI 运行时架构（Projection / ViewModel / Widget Contract / Dirty<T>）之上追加规范描述层 SSPEC。

| 标记 | 规则 |
|------|------|
| 🟩 | 每个 Screen 必须有对应的 Screen Spec 文档（SSPEC），位于 `docs/06-ui/07-specs/screens/{name}.md` |
| 🟩 | SSPEC 必须包含「三位一体」：ASCII Wireframe + Widget Tree + Flexbox Layout |
| 🟩 | AI 生成 UI 代码前必须读取对应的 SSPEC 文件 |
| 🟩 | SSPEC 必须通过 DoD 14 项 P0 检查才可标记为 `active` |
| 🟩 | 新增 Screen 必须先写 SSPEC（status: draft），通过 DoD 检查后再实现代码 |
| 🟥 | 禁止在不读取 SSPEC 的情况下由 AI 生成 Screen 布局代码 |
| 🟥 | 禁止 SSPEC 缺少任意一个「三位一体」元素（ASCII Wireframe / Widget Tree / Flexbox Layout） |
| 🟥 | 禁止 AI 在生成 UI 时新增/删除/修改 SSPEC 中定义的 region |
| 🟥 | 禁止新增 Screen 不写 SSPEC 直接实现代码 |

### 第 X 条：Widget ID 稳定（P0）

Widget ID 是 Screen Spec、Widget Contract、UiBinding 之间的连接点，必须保持永久稳定。

| 标记 | 规则 |
|------|------|
| 🟩 | 每个 Widget 实体必须有稳定的 `widget_id`，格式为 `{screen}_{region}_{element}_{variant}` |
| 🟩 | `widget_id` 是永久标识，业务重命名时不允许修改（只标记 deprecated，永不重新分配） |
| 🟩 | Widget ID → UiBinding → ViewModel Field → Def Registry 映射必须记录在 `widget-id-map.md` |
| 🟥 | 禁止在重构时修改 `widget_id`（只能标记 deprecated + replaced_by + deprecation_reason） |

### 第 X 条：UI 设计工具链（P0）

项目使用纯文本工具链进行 UI 设计，零 Figma 策略。

| 标记 | 规则 |
|------|------|
| 🟩 | 项目使用纯文本工具链进行 UI 设计：ASCII Wireframe + Markdown + YAML |
| 🟩 | Excalidraw / draw.io 可选用于复杂布局的快速草图 |
| 🟥 | 禁止引入 Figma / Adobe XD / Sketch 等 GUI 设计工具 |
| 🟥 | 禁止将视觉设计稿作为 Screen 实现的输入（SSPEC 才是权威输入源） |

### 第 X 条：动画所有权（P1）

| 标记 | 规则 |
|------|------|
| 🟩 | 动画必须明确声明 ownership（screen 级 vs widget 级），记录在 SSPEC 的 Lifecycle 节 |
| 🟥 | 禁止无动画归属声明的 UI 元素产生动画行为 |

### 第 X 条：Widget Budget 与 Screen Metrics（P1）

| 标记 | 规则 |
|------|------|
| 🟩 | 每个 Screen 必须在 SSPEC 中定义 complexity budget（max_depth ≤ 6, max_children ≤ 20） |
| 🟩 | 新增 Screen 时必须设定 metrics 基线（widget_count / container_count / interactive_count / max_depth） |
| ⚠️ | CI 中 metrics 增长超过 30% → 告警 |
| ⚠️ | Widget 嵌套深度超过 6 层 → 强制重构 |
