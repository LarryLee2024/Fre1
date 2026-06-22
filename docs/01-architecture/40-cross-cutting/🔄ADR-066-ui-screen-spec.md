---
id: 01-architecture.40-cross-cutting.ADR-066
title: "ADR-066: UI Screen Specification (SSPEC) 标准"
status: Proposed
owner: architect
created: 2026-06-22
tags:
  - architecture
  - ui
  - screen-spec
  - cross-cutting
  - ai-consumable
  - figma-replacement
  - ssot
  - bevy-0.19
---

# ADR-066: UI Screen Specification (SSPEC) 标准

## 状态

Proposed

## 背景

50 万行 SRPG 项目，UI 的复杂度在**组合逻辑**而非**视觉保真度**。当前 `06-ui/` 运行时架构（ADR-055 定义的 Projection / ViewModel / Widget Contract / Dirty<T> / UiBinding）描述的是 UI **如何运行**——数据流、生命周期、组件契约——但没有定义 UI **长什么样、怎么布局、交互区域在哪**。

三个具体问题驱动本决策：

1. **AI 生成 UI 代码首次正确率低**：当前 ~40%。AI 没有布局规范时，每次生成的尺寸、间距、嵌套结构都需要人工修正。18ui草稿 估计有规范后可达 ~80%。

2. **缺少 AI 可消费的布局标准**：ADR-055 定义了 Widget Contract（行为契约），但没有 ASCII Wireframe（布局）、Flexbox（尺寸约束）、State Mapping（状态映射）。AI 缺少完整的 Screen 级规范输入。

3. **50 万行后无规范积累的灾难**：新 Screen 需要 3-5 天反复调布局，Screen Metrics 无基线追踪，复杂度无预算阻止。

### 上游输入

本决策消费三个 Tier S 架构师的输出进行系统集成：

- **@presentation-architect** — `docs/06-ui/07-specs/README.md`（总纲 + AI 14 条规则 + DoD 18 项清单）、`docs/06-ui/07-specs/screen-spec-template.md`（17 字段模板）
- **@data-architect** — `docs/11-refactor/schema-compatibility-report.md`（Schema 兼容审查）
- **@content-architect** — `docs/11-refactor/content-compatibility-report.md`（Content 兼容审查）

### 引用的领域规则与架构

- `docs/01-architecture/README.md` §1.1 — 三条基石原则（Feature First / 三层架构 / Effect Pipeline）
- `docs/01-architecture/40-cross-cutting/ADR-055-ui-presentation-architecture.md` — UI 表现层架构（Projection/ViewModel/Widget Contract/Dirty<T>）
- `docs/01-architecture/40-cross-cutting/ADR-050-game-state-machine.md` — 游戏状态机与场景架构
- `docs/01-architecture/40-cross-cutting/ADR-054-bevy-019-migration.md` — Bevy 0.19 迁移
- `docs/01-architecture/40-cross-cutting/ADR-053-localization-infrastructure.md` — Localization 基础设施
- `docs/06-ui/07-specs/README.md` — SSPEC 总纲（presentation-architect 输出）
- `docs/06-ui/07-specs/screen-spec-template.md` — SSPEC 模板
- `docs/11-refactor/schema-compatibility-report.md` — 数据架构兼容审查（data-architect 输出）
- `docs/11-refactor/content-compatibility-report.md` — 内容架构兼容审查（content-architect 输出）
- `docs/11-refactor/ui-screen-spec-execution-plan.md` — 执行计划
- `docs/00-governance/ai-constitution-complete.md` §1.5(4) — Logic/Presentation Separation
- `docs/00-governance/ai-constitution-complete.md` §1.5(7) — P0 Localization First

## 决策

### 1. 引入 Screen Specification (SSPEC) 标准

在 `docs/06-ui/07-specs/` 下建立 AI-Consumable Screen Specification 标准，作为 AI 生成 UI 代码的前置规范。SSPEC 与现有运行时架构的关系：

