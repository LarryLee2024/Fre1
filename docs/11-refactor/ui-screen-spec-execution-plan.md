---
id: 11-refactor.ui-screen-spec-execution
title: UI Screen Specification 重构执行计划 v2.0（完整版）
status: proposed
owner: architect
created: 2026-06-22
tags:
  - ui
  - screen-spec
  - refactoring
  - ai-consumable
  - agent-orchestration
---

# UI Screen Specification 重构执行计划 v2.0

> **宪法依据**: 第九编（UI 系统宪法）、第十八编（工程质量）、第二十一编（红线禁止事项）
> **上游架构**: ADR-055（UI 表现层架构）、ADR-054（Bevy 0.19 迁移）、ADR-050（游戏状态机）
> **现有文档**: `docs/06-ui/`（16 个文件）
> **输入源**: `docs/99-history/ai_ignore_this_dir/18ui草稿.md`

---

## 0. 核心结论

### 0.1 三句话总结

1. **现有 `docs/06-ui/` 运行时架构零改动** — Projection/ViewModel/Screen 生命周期/Widget Contract/Dirty<T> 全部保留
2. **新增 `docs/06-ui/07-specs/` 目录** — AI 可消费的 Screen Specification 标准，与架构层物理隔离
3. **`18ui草稿` 中的遗漏概念补全** — 首次分析遗漏 7 个概念，全部在此版本追加

### 0.2 与现有架构的边界

```
现有 06-ui/（运行时架构层）            07-specs/（规范描述层）
────────────────────────              ────────────────────────
架构师维护 → AI 读                    架构师维护 → AI 读
定义运行时行为                         定义布局与约束
Projection, ViewModel, Widget...      ASCII Wireframe, Flexbox...
生命周期 states                        交互区域, 状态映射
禁止修改                               允许根据 spec 生成 screen
```

**物理隔离**: 新增 `07-specs/` 目录，不改动现有 14 个文件的 status/owner 行。两套文档共存，通过 README.md 关联引用。

---

## 1. 遗漏概念补全集（首次分析遗漏 7 项）

### 1.1 🆕 Responsive Rules（18ui草稿 §6）

**原文核心**: 如果项目固定分辨率，必须显式写 `Responsive: None`。否则定义断点：

```yaml
<1280:
  character_panel:
    width: 240
>=1280:
  character_panel:
    width: 320
```

**项目适配**: Fre 项目当前是固定分辨率设计。每个 Screen Spec 必须含：

```yaml
responsive:
  strategy: "none"  # 固定分辨率，不响应
  # 若未来引入响应式，在此追加断点
```

**Why missed**: 首次分析只关注了 Flexbox 的 width/height，漏掉了"不响应也要显式声明"这个约束。

### 1.2 🆕 OnReady 生命周期阶段（18ui草稿 §13）

**原文核心**: OnEnter 和 OnReady 是分开的：

```
OnEnter — LoadInventory（加载数据）
OnReady — SelectCurrentCharacter（初始化 UI 状态）
```

**项目适配**: 现有 `screen-lifecycle.md` 只有 OnEnter → Loading → Active，没有 OnReady。需要在 Screen Spec 中增加 OnReady 描述：

```yaml
lifecycle:
  on_enter:
    - register_projection_observers()
    - load_initial_viewmodel()
  on_ready:
    - select_default_character()
    - set_initial_focus()
  on_exit:
    - unregister_observers()
    - cleanup_selection()
```

**现有架构修改**: `docs/06-ui/03-screens/screen-lifecycle.md` §2.2 的状态机需要增加 OnReady 弧：`Loading → OnReady → Active`

### 1.3 🆕 Definition of Done 检查清单（18ui草稿 §16）

**原文核心**: 12 项 checklist，缺一不可。这是最被低估的强制约束机制。

**项目适配**: 在 `07-specs/README.md` 中固定 14 项 DoD（从 12 项扩展）：

