---
id: 06-ui.projection-viewmodel
title: Projection and ViewModel Architecture — 投影层与视图模型架构
status: code-aligned
owner: presentation-architect
created: 2026-06-20
updated: 2026-06-21
tags:
  - ui
  - projection
  - viewmodel
  - dirty-flag
  - uistore
---

# Projection and ViewModel Architecture — 投影层与视图模型架构

> **职责**: @presentation-architect | **上游**: domain rules §1, §5.1, §7, §INV-UI-001, §INV-UI-004, §INV-UI-008, §INV-UI-009 | ADR-055 §3, §5.1, §13 | schema §1-§4, §9, §11, §23

---

## 1. 设计目的

Projection 和 ViewModel 是 UI 层与 Domain 层之间的防火墙。它们是 L3 层解耦的核心机制：

- **P1**：无统一容器，各 UI 面板直接读取 Domain 数据 → 违反 Presentation/Logic 分离（schema §2 P1）
- **P7**：无 Projection 层定义，Domain → ViewModel 转换逻辑散落 → 投影逻辑不可测试（schema §2 P7）
- **P3**：无 Dirty Flag 机制，每帧全量刷新所有 Widget → 性能浪费（schema §2 P3）

本文档定义 Projection 的纯函数契约、ViewModel 的结构规范、Dirty<T> 脏标记机制、以及 UiStore 统一容器设计。

---

## 2. Projection 架构

### 2.1 Projection 定义

Projection 是 **Domain → ViewModel 的转换函数**，是 UI 层与 Domain 层之间的防火墙。Projection 的输入是 Domain Event 和 Domain 数据，输出是 ViewModel 的更新。

| 职责 | 不负责 |
|------|--------|
| 将 Domain Event 数据映射为 ViewModel 更新 | 修改 Domain 状态 |
| 查询 DefRegistry，将配置投影到 UI 数据 | 包含 Widget 渲染逻辑 |
| 保证投影结果的确定性 | 持有 UI 状态 |

（引用：domain rules §1 — 统一术语，Projection 定义；ADR-055 §5.1 — Projection 防火墙）

### 2.2 Projection 必须是纯函数

Projection 是系统中最关键的纯函数区域：

```
Pure Function Contract:
  Input:  Domain Event + Domain Data（如 Component）
  Output: ViewModel 更新（UiStore 字段变更）
  Side Effects: 无
  Determinism: 相同输入 → 相同输出
```

**为什么必须是纯函数**：
- 可测试：输入固定的 Domain Event → 断言固定的 ViewModel
- 可回放：Replay 时相同 Domain Event → 相同 UI 状态
- 可追踪：状态变更路径清晰，调试友好

**允许的依赖**：
- 读取 Domain Component（Query）
- 读取 DefRegistry（Res）
- 读取当前 ViewModel（用于增量更新）

**禁止的行为**：
- 修改 Domain 状态
- 写入文件/网络
- 使用随机数
- 记录 UI 特有的瞬态状态

（引用：domain rules §5.1 — Projection 更新流程；schema §3.2 — 数据流向关键约束）

### 2.3 Projection 没有 System 依赖

Projection 不依赖 ECS System 执行顺序，通过 Observer 触发：

```rust
// Projection 注册为 Observer
app.add_observer(
    on_damage_applied
        .run_if(screen_is_active::<BattleScreen>)
        .into(|trigger: Trigger<DamageApplied>, mut store: ResMut<UiStore>| {
            // Projection 逻辑在此执行
            store.battle_hud.hp = trigger.new_hp;
            store.battle_hud.mark_dirty();
        })
);
```

（引用：ADR-055 §4.5 — Observer + run_if）

### 2.4 Projection 文件结构

每个 Domain 对应一个 Projection 文件。当前 MVP 仅实现了 BattleProjection：

```
projections/
├── mod.rs              # ProjectionPlugin + 公共导出
├── battle.rs           # BattleProjection（监听 TurnStarted / EffectApplied Domain Events）
```

后续迭代将补充 inventory.rs、character.rs、quest.rs、economy.rs 等文件。

Projection 文件结构模板：
```rust
// src/ui/projections/battle.rs
pub(crate) fn register(plugin: &mut UiProjectionPlugin) {
    // 注册所有 Battle 相关的 Projection Observer
}

// 每个 Projection 是一个纯函数
fn project_damage(
    trigger: Trigger<DamageApplied>,
    mut store: ResMut<UiStore>,
) { ... }

fn project_turn(
    trigger: Trigger<TurnStarted>,
    mut store: ResMut<UiStore>,
) { ... }
```

