# Container（容器）系统详细方案

> 核心原则：**不要设计"背包系统"，而要设计"Container（容器）系统"；背包只是容器的一种表现形式。**

---

## 一、现状分析

### 当前已有的

| 层次 | 已有实现 | 问题 |
|------|---------|------|
| 定义层 | `EquipmentDef`（RON 配置，不可变） | 只覆盖装备，不覆盖消耗品/材料/任务物品 |
| 实例层 | `EquipmentInstance`（运行时，耐久/强化/附魔） | 只服务于装备，消耗品/堆叠/绑定无法表达 |
| 背包层 | `Inventory { items: Vec<EquipmentInstance>, capacity: u32 }` | 只能存装备实例，无法存消耗品/材料/金币 |
| 槽位层 | `EquipmentSlots { slots: HashMap<Slot, (u64, String)> }` | 装备与背包已分离，设计正确 |
| 穿脱层 | `equip_item_system` / `unequip_item_system` | 完整，但只处理装备 |
| 修饰符 | `ModifierSource` 三区间（Trait/Equipment/Buff） | Equipment 区间已预留 |
| 标签 | `PersistentTags { from_traits, from_equipment }` | 两层标签，需扩展 from_container |
| Trait | `TraitSource::Equipment { slot }` | 需扩展支持 Container 来源 |

### 当前缺失的（9.md 要求）

1. **通用 ItemDef / ItemInstance**：装备只是物品的一种，消耗品/材料/任务物品/弹药/卷轴都是物品
2. **Container 统一抽象**：背包、仓库、宝箱、商店、尸体、掉落袋本质都是 Container
3. **ContainerOps 统一接口**：add / remove / transfer 跨容器操作
4. **堆叠系统**：99 个药水 = 1 个 ItemStack，不是 99 个实例
5. **重量/容量系统**：DND 风格的 Weight 而非固定格数
6. **资源统一化**：金币/银币/木材/铁矿/声望统一为 ResourceStack
7. **容器嵌套**：背包 → 药水袋/材料包/箭袋
8. **战场背包**：战斗中只能访问 BattleInventory
9. **容器特性**：次元袋（超大容量）、诅咒袋（随机吞物品）通过 Trait 实现

---

## 二、架构设计

### 四层模型

```
Container（容器）
    ↓ 包含
ItemStack（物品堆叠）
    ↓ 引用
ItemDef（物品定义，不可变）
    ↓ 包含
Trait + Modifier + Tag（效果体系，已有）
```

### 与现有系统的关系

```
                    ItemDef（新增，通用物品定义）
                   /        \
          EquipmentDef      ConsumableDef / MaterialDef / ...
          (已有，改为       (新增，消耗品/材料/...)
           ItemDef 子集)
               ↓                ↓
          ItemInstance（新增，通用物品实例，替代 EquipmentInstance）
               ↓
          ItemStack（新增，堆叠 = ItemInstance × count）
               ↓
          Container（新增，替代 Inventory）
           ↓         ↓
    EquipmentSlots   Container（背包/仓库/宝箱/商店...）
    （已有，不变）    （新增，统一容器）
```

**关键决策**：`EquipmentDef` 保留为 `ItemDef` 的特殊化，不删除。`EquipmentDef` 是 `ItemDef` 中 `item_type = Equipment` 的子集。RON 配置中装备定义自然扩展为物品定义。

---

## 三、核心类型定义

### 3.1 ItemDef — 通用物品定义

