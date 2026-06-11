# 角色领域规则 (Character Rules)

## 1. 领域概述

角色是 SRPG 的核心实体，代表战场上的一切战斗单位（玩家角色与敌方单位）。角色系统负责单位的**定义、生成、身份标识、阵营划分、位置管理、移动动画**和**Trait扩展**。

### 核心原则

- **Definition / Instance 分离**：`UnitTemplate`（配置）独立于 `Unit`（运行时状态）
- **Rule / Content 分离**：代码负责规则，RON 配置负责内容
- **组合优于继承**：角色由 Trait + Modifier 组合构成，而非继承树
- **Entity 只是 ID**：不把 Entity 当对象使用

---

## 2. 核心组件

### 2.1 Unit — 战斗单位

```rust
#[derive(Component, Reflect)]
#[require(
    Attributes, SkillSlots, SkillCooldowns, ActiveBuffs,
    GameplayTags, PersistentTags, TraitCollection,
    EquipmentSlots, Container, GridPosition
)]
pub struct Unit {
    pub faction: Faction,
    pub acted: bool,
}
```

**规则**：
- `Unit` 是角色的核心身份组件，生成时自动插入 9 个 Required Components
- `faction` 决定阵营归属（Player / Enemy），影响技能目标选择和 AI 行为
- `acted` 标记本回合是否已行动，回合开始时重置为 `false`

### 2.2 身份组件

| 组件 | 用途 | 示例 |
|------|------|------|
| `UnitId(String)` | 业务逻辑标识 | `"knight_001"` |
| `UnitName(String)` | UI 显示名称 | `"战士"` |
| `UnitRace(String)` | 种族标识 | `"人类"` |
| `UnitClass(String)` | 职业标识 | `"战士"` |
| `AiBehaviorId(String)` | AI 行为配置 ID | `"aggressive"` |

**规则**：
- `UnitId` 用于业务逻辑查找，`UnitName` 用于 UI 展示，`Name`（Bevy 内置）用于 Inspector 调试
- `UnitId` 实现 `PartialEq + Hash`，可作为 HashMap key
- `AiBehaviorId` 仅敌方单位使用，玩家单位默认 `"default"`

### 2.3 GridPosition — 格子坐标

```rust
#[derive(Component, Reflect)]
pub struct GridPosition {
    pub coord: IVec2,
}
```

**规则**：
- 默认值为 `IVec2::ZERO`
- 移动动画完成后由 `animate_movement` 系统更新
- 坐标用于寻路、攻击范围计算、地形查询

### 2.4 Faction — 阵营

```rust
pub enum Faction {
    Player,
    Enemy,
}
```

**规则**：
- 阵营决定敌我关系，影响技能目标筛选（`SingleEnemy` / `SingleAlly`）
- 阵营颜色由 `faction_color()` 函数映射，用于棋子渲染

### 2.5 Selected — 选中标记

Tag Component，表示当前被玩家选中的单位。

### 2.6 Dead — 死亡标记

```rust
#[component(on_add = Dead::on_add_dead)]
pub struct Dead;
```

**Hook 行为**（添加 Dead 时自动触发）：
1. 将 `Unit.acted` 设为 `true`，防止死亡单位继续行动
2. 移除 `Selected` 组件

**规则**：
- Dead 是 Tag Component，优于 `is_dead: bool`
- 死亡判定在效果执行管线中触发（HP ≤ 0 时插入 Dead）
- Hook 保证死亡状态的固有行为，无需外部系统手动处理

### 2.7 PersistentTags — 持久化标签

```rust
pub struct PersistentTags {
    pub from_traits: GameplayTags,      // Trait 授予（种族/职业/天赋）
    pub from_equipment: GameplayTags,   // 装备授予（穿脱变化）
}
```

**规则**：
- 标签分两层追踪来源，支持精确增减
- 装备穿脱时只修改 `from_equipment`，不影响 `from_traits`
- `GameplayTags` 最终值 = `from_traits | from_equipment`

### 2.8 MovingUnit — 移动动画

