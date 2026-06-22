---
id: 11-refactor.ui-screen-spec-execution-v3
title: UI Screen Specification 重构执行计划 v3.0（终结版）
status: completed
owner: architect
created: 2026-06-22
archived: 2026-06-22
reason: All 4 phases executed. ADR-066 accepted, 6/6 Screen Specs active, 4 reference files created, constitution amended, READMEs updated.
tags:
  - ui
  - screen-spec
  - refactoring
  - ai-consumable
  - agent-orchestration
  - figma-replacement
  - completeness
---

# UI Screen Specification 重构执行计划 v3.0

> **宪法依据**: 第九编（UI 系统宪法）、第十八编（工程质量）、第二十一编（红线禁止事项）
> **上游架构**: ADR-055（UI 表现层架构）、ADR-054（Bevy 0.19 迁移）、ADR-050（游戏状态机）
> **现有文档**: `docs/06-ui/`（16 个文件）
> **输入源**: `docs/99-history/ai_ignore_this_dir/18ui草稿.md`
> **设计原则**: 不改架构，追加规范；AI-Consumable 优先；零 Figma，纯文本工具链

---

## 0. 总纲

### 0.1 核心原则（不妥协）

```
▌不改架构，追加规范
  现有 06-ui/ 运行时架构（Projection/ViewModel/Screen 生命周期/Widget Contract/Dirty<T>）
  全部保留不动。07-specs/ 是物理隔离的规范层，不修改现有 14 个文件的 status/owner。

▌AI-Consumable 优先
  规范的目标读者是 AI（Claude/ChatGPT/Gemini），不是人。
  ASCII Wireframe + Flexbox + YAML 结构，AI 读完 100% 知道怎么生成 UI。

▌零 Figma，纯文本工具链
  Figma 彻底移除出工作流。替代方案：ASCII Wireframe（布局）+ Excalidraw（草图）
  + Markdown+YAML（结构化规范）。所有工具不需要 GUI、不需要设计稿、不需要 Figma 账号。
```

### 0.2 三句话总结

1. **现有运行时架构零改动** — 06-ui/ 全部保留
2. **新增 07-specs/ 目录** — AI-Consumable Screen Specification 标准
3. **18ui草稿 的所有概念全部吸收** — 首次分析遗漏 7 项，v1 响应遗漏 9 项，本次全部补全

### 0.3 物理隔离示意

```
现有 06-ui/（运行时架构层）            07-specs/（规范描述层）
────────────────────────              ────────────────────────
架构师维护 → AI 读                    架构师维护 → AI 读
定义运行时行为                         定义布局与约束
Projection, ViewModel, Widget...      ASCII Wireframe, Flexbox...
生命周期 states, Dirty<T>...          交互区域, 状态映射, Scroll Policy
                                      允许根据 spec 生成 screen

06-ui/ 修改范围（仅 3 处微调）:
  ├── screen-lifecycle.md §2.2 → 状态机增加 OnReady 弧
  ├── screens.md → 每 Screen 加一行 "See also: 07-specs/screens/{name}.md"
  └── README.md → 加一行 07-specs/ 索引
```

---

## 1. 兼容性分析表（v1 补回）

18ui草稿 全部概念与现有架构的逐项对比，明确哪些要改、哪些不动：

### 1.1 核心规范（18ui草稿 §1-§16）

| 18ui草稿 概念 | 现有架构等价物 | 变动 | 行动 |
|--------------|---------------|------|------|
| ASCII Wireframe | ❌ 不存在 | 🆕 **新增** | 07-specs/ 必含 |
| Widget Tree | ✅ `screens.md` 有描述 | 🔄 **ID 标注版** | 07-specs/ 中加 widget_id 标注 |
| Flexbox Layout | ❌ 不存在（尺寸散落在代码中） | 🆕 **新增** | 07-specs/ 必含 width/height/flex_grow |
| Design System | ✅ `widget-atoms.md` + `widget-composites.md` | ✅ **不动** | 现有完整 |
| Screen→Widget→Primitive | ✅ `architecture.md` §4.4 三层渲染层 | ✅ **不动** | 现有完整 |
| Figma 核心页面 | ❌ 项目不用 Figma | 🔄 **替换** | 见 §2 Figma 替代方案 |
| ADR-UI-001（7条规则） | ❌ 不存在 | 🆕 **新增** | ADR-066 吸收 |
| AI-Consumable 理念 | ❌ 不存在 | 🆕 **新增** | 07-specs/README.md 总纲 |
| Screen Header | 🟡 `screens.md` 有散落属性 | 🔄 **结构化** | 07-specs/模板格式 |
| ASCII Wireframe（要求） | ❌ 不存在 | 🆕 **新增** | 强制区域命名，禁匿名面板 |
| Widget Tree（要求） | 🟡 文本描述无 ID | 🔄 **ID 标注** | 07-specs/强制格式 |
| Flexbox Layout（要求） | ❌ 不存在 | 🆕 **新增** | 每 widget 必须 width/height/flex_grow |
| Responsive Rules | ❌ 不存在 | 🆕 **新增** | 至少写 "strategy: none" |
| Region Responsibility | 🟡 文本描述 | 🔄 **结构化** | YAML 列表格式 |
| Widget Contract | ✅ `widget-atoms.md` 完整 | ✅ **不动** | 现有完整 |
| State Mapping | 🟡 `screen-lifecycle.md` 有 | 🔄 **Per-Region** | 每个 region 独立状态 |
| Focus Navigation | ✅ `focus-binding.md` 完整 | ✅ **不动** | 现有完整 |
| Interaction Zones | 🟡 Widget 级别有 | 🆕 **Screen 级补** | 07-specs/中 region 级标注 |
| Overlay Definition | ✅ `overlays.md` 完整 | ✅ **不动** | 现有完整 |
| Screen Lifecycle | ✅ `screen-lifecycle.md` 完整 | 🔄 **微调** | 状态机增加 OnReady |
| Data Ownership | ✅ `projection-viewmodel.md` 有 | ✅ **不动** | 现有完整 |
| AI Generation Rules (10条) | ❌ 不存在 | 🆕 **新增** | 07-specs/README.md |
| Definition of Done (12项) | ❌ 不存在 | 🆕 **新增（14项）** | 07-specs/README.md |