```markdown
## Definition of Done Checklist

每个 Screen Spec 完成时必须全部满足：

- [ ] Header（Screen Name, Purpose, Navigation, GameState）
- [ ] ASCII Wireframe（所有区域命名，无匿名面板）
- [ ] Widget Tree（标注 widget_id，无隐藏节点）
- [ ] Flexbox Layout（width/height/flex_grow 全部标注）
- [ ] Responsive Rules（至少写 "strategy: none"）
- [ ] Region Responsibility（每区域 3-8 条具体职责）
- [ ] Widget Contract（Inputs/Outputs/Selection Model）
- [ ] State Mapping（Loading/Empty/Normal/Error 每个状态）
- [ ] Focus Navigation（键盘/手柄导航路径）
- [ ] Interaction Zones（每个区域交互类型标注）
- [ ] Overlay Definition（需要哪些 Overlay + Z-Layer）
- [ ] Lifecycle（OnEnter/OnReady/OnExit）
- [ ] Data Ownership（Owns/Uses 分离）
- [ ] Layout Intent（关键尺寸的理由说明）
- [ ] Scroll & Overflow Policy（每个滚动区域）
- [ ] Screen Metrics（widget_count/container_count/interactive_count/overlay_count/max_depth）

附加（P1 级别，非强制但有更好）：
- [ ] Widget Reuse Policy（是否强制现有 Widget 库）
- [ ] Animation Ownership（widget级/screen级动画归属）
- [ ] Widget Budget（max_widget_depth/max_children）
- [ ] Empty / Error / Loading 各区域的独立状态 Widget
```

### 1.4 🆕 Widget Reuse Policy（18ui草稿 §28）

**原文核心**: `reuse_only: true` — 禁止创建新 Widget，AI 必须使用现有组件库。

**项目适配**: 在 Screen Spec 中可选的 strict 声明：

```yaml
widget_reuse_policy:
  reuse_only: true
  allowed_widgets:
    - PanelWidget
    - ListWidget
    - GridWidget
    - TooltipWidget
  # 如果 false，允许创建 Screen-specfic 的专属 widget
```

这个政策在大型项目中防止 Widget 组件库爆炸。

### 1.5 🆕 Animation Ownership（18ui草稿 §32）

**原文核心**: 动画归 widget 管还是归 screen 管？不定义后期全乱。

**项目适配**: 在 Screen Spec 中声明：

```yaml
animation:
  ownership: screen      # screen 级别编排（进入/退出过渡）
  # 或 ownership: widget  # widget 自身管理（Tooltip 弹出/ProgressBar 过渡）

  transitions:
    enter: Fade(0.3s)
    exit: Fade(0.2s)
```

**现有架构影响**: `implementation-patterns.md` §4 Overlay 触发模式已涉及动画，但缺少 ownership 的明确声明。需在宪法第九编（UI 系统宪法）补充动画所有权规则。

### 1.6 🆕 Widget Budget（18ui草稿 §33）

**原文核心**: 限制复杂度，防止 AI 生成过度嵌套的节点树。

```yaml
budget:
  max_widget_depth: 6           # 容器嵌套不超过 6 层
  max_children_per_container: 20  # 单个容器不超过 20 个子节点
```

**项目适配**: 与 Screen Metrics 配套，在宪法中和 Screen Spec 中同时声明。

### 1.7 🆕 Per-Region State Mapping（18ui草稿 §22-24 的深层含义）

**原文核心**: Empty State、Error State、Loading State 不是 Screen 级别的概念——**每个区域都需要自己的状态处理**。

```yaml
inventory_screen:
  # Screen 级状态
  states:
    loading: LoadingSpinner
    error: ErrorBanner

  # 每个区域独立的状态
  regions:
    inventory_grid:
      loading_state: InventorySkeletonGrid
      empty_state: EmptyInventoryWidget    # ← 原文强调
      error_state: ErrorRetryWidget

    character_panel:
      loading_state: CharacterSkeletonCard
      empty_state: NoCharacterWidget       # ← 原文强调

    description_panel:
      empty_state: NoDescriptionWidget
```

**Why missed**: 首次分析把 State Mapping 简单归到 Screen 级别，忽略了"每个 region 各自处理状态"这一更细粒度的要求。

---

## 2. Agent 调度方案