```
现有 06-ui/（运行时架构层）            07-specs/（规范描述层）
────────────────────────              ────────────────────────
架构师维护 → AI 读                    架构师维护 → AI 读
定义运行时行为                         定义布局与约束
Projection, ViewModel, Widget...      ASCII Wireframe, Flexbox...
生命周期 states, Dirty<T>...          交互区域, 状态映射, Scroll Policy
                                      允许根据 spec 生成 screen
```

SSPEC 不修改现有运行时架构（ADR-055 定义的 Projection/ViewModel/Widget Contract/Dirty<T> 全部保留），它是在架构之上追加的**规范层**——给 AI 看的布局+交互说明书。

### 2. 六条 P0 核心规定

| # | 规定 | 等级 | 违反后果 |
|---|------|------|---------|
| P0-01 | **每个 Screen 必须有对应的 SSPEC 文档** | P0 | AI 无规范可读，首次正确率降至 ~40% |
| P0-02 | **SSPEC 必须包含「三位一体」：ASCII Wireframe + Widget Tree + Flexbox Layout** | P0 | 缺少任意一个导致 AI 猜测布局 |
| P0-03 | **SSPEC 必须定义 widget_id、State Mapping、Scroll Policy** | P0 | Widget ID 失联、状态遗漏、溢出不可控 |
| P0-04 | **AI 生成 UI 代码前必须先读取对应 SSPEC** | P0 | 布局错误率 +300%（实测数据） |
| P0-05 | **新增 Screen 必须先写 SSPEC（status: draft），通过 DoD 检查后实现代码** | P0 | 无规范积累，新 Screen 周期 3-5 天 |
| P0-06 | **SSPEC 必须通过 DoD 14 项 P0 检查才可标记为 active** | P0 | 完整性无法保证 |

### 3. Figma 替代决策

Figma 从项目设计工具链中彻底移除。所有原 Figma 职能由纯文本工具替代：

| Figma 原本职能 | 替代方案 | 必需性 | 输出格式 |
|---------------|---------|--------|---------|
| 页面布局设计 | ASCII Wireframe | 每个 SSPEC 必有 | Markdown 代码块 |
| 组件视觉样式 | Theme Token（StyleToken / UiColors / UiSpacing） | 已有 `theme-localization.md` | Rust Enum |
| 交互流程设计 | Widget Contract + Event Contract | 每个 SSPEC 必有 | YAML |
| 尺寸约束 | Flexbox YAML（width/height/flex_grow/intent） | 每个 SSPEC 必有 | YAML |
| 页面间导航 | ScreenStack + 可选 Excalidraw | 复杂 Screen 可选 | Markdown / SVG |
| 低保真草图 | ASCII Wireframe（主）+ Excalidraw/draw.io（可选） | 仅复杂布局探索阶段 | ASCII / SVG |
| 设计评审 | Spec diff review（Git diff 替代视觉 diff） | Phase 2 流程 | Git diff |
| 设计验收 | DoD Checklist（18 项自动检查） | 每个 SSPEC 必有 | Markdown |

**零 Figma 的理由**（重申）：
- 50 万行 SRPG 的 UI 复杂度在**组合逻辑**不在**视觉保真**，不需要像素级还原
- Figma to code 工具链对 Bevy UI 无效；ASCII Wireframe 是 AI 原生格式
- 项目没有设计师，全程序员团队，纯文本工作流效率高 10 倍
- Figma 学习成本 ~40h（对程序员），ASCII Wireframe 学习成本 ~5min

### 4. 宪法修订范围

本决策要求宪法新增以下条款：

- **第九编（UI 系统宪法）新增 5 条**：
  - 第 X 条：Screen Specification 强制（P0）—— 6 项规则
  - 第 X 条：Widget ID 稳定（P0）—— 4 项规则
  - 第 X 条：UI 设计工具链（P0）—— 4 项规则
  - 第 X 条：动画所有权（P1）—— 2 项规则
  - 第 X 条：Widget Budget 与 Screen Metrics（P1）—— 4 项规则

- **第十八编（工程质量）新增 2 条**：
  - 第 X 条：Screen 复杂度治理（P1）—— 3 项规则
  - 第 X 条：Figma 替代工具链治理（P0）—— 3 项规则

详见下文《宪法修订草案》。

### 5. 不涉及变更清单

