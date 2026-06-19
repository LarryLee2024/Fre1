---
id: 06-ui.projection-viewmodel
title: Projection and ViewModel Architecture — 投影层与视图模型架构
status: draft
owner: presentation-architect
created: 2026-06-20
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

每个 Domain 对应一个 Projection 文件：

```
projections/
├── mod.rs              # ProjectionPlugin
├── battle.rs           # BattleProjection（监听 Combat Domain Events）
├── inventory.rs        # InventoryProjection（监听 Inventory Domain Events）
├── character.rs        # CharacterProjection（监听 Attribute/Effect Events）
├── quest.rs            # QuestProjection（监听 Quest Domain Events）
├── economy.rs          # EconomyProjection（监听 Economy Domain Events）
```

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

| 字段 | 类型 | 说明 |
|------|------|------|
| hp | u32 | 当前 HP |
| max_hp | u32 | 最大 HP |
| mp | u32 | 当前 MP |
| max_mp | u32 | 最大 MP |
| current_turn | u32 | 当前回合数 |
| active_character | Option\<CharacterId\> | 当前行动角色 |
| phase | BattlePhaseVm | 战斗阶段 |
| cooldowns | HashMap\<SkillId, f32\> | 技能冷却 |
| action_points | u32 | 行动点 |
| max_action_points | u32 | 最大行动点 |

#### CharacterPanelVm

| 字段 | 类型 | 说明 |
|------|------|------|
| character_id | Option\<CharacterId\> | 当前角色 |
| name_key | UiTextKey | 角色名 |
| level | u32 | 等级 |
| hp / max_hp / mp / max_mp | u32 | 基本属性 |
| exp / exp_to_next | u32 | 经验 |
| buffs | Vec\<BuffVm\> | Buff 列表 |
| stats | StatsVm | 属性面板 |

#### SkillPanelVm

| 字段 | 类型 | 说明 |
|------|------|------|
| skills | Vec\<SkillSlotVm\> | 技能槽位列表 |
| selected | Option\<SkillId\> | 当前选中 |
| ap_remaining | u32 | 剩余行动点 |
| max_ap | u32 | 最大行动点 |

#### InventoryVm

| 字段 | 类型 | 说明 |
|------|------|------|
| items | Vec\<InventorySlotVm\> | 物品列表 |
| gold | u32 | 金币 |
| filter | InventoryFilterVm | 当前筛选 |
| selected | Option\<ItemId\> | 当前选中 |
| sort_order | InventorySortOrder | 排序方式 |

#### ShopVm / QuestLogVm / NotificationVm / ModalVm

参见 schema §4.6-§4.9 完整定义。

（引用：domain rules §7 — ViewModel 定义；schema §4 — Schema Design）

---

## 4. Dirty<T> 脏标记机制

### 4.1 设计目的

Widget 只在数据变化时刷新，避免每帧全量遍历所有 Widget。

### 4.2 机制设计

```rust
#[derive(Component, Reflect, Default)]
pub struct Dirty<T: Reflect + Default> {
    pub inner: T,
    pub is_dirty: bool,
}

impl<T: Reflect + Default> Dirty<T> {
    pub fn mark_dirty(&mut self);
    pub fn consume(&mut self) -> bool;
}
```

**工作流程**：
1. Projection 更新 ViewModel 后调用 `mark_dirty()`
2. Widget 系统调用 `consume()` 检测脏标记
3. 脏则刷新 Widget 渲染，否者跳过
4. `consume()` 自动清除脏标记，保证每帧最多刷新一次

### 4.3 注册要求

每个 ViewModel 类型的 `Dirty<T>` 必须在 UiPlugin 中注册：

```rust
app.register_type::<Dirty<BattleHudVm>>();
app.register_type::<Dirty<CharacterPanelVm>>();
// ... 每个 ViewModel 类型
```

### 4.4 使用约束

- Projection 更新 ViewModel 后必须 `mark_dirty()`
- Widget 只在 `consume() == true` 时执行刷新逻辑
- 加载存档后所有 Dirty 标记重置为 true，触发首次全量刷新
- 禁止手动设置 Dirty 标记（只能通过 Projection）

（引用：schema §9 — Dirty Flag Schema；domain rules §5.2 — Widget 刷新流程）

---

## 5. UiStore 统一容器设计

### 5.1 设计目的

UiStore 是类似 Redux Store 的统一状态容器，所有 ViewModel 集中管理，Projection 更新此容器，Widget 从此容器读取。

### 5.2 结构设计

```rust
#[derive(Resource, Reflect, Default)]
pub struct UiStore {
    pub battle_hud: BattleHudVm,
    pub character_panel: CharacterPanelVm,
    pub skill_panel: SkillPanelVm,
    pub inventory: InventoryVm,
    pub shop: ShopVm,
    pub quest_log: QuestLogVm,
    pub notification_queue: Vec<NotificationVm>,
    pub modal_stack: Vec<ModalVm>,
}
```

### 5.3 设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| Store 类型 | Resource（非 Component） | 全局唯一，无需挂载到 Entity |
| 字段组织 | 平铺（非 HashMap） | 利用 Rust 类型系统保证访问安全 |
| 队列容器 | Vec（非专用队列） | 保持 Reflect 兼容性 |
| 更新方式 | 直接字段赋值（非 Diff） | 简单直接，Dirty 标记额外管理 |