```rust
/// 物品类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum ItemType {
    Equipment,      // 装备（可穿戴）
    Consumable,     // 消耗品（药水/卷轴/食物）
    Material,       // 材料（矿石/草药/木材）
    Quest,          // 任务物品
    Ammo,           // 弹药（箭矢/子弹）
    Currency,       // 货币（金币/银币）
    Container,      // 容器物品（背包/箭袋/次元袋）
}

/// 物品定义（RON 配置，不可变）
#[derive(Clone, Debug, Deserialize)]
pub struct ItemDef {
    pub version: u32,
    pub id: String,
    pub name: String,
    pub description: String,
    pub item_type: ItemType,
    pub rarity: Rarity,
    /// 物品标签（如 SWORD, CONSUMABLE, HEALING）
    pub tags: Vec<TagName>,
    /// 最大堆叠数（1 = 不可堆叠，如装备；99 = 可堆叠，如药水）
    #[serde(default = "default_stack_size")]
    pub stack_size: u32,
    /// 重量（DND 风格，0 = 不占重量）
    #[serde(default)]
    pub weight: f32,
    /// 属性修饰（装备/消耗品都可能提供）
    #[serde(default)]
    pub modifiers: Vec<AttributeModifierDef>,
    /// 授予的 Trait
    #[serde(default)]
    pub traits: Vec<String>,
    /// 需求条件（仅装备）
    #[serde(default)]
    pub requirements: Vec<EquipmentRequirement>,
    /// 装备槽位（仅装备）
    #[serde(default)]
    pub slot: Option<EquipmentSlot>,
    /// 使用效果（仅消耗品）
    #[serde(default)]
    pub use_effects: Vec<UseEffect>,
    /// 容器容量（仅 Container 类型）
    #[serde(default)]
    pub container_capacity: Option<u32>,
    /// 容器最大重量（仅 Container 类型）
    #[serde(default)]
    pub container_max_weight: Option<f32>,
}

fn default_stack_size() -> u32 { 1 }

/// 消耗品使用效果
#[derive(Clone, Debug, Deserialize)]
pub enum UseEffect {
    /// 恢复 HP/MP/Stamina
    RestoreVital { kind: AttributeKind, value: f32 },
    /// 施加 Buff
    ApplyBuff { buff_id: String, duration: u32 },
    /// 授予临时 Trait
    GrantTempTrait { trait_id: String, duration: u32 },
    /// 释放技能（卷轴）
    CastSkill { skill_id: String },
}
```

### 3.2 ItemInstance — 通用物品实例

```rust
/// 物品实例（运行时，可变）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemInstance {
    /// 唯一实例 ID
    pub instance_id: u64,
    /// 指向定义 ID
    pub def_id: String,
    /// 当前耐久度（仅装备）
    #[serde(default)]
    pub durability: u32,
    /// 最大耐久度（仅装备）
    #[serde(default)]
    pub max_durability: u32,
    /// 强化等级（仅装备）
    #[serde(default)]
    pub enhance_level: u32,
    /// 附魔 trait（仅装备）
    #[serde(default)]
    pub enchantments: Vec<String>,
    /// 绑定状态
    #[serde(default)]
    pub bind: ItemBind,
    /// 签名/定制标记
    #[serde(default)]
    pub signature: Option<String>,
    /// 任务状态标记
    #[serde(default)]
    pub quest_state: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemBind {
    #[default]
    None,           // 无绑定
    Pickup,         // 拾取绑定
    Equip,          // 装备绑定
    Account,        // 账号绑定
}
```

### 3.3 ItemStack — 堆叠

```rust
/// 物品堆叠：一个实例 × 数量
#[derive(Clone, Debug)]
pub struct ItemStack {
    pub instance: ItemInstance,
    pub count: u32,
}

impl ItemStack {
    pub fn new(instance: ItemInstance, count: u32) -> Self {
        Self { instance, count }
    }

    /// 从定义创建单个实例的堆叠
    pub fn from_def(instance_id: u64, def: &ItemDef, count: u32) -> Self {
        let instance = ItemInstance {
            instance_id,
            def_id: def.id.clone(),
            durability: if def.item_type == ItemType::Equipment { 100 } else { 0 },
            max_durability: if def.item_type == ItemType::Equipment { 100 } else { 0 },
            enhance_level: 0,
            enchantments: vec![],
            bind: ItemBind::None,
            signature: None,
            quest_state: None,
        };
        Self { instance, count }
    }

    /// 能否与另一个堆叠合并
    pub fn can_merge_with(&self, other: &ItemStack, def: &ItemDef) -> bool {
        self.instance.def_id == other.instance.def_id
            && self.count + other.count <= def.stack_size
            && self.instance.bind == ItemBind::None
            && other.instance.bind == ItemBind::None
            && self.instance.enhance_level == other.instance.enhance_level
            && self.instance.enchantments == other.instance.enchantments
    }

    /// 总重量
    pub fn total_weight(&self, def: &ItemDef) -> f32 {
        def.weight * self.count as f32
    }
}
```

### 3.4 Container — 统一容器