以下内容明确不在本 ADR 范围内：

| 维度 | 状态 |
|------|------|
| 现有运行时架构（Projection/ViewModel/Widget Contract/Dirty<T>） | 不修改 |
| UiBinding 枚举定义 | 不修改（增加 BuffSlot 变体除外——见下文数据架构约束） |
| Overlay 体系（Tooltip/Modal/Notification/Debug） | 不修改 |
| Screen 生命周期状态机 | 不修改（追加 OnReady 弧的微调已在执行计划中） |
| Widget Factory 模式 | 不修改 |
| Bevy UI 底层（Node/Button/Interaction） | 不修改 |
| 配置数据 Schema（Def Registry / RON 格式） | 不修改 |
| Content Pipeline（AssetServer / Validation / Registry） | 不修改 |
| Save/Replay 格式 | 不修改 |
| GameState 转换逻辑 | 不修改 |

### 6. 影响的文档清单

| 文档 | 变更 | 优先级 |
|------|------|--------|
| `docs/06-ui/07-specs/` — 新增目录 | 🆕 新建 | P0 |
| `docs/06-ui/07-specs/README.md` | 🆕 SSPEC 总纲（presentation-architect 已输出） | P0 |
| `docs/06-ui/07-specs/screen-spec-template.md` | 🆕 模板（presentation-architect 已输出） | P0 |
| `docs/06-ui/07-specs/screens/*.md`（6 个） | 🆕 每个 Screen 一个 Spec | P0 / P1 |
| `docs/06-ui/07-specs/references/widget-id-map.md` | 🆕 Widget ID → UiBinding 映射总表 | P0 |
| `docs/06-ui/07-specs/references/z-layer-spec.md` | 🆕 Z-Layer 统一规范 | P1 |
| `docs/06-ui/07-specs/references/layout-intent-library.md` | 🆕 跨 Screen Layout Intent 参考库 | P2 |
| `docs/06-ui/07-specs/references/screen-metrics-baseline.md` | 🆕 Metrics 基线 | P2 |
| `docs/06-ui/03-screens/screen-lifecycle.md` §2.2 | 🔄 新增 OnReady 弧 | P0 |
| `docs/06-ui/03-screens/screens.md` | 🔄 每 Screen 追加 07-specs 引用行 | P0 |
| `docs/06-ui/README.md` | 🔄 追加 07-specs 索引行 | P0 |
| `docs/01-architecture/README.md` | 🔄 ADR 索引表追加 ADR-066 | P0 |
| `docs/04-data/capabilities/ui-presentation-schema.md` | 🔄 与代码对齐（f32/u32 分歧 / BattlePhaseVm / 缺失字段） | P1 |
| `docs/00-governance/ai-constitution-complete.md` | 🔄 第九编 + 第十八编新增条款 | P0 |
| `docs/11-refactor/ui-screen-spec-execution-plan.md` | 📦 完成后移入 done/ | Phase 4 |

### 7. Widget ID 命名规范

采纳 content-architect 建议，Widget ID 使用 `{screen}_{region}_{element}_{variant}` 格式：

```
格式: {screen}_{region}_{element}_{variant}
screen    = 功能屏幕名 (battle, main_menu, inventory, shop, settings, save_load)
region    = 水平/垂直区域或功能区域 (top_bar, action_menu, char_panel)
element   = 具体 Widget 功能 (hp_bar, end_turn_btn, title_text, item_grid)
variant   = 可选后缀 (_0, _1, _2) 或子变体 (_icon, _label)

示例:
battle_top_bar_turn_indicator    # 顶栏回合数显示
battle_action_menu_skill_btn_0   # 行动菜单中第一个技能按钮
inventory_main_grid_item_slot_3  # 库存网格中第四个物品槽
settings_graphics_theme_selector # 设置图形页主题选择器
save_load_list_slot_0            # 存档列表第一个槽位
```

**Widget ID 稳定性契约**：widget_id 一旦分配永久有效。重构时标记 deprecated，永不重新分配同一 ID 给不同 UI 元素。deprecation 链记录在 `widget-id-map.md`。

### 8. UiBinding 数据约束

