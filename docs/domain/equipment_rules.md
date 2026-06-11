# 装备领域规则 (Equipment Rules)

## 1. 领域概述

装备系统管理角色的武器、防具和饰品，通过穿脱操作影响属性、标签和 Trait。遵循 **Definition / Instance 分离**和 **装备 = Modifier + Trait** 原则。

### 核心原则

- **Definition / Instance 分离**：`EquipmentDef`（不可变定义）与 `EquipmentInstance`（可变实例）分离
- **装备 = Modifier + Trait**：装备通过属性修饰符和 Trait 授予影响角色
- **Rule / Content 分离**：新增装备修改 RON 配置，不修改逻辑代码
- **标签三层架构**：Trait 授予 → 装备授予 → Buff 授予

---

## 2. EquipmentDef — 装备定义

```rust
pub struct EquipmentDef {
    pub id: String,                           // 装备唯一标识
    pub name: String,                         // 显示名称
    pub description: String,                  // 描述
    pub slot: EquipmentSlot,                  // 装备槽位
    pub rarity: Rarity,                       // 稀有度
    pub tags: Vec<TagName>,                   // 装备标签（SWORD, FIRE, MARTIAL 等）
    pub modifiers: Vec<AttributeModifierDef>, // 属性修饰符
    pub traits: Vec<String>,                  // 授予的 Trait ID
    pub requirements: Vec<EquipmentRequirement>, // 需求条件
    pub weight: f32,                          // 重量
}
```

### 2.1 EquipmentSlot — 装备槽位

| 槽位 | 标签 | 说明 |
|------|------|------|
| `MainHand` | 主手 | 武器 |
| `OffHand` | 副手 | 盾牌/副武器 |
| `Head` | 头部 | 头盔 |
| `Body` | 身体 | 铠甲 |
| `Legs` | 腿部 | 腿甲 |
| `Feet` | 脚部 | 靴子 |
| `Accessory1` | 饰品1 | 戒指/项链 |
| `Accessory2` | 饰品2 | 戒指/项链 |

### 2.2 Rarity — 稀有度

| 稀有度 | 标签 | 排序 |
|--------|------|------|
| `Common` | 普通 | 1 |
| `Uncommon` | 精良 | 2 |
| `Rare` | 稀有 | 3 |
| `Epic` | 史诗 | 4 |
| `Legendary` | 传说 | 5 |

### 2.3 EquipmentRequirement — 装备需求

| 需求 | 参数 | 说明 |
|------|------|------|
| `RequireTag(tag)` | TagName | 需要拥有指定标签（如 MARTIAL） |
| `AttributeMin { kind, value }` | AttributeKind, f32 | 属性最低要求 |

---

## 3. EquipmentInstance — 装备实例

```rust
pub struct EquipmentInstance {
    pub instance_id: u64,         // 唯一实例 ID
    pub def_id: String,           // 指向定义 ID
    pub durability: u32,          // 当前耐久度
    pub max_durability: u32,      // 最大耐久度
    pub enhance_level: u32,       // 强化等级
    pub enchantments: Vec<String>, // 附魔 trait
}
```

**规则**：
- 同一装备定义可以有多个实例
- 实例拥有独立的耐久、强化等级、附魔
- 创建时 durability = max_durability

---

## 4. EquipmentSlots — 装备槽位组件

```rust
#[derive(Component)]
pub struct EquipmentSlots {
    pub slots: HashMap<EquipmentSlot, (u64, String)>,  // 槽位 → (实例ID, 定义ID)
    pub next_instance_id: u64,
}
```

| 方法 | 说明 |
|------|------|
| `get(slot)` | 获取槽位实例 ID |
| `get_def_id(slot)` | 获取槽位定义 ID |
| `is_equipped(slot)` | 槽位是否已装备 |
| `equip(slot, instance_id, def_id)` | 装备，返回被替换的旧装备 |
| `unequip(slot)` | 卸下，返回被卸下的装备 |
| `equipped_slots()` | 获取所有已装备槽位 |

**规则**：
- 装备到已占用槽位时，自动替换旧装备
- 卸下空槽位返回 None

---

## 5. 装备需求检查

### 5.1 check_equipment_requirements

```rust
pub fn check_equipment_requirements(def, attrs, tags) -> RequirementCheckResult
```

**检查顺序**：
1. 遍历 `def.requirements`，按定义顺序
2. `RequireTag`：检查 `tags.has(tag_name.to_tag())`
3. `AttributeMin`：检查 `attrs.get(kind) >= value`
4. 第一个不满足即返回 `Failed(reason)`
5. 全部满足返回 `Satisfied`

### 5.2 RequirementCheckResult

| 结果 | 说明 |
|------|------|
| `Satisfied` | 满足所有需求 |
| `Failed(reason)` | 不满足，附带原因 |

---

## 6. 穿脱流程

### 6.1 EquipItem Message

```rust
#[derive(Message)]
pub struct EquipItem {
    pub target_entity: Entity,
    pub instance_id: u64,
}
```

**穿戴流程**：

```
1. 从背包查找装备实例
2. 从注册表查找装备定义
3. 需求检查 → 失败发送 EquipFailed
4. 槽位已占用 → 先脱卸旧装备
5. 从背包移除
6. 装备到槽位
7. 应用装备效果（修饰符 + 标签 + Trait）
8. 重建 Trait 效果
9. 重建 GameplayTags
10. 发送 ItemEquipped Message
```

