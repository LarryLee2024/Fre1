---
id: 00-governance.constitution-amendments-ui-spec-v1
title: 宪法修订草案 v1 — 第九编 / 第十八编 新增条款（基于 ADR-066）
status: draft
owner: architect
created: 2026-06-22
tags:
  - governance
  - constitution
  - screen-spec
  - figma-replacement
  - widget-id
  - screen-metrics
---

# 宪法修订草案 v1 — 第九编 / 第十八编 新增条款

> 本草案基于 ADR-066（UI Screen Specification 标准），消费三个 Tier S 架构师输出。修订不修改现有条款编号，只在对应编末尾追加新条款。
>
> **文件操作说明**：以下内容应插入 `docs/00-governance/ai-constitution-complete.md` 的对应编末尾。第九编（UI 系统宪法）的结尾在 `---` 分割线之前（第 1012 行），第十八编（工程质量与技术债治理）的结尾在 `---` 分割线之前（第 1628 行）。

---

## 第九编新增条款

（插入位置：第九编末尾，即 `### 第 X 条：Primitives 隔离` 之后、`---` 分割线之前）

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

---

## 第十八编新增条款

（插入位置：第十八编末尾，即 `### 18.7.6 Debt Lifecycle` 之后、`---` 分割线之前）

### 第 X 条：Screen 复杂度治理（P1）

| 标记 | 规则 |
|------|------|
| 🟩 | Screen Metrics 基线追踪：每个 Screen 必须记录 `widget_count` / `container_count` / `interactive_count` / `max_depth` |
| 🟩 | Widget Budget：`max_widget_depth ≤ 6`，`max_children_per_container ≤ 20` |
| ⚠️ | 超过阈值时必须重构，不得累积复杂度债务 |

### 第 X 条：Figma 替代工具链治理（P0）

| 标记 | 规则 |
|------|------|
| 🟩 | 新增 Screen 的 UI 设计流程为：写 SSPEC → DoD 检查 → 实现代码 |
| 🟩 | SSPEC 是 UI 设计的唯一真相源（SSOT），不依赖任何 GUI 设计工具 |
| 🟥 | 禁止将 Figma / PSD / Sketch 文件作为 UI 需求附件 |
| 🟥 | 禁止在 SSPEC 中引用 GUI 设计工具的输出作为布局依据 |

---

## 宪法修改影响注记

| 维度 | 影响 |
|------|------|
| 现有第九编条款 | 不修改，仅追加新条款 |
| 现有第 X 条（Primitives 隔离） | 不变 |
| 现有第十八编条款 | 不修改，仅追加新条款 |
| 现有第十八编 §18.1-18.7 | 不变 |
| 新条款的宪法效力 | 第九编追加条款为 P0/P1 级，第十八编追加条款为 P0/P1 级，违反对应标记等级的后果 |
| 与 ADR-055 的关系 | 互补而非替代——ADR-055 定义运行时行为，ADR-066 定义布局规范，宪法新增条款覆盖两者 |
| 与 ADR-053（Localization）的关系 | 第九编新增 Screen Spec 强制条款引用 LocalizationKey，与 ADR-053 一致 |

---

*本草案由 @architect 起草，基于 ADR-066（UI Screen Specification 标准）。提交审批后修改正式宪法文件。*
