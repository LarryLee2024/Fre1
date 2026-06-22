---
id: 01-architecture.ADR-032
title: ADR-032 — Economy & Crafting Architecture
status: approved
owner: architect
created: 2026-06-16
updated: 2026-06-16
supersedes: none
---

# ADR-032: 经济与制造系统架构

## 状态

**Approved** — 依赖 ADR-030（Progression/Inventory）和 ADR-011（Modifier Pipeline），本架构决策正式生效。

## 背景

经济系统（货币、商店、交易）和制造系统（配方、材料、合成）构成游戏的资源循环。玩家通过战斗获得货币和材料，在商店购买装备或在工坊制造物品，从而强化角色。

## 引用的领域规则与数据架构

- `docs/02-domain/domains/economy_domain.md` — Economy 领域规则
- `docs/02-domain/domains/crafting_domain.md` — Crafting 领域规则
- `docs/02-domain/domains/summon_domain.md` — Summon 领域规则
- `docs/04-data/domains/economy_schema.md` — Economy Schema
- `docs/04-data/domains/crafting_schema.md` — Crafting Schema
- `docs/04-data/domains/summon_schema.md` — Summon Schema

## 决策

### 1. Economy 架构

#### 1.1 货币系统

```rust
/// Currency — 每个角色/队伍持有的货币
/// 挂载在 Party Resource 或 Player Entity 上
#[derive(Component)]
pub struct Wallet {
    pub gold: u64,
    pub premium_currency: u64,     // 高级货币（如"晶石"）
    pub special_tokens: HashMap<TokenId, u32>, // 代币（如"勋章"）
}

/// 货币操作
impl Wallet {
    pub fn deduct_gold(&mut self, amount: u64) -> Result<(), EconomyError>;
    pub fn add_gold(&mut self, amount: u64);
    pub fn transfer(&mut self, other: &mut Wallet, amount: u64) -> Result<(), EconomyError>;
}
```

#### 1.2 商店系统

```rust
/// ShopDef — 商店配置
#[derive(Asset, TypePath)]
pub struct ShopDef {
    pub id: ShopDefId,
    pub name: LocalizationKey,
    pub items: Vec<ShopItem>,
    pub restock_policy: RestockPolicy,   // 是否每次休整补货
    pub tags: Vec<TagId>,
}

pub struct ShopItem {
    pub item_def_id: ItemDefId,
    pub price: Price,
    pub stock: u32,                   // -1 = 无限
    pub required_tags: Vec<TagId>,    // 购买条件
}

pub struct Price {
    pub gold: u64,
    pub tokens: Vec<(TokenId, u32)>,  // 额外代币需求
}

/// ShopSession — 购物会话状态
#[derive(Resource)]
pub struct ShopSession {
    pub shop_def_id: ShopDefId,
    pub selected_item: Option<usize>,
    pub transaction_log: Vec<TransactionRecord>,
}
```

#### 1.3 交易流程

```
打开商店 (CampPhase::FreeRoam)
       │
       ▼
ShopSession 创建
       │
       │
玩家浏览/选择物品
       │
       ▼
购买确认
       │
       ▼
BuyItemSystem
  ├── 校验 Wallet 余额
  ├── 校验条件（等级、Tag）
  ├── 扣除货币
  ├── 创建 ItemStack → 放入 Inventory
  ├── 记录交易日志
  └── 发布 ItemPurchased 事件
```

### 2. Crafting 架构

#### 2.1 配方定义

```rust
/// RecipeDef — 配置加载
#[derive(Asset, TypePath)]
pub struct RecipeDef {
    pub id: RecipeDefId,
    pub name: LocalizationKey,
    pub category: CraftCategory,         // Weapon | Armor | Potion | Accessory
    pub materials: Vec<MaterialCost>,     // 材料需求
    pub tools_required: Vec<TagId>,       // 所需工具
    pub result: CraftResult,              // 产物
    pub skill_check: Option<SkillCheck>,  // 技能检定
    pub difficulty: CraftDifficulty,      // 难度
}

pub struct MaterialCost {
    pub item_def_id: ItemDefId,
    pub quantity: u32,
    pub consumed: bool,    // 是否消耗
}

pub enum CraftResult {
    Item(ItemDefId),
    Modifier(ModifierDefId),  // 附魔/强化
    Random(Vec<(ItemDefId, f32)>), // 随机产出（含概率）
}

pub enum CraftDifficulty {
    Trivial,    // 必定成功
    Easy,       // 高成功率
    Normal,     // 普通
    Hard,       // 低成功率
    Legendary,  // 极低成功率
}
```

#### 2.2 制造流程

```
打开工坊 (CampPhase::FreeRoam)
       │
       ▼
选择配方
       │
       ▼
CraftSystem
  ├── 检查 Inventory 中材料是否充足
  ├── 扣除材料
  ├── 技能检定（随机判定）
  ├── ├── 成功 → 生成产物 → 放入 Inventory
  ├── └── 失败 → 部分材料返还 / 全部消耗
  ├── 发布 ItemCrafted / CraftFailed 事件
  └── 经验奖励（可选）
```

### 3. Summon 架构

#### 3.1 召唤定义