```rust
/// 容器类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum ContainerKind {
    Backpack,       // 角色背包
    Warehouse,      // 仓库
    Chest,          // 宝箱
    Shop,           // 商店
    Corpse,         // 尸体
    LootBag,        // 掉落袋
    Mail,           // 邮件
    BattleBag,      // 战场背包
    GuildBank,      // 公会仓库
}

/// 容器组件
#[derive(Component, Debug, Clone)]
pub struct Container {
    /// 容器类型
    pub kind: ContainerKind,
    /// 物品堆叠列表
    pub stacks: Vec<ItemStack>,
    /// 最大格数（0 = 无限制）
    pub capacity: u32,
    /// 最大重量（0 = 无限制，DND 风格）
    pub max_weight: f32,
    /// 所属实体（如角色背包的 owner = 角色 Entity）
    pub owner: Option<Entity>,
    /// 容器 Trait（如 extra_capacity, consume_random_item）
    #[serde(default)]
    pub container_traits: Vec<String>,
}

impl Container {
    pub fn new(kind: ContainerKind, capacity: u32, max_weight: f32) -> Self {
        Self {
            kind,
            stacks: Vec::new(),
            capacity,
            max_weight,
            owner: None,
            container_traits: vec![],
        }
    }

    /// 当前总重量
    pub fn current_weight(&self, registry: &ItemRegistry) -> f32 {
        self.stacks.iter()
            .filter_map(|s| registry.get(&s.instance.def_id).map(|d| s.total_weight(d)))
            .sum()
    }

    /// 是否超重
    pub fn is_overweight(&self, registry: &ItemRegistry) -> bool {
        if self.max_weight <= 0.0 { return false; }
        self.current_weight(registry) > self.max_weight
    }

    /// 是否已满
    pub fn is_full(&self) -> bool {
        self.capacity > 0 && self.stacks.len() >= self.capacity as usize
    }

    /// 添加物品堆叠（自动合并同类型）
    pub fn add_stack(&mut self, mut stack: ItemStack, registry: &ItemRegistry) -> bool {
        if let Some(def) = registry.get(&stack.instance.def_id) {
            // 尝试合并到已有堆叠
            if def.stack_size > 1 {
                for existing in &mut self.stacks {
                    if existing.can_merge_with(&stack, def) {
                        let space = def.stack_size - existing.count;
                        let to_merge = space.min(stack.count);
                        existing.count += to_merge;
                        stack.count -= to_merge;
                        if stack.count == 0 { return true; }
                    }
                }
            }
            // 剩余部分作为新堆叠
            if stack.count > 0 && !self.is_full() {
                self.stacks.push(stack);
                return true;
            }
            // 容量满但已部分合并
            stack.count > 0
        } else {
            false
        }
    }

    /// 移除指定实例 ID 的物品
    pub fn remove(&mut self, instance_id: u64) -> Option<ItemStack> {
        if let Some(idx) = self.stacks.iter().position(|s| s.instance.instance_id == instance_id) {
            Some(self.stacks.remove(idx))
        } else {
            None
        }
    }

    /// 减少指定堆叠的数量（用于消耗品）
    pub fn reduce_stack(&mut self, instance_id: u64, count: u32) -> Option<ItemStack> {
        if let Some(idx) = self.stacks.iter().position(|s| s.instance.instance_id == instance_id) {
            let stack = &mut self.stacks[idx];
            let to_remove = count.min(stack.count);
            stack.count -= to_remove;
            let removed = ItemStack {
                instance: stack.instance.clone(),
                count: to_remove,
            };
            if stack.count == 0 {
                self.stacks.remove(idx);
            }
            Some(removed)
        } else {
            None
        }
    }

    /// 查找指定实例 ID
    pub fn get(&self, instance_id: u64) -> Option<&ItemStack> {
        self.stacks.iter().find(|s| s.instance.instance_id == instance_id)
    }

    /// 按定义 ID 查找第一个堆叠
    pub fn find_by_def(&self, def_id: &str) -> Option<&ItemStack> {
        self.stacks.iter().find(|s| s.instance.def_id == def_id)
    }

    /// 按标签筛选
    pub fn filter_by_tag(&self, tag: GameplayTag, registry: &ItemRegistry) -> Vec<&ItemStack> {
        self.stacks.iter()
            .filter(|s| {
                registry.get(&s.instance.def_id)
                    .map(|d| d.tags.iter().any(|t| t.to_tag() == tag))
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn len(&self) -> usize { self.stacks.len() }
    pub fn is_empty(&self) -> bool { self.stacks.is_empty() }
}
```

