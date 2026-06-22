---
id: 06-ui.architecture
title: UI Architecture — L3 表现层架构总纲
status: active
updated: 2026-06-22
owner: presentation-architect
created: 2026-06-20
tags:
  - ui
  - architecture
  - presentation
  - dataflow
  - l3
  - bevy-0.19
  - bsn
---

# UI Architecture — L3 表现层架构总纲

> **职责**: @presentation-architect | **上游**: ADR-055 (§1-§14), ADR-050, ADR-043, ADR-053, ADR-012, ADR-054
> **核心约束**: L3 是纯消费层，Domain/Core/Infra 禁止依赖 UI 层

> **SSPEC参考**: docs/06-ui/07-specs/ — AI-Consumable Screen Specification 标准。新增 Screen 必须先写 SSPEC，见 ADR-066。

---

## 1. 设计目的

项目 15 个 Capabilities + 15 个业务域 + 全部基础设施已实现，但缺少 Presentation Layer 的具体规范（ADR-055 §背景）。没有架构约束的 UI 代码在 50 万行规模下会迅速退化为不可维护的 Node 堆。

本文档定义 L3 UI 层的总体架构：层的定位、依赖方向、数据流、通信机制、对外接口。

---

## 2. L3 UI 层的定位

### 2.1 纵向四层体系

```
L0 Shared                      — 零业务语义的原子工具
L1 Core (Capabilities + Domains) — 业务逻辑与领域规则
L2 Infra                       — 技术基础设施
L3 UI (Presentation Layer)     — 表现层（本层）
```

（引用：ADR-055 §1 — UI 作为独立顶层模块）

### 2.2 依赖方向

```
UI → Core → Shared             （允许）
UI → Infra/Input               （允许，通过接口）
UI → Infra/Localization        （允许，通过 LocalizedText Component）
Core → UI                      （禁止，铁律）
Infra → UI                     （禁止）
```

**架构变更说明**：L3 在现有 DDD 三层 + 横切四层模型中新增。不改变现有纵向三层的依赖方向（Shared → Core → Infra），而是在 Infra 之上新增一个只读消费层。

### 2.3 与横切层的关系

| 横切层 | 与 UI 的关系 |
|--------|------------|
| App | 负责注册 UiPlugin（与注册其他 Plugin 一致），不包含 UI 业务逻辑 |
| Content | 不涉及 UI——UI 没有 Definition 需要加载；UI 的配置（Theme/样式）属于 StyleToken 体系 |
| Tools | DebugOverlay 通过 UI 的 `overlay/debug.rs` 子模块实现，仅在 dev feature 启用 |
| Modding | 未来可扩展 UI Mod，当前不在范围内；Mod 必须通过 Content → DefRegistry → Projection 路径影响 UI |

### 2.4 与运行时三层的关系

ADR-050 定义的 `GameState`/`OverlayState` 管理场景生命周期，`src/app/scenes/` 负责 GameState 的生命周期管理。UI Screen 由 `src/ui/screens/` 定义，两者通过 `OnEnter(GameState::X)` 关联：

### 2.5 Bevy 0.19 约束

本节汇总 Bevy 0.19 版本对 UI 架构的关键约束（依据 ADR-054）：

| 约束 | 范围 | 理由 |
|------|------|------|
| BSN 仅限声明式静态场景 | `src/app/scenes/`、Editor Prototype、Debug UI | BSN API 可能变动，声明式适合一次性装配（ADR-054 DR-003）；实证分析见下文 §2.5.1 |
| BSN 禁止带状态/逻辑/业务语义 | 所有使用 BSN 的地方 | 防止 BSN 树退化为不可维护的大型 DSL 块；BSN 无 if/match/循环语法，无法表达条件逻辑 |
| Screen 禁止使用 BSN | `src/ui/screens/` | Screen 生命周期复杂，需要 Factory 管理 spawn/despawn；实证分析见下文 §2.5.1 |
| Widget 禁止使用 BSN | `src/ui/widgets/` | Widget 需要测试、复用、审查、AI 独立生成；实证分析见下文 §2.5.1 |
| `Reflect` 全覆盖 | 所有 Component/Event/Resource | 编辑器支持、序列化、Scene 兼容（ADR-054 DR-005） |
| `trigger()` + Observer 优先 | 全项目 | 禁止 `EventWriter/EventReader`（ADR-054 DR-001） |
| `ButtonInput<T>` 替代 `Input<T>` | 输入系统 | 旧 API 在 0.19 废弃（ADR-054 §背景） |

**BSN 边界策略**：

```
BSN 允许范围：
  ├── src/app/scenes/        ✅ 允许 — Composition Root，一次性装配
  ├── Editor Prototype       ✅ 允许 — 快速原型
  ├── Debug UI               ✅ 允许 — 工具不涉及业务
  │
  ├── src/ui/screens/        🟥 禁止 — Screen 有复杂生命周期
  └── src/ui/widgets/        🟥 禁止 — Widget 需要 Factory 契约
```

### 2.5.1 BSN 适用性实证分析（2026-06-22）