### 5.4 未来扩展点

| 扩展点 | UiStore 新增字段 |
|--------|----------------|
| MiniMapVm | battle_hud → minimap |
| DialogueVm | → dialogue |
| CraftingVm | → crafting |
| TutorialVm | → tutorial |
| AchievementVm | → achievement |

（引用：schema §4.1 — UiStore 统一 ViewModel 容器；schema §19 — Future Extension）

---

## 6. Domain Event → Projection 映射表

| Domain Event | Projection | ViewModel 更新 |
|-------------|-----------|---------------|
| DamageApplied | BattleProjection | BattleHudVm.hp |
| HealthChanged | CharacterProjection | CharacterPanelVm.hp/max_hp |
| ManaChanged | CharacterProjection | CharacterPanelVm.mp/max_mp |
| TurnStarted | BattleProjection | BattleHudVm.current_turn |
| BuffApplied | CharacterProjection | CharacterPanelVm.buffs |
| BuffExpired | CharacterProjection | CharacterPanelVm.buffs |
| AbilityUsed | BattleProjection | SkillPanelVm.cooldowns |
| ItemAcquired | InventoryProjection | InventoryVm.items |
| QuestUpdated | QuestProjection | QuestLogVm.quests |
| LevelUp | CharacterProjection | CharacterPanelVm.level |
| GoldChanged | EconomyProjection | ShopVm.gold |

（引用：domain rules §6 — 领域事件与订阅关系）

---

## 7. Projection 与 Content 数据流

### 7.1 数据流路径

```
Content (assets/config/*.ron)
    ↓ AssetServer 加载
DefRegistry (Resource)
    ↓ Projection 查询
ViewModel (UiStore)
    ↓ Dirty Flag
Widget
```

### 7.2 合法模式：Projection 查询 Def

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

### 7.3 禁止模式：Widget 直接读 Def

```rust
// ❌ Widget 直接读 Def
fn update_skill_icon(defs: Res<DefRegistry<SpellDef>>) { ... }

// ❌ Widget 直接读 Content
fn load_skill_icon(assets: Res<AssetServer>) { ... }
```

### 7.4 Modding 数据流

```
Mod → Content → DefRegistry → Projection → ViewModel → Widget
```

禁止 Mod 直接扩展 UI Widget。Mod 扩展 Content 后，Projection 自动投影到 ViewModel，无需修改 UI 代码。

（引用：ADR-055 §13 — UI 与 Content/Modding 数据流；schema §26 — UI 与 Content 数据流）

---

## 8. UiBinding 反 Marker 模式

为了应对 50 万行代码规模，禁止为每个 UI 元素创建独立 Marker 结构体：

```rust
// ❌ 禁止：400+ Marker 导致 Archetype 爆炸
struct HpText;
struct ManaText;
struct ExpText;

// ✅ 允许：统一枚举
#[derive(Component, Reflect, Clone, Copy, PartialEq)]
pub enum UiBinding {
    Hp, MaxHp, Mp, MaxMp, Ap, MaxAp,
    Turn, Phase, Level, Exp, Name,
    SkillSlot(u8),
    ItemSlot(u8),
    QuestEntry(u16),
    Tooltip, Modal, Notification,
    Cooldown, Gold,
}
```

- 带参数变体（`SkillSlot(u8)`）支持动态数量的 UI 元素
- 单一 Archetype 查询 `Query<&UiBinding>` 即可覆盖所有 UI 绑定
- AI 生成代码时不会困惑于选择哪个 Marker

（引用：ADR-055 §11 — UiBinding 反 Marker 模式；schema §23 — UiBinding Schema）

---

## 9. ViewModel 的 Replay/Save 策略

| 维度 | 策略 |
|------|------|
| Replay | ViewModel 不进入 Replay——从 Domain Event 重新投影 |
| Save | ViewModel 不进入 Save——从 Domain 重新生成 |
| 加载后 | 所有 Dirty 标记设为 true，触发首次全量刷新 |
| 确定性 | Projection 是纯函数，相同 Domain Event → 相同 ViewModel |

（引用：schema §16 — Replay Compatibility；schema §17 — Save Compatibility）

---

## 10. 验证规则

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V-VM-01 | ViewModel 数值范围 | Projection 更新时 | hp ≤ max_hp, mp ≤ max_mp, exp ≤ exp_to_next |
| V-VM-02 | ViewModel 不含 Domain 类型 | 代码审查 | 字段类型不引用 Domain Component |
| V-VM-03 | UiTextKey 格式合法 | 编译期/加载期 | 匹配 `ui\.[a-z0-9_]+.[a-z0-9_]+.[a-z0-9_]+` |
| V-VM-04 | AssetKey 非空 | Def 加载 | 字符串长度 > 0 |
| V-PROJ-01 | Projection 不修改 Domain | 代码审查 | 参数不含 ResMut\<非 UiStore 的 Domain Resource\> |

---

*本文档由 @presentation-architect 维护。新增 ViewModel 或 Projection 需经过架构审查。*