遵循项目 AGENTS.md 定义的 Tier S → Tier A → Tier B 协作流程：

```
Tier S: 架构委员会（定义规则与边界）
Tier A: 工程委员会（确保合规与质量）
Tier B: 执行层（按规则交付）
```

### Phase 1: Tier S 架构委员会并行启动

```
Day 1-2: 四个架构师并行工作
```

| Agent | 职责 | 输入 | 输出 | 调用理由 |
|-------|------|------|------|---------|
| **@presentation-architect** | 设计 Screen Spec 格式、模板、07-specs/ 目录结构 | 18ui草稿 + 现有 06-ui/ | `07-specs/screen-spec-template.md` + 各 Screen Spec 草案 | **专长领域**: UI 架构设计，Tier S 中直接负责 Presentation 层 |
| **@data-architect** | 审查 Spec 中的 ViewModel 映射与现有 Schema 一致性 | `docs/04-data/` + Spec 模板草案 | 数据映射约束报告 | **双角色**: Tier S 设计 + Tier A 审查，确保 Replay/Save 兼容 |
| **@content-architect** | 审查 Spec 中的 Widget ID ↔ Def Registry 映射一致性 | `docs/03-content/` + Spec 草案 | Content 兼容性报告 | **专长**: Def 落地与 Registry 管理 |
| **@architect**（首席） | 撰写 ADR-066、宪法修订、系统集成 | 上述三者输出 + 现有架构 | `ADR-066-ui-screen-spec.md` + 宪法第九编/第十八编修订 | **首席职责**: 系统集成、ADR、模块边界 |

**协作原则**: 四者并行，但 architect 需要等前三者的输出才做集成。所以时间线：

```
Day 1:   presentation-architect 开始设计 Spec 格式
Day 1:   data-architect 开始审查 Schema 映射
Day 1:   content-architect 开始审查 Content 一致性
Day 2:   architect 收到全部输入，开始写 ADR-066
     ↓
Day 3:   ADR-066 完成 + 宪法修订
```

### Phase 2: Tier A 工程委员会审查

```
Day 3-4: 两个审查角色并行
```

| Agent | 职责 | 输入 | 输出 |
|-------|------|------|------|
| **@code-reviewer** | 审查现有 src/ui/ 代码，标注不符合新规范的模式 | `src/ui/` 代码 | `docs/10-reviews/screen-spec-code-gaps.md` |
| **@test-guardian** | 确保 Screen Spec 中的 State Mapping 有对应的测试模式 | `docs/05-testing/` | 测试扩展方案 |

**调用理由**:
- `@code-reviewer` → 做代码审查（专长领域：检查合规性）
- `@test-guardian` → 做测试评估（专长领域：以领域规则优先的测试设计）

### Phase 3: Tier B 执行层输出

```
Day 4-6: 两个执行角色
```

| Agent | 职责 | 输入 | 输出 |
|-------|------|------|------|
| **@feature-developer** | 实现 BattleScreen 的完整 Spec 文件作为参考实现 | ADR-066 + Screen Spec 模板 | `07-specs/screens/battle_screen.md` |
| **@refactor-guardian** | 扫描 `docs/06-ui/` 现有文档，标注缺失的 spec 类目 | `docs/06-ui/` | `docs/11-refactor/ui-doc-gaps.md` |

**调用理由**:
- `@feature-developer` → 按架构编码（专长：消费架构文档产出实现）
- `@refactor-guardian` → 扫描技术债（专长：六大维度债务扫描）

### Phase 4: 剩余 5 个 Screen Spec 批量执行

```
Day 6-10: feature-developer 逐个产出
```

顺序（按复杂度从低到高）：
1. MainMenuScreen（最简单，3 个 Widget）
2. SettingsScreen（TabPanel + Toggle，中等）
3. InventoryScreen（Grid + List，现有 MVP 需验证）
4. ShopScreen（未实现，Spec 先于代码）
5. SaveLoadScreen（未实现，Spec 先于代码）

每个 Screen Spec 产出后，经过 `@presentation-architect` 快速审查。

---

