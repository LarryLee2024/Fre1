# 装备系统详细方案

> 基于 `docs/8.md` 思想、项目铁律、Bevy 0.18 特性、现有架构的实战方案

---

## 一、核心设计哲学

```
装备 = Modifier + Trait + Tag + Rule
职业 = Modifier + Trait + Tag + Rule
种族 = Modifier + Trait + Tag + Rule
Buff = Modifier + Trait + Tag + Rule
```

**一套扩展机制，四种来源。** 不造四套代码。

---

## 二、现有系统评估

### 2.1 可直接复用

| 现有能力 | 装备复用方式 |
|---|---|
| `AttributeModifierInstance` + `ModifierOp` | 装备属性加成直接复用，无需新建修饰符类型 |
| `Attributes.add_modifier() / remove_modifiers_from()` | 穿戴/脱卸时增删修饰符 |
| `GameplayTag / GameplayTags` 位掩码 | 装备授予标签（火剑→FIRE，重甲→HEAVY_ARMOR） |
| `TraitDefinition / TraitData / TraitEffect` | 装备被动效果引用现有 trait |
| `TraitEffectHandlerRegistry` | 装备新增效果类型只需注册新 Handler |
| `RegistryLoader` trait | 装备定义的 RON 加载模式完全一致 |
| `ModifierRuleRegistry` | 装备标签参与战斗修饰规则（如"重甲减伤"） |

### 2.2 必须填补的缺口

| 缺口 | 严重程度 | 说明 |
|---|---|---|
| 装备槽系统 | P0 | 完全不存在，是装备系统的基础 |
| 装备 Definition / Instance | P0 | 只有 Trait Definition，没有装备层 |
| Equip/Unequip Message | P0 | 穿戴脱卸没有事件驱动 |
| 修饰符 source 语义 | P1 | `BuffInstanceId` 语义不够通用 |
| 标签持久化三层 | P1 | 只有 Trait+Buff 两层，装备需要第三层 |
| TraitCollection 来源追踪 | P1 | 扁平 ID 列表，不知道 trait 来自哪里 |
| Tag 空间扩展 | P2 | 没有装备类型标签 |
| 装备需求/条件 | P2 | 不存在条件检查机制 |

### 2.3 现有系统的限制

1. **`apply_passive_traits()` 只在生成时调用** — 不支持装备穿脱时的动态增减
2. **`TraitCollection` 只存 ID，不追踪来源** — 脱卸装备时无法确定该移除哪些 trait
3. **`rebuild_tags_from_buffs()` 只保留 Trait+Buff** — 装备标签需要第三层
4. **`BuffInstanceId` 语义混淆** — 装备修饰符不是 Buff

---

## 三、架构设计

### 3.1 修饰符来源统一（ModifierSource）

**问题**：当前 `AttributeModifierInstance.source` 是 `BuffInstanceId`，装备修饰符不是 Buff。

**方案**：引入 `ModifierSource` 枚举，替代 `BuffInstanceId` 作为修饰符来源标识。

```rust
/// 修饰符来源：统一标识 Trait / Buff / Equipment
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ModifierSource(pub u64);

impl ModifierSource {
    // ── Trait 区间：u64::MAX - 0 ~ u64::MAX - 999 ──
    pub fn trait_source(index: u64) -> Self {
        Self(u64::MAX - index)
    }

    // ── Equipment 区间：u64::MAX - 1000 ~ u64::MAX - 1999 ──
    pub fn equipment_source(index: u64) -> Self {
        Self(u64::MAX - 1000 - index)
    }

    // ── Buff 区间：1 ~ 999999 ──
    pub fn buff_source(id: BuffInstanceId) -> Self {
        Self(id.0)
    }

    pub fn is_trait(&self) -> bool { self.0 > u64::MAX - 1000 }
    pub fn is_equipment(&self) -> bool { self.0 > u64::MAX - 2000 && self.0 <= u64::MAX - 1000 }
    pub fn is_buff(&self) -> bool { self.0 < u64::MAX - 2000 }
}
```