### 3.5 ResourceStack — 资源统一化

```rust
/// 资源堆叠（金币/银币/木材/铁矿/声望/贡献点）
#[derive(Clone, Debug)]
pub struct ResourceStack {
    pub resource_id: String,  // "gold", "silver", "wood", "iron", "fame", "contribution"
    pub amount: u32,
}

/// 资源容器（挂在角色 Entity 上）
#[derive(Component, Default, Debug, Clone)]
pub struct Resources {
    pub stacks: Vec<ResourceStack>,
}

impl Resources {
    pub fn add(&mut self, resource_id: &str, amount: u32) {
        if let Some(stack) = self.stacks.iter_mut().find(|s| s.resource_id == resource_id) {
            stack.amount += amount;
        } else {
            self.stacks.push(ResourceStack { resource_id: resource_id.into(), amount });
        }
    }

    pub fn spend(&mut self, resource_id: &str, amount: u32) -> bool {
        if let Some(stack) = self.stacks.iter_mut().find(|s| s.resource_id == resource_id) {
            if stack.amount >= amount {
                stack.amount -= amount;
                return true;
            }
        }
        false
    }

    pub fn get(&self, resource_id: &str) -> u32 {
        self.stacks.iter().find(|s| s.resource_id == resource_id).map(|s| s.amount).unwrap_or(0)
    }
}
```

### 3.6 ItemRegistry — 物品注册表

```rust
/// 物品注册表资源（替代 EquipmentRegistry）
#[derive(Resource, Default)]
pub struct ItemRegistry {
    defs: HashMap<String, ItemDef>,
}

impl ItemRegistry {
    pub fn get(&self, id: &str) -> Option<&ItemDef> { self.defs.get(id) }
    pub fn register(&mut self, def: ItemDef) { self.defs.insert(def.id.clone(), def); }
    pub fn iter(&self) -> impl Iterator<Item = &ItemDef> { self.defs.values() }
    pub fn len(&self) -> usize { self.defs.len() }
    pub fn is_empty(&self) -> bool { self.defs.is_empty() }

    /// 按类型筛选
    pub fn iter_by_type(&self, item_type: ItemType) -> impl Iterator<Item = &ItemDef> {
        self.defs.values().filter(move |d| d.item_type == item_type)
    }

    /// 按标签筛选
    pub fn iter_by_tag(&self, tag: GameplayTag) -> impl Iterator<Item = &ItemDef> {
        self.defs.values().filter(move |d| d.tags.iter().any(|t| t.to_tag() == tag))
    }
}

impl RegistryLoader for ItemRegistry {
    type Item = ItemDef;
    fn register_item(&mut self, item: ItemDef) {
        let id = item.id.clone();
        self.register(item);
        bevy::log::info!(target: "inventory", id = %id, "物品定义已加载");
    }
    fn register_defaults(&mut self) { /* 内置默认物品 */ }
    fn is_empty(&self) -> bool { self.defs.is_empty() }
    fn registry_name() -> &'static str { "Item" }
}
```

---

## 四、ContainerOps — 统一容器操作

```rust
/// 容器操作结果
#[derive(Debug)]
pub enum ContainerResult {
    Ok,
    Full,
    Overweight,
    NotFound,
    StackMerged { remaining: u32 },  // 部分合并，还有剩余
}

/// 容器间转移
pub fn transfer_item(
    from: &mut Container,
    to: &mut Container,
    instance_id: u64,
    count: u32,
    registry: &ItemRegistry,
) -> ContainerResult {
    let stack = match from.get(instance_id) {
        Some(s) => s.clone(),
        None => return ContainerResult::NotFound,
    };

    let to_remove = count.min(stack.count);
    let new_stack = ItemStack {
        instance: stack.instance.clone(),
        count: to_remove,
    };

    // 检查目标容器
    if to.is_full() { return ContainerResult::Full; }
    if to.max_weight > 0.0 {
        let added_weight = new_stack.total_weight(
            registry.get(&new_stack.instance.def_id).unwrap()
        );
        if to.current_weight(registry) + added_weight > to.max_weight {
            return ContainerResult::Overweight;
        }
    }

    // 从源容器移除
    from.reduce_stack(instance_id, to_remove);

    // 添加到目标容器
    to.add_stack(new_stack, registry);

    ContainerResult::Ok
}
```