### 1.2 扩展规范（18ui草稿 §17-§34）

| 18ui草稿 概念 | 现有架构等价物 | 变动 | 行动 |
|--------------|---------------|------|------|
| Widget ID | 🟡 `UiBinding` 有参数化变体 | 🆕 **文档映射层** | 07-specs/references/widget-id-map.md |
| Layout Intent | ❌ 不存在 | 🆕 **新增** | 07-specs 中每个 Screen 标注 |
| Layout Priority | ❌ 不存在 | 🆕 **新增** | shrink: none/low/high |
| Scroll Policy | ❌ 不存在（代码里隐含） | 🆕 **新增** | 07-specs 中每个 region 标注 |
| Overflow Policy | ❌ 不存在 | 🆕 **新增** | clip/ellipsis/scroll |
| Empty State（per-region） | ❌ 不存在 | 🆕 **新增** | 07-specs 中 per-region 标注 |
| Error State（per-region） | ❌ 不存在 | 🆕 **新增** | 同上 |
| Loading State（per-region） | ❌ 不存在 | 🆕 **新增** | 同上 |
| Selection Model | ❌ 不存在 | 🆕 **新增** | single/multi/none |
| Ownership Boundary | ✅ 宪法有 | ✅ **不动** | 现有完整 |
| Event Contract | 🟡 散落在各处 | 🆕 **集中化** | 07-specs 中 per-Screen |
| Widget Reuse Policy | ❌ 不存在 | 🆕 **新增** | 07-specs 可选字段 |
| Forbidden Rules | ✅ 宪法 21 编有 | ✅ **不动** | 宪法完整 |
| Accessibility | 🟡 焦点系统有 | ✅ **预留** | 宪法已有条款 |
| Z-Layer | ❌ 不存在 | 🆕 **新增** | 07-specs/references/z-layer-spec.md |
| Animation Ownership | ❌ 不存在 | 🆕 **新增** | 07-specs + 宪法 |
| Widget Budget | ❌ 不存在 | 🆕 **新增** | max_depth / max_children |
| Screen Metrics | ❌ 不存在 | 🆕 **新增** | widget_count / container_count / 等 |

**统计**: 新增约 45% / 修改约 10% / 保留不动约 45%

---

## 2. Figma 彻底替代方案（用户明确要求：零 Figma）

### 2.1 核心理念

```
Figma 是"给人看"的 → 不适合 AI 消费
ASCII Wireframe 是"给 AI 看"的 → 机器可解析，人也能读
Excalidraw 是"给人画草图"的 → 偶尔需要视觉确认时使用
```

Fre 项目**零 Figma**。所有 Figma 规划的职能全部用已有的纯文本工具替代：

### 2.2 替代映射表