**迁移策略**：`AttributeModifierInstance.source` 从 `BuffInstanceId` 改为 `ModifierSource`。Buff 系统的 `remove_modifiers_from()` 改为接收 `ModifierSource`。这是破坏性变更，但一次性完成。

### 3.2 标签持久化三层

**问题**：`rebuild_tags_from_buffs()` 只保留 `TraitGrantedTags` + Buff 标签。

**方案**：扩展为三层持久化。

```rust
/// 持久化标签（不被 rebuild 丢失）
#[derive(Component, Default, Debug, Clone)]
pub struct PersistentTags {
    /// Trait 授予的标签（种族/职业/天赋）
    pub from_traits: GameplayTags,
    /// 装备授予的标签（武器/防具/饰品）
    pub from_equipment: GameplayTags,
}
```

**标签重建逻辑**：

```rust
fn rebuild_tags(buffs: &ActiveBuffs, tags: &mut GameplayTags, persistent: &PersistentTags) {
    let mut new_tags = GameplayTags::default();
    // 第一层：Trait 授予（最持久）
    new_tags.0 |= persistent.from_traits.0;
    // 第二层：装备授予（穿脱变化）
    new_tags.0 |= persistent.from_equipment.0;
    // 第三层：Buff 授予（临时）
    for buff in &buffs.instances {
        if buff.remaining_turns == 0 { continue; }
        for tag in &buff.tags { new_tags.add(*tag); }
    }
    tags.0 = new_tags.0;
}
```

**迁移**：`TraitGrantedTags` 合并进 `PersistentTags.from_traits`，所有引用点更新。

### 3.3 TraitCollection 来源追踪

**问题**：`TraitCollection.trait_ids` 是扁平列表，不知道来源。

**方案**：增加来源标记。

```rust
/// Trait 来源
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TraitSource {
    /// 种族/职业/天赋（生成时固定）
    Intrinsic,
    /// 装备授予（穿脱变化）
    Equipment { slot: EquipmentSlot },
}

/// Trait 条目
#[derive(Clone, Debug)]
pub struct TraitEntry {
    pub trait_id: String,
    pub source: TraitSource,
}

/// Trait 集合组件
#[derive(Component, Default, Debug, Clone)]
pub struct TraitCollection {
    pub entries: Vec<TraitEntry>,
}
```

**影响**：`apply_passive_traits()` 需要改为支持增量更新——穿戴装备时添加 `TraitEntry`，脱卸时按 `source` 批量移除。

### 3.4 装备定义（EquipmentDefinition）

```rust
/// 装备槽位
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentSlot {
    MainHand,
    OffHand,
    Head,
    Body,
    Legs,
    Feet,
    Accessory1,
    Accessory2,
}

/// 装备稀有度
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// 装备定义（RON 配置，不可变）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EquipmentDef {
    pub id: String,
    #[serde(default)]
    pub version: u32,
    pub name: String,
    pub description: String,
    pub slot: EquipmentSlot,
    pub rarity: Rarity,
    /// 装备标签（如 sword, fire, heavy, martial）
    pub tags: Vec<TagName>,
    /// 属性修饰
    pub modifiers: Vec<AttributeModifierDef>,
    /// 授予的 Trait（如 flaming_weapon, dragon_bane）
    #[serde(default)]
    pub traits: Vec<String>,
    /// 需求条件
    #[serde(default)]
    pub requirements: Vec<EquipmentRequirement>,
}

/// 装备需求
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EquipmentRequirement {
    /// 需要指定标签（如 martial_weapon 熟练度）
    RequireTag(TagName),
    /// 属性最低要求
    AttributeMin { kind: AttributeKind, value: f32 },
}

/// 装备注册表
#[derive(Resource, Default)]
pub struct EquipmentRegistry {
    defs: HashMap<String, EquipmentDef>,
}
```

**RON 示例**（`assets/equipment/flame_dragon_sword.ron`）：