---

## 五、ModifierSource 扩展

当前 ModifierSource 三区间不变，但 Equipment 区间语义扩展为 **Container 来源**：

```rust
impl ModifierSource {
    // 已有
    pub fn trait_source(index: u64) -> Self { Self(u64::MAX - index) }
    pub fn equipment_source(index: u64) -> Self { Self(u64::MAX - 1000 - index) }
    pub fn buff_source(id: u64) -> Self { Self(id) }

    // 新增：Container 来源（复用 Equipment 区间，因为装备也在 Container 中）
    // 不需要新区间——装备穿戴时仍用 equipment_source
    // 消耗品使用时用 buff_source（因为效果是临时的）
}
```

**设计决策**：消耗品使用效果走 Buff 系统（`ApplyBuff`），因此修饰符来源用 `buff_source`。装备穿戴效果仍用 `equipment_source`。不需要新的 ModifierSource 区间。

---

## 六、PersistentTags 扩展

```rust
#[derive(Component, Default, Debug, Clone)]
pub struct PersistentTags {
    pub from_traits: GameplayTags,      // 种族/职业/天赋
    pub from_equipment: GameplayTags,   // 装备
    // 不需要 from_container 层——容器本身不提供标签
    // 消耗品效果走 Buff 层（临时），装备效果走 Equipment 层（持久）
}
```

**设计决策**：不需要第三层 `from_container`。Container 是存储概念，不提供标签。物品效果要么是持久的（装备 → `from_equipment`），要么是临时的（消耗品 → Buff 层）。

---

## 七、Message 定义

```rust
// 容器操作 Message
#[derive(Message, Debug, Clone)]
pub struct AddItemToContainer {
    pub container_entity: Entity,
    pub def_id: String,
    pub count: u32,
}

#[derive(Message, Debug, Clone)]
pub struct RemoveItemFromContainer {
    pub container_entity: Entity,
    pub instance_id: u64,
    pub count: u32,
}

#[derive(Message, Debug, Clone)]
pub struct TransferItem {
    pub from_entity: Entity,
    pub to_entity: Entity,
    pub instance_id: u64,
    pub count: u32,
}

#[derive(Message, Debug, Clone)]
pub struct UseItem {
    pub user_entity: Entity,
    pub container_entity: Entity,
    pub instance_id: u64,
}

// 通知 Message
#[derive(Message, Debug, Clone)]
pub struct ItemAdded {
    pub container_entity: Entity,
    pub def_id: String,
    pub count: u32,
}

#[derive(Message, Debug, Clone)]
pub struct ItemRemoved {
    pub container_entity: Entity,
    pub def_id: String,
    pub count: u32,
}

#[derive(Message, Debug, Clone)]
pub struct ItemUsed {
    pub user_entity: Entity,
    pub def_id: String,
}

// 装备穿脱（保留已有，微调）
#[derive(Message, Debug, Clone)]
pub struct EquipItem {
    pub target_entity: Entity,
    pub instance_id: u64,
}

#[derive(Message, Debug, Clone)]
pub struct UnequipItem {
    pub target_entity: Entity,
    pub slot: EquipmentSlot,
}

// 已有：ItemEquipped, ItemUnequipped, EquipFailed
```

---

## 八、消耗品使用系统