| Figma 原本职能 | 替代工具 | 存在性 | 输出格式 | 何时使用 |
|---------------|---------|--------|---------|---------|
| 页面布局设计 | **ASCII Wireframe** | 🆕 07-specs/ 强制 | Markdown 代码块 | 每个 Screen Spec 必有 |
| 组件视觉样式 | **Theme Token**（StyleToken / UiColors / UiSpacing） | ✅ 已有 `theme-localization.md` | Rust Enum | Widget 开发时引用 |
| 交互流程设计 | **Widget Contract**（Inputs/Outputs/Events） | ✅ 已有 `widget-atoms.md` | YAML Schema | 每个 Widget 定义时 |
| 页面间导航 | **ScreenStack + Excalidraw** | ✅ 已有 `navigation-overlay.md` | Markdown + 可选 Excalidraw | Screen 关系复杂时 |
| 高保真视觉稿 | **无需**（SRPG 不需要高保真原型） | — | — | 确定不做 |
| 设计评审 | **Spec diff review**（文档 diff 替代视觉 diff） | 🆕 流程创新 | Git diff | Spec 变更时 |
| 设计验收 | **DoD Checklist**（14 项自动检查） | 🆕 07-specs/README.md | Markdown checklist | 每个 Screen Spec 完成时 |
| 低保真草图 | **ASCII Wireframe**（主） + **Excalidraw**（辅，可选） | 🆕 07-specs/ | ASCII 图 / SVG | Screen 复杂布局时 |
| 标注文档 | **YAML Frontmatter + Flexbox 参数** | 🆕 07-specs/模板 | YAML | 每个 Widget 区域 |

### 2.3 工具链依赖声明

```
本项目的 UI 设计工具链（零 Figma）:

必须（每个 Screen Spec）:
├── ASCII Wireframe      ── 纯文本，Markdown 代码块，机器可读
├── YAML Layout Spec     ── 结构化尺寸/约束/意图
└── DoD Checklist        ── 14 项验证，确保完整性

可选（布局探索阶段）:
├── Excalidraw           ── 免费在线，无需账号，可导出 SVG
├── draw.io              ── 开源离线，VS Code 插件可用
└── Miro                 ── 团队协作白板，备选

绝对不使用:
❌ Figma                 ── 当前无人会用，且项目不需要视觉设计还原
❌ Adobe XD              ── 同上
❌ Sketch                ── 同上
❌ Photoshop / GIMP      ── 位图不适合 UI 布局定义
```

### 2.4 决策理由

```
为什么零 Figma 对 Fre 项目是更好的选择:

1. 50 万行 SRPG 的 UI 复杂度在「组合逻辑」不在「视觉保真」
   ── 战斗 HUD 不需要像素级还原，需要 Data Flow + Layout Constraint

2. Figma 的设计稿不能被 AI 直接消费
   ── Figma to code 工具链对 Bevy UI 无效
   ── ASCII Wireframe 是「AI 原生」的规范格式

3. 项目没有设计师
   ── 全程序员团队，纯文本工作流比 Figma 效率高 10 倍

4. 收益/投入比
   ── Figma 学习成本: 40h+（对程序员）
   ── ASCII Wireframe 学习成本: 5min
   ── 同样产出 Screen 布局规范，后者在一个大型项目中 ROI 高 100 倍
```

---

## 3. 遗漏概念补全集（v2 的 7 项 + v1 补充的 3 项 = 10 项）

### 3.1-3.7 v2 已覆盖的 7 项

| 编号 | 概念 | 18ui草稿 位置 | 文件已有 |
|------|------|--------------|---------|
| 3.1 | Responsive Rules | §6 | ✅ 已含 |
| 3.2 | OnReady 生命周期 | §13 | ✅ 已含 |
| 3.3 | Definition of Done | §16 | ✅ 已含 |
| 3.4 | Widget Reuse Policy | §28 | ✅ 已含 |
| 3.5 | Animation Ownership | §32 | ✅ 已含 |
| 3.6 | Widget Budget | §33 | ✅ 已含 |
| 3.7 | Per-Region State Mapping | §22-24 | ✅ 已含 |

### 3.8 🆕 Event Contract 的完整格式（v1 补回）

18ui草稿 §27 强调：不要写"点击物品"，要写结构化 Event Contract。

**项目适配**: 每个 Screen Spec 新增 §Event Contract 节，列出所有 UI ↔ Domain 事件：

```yaml
## Event Contract

### UI → Domain（通过 UiCommand 传递）

CastSkill:
  trigger_widget: action_menu → skill_btn → click
  data: { skill_id: SkillId, target_id: UnitId }
  conditions:
    - selected_skill != None
    - selected_target != None
    - ap_remaining >= skill_cost
  emits: UiCommand::CastSkill(SkillId, UnitId)
  domain_event: SkillExecuted | EffectApplied

EndTurn:
  trigger_widget: top_bar → end_turn_btn → click
  data: {}
  conditions: is_player_turn
  emits: UiCommand::EndTurn
  domain_event: TurnEnded

### Domain → UI（通过 Projection 消费）

DamageApplied:
  source: Domain Event (Combat Domain)
  projection: BattleProjection.project_damage()
  vm_update: BattleHudVm.hp ← damage_value
  side_effect: mark_dirty::<BattleHudVm>()

TurnStarted:
  source: Domain Event (Combat Domain)
  projection: BattleProjection.project_turn()
  vm_update: BattleHudVm.turn_number += 1
  vm_update: BattleHudVm.phase_key = "ui.battle.phase.player"
  side_effect: mark_dirty::<BattleHudVm>()
```