```ron
(
    id: "flame_dragon_sword",
    version: 1,
    name: "炎龙长剑",
    description: "蕴含龙焰的古老长剑",
    slot: MainHand,
    rarity: Epic,
    tags: [Sword, Fire, Martial, TwoHanded],
    modifiers: [
        (kind: Attack, op: Add, value: 15.0),
        (kind: CritRate, op: Add, value: 0.05),
    ],
    traits: ["flaming_weapon", "dragon_bane"],
    requirements: [
        RequireTag(Martial),
    ],
)
```

### 3.5 装备实例（EquipmentInstance）

```rust
/// 装备实例（运行时，可变）
#[derive(Clone, Debug)]
pub struct EquipmentInstance {
    /// 唯一实例 ID
    pub instance_id: u64,
    /// 指向定义
    pub def_id: String,
    /// 当前耐久度
    pub durability: u32,
    /// 最大耐久度
    pub max_durability: u32,
    /// 强化等级
    pub enhance_level: u32,
    /// 附魔 trait
    pub enchantments: Vec<String>,
}
```

**为什么需要实例层**：
- 同一把"炎龙长剑"，耐久不同
- 强化等级影响属性（+1 → attack+2）
- 附魔增加额外 trait
- 未来存档需要序列化实例

### 3.6 装备槽组件

```rust
/// 装备槽组件：记录每个槽位装备了哪个实例
#[derive(Component, Default, Debug, Clone)]
pub struct EquipmentSlots {
    /// 槽位 → 装备实例 ID
    pub slots: HashMap<EquipmentSlot, u64>,
    /// 下一个实例 ID
    pub next_instance_id: u64,
}

impl EquipmentSlots {
    /// 获取指定槽位的装备实例 ID
    pub fn get(&self, slot: EquipmentSlot) -> Option<u64> {
        self.slots.get(&slot).copied()
    }

    /// 是否已装备指定槽位
    pub fn is_equipped(&self, slot: EquipmentSlot) -> bool {
        self.slots.contains_key(&slot)
    }
}

/// 背包组件：存储未装备的物品实例
#[derive(Component, Default, Debug, Clone)]
pub struct Inventory {
    pub items: Vec<EquipmentInstance>,
    pub capacity: u32,
}
```

### 3.7 Equip / Unequip Message

```rust
/// 穿戴装备消息
#[derive(Message, Debug, Clone)]
pub struct EquipItem {
    pub target_entity: Entity,
    pub instance_id: u64,
}

/// 脱卸装备消息
#[derive(Message, Debug, Clone)]
pub struct UnequipItem {
    pub target_entity: Entity,
    pub slot: EquipmentSlot,
}

/// 装备已穿戴消息（供 UI/日志响应）
#[derive(Message, Debug, Clone)]
pub struct ItemEquipped {
    pub entity: Entity,
    pub slot: EquipmentSlot,
    pub def_id: String,
    pub instance_id: u64,
}

/// 装备已脱卸消息
#[derive(Message, Debug, Clone)]
pub struct ItemUnequipped {
    pub entity: Entity,
    pub slot: EquipmentSlot,
    pub def_id: String,
}
```

---

## 四、穿脱装备核心流程

### 4.1 穿戴流程

```
EquipItem Message
  │
  ├─ 1. 需求检查（标签/属性/槽位）
  │     └─ 不满足 → warn + return
  │
  ├─ 2. 脱卸旧装备（如果槽位已占用）
  │     └─ 调用 unequip_internal()
  │
  ├─ 3. 从背包移除实例
  │
  ├─ 4. 写入 EquipmentSlots
  │
  ├─ 5. 应用装备效果
  │     ├─ 添加修饰符到 Attributes（source = ModifierSource::equipment_source）
  │     ├─ 添加标签到 PersistentTags.from_equipment
  │     ├─ 添加 TraitEntry 到 TraitCollection（source = Equipment { slot }）
  │     └─ 应用 trait 被动效果
  │
  ├─ 6. 重建 GameplayTags（rebuild_tags）
  │
  └─ 7. 发送 ItemEquipped Message
```

### 4.2 脱卸流程