## 3. 完整执行步骤（含遗漏概念补全）

### Step 1: 创建 `07-specs/` 目录结构

```
docs/06-ui/
├── 07-specs/                    # NEW — AI-Consumable Screen Specification
│   ├── README.md                # 总纲：为何需要 Spec + AI 生成规则 14 条 + DoD 清单
│   ├── screen-spec-template.md  # Screen Spec 模板（含全部 17 个字段）
│   ├── screens/                 # 每个 Screen 一个文件
│   │   ├── battle_screen.md
│   │   ├── main_menu_screen.md
│   │   ├── inventory_screen.md
│   │   ├── settings_screen.md
│   │   ├── shop_screen.md
│   │   └── save_load_screen.md
│   └── references/              # 跨 Screen 参考
│       ├── widget-id-map.md     # Widget ID → UiBinding 映射总表
│       ├── z-layer-spec.md      # Z-Layer 统一规范
│       └── screen-metrics.md    # 所有 Screen 的 metrics 基准线
```

### Step 2: `07-specs/README.md` 核心内容

```markdown
# AI-Consumable Screen Specification (SSPEC)

## 目标

本规范用于描述 Screen 的结构，目标：

* AI 能直接生成 Bevy UI 代码
* 人类能快速阅读
* Layout 与 Widget 解耦
* Layout 可长期维护，可独立演进

## 两位一体约束

Screen = Layout（布局结构）
Widget = Implementation（渲染实现）

Screen Spec 只描述：信息架构、Widget 结构、Flexbox 布局、交互区域
Screen Spec 禁止描述：视觉样式、颜色、字体（这些归 Design System）

## AI 生成规则（14 条）

1. 严格按照 Widget Tree 生成，禁止新增区域
2. 严格按照 Flexbox Layout 生成尺寸
3. 禁止删除 Spec 中定义的任何区域
4. 禁止修改 Widget 名称和 widget_id
5. 禁止推测业务逻辑（让 Projection 负责）
6. 禁止推测视觉设计（让 Theme 负责）
7. 只负责布局实现，不负责业务数据获取
8. Widget 内部逻辑由 Widget 自身 Contract 负责
9. 若 Spec 与代码冲突：以 Spec 为准
10. 禁止跳过 Layout Intent 中标注的约束
11. Scroll Policy 必须按 Spec 实现，不能遗漏
12. 所有交互区域必须按 Interaction Zones 定义实现
13. 状态映射（Loading/Empty/Error）必须全部实现
14. 必须遵守 Widget Reuse Policy（若指定 reuse_only）

## Definition of Done（14 项，缺一不可）

[完整 14 项 checklist，见上文 §1.3]
```

### Step 3: ADR-066 核心内容

```markdown
# ADR-066: UI Screen Specification 标准

状态: Proposed
负责人: architect

## 决策

引入 Screen Specification（SSPEC）标准，作为 AI 生成 UI 的前置规范。

## 核心规定

1. 每个 Screen 必须有对应的 Screen Spec 文档
2. Screen Spec 必须包含：ASCII Wireframe + Widget Tree + Flexbox Layout
3. Screen Spec 必须定义：widget_id、State Mapping、Scroll Policy
4. AI 生成 UI 代码前必须读取对应 Screen Spec
5. 新增 Screen 必须先写 Spec，再写代码

## 宪法修订

- 第九编（UI 系统宪法）：新增 Screen Spec 强制条款
- 第十八编（工程质量）：新增 Screen Metrics + Widget Budget 条款

## 不涉及变更

- 不修改现有运行时架构
- 不修改 Projection/ViewModel/Dirty<T>/UiBinding
- 不修改 Screen 生命周期状态机（仅增加 OnReady 弧）
- 不修改 Widget Contract 模式
```

### Step 4: BattleScreen Spec 完整示例（Scaled-down）

`07-specs/screens/battle_screen.md` 的完整结构（约 200 行）：