（引用：ADR-055 §2 — `src/ui/projections/` 目录结构）

### 2.5 CQRS 视角下的 Projection 定位

#### 2.5.1 宪法 §8.9 的 CQRS 原则

宪法 §8.9 规定读写分离原则（CQRS），要求 Domain integration 层显式分离 WriteFacade（命令处理）和 ReadFacade（查询 API）。Projection 在此框架中的定位如下：

```
                   CQRS 架构
    ┌─────────────────────────────────┐
    │      Write Side                 │
    │  UiCommand → GameCommand →      │
    │  Domain Mutation → Event        │
    └─────────────────────────────────┘
                      │
                      ▼ Domain Event
                      │
    ┌─────────────────────────────────┐
    │      Read Side                  │
    │  Domain Read Model              │
    │      ↓（Projection 转换）       │
    │  ViewModel（UI 投影）           │
    │      ↓                          │
    │  Widget 消费                    │
    └─────────────────────────────────┘
```

#### 2.5.2 Projection 是 ReadFacade 的 UI 端实现

宪法 §8.9 明确：**"UI 层的 Projection 模式 = ReadFacade 的 UI 端实现"**。这意味着：

- Projection 并非独立于 Domain 的新概念——它是 CQRS 读模型在 UI 层的自然延伸
- Domain 的 `integration/facade.rs`（如 `CombatFacade::build_effect_view`）是读模型的 Domain 侧出口
- UI 端的 `projections/battle.rs` 是读模型的 UI 侧入口，两者构成完整的 ReadFacade 链路

```
Domain 侧 ReadFacade              UI 侧 ReadFacade（Projection）
┌──────────────────────┐          ┌──────────────────────────┐
│ Integration Facade   │          │ Projection Observer      │
│ （读 API）            │ ──Event──→ （监听 Domain Event）    │
│ build_effect_view()  │          │ project_damage()         │
│ get_character_stats()│          │ project_health_change()  │
└──────────────────────┘          └──────────────────────────┘
         ↓                                  ↓
Domain Read Model                    UI ViewModel（UiStore）
```

**关键规则**：
1. UI Projection 不重复实现 Domain 侧已存在的读逻辑——它只做格式转换（Domain 类型 → UI 友好的 ViewModel）
2. 复杂查询（涉及多领域关联）应在 Domain 侧的 ReadFacade 中预计算，UI Projection 直接消费计算结果
3. ViewModel 的定义应当与 Domain Read Model 的形状对齐——ViewModel 是 Domain Read Model 的 UI 投影，而不是一个独立的数据层

#### 2.5.3 ViewModel 是 Domain Read Model 的 UI 投影

ViewModel **不是**独立的数据架构——它是 Domain Read Model 经过 UI 适配后的表层：

```
Domain Read Model（Aggregate View）       UI ViewModel
────────────────────────                 ────────────
CharacterStats {                           CharacterPanelVm {
    hp: u32,                                   hp: u32,
    max_hp: u32,                               max_hp: u32,
    level: u32,                                level: u32,
    buffs: Vec<BuffInstance>,  ──投影──→       name_key: UiTextKey,   ← 文本字段本地化
}                                              buffs: Vec<BuffVm>,   ← 类型扁平化
                                        }
```

**投影规则**：
- 数值字段（hp, max_hp, level）直接映射，保持类型一致
- 业务类型（`BuffInstance`）扁平化为 UI 友好的 `BuffVm`
- 文本字段（name, description）替换为 `UiTextKey`（本地化在 UI 侧处理）
- Domain 不需要为 UI 创建专门的读模型——View 结构体复用已有的 Aggregate View 即可

#### 2.5.4 UI 写操作：禁止调用 WriteFacade

UI 层有一条绝对禁令：**UI 代码禁止调用 Domain 的 WriteFacade 方法**。

```
// ❌ 绝对禁止
fn on_confirm_purchase(
    mut economy_write: ResMut<EconomyWriteFacade>,  // UI 不可以持有 WriteFacade
) { ... }

// ✅ 唯一合法路径
UiAction::Click
    → UiCommand::BuyItem(item_id, quantity)
    → GameCommand::Economy(BuyItem { ... })
    → EconomyWriteFacade::process_buy()       // WriteFacade 只在 Domain 侧调用
    → Domain Event（PurchaseProcessed）
    → UI Observer（Projection 监听事件，更新 ViewModel）
```