```rust
pub struct MovingUnit {
    pub path: Vec<IVec2>,        // 路径坐标序列
    pub current_index: usize,    // 当前目标路径索引
    pub speed: f32,              // 每格耗时（秒）
    pub elapsed: f32,            // 当前格已用时间
    pub next_phase: TurnPhase,   // 移动完成后的回调阶段
}
```

**规则**：
- 移动动画期间挂在单位上，逐格线性插值
- `is_finished()` 判断是否到达终点（`current_index >= path.len()`）
- 动画完成后移除 `MovingUnit`，更新 `GridPosition`，切换 `TurnPhase`
- 同时清除所有 `PathArrow` 导航箭头

---

## 3. Definition / Instance 分离

### 3.1 UnitTemplate（定义 / 配置）

```rust
pub struct UnitTemplate {
    pub id: String,
    pub name: String,
    pub faction: Faction,
    pub race: String,
    pub background: String,
    pub class: String,
    pub base_attributes: HashMap<AttributeKind, f32>,  // 仅8维核心属性
    pub base_attack_range: u32,
    pub skill_ids: Vec<String>,
    pub trait_ids: Vec<String>,
    pub ai_behavior: String,
    pub initial_equipment: Vec<(EquipmentSlot, String)>,
}
```

**规则**：
- `base_attributes` 仅包含 8 维核心属性（Might/Dexterity/Agility/Vitality/Intelligence/Willpower/Presence/Luck）
- 生命值/魔法值等 Vital Resources 由 `attributes.fill_vital_resources()` 从核心属性推导
- `initial_equipment` 指定初始装备槽位映射，生成时自动穿戴
- 模板数据不可变，运行时状态存储在 `Unit` 及其 Required Components 中

### 3.2 UnitTemplateDef（RON 反序列化用）

```rust
pub struct UnitTemplateDef {
    #[serde(default)]
    pub version: u32,           // 版本号，默认 0
    // ... 其余字段同 UnitTemplate，faction 使用 FactionDef
}
```

**规则**：
- `version` 字段使用 `#[serde(default)]`，旧配置无需 version 字段
- `FactionDef` 使用 PascalCase（`Player` / `Enemy`），转换为运行时 `Faction`
- `initial_equipment` 使用 `#[serde(default)]`，旧配置无此字段时默认为空

### 3.3 UnitTemplateRegistry — 模板注册表

```rust
#[derive(Resource, Default)]
pub struct UnitTemplateRegistry {
    pub templates: HashMap<String, UnitTemplate>,
}
```

**规则**：
- 启动时从 `assets/units/*.ron` 加载，无配置文件时使用内置默认模板
- 内置默认模板：`player_warrior`、`player_archer`、`enemy_goblin`、`enemy_dark_knight`
- 通过 `RegistryLoader` trait 统一加载流程

---

## 4. 单位生成流程

### 4.1 生成入口

```
OnEnter(AppState::InGame) → spawn_units → spawn_unit_from_template
```

### 4.2 生成步骤

1. **从 LevelConfig 获取部署列表**（`player_units` + `enemy_units`）
2. **查找模板**：`template_registry.get(deploy.template)`
3. **构建 Attributes**：从模板设置核心属性基础值，调用 `fill_vital_resources()`
4. **构建 TraitCollection**：从模板 `trait_ids` 创建
5. **应用被动 Trait**：`apply_passive_traits()` 收集标签和属性修饰符
6. **穿戴初始装备**：遍历 `initial_equipment`，跳过需求检查直接装备
7. **重建 Trait 效果**：装备可能添加新 Trait，调用 `rebuild_trait_effects()`
8. **重建 GameplayTags**：`from_traits | from_equipment`
9. **Spawn Entity**：插入所有组件 + 子实体（名称标注、行动顺序标签、HP条）

### 4.3 子实体结构

```
Unit Entity
├── Text2d（棋子首字标注）
├── Text2d + TurnOrderLabel（行动顺序数字）
├── Sprite + HpBarBg（HP条背景，红色）
└── Sprite + HpBarFg（HP条前景，绿色）
```