```
UnequipItem Message
  │
  ├─ 1. 检查槽位是否有装备
  │     └─ 无 → return
  │
  ├─ 2. 调用 unequip_internal()
  │     ├─ 移除修饰符（ModifierSource::equipment_source）
  │     ├─ 移除标签（从 PersistentTags.from_equipment）
  │     ├─ 移除 TraitEntry（source = Equipment { slot }）
  │     ├─ 重新应用 trait 被动效果（全量重建）
  │     └─ 重建 GameplayTags
  │
  ├─ 3. 实例放回背包
  │
  ├─ 4. 清除 EquipmentSlots 对应槽位
  │
  └─ 5. 发送 ItemUnequipped Message
```

### 4.3 trait 被动效果的增量更新

**问题**：当前 `apply_passive_traits()` 是一次性全量应用，不支持增量。

**方案**：实现 `rebuild_trait_effects()` 全量重建函数。

```rust
/// 全量重建所有 trait 的被动效果
/// 1. 移除所有 Trait 来源的修饰符
/// 2. 清空 PersistentTags.from_traits
/// 3. 重新遍历 TraitCollection，应用所有 Passive trait
/// 4. 重建 GameplayTags
fn rebuild_trait_effects(
    attrs: &mut Attributes,
    tags: &mut GameplayTags,
    persistent: &mut PersistentTags,
    trait_collection: &TraitCollection,
    trait_registry: &TraitRegistry,
    handler_registry: &TraitEffectHandlerRegistry,
) {
    // 移除所有 Trait 来源的修饰符
    attrs.retain_modifiers(|source| !source.is_trait());

    // 清空 trait 授予的标签
    persistent.from_traits = GameplayTags::default();

    // 重新应用所有 Passive trait
    let mut trait_source_counter = 0u64;
    for entry in &trait_collection.entries {
        if let Some(trait_data) = trait_registry.get(&entry.trait_id) {
            for effect in &trait_data.effects {
                if trait_data.trigger != TraitTrigger::Passive { continue; }
                let source = ModifierSource::trait_source(trait_source_counter);
                // 通过 handler registry 分发
                apply_trait_effect(effect, attrs, &mut persistent.from_traits, source, handler_registry);
                trait_source_counter += 1;
            }
        }
    }

    // 重建 GameplayTags
    // rebuild_tags(buffs, tags, persistent); -- 需要 buffs 参数
}
```

**为什么选全量重建而非增量**：
- Trait 数量通常 < 20，全量重建成本极低
- 全量重建天然幂等，不会出现"漏删"问题
- 增量更新需要精确追踪每个 trait 的每个效果，复杂度高

---

## 五、Tag 空间扩展

当前位掩码空间充足（64 位只用了约 18 位），需要新增装备相关标签：

```rust
impl GameplayTag {
    // ── 装备类型（位 20-23）──
    pub const SWORD: Self = Self(1 << 20);
    pub const AXE: Self = Self(1 << 21);
    pub const BOW: Self = Self(1 << 22);
    pub const STAFF: Self = Self(1 << 23);
    pub const SHIELD: Self = Self(1 << 44);

    // ── 装备属性（位 42-47）──
    pub const HEAVY_ARMOR: Self = Self(1 << 42);
    pub const LIGHT_ARMOR: Self = Self(1 << 43);
    pub const TWO_HANDED: Self = Self(1 << 45);
    pub const MARTIAL: Self = Self(1 << 46);
    pub const SIMPLE: Self = Self(1 << 47);
}
```

`TagName` 枚举同步扩展，`TagCategory` 新增 `Equipment` 分类。

---

## 六、目录结构

遵循 Feature First 铁律，按业务拆分：

```text
equipment/
├── mod.rs              -- 模块声明 + 公共 re-exports
├── plugin.rs           -- EquipmentPlugin
├── definition.rs       -- EquipmentDef, EquipmentRegistry, RegistryLoader
├── instance.rs         -- EquipmentInstance, Inventory
├── slots.rs            -- EquipmentSlot, EquipmentSlots 组件
├── equip.rs            -- 穿戴/脱卸逻辑 + Message 处理
├── requirements.rs     -- 需求检查
├── rebuild.rs          -- 全量重建 trait 效果 + 标签
└── defaults.rs         -- 内置默认装备定义
```

