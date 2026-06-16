---
id: domains.economy.schema.v1
title: Economy Schema — 经济/交易数据架构
status: draft
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: definition, instance, persistence
replay-safe: false
---

# Economy Schema — 经济/交易数据架构

> **领域归属**: Domains — 经济系统层 | **依赖 Schema**: Inventory, Faction, Event | **定义依据**: `docs/02-domain/economy_domain.md`

---

## 1. Schema Design

### 1.1 CurrencyType（Definition 层）

```rust
/// 货币类型枚举。运行时不修改。
enum CurrencyType {
    Gold,    // 金币 GP
    Silver,  // 银币 SP (1 GP = 10 SP)
    Copper,  // 铜币 CP (1 SP = 10 CP)
    Special(String), // 特殊货币
}
```

### 1.2 Wallet（Instance 层/Persistence 层）

```rust
/// 角色的钱包——持有所有货币类型的数量。
struct Wallet {
    /// 各货币类型的持有量
    currencies: HashMap<CurrencyType, u64>,
}

impl Wallet {
    fn can_afford(&self, cost: &Price) -> bool { ... }
    fn deduct(&mut self, cost: &Price) -> bool { ... }
    fn add(&mut self, currency: CurrencyType, amount: u64) { ... }
}
```

### 1.3 ShopDef（Definition 层）

```rust
/// 商店定义。内容团队配置。
struct ShopDef {
    /// 商店唯一标识（前缀: `shp_`）
    id: ShopDefId,

    /// 商店名称本地化 Key
    name_key: LocalizationKey,

    /// 所属阵营
    faction_id: FactionDefId,

    /// 商品条目列表
    inventory: Vec<ShopEntryDef>,

    /// 补货规则
    restock_policy: RestockPolicy,
}

struct ShopEntryDef {
    /// 物品模板 ID
    item_id: ItemDefId,

    /// 基础价格（覆盖 ItemDef.base_price，如未设置则使用 ItemDef 默认）
    base_price: Option<u64>,

    /// 初始库存数量（-1 = 无限）
    initial_stock: i32,

    /// 每次补货恢复的数量
    restock_amount: u32,

    /// 是否为赃物收购（黑市特征）
    buys_stolen: bool,
}

enum RestockPolicy {
    /// 固定时间间隔补货（游戏内小时）
    Timed { interval_hours: u32 },
    /// 每次玩家访问时补货
    OnVisit { full_restock: bool },
    /// 永不补货
    Never,
}
```

### 1.4 ShopInstance（Instance 层/Persistence 层）

```rust
/// 商店运行时实例——追踪当前库存和价格状态。
struct ShopInstance {
    /// 引用的商店定义
    shop_def_id: ShopDefId,

    /// 当前库存（item_id → 剩余数量）
    current_stock: HashMap<ItemDefId, i32>,

    /// 各物品的供需系数
    supply_demand: HashMap<ItemDefId, SupplyDemand>,

    /// 上次补货时间
    last_restock: GameTime,
}

enum SupplyDemand {
    Surplus,    // × 0.8
    Balanced,   // × 1.0
    Scarce,     // × 1.5
    Shortage,   // × 2.0
}
```

### 1.5 Price（Value 对象）

```rust
/// 价格值对象——封装价格计算逻辑。
struct Price {
    base: u64,
    reputation_modifier: f32,   // 声望折扣
    supply_modifier: f32,       // 供需系数
    stolen_modifier: f32,       // 赃物折扣（如有）
}

impl Price {
    fn final_price(&self) -> u64 {
        (self.base as f32 * self.reputation_modifier
            * self.supply_modifier * self.stolen_modifier) as u64
    }
}
```

### 1.6 EconomyState（Persistence 层）

```rust
/// 经济系统的持久化状态。
struct EconomyState {
    /// 所有角色的钱包
    wallets: Vec<(EntityId, Wallet)>,

    /// 商店实例状态
    shops: Vec<ShopInstance>,
}
```

---

## 2. Layer Summary

| Layer | Structures | 说明 |
|-------|-----------|------|
| **Definition** | `CurrencyType`, `ShopDef` | 货币体系和商店定义为静态配置 |
| **Spec** | — | Economy 无 Spec 层 |
| **Instance** | `Wallet`, `ShopInstance`, `Price` | 钱包和商店运行时状态；Price 为瞬时值对象 |
| **Persistence** | `EconomyState` | 钱包和商店库存持久化 |

---

## 3. Dependency Analysis

| 依赖 | 说明 |
|------|------|
| → InventorySchema | 交易涉及物品的获得/移除 |
| → FactionSchema | 价格折扣依赖声望数据 |
| → EventSchema | 交易事件发布（TransactionCompleted, CurrencyChanged） |
| ← CraftingSchema | 制作材料可购买，成品可出售 |

---

## 4. Replay & Save

### Replay

- 标记 `replay-safe: false` — 经济是玩家进程数据

### Save

- `EconomyState` 持久化（钱包 + 商店库存状态）
- ShopDef 从配置加载
- 交易记录（TransactionRecord）可选持久化（调试/回滚用）

---

## 5. Validation Rules

| 规则 | 说明 | 违反处理 |
|------|------|----------|
| 货币非负 | 任何货币持有量 >= 0 | 运行时断言 |
| 交易物存在 | 购买时商店有库存；出售时背包有物品 | 交易失败 |
| 价格确定性 | 相同条件下同一物品价格一致 | 运行时断言 |
| 不透支 | 不支持贷款/透支 | Wallet.deduct 返回 false |

---

## 6. Constitution Check

- ✅ **Data Law 001 (Def-Instance分离)**: ShopDef 为 Definition，Wallet/ShopInstance 为 Instance
- ✅ **Data Law 002 (Rule-Content分离)**: 价格计算公式为代码规则
- ✅ **Data Law 003 (配置只引用ID)**: ShopDef 引用 ItemDefId/FactionDefId
- ✅ **Data Law 011 (Schema版本化)**: EconomyState 携带版本号
- ✅ **Data Law 012 (域间禁止直接数据引用)**: Economy 通过 Event 与 Inventory/Faction 通信