本条目记录 presentation-architect 对 BSN 在本项目 UI 代码中的适用性所做的全面分析结论（见 `docs/10-reviews/ui-design-code-drift-review.md`）。

**核心结论**：BSN 能为本项目带来**有限**的增量收益（主要是 `Children[]` 嵌套语法使 UI 树结构更直观），但不足以推翻现有架构决策。BSN 与当前 Factory + ViewModel + Theme 注入 + LocalizedText 的 UI 架构存在系统性范式冲突。

**实证对比（取自实际代码）**：

| 维度 | 当前模式 | BSN 模式 | 收益 |
|------|---------|---------|------|
| Screen 层样板代码 | `spawn_zone() + set_parent_in_place + insert()` 链式调用 | `Children[]` 隐式嵌套 | **省 35-40% 行数** |
| Widget 层复杂度 | 工厂函数封装条件逻辑 + Theme 注入 | 需 `FromTemplate` + `Res<Theme>` 运行时解析 | **0-25% 行数**，核心逻辑不变 |
| Primitives 层核心工作 | match variant 决定 Node/layout | BSN 无 if/match 语法 | **0-5%**，无法替代 |
| UI 树可读性 | 线性代码 + ASCII 文档注释 | `Children[]` 嵌套声明 | **++** 但增量而非突破 |
| Theme 注入 | `&Theme` 显式参数传递 | `Res<Theme>` 隐式 World 访问 | 风格冲突，增加隐式依赖 |
| LocalizedText | `spawn_localized_text` 工厂处理 FTL key | 无现成集成，需自定义 `FromTemplate` | 无收益 |
| ViewModel/Dirty 刷新 | Projection → ViewModel → Dirty<T> 运行时链 | BSN 只影响 spawn 阶段 | **无关** |
| 条件逻辑 (variant) | match 分支选择不同布局 | BSN 不支持 if/match | **无法替代** |

**关键发现**：对 `src/app/scenes/` 中三类代码（循环驱动 spawn、后期附加组件、UI 工厂链）的实际 BSN 改写试验也确认无收益 — 当前 scenes 代码全是**运行时动态数据驱动**模式，BSN 的"静态声明式场景定义"优势不适用。

**推荐策略**：
1. 保持现有 §2.5 的 BSN 边界不变
2. 在 `src/app/scenes/` 中可用于新增的**纯静态场景**（如初始关卡布局），不对现有动态 spawn 做翻新
3. 在 Editor Prototype / Debug UI 等一次性工具代码中灵活使用
4. 待 Bevy 0.20+ 稳定（BSN 支持条件语法、FromTemplate/Localization 集成有最佳实践）后重新评估

**Factory 替代方案**：
```
✅ spawn_battle_screen(commands, battle_hud_vm)
✅ spawn_skill_panel(commands, skill_panel_vm, theme)
✅ spawn_hp_bar(commands, hp_bar_props, theme)
```

所有 Screen/Widget 通过 Factory 构建，Factory 是 UI 的唯一构建入口。Factory 输入仅限 Props、ViewModel、Theme，禁止直接读取 Domain。（宪法第九编 + ADR-054 DR-003）

> 🟥 `src/ui/` 层禁止使用 `bsn!{}` 宏

```
GameState ──OnEnter──→ Screen spawn（ScreenLayer）
GameState ──OnExit───→ Screen despawn（仅清理 ScreenLayer）
OverlayState ──Push──→ Popup spawn（PopupLayer，独立于 ScreenLayer）
```

（引用：ADR-055 §6 — UI Root 分层；ADR-055 §9 — 与 ADR-050 GameState 的 Screen 映射）

---

## 3. src/ui/ 目录结构