**assets 目录**：

```text
assets/equipment/
├── flame_dragon_sword.ron
├── iron_shield.ron
├── leather_armor.ron
└── ...
```

---

## 七、分阶段实施计划

### 阶段 1：基础设施（ModifierSource + PersistentTags + Tag 扩展）

**改动范围**：`core/attribute`、`core/tag`、`buff/resolve`

1. 引入 `ModifierSource`，替代 `BuffInstanceId` 作为修饰符来源
2. `AttributeModifierInstance.source` 改为 `ModifierSource`
3. 引入 `PersistentTags` 组件，替代 `TraitGrantedTags`
4. `rebuild_tags_from_buffs()` 改为 `rebuild_tags()`，支持三层
5. 扩展 `GameplayTag` 常量 + `TagName` + `TagCategory`
6. 更新所有 Buff 系统的 `remove_modifiers_from()` 调用

**验证**：现有 312 个测试全部通过

### 阶段 2：装备定义 + 注册表

**改动范围**：新建 `equipment/` 模块

1. 实现 `EquipmentDef`、`EquipmentSlot`、`Rarity`、`EquipmentRequirement`
2. 实现 `EquipmentRegistry` + `RegistryLoader`
3. 实现 `EquipmentPlugin`（注册 Resource + Message）
4. 创建 RON 配置文件（3-5 个初始装备）
5. 编写单元测试

**验证**：装备定义可从 RON 加载，注册表查询正确

### 阶段 3：装备实例 + 槽位

**改动范围**：`equipment/`

1. 实现 `EquipmentInstance`、`EquipmentSlots`、`Inventory`
2. 实现 `Unit` 的 Required Components 自动注入 `EquipmentSlots` + `Inventory` + `PersistentTags`
3. 编写单元测试

**验证**：实体可拥有装备槽和背包

### 阶段 4：穿戴/脱卸逻辑

**改动范围**：`equipment/equip.rs`、`equipment/rebuild.rs`、`character/traits/`

1. 实现 `EquipItem` / `UnequipItem` / `ItemEquipped` / `ItemUnequipped` Message
2. 实现 `equip_item()` 系统和 `unequip_item()` 系统
3. 实现 `rebuild_trait_effects()` 全量重建
4. 更新 `TraitCollection` 增加 `TraitEntry` + `TraitSource`
5. 更新 `apply_passive_traits()` 使用新的 `TraitCollection`
6. 编写集成测试

**验证**：穿戴装备 → 属性变化 + 标签变化 + trait 生效；脱卸 → 全部恢复

### 阶段 5：装备需求 + 熟练度

**改动范围**：`equipment/requirements.rs`

1. 实现 `EquipmentRequirement` 检查逻辑
2. 穿戴前验证需求
3. 不满足需求时的处理（拒绝穿戴 / 穿戴但惩罚）
4. 编写测试

**验证**：不满足需求的装备无法穿戴

### 阶段 6：装备与战斗系统联动

**改动范围**：`battle/pipeline/`、`core/modifier_rule.rs`

1. 装备标签参与 `ModifierRule` 修饰（如"重甲减伤"规则）
2. 装备 trait 的 `OnAttack` / `OnHit` 触发器在战斗管线中生效
3. 装备修饰规则 RON 配置
4. 编写集成测试

**验证**：装备效果在战斗中正确触发

### 阶段 7：UI + Debug Viewer

**改动范围**：`ui/`、`debug/`

1. 装备面板 UI（bevy_ui）
2. 背包 UI
3. 装备 Viewer（bevy_egui 调试工具）
4. 穿戴/脱卸的战斗日志

**验证**：可通过 UI 穿脱装备

---

## 八、与 docs/8.md 的对照