```
---
id: 07-specs.battle-screen
title: BattleScreen Specification — AI-Consumable
status: draft
---

# BattleScreen

## 1. Screen Header

Screen: BattleScreen
Purpose: 战斗主界面，展示 HUD、角色状态、技能面板、行动菜单
Navigation: MainMenu → PartySetup → Combat
GameState: GameState::Combat

## 2. ASCII Wireframe

+----------------------------------------------------------+
| TopBar (top_bar)                              h:64        |
+----------------------------------------------------------+
| BattleArea (battle_area)  | CharacterPanel (char_panel)   |
| flex_grow:1              | w:320                         |
|                           |                               |
+----------------------------------------------------------+
| ActionMenu (action_menu)                    h:120         |
+----------------------------------------------------------+

## 3. Widget Tree

Root [battle_screen_root]
├── TopBar [top_bar: TurnBar]
│   ├── TurnIndicator [turn_indicator: LocalizedText]
│   ├── PhaseLabel [phase_label: LocalizedText]
│   └── EndTurnBtn [end_turn_btn: PrimaryButton]
├── BattleArea [battle_area: Panel]
│   └── (由 Tactical Domain 填充，UI 层为占位容器)
├── CharacterPanel [char_panel: CharacterCard]
│   ├── Portrait [portrait: Image]
│   ├── NameLabel [name_label: LocalizedText]
│   ├── HpBar [hp_bar: ProgressBar]
│   ├── MpBar [mp_bar: ProgressBar]
│   └── BuffIcons [buff_icons: StatusIcon × N]
└── ActionMenu [action_menu: Panel]
    ├── AttackBtn [attack_btn: PrimaryButton]
    ├── SkillBtn [skill_btn: SecondaryButton]
    ├── DefendBtn [defend_btn: SecondaryButton]
    └── WaitBtn [wait_btn: DangerButton]

## 4. Flexbox Layout

battle_screen_root:
  direction: Column
  intent: "全屏根容器，纵向排列"

top_bar:
  height: 64
  direction: Row
  intent: "固定高度条，显示回合信息和结束回合按钮"

battle_area:
  flex_grow: 1
  intent: "战斗场地，占满剩余空间，由 Tactical Domain 管理"

char_panel:
  width: 320
  intent: "固定宽度角色面板，最小 320px 保证可读性。禁止压缩"

action_menu:
  height: 120
  direction: Row
  intent: "固定高度底部菜单栏"

## 5. Responsive Rules

responsive:
  strategy: "none"   # 固定分辨率项目
  # 若后续支持窗口缩放，在此追加断点

## 6. Region Responsibility

top_bar:
  - 显示当前回合数
  - 显示当前阶段（己方/敌方）
  - 结束回合按钮

battle_area:
  - 战斗网格渲染（由 Tactical Domain 负责）
  - 单位选中高亮
  - 技能范围预览

char_panel:
  - 当前选中角色头像
  - 当前选中角色 HP/MP 条
  - 当前选中角色 Buff 图标

action_menu:
  - 可用行动按钮列表
  - 技能子菜单（展开时）

## 7. Widget Contract

top_bar:
  widget: TurnBar
  inputs: BattleHudVm
  outputs: UiAction::EndTurn
  selection: none

char_panel:
  widget: CharacterCard
  inputs: CharacterPanelVm
  outputs: UiAction::SelectCharacter
  selection: single

action_menu:
  widget: Panel (组合 PrimaryButton × N)
  inputs: BattleHudVm (ap_remaining 决定可用按钮)
  outputs: UiAction::Click (Attack/Skill/Defend/Wait)
  selection: none

## 8. State Mapping (Per-Region)

top_bar:
  loading: SkeletonBar
  normal: TurnBar
  error: ErrorText ("无法加载回合信息")

char_panel:
  loading: SkeletonCard
  empty_state: NoCharacterWidget
  normal: CharacterCard
  error: ErrorText ("角色数据异常")

battle_area:
  loading: LoadingSpinner
  normal: BattleGrid
  error: ErrorText ("战斗场地初始化失败")

action_menu:
  loading: SkeletonButtons × 4
  empty_state: NoActionsWidget
  normal: ActionMenu (按钮根据 AP 启用/禁用)
  error: ErrorText ("行动数据异常")

## 9. Focus Navigation

top_bar → (下) → battle_area → (右) → char_panel → (下) → action_menu

键盘导航路径：
  Tab: top_bar → battle_area → char_panel → action_menu
  Shift+Tab: 反向

## 10. Interaction Zones

top_bar:
  supports: [Click]  # EndTurnButton

battle_area:
  supports: [Click, Hover]  # 选中单位、悬停预览

char_panel:
  supports: [Click]  # 选择角色

action_menu:
  supports: [Click]  # 各按钮

## 11. Overlay Definition

overlays:
  - DamageTextOverlay     # 伤害数字浮层
  - TooltipOverlay        # 技能/物品提示
  - NotificationOverlay   # 升级/获得物品通知
  - ModalOverlay          # 暂停菜单/退出确认

z_layers:
  background: 0
  content: 100
  tooltip: 200
  modal: 300
  notification: 400

## 12. Lifecycle

on_enter:
  - register_battle_observers()
  - init_battle_hud_vm()
on_ready:
  - select_active_character()
  - set_focus_to_action_menu()
on_exit:
  - unregister_observers()
  - cleanup_selection_state()

## 13. Data Ownership

owns:
  - selected_skill: Option<SkillId>    # UI 内部选择状态
  - selected_target: Option<UnitId>    # UI 内部目标状态
  - battle_mode: BattleMode            # UI 内部模式

uses:
  - BattleHudVm                        # 从 Domain Projection 获取
  - CharacterPanelVm                   # 从 Domain Projection 获取
  - SkillPanelVm                       # 从 Domain Projection 获取

## 14. Widget Reuse Policy

widget_reuse_policy:
  reuse_only: true                     # 禁止创建新 Widget 类型
  comment: "BattleScreen 是平台屏，所有 Widget 必须来自现有组件库"

## 15. Animation Ownership

animation:
  ownership: screen                    # Screen 级别编排
  transitions:
    enter: Fade(0.3s)
    exit: Fade(0.2s)
  comment: "战斗进场/退场的过渡动画由 Screen 管理"

## 16. Widget Budget

budget:
  max_widget_depth: 4                  # Root → Panel → Widget → Child
  max_children_per_container: 8        # 不超过 8 个子节点

## 17. Screen Metrics

metrics:
  widget_count: 15                     # 主要 UI 元素数
  container_count: 5                   # 容器数
  interactive_count: 6                 # 交互元素数（按钮等）
  overlay_count: 4                     # Overlay 依赖数
  max_depth: 4                         # 最大嵌套深度
```