**理由**：
1. 违反宪法 §8.9 的写路径收口要求——所有状态修改必须通过命令与执行系统
2. 绕过 Command 系统直接调用 WriteFacade 会破坏 Replay 确定性
3. WriteFacade 的方法签名可能涉及 Domain 内部类型（`&mut World`），UI 层不应感知

**唯一写入口**：UI 写操作的唯一合法出口是 `UiCommand → GameCommand` 转换器（定义于 `application-layer.md §4.3`）。所有 Domain 写操作必须经过 ADR-043 的 CommandQueue。

#### 2.5.5 现有参考实现

| Domain | ReadFacade（Domain 侧） | Projection（UI 侧） |
|--------|------------------------|---------------------|
| Combat | `combat/integration/facade.rs` → `build_effect_view()` | `projections/battle.rs` (MVP ✅) |
| Economy | `economy/integration/facade.rs` → `get_wallet()` | ❌ 待实现 |
| Spell | `spell/integration/query.rs` → `SpellQueryParam` | ❌ 待实现 |
| Campaign | `party/integration/facade.rs` → query methods | ❌ 待实现 |

（引用：宪法 §8.9 — 读写分离原则；`src/core/domains/*/integration/facade.rs` — ReadFacade 参考实现）

---

## 3. ViewModel 架构

### 3.1 ViewModel 定义

ViewModel 是 **UI 状态的投影**，Domain 数据的 UI 视图。ViewModel 只包含 UI 展示需要的数据，不含业务逻辑。

| 职责 | 不负责 |
|------|--------|
| 承载 UI 展示所需的结构化数据 | 业务逻辑计算 |
| 作为 Widget 的唯一数据源 | 修改 Domain 状态 |
| 保持与 Domain 的松耦合 | 持有 Domain 类型 |

（引用：domain rules §1 — 统一术语，ViewModel 定义）

### 3.2 ViewModel 命名规范

```
格式：{领域}Vm
示例：BattleHudVm, CharacterPanelVm, SkillPanelVm, InventoryVm, ShopVm, QuestLogVm
```

- ViewModel 统一使用 `Vm` 后缀
- 枚举类型使用 `Vm` 后缀（`BattlePhaseVm`、`InventoryFilterVm`）
- 嵌套的子 ViewModel 使用有意义的名称（`SkillSlotVm`、`BuffVm`、`StatsVm`）

### 3.3 ViewModel 设计规范

**规范 V-VM-01：ViewModel 不包含 Domain 类型**

```rust
// ❌ 禁止
struct CharacterPanelVm {
    health: Health,          // Domain Component
    ability: Ability,        // Domain Component
}

// ✅ 允许
struct CharacterPanelVm {
    hp: u32,
    max_hp: u32,
    level: u32,
    name_key: UiTextKey,
}
```

**规范 V-VM-02：用户可见文本使用 UiTextKey**

```rust
// ❌ 禁止
struct SkillSlotVm {
    name: String,            // 硬编码或翻译后的文本
}

// ✅ 允许
struct SkillSlotVm {
    name_key: UiTextKey,     // 本地化 Key
}
```

**规范 V-VM-03：引用 Domain 实体使用 ID，不嵌入定义**

```rust
// ✅ 允许
struct SkillSlotVm {
    skill_id: SkillId,       // 强类型 ID
    icon_key: AssetKey,      // 资源引用 Key
}
```

（引用：domain rules §INV-UI-001 — UI 不直接读取 Domain 组件；schema §V11 — ViewModel 不直接引用 Domain Component；schema §13 — ID 策略）

### 3.4 ViewModel 完整清单

#### BattleHudVm

```rust
#[derive(Clone, Reflect, Default)]
pub struct BattleHudVm {
    pub hp: f32,              // 当前 HP
    pub max_hp: f32,          // 最大 HP
    pub mp: f32,              // 当前 MP
    pub max_mp: f32,          // 最大 MP
    pub ap: f32,              // 当前 AP
    pub max_ap: f32,          // 最大 AP
    pub turn_number: u32,     // 当前回合数
    pub phase_key: &'static str,  // 阶段描述（本地化 Key）
}
```

#### CharacterPanelVm