```rust
/// 使用消耗品
pub fn use_item_system(
    mut messages: MessageReader<UseItem>,
    containers: Query<&mut Container>,
    units: Query<(&mut Attributes, &mut ActiveBuffs, &mut GameplayTags, &PersistentTags)>,
    item_registry: Res<ItemRegistry>,
    buff_registry: Res<BuffRegistry>,
    trait_registry: Res<TraitRegistry>,
    effect_handlers: Res<TraitEffectHandlerRegistry>,
) {
    for msg in messages.read() {
        let Ok(mut container) = containers.get(msg.container_entity) else { continue };
        let Some(stack) = container.get(msg.instance_id).cloned() else { continue };
        let Some(def) = item_registry.get(&stack.instance.def_id) else { continue };

        if def.item_type != ItemType::Consumable { continue; }

        // 应用使用效果
        let Ok((mut attrs, mut buffs, mut tags, persistent)) = units.get_mut(msg.user_entity) else { continue };
        for effect in &def.use_effects {
            match effect {
                UseEffect::RestoreVital { kind, value } => {
                    attrs.apply_modifier(ModifierSource::buff_source(stack.instance.instance_id), *kind, ModifierOp::Add, *value);
                }
                UseEffect::ApplyBuff { buff_id, duration } => {
                    if let Some(buff_data) = buff_registry.get(buff_id) {
                        let instance = buffs.add(buff_data.clone(), *duration, Some(msg.user_entity));
                        apply_buff(&mut attrs, &mut tags, &persistent, &buffs.instances.last().unwrap(), instance);
                    }
                }
                UseEffect::GrantTempTrait { trait_id, duration } => {
                    // 临时 Trait 通过 Buff 实现
                }
                UseEffect::CastSkill { skill_id } => {
                    // 技能释放由技能系统处理
                }
            }
        }

        // 消耗一个
        container.reduce_stack(msg.instance_instance_id, 1);
    }
}
```

---

## 九、战场背包

```rust
/// 战场背包组件（战斗中生成，战斗结束合并回角色背包）
#[derive(Component, Debug, Clone)]
pub struct BattleInventory {
    pub container: Container,
    /// 原始背包 Entity（战斗结束后归还）
    pub source_backpack: Entity,
}

/// 战斗开始时：从角色背包生成战场背包
pub fn create_battle_inventory(
    backpacks: Query<(Entity, &Container), With<PlayerUnit>>,
    mut commands: Commands,
) {
    for (entity, backpack) in &backpacks {
        let battle_bag = BattleInventory {
            container: Container {
                kind: ContainerKind::BattleBag,
                capacity: 6,  // 战场背包只允许带 6 个物品
                max_weight: 0.0,
                stacks: vec![],  // 玩家选择携带的物品
                owner: Some(entity),
                container_traits: vec![],
            },
            source_backpack: entity,
        };
        commands.spawn(battle_bag);
    }
}

/// 战斗结束时：战场背包物品归还角色背包
pub fn merge_battle_inventory(
    mut battle_bags: Query<&mut BattleInventory>,
    mut backpacks: Query<&mut Container>,
    item_registry: Res<ItemRegistry>,
) {
    for battle_bag in &mut battle_bags {
        if let Ok(mut backpack) = backpacks.get_mut(battle_bag.source_backpack) {
            for stack in battle_bag.container.stacks.drain(..) {
                backpack.add_stack(stack, &item_registry);
            }
        }
    }
}
```

---

## 十、目录结构

```
src/inventory/
├── mod.rs                    # 模块入口 + re-exports
├── plugin.rs                 # InventoryPlugin（注册 Message + Resource）
├── definition.rs             # ItemDef / ItemType / UseEffect / ItemRegistry
├── instance.rs               # ItemInstance / ItemBind / ItemStack
├── container.rs              # Container / ContainerKind / ContainerOps / transfer_item
├── resources.rs              # ResourceStack / Resources
├── commands/
│   ├── mod.rs                # re-exports
│   ├── add_item.rs           # AddItemToContainer 系统
│   ├── remove_item.rs        # RemoveItemFromContainer 系统
│   ├── transfer_item.rs      # TransferItem 系统
│   └── use_item.rs           # UseItem 系统
├── observers/
│   ├── mod.rs                # re-exports
│   └── inventory_changed.rs  # ItemAdded/ItemRemoved Observer
├── views/
│   └── inventory_view.rs     # InventoryViewModel（UI 读取用）
└── battle/
    ├── mod.rs                # re-exports
    └── battle_inventory.rs   # 战场背包系统
```

**与现有 equipment/ 的关系**：

```
src/equipment/                 # 保留，专注装备穿脱逻辑
├── definition.rs              # EquipmentDef 保留（ItemDef 的装备子集）
├── instance.rs                # EquipmentInstance → 改为引用 ItemInstance
├── slots.rs                   # EquipmentSlots 保留不变
├── equip.rs                   # 穿脱逻辑保留，改用 ItemRegistry
├── requirements.rs            # 需求检查保留不变
└── plugin.rs                  # EquipmentPlugin 保留

src/inventory/                 # 新增，通用容器系统
├── ...（如上）
```