### Step 5: 宪法修订清单

#### 5.1 第九编（UI 系统宪法）新增 2 条

```yaml
第 X 条：Screen Specification 强制（P0）
  | 标记 | 规则 |
  |------|------|
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

第 X 条：动画所有权（P1）
  | 🟩 | 动画必须明确声明 ownership（screen 级 vs widget 级） |
  | 🟥 | 禁止无动画归属声明的 UI 元素产生动画行为 |

第 X 条：Widget Budget 与 Screen Metrics（P1）
  | 🟩 | 每个 Screen 必须在 Spec 中定义 complexity budget |
  | 🟩 | 新增 Screen 时必须设定 metrics 基线 |
  | ⚠️ | CI 中 metrics 增长超过 30% → 告警 |
  | ⚠️ | Widget 嵌套深度超过 6 层 → 强制重构 |
```

#### 5.2 第十八编（工程质量）新增

```yaml
第 X 条：Screen 复杂度治理
  🟩 Screen Metrics 基线追踪：每个 Screen 必须记录 widget_count / container_count
  🟩 Widget Budget：max_widget_depth ≤ 6，max_children_per_container ≤ 20
  ⚠️ 超过阈值时必须重构，不得累积复杂度债务
```

### Step 6: 现有架构文档微调

| 文件 | 修改内容 | 影响范围 |
|------|---------|---------|
| `06-ui/README.md` | 新增 `07-specs/` 索引行 | 仅追加一行 |
| `06-ui/03-screens/screen-lifecycle.md` §2.2 | 状态机新增 `OnReady` 弧（Loading → OnReady → Active） | 生命周期语义升级 |
| `06-ui/03-screens/screens.md` | 每个 Screen 增加引用：`See also: 07-specs/screens/{name}.md` | 仅追加引用行 |
| `06-ui/01-architecture/implementation-patterns.md` | 新增 ScrollPanel 完整实现模式、动画 ownership 规范 | 新增内容 |
| `docs/01-architecture/README.md` | ADR 索引表新增 ADR-066 | 仅追加一行 |

