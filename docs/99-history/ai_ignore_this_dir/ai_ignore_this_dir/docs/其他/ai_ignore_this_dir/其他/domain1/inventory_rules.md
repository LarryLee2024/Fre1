# 背包领域规则 (Inventory Rules)

## 1. 领域概述

背包系统管理物品的存储、堆叠、使用和转移。采用统一容器模型，背包、仓库、宝箱、商店、尸体、掉落袋均为 Container 实例。遵循 **Definition / Instance 分离**和 **Rule / Content 分离**原则。

### 核心原则

- **统一容器模型**：所有存储空间本质都是 Container
- **Definition / Instance 分离**：ItemDef（不可变配置）与 ItemInstance（运行时可变）
- **堆叠合并**：同类型物品自动合并，受 stack_size 和 bind 约束
- **容量与重量双限制**：格数 + 重量双重约束
- **消耗品使用**：通过 UseItem Message 触发，自动消耗数量

---

## 2. ItemType — 物品类型

| 类型 | 说明 | stack_size | 重量 | 特殊字段 |
|------|------|-----------|------|----------|
| `Equipment` | 装备 | 1 | 有 | slot, requirements, modifiers, traits |
| `Consumable` | 消耗品 | 99 | 有 | use_effects |
| `Material` | 材料 | 99 | 有 | — |
| `Quest` | 任务物品 | 1 | 0 | quest_state |
| `Ammo` | 弹药 | 99 | 有 | modifiers |
| `Currency` | 货币 | 9999 | 0 | — |
| `Container` | 容器 | 1 | 有 | container_capacity, container_max_weight |

---

## 3. ItemDef — 物品定义

```rust
pub struct ItemDef {
    pub version: u32,
    pub id: String,
    pub name: String,
    pub description: String,
    pub item_type: ItemType,
    pub rarity: Rarity,
    pub tags: Vec<TagName>,
    pub stack_size: u32,           // 1=不可堆叠, 99=可堆叠
    pub weight: f32,               // 0=不占重量
    pub modifiers: Vec<AttributeModifierDef>,
    pub traits: Vec<String>,
    pub requirements: Vec<EquipmentRequirement>,
    pub slot: Option<EquipmentSlot>,
    pub use_effects: Vec<UseEffect>,
    pub container_capacity: Option<u32>,
    pub container_max_weight: Option<f32>,
}
```

**规则**：
- `item_type` 默认 `Equipment`，兼容旧装备 RON
- `stack_size` 默认 1
- 装备的 `slot` 和 `requirements` 仅对 Equipment 类型有意义
- 消耗品的 `use_effects` 仅对 Consumable 类型有意义

---

## 4. UseEffect — 使用效果

| 效果 | 参数 | 说明 |
|------|------|------|
| `RestoreVital { kind, value }` | AttributeKind, f32 | 恢复 HP/MP/Stamina |
| `ApplyBuff { buff_id, duration }` | String, u32 | 施加 Buff |
| `GrantTempTrait { trait_id, duration }` | String, u32 | 授予临时 Trait |
| `CastSkill { skill_id }` | String | 释放技能（卷轴） |

---

## 5. ItemInstance — 物品实例

```rust
pub struct ItemInstance {
    pub instance_id: u64,          // 全局唯一 ID
    pub def_id: String,            // 指向定义 ID
    pub durability: u32,           // 当前耐久度（仅装备）
    pub max_durability: u32,       // 最大耐久度（仅装备）
    pub enhance_level: u32,        // 强化等级
    pub enchantments: Vec<String>, // 附魔 trait
    pub bind: ItemBind,            // 绑定状态
    pub signature: Option<String>, // 签名/定制标记
    pub quest_state: Option<String>, // 任务状态标记
}
```

### 5.1 ItemBind — 绑定状态

| 状态 | 说明 |
|------|------|
| `None` | 未绑定 |
| `Pickup` | 拾取绑定 |
| `Equip` | 装备绑定 |
| `Account` | 账号绑定 |

### 5.2 实例创建

```rust
ItemInstance::from_def(instance_id, def)
```

- 装备：durability = 100, max_durability = 100
- 非装备：durability = 0, max_durability = 0

### 5.3 InstanceIdCounter

```rust
#[derive(Resource)]
pub struct InstanceIdCounter(pub u64);
```

全局自增 ID 生成器，每次 `next()` 返回递增的 ID。

---

## 6. ItemStack — 物品堆叠

