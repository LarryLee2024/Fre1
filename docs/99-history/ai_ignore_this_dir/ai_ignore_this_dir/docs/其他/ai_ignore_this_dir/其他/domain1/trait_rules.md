# Trait 扩展体系领域规则 (Trait Rules)

## 1. 领域概述

Trait 系统是 SRPG 角色能力的统一扩展机制。种族、职业、天赋、装备、Buff 均通过 Trait + Modifier 管线影响角色。遵循 **Trait 表示能力，不表示分类**和 **组合优于继承**原则。

### 核心原则

- **Trait 表示能力，不表示分类**：Trait 授予标签、属性修饰或触发 Buff
- **组合优于继承**：角色由 Trait、Modifier、能力组合构成
- **统一扩展机制**：种族 = Trait 集合，职业 = Trait 集合，天赋 = 特殊 Trait，装备 = Modifier + Trait，Buff = 临时 Trait
- **Handler 分发**：新增效果类型只需实现 `TraitEffectHandler` 并注册

---

## 2. TraitTrigger — 触发时机

| 触发器 | 说明 | 效果类型 |
|--------|------|----------|
| `Passive` | 被动，始终生效 | GrantTag, ModifyAttribute |
| `OnTurnStart` | 回合开始时触发 | ApplyBuff |
| `OnTurnEnd` | 回合结束时触发 | ApplyBuff |
| `OnAttack` | 攻击时触发 | ApplyBuff |
| `OnHit` | 被攻击时触发 | ApplyBuff |
| `OnKill` | 击杀时触发 | ApplyBuff |

**规则**：
- Passive Trait 在角色生成时应用，穿脱装备时重建
- 触发型 Trait 在对应时机将 ApplyBuff 效果推入 EffectQueue

---

## 3. TraitEffect — 效果类型

| 效果 | 参数 | 触发时机 | 说明 |
|------|------|----------|------|
| `GrantTag(tag)` | GameplayTag | Passive | 授予标签 |
| `ModifyAttribute(mod_def)` | AttributeModifierDef | Passive | 属性修饰 |
| `ApplyBuff { buff_id, duration }` | String, u32 | 触发型 | 触发时施加 Buff |

### 3.1 效果与触发器的对应关系

| 触发器 | 可用效果 |
|--------|----------|
| Passive | GrantTag, ModifyAttribute |
| OnAttack/OnHit/OnKill/OnTurnStart/OnTurnEnd | ApplyBuff |

**规则**：
- Passive Trait 的 GrantTag 和 ModifyAttribute 在 `apply_passive_traits()` 中应用
- 触发型 Trait 的 ApplyBuff 在 `trigger_traits()` 中推入 EffectQueue
- ApplyBuff 效果在 Passive 触发器下无意义（无触发时机）

### 3.2 type_name — 效果类型标识

```rust
pub fn type_name(&self) -> &'static str
```

返回效果变体名，用于 Handler 查找：
- `"GrantTag"` → GrantTagHandler
- `"ModifyAttribute"` → ModifyAttributeHandler
- `"ApplyBuff"` → ApplyBuffHandler

---

## 4. TraitEffectHandler — 效果处理器

### 4.1 Handler Trait

```rust
pub trait TraitEffectHandler: Send + Sync + 'static {
    fn type_name(&self) -> &'static str;
    fn granted_tags(&self, effect: &TraitEffect) -> Vec<GameplayTag>;
    fn attribute_modifiers<'a>(&self, effect: &'a TraitEffect) -> Vec<&'a AttributeModifierDef>;
}
```

### 4.2 内置 Handler

| Handler | type_name | granted_tags | attribute_modifiers |
|---------|-----------|-------------|---------------------|
| `GrantTagHandler` | "GrantTag" | 提取 GameplayTag | 空 |
| `ModifyAttributeHandler` | "ModifyAttribute" | 空 | 提取 AttributeModifierDef |
| `ApplyBuffHandler` | "ApplyBuff" | 空 | 空 |

### 4.3 TraitEffectHandlerRegistry

```rust
#[derive(Resource)]
pub struct TraitEffectHandlerRegistry {
    handlers: HashMap<&'static str, Box<dyn TraitEffectHandler>>,
}
```

**规则**：
- 通过 `type_name` 查找 Handler
- 新增效果类型只需实现 Handler 并注册，无需修改 TraitData
- 默认注册三个内置 Handler

---

## 5. TraitData — Trait 运行时数据

```rust
pub struct TraitData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub trigger: TraitTrigger,
    pub effects: Vec<TraitEffect>,
}
```

### 5.1 TraitData 方法

| 方法 | 说明 |
|------|------|
| `granted_tags(handlers)` | 通过 Handler 分发收集授予的标签 |
| `attribute_modifiers(handlers)` | 通过 Handler 分发收集属性修饰 |

### 5.2 TraitDefinition — RON 配置

```rust
pub struct TraitDefinition {
    pub version: u32,           // 可选，默认 0
    pub id: String,
    pub name: String,
    pub description: String,
    pub trigger: TraitTrigger,
    pub effects: Vec<TraitEffectDef>,  // TagName 替代 GameplayTag
}
```