---

## 4. 时间线总览

```
Week 1 (激进起步 — 不可退让)        Agent
─────────────────────────         ─────────────
Day 1: ADR-066 撰写                @architect
Day 1: Screen Spec 模板创建         @presentation-architect
Day 1: 数据/内容兼容审查             @data-architect + @content-architect
Day 2: 宪法修订                     @architect
Day 2: 现有代码差距扫描              @refactor-guardian
Day 2: 测试模式评估                  @test-guardian
Day 3: BattleScreen Spec 完成       @feature-developer
Day 3: ADR-066 归档 + 状态更新       @architect

Week 2 (覆盖推进)
Day 4-5: 剩余 5 个 Screen Spec     @feature-developer × 5
Day 5-6: 每个 Spec 的架构审查       @presentation-architect
Day 5: Widget ID 映射表             @feature-developer
Day 6: Z-Layer 统一规范             @presentation-architect
Day 6: 文档更新与 README 同步        @architect

Week 3 (收尾)
Day 7-8: 宪法最终修订                @architect
Day 7-8: 文档归档与 done/ 移动       @architect
Day 9: 整体审查                      @code-reviewer
Day 10: 验证 + 关闭                  @architect
```

---

## 5. 收尾流程

按照 `docs/00-governance/ai-constitution-complete.md` 中的文档生命周期管理要求：

1. **更新状态标记**
   - ADR-066 → 完成后设置 status: approved
   - 各 Screen Spec → status: active
   - 11-refactor/ui-screen-spec-execution-plan.md → status: completed（全部完成后）

2. **归档完成项**
   - 当所有 Screen Spec 完成后，`07-specs/screens/` 下文件全部 active
   - 本执行计划完成后移入 `docs/11-refactor/done/`

3. **更新 README.md**
   - `docs/06-ui/README.md` — 添加 07-specs/ 索引
   - `docs/11-refactor/README.md` — 更新状态

---

## 6. 风险与缓解

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| Screen Spec 过于冗长导致 AI 过载 | 中 | 中 | 模板控制在 200 行内，核心字段 17 个，不赘述 |
| 现有架构文档需小量修改 | 低 | 低 | 仅改 screen-lifecycle.md 的 state machine + 增加 OnReady |
| Widget ID 维护成本 | 中 | 低 | 初期只是文档层映射，不涉及代码改动 |
| 团队不接受新的文档格式 | 低 | 高 | 通过 ADR 评审确保共识 |
| 与 Figma 或其他设计工具冲突 | 低 | 低 | Figma 作为可选补充，Spec 是主要规范 |

---

## 7. 最终验收标准

所有 Phase 完成后检查：

- [ ] ADR-066 已批准并归档
- [ ] `docs/06-ui/07-specs/` 目录已创建，包含 README.md + template + 6 个 Screen Spec
- [ ] 宪法第九编已增加 Screen Spec 条款
- [ ] 宪法第十八编已增加 Screen Metrics 条款
- [ ] `screen-lifecycle.md` §2.2 已增加 OnReady 弧
- [ ] Widget ID → UiBinding 映射表已建立
- [ ] Z-Layer 统一规范已写入
- [ ] `docs/06-ui/README.md` 已追加 07-specs/ 索引
- [ ] ADR 索引表已追加 ADR-066
- [ ] Screen Spec 模板已包含 17 个完整字段（含遗漏补全的 7 项）
- [ ] 所有 Screen 的 Definition of Done 检查已完成（14 项）


*本文档由 @architect 维护。所有 Agent 执行时严格遵循 Tier S → Tier A → Tier B 协作顺序。*