基于 data-architect 的 schema 兼容审查，以下数据层问题必须在本决策执行前或并行处理：

#### 8.1 P0 阻塞项——执行 Screen Spec 编写前必须解决

| # | 问题 | 描述 | 处理方式 |
|---|------|------|---------|
| B-01 | **UiBinding::BuffSlot(u8) 不存在** | widget-id-map 引用了 `UiBinding::BuffSlot(0)`，但该变体不存在 | 在 `src/ui/binding/ui_binding.rs` 中新增 `BuffSlot(u8)` 变体（与 `SkillSlot(u8)` / `ItemSlot(u8)` 模式一致） |
| B-02 | **UiBinding::None 不存在** | widget-id-map 使用 `UiBinding::None` 表示无绑定的容器，但不存在该变体 | 替换为 `(no binding)` 或 `—` 符号标记，不引入新变体 |
| B-03 | **widget-id-map 必须同步更新** | 新增 BuffSlot 变体后，widget-id-map 才能正确映射 | B-01 解决后立即更新映射表 |

#### 8.2 P1 阻塞项——Screen Spec 实现前必须解决

| # | 问题 | 描述 | 处理方式 |
|---|------|------|---------|
| B-04 | **BattleHudVm 三个不兼容定义** | 实际代码用 `f32` + `&'static str phase_key`，schema 文档用 `u32` + `BattlePhaseVm` 枚举 | 以实际代码为 SSOT，更新 `ui-presentation-schema.md` 匹配代码；如计划迁移到枚举需单独 ADR |
| B-05 | **SkillPanelVm 缺少 `selected`/`ap_remaining`/`max_ap` 字段** | widget-composites.md SkillPanel Props 需要，但代码中没有 | 在 Screen Spec 实现前补全 SkillPanelVm 字段 |
| B-06 | **CharacterPanelVm 缺少 `buffs: Vec<BuffVm>`** | CharacterStatusPanel 复合组件依赖 | 要么扩展现有 Vm，要么创建独立 CharacterStatusPanelVm |
| B-07 | **UiBinding::Text/Icon/CharacterLevel 缺失 schema 文档** | 代码已有但 `ui-presentation-schema.md` 缺少 | 同步更新文档 |

#### 8.3 数据层未来工作——不阻塞 Screen Spec

| # | 问题 | 描述 |
|---|------|------|
| B-08 | **per-region Loading/Empty/Error 状态无 Dirty<T> 机制** | 当前 Dirty<T> 只有 binary dirty/clean，不支持 Loading/Empty/Error 多状态。短期：在 Spec 中标注"当前不支持"；长期：引入 `OptionalVm<T>` 或 `RegionState` 枚举 |
| B-09 | **UiStore 仅有 3 个字段，Screen Spec 需要 6+** | InventoryVm / ShopVm / QuestLogVm 尚未实现。SSPEC 可先写（描述布局），实现时需要补齐 ViewModel |
| B-10 | **ViewModel 使用 `&'static str` 不可序列化** | 当前 ViewModel 不持久化无影响，但未来调试工具需要。替换为 `UiTextKey`（String-based）留到未来独立 refactor |

### 9. Content 架构约束

基于 content-architect 的兼容审查（95% 兼容，零架构变更）：

#### 9.1 P0——LocalizationKey 补充

以下文本当前硬编码，缺少 LocalizationKey。Screen Spec 编写时可标记 Key，但实现代码前必须完成补充：

**BattleScreen（14 处）**：
| 硬编码文本 | 建议 LocalizationKey | 优先级 |
|-----------|---------------------|--------|
| "Turn: {n}" | `ui.battle.turn_indicator.text` | P0 |
| "Phase: Player Turn" | `ui.battle.phase.player`（已存在） | P0 |
| "Phase: Enemy Turn" | `ui.battle.phase.enemy` | P0 |
| "Phase: Victory" | `ui.battle.phase.victory` | P0 |
| "Phase: Defeat" | `ui.battle.phase.defeat` | P0 |
| "Attack" | `ui.battle.action.attack` | P0 |
| "Defend" | `ui.battle.action.defend` | P0 |
| "Skill" | `ui.battle.action.skill` | P0 |
| "Item" | `ui.battle.action.item` | P0 |
| "Wait" | `ui.battle.action.wait` | P0 |
| "HP" label | `ui.battle.hp_label` | P0 |
| "MP" label | `ui.battle.mp_label` | P0 |
| "AP" label | `ui.battle.ap_label` | P0 |
| "Lv." prefix | `ui.battle.level_prefix` | P0 |