**Why 补回**: Event Contract 是 18ui草稿 §27 强调的"最高价值 7 规则"之一。v2 文件中只在 Widget Contract 的 outputs 字段提了一嘴，没有形成独立的、完整的 Event Contract 节。

### 3.9 🆕 Widget ID → UiBinding 映射表具体格式（v1 补回）

**项目适配**: `07-specs/references/widget-id-map.md` 的完整格式：

```yaml
# Widget ID → UiBinding 映射总表
# widget_id 一旦分配，永久有效。重构时标记 deprecated，不重新分配。

BattleScreen:
  root:                UiBinding::None        # Screen Root 容器
  top_bar:             UiBinding::None        # 顶栏容器
  turn_indicator:      UiBinding::Turn        # 回合数文本
  phase_label:         UiBinding::Phase       # 阶段标签
  end_turn_btn:        UiBinding::None        # 结束回合按钮（无 binding，由 Screen 管理交互）
  battle_area:         UiBinding::None        # 战斗场地容器
  char_panel:          UiBinding::None        # 角色面板容器
  hp_bar:              UiBinding::Hp          # HP 进度条
  mp_bar:              UiBinding::Mp          # MP 进度条
  buff_icons_0:        UiBinding::BuffSlot(0) # Buff 图标槽位 0
  buff_icons_1:        UiBinding::BuffSlot(1) # Buff 图标槽位 1
  action_menu:         UiBinding::None
  attack_btn:          UiBinding::None
  skill_btn:           UiBinding::None
  defend_btn:          UiBinding::None
  wait_btn:            UiBinding::None

MainMenuScreen:
  root:                UiBinding::None
  title_text:          UiBinding::Text        # 游戏标题
  new_game_btn:        UiBinding::None
  load_game_btn:       UiBinding::None
  settings_btn:        UiBinding::None
  version_text:        UiBinding::None

# ... 其他 Screen 同理
# 每当 UiBinding 枚举新增变体时，必须在此表追加一行
```

### 3.10 🆕 Layout Intent 跨 Screen 统一参考库（v1 补回）

**项目适配**: `07-specs/references/layout-intent-library.md`：

```yaml
# Layout Intent 统一参考库
# 跨 Screen 共享的尺寸约束意图

## 固定宽度 320px
regions:
  - widget_id: char_panel (battle_screen)
    width: 320
    intent: "角色头像+状态栏需要最小宽度，<320px 会导致文字折行、图标挤压"
    shrink: none
  - widget_id: character_list (inventory_screen)
    width: 320
    intent: "与战斗屏保持一致的角色列表宽度"
    shrink: none

## 固定高度 64px（顶栏）
regions:
  - widget_id: top_bar (battle_screen)
    height: 64
    intent: "标准顶栏高度，容纳回合信息 + 结束回合按钮"
    shrink: none

## 固定高度 120px（底栏）
regions:
  - widget_id: action_menu (battle_screen)
    height: 120
    intent: "底部行动菜单，120px 容纳 4 个按钮行 + 边距"
    shrink: none

## FlexGrow 优先占满
regions:
  - widget_id: battle_area (battle_screen)
    flex_grow: 1
    intent: "战斗场地优先占满剩余空间，地图越大越好"
  - widget_id: inventory_grid (inventory_screen)
    flex_grow: 1
    intent: "物品网格占满剩余空间，物品展示越多越好"

## 通用约束
global:
  min_interactive_height: 40px   # 可交互元素最小高度
  min_interactive_width: 40px    # 可交互元素最小宽度
  standard_padding: 8px          # 标准内边距
  standard_gap: 4px              # 标准间距
```

---

## 4. 可保留技术债清单（v1 补回）

以下内容**故意不修**，在当前阶段收益有限：

| 不修的债 | 理由 | 何时修 |
|---------|------|--------|
| `screens.md` 现有的纯文本描述 | Human 可读补充，不与 Spec 冲突 | Spec 全部完成后归档 |
| Screen Metrics 手动维护 | 初期数据量小，不值得自动化 | CI 门禁 Phase |
| ViewModel 未覆盖的字段 | 现有 BattleHudVm 够用，其他逐步补 | 各 Projection 实现时 |
| 现有代码中硬编码的 layout 尺寸 | 在重写对应 Screen 时自然消除 | 逐步替换 |
| MVP 阶段的静态样本数据 | BattleScreen/InventoryScreen 的硬编码数据 | ViewModel 集成迭代 |
| Scroll/Overflow 在已有 Widget 中缺失 | 按需补，不全局一次性改 | 各自维护迭代 |

---

## 5. 收益量化（v1 补回）

