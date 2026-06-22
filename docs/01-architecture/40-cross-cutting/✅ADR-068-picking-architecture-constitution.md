---
id: 01-architecture.ADR-068
title: "ADR-068: Picking 架构总纲（宪法级）"
status: Proposed
owner: architect
created: 2026-06-23
updated: 2026-06-23
supersedes: ADR-067
---

# ADR-068: Picking 架构总纲（宪法级）

## 状态

**Proposed** — 本 ADR 取代 ADR-067 中关于模块定位和架构原则的部分，保留 ADR-067 的技术实现分析（SpritePickingPlugin 兼容性、帧时序分析等）作为参考。

## 背景

现有 Picking 架构（ADR-067）将 Picking 放在 `src/infra/picking/` 作为基础设施层模块。经过对项目架构总纲 (`docs/01-architecture/README.md` §2.4 运行时三层)、UI 表现层架构 (`docs/06-ui/`)、和宪法 §1.5 P0 原则的综合分析，确认 **Picking 不属于 Infra 层**。

理由：
1. **Picking 是用户输入到业务语义的第一层转换** — 属于 Presentation Layer 的 Input Adapter 职责
2. **Infra 层是纯技术实现** — Registry/Pipeline/Replay/Save/Camera/Input — 这些都是"无业务语义"的技术基础设施
3. **Picking 的产物是业务意图（PickIntent/PickTarget）** — 已经包含领域语义（Unit/BattleUnitId），不属于纯技术范畴
4. **Presentation Layer 包含输入适配** — ADR-055 §3 和 `architecture.md` §5.1 定义 UI→Domain 反向流，从 UiIntent 开始；Picking 是 UiIntent 的源头之一

## 引用的领域规则与架构规则

- **宪法 §1.5(4)** — Logic/Presentation Separation：业务逻辑与视觉表现彻底解耦
- **宪法 §1.5(9)** — Camera Event 驱动：所有外部镜头操作必须通过 CameraRequest
- **ADR-055 §3 反向流** — UiIntent 是 UI 层的输入意图抽象
- **docs/06-ui/01-architecture/architecture.md §5.1** — Communication 全景图
- **docs/06-ui/01-architecture/architecture.md §2.4** — 运行时三层：Domain / Application / Presentation
- **docs/01-architecture/README.md §2.4** — 运行时三层 = Domain (纯函数) / Application (ECS Systems) / Presentation (UI/VFX/SFX/Camera)
- **用户提供的 14 条 Picking 架构原则**（作为本 ADR 的直接输入）

## 决策

### 1. Picking 归属 Presentation Layer（L3）

Picking 从 `src/infra/picking/` 迁移到 `src/ui/picking/`，作为 Presentation Layer 的**输入适配子层**。

```
架构层次变化:

之前 (ADR-067):
  infra/picking/          ← L2 Infra（技术基础设施）
    ↓ SpritePickingPlugin  ← Bevy 内置技术实现
    ↓ Selection Resource   ← 直接存储 Entity

之后 (本 ADR):
  ui/picking/             ← L3 Presentation（输入适配层）
    ↓ backend/             ← Bevy Picking 后端封装
    ↓ intent/              ← PickIntent 转换
  ui/selection/           ← L3 Presentation（选择状态管理）
    ↓ bridge.rs            ← 领域事件桥接
```

依赖方向变化:
- 之前: `infra/picking/` 符合 Infra 规则，但 `Selection` 被 Core Domain 读取，形成 Infra→Core 的数据流入
- 之后: `ui/picking/` → `core/domains/` 通过领域事件，符合 L3→L1 方向

### 2. Picking 三层职责

| 层 | 职责 | 禁止 |
|----|------|------|
| **Backend** | 命中检测（Sprite/Grid 坐标换算），产出 `PointerHits` | 禁止引入任何业务类型；禁止修改游戏状态 |
| **Intent** | 将 Pointer 事件转换为 `PickIntent`，附加 `PickContext` | 禁止直接产生 Selection；禁止引入 Domain 规则 |
| **Domain Bridge** | 将 `PickIntent` 转换为领域事件（如 `UnitClicked`） | 禁止包含业务规则；只做类型转换 |

### 3. Hover / Focus / Selection 三态分离

当前 `Selection` Resource 同时承担 hover、selected 和部分 focus 功能，必须分离：