```rust
#[derive(Clone, Reflect, Default)]
pub struct CharacterPanelVm {
    pub character_id: u32,        // 角色 ID
    pub name_key: &'static str,   // 角色名称（本地化 Key）
    pub level: u32,               // 等级
    pub hp: f32,                  // 当前 HP
    pub max_hp: f32,              // 最大 HP
    pub mp: f32,                  // 当前 MP
    pub max_mp: f32,              // 最大 MP
}
```

#### SkillPanelVm 和 SkillSlotVm

```rust
#[derive(Clone, Reflect, Default)]
pub struct SkillSlotVm {
    pub skill_id: u32,                // 技能 ID
    pub name_key: &'static str,       // 技能名称（本地化 Key）
    pub cooldown_remaining: u32,      // 剩余冷却回合数
    pub max_cooldown: u32,            // 最大冷却回合数
    pub is_usable: bool,              // 是否可用
    pub ap_cost: u32,                 // AP 消耗
}

#[derive(Clone, Reflect, Default)]
pub struct SkillPanelVm {
    pub skills: HashMap<u32, SkillSlotVm>,  // 技能数据映射（skill_id → SkillSlotVm）
}
```

（引用：domain rules §7 — ViewModel 定义；schema §4 — Schema Design）

---

## 4. 复合级 ViewModel

复合组件（Molecules / Organisms，定义于 widget-composites.md）消费的 ViewModel 分为两类：

1. **独立定义**：复合组件独有的 ViewModel，不对应 UiStore 中的独立字段（作为父级 ViewModel 的嵌套字段存在）
2. **派生自基础 VM**：复合组件直接使用 UiStore 中已有的基础 ViewModel，不额外命名（引用：widget-composites.md §1.2 — 复合组件与 ViewModel 的对应关系）

### 4.1 Molecule 级 ViewModel

当前 MVP 实现了 `SkillSlotVm` 作为独立定义的 Molecule 级 ViewModel。其余 Molecule ViewModel（CharacterPortraitVm、InventoryItemRowVm、QuestEntryVm、DialogueChoiceVm、ShopItemCardVm、BuffVm、TurnIndicatorVm）将在后续迭代中补充。

#### SkillSlotVm

```rust
#[derive(Clone, Reflect, Default)]
pub struct SkillSlotVm {
    pub skill_id: u32,
    pub name_key: &'static str,
    pub cooldown_remaining: u32,
    pub max_cooldown: u32,
    pub is_usable: bool,
    pub ap_cost: u32,
}
```

- **来源**: widget-composites.md §2.1 SkillSlot
- **定义方式**: 独立定义
- **使用场景**: 作为 SkillPanelVm 的 skills 字段值（`HashMap<u32, SkillSlotVm>`）

### 4.2 Organism 级 ViewModel

当前 MVP 实现的 Organism 级 ViewModel 均为 §3.4 中定义的基础 ViewModel（BattleHudVm、CharacterPanelVm、SkillPanelVm），UiStore 直接作为 Organism 的输入源。暂无非基础 ViewModel 的独立 Organism ViewModel。

后续迭代将补充：
- CharacterStatusPanelVm（独立定义）
- TurnOrderBarVm（独立定义）
- InventoryVm（等同于 InventoryGrid 输入源）
- QuestLogVm
- DialoguePanelVm
- ShopPanelVm

---

## 5. Dirty<T> 脏标记机制

### 5.1 设计目的

Widget 只在数据变化时刷新，避免每帧全量遍历所有 Widget。

### 5.2 机制设计

```rust
#[derive(Component, Debug, Clone, Reflect)]
pub struct Dirty<T: Reflect + Default + Clone + Send + Sync + 'static> {
    pub inner: T,
    is_dirty: bool,
}

impl<T: Reflect + Default + Clone + Send + Sync + 'static> Dirty<T> {
    /// 创建新的 Dirty 包装（初始状态为 dirty，触发首次刷新）
    pub fn new(inner: T) -> Self;

    /// 手动标记为脏
    pub fn mark_dirty(&mut self);

    /// 消费脏标记 —— 返回 true 表示需要刷新
    /// 消费后标记自动清除，避免同帧重复消费
    pub fn consume(&mut self) -> bool;

    /// 获取内部数据引用（不触发 dirty）
    pub fn get(&self) -> &T;

    /// 获取内部数据可变引用（自动标记 dirty）
    pub fn get_mut(&mut self) -> &mut T;
}
```