| 收益指标 | 当前基线 | 实施后预期 | 来源 |
|---------|---------|-----------|------|
| AI 生成 Screen 代码首次正确率 | ~40%（全靠猜布局） | **~80%** | 18ui草稿 §2 + §4 |
| 新 Screen 开发周期 | 3-5 天（反复调布局） | **1-2 天** | 综合评估 |
| Screen 布局相关 Bug | 高频（宽度溢出/高度不够/区域错位） | **减少 ~70%** | 18ui草稿 §18-21 |
| AI 猜测布局方向的无效代码 | ~60% 生成的代码需要手动修改 | **减少 ~90%** | 18ui草稿 §15 |
| Screen 复杂度失控预警 | 无任何预警 | **Widget Budget 拦截** | 18ui草稿 §33-34 |
| 多语言 UI 溢出问题 | 遇到才修 | **Overflow Policy 提前预防** | 18ui草稿 §21 |
| 焦点导航遗漏 | 经常遗漏键盘/手柄支持 | **Focus Navigation 强制定义** | 18ui草稿 §10 |

---

## 6. Agent 调度方案（v2 保留 + v3 细化）

遵循 AGENTS.md 的 Tier S → Tier A → Tier B 三级分治：

```
Tier S: 架构委员会 — 定义规则
Tier A: 工程委员会 — 确保合规
Tier B: 执行层 — 按规则交付
```

### Phase 1: Tier S 架构委员会并行启动

```
Day 1: presentation-architect 开始设计 Spec 格式
Day 1: data-architect 开始审查 Schema 映射
Day 1: content-architect 开始审查 Content 一致性
Day 2: architect 收到全部输入，开始写 ADR-066 和宪法修订
```

| Agent | 职责 | 输入 | 输出 |
|-------|------|------|------|
| **@presentation-architect** | 设计 Screen Spec 格式、模板、07-specs/ 目录结构 | 18ui草稿 + 现有 06-ui/ + 本计划 v3.0 | `07-specs/screen-spec-template.md` + Widget ID 映射表草案 + Z-Layer 规范草案 |
| @data-architect | 审查 Spec 中的 ViewModel 映射与已有 Schema 一致性 | `docs/04-data/` + Spec 模板草案 | 数据映射约束报告（确保 UiBinding ↔ ViewModel 字段一致） |
| @content-architect | 审查 Widget ID ↔ Def Registry ↔ LocalizationKey 映射一致性 | `docs/03-content/` + Spec 草案 | Content 兼容性报告（确保 Def 引用正确、Localization Key 不遗漏） |
| **@architect**（首席） | 撰写 ADR-066、宪法修订、系统集成 | 上述三者输出 + 现有架构 | `ADR-066-ui-screen-spec.md` + 宪法第九编/第十八编修订 |

**Tier S 协作规则**：
- presentation-architect 输出模板 → data-architect + content-architect 基于模板做审查
- architect 等三者全部输出后才开始集成
- data-architect 和 content-architect 可以并行审查

### Phase 2: Tier A 工程委员会审查

```
Day 3-4: 两个审查角色并行
```

| Agent | 职责 | 输入 | 输出 |
|-------|------|------|------|
| **@code-reviewer** | 审查现有 src/ui/ 代码，标注不符合新规范的模式 | `src/ui/` 代码 | `docs/10-reviews/screen-spec-code-gaps.md` |
| **@test-guardian** | 确保 Screen Spec 中的 State Mapping 有对应的测试模式 | `docs/05-testing/` + Spec 模板 | 测试扩展方案（Skeleton Widget / EmptyState Widget 的测试覆盖） |

**调用理由**:
- `@code-reviewer` → 只分析不修改（专长：代码质量审查）
- `@test-guardian` → 以领域规则优先设计测试（专长：不变量 + 边界条件）

### Phase 3: Tier B 执行层 + 快速迭代

```
Day 4-8: feature-developer 逐个产出，presentation-architect 做快速审查

顺序（按复杂度从低到高）:
1. MainMenuScreen    ← 最简单，验证模板可用性
2. SettingsScreen    ← 中等复杂，验证 TabPanel + Toggle 规范
3. InventoryScreen   ← 中等，需处理 Grid + List + Empty State
4. BattleScreen      ← 最复杂，最核心，需要最多迭代
5. ShopScreen        ← 未实现，Spec 先于代码
6. SaveLoadScreen    ← 未实现，Spec 先于代码
```

| Agent | 职责 | 输入 | 输出 |
|-------|------|------|------|
| **@feature-developer** | 实现 6 个 Screen Spec 文件 | ADR-066 + Spec 模板 + 各 Screen 现有文档 | `07-specs/screens/*.md` |
| **@refactor-guardian** | 扫描 `docs/06-ui/` 现有文档，标注缺失的 spec 类目 | `docs/06-ui/` | `docs/11-refactor/ui-doc-gaps.md` |
| **@presentation-architect**（副角色） | 每个 Screen Spec 的快速审查 | feature-developer 输出 | 审查意见 + spec 最终版本 |

**变更理由（相对于 v2）**: v2 把 BattleScreen 放在第一个，但更好的方式是**从简单到复杂**——先拿 MainMenuScreen 验证模板的完备性，再处理重头戏 BattleScreen。

### Phase 4: 宪法修订 + 文档归档