---

## 6. TraitSource — 来源追踪

```rust
pub enum TraitSource {
    Intrinsic,                      // 内在来源（种族/职业/天赋）
    Equipment { slot: EquipmentSlot }, // 装备来源（记录具体槽位）
}
```

**规则**：
- Intrinsic：种族、职业、天赋授予的 Trait
- Equipment：装备授予的 Trait，记录具体槽位
- 脱卸装备时按 `TraitSource::Equipment { slot }` 精确移除

---

## 7. TraitCollection — Trait 集合组件

```rust
#[derive(Component)]
pub struct TraitCollection {
    pub entries: Vec<TraitEntry>,
}
```

### 7.1 TraitEntry

```rust
pub struct TraitEntry {
    pub trait_id: String,
    pub source: TraitSource,
}
```

### 7.2 核心操作

| 方法 | 说明 |
|------|------|
| `new(trait_ids)` | 从 ID 列表创建（全部 Intrinsic） |
| `has(trait_id)` | 是否拥有指定 Trait |
| `add_entry(trait_id, source)` | 添加一条 Trait |
| `remove_by_source(source)` | 按来源移除，返回被移除的 ID 列表 |
| `trait_ids()` | 获取所有 trait_id（去重） |

---

## 8. apply_passive_traits — 被动 Trait 应用

```rust
pub fn apply_passive_traits(
    trait_collection, registry, handlers
) -> (GameplayTags, Vec<AttributeModifierInstance>)
```

**流程**：

```
1. 遍历 TraitCollection.entries
2. 跳过非 Passive 触发的 Trait
3. 通过 Handler 收集 granted_tags
4. 通过 Handler 收集 attribute_modifiers
5. 每个 Trait 分配独立的 ModifierSource::trait_source(index)
6. 返回 (标签集合, 修饰符列表)
```

**修饰符来源**：
```rust
ModifierSource::trait_source(0)  // 第一个 Trait
ModifierSource::trait_source(1)  // 第二个 Trait
// ...
```

Trait 区间：`u64::MAX ~ u64::MAX - 999`，避免与 Buff/Equipment 区间冲突。

---

## 9. TraitRegistry — Trait 注册表

```rust
#[derive(Resource)]
pub struct TraitRegistry {
    pub traits: HashMap<String, TraitData>,
}
```

### 9.1 内置默认 Trait

| ID | 名称 | 触发 | 效果 |
|----|------|------|------|
| `warrior_mastery` | 战士精通 | Passive | GrantTag(WARRIOR), GrantTag(MELEE) |
| `archer_mastery` | 弓手精通 | Passive | GrantTag(ARCHER), GrantTag(RANGED) |
| `mage_mastery` | 法师精通 | Passive | GrantTag(MAGE) |
| `fire_affinity` | 火焰亲和 | Passive | GrantTag(FIRE) |
| `heavy_armor` | 重甲 | Passive | ModifyAttribute(Defense +3) |

### 9.2 数据加载

- 加载目录：`assets/traits/`
- 通过 `RegistryLoader` trait 实现 RON 文件加载
- `TraitDefinition` 通过 `From<TraitDefinition> for TraitData` 转换
- 注册表幂等

---

## 10. Trait 重建流程

装备穿脱后触发 Trait 重建（`rebuild_trait_effects`）：

```
1. 清除所有 Trait 来源的修饰符（attrs.remove_trait_modifiers()）
2. 清除 Trait 授予的标签（persistent.from_traits = default）
3. 重新应用所有 Passive Trait：
   - 遍历 TraitCollection.entries
   - 跳过非 Passive 触发
   - 授予标签到 persistent.from_traits
   - 授予属性修饰符（ModifierSource::trait_source）
4. 重建 GameplayTags（Trait + Equipment 层）
```

---

## 11. RON 配置格式

### 11.1 被动 Trait

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

### 11.2 触发型 Trait

```ron
(
    id: "leader_aura",
    name: "领袖光环",
    description: "回合开始时为友军施加增益",
    trigger: OnTurnStart,
    effects: [
        ApplyBuff(buff_id: "attack_up", duration: 1),
    ],
)
```

---

## 12. 关键约束

1. **Trait 表示能力不表示分类**：不用于模拟继承树
2. **Passive 效果仅 GrantTag 和 ModifyAttribute**：ApplyBuff 在 Passive 下无意义
3. **触发型效果仅 ApplyBuff**：GrantTag/ModifyAttribute 是永久效果，不需要触发
4. **Handler 分发扩展**：新增效果类型只需实现 Handler 并注册
5. **来源精确追踪**：Intrinsic 和 Equipment { slot } 区分来源
6. **装备穿脱触发重建**：确保 Trait 效果与装备状态一致
7. **修饰符独立 source**：每个 Trait 分配独立 ModifierSource，支持精确移除
8. **Trait 区间隔离**：`u64::MAX ~ u64::MAX - 999`，不与 Buff/Equipment 冲突
9. **注册表幂等**：重复调用不会重复注册
10. **触发器在战斗管线中调用**：OnAttack/OnHit/OnKill 由 `trigger_traits()` 处理