| 状态 | 生命周期 | 驱动方式 | 存储位置 |
|------|---------|---------|---------|
| **Hovered** | 帧级瞬态，Pointer<Over>/<Out> | 自动由 Bevy picking 驱动 | `On<Pointer<Over>>` observer 本地处理 |
| **Focused** | 会话级，跨多次交互 | Tab/Click 切换 | `FocusTarget` Component 或 Resource |
| **Selected** | 行动级，Commit 后保持到下一个 Commit | PickIntent::Commit → Selection 状态机 | `SelectionState` Resource |
| **Targeted** | 技能瞄准时 | PickIntent + 当前 ActionContext | `TargetingState` Component |
| **Activated** | 行动执行中 | Selection 状态机自动管理 | 瞬态，事件触发即消失 |

### 4. Picking 产物为 PickTarget，非 Entity

```
PickTarget::Unit(BattleUnitId)    ← 领域 ID，稳定且与 Def 关联
PickTarget::Tile(GridPos)         ← 网格坐标，适用于地图交互
PickTarget::Entity(Entity)        ← 只用于非单位实体的瞬态交互
PickTarget::UI(Entity)            ← UI 元素，通过 Interaction 系统处理
PickTarget::None                  ← 空点击
```

**Entity 禁止作为主识别符**：仅在跨帧需要引用 ECS 实体时使用，且必须伴随 Domain ID 校验。

### 5. PickIntent 与 PickContext

```rust
/// PickIntent — Picking 的输出，Selection 的输入
pub struct PickIntent {
    pub target: PickTarget,
    pub pointer_button: PointerButton,
    pub context: PickContext,
    pub phase: InteractionPhase,
}

pub enum InteractionPhase {
    /// Preview：预览/悬停，不应产生副作用
    Preview,
    /// Commit：确认/点击，可触发状态变更
    Commit,
}

pub enum PickContext {
    /// 普通模式 — 点击选中/取消
    Normal,
    /// 攻击瞄准 — 点击施法者/技能目标
    AttackTargeting { caster: BattleUnitId, skill: SkillId },
    /// 技能瞄准 — 点击目标格/单位
    SkillTargeting { caster: BattleUnitId, skill: SkillId },
    /// 移动选择 — 点击目标格
    MoveSelection { unit: BattleUnitId, range: u32 },
    /// 物品使用 — 点击目标
    ItemTargeting { item: ItemId },
}
```

PickContext 由当前游戏模式决定：Normal 模式下点击产生 UnitSelected，AttackTargeting 模式下点击产生 AttackTarget。

### 6. Selection 与 Picking 完全分离

```
Picking System (PreUpdate)
  ↓ PickIntent
Intent Router (检测当前 PickContext)
  ↓
Domain Event (UnitClicked / TileClicked / ActionSelected / TargetSelected)
  ↓
Selection State Machine (消费 Domain Event，更新 State)
  ↓
Projection System (读取 SelectionState → ViewModel)
```

Picking 不知道 Selection 的存在。Picking 只产生 PickIntent，不读取也不修改任何游戏状态。

## Module Design

### 目标目录结构

```
src/ui/
├── picking/                      # Presentation Layer 输入适配子层（只做命中检测 + PickIntent 生产）
│   ├── mod.rs                    # 模块声明 + pub re-export
│   ├── plugin.rs                 # PickingUiPlugin（注册 backends + observers + resources）
│   ├── pick_target.rs            # PickTarget 枚举
│   ├── pick_intent.rs            # PickIntent + InteractionPhase
│   │
│   ├── backend/                  # Picking 后端封装层
│   │   ├── mod.rs
│   │   ├── sprite.rs             # SpritePickingPlugin 配置 + BoundingBox 模式
│   │   ├── grid.rs               # 网格后端（Tiled 集成后启用：world_pos → tile_pos）
│   │   └── passthrough.rs        # UI 穿透策略（Pickable::IGNORE 自动标记）
│   │
│   └── intent/                   # Pointer 事件 → PickIntent 转换
│       ├── mod.rs
│       ├── click.rs              # Pointer<Click> → PickIntent::Commit
│       └── hover.rs              # Pointer<Over>/<Out> → PickIntent::Preview
│
├── selection/                    # 选择状态管理（与 picking 平级，职责分离）
│   ├── mod.rs
│   ├── state.rs                  # SelectionState 五态状态机（Hovered/Focused/Selected/Targeted/Activated）
│   ├── pick_context.rs           # PickContext Resource（Normal/AttackTargeting/SkillTargeting/Inspect）
│   └── bridge.rs                 # PickIntent → Domain Event 桥接（UnitClicked, TileClicked, TargetConfirmed）
│
└── projections/selection.rs      # SelectionProjection — SelectionState → ViewModel 投影
```

### 外部依赖