```
src/ui/
├── application/          # UI 应用层（Command Bus + Event Bus）
│   ├── command.rs        # UiCommand 枚举 + 分发
│   ├── event.rs          # UiEvent 枚举 + 广播
│   ├── intent.rs         # UiIntent（输入意图抽象）
│   └── mod.rs
├── projections/          # 投影层（Domain → ViewModel 的防火墙）
│   ├── battle.rs         # BattleProjection
│   ├── inventory.rs      # InventoryProjection
│   ├── character.rs      # CharacterProjection
│   ├── quest.rs          # QuestProjection
│   ├── economy.rs        # EconomyProjection
│   ├── selection.rs      # SelectionProjection（消费 SelectionChanged）
│   └── mod.rs
├── view_models/          # ViewModel 定义
│   ├── battle_hud.rs     # BattleHudVm
│   ├── character_panel.rs# CharacterPanelVm
│   ├── skill_panel.rs    # SkillPanelVm
│   ├── inventory.rs      # InventoryVm
│   ├── shop.rs           # ShopVm
│   ├── quest_log.rs      # QuestLogVm
│   ├── notification.rs   # NotificationVm
│   ├── selection.rs      # SelectionVm（选中/高亮状态）
│   └── mod.rs
├── primitives/           # L3-P: UI 原语层（唯一依赖底层 UI 实现的地方）
│   ├── button/           # PrimaryButton, SecondaryButton, DangerButton
│   ├── progress_bar/     # ProgressBar
│   ├── panel/            # Panel, CardPanel
│   ├── text/             # LocalizedText（对接 ADR-053）
│   ├── list/             # VirtualList
│   └── modal/            # ModalService
├── widgets/              # L3-W: 游戏业务控件（组合原语，骨架阶段）
│   ├── tooltip/          # TooltipService
│   ├── notification/     # NotificationService
│   └── mod.rs
├── screens/              # L3-S: 页面（组合控件，与 GameState 对应）
│   ├── battle/           # BattleScreen（对应 GameState::Combat）
│   ├── menu/             # MainMenuScreen（对应 GameState::MainMenu）
│   ├── inventory/        # InventoryScreen
│   ├── shop/             # ShopScreen（对应 OverlayState::Shop）
│   ├── settings/         # SettingsScreen
│   ├── save_load/        # SaveLoadScreen
│   └── mod.rs
├── overlay/              # 独立叠加层（不挂在 Screen 下）
│   ├── tooltip.rs        # TooltipOverlay
│   ├── damage_text.rs    # DamageTextOverlay（消费 CueType::Popup）
│   ├── notification.rs   # NotificationOverlay
│   ├── loading.rs        # LoadingOverlay
│   ├── debug.rs          # DebugOverlay（仅 dev feature）
│   └── mod.rs
├── navigation/           # 页面导航栈
│   ├── screen_stack.rs   # ScreenStack（push/pop/replace）
│   ├── screen_state.rs   # UiScreenState
│   └── mod.rs
├── theme/                # 主题/设计令牌
│   ├── colors.rs         # UiColors
│   ├── spacing.rs        # UiSpacing
│   ├── typography.rs     # UiTypography
│   ├── theme.rs          # Theme 系统
│   └── mod.rs
├── localization/         # UI 文本国际化
│   ├── text_keys.rs      # UiTextKey 枚举
│   └── mod.rs
├── picking/              # L3-I: 输入适配子层（命中检测 → PickIntent）
│   ├── mod.rs
│   ├── plugin.rs         # PickingUiPlugin（配置 backend + observer）
│   ├── pick_target.rs    # PickTarget 枚举
│   ├── pick_intent.rs    # PickIntent + InteractionPhase
│   ├── backend/          # 命中检测后端封装
│   │   ├── mod.rs
│   │   ├── sprite.rs     # SpritePickingPlugin 配置 + BoundingBox 模式
│   │   ├── grid.rs       # 网格后端（future）
│   │   └── passthrough.rs# UI 穿透（Pickable::IGNORE 策略）
│   └── intent/           # Pointer 事件 → PickIntent 转换
│       ├── mod.rs
│       ├── click.rs      # On<Pointer<Click>> → PickIntent::Commit
│       └── hover.rs      # On<Pointer<Over|Out>> → PickIntent::Preview
├── selection/            # 选择状态管理（消费 PickIntent，供给 Projection）
│   ├── mod.rs
│   ├── state.rs          # SelectionState 状态机（Option<BattleUnitId>）
│   ├── pick_context.rs   # PickContext Resource（Screen 写入，Picking 读取）
│   └── bridge.rs         # PickIntent → Domain Event 桥接
├── focus/                # 焦点系统
│   ├── focusable.rs      # Focusable Component
│   ├── focus_group.rs    # FocusGroup
│   └── mod.rs
├── binding/              # 数据绑定（ViewModel → Widget）
│   ├── dirty_flag.rs     # Dirty<T> 机制
│   ├── vm_binding.rs     # VmBinding trait
│   └── mod.rs
├── tests/                # UI 测试
│   ├── unit/             # Widget 测试
│   ├── snapshot/         # UI 树结构快照测试
│   └── integration/      # Screen 集成测试
├── plugin.rs             # UiPlugin
└── mod.rs
```

（引用：ADR-055 §2 — `src/ui/` 目录结构）

**与现有目录结构的关系**：
- `src/ui/` 是新增顶层目录，与 `src/core/`、`src/infra/` 平级
- `picking/` 和 `selection/` 从 `src/infra/picking/` 迁移而来（详见 ADR-068）
- 不修改现有 `src/` 下其他目录
- UiPlugin 注册在 Phase 11（在 Infra Phase 8 和 ScenePlugin Phase 9 之后）

### 3.6 三层依赖规则（Primitives / Widgets / Screens）

```
src/ui/
├── primitives/    L3-P: UI 原语层（依赖 Theme，实现底层 UI）
├── widgets/       L3-W: 游戏业务控件（组合原语）
└── screens/       L3-S: 页面（组合控件）
```

依赖规则：
- Primitives → 依赖 Theme（允许）
- Widgets   → 依赖 Primitives + Theme（允许）
- Screens   → 依赖 Widgets + Primitives + Theme（允许）
- Widgets   → 依赖 Bevy Node/Button/Interaction（禁止，必须通过 Primitives）
- Screens   → 直接操作 Primitives 组件（允许 — 组合层）

核心原则：**Primitives 是 UI 层与底层实现的唯一桥梁**。Widgets 和 Screens 只能通过 Primitives 的工厂函数和组件访问底层 UI 能力。