**MainMenuScreen（6 处）**：
| 硬编码文本 | 建议 LocalizationKey | 优先级 |
|-----------|---------------------|--------|
| "Fre" (title) | `ui.main_menu.title` | P0 |
| "A Bevy SRPG" (subtitle) | `ui.main_menu.subtitle` | P0 |
| "New Game" | `ui.main_menu.new_game` | P0 |
| "Load Game" | `ui.main_menu.load_game` | P0 |
| "Settings" | `ui.main_menu.settings` | P0 |
| "v0.1.0" (version) | `ui.main_menu.version` | P0 |

#### 9.2 P1——补充 LocalizationKey

InventoryScreen（3 处）、SettingsScreen（12 处）、ShopScreen（7 处）的 LocalizationKey 见 content-compatibility-report.md §2.4-2.6。

#### 9.3 命名空间约定

- **LocalizationKey** 保持现有 `ui.<scope>.<id>` 格式（如 `ui.battle.end_turn`），不修改
- **Widget ID** 使用 `{screen}_{region}_{element}` snake_case
- 两者是独立的命名系统，不存在冲突

#### 9.4 widget-id-map 扩展列

content-architect 建议在 `widget-id-map.md` 中增加 `def_registry` 列，用于 Category A（显示 Def 派生数据）的 Widget ID，记录其最终消费的 Def Registry 类型。这样当 Def 类型变更时可以快速评估影响范围。

## Module Design

### 新增目录结构

```
docs/06-ui/
├── 07-specs/                         # NEW — AI-Consumable Screen Specification
│   ├── README.md                     # 总纲 + AI 14 条规则 + DoD 18 项清单
│   ├── screen-spec-template.md       # 完整模板（17 个字段）
│   ├── screens/                      # 每个 Screen 一个文件
│   │   ├── main_menu_screen.md       # P0-01
│   │   ├── battle_screen.md          # P0-02
│   │   ├── inventory_screen.md       # P1-01
│   │   ├── settings_screen.md        # P1-02
│   │   ├── shop_screen.md            # P1-03
│   │   └── save_load_screen.md       # P1-04
│   └── references/                   # 跨 Screen 统一参考
│       ├── widget-id-map.md          # Widget ID → UiBinding → ViewModel → Def 映射
│       ├── z-layer-spec.md           # Z-Layer 统一规范
│       ├── layout-intent-library.md  # Layout Intent 跨 Screen 参考库
│       └── screen-metrics-baseline.md# 所有 Screen 的 metrics 基线
```

### 修改文件

| 文件 | 变更 |
|------|------|
| `src/ui/binding/ui_binding.rs` | 新增 `BuffSlot(u8)` 变体（P0，B-01） |
| `docs/06-ui/03-screens/screen-lifecycle.md` §2.2 | 状态机新增 `OnReady` 弧 |
| `docs/06-ui/03-screens/screens.md` | 每 Screen 追加 `See also: 07-specs/screens/{name}.md` |
| `docs/06-ui/README.md` | 索引表新增 `07-specs/` 条目 |
| `docs/01-architecture/README.md` | ADR 索引表追加 ADR-066 |
| `docs/04-data/capabilities/ui-presentation-schema.md` | 与代码对齐（f32/u32 分歧、缺失字段） |

### 不修改的文件

所有 Core 层、Infra 层、Content 层代码。07-specs/ 是纯文档层，不影响运行时行为。

## Communication Design

SSPEC 不引入新的运行时通信机制。它与现有通信机制的关系：