**规则**：
- 子实体使用 `Pickable::IGNORE`，不拦截鼠标事件
- HP条使用 `Anchor::CENTER_LEFT`，从左向右填充
- 行动顺序标签由 `update_turn_order_label` 系统动态更新

---

## 5. Trait 扩展体系

### 5.1 核心概念

Trait 是角色能力的统一抽象，所有能力来源（种族、职业、天赋、装备）均通过 Trait 表达：

- **种族 = Trait + Modifier 集合**
- **职业 = 成长率 + 技能池 + Trait 集合**
- **天赋 = 特殊 Trait**
- **装备 = Modifier + Trait**

### 5.2 TraitTrigger — 触发时机

| 触发器 | 说明 | 应用场景 |
|--------|------|----------|
| `Passive` | 始终生效 | 授予标签、属性修饰 |
| `OnTurnStart` | 回合开始时 | 回合开始增益 |
| `OnTurnEnd` | 回合结束时 | 回合结束效果 |
| `OnAttack` | 攻击时 | 攻击触发 Buff |
| `OnHit` | 被攻击时 | 受击反击 |
| `OnKill` | 击杀时 | 击杀奖励 |

### 5.3 TraitEffect — 效果类型

| 效果 | 说明 | Handler |
|------|------|---------|
| `GrantTag(GameplayTag)` | 授予标签 | `GrantTagHandler` |
| `ModifyAttribute(AttributeModifierDef)` | 属性修饰 | `ModifyAttributeHandler` |
| `ApplyBuff { buff_id, duration }` | 触发时施加 Buff | `ApplyBuffHandler` |

### 5.4 TraitEffectHandler — 效果处理器

```rust
pub trait TraitEffectHandler: Send + Sync + 'static {
    fn type_name(&self) -> &'static str;
    fn granted_tags(&self, effect: &TraitEffect) -> Vec<GameplayTag>;
    fn attribute_modifiers<'a>(&self, effect: &'a TraitEffect) -> Vec<&'a AttributeModifierDef>;
}
```

**规则**：
- 新增效果类型只需实现 `TraitEffectHandler` 并注册到 `TraitEffectHandlerRegistry`
- 通过 `type_name()` 分发，无需修改 `TraitData` 的方法
- 内置三个处理器：`GrantTagHandler`、`ModifyAttributeHandler`、`ApplyBuffHandler`

### 5.5 TraitSource — 来源追踪

```rust
pub enum TraitSource {
    Intrinsic,                    // 内在来源（种族/职业/天赋）
    Equipment { slot: EquipmentSlot },  // 装备来源（记录具体槽位）
}
```

**规则**：
- `TraitEntry` 记录 `trait_id + source`，支持按来源精确移除
- 装备穿脱时调用 `remove_by_source(&TraitSource::Equipment { slot })` 清理
- 内在 Trait 在单位生成时标记为 `Intrinsic`，不会被装备操作误删

### 5.6 TraitCollection — 单位 Trait 集合

```rust
pub struct TraitCollection {
    pub entries: Vec<TraitEntry>,
}
```

**操作**：
- `new(trait_ids)` — 从 ID 列表创建，全部标记为 Intrinsic
- `has(trait_id)` — 查询是否拥有指定 Trait
- `add_entry(trait_id, source)` — 添加一条 TraitEntry
- `remove_by_source(source)` — 按来源移除，返回被移除的 trait_id 列表
- `trait_ids()` — 获取所有 trait_id（去重）

### 5.7 apply_passive_traits — 被动效果应用

```rust
pub fn apply_passive_traits(
    trait_collection: &TraitCollection,
    registry: &TraitRegistry,
    handlers: &TraitEffectHandlerRegistry,
) -> (GameplayTags, Vec<AttributeModifierInstance>)
```

**规则**：
- 仅处理 `TraitTrigger::Passive` 的 Trait，跳过其他触发类型
- 每个 Trait 分配独立的 `ModifierSource::trait_source(index)`，从 0 递增
- Trait 区间：`u64::MAX ~ u64::MAX - 999`，避免与 Buff/Equipment 区间冲突
- 返回的标签和修饰符由调用方合并到 `GameplayTags` 和 `Attributes`