```rust
pub struct ItemStack {
    pub instance: ItemInstance,
    pub count: u32,
}
```

### 6.1 合并规则

```rust
can_merge_with(other, def) -> bool
```

条件：
1. `def_id` 相同
2. `count + other.count <= def.stack_size`
3. 双方 `bind == None`
4. `enhance_level` 相同
5. `enchantments` 相同

### 6.2 拆分

```rust
split(count) -> Option<ItemStack>
```

- `count == 0` 或 `count >= self.count` → 返回 None
- 拆分出的实例 `instance_id = 0`，由调用方分配新 ID

### 6.3 重量

```rust
total_weight(def) -> f32  // def.weight * count
```

---

## 7. Container — 统一容器

```rust
#[derive(Component)]
pub struct Container {
    pub kind: ContainerKind,
    pub stacks: Vec<ItemStack>,
    pub capacity: u32,             // 0=无限制
    pub max_weight: f32,           // 0=无限制
    pub owner: Option<Entity>,
    pub container_traits: Vec<String>,
}
```

### 7.1 ContainerKind — 容器类型

| 类型 | 说明 |
|------|------|
| `Backpack` | 角色背包 |
| `Warehouse` | 仓库 |
| `Chest` | 宝箱 |
| `Shop` | 商店 |
| `Corpse` | 尸体 |
| `LootBag` | 掉落袋 |
| `Mail` | 邮件 |
| `BattleBag` | 战场背包 |
| `GuildBank` | 公会银行 |

### 7.2 预设容器

| 工厂方法 | 类型 | 容量 | 重量 |
|----------|------|------|------|
| `backpack()` | Backpack | 20 | 100.0 |
| `battle_bag()` | BattleBag | 8 | 30.0 |
| `chest(cap, weight)` | Chest | 自定义 | 自定义 |

### 7.3 核心操作

| 方法 | 说明 |
|------|------|
| `add_stack(stack, registry)` | 添加物品（自动合并），返回成功添加数量 |
| `remove(instance_id)` | 移除指定实例 |
| `reduce_stack(instance_id, count)` | 减少堆叠数量，归零自动移除 |
| `get(instance_id)` | 查找指定实例 |
| `find_by_def(def_id)` | 按定义 ID 查找 |
| `filter_by_type(item_type, registry)` | 按类型筛选 |
| `is_full()` | 是否已满 |
| `is_overweight(registry)` | 是否超重 |
| `current_weight(registry)` | 当前总重量 |

### 7.4 add_stack 流程

```
1. 查找 ItemDef
2. 尝试合并到已有堆叠（每次合并前检查重量）
3. 剩余部分作为新堆叠（检查容量和重量）
4. 返回成功添加的数量（0=完全失败，< count=部分失败）
```

---

## 8. BattleInventory — 战场背包

```rust
#[derive(Component)]
pub struct BattleInventory {
    pub container: Container,
    pub source_backpack: Entity,   // 原始背包 Entity
}
```

### 8.1 战斗开始

`create_battle_inventory`：
1. 遍历所有 `Container(Backpack)`
2. 创建 `BattleInventory`（容量 8，重量 30）
3. 复制源背包物品到战场背包
4. Spawn 战场背包实体

### 8.2 战斗结束

`merge_battle_inventory`：
1. 遍历所有 `BattleInventory`
2. 将战场背包物品 `drain(..)` 转移回源背包
3. 战场背包实体由 Despawn 清理

---

## 9. UseItem — 消耗品使用

### 9.1 Message

```rust
#[derive(Message)]
pub struct UseItem {
    pub user_entity: Entity,
    pub container_entity: Entity,
    pub instance_id: u64,
}
```

### 9.2 使用流程

```
1. 从容器查找物品
2. 检查 item_type == Consumable
3. 应用 use_effects：
   - RestoreVital → 添加属性修饰符
   - ApplyBuff → 通过 ActiveBuffs 添加 Buff 实例
4. container.reduce_stack(instance_id, 1)
5. 发送 ItemUsed Message
```

### 9.3 ItemUsed 通知

```rust
#[derive(Message)]
pub struct ItemUsed {
    pub user_entity: Entity,
    pub def_id: String,
}
```

---

## 10. TransferItem — 容器间转移

### 10.1 Message

```rust
#[derive(Message)]
pub struct TransferItem {
    pub from_entity: Entity,
    pub to_entity: Entity,
    pub instance_id: u64,
    pub count: u32,
}
```

### 10.2 转移流程