**工作流程**：
1. Projection 更新 ViewModel 后调用 `get_mut()`（自动 mark_dirty）或 `mark_dirty()`
2. Widget 系统调用 `consume()` 检测脏标记
3. 脏则刷新 Widget 渲染，否则跳过
4. `consume()` 自动清除脏标记，保证每帧最多刷新一次
5. `get()` 只读访问，不触发脏标记
6. `get_mut()` 可变访问时自动标记脏

### 5.3 注册要求

每个 ViewModel 类型的 `Dirty<T>` 需要在 App 构建时注册类型：

```rust
app.register_type::<BattleHudVm>();
app.register_type::<CharacterPanelVm>();
app.register_type::<SkillPanelVm>();
app.register_type::<Dirty<BattleHudVm>>();
app.register_type::<Dirty<CharacterPanelVm>>();
app.register_type::<Dirty<SkillPanelVm>>();
```

### 5.4 使用约束

- Projection 更新 ViewModel 后必须 `mark_dirty()`
- Widget 只在 `consume() == true` 时执行刷新逻辑
- 加载存档后所有 Dirty 标记重置为 true，触发首次全量刷新
- 禁止手动设置 Dirty 标记（只能通过 Projection）

（引用：schema §9 — Dirty Flag Schema；domain rules §5.2 — Widget 刷新流程）

---

## 6. UiStore 统一容器设计

### 6.1 设计目的

UiStore 是类似 Redux Store 的统一状态容器，所有 ViewModel 集中管理，Projection 更新此容器，Widget 从此容器读取。

### 6.2 结构设计

```rust
#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct UiStore {
    pub battle_hud: BattleHudVm,
    pub character_panel: CharacterPanelVm,
    pub skill_panel: SkillPanelVm,
}
```

### 6.3 设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| Store 类型 | Resource（非 Component） | 全局唯一，无需挂载到 Entity |
| 字段组织 | 平铺结构体（非 HashMap） | 利用 Rust 类型系统保证访问安全 |
| 更新方式 | 直接字段赋值 + Dirty<T> | 通过 get_mut() 自动标记脏 |

### 6.4 未来扩展点

当前 UiStore 仅包含战斗和角色相关的三个字段。后续迭代将新增：

| 扩展点 | UiStore 新增字段 |
|--------|----------------|
| InventoryVm | → inventory |
| ShopPanelVm | → shop |
| QuestLogVm | → quest_log |
| MiniMapVm | → minimap |
| DialogueVm | → dialogue |
| CraftingVm | → crafting |
| NotificationVm 队列 | → notification_queue |

（引用：schema §4.1 — UiStore 统一 ViewModel 容器；schema §19 — Future Extension）

---

## 7. Domain Event → Projection 映射表

| Domain Event | Projection | ViewModel 更新 | 实现状态 |
|-------------|-----------|---------------|---------|
| TurnStarted | BattleProjection | BattleHudVm.turn_number += 1 / phase_key 设置 | ✅ |
| TurnStarted | BattleProjection | SkillPanelVm cooldown tick（每回合冷却-1） | ✅ |
| TurnStarted | CharacterPanelProjection | CharacterPanelVm 从活动单位填充 | ✅ |
| TurnEnded | BattleProjection | BattleHudVm.phase_key = "enemy" | ✅ |
| EffectApplied | BattleProjection | SkillPanelVm 冷却匹配 + cooldown_remaining 设置 | ✅ |
| BattleStarted | BattleProjection | BattleHudVm 初始化（turn_number=1, phase_key） | ✅ |

**当前实现说明**：
- BattleProjection 是唯一已实现的 projection，包含以下纯函数：
  - `on_battle_started` — 初始化 HUD（turn_number=1, phase_key）
  - `on_turn_started` — 递增回合计数、设置阶段键
  - `on_turn_ended` — 阶段键切换到敌方
  - `on_effect_applied` — 技能冷却匹配与更新
  - `on_turn_started_for_skills` — 每回合递减技能冷却
- `on_turn_started`：递增 `turn_number`，设置 `phase_key` 为 `"ui.battle.phase.player"`
- `on_effect_applied`：当前为 no-op，仅日志记录。标记了 TODO 以便后续实现技能冷却更新
- 后续迭代将补充 DamageApplied、HealthChanged、ItemAcquired 等事件的 projection

（引用：domain rules §6 — 领域事件与订阅关系）

---