---

## 6. 移动系统

### 6.1 导航路径可视化

- `spawn_path_arrows()` — 生成路径线段 + 末端箭头
- `despawn_path_arrows()` — 清除所有 `PathArrow` 实体
- 线段使用 Sprite 渲染，颜色为半透明黄色
- 箭头使用菱形压扁成箭头形状

### 6.2 移动动画

`animate_movement` 系统每帧执行：

1. 累加 `elapsed` 时间
2. 计算 `t = elapsed / speed`，线性插值 Transform
3. 到达当前格时更新 `GridPosition`，前进 `current_index`
4. 全部走完后：移除 `MovingUnit`，更新最终 `GridPosition`，切换 `TurnPhase`，清除导航箭头

### 6.3 标记组件

| 标记 | 用途 |
|------|------|
| `MovableRange` | 可移动范围标记 |
| `AttackRange` | 可攻击范围标记 |
| `SelectionHighlight` | 选中高亮（独立实体） |
| `PathArrow` | 导航路径箭头 |

**规则**：
- 标记是表现层组件，由逻辑层添加、表现层读取
- `clear_markers()` 清除所有范围标记和高亮（不含 `Selected` 移除）

---

## 7. 行动顺序显示

`update_turn_order_label` 系统在每帧更新：

- 从 `TurnOrder` 队列构建实体→位置映射
- 未行动单位显示序号（从 1 开始）
- 已行动单位不显示编号

---

## 8. 数据驱动配置

### 8.1 RON 文件路径

- 单位模板：`assets/units/*.ron`
- Trait 定义：`assets/traits/*.ron`

### 8.2 单位模板 RON 示例

```ron
(
    id: "player_warrior",
    name: "战士",
    faction: Player,
    race: "人类",
    background: "士兵",
    class: "战士",
    base_attributes: {
        Might: 5.0, Dexterity: 3.0, Agility: 6.0,
        Vitality: 5.0, Intelligence: 2.0, Willpower: 3.0,
        Presence: 2.0, Luck: 2.0,
    },
    base_attack_range: 1,
    skill_ids: ["basic_attack", "charge"],
    trait_ids: ["warrior_mastery"],
    ai_behavior: "default",
    initial_equipment: [
        (MainHand, "iron_sword"),
        (Body, "leather_armor"),
    ],
)
```

### 8.3 Trait 定义 RON 示例

```ron
(
    id: "warrior_mastery",
    name: "战士精通",
    description: "近战职业，擅长正面作战",
    trigger: Passive,
    effects: [
        GrantTag(WARRIOR),
        GrantTag(MELEE),
    ],
)
```

---

## 9. 插件注册

`CharacterPlugin` 组合注册以下子插件和系统：

| 子插件 | 职责 |
|--------|------|
| `UnitTemplatePlugin` | 加载单位模板注册表 |
| `TraitPlugin` | 加载 Trait 注册表 + 效果处理器注册表 |
| `UnitPlugin` | 注册 `spawn_units` 系统 |

| 系统 | 运行条件 | 职责 |
|------|----------|------|
| `spawn_units` | `OnEnter(AppState::InGame)` | 生成初始单位 |
| `animate_movement` | `Update + InGame` | 移动动画插值 |
| `update_turn_order_label` | `Update + InGame` | 更新行动顺序标签 |

---

## 10. 关键约束

1. **Required Components 保证完整性**：生成 `Unit` 时自动插入 9 个依赖组件，防止遗漏
2. **Dead Hook 保证死亡行为**：添加 Dead 时自动标记已行动 + 移除选中，无需外部手动处理
3. **Trait 来源追踪**：`TraitSource` 区分内在/装备来源，支持精确增减
4. **Modifier Source 区间隔离**：Trait/Buff/Equipment 各有独立 source id 区间，避免冲突
5. **配置兼容性**：`version` 和 `initial_equipment` 字段使用 `#[serde(default)]`，旧配置无需修改
6. **子实体不拦截鼠标**：所有子实体使用 `Pickable::IGNORE`，鼠标事件穿透到父级 Unit