**迁移策略**：`EquipmentDef` 不删除，`ItemDef` 是超集。`EquipmentDef` 的 RON 配置自然兼容 `ItemDef`（`item_type: Equipment`）。`EquipmentInstance` 内部改为包含 `ItemInstance`。

---

## 十一、RON 配置示例

### 装备（兼容现有格式，自然扩展）

```ron
// assets/items/iron_sword.ron
(
    version: 1,
    id: "iron_sword",
    name: "铁剑",
    description: "普通的铁剑",
    item_type: Equipment,
    rarity: Common,
    tags: [Sword, Melee, Martial],
    stack_size: 1,
    weight: 3.0,
    modifiers: [
        (kind: Attack, op: Add, value: 3.0),
    ],
    slot: MainHand,
)
```

### 消耗品

```ron
// assets/items/potion_healing.ron
(
    version: 1,
    id: "potion_healing",
    name: "治疗药水",
    description: "恢复 50 HP",
    item_type: Consumable,
    rarity: Common,
    tags: [Consumable, Potion, Healing],
    stack_size: 99,
    weight: 0.5,
    use_effects: [
        RestoreVital(kind: Hp, value: 50.0),
    ],
)
```

### 材料

```ron
// assets/items/iron_ore.ron
(
    version: 1,
    id: "iron_ore",
    name: "铁矿石",
    description: "锻造材料",
    item_type: Material,
    rarity: Common,
    tags: [Material, Metal],
    stack_size: 999,
    weight: 2.0,
)
```

### 弹药

```ron
// assets/items/arrow.ron
(
    version: 1,
    id: "arrow",
    name: "箭矢",
    description: "普通箭矢",
    item_type: Ammo,
    rarity: Common,
    tags: [Ammo, Arrow],
    stack_size: 99,
    weight: 0.1,
    modifiers: [
        (kind: Attack, op: Add, value: 1.0),
    ],
)
```

### 容器物品

```ron
// assets/items/bag_of_holding.ron
(
    version: 1,
    id: "bag_of_holding",
    name: "次元袋",
    description: "容量远超普通背包的魔法袋",
    item_type: Container,
    rarity: Rare,
    tags: [Container, Magical],
    stack_size: 1,
    weight: 1.0,
    container_capacity: 100,
    container_max_weight: 500.0,
    traits: ["extra_capacity"],
)
```

### 货币

```ron
// assets/items/gold_coin.ron
(
    version: 1,
    id: "gold_coin",
    name: "金币",
    description: "通用货币",
    item_type: Currency,
    rarity: Common,
    tags: [Currency],
    stack_size: 999999,
    weight: 0.01,
)
```

---

## 十二、Unit 组件变更

```rust
// 之前
#[derive(Component)]
#[require(
    Attributes, SkillSlots, SkillCooldowns, ActiveBuffs, GameplayTags,
    PersistentTags, TraitCollection, EquipmentSlots, Inventory, GridPosition
)]
pub struct Unit { ... }

// 之后
#[derive(Component)]
#[require(
    Attributes, SkillSlots, SkillCooldowns, ActiveBuffs, GameplayTags,
    PersistentTags, TraitCollection, EquipmentSlots, GridPosition,
    // Inventory → Container（背包类型）
)]
pub struct Unit { ... }

// Container 作为 Required Component 的默认值
impl Default for Container {
    fn default() -> Self {
        Container::new(ContainerKind::Backpack, 20, 0.0)
    }
}
```

---

## 十三、ViewModel 变更

```rust
/// 背包条目（UI 显示用）
#[derive(Clone, Debug)]
pub struct InventoryEntry {
    pub item_name: String,
    pub rarity: String,
    pub count: u32,          // 新增：堆叠数量
    pub weight: f32,         // 新增：单件重量
    pub item_type: String,   // 新增：物品类型
}

/// 更新逻辑
fn update_inventory_view(container: &Container, registry: &ItemRegistry) -> Vec<InventoryEntry> {
    container.stacks.iter()
        .filter_map(|stack| {
            registry.get(&stack.instance.def_id).map(|def| InventoryEntry {
                item_name: def.name.clone(),
                rarity: def.rarity.label().to_string(),
                count: stack.count,
                weight: def.weight,
                item_type: match def.item_type {
                    ItemType::Equipment => "装备",
                    ItemType::Consumable => "消耗品",
                    ItemType::Material => "材料",
                    ItemType::Quest => "任务物品",
                    ItemType::Ammo => "弹药",
                    ItemType::Currency => "货币",
                    ItemType::Container => "容器",
                }.to_string(),
            })
        })
        .collect()
}
```