```
Day 9-10: architect 收尾
```

| Agent | 职责 | 输出 |
|-------|------|------|
| **@architect** | 宪法最终修订 + 文档生命周期管理 + 状态更新 | 宪法修订 + 归档 |

---

## 7. P0/P1/P2 优先级框架

| 等级 | 含义 | 不可退让 | 对应执行内容 |
|------|------|---------|------------|
| **P0** | 不完成则整个计划失败 | 🟥 绝对 | ADR-066 审批、宪法修订、Screen Spec 模板、BattleScreen + MainMenuScreen Spec、Widget ID 映射表 |
| **P1** | 重要，可稍后，但必须做 | 🟡 高优先级 | 剩余 4 个 Screen Spec、Z-Layer 规范、Scroll/Overflow 补全、Selection Model、Event Contract 集中化 |
| **P2** | 完善项，长期维护 | 🟢 有时间再做 | Layout Intent 库、Screen Metrics CI 门禁、Animation Ownership 代码层实现、Accessibility 增强 |

执行时必须**优先保证 P0 完成**，P0 不完成不进 P1。

---

## 8. 完整执行步骤（v2 + v3 合并）

### Step 1: 创建 07-specs/ 目录结构

```
docs/06-ui/
├── 07-specs/                         # NEW — AI-Consumable Screen Specification
│   ├── README.md                     # 总纲 + AI 14 条规则 + DoD 14 项清单
│   ├── screen-spec-template.md       # 完整模板（17 个字段）
│   ├── screens/                      # 每个 Screen 一个文件
│   │   ├── main_menu_screen.md       # P0-01: 最简单，先验证模板
│   │   ├── battle_screen.md          # P0-02: 最核心，最大工作量
│   │   ├── inventory_screen.md       # P1-01
│   │   ├── settings_screen.md        # P1-02
│   │   ├── shop_screen.md            # P1-03（Spec 先于代码）
│   │   └── save_load_screen.md       # P1-04（Spec 先于代码）
│   └── references/                   # 跨 Screen 统一参考
│       ├── widget-id-map.md          # Widget ID → UiBinding 映射总表
│       ├── z-layer-spec.md           # Z-Layer 统一规范
│       ├── layout-intent-library.md  # Layout Intent 跨 Screen 参考库
│       └── screen-metrics-baseline.md# 所有 Screen 的 metrics 基线
```

### Step 2: ADR-066 核心内容

```markdown
# ADR-066: UI Screen Specification 标准

状态: Proposed → Approved
负责人: architect

## 决策

引入 Screen Specification（SSPEC）标准，作为 AI 生成 UI 的前置规范。

## 核心规定

1. 每个 Screen 必须有对应的 Screen Spec 文档（P0）
2. Screen Spec 必须包含三位一体：ASCII Wireframe + Widget Tree + Flexbox Layout（P0）
3. Screen Spec 必须定义 widget_id、State Mapping、Scroll Policy（P0）
4. AI 生成 UI 代码前必须读取对应 Screen Spec（P0）
5. 新增 Screen 必须先写 Spec，再写代码（P0）
6. Screen Spec 必须通过 DoD 14 项检查才视为完成（P0）

## Figma 替代决策

Figma 从项目设计工具链中彻底移除。
替代方案：
  - ASCII Wireframe（布局规范，必选）
  - Excalidraw 或 draw.io（可选草图）
  - Markdown + YAML（结构化规范）
详见 `docs/06-ui/07-specs/README.md`。

## 宪法修订

- 第九编（UI 系统宪法）：Screen Spec 强制 + Widget ID 稳定 + 动画所有权 + Widget Budget
- 第十八编（工程质量）：Screen Metrics 基线追踪

## 不涉及变更

✅ 不修改现有运行时架构
✅ 不修改 Projection / ViewModel / Dirty<T> / UiBinding
✅ 不修改 Widget Contract 模式
✅ 不修改 Overlay 体系
```

### Step 3: 宪法修订清单

#### 3.1 第九编（UI 系统宪法）新增 3 条