| docs/8.md 层次 | 本方案对应 | 状态 |
|---|---|---|
| 第一层：装备定义 | `EquipmentDef` + RON + `EquipmentRegistry` | 阶段 2 |
| 第二层：装备实例 | `EquipmentInstance` + `Inventory` | 阶段 3 |
| 第三层：装备槽 | `EquipmentSlot` + `EquipmentSlots` | 阶段 3 |
| 第四层：Tag 系统 | 复用 `GameplayTag`，扩展装备标签 | 阶段 1 |
| 第五层：Trait 系统 | 复用 `TraitData` + `TraitEffectHandler`，增加来源追踪 | 阶段 4 |
| 第六层：Modifier 系统 | 复用 `AttributeModifierInstance`，引入 `ModifierSource` | 阶段 1 |
| 第七层：关键词系统 | 就是 Tag 系统（位掩码），无需额外抽象 | 已有 |
| 第八层：装备熟练度 | `EquipmentRequirement::RequireTag` | 阶段 5 |
| 第九层：装备能力事件 | TraitTrigger::OnAttack/OnHit + Observer | 阶段 6 |
| 第十层：装备套装 | 套装 = Trait，统计 Tag 激活 Trait | 阶段 6+ |
| 第十一层：传奇装备 | 更多 Trait + Modifier + Rule，无需特殊代码 | 配置层 |

---

## 九、关键设计决策

### 9.1 为什么用 ModifierSource 而非继续用 BuffInstanceId

`BuffInstanceId` 语义是"Buff 实例 ID"，装备修饰符不是 Buff。如果继续复用：
- 代码阅读者会困惑"为什么装备修饰符的 source 是 BuffInstanceId"
- `remove_modifiers_from()` 接收 `BuffInstanceId`，装备脱卸时需要构造一个假的 BuffInstanceId
- 未来可能还有其他来源（天气、称号、祝福），继续用 BuffInstanceId 会越来越混乱

`ModifierSource` 一次性解决语义问题，且向后兼容（Buff 区间不变）。

### 9.2 为什么选全量重建而非增量更新 trait

装备穿脱时需要增删 trait 效果，有两种方案：

**增量**：穿戴时添加效果，脱卸时精确移除。需要追踪每个 trait 的每个效果 ID。
**全量重建**：移除所有 Trait 来源效果，重新应用所有当前 trait。

选择全量重建的理由：
- Trait 数量通常 < 20，全量重建成本可忽略
- 全量重建天然幂等，不会出现"漏删"问题
- 增量更新需要精确追踪每个效果的 source ID，复杂度高
- 项目铁律："先正确，再优化"

### 9.3 为什么装备实例不做成 Entity

docs/8.md 没有明确说装备实例是否做 Entity。本方案选择**组件内存储**（`Inventory.items: Vec<EquipmentInstance>`），理由：

- SRPG 装备实例数量有限（通常 < 100），不需要 ECS 的并行查询能力
- 装备实例的生命周期与角色绑定，不需要独立存在
- 避免大量 Entity 带来的 World 膨胀
- 如果未来需要装备做 Entity（如掉落物在地图上），可以在地图层单独处理

### 9.4 为什么套装不特殊处理

套装本质是"统计同类 Tag 数量 → 达到阈值 → 激活 Trait"。

例如"龙套装 2 件"：
- 装备 A 有 tag `dragon_set`，装备 B 也有 tag `dragon_set`
- 系统统计 `dragon_set` 出现 2 次
- 激活 `dragon_set_2` trait
- trait 授予 `dragon_power` 效果

这完全在现有 Trait + Tag 体系内，无需特殊代码。只需一个 `check_set_bonuses()` 系统在穿脱装备时调用。

---

## 十、风险与缓解

| 风险 | 缓解措施 |
|---|---|
| ModifierSource 迁移破坏现有代码 | 一次性迁移，312 个测试覆盖 |
| 全量重建 trait 性能问题 | Trait 数量 < 20，成本可忽略；如果未来 > 100 再考虑增量 |
| Tag 位掩码空间不足 | 64 位只用了约 18 位，剩余空间充足 |
| 装备穿脱属性闪烁 | 先脱旧再穿新，在同一个系统调用内完成 |
| 装备实例序列化 | `EquipmentInstance` derive Serialize/Deserialize，存档时序列化 `Inventory` |