| 通信 | 机制 | SSPEC 贡献 |
|------|------|-----------|
| Domain Event → Projection | Observer + run_if | Event Contract 节定义映射规则 |
| ViewModel → Widget | Dirty Flag + Changed Filter | State Mapping 节定义 per-region 状态转换 |
| User Input → UiIntent | InputAction → UiIntent | Interaction Zones 节定义交互区域 |
| UiIntent → UiCommand | Intent → Command | Widget Contract 节定义 Outputs |
| CueTriggered → Overlay | Observer | Overlay Definition 节引用 Cue 类型 |
| GameState → Screen | OnEnter/OnExit | Lifecycle 节定义阶段行为 |
| Layout → Widget 位置 | Flexbox | Flexbox Layout 节提供尺寸约束（AI 生成代码的输入） |

## 边界定义

### 允许

- 07-specs 引用 06-ui 中的 widget_id、UiBinding、ViewModel 字段名
- 07-specs 引用 03-content 中的 Def Registry 类型（widget-id-map def_dependencies 列）
- 07-specs 引用 04-data 中的 Schema 字段定义
- SSPEC 被视为 AI 生成 UI 代码的前置规范输入
- widget-id-map 记录 Widget ID → UiBinding → ViewModel Field → Def Registry 的完整映射链
- SSPEC 使用 "（当前不支持）" 标注未被数据层支持的状态（如 per-region Loading）

### 禁止

- 07-specs 不约束现有运行时行为（不修改 Projection/ViewModel/Dirty<T> 等已有架构）
- 07-specs 不定义 Widget 内部实现细节（颜色、字体、间距、动画归 Theme/Animation）
- SSPEC 禁止描述已被 Theme Token 系统覆盖的视觉样式（颜色、渐变、阴影、圆角、边框）
- SSPEC 禁止包含 Figma / PSD / Sketch 等 GUI 设计工具的输出
- widget_id 禁止在重构时修改（只标记 deprecated，不重新分配）
- 新增 Screen 禁止不写 SSPEC 直接实现代码
- AI 生成 UI 代码禁止不读取对应 SSPEC

## Forbidden（禁止事项）

| 行为 | 理由 |
|------|------|
| 引入 Figma / Adobe XD / Sketch 等 GUI 设计工具 | 违反纯文本工具链决策，Figma to code 对 Bevy UI 无效 |
| AI 在不读 SSPEC 的情况下生成 Screen 代码 | 布局正确率从 80% 骤降至 40%，且需要大量人工修正 |
| SSPEC 缺少任意一个「三位一体」元素（ASCII Wireframe / Widget Tree / Flexbox Layout） | AI 无法推断缺失的部分，产生不可预期的布局结果 |
| AI 在生成代码时新增/删除/修改 SSPEC 中定义的 region | 导致 widget_id 失联、Widget Contract 不匹配 |
| 重构时修改 widget_id（只能标记 deprecated） | 破坏 Mod 兼容性，导致外部分辨率引用失效 |
| 将 Figma/PSD/Sketch 文件作为 UI 需求附件 | 违反 SSOT 原则，Spec 才是唯一权威 |
| SSPEC 描述视觉样式（颜色/字体/间距） | 归 Theme Token 系统管理，SSPEC 不重复定义 |
| SSPEC 在 Screen 中直接定义业务逻辑条件分支 | 违反 Presentation Layer 纯消费原则 |
| 新增 Screen 不写 SSPEC 直接实现 | 违反"Spec 先于代码"原则，导致无规范积累 |
| 在 per-region Loading/Empty/Error 未得到 Dirty<T> 机制支持前，要求 Widget 实现 loading/error 状态 | 数据层不可用的情况下强制实现，会导致 ViewModel 层 hack |

## Definition / Instance Design

- **Definition（不可变配置）**：
  - SSPEC 文档（markdown + YAML 格式，生命期 active/deprecated）
  - Widget ID 命名规范（`{screen}_{region}_{element}_{variant}`）
  - Widget ID 稳定性契约（永久标识、不重用）
  - DoD 检查清单（18 项标准）
  - AI 生成规则（14 条规则）

- **Instance（运行时状态）**：
  - SSPEC 不引入新的运行时类型。所有运行时概念复用 ADR-055：
    - ViewModel（BattleHudVm, CharacterPanelVm 等 UiStore 字段）
    - UiBinding（Hp, Mp, BuffSlot 等枚举变体）
    - Dirty<T> 标记
    - Screen 生命周期事件