```
1. 查找源容器中的物品
2. 检查目标容器容量和重量
3. 从源容器 reduce_stack
4. 添加到目标容器 add_stack
5. 发送 ItemTransferred Message
```

### 10.3 ContainerResult

| 结果 | 说明 |
|------|------|
| `Ok` | 转移成功 |
| `Full` | 目标容器已满 |
| `Overweight` | 目标容器超重 |
| `NotFound` | 源容器中未找到物品 |

### 10.4 纯函数版本

```rust
transfer_item(from, to, instance_id, count, registry) -> ContainerResult
```

用于测试和程序化调用，不依赖 ECS。

---

## 11. Resources — 资源系统

```rust
#[derive(Component)]
pub struct Resources {
    pub stacks: Vec<ResourceStack>,
}
```

### 11.1 ResourceStack

```rust
pub struct ResourceStack {
    pub resource_id: String,  // "gold", "silver", "fame" 等
    pub amount: u32,
}
```

### 11.2 操作

| 方法 | 说明 |
|------|------|
| `add(resource_id, amount)` | 添加资源（自动累加） |
| `spend(resource_id, amount)` | 消费资源（不足返回 false） |
| `get(resource_id)` | 查询数量（不存在返回 0） |

---

## 12. ItemRegistry — 物品注册表

```rust
#[derive(Resource)]
pub struct ItemRegistry {
    defs: HashMap<String, ItemDef>,
}
```

| 方法 | 说明 |
|------|------|
| `get(id)` | 查找定义 |
| `register(def)` | 注册定义 |
| `iter()` | 遍历所有定义 |
| `iter_by_type(item_type)` | 按类型筛选 |
| `len()` / `is_empty()` | 数量查询 |

**数据加载**：`assets/items/` 目录，通过 `RegistryLoader` 加载 RON 文件。

---

## 13. 内置默认物品

| ID | 名称 | 类型 | 稀有度 | 特殊属性 |
|----|------|------|--------|----------|
| `iron_sword` | 铁剑 | Equipment | Common | Attack+3, MainHand |
| `leather_armor` | 皮甲 | Equipment | Common | Defense+2, Body |
| `flame_dragon_sword` | 炎龙长剑 | Equipment | Epic | Attack+15, CritRate+5, flaming_weapon+dragon_bane |
| `iron_shield` | 铁盾 | Equipment | Common | Defense+3, OffHand |
| `mage_staff` | 法师法杖 | Equipment | Uncommon | MagicAttack+5, MainHand |
| `potion_healing` | 治疗药水 | Consumable | Common | RestoreVital(Hp, 50) |
| `potion_mana` | 法力药水 | Consumable | Common | RestoreVital(Mp, 30) |
| `arrow` | 箭矢 | Ammo | Common | Attack+1 |

---

## 14. RON 配置格式

### 14.1 装备

```ron
(
    id: "iron_sword",
    name: "铁剑",
    description: "普通的铁剑",
    item_type: Equipment,
    rarity: Common,
    tags: [SWORD, MELEE, MARTIAL],
    stack_size: 1,
    weight: 3.0,
    modifiers: [
        (kind: Attack, op: Add, value: 3.0),
    ],
    slot: Some(MainHand),
)
```

### 14.2 消耗品

```ron
(
    id: "potion_healing",
    name: "治疗药水",
    description: "恢复 50 HP",
    item_type: Consumable,
    rarity: Common,
    tags: [CONSUMABLE],
    stack_size: 99,
    weight: 0.5,
    use_effects: [
        RestoreVital(kind: Hp, value: 50.0),
    ],
)
```

---

## 15. 关键约束

1. **统一容器模型**：所有存储空间都是 Container，不单独实现背包/仓库/宝箱
2. **堆叠合并条件严格**：def_id + bind + enhance_level + enchantments 全部匹配
3. **容量与重量双限制**：add_stack 逐步检查，部分成功返回已添加数量
4. **消耗品仅 Consumable 可使用**：Equipment 类型忽略 UseItem
5. **战场背包战斗开始复制、战斗结束归还**：不直接操作角色背包
6. **InstanceIdCounter 全局唯一**：跨容器实例 ID 不重复
7. **转移时不可同时可变借用同一 Entity**：from_entity != to_entity
8. **Resources 独立于 Container**：货币/声望等不走物品堆叠
9. **注册表幂等**：重复调用不会重复注册
10. **旧装备 RON 兼容**：缺少 item_type 默认 Equipment，缺少 stack_size 默认 1