```rust
/// SummonDef — 配置加载
#[derive(Asset, TypePath)]
pub struct SummonDef {
    pub id: SummonDefId,
    pub minion_template: MinionTemplate,   // 召唤物模板
    pub duration: SummonDuration,          // 持续回合数
    pub summon_limit: u32,                 // 同时召唤上限
    pub consume_slot: bool,                // 是否占用队伍槽位
}

pub struct MinionTemplate {
    pub name: LocalizationKey,
    pub base_stats: HashMap<AttributeId, f32>,
    pub abilities: Vec<AbilityDefId>,
    pub tags: Vec<TagId>,
    pub visual_override: Option<VisualDef>,
}
```

#### 3.2 召唤流程

```
Unit 使用 Summon Ability (Ability Pipeline)
       │
       ▼
SummonExecutionSystem
  ├── 检查 summon_limit（已有召唤物数量）
  ├── 创建新的 Minion Entity
  ├── 应用 MinionTemplate 的属性/技能
  ├── 添加 Summoned Tag + Duration 组件
  └── 发布 SummonEvent
       │
       ▼
OnSummonDurationEnd (Observer)
  ├── 移除 Minion Entity
  └── 清理关联的 Modifier
```

## Module Design

```
src/core/domains/economy/
  ├── plugin.rs              — EconomyPlugin
  ├── components.rs          — Wallet
  ├── systems.rs             — buy_item, sell_item, restock_shop
  ├── resources.rs           — ShopSession
  ├── events.rs              — ItemPurchased, ItemSold, GoldChanged
  └── integration/           — 跨域访问 ACL（ADR-046） ShopDef, Price, Wallet API

src/core/domains/crafting/
  ├── plugin.rs              — CraftingPlugin
  ├── systems.rs             — craft_item, roll_craft_result
  ├── events.rs              — ItemCrafted, CraftFailed
  └── integration/           — 跨域访问 ACL（ADR-046） RecipeDef, CraftCategory

src/core/domains/summon/
  ├── plugin.rs              — SummonPlugin
  ├── components.rs          — Summoned, SummonDuration, MinionTag
  ├── systems.rs             — summon_minion, despawn_minion
  ├── events.rs              — SummonEvent
  └── integration/           — 跨域访问 ACL（ADR-046） SummonDef, MinionTemplate
```

## Communication Design

| 通信 | 机制 | 方向 |
|------|------|------|
| 购买 → Inventory | Event (`ItemPurchased`) | economy → inventory |
| 出售 → Wallet | Event (`ItemSold`) | inventory → economy |
| 制造 → Inventory | Event (`ItemCrafted`) | crafting → inventory |
| 召唤 → Entity 创建 | 直接 `commands.spawn()` | summon 内部 |
| 召唤持续时间 → 清除 | Observer (`OnRemove`) | summon 内部 |

## 边界定义

### 允许
- Shop 读取 Wallet 和 Inventory
- Crafting 从 Inventory 扣除材料并添加产物
- Summon 创建新的 Entity（使用 `commands.spawn()`）
- 商店补货策略使用配置定义

### 🟥 禁止
- Economy 直接读取角色属性（只关心 Wallet）
- Crafting 直接产生 Modifier（必须创建物品，物品穿戴后走 Modifier Pipeline）
- Summon 创建超出 limit 数量的召唤物
- 商店直接写入玩家的 Inventory（必须通过 `ItemPurchased` Event）
- 制造失败时消耗全部材料（至少返还部分）

## Forbidden

| 禁止行为 | 理由 |
|---------|------|
| Shop 直接操作 Inventory Component | 通过 Event 解耦 |
| Crafting 成功率硬编码 | 必须配置化 |
| Summon 无限召唤 | summon_limit 检查 |
| 经济系统依赖于战斗逻辑 | Layer 4 不能依赖 Layer 3 |
| 制造产出直接生成 Modifier | 制造产出物品，物品穿戴才走 Modifier |

## Definition / Instance Design

- **Definition**: `ShopDef` (Asset), `RecipeDef` (Asset), `SummonDef` (Asset), `Price` (config)
- **Instance**: `Wallet` (Component), `ShopSession` (Resource), `Summoned` (Component), `SummonDuration` (Component)
- **Persistence**: `Wallet`、`Summoned`（召唤物 Entity 及 Duration 剩余）

## 后果

### 正面
- 经济循环清晰：战斗 → 货币 → 商店/制造 → 装备 → 更强战斗
- Crafting 的随机产出提供了 replayability
- Summon 独立 Entity 管理，不污染单位 Entity

### 负面
- 三个 Feature 之间的交互较多（economy → inventory, crafting → inventory），需要注意 Event 链的顺序
- Summon 创建的 Entity 在存档读档时需要特殊处理（重新关联 SummonDef）

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 单一"资源系统"合并 Economy/Crafting | 两个关注点不同，合并后内聚性差 |
| 制造直接产出 Modifier 而不通过物品 | 绕过 Inventory 系统 |
| 召唤物作为父 Entity 的子 Entity | Bevy Relationship 虽然可以，但初期不需要嵌套 |

## 评审要点

- [ ] 货币体系——是否支持多种货币？兑换机制？
- [ ] 制造成功率——失败时的材料返还比例如何配置？
- [ ] 召唤物是否能使用物品/技能？能否升级？
- [ ] 商店的补货策略——固定周期还是事件驱动？