### 6.2 UnequipItem Message

```rust
#[derive(Message)]
pub struct UnequipItem {
    pub target_entity: Entity,
    pub slot: EquipmentSlot,
}
```

**脱卸流程**：

```
1. 检查槽位是否有装备
2. 移除装备修饰符
3. 移除装备授予的标签
4. 移除装备授予的 Trait
5. 清除槽位
6. 创建 ItemStack 放回背包
7. 重建 Trait 效果
8. 重建 GameplayTags
9. 发送 ItemUnequipped Message
```

### 6.3 Message 类型

| Message | 方向 | 说明 |
|---------|------|------|
| `EquipItem` | UI → 系统 | 请求穿戴装备 |
| `UnequipItem` | UI → 系统 | 请求脱卸装备 |
| `ItemEquipped` | 系统 → UI | 装备已穿戴 |
| `ItemUnequipped` | 系统 → UI | 装备已脱卸 |
| `EquipFailed` | 系统 → UI | 穿戴失败 |

---

## 7. 装备效果应用

### 7.1 apply_equipment_effects

```
1. 添加修饰符到 Attributes（ModifierSource::equipment_source）
2. 添加标签到 PersistentTags.from_equipment
3. 添加 Trait 到 TraitCollection（TraitSource::Equipment { slot }）
```

### 7.2 修饰符来源

```rust
ModifierSource::equipment_source(instance_id)  // Equipment 区间
```

装备修饰符通过 `ModifierSource` 追踪，脱卸时按来源精确移除。

### 7.3 标签持久化

装备授予的标签存入 `PersistentTags.from_equipment`，不受 Buff 过期影响。

---

## 8. Trait 重建

### 8.1 rebuild_trait_effects

装备穿脱后调用，确保 Trait 授予的标签和属性修饰符正确：

```
1. 清除所有 Trait 来源的修饰符
2. 清除 Trait 授予的标签（persistent.from_traits）
3. 重新应用所有 Passive Trait：
   - 授予标签到 persistent.from_traits
   - 授予属性修饰符（ModifierSource::trait_source）
```

### 8.2 rebuild_tags_from_components

从组件重建 GameplayTags（Trait + Equipment 层，不含 Buff 层）：

```
new_tags = persistent.from_traits | persistent.from_equipment
```

Buff 层由 `resolve_status_effects` 中的 `rebuild_tags` 管理。

---

## 9. EquipmentRegistry — 装备注册表

```rust
#[derive(Resource)]
pub struct EquipmentRegistry {
    defs: HashMap<String, EquipmentDef>,
}
```

### 9.1 内置默认装备

| ID | 名称 | 槽位 | 稀有度 | 修饰 | 标签 | Trait |
|----|------|------|--------|------|------|-------|
| `iron_sword` | 铁剑 | MainHand | Common | Attack +3 | SWORD, MELEE, MARTIAL | — |
| `leather_armor` | 皮甲 | Body | Common | Defense +2 | LIGHT_ARMOR | — |
| `flame_dragon_sword` | 炎龙长剑 | MainHand | Epic | Attack +15, CritRate +5 | SWORD, FIRE, MARTIAL, TWO_HANDED | flaming_weapon, dragon_bane |
| `iron_shield` | 铁盾 | OffHand | Common | Defense +3 | SHIELD | — |
| `mage_staff` | 法师法杖 | MainHand | Uncommon | MagicAttack +5 | STAFF, SIMPLE | — |

### 9.2 数据加载

- 加载目录：`assets/equipment/`
- 通过 `RegistryLoader` trait 实现 RON 文件加载
- 注册表幂等：重复调用不会重复注册

---

## 10. RON 配置格式

```ron
(
    version: 1,
    id: "flame_dragon_sword",
    name: "炎龙长剑",
    description: "蕴含龙焰的古老长剑",
    slot: MainHand,
    rarity: Epic,
    tags: [SWORD, FIRE, MARTIAL, TWO_HANDED],
    modifiers: [
        (kind: Attack, op: Add, value: 15.0),
        (kind: CritRate, op: Add, value: 5.0),
    ],
    traits: ["flaming_weapon", "dragon_bane"],
    requirements: [RequireTag(MARTIAL)],
    weight: 5.0,
)
```

**规则**：
- `version`、`description`、`tags`、`modifiers`、`traits`、`requirements`、`weight` 均可选（有默认值）
- 旧配置无 `version` 字段时默认为 0

---

## 11. 关键约束

1. **穿脱必须走 Message**：`EquipItem` / `UnequipItem`，不直接调用函数
2. **需求检查先于穿戴**：不满足需求发送 `EquipFailed`，不执行穿戴
3. **替换旧装备**：槽位已占用时先脱卸再穿戴
4. **脱卸放回背包**：创建 `ItemStack` 放回 `Container`
5. **修饰符来源追踪**：通过 `ModifierSource::equipment_source(instance_id)` 精确关联
6. **标签分层管理**：装备标签存入 `persistent.from_equipment`，不与 Buff 混淆
7. **Trait 重建在穿脱后**：确保 Passive Trait 效果正确
8. **GameplayTags 重建不含 Buff 层**：装备层只管 Trait + Equipment，Buff 层由 resolve 管理
9. **注册表幂等**：重复调用 `register_defaults()` 不会重复注册
10. **实例独立**：同一装备定义可有多个实例，各自拥有独立状态