```
ui/picking/       → bevy::picking (bevy_picking backend traits)
                  → bevy::sprite (SpritePickingPlugin)
                  → ui/selection/ (PickIntent → Domain Event)
                  → ui/application/ (UiCommand 通道)
                  → core/domains/combat (BattleUnitId)
                  → core/domains/tactical (GridPos)

ui/selection/     → core/domains/ (Domain Event: UnitClicked, TileClicked, TargetConfirmed)
                  → infra/camera/ (CameraRequest 👈 通过事件解耦)
                  → infra/picking/ ❌ 删除，迁移完成即可移除
```

## Communication Design

### 事件流全景

```
玩家点击
  │
  ▼
PreUpdate: bevy_picking::PickingPipeline (Backend → Hover → PointerEvent)
  │  SpritePickingPlugin 检测命中 → PointerHits
  │  InteractionPlugin → Pointer<Click>/<Over>/<Out>
  │
  ▼
PreUpdate: ui/picking/backend/ (配置项，非系统)
  │  Pickable::IGNORE / BoundingBox 模式
  │
  ▼
Observer: ui/picking/intent/click.rs
  │  On<Pointer<Click>> → 读取 PickContext → 创建 PickIntent
  │  输出: PickIntent { target, context, phase: Commit }
  │
  ▼
Observer: ui/selection/bridge.rs
  │  消费 PickIntent → 如果是 PickTarget::Unit → trigger UnitClicked
  │  如果是 PickTarget::Tile → trigger TileClicked
  │
  ▼
Domain Observer: core/domains/tactical/events.rs 或 selection/
  │  消费 UnitClicked → 更新 SelectionState
  │
  ▼
PostUpdate: ui/projections/selection.rs
  │  读取 SelectionState → 更新 UiStore ViewModel
  │
  ▼
Widget 刷新
```

### 四级通信映射

| 通信 | 机制 | 方向 | 说明 |
|------|------|------|------|
| Pointer → PickIntent | Observer | 模块内部 | `On<Pointer<Click>>` → PickIntent |
| PickIntent → Domain Event | Observer | UI → Core | `On<PickIntent>` → `trigger(UnitClicked)` |
| Domain Event → Selection | Observer | Core 内部 | `On<UnitClicked>` → SelectionState |
| Selection → ViewModel | Observer | Core → UI | `On<SelectionChanged>` → Projection |
| Hover Highlight | Observer | UI 内部 | `On<Pointer<Over>>` → Sprite.color 修改 |
| Camera Follow | trigger(CameraRequest) | UI → Infra | 选中单位时触发镜头跟随 |

### 事件类型清单

```rust
/// 领域事件（在 core/domains/ 中定义）
pub struct UnitClicked {
    pub unit_id: BattleUnitId,
    pub button: PointerButton,
    pub context: PickContext,
}

pub struct TileClicked {
    pub tile: GridPos,
    pub button: PointerButton,
    pub context: PickContext,
}

pub struct SelectionChanged {
    pub previous: Option<BattleUnitId>,
    pub current: Option<BattleUnitId>,
    pub phase: SelectionPhase,
}

/// UI 内部事件（在 ui/picking/ 中消费，不跨层）
pub struct PickIntentReceived(pub PickIntent);  // 内部 trigger
```

## 边界定义

### 允许
- `ui/picking/` 读取 `bevy_picking` 类型用于事件转换
- `ui/picking/` 读取 `core/domains/` 的 ID 类型（BattleUnitId, SkillId, ItemId）
- `ui/picking/` 通过 `trigger(CameraRequest)` 与 Camera 交互
- `ui/selection/` 持有 `SelectionState` Resource（Presentation 层状态）
- `ui/selection/bridge.rs` 触发领域事件（UnitClicked, TileClicked）
- `ui/picking/backend/` 配置 `SpritePickingMode::BoundingBox` 和 `Pickable`
- Screen/Widget 通过 `Projection` 读取 SelectionState 而非直接访问
- 分领域 picking 模块（battle/, world/）各自处理场景特化逻辑

### 禁止
- 🟥 `ui/picking/` 直接修改 Domain Component（必须通过领域事件）
- 🟥 `ui/picking/` 存储 `Entity` 作为跨帧引用（必须伴随 Domain ID 校验）
- 🟥 Selection `selected_unit` 使用 `Entity`（必须使用 `BattleUnitId`）
- 🟥 `core/domains/` import `ui/picking/` 类型（违反依赖方向铁律）
- 🟥 `infra/` 包含任何 Picking 模块（全部迁移到 `ui/picking/`）
- 🟥 Picking backend 包含业务逻辑或规则
- 🟥 全局 picking 神系统（按领域拆分）
- 🟥 使用 `EventWriter/EventReader` 代替 Observer（违反 ADR-054）
- 🟥 Camera 跟随绕过 `CameraRequest` 直接写 TargetPose
- 🟥 `infra/picking/` 保留任何代码（迁移后彻底删除）