---

## 十四、实施阶段

### 阶段 1：ItemDef + ItemRegistry（基础设施）
- 创建 `src/inventory/` 模块
- 定义 `ItemType` / `ItemDef` / `UseEffect`
- 实现 `ItemRegistry`（实现 `RegistryLoader`）
- 从 `assets/items/*.ron` 加载
- `ItemDef` 兼容现有 `EquipmentDef` RON 格式（`item_type` 默认为 Equipment）
- **测试**：ItemDef RON 反序列化、ItemRegistry 查询/筛选

### 阶段 2：ItemInstance + ItemStack（实例与堆叠）
- 定义 `ItemInstance` / `ItemBind` / `ItemStack`
- `EquipmentInstance` 内部改为包含 `ItemInstance`
- 堆叠合并逻辑（`can_merge_with` / `add_stack`）
- **测试**：堆叠创建/合并/拆分

### 阶段 3：Container（统一容器）
- 定义 `Container` / `ContainerKind`
- 实现 `ContainerOps`（add / remove / reduce / transfer）
- `Inventory` 组件迁移为 `Container { kind: Backpack }`
- `Unit` 的 `#[require]` 更新
- **测试**：Container CRUD、容量/重量检查、跨容器转移

### 阶段 4：ResourceStack（资源统一化）
- 定义 `ResourceStack` / `Resources`
- 金币/银币/声望等统一管理
- **测试**：资源增减/查询

### 阶段 5：消耗品使用系统
- 定义 `UseItem` Message
- 实现 `use_item_system`（RestoreVital / ApplyBuff / CastSkill）
- 消耗品消耗后减少堆叠
- **测试**：药水使用/卷轴使用

### 阶段 6：容器间转移
- 定义 `TransferItem` Message
- 实现 `transfer_item_system`
- Observer：`ItemAdded` / `ItemRemoved` 通知
- **测试**：背包→仓库、宝箱→背包、商店买卖

### 阶段 7：战场背包
- 定义 `BattleInventory` 组件
- 战斗开始生成战场背包（限制携带数量）
- 战斗结束归还
- **测试**：战场背包生成/归还

### 阶段 8：UI 更新
- 背包面板支持堆叠数量显示
- 背包面板支持物品类型筛选（装备/消耗品/材料/全部）
- 容器面板（宝箱/商店/尸体）
- 资源面板（金币/声望）
- **测试**：UI ViewModel 更新

### 阶段 9：迁移 + 清理
- `EquipmentDef` → `ItemDef` 兼容层
- `EquipmentInstance` → `ItemInstance` 迁移
- `Inventory` → `Container` 迁移
- `EquipmentRegistry` 保留（内部委托 `ItemRegistry`）
- 移除 `src/equipment/instance.rs` 中的 `Inventory`
- 更新所有引用
- **测试**：全量回归

---

## 十五、关键设计决策总结

| 决策 | 选择 | 原因 |
|------|------|------|
| ModifierSource 新区间？ | 不需要 | 消耗品走 Buff 系统，装备走 Equipment 区间 |
| PersistentTags 新增 from_container？ | 不需要 | Container 是存储概念，不提供标签 |
| EquipmentDef 删除？ | 不删除 | 保留为 ItemDef 的装备子集，兼容现有 RON |
| EquipmentInstance 删除？ | 逐步迁移 | 内部改为包含 ItemInstance，外部接口不变 |
| Inventory 删除？ | 迁移为 Container | Backpack 只是 Container 的一种 |
| 堆叠方式 | ItemStack（instance + count） | 99 个药水 = 1 个 ItemStack，不是 99 个实例 |
| 容量方式 | Weight + Capacity 双限制 | DND 风格重量 + 传统格数上限 |
| 容器嵌套 | Container Trait 实现 | 次元袋/诅咒袋通过 Trait 扩展 |
| 战场背包 | 独立 BattleInventory 组件 | 战斗中限制携带，策略性 |
| 资源统一 | ResourceStack | 金币/银币/声望统一，不写死 gold: u32 |