```yaml
第 X 条：Screen Specification 强制（P0）
  | 🟩 | 每个 Screen 必须有对应的 Screen Spec 文档 |
  | 🟩 | Screen Spec 必须包含 ASCII Wireframe + Widget Tree + Flexbox Layout |
  | 🟩 | AI 生成 UI 代码前必须读取对应 Screen Spec |
  | 🟥 | 禁止在不读取 Screen Spec 的情况下由 AI 生成 Screen 布局代码 |
  | 🟥 | 禁止 Screen Spec 缺少任意一个「三位一体」元素 |
  | 🟥 | 禁止 AI 在生成 UI 时新增/删除/修改 Spec 中定义的 region |

第 X 条：Widget ID 稳定（P0）
  | 🟩 | 每个 Widget 实例必须有稳定的 widget_id |
  | 🟩 | widget_id 是永久标识，业务重命名时不允许修改 |
  | 🟩 | Widget ID → UiBinding 映射必须记录在 widget-id-map.md |
  | 🟥 | 禁止在重构时修改 widget_id（只能标记 deprecated，永不复用） |

第 X 条：UI 设计工具链（P0）
  | 🟩 | 项目使用纯文本工具链进行 UI 设计：ASCII Wireframe + Markdown + YAML |
  | 🟩 | Excalidraw / draw.io 可选用于复杂布局的快速草图 |
  | 🟥 | 禁止引入 Figma / Adobe XD / Sketch 等 GUI 设计工具 |
  | 🟥 | 禁止将视觉设计稿作为 Screen 实现的输入（Spec 才是权威） |

第 X 条：动画所有权（P1）
  | 🟩 | 动画必须明确声明 ownership（screen 级 vs widget 级） |
  | 🟥 | 禁止无动画归属声明的 UI 元素产生动画行为 |

第 X 条：Widget Budget 与 Screen Metrics（P1）
  | 🟩 | 每个 Screen 必须在 Spec 中定义 complexity budget |
  | 🟩 | 新增 Screen 时必须设定 metrics 基线 |
  | ⚠️ | CI 中 metrics 增长超过 30% → 告警 |
  | ⚠️ | Widget 嵌套深度超过 6 层 → 强制重构 |
```

#### 3.2 第十八编（工程质量）新增

```yaml
第 X 条：Screen 复杂度治理（P1）
  🟩 Screen Metrics 基线追踪：每个 Screen 必须记录 widget_count / container_count
  🟩 Widget Budget：max_widget_depth ≤ 6，max_children_per_container ≤ 20
  ⚠️ 超过阈值时必须重构，不得累积复杂度债务

第 X 条：Figma 替代工具链治理（P0）
  🟩 新增 Screen 的 UI 设计流程：写 Spec → DoD 检查 → 实现代码
  🟩 Spec 是 UI 设计的唯一真相源（SSOT），不依赖任何 GUI 设计工具
  🟥 禁止将 Figma/PSD/Sketch 文件作为 UI 需求附件
```

### Step 4: Screen Spec 模板关键字段（17 个）

| # | 字段 | P0/P1 | 对应 18ui草稿 位置 | 说明 |
|---|------|-------|-------------------|------|
| 1 | Screen Header | P0 | §2 | Screen Name, Purpose, Navigation, GameState |
| 2 | ASCII Wireframe | P0 | §3 | 纯文本线框图，所有区域命名 |
| 3 | Widget Tree | P0 | §4 | 标注 widget_id 的完整树 |
| 4 | Flexbox Layout | P0 | §5 | width/height/flex_grow + intent |
| 5 | Responsive Rules | P0 | §6 | 至少 strategy: none |
| 6 | Region Responsibility | P0 | §7 | 每区域 3-8 条职责 |
| 7 | Widget Contract | P0 | §8 | Inputs/Outputs/Selection Model |
| 8 | State Mapping | P0 | §9 + §22-24 | Per-Region Loading/Empty/Normal/Error |
| 9 | Focus Navigation | P0 | §10 | Tab 导航路径 |
| 10 | Interaction Zones | P0 | §11 | Click/Hover/Drag/Drop |
| 11 | Overlay Definition | P0 | §12 | Overlay 列表 + Z-Layer |
| 12 | Lifecycle | P0 | §13 | OnEnter/OnReady/OnExit |
| 13 | Data Ownership | P0 | §14 | Owns/Uses 分离 |
| 14 | Layout Intent | P0 | §18 | 关键尺寸的理由 |
| 15 | Scroll & Overflow Policy | P1 | §20-21 | 每个滚动区域 |
| 16 | Event Contract | P1 | §27 | UI↔Domain 事件完整契约 |
| 17 | Screen Metrics | P1 | §34 | widget_count / container_count 等 |

### Step 5: 现有架构文档微调（3 处）

| 文件 | 修改内容 | 影响范围 |
|------|---------|---------|
| `06-ui/03-screens/screen-lifecycle.md` §2.2 | 状态机新增 `OnReady` 弧：`Loading → OnReady → Active` | 生命周期语义升级 |
| `06-ui/03-screens/screens.md` | 每个 Screen 增加一行引用：`See also: 07-specs/screens/{name}.md` | 仅追加引用行 |
| `06-ui/README.md` | 索引表新增 `07-specs/` 条目 | 仅追加一行 |

---

## 9. 时间线总览