## Forbidden（禁止事项）

| # | 禁止行为 | 理由 |
|---|---------|------|
| F1 | Picking 代码放在 `src/infra/picking/` | Picking 是 Presentation Layer 输入适配，不属于纯技术 Infra |
| F2 | `Selection` 使用 `Option<Entity>` | Entity 生命周期不稳定，违反 DDD 身份原则 |
| F3 | On<Pointer<Click>> 直接写 Selection | 违反 Picking→Intent→Domain Event→Selection 分层 |
| F4 | Picking observer 直接修改 Sprite.color 以外的表现层 | Sprite.color 等简单视觉反馈是例外，复杂修改必须通过 ViewModel |
| F5 | Camera 跟随绕过 CameraRequest | 违反宪法 §1.5(9) Camera Event 驱动原则 |
| F6 | Hover 和 Selection 共用同一个 Resource | 三种状态（Hover/Focus/Selection）生命周期和语义完全不同 |
| F7 | 没有 PickContext 的全局点击处理 | 不同模式下同一点击行为不同的需求无法满足 |
| F8 | Preview 和 Commit 在一个 handler 中混合处理 | 导致 Hover 有副作用、Click 被误判的问题 |
| F9 | `infra/picking/` 存在 `selection.rs` | Selection 是业务/表现层状态，不属于 Infra |
| F10 | 为 Entity 硬编码 `to_bits() as u32` | Entity::to_bits() 不是稳定 ID，跨帧/跨存档不可靠 |

## Definition / Instance Design

| 类型 | 层级 | 存储 | 可变性 | 说明 |
|------|------|------|--------|------|
| `PickIntent` | Transient | Event | 瞬时 | 事件总线传递，帧末销毁 |
| `PickTarget` | Transient | Event | 瞬时 | PickIntent 的组成部分 |
| `PickContext` | Runtime | Resource/Component | 场景生命周期 | 由当前游戏模式设置 |
| `SelectionState` | Instance | Resource | 运行时可变 | 选中/未选中/瞄准中 |
| `HoverState` | Instance | Resource | 帧级可变 | 当前悬停目标 |
| `FocusState` | Instance | Resource | 会话级 | 键盘/手柄焦点 |
| `UnitClicked` | Transient | Event (Domain Event) | 瞬时 | 触发 Selection 状态机 |
| `SelectionChanged` | Transient | Event | 瞬时 | 驱动 Projection |
| `Pickable` | Instance | Component | Entity 级 | Bevy 原生，配置哪些 Entity 可被点击 |

## 后果

### 正面
1. **Picking 职责清晰** — 只做命中检测和事件转换，不包含业务规则
2. **Selection 独立演进** — 与 Picking 完全解耦，可独立测试
3. **PickContext 支撑多模式** — Normal/Attack/Skill/Move 模式切换自然
4. **Hover/Focus/Selection 分离** — 各自独立管理生命周期
5. **Domain Event 驱动** — 符合现有架构的四级通信规范
6. **Camera 合规** — 镜头跟随通过 CameraRequest，符合宪法要求
7. **UI Architecture 对齐** — Picking 在 Presentation Layer，符合 L3 定位

### 负面
1. **迁移成本** — 需要重写 currently ~200 行 picking + projection 代码
2. **新增间接层** — 从直接写 Selection 到 4 层管道（Pointer → Intent → Domain Event → Selection）
3. **PickTarget 泛型增加复杂度** — 需要维护 PickTarget 四种变体的匹配逻辑
4. **上下文管理** — PickContext 栈需要 push/pop 管理，增加状态管理复杂度

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 保留在 infra/picking/，只修正 API | 根本问题是归属层错误，修补 API 无法解决。Infra 层不应包含业务语义的类型（PickTarget/PickContext） |
| Picking 放在 app/ 层 | App 是 Composition Root，不含业务逻辑和表现逻辑。Picking 本身属于 Presentation 职责 |
| 不拆分 Hover/Focus/Selection | 用户已明确要求三态分离（原则 #7）。合并会导致一个 observer 中混杂帧级瞬态和会话级状态逻辑 |
| Selection 保留在 Core Domain | Selection 是表现层概念（什么被高亮/显示），不是业务规则。放在 Core 导致 Core 依赖 UI 概念 |
| 使用 EventWriter/EventReader 代替 Observer | 违反 ADR-054 和 Bevy 0.19 规范 |
| 从 pointer events 直接生成领域事件（跳过 PickIntent） | 丢失 PickContext 信息，无法支持多模式。PickIntent 是上下文注入点 |