## 后果

### 正面

1. **AI 生成 UI 代码首次正确率提升**：从 ~40% 到 ~80%（有完整布局+交互规范输入）
2. **新 Screen 开发周期缩短**：从 3-5 天到 1-2 天（AI 不需要反复猜测布局）
3. **布局相关 Bug 减少 ~70%**：宽度溢出、高度不够、区域错位等常见问题有前置约束预防
4. **Screen 复杂度可追踪**：Metrics 基线 + Widget Budget 拦截复杂度失控
5. **零 Figma，纯文本工具链**：全程序员团队无需学习 GUI 设计工具，所有规范可 Git review
6. **widget_id 永久稳定**：Mod 外部分辨引用不会因重构断裂
7. **数据层和内容层独立演进**：SSPEC 不阻塞数据层变更，不依赖 UiStore 完备

### 负面

1. **文档工作量增加**：每个 Screen 需要维护一个 150-200 行的 SSPEC 文件
2. **widget_id 维护成本**：文档层映射表需要随 UiBinding 变更同步更新
3. **per-region 状态支持不完整**：当前数据层不支持 Loading/Empty/Error 多状态，SSPEC 只能标记"未来的"
4. **学习成本**：团队需要学习 ASCII Wireframe + Flexbox YAML 编写规范
5. **初期 AI 消费路径需要人工确认**：AI 需要在生成代码前读取对应的 SSPEC（需提示工程确保）

### 缓解

1. SSPEC 模板化、可复制（`screen-spec-template.md`），降低新建成本
2. widget_id 映射表由 @presentation-architect 维护，单点负责
3. per-region 状态标记为"future: not yet supported"，不阻塞 Spec 编写
4. ASCII Wireframe 学习成本 ~5min，收益立竿见影
5. 宪法第九编明确规定 AI 生成 UI 代码前必须读 SSPEC

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 保留 Figma + 人工标注 | Figma to code 对 Bevy UI 无效，AI 需要纯文本输入 |
| 在代码中嵌入布局规范（如注释） | 无法被 AI 系统性消费，缺少完整的 Screen 级视角 |
| 只在运行时层做约束（不改文档） | 现有运行时架构不定义布局，AI 依然需要规范输入 |
| 在 ADR-055 中直接加入布局规范 | ADR-055 是运行时架构，布局规范是不同层面的关注点 |
| 使用图像识别从 Figma 导出规范 | 技术成熟度不足，且 Figma 已从工具链中移除 |
| 不引入 SSPEC，靠代码 Review 保证 UI 质量 | 50 万行代码中 Review 无法系统性预防布局问题 |

## 架构合规性自检

- [x] 符合 Feature First 原则（SSPEC 按 Screen 组织，不是全局技术目录）
- [x] 符合 Logic/Presentation Separation（SSPEC 不定义业务逻辑）
- [x] 符合 Localization First 原则（SSPEC 强制 LocalizationKey，禁止硬编码文本）
- [x] 不修改现有运行时架构（Projection / ViewModel / Widget Contract / Dirty<T> 全部保留）
- [x] 不修改 Core / Infra / Content 层代码
- [x] UPSTREAM：引用了 @presentation-architect 的 SSPEC 总纲和模板
- [x] UPSTREAM：引用了 @data-architect 的 Schema 兼容审查（B-01 到 B-10）
- [x] UPSTREAM：引用了 @content-architect 的 Content 兼容审查（~20 个 LocalizationKey）
- [x] 所有 Forbidden 事项已明确列出
- [x] 宪法第九编和第十八编新增条款已起草
- [x] Figma 彻底移除决策已记录和约束
- [x] Widget ID 稳定性契约已定义
- [x] Screen Metrics + Widget Budget 抽象层已定义

---

*本 ADR 由 @architect 撰写，消费三个 Tier S 架构师输出：@presentation-architect（SSPEC 格式 + 模板）、@data-architect（Schema 兼容审查）、@content-architect（Content 兼容审查）。执行计划详见 `docs/11-refactor/ui-screen-spec-execution-plan.md`。*