### 3.7 Picking/Selection 子层架构（L3-I）

`picking/` 和 `selection/` 是 UI 层的**输入适配子层**，职责是从 Bevy Picking 管线的原始命中检测到业务语义 PickIntent 再到领域事件的完整输入管线。

**设计原则**：

1. **Picking 只产生 PickIntent** — `picking/` 模块只做命中检测后端封装和 Pointer 事件到 PickIntent 的转换。不包含任何业务规则、Selection 状态管理、或领域事件触发。
2. **Picking 和 Selection 分离** — `selection/` 是 `picking/` 的消费者，不是子模块。Picking 不读取也不修改任何选择状态。
3. **Screen 决定 PickContext** — PickContext（当前交互模式：Normal/Attack/Skill/Move）由 Screen 根据游戏模式设置。Screen 写入 `PickContext` Resource（在 `ui/selection/pick_context.rs` 中定义），Picking 读取后附加到 PickIntent。
4. **Domain Bridge 在 selection/** — PickIntent → Domain Event（UnitClicked/TileClicked）转换在 `ui/selection/bridge.rs` 中实现。这是 UI → Domain 唯一出口。
5. **SelectionState 通过 Projection 流向 Widget** — SelectionState 不直接驱动 UI 表现，而是通过 `SelectionProjection` → `SelectionVm` → Dirty<T> 路径。

**数据流**：

```
Bevy Picking Pipeline (PreUpdate)
    ↓ Pointer<Click>/<Over>/<Out>
picking/intent/click.rs  (Observer)
    ↓ PickIntent { target, phase, context }
selection/bridge.rs  (Observer)
    ↓ trigger(UnitClicked) / trigger(TileClicked)
Domain (消费领域事件)
    ↓ SelectionChanged 事件
projections/selection.rs  (Observer)
    ↓ SelectionVm in UiStore
    ↓ Dirty<SelectionVm>
Widget/Overlay (消费 ViewModel)
```

**与三层渲染层的关系**：
- Picking 和 Selection 不操作 Node/Button/Interaction —— 它们通过 Observer 和 Resource 交互
- SelectionVm 的消费者可能是一个 Widget（如 TargetCursorWidget）或 Overlay（如 SelectionHighlight）
- Screen（如 BattleScreen）负责设置 PickContext 和注册领域事件的屏幕级响应

**Plugin 注册顺序**（详见 §8.2）：
```
6. OverlayPlugin
6.5 PickingUiPlugin       ← picking/plugin.rs（backend + observer）
6.6 SelectionPlugin       ← selection/（bridge + state machine）
7. Projection Observers   ← 包括 SelectionProjection
```

---

## 4. 四层单向数据流

### 4.1 Domain → UI 正向流

```
Domain Event（DamageApplied, HealthChanged, TurnStarted...）
    │  Observer + run_if（UI 注册的 Observer，带条件筛选）
    ▼
Projection（BattleProjection::project_damage）
    │  纯函数映射，输入 Domain Event → 输出 ViewModel 更新
    ▼
ViewModel（BattleHudVm.hp = 80）
    │  Dirty Flag 标记（Dirty<T> 机制）
    ▼
Widget（HpBarWidget::refresh）
    │  检测 Dirty 标记 → 刷新渲染 → 清除 Dirty
    ▼
屏幕更新
```

**数据流约束**：
1. Domain Event 是 UI 感知业务变化的唯一入口
2. Projection 是纯函数 —— 无副作用、无 System 依赖、可单元测试
3. ViewModel 是 Widget 的唯一数据源 —— Widget 不读取任何 Domain Component
4. Dirty<T> 机制确保 Widget 按需刷新，禁止每帧全量刷新

（引用：ADR-055 §3 — 单向数据流；domain rules §5.1 — Projection 更新流程；schema §3 — 数据流向）

### 4.2 UI → Domain 反向流

```
User Input（Click/Key/Touch）
    │  Input System 将原始输入映射为 UiIntent
    ▼
UiIntent（SelectSkill, ConfirmAction）
    │  路由到当前活跃 FocusGroup
    ▼
UiAction（Widget 级别的交互输出）
    │  UiActionHandler 转换为 UiCommand
    ▼
UiCommand（UiCommand::CastSkill(skill_id)）
    │  UiCommand → GameCommand 转换器
    ▼
Domain（执行业务逻辑，产生新的 Domain Event）
```

**反向流约束**：
1. UiIntent 是 UI 层的输入意图抽象，比 InputAction 更高层（InputAction 是硬件语义，UiIntent 是业务语义）
2. UiCommand 通过 `UiCommand → GameCommand` 转换器进入 ADR-043 定义的 CommandQueue，是 UI 层与 Command 层的唯一桥梁
3. 禁止 Widget 内直接使用 `EventWriter`/`EventReader` —— 使用 Observer 优先原则
4. **Picking 是 UiIntent 的补充而非替代**： Picking 输出 PickIntent（空间命中语义），不经过 UiIntent/UiAction 管线。PickIntent 通过 `selection/bridge.rs` 转换为 Domain Event 进入 CommandQueue。两条输入路径的关系： (a) 键盘/手柄 → InputSystem → UiIntent → UiAction → UiCommand (b) 鼠标/触摸 → bevy_picking → PickIntent → Domain Event → Selection → (可选) UiCommand

（引用：ADR-055 §3 — 反向流；ADR-055 §Communication Design — 通信表；domain rules §5.3 — 用户输入处理流程）

### 4.3 完整数据流图

```
Domain 层                              UI 层

EffectApplied ───────────────────────→ Observer
    │                                      │
    │                                CharacterProjection（纯函数）
    │                                      │
    │                                      ▼
    │                                CharacterPanelVm（UiStore 中）
    │                                      │
    │                                      ▼
    │                                CharacterPanel Widget 刷新
    │
    ▼
用户点击"施放技能" ──→ UiIntent::CastSkill
                          │
                          ▼
                     UiAction::CastSkill
                          │
                          ▼
                     UiCommand::CastSkill(SkillId)
                          │
                          ▼
                     GameCommand（ADR-043 CommandQueue）
                          │
                          ▼
                     Ability Domain 执行
```

### 4.4 三层渲染层（Primitives / Widgets / Screens）

正向数据流到达 Widget 后，经过三层渲染层构建实际 UI 树：

```
ViewModel（BattleHudVm）
    │
    ▼
Screen（BattleScreen）          — L3-S: 页面组合层
    │  组合 Widget，传递 ViewModel，管理生命周期
    ▼
Widget（CharacterStatusPanel）   — L3-W: 游戏业务控件层
    │  组合 Primitives，管理业务交互逻辑
    ▼
Primitives（Panel, ProgressBar） — L3-P: UI 原语层
    │  唯一操作 Bevy UI 底层类型（Node/Button/Interaction）的层
    ▼
Bevy UI 渲染
```

每层只依赖下层或同层，禁止跨层引用。Widget 不能直接操作 Node，Screen 不能绕过 Widget 直接操作 Primitives。

---

## 5. 通信机制

### 5.1 通信全景

| 通信路径 | 机制 | 方向 | 说明 |
|---------|------|------|------|
| Domain Event → Projection | Observer + run_if | Core → UI | UI 监听 Domain 事件，更新 ViewModel |
| ViewModel → Widget | Dirty Flag + Changed Filter | UI 内部 | Widget 检测 ViewModel 变化后刷新 |
| User Input → UiIntent | InputAction 映射 | Infra/Input → UI | 硬件输入转为业务意图 |
| UiIntent → UiCommand | Intent 解析 | UI 内部 | 业务意图转为 UI 命令 |
| UiCommand → GameCommand | 转换器 | UI → Core | UI 命令转为业务命令（唯一出口） |
| Pointer Event → PickIntent | Observer | UI 内部 | On<Pointer<Click\|Over\|Out>> → PickIntent |
| PickIntent → Domain Event | Observer | UI → Core | On<PickIntent> → trigger(UnitClicked) |
| Domain Event → Selection | Observer | Core → UI | On<UnitClicked> → SelectionState 状态机 |
| Selection → ViewModel | Observer | UI 内部 | On<SelectionChanged> → SelectionProjection |
| CueTriggered → Overlay | Observer | Core → UI | Cue 信号驱动 UI 表现 |
| GameState → Screen | OnEnter/OnExit | App → UI | 场景生命周期驱动 Screen spawn/despawn |
| OverlayState → Popup | PushOverlay/PopOverlay | App → UI | 覆盖层生命周期驱动 Popup spawn/despawn |
| LocalizationKey → Text | LocalizedText Component | Infra → UI | UI 读取 LocalizedText 自动渲染 |

（引用：ADR-055 §Communication Design — 通信表）

### 5.2 与 ADR-012（Cue）的对接

战斗中的伤害数字、治疗数字、状态提示等表现通过 Cue 系统实现：

```
Domain 触发 Cue → CueTriggered Event → UI Observer（damage_text.rs）→ Overlay 渲染
```

- `CueType::Popup` 由 `src/ui/overlay/damage_text.rs` 消费
- UI Observer 监听 `CueTriggered` 事件，根据 `CueType` 路由到对应 Overlay
- Cue 是逻辑 → UI 的单向通道，UI 不回写 Cue

（引用：ADR-055 §3 — 与 ADR-012/Cue 的对接）

### 5.3 与 ADR-043（Command）的对接

- UiIntent 是 UI 层的输入意图抽象，比 InputAction 更高层
- UiCommand 通过 `UiCommand → GameCommand` 转换器进入 ADR-043 定义的 CommandQueue
- 转换器在 `src/ui/application/command.rs` 中实现，是 UI 层与 Command 层的唯一桥梁

### 5.4 与 ADR-053（Localization）的对接

- 所有用户可见文本使用 `LocalizedText(UiTextKey)` 统一包装
- ViewModel 中文本字段使用 `UiTextKey`，不存储翻译后的文本
- 字体族使用语义类别（`FontSource::Family("heading")` / `"body"` / `"mono"`）
- FontSize 使用枚举（`FontSize::Px(14.0)` / `FontSize::Rem(1.2)`），禁止裸 f32

（引用：ADR-055 §4.3 — FontSize 枚举；§4.4 — FontSource 语义类别；domain rules §INV-UI-007）

### 5.5 与 ADR-050（GameState）的对接

GameState 与 UI Screen 的完整映射表参见 `screen-lifecycle.md §4`（Screen ↔ GameState 映射表）。

关键规则：
- `ScreenLayer` 的内容由 `OnEnter(GameState::X)` 驱动 spawn
- `PopupLayer` 的内容由 `PushOverlay(OverlayState::X)` 驱动 spawn
- `OnExit(GameState::X)` 只清理 `ScreenLayer`，不影响其他层
- `PopOverlay` 只清理 `PopupLayer`

### 5.6 与 Save/Replay 的对接

**Replay 原则**：UI 数据不影响游戏逻辑回放。Replay 只录制 Domain 级别的 Command（如 `CastSkill`, `EndTurn`），UI 从 Domain Event 重新投影。

| 数据类型 | 进入 Replay | 进入 Save |
|---------|------------|----------|
| ViewModel | 否（从 Domain Event 重新投影） | 否 |
| UiState（瞬态） | 否 | 否 |
| UiSettings | 否 | 是（SettingsGroup） |
| StyleToken/Theme | 否 | 否 |
| ScreenStack | 否 | 是（仅当前 Screen） |
| UiCommand（逻辑类） | 是（CastSkill, EndTurn 等） | 否 |
| UiCommand（UI 类） | 否（OpenScreen, CloseScreen） | 否 |

（引用：schema §16 — Replay Compatibility；schema §17 — Save Compatibility）

---

## 6. UI 宪法级规则

### 6.1 UI Query 禁止规则

禁止在 UI 模块中直接 Query Domain 组件。Projection 层自身可以 Query Domain Component（这是 Projection 的职责），但 Projection 输出的 ViewModel 不包含任何 Domain 类型。

```
// 禁止
fn update_hp_ui(query: Query<&Health>) { ... }

// 允许
fn update_hp_ui(vm: Res<BattleHudVm>) { ... }
```

**唯一例外**：Projection 自身可以 Query Domain Component，但 Projection 输出的 ViewModel 不包含任何 Domain 类型。

### 6.2 Widget 不持有 Entity

Widget 组件禁止包含 `Entity` 字段，使用业务 ID 替代（`SkillId`、`CharacterId`、`BuffId`）。

### 6.3 UI 动画不驱动业务逻辑

UI 是 Presentation Layer，不驱动业务流程。业务逻辑的执行时机由 Domain 决定，UI 只负责表现。违反此规则会导致"动画卡住则业务卡住"的耦合问题。

### 6.4 Screen 组合 Widget

Screen 只做组合，不直接拼 Node。布局细节属于 Widget 内部。

### 6.5 五层核心铁律（精简版）

1. **Domain 不依赖 UI** — Core/Infra 层禁止 import `ui/` 中的任何类型
2. **UI 不直接读 Domain** — 通过 ViewModel，禁止 Query Domain Component
3. **Screen 组合 Widget** — Screen 不直接拼 Node
4. **颜色字体间距统一 Token 化** — 禁止 `Color::srgb()` 直接写在 Widget 中
5. **Primitives 隔离** — Widgets 和 Screens 禁止直接 import Bevy UI 原语（Node、Button、Interaction、BackgroundColor 等），必须通过 Primitives 层

   > **豁免**: UI Root 层节点（Screen 根容器的直接子节点，如 Overlay 层级、占位符容器）允许直接 `commands.spawn` 原始 Node/BackgroundColor。此豁免仅适用于**没有对应 Primitives 工厂的特殊场景**，常规 UI 元素仍需通过工厂创建。

违反任何一条，50 万行后 UI 必然成为最大技术债。

（引用：ADR-055 §5 — UI 宪法级规则；§14 — 四条铁律；domain rules §INV-UI-009 — 四条铁律精简版）

### 6.6 INV-UI-006 例外声明：DamageTextOverlay

| 例外项 | 说明 |
|--------|------|
| 违反的不变量 | INV-UI-006（Overlay 必须有独立 Root 节点，Screen 销毁不影响 Overlay） |
| 违反的条款 | 本文件 §6.5 铁律第 1 条（Overlay 不挂在 Screen 下） |
| 例外对象 | DamageTextOverlay |
| 例外理由 | 战斗伤害数字是 BattleScreen 场景特有的表现元素（非全局 Overlay），其生命周期天然与战斗绑定：战斗结束（BattleScreen 销毁）时伤害数字必须同时消失。若将其放在独立层，需要在 OnExit(Combat) 中额外清理，增加维护复杂度。例外仅适用于纯粹的、无交互的、场景绑定的动画浮层。 |
| 约束条件 | (1) 仅限 DamageTextOverlay，不扩展至其他 Overlay 类型；(2) DamageText 无交互事件（不可点击），不产生 UiAction；(3) 必须在 architecture.md、navigation-overlay.md 和 overlays.md 中显式记录此项例外 |

### 6.7 Query Facade UI 约束

#### 6.7.1 原则

UI 层（Screen / Widget / Overlay）对 Domain 数据的访问必须通过 **ReadFacade / Integration 层**，禁止直接通过 ECS Query 访问 Domain 组件。

```
// ❌ 禁止：UI 代码直接 Query Domain 组件
fn update_spell_ui(
    query: Query<&Spellbook>,              // 直接读取 Domain 组件
    slot_query: Query<&SpellSlotPool>,     // 直接读取 Domain 组件
) { ... }

// ✅ 允许：UI 通过 ViewModel 消费数据
fn update_spell_ui(
    spell_vm: Res<SpellPanelVm>,           // UiStore 中的 ViewModel
) { ... }

// ✅ 允许（Projection 层专属）：通过 Integration QueryParam
fn project_spell_change(
    trigger: Trigger<SpellChanged>,
    spell_query: SpellQueryParam,          // Domain integration 层的 SystemParam 封装
    mut store: ResMut<UiStore>,
) { ... }
```

#### 6.7.2 数据访问层级

| 层级 | 数据源 | 适用模块 | 禁止 |
|------|--------|---------|------|
| Screen | ViewModel（`Res<BattleHudVm>`） | `src/ui/screens/` | 禁止 `Query<&Health>` |
| Widget | ViewModel（`Res<CharacterPanelVm>`） | `src/ui/widgets/` | 禁止 `Query<&Spellbook>` |
| Overlay | Cue 负载或 ViewModel | `src/ui/overlay/` | 禁止 `Query<&Health>` |
| Projection | Integration QueryParam + ViewModel | `src/ui/projections/` | 禁止直接修改 Domain |
| Primitives | 仅 Props 参数 | `src/ui/primitives/` | 禁止任何 Domain 感知 |

#### 6.7.3 SystemParam 封装模式

UI 代码需要读取 Domain 数据时，必须使用 Domain integration 层定义的 SystemParam 封装，而非原始 Query：

**合法模式**（Projection 层专属）：

```rust
// Spell 域的 SpellQueryParam 封装了所有法术相关的只读查询
fn project_spell_change(
    trigger: Trigger<SpellChanged>,
    spell_query: SpellQueryParam,           // 来自 spell/integration/query.rs
    mut store: ResMut<UiStore>,
) {
    if let Some(spellbook) = spell_query.get_spellbook(entity) {
        store.skill_panel.update_from_spellbook(spellbook);
        store.skill_panel.mark_dirty();
    }
}
```

**禁止模式**（所有 UI 代码）：

```rust
// ❌ 禁止：Screen 直接 Query Domain 组件
fn inventory_screen(query: Query<&Inventory>) { ... }

// ❌ 禁止：Widget 直接 Query Domain 组件
fn item_icon_widget(query: Query<&Item>) { ... }

// ❌ 禁止：在 Projection 中使用原始 Query 而非封装
fn project_bad(
    query: Query<&Health>,                  // 应该用 CharacterQueryParam
    mut store: ResMut<UiStore>,
) { ... }
```

#### 6.7.4 现有 QueryParam 参考

| Domain | SystemParam | 文件路径 | 提供的查询 |
|--------|------------|---------|-----------|
| Spell | `SpellQueryParam` | `src/core/domains/spell/integration/query.rs` | `get_spellbook()`, `get_slot_pool()`, `has_concentration()`, `remaining_slots()` |
| Combat | （拟新增 `CombatQueryParam`） | `src/core/domains/combat/integration/query.rs` | 战斗状态查询 |
| Inventory | （拟新增 `InventoryQueryParam`） | `src/core/domains/inventory/integration/query.rs` | 背包数据查询 |
| Party | （拟新增 `PartyQueryParam`） | `src/core/domains/party/integration/query.rs` | 队伍数据查询 |

#### 6.7.5 验证规则

| # | 规则 | 校验逻辑 |
|---|------|----------|
| QRY-VAL-01 | UI 模块参数不含 `Query<&DomainComponent>` | CI 静态分析：禁止在 `src/ui/` 下出现 `Query<&` 且类型属于 Domain 模块 |
| QRY-VAL-02 | Projection 使用 QueryParam 而非裸 Query | 代码审查：Projection 函数签名应使用 `SpellQueryParam` 而非 `Query<&Spellbook>` |
| QRY-VAL-03 | Screen/Widget 函数不含 Domain 组件引用 | 代码审查：Screen 和 Widget 系统函数的参数列表不应包含 `Query<&Component>` |
| QRY-VAL-04 | 新增 Domain 须提供对应的 QueryParam | 架构审查：每个 Domain 的 `integration/` 模块必须包含至少一个 QueryParam 封装 |

（引用：宪法 §8.9 — 读写分离原则；`src/core/domains/spell/integration/query.rs` — SpellQueryParam 实现参考；`src/core/domains/*/integration/facade.rs` — ReadFacade 参考）

---

## 7. UI 状态分级管理

| 级别 | 类型 | 生命周期 | 存储位置 | 示例 |
|------|------|---------|---------|------|
| Level 1 | 持久状态 | 跨会话 | `UiSettings`（bevy_settings） | show_damage_numbers, battle_speed |
| Level 2 | 会话状态 | 单次游戏 | ECS Resource | InventoryFilter, SelectedTab |
| Level 3 | 瞬态状态 | 单次交互 | ECS Component | Hover, Drag, Tooltip |

**关键约束**：
- 分开管理，禁止一个巨型 `UiState` Resource
- Level 1 通过 bevy_settings 持久化，不进入 Save 文件
- Level 2 不影响业务逻辑时，不进入 Save 文件
- Level 3 不持久化

（引用：ADR-055 §7 — UI 状态分级）

---

## 8. Plugin 注册架构

### 8.1 UiPlugin 外部位置

```rust
// Phase 11: UI Presentation Layer (L3)
// 在 Infra (Phase 8) 和 ScenePlugin (Phase 9) 之后
// 确保 Localization 已初始化（ADR-053）、GameState 已注册（ADR-050）
.add_plugins(ui::UiPlugin)
```

### 8.2 UiPlugin 内部注册顺序

```
1. ThemePlugin             — 主题与设计令牌（最优先）
2. FocusPlugin             — 焦点系统
3. PrimitivesPlugin        — UI 原语层注册（Button、Panel、Text 等）
4. WidgetsPlugin           — 游戏业务控件注册
5. ScreenPlugin            — Screen 注册
6. OverlayPlugin           — Overlay 注册
7. PickingUiPlugin         — Picking 输入适配（backend + observer）（需在 Screen 之后，因为 Screen 设置 PickContext）
8. SelectionPlugin         — 选择状态管理 + Domain Bridge（需在 Picking 之后，消费 PickIntent）
9. Projection Observers    — Domain Event → ViewModel Observer 注册（包括 SelectionProjection）
```

**Picking/Selection 注册时序说明**：
- PickingUiPlugin 在 ScreenPlugin 之后：因为 Screen 负责设置 PickContext Resource
- SelectionPlugin 在 PickingUiPlugin 之后：因为 Selection Bridge 消费 PickIntent
- Projection Observers 在 SelectionPlugin 之后：因为 SelectionProjection 消费 SelectionState
- 此顺序确保 Observer 的触发顺序：Pointer → PickIntent → DomainEvent → SelectionState → Projection → ViewModel

实际注册顺序见 `src/ui/plugin.rs`。

---

## 9. WidgetFactory Trait

所有 Widget 实现统一的工厂 trait，Screen 通过工厂组合 Widget：

```
pub trait WidgetFactory: Component {
    type Vm: Reflect + Default;
    fn create(commands: &mut Commands, vm: &Self::Vm) -> Entity;
    fn refresh(entity: Entity, vm: &Self::Vm, query: &mut Query<&mut Self>);
    fn destroy(commands: &mut Commands, entity: Entity);
}
```

- `refresh` 仅在 `Dirty<T>` 为 true 时调用，避免无谓刷新
- Screen 通过组合 `WidgetFactory` 构建 UI 树，不直接操作 Entity

（引用：ADR-055 §12 — WidgetFactory trait；schema §25 — WidgetFactory Schema）

---

## 10. 架构合规性约束

| # | 约束 | 违反后果 |
|---|------|---------|
| C1 | Core/Infra 禁止 import `ui/` 中的任何类型 | 编译期错误，CI 拒绝 |
| C2 | UI 模块禁止直接 Query Domain Component | CI 审查，ViewModel 空数据 |
| C3 | Widget 组件禁止包含 Entity 字段 | 运行时悬空引用 |
| C4 | UI 动画禁止驱动业务逻辑 | 动画 Skip 导致逻辑跳过 |
| C5 | Screen 禁止直接拼 Node | 审查拒绝，要求 Widget 封装 |
| C6 | 所有文本必须通过 LocalizationKey | 审查拒绝，违反宪法 §22 |
| C7 | 颜色/字体/间距必须使用 StyleToken | 审查拒绝，主题切换失败 |
| C8 | UI 不直接调用 NextState（走 StateTransitionQueue） | 违反 ADR-050 |

---

## 11. 架构决策索引

本架构规范引用以下 ADR：

| ADR | 内容 | 关系 |
|-----|------|------|
| ADR-055 | UI 表现层架构（本文档的上游） | UI 层定义的源文档 |
| ADR-050 | 游戏状态机与场景架构 | GameState/OverlayState → Screen 映射 |
| ADR-043 | 命令层与输入抽象 | UiCommand → GameCommand 转换 |
| ADR-053 | Localization 基础设施 | UiTextKey + LocalizedText 对接 |
| ADR-054 | Bevy 0.19 迁移决策 | BSN/Observer/Delayed Commands 采用 |
| ADR-012 | Stacking/Trigger/Cue 分离 | Cue → Overlay 消费路径 |
| ADR-040 | 数据流与所属权策略 | UI 只读 Domain 数据原则 |

---

*本文档由 @presentation-architect 维护。所有 L3 架构变更需经过 Presentation Architect 审查。*