## 8. Projection 与 Content 数据流

### 8.1 数据流路径

```
Content (assets/config/*.ron)
    ↓ AssetServer 加载
DefRegistry (Resource)
    ↓ Projection 查询
ViewModel (UiStore)
    ↓ Dirty Flag
Widget
```

### 8.2 合法模式：Projection 查询 Def

```rust
// 允许：Projection 读取 DefRegistry，写入 UiStore
fn project_skill_info(
    trigger: Trigger<AbilityUsed>,
    defs: Res<DefRegistry<SpellDef>>,   // Projection 可以读 Def
    mut store: ResMut<UiStore>,          // 写入 ViewModel
) {
    if let Some(def) = defs.get(trigger.ability_id) {
        store.skill_panel.update_from_def(def);
        store.skill_panel.mark_dirty();
    }
}
```

### 8.3 禁止模式：Widget 直接读 Def

```rust
// ❌ Widget 直接读 Def
fn update_skill_icon(defs: Res<DefRegistry<SpellDef>>) { ... }

// ❌ Widget 直接读 Content
fn load_skill_icon(assets: Res<AssetServer>) { ... }
```

### 8.4 Modding 数据流

```
Mod → Content → DefRegistry → Projection → ViewModel → Widget
```

禁止 Mod 直接扩展 UI Widget。Mod 扩展 Content 后，Projection 自动投影到 ViewModel，无需修改 UI 代码。

（引用：ADR-055 §13 — UI 与 Content/Modding 数据流；schema §26 — UI 与 Content 数据流）

---

## 9. UiBinding 反 Marker 模式

为了应对 50 万行代码规模，禁止为每个 UI 元素创建独立 Marker 结构体：

```rust
// ❌ 禁止：400+ Marker 导致 Archetype 爆炸
struct HpText;
struct ManaText;
struct ExpText;

// ✅ 允许：统一枚举
#[derive(Component, Reflect, Clone, PartialEq, Eq, Hash)]
#[reflect(Component)]
pub enum UiBinding {
    // ── Battle HUD ──
    Hp, MaxHp, Mp, MaxMp, Ap, MaxAp,
    Turn, Phase,

    // ── Character Panel ──
    Level, Exp, Name, CharacterLevel,

    // ── Skill Panel ──
    SkillSlot(u8), Cooldown,

    // ── Inventory ──
    ItemSlot(u8), Gold,

    // ── Quest ──
    QuestEntry(u16),

    // ── General ──
    Tooltip, Modal, Notification,
    Text, Icon,
}
```

- 当前 MVP 实现了 Battle HUD、Character Panel、Skill Panel 和 General 分类的变体
- Inventory、Quest 分类的变体已定义预留，等待对应 Widget 实现后启用
- 带参数变体（`SkillSlot(u8)`）支持动态数量的 UI 元素
- 单一 Archetype 查询 `Query<&UiBinding>` 即可覆盖所有 UI 绑定
- AI 生成代码时不会困惑于选择哪个 Marker

（引用：ADR-055 §11 — UiBinding 反 Marker 模式；schema §23 — UiBinding Schema）

---

## 10. ViewModel 的 Replay/Save 策略

| 维度 | 策略 |
|------|------|
| Replay | ViewModel 不进入 Replay——从 Domain Event 重新投影 |
| Save | ViewModel 不进入 Save——从 Domain 重新生成 |
| 加载后 | 所有 Dirty 标记设为 true，触发首次全量刷新 |
| 确定性 | Projection 是纯函数，相同 Domain Event → 相同 ViewModel |

（引用：schema §16 — Replay Compatibility；schema §17 — Save Compatibility）

---

## 11. 验证规则

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V-VM-01 | ViewModel 数值范围 | Projection 更新时 | hp ≤ max_hp, mp ≤ max_mp |
| V-VM-02 | ViewModel 不含 Domain 类型 | 代码审查 | 字段类型不引用 Domain Component |
| V-VM-03 | 文本字段使用本地化 Key | 代码审查 | 文本字段使用 `&'static str` 而非硬编码字符串 |
| V-PROJ-01 | Projection 不修改 Domain | 代码审查 | 参数不含 `ResMut`\<非 UiStore 的 Domain Resource\> |

---

*本文档由 @presentation-architect 维护。新增 ViewModel 或 Projection 需经过架构审查。*

*最后更新: 2026-06-21 — 与实际代码实现对齐 (commit 903d039)*