```
Week 1: P0 激进起步
────────────────────────────────────────────────────────────────────
Day 1:
  │ @presentation-architect → 设计 Spec 模板 + 07-specs/ 目录结构
  │ @data-architect → Schema 兼容性审查（并行）
  │ @content-architect → Content 一致性审查（并行）
  │
Day 2:
  │ @architect → 撰写 ADR-066（收到前三者输入后）
  │ @presentation-architect → 完成模板终稿
  │
Day 3:
  │ @architect → ADR-066 归档 + 宪法第九编/第十八编修订
  │ @code-reviewer → 现有代码差距扫描
  │ @test-guardian → 测试模式评估
  │ @refactor-guardian → 现有文档债务扫描
  │
Day 4-5: 第一个 Screen Spec
  │ @feature-developer → MainMenuScreen Spec（验证模板可用性）
  │ @presentation-architect → 快速审查（验证模板完备性）
  │
Day 6-7: 最核心的 Spec
  │ @feature-developer → BattleScreen Spec（最大工作量）
  │ @presentation-architect → 审查
  │

Week 2: P0 覆盖 + P1 推进
────────────────────────────────────────────────────────────────────
Day 8-9:
  │ @feature-developer → InventoryScreen + SettingsScreen Spec
  │ @presentation-architect → 批量审查

Day 10-11:
  │ @feature-developer → ShopScreen + SaveLoadScreen Spec
  │ @feature-developer → Widget ID 映射表
  │ @presentation-architect → Z-Layer 规范

Day 12:
  │ @architect → Widget ID 映射表终审 + Z-Layer 规范终审
  │ @architect → 更新 06-ui/README.md + screens.md 引用

Week 3: P1 完善 + 收尾
────────────────────────────────────────────────────────────────────
Day 13-14:
  │ @architect → Layout Intent 库 + Screen Metrics 基线
  │ @architect → 宪法最终修订

Day 15:
  │ @code-reviewer → 全面审查
  │ @architect → 文档归档 + 状态更新 + done/ 移动
```

---

## 10. 收尾流程

### 10.1 状态更新

| 文档 | 最终状态 |
|------|---------|
| `ADR-066-ui-screen-spec.md` | `status: approved` |
| `07-specs/README.md` | `status: active` |
| `07-specs/screen-spec-template.md` | `status: active` |
| `07-specs/screens/*.md`（6 个） | `status: active` |
| `07-specs/references/*.md`（4 个） | `status: active` |
| `docs/11-refactor/ui-screen-spec-execution-plan.md` | ✅ moved to `docs/11-refactor/done/` |
| `docs/06-ui/README.md` | 追加 07-specs/ 索引行 |
| `docs/01-architecture/README.md` | ADR 索引表追加 ADR-066 |

### 10.2 最终验收标准

- [ ] ADR-066 已批准并归档
- [ ] 宪法第九编已增加 Screen Spec / Widget ID / Figma替代 / 动画所有权 / Widget Budget 条款
- [ ] 宪法第十八编已增加 Screen Metrics + Figma 替代工具链治理条款
- [ ] `docs/06-ui/07-specs/` 目录已创建
- [ ] 07-specs/README.md 包含：总纲 + AI 14 条规则 + DoD 14 项清单
- [ ] screen-spec-template.md 包含全部 17 个字段
- [ ] 6 个 Screen Spec 全部完成（14 项 DoD 检查通过）
- [ ] widget-id-map.md 建立完整 Widget ID → UiBinding 映射
- [ ] z-layer-spec.md 建立 Z-Layer 统一规范
- [ ] layout-intent-library.md 建立跨 Screen 参考库
- [ ] screen-metrics-baseline.md 建立 metrics 基线
- [ ] `screen-lifecycle.md` §2.2 已增加 OnReady 弧
- [ ] `screens.md` 已追加 07-specs/ 引用行
- [ ] `06-ui/README.md` 已追加 07-specs/ 索引
- [ ] ADR 索引表已追加 ADR-066
- [ ] Figma 已从项目工具链中彻底移除（宪法明确规定）

---

## 11. 风险与缓解

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| Screen Spec 过于冗长导致 AI 过载 | 中 | 中 | 模板 17 个字段控制在 200 行内 |
| 无 Figma 导致视觉风格不一致 | 低 | 低 | Theme Token（StyleToken/Color/Spacing）统一管理视觉 |
| Widget ID 维护成本（仅文档层） | 中 | 低 | 初期只是文档映射，不涉及代码改动 |
| 团队不习惯 ASCII Wireframe | 中 | 中 | 学习成本 5min，收益立竿见影 |
| 现有代码中硬编码的 layout 与新 Spec 冲突 | 高 | 中 | 逐步替换，不一次性改 |
| 遗漏 18ui草稿 中的潜在概念 | 低 | 中 | 本文件已经覆盖全部 34 节 |
| Event Contract 导致文档膨胀 | 低 | 低 | 仅 Screen 级事件，不细化到每个 Widget 内部事件 |

---

*本文档由 @architect 维护。本计划 v3.0 吸收 18ui草稿 全部 34 节内容，上承 v1（架构分析）+ v2（遗漏概念补全 + Agent 编排），新增 Figma 替代方案 + v1 遗漏的 9 项内容。执行时严格遵循 AGENTS.md 的 Tier S → Tier A → Tier B 协作顺序。*
