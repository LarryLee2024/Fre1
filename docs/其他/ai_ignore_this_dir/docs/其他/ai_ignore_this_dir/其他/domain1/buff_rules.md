# Buff 领域规则 (Buff Rules)

## 1. 领域概述

Buff 系统管理所有临时状态效果，包括增益（Buff）、减益（Debuff）、持续伤害（DoT）、持续治疗（HoT）和晕眩（Stun）。遵循 **Definition / Instance 分离**原则，BuffData 为静态定义，BuffInstance 为运行时实例。

### 核心原则

- **Definition / Instance 分离**：`BuffData`（注册表中的静态数据）与 `BuffInstance`（运行时实例）分离
- **Rule / Content 分离**：代码负责规则（施加、移除、结算），RON 配置负责内容（Buff 数值）
- **统一修饰符管线**：Buff 通过 `ModifierSource::buff_source` 进入属性修饰管线
- **标签三层架构**：Trait 授予 → 装备授予 → Buff 授予，按优先级重建

---

## 2. BuffData — Buff 定义

```rust
pub struct BuffData {
    pub id: String,                    // Buff 唯一标识
    pub name: String,                  // 显示名称
    pub default_duration: u32,         // 默认持续回合
    pub modifiers: Vec<AttributeModifierDef>,  // 属性修饰符
    pub tags: Vec<GameplayTag>,        // 标签
    pub dot_damage: i32,               // 每回合 DoT 伤害
    pub hot_heal: i32,                 // 每回合 HoT 治疗
    pub is_stun: bool,                 // 是否晕眩
    pub is_cleanse: bool,              // 是否净化
    pub is_buff: bool,                 // true=增益, false=减益
}
```

### 2.1 Buff 分类

| 分类 | is_buff | 说明 |
|------|:-------:|------|
| 增益 (Buff) | true | 正面效果，如攻+5、防+5 |
| 减益 (Debuff) | false | 负面效果，如攻-5、灼烧、中毒、晕眩 |

`is_debuff()` = `!is_buff`

### 2.2 Buff 特殊类型

| 类型 | 标识 | 行为 |
|------|------|------|
| 晕眩 | `is_stun = true` | 单位本回合无法行动，消耗后移除 |
| 净化 | `is_cleanse = true` | 立即驱散所有 Debuff，不创建实例 |
| DoT | `dot_damage > 0` | 每回合结算伤害 |
| HoT | `hot_heal > 0` | 每回合结算治疗 |

---

## 3. BuffDef — RON 配置格式

```ron
(
    version: 1,                    // 配置版本（可选，默认 0）
    id: "burn",                    // Buff ID
    name: "灼-2",                  // 显示名称
    default_duration: 2,           // 默认持续回合
    modifiers: [
        (kind: Defense, op: Add, value: -2.0),
    ],
    tags: [DEBUFF, BURN, FIRE],    // 标签
    dot_damage: 2,                 // DoT 伤害
    hot_heal: 0,                   // HoT 治疗
    is_stun: false,                // 晕眩
    is_cleanse: false,             // 净化
    is_buff: false,                // 增益/减益
)
```

**规则**：
- `version` 字段可选，缺失时默认为 0（向后兼容）
- `tags` 使用 `TagName` 枚举，反序列化时转为 `GameplayTag`

---

## 4. BuffRegistry — Buff 注册表

```rust
#[derive(Resource)]
pub struct BuffRegistry {
    pub buffs: HashMap<String, BuffData>,
}
```

### 4.1 内置默认 Buff

| ID | 名称 | 持续 | 修饰 | 特殊 | 类型 |
|----|------|------|------|------|------|
| `attack_up` | 攻+5 | 3 | Attack +5 | — | Buff |
| `attack_down` | 攻-5 | 3 | Attack -5 | — | Debuff |
| `defense_up` | 防+5 | 3 | Defense +5 | — | Buff |
| `defense_down` | 防-5 | 3 | Defense -5 | — | Debuff |
| `burn` | 灼-2 | 2 | Defense -2 | DoT=2, FIRE | Debuff |
| `poison` | 毒-3 | 3 | — | DoT=3 | Debuff |
| `regen` | 愈+4 | 3 | — | HoT=4 | Buff |
| `stun` | 晕眩 | 1 | — | Stun | Debuff |

### 4.2 数据加载

通过 `RegistryLoader` trait 实现 RON 文件加载：
- 加载目录：`assets/buffs/`
- `BuffDef` 通过 `From<BuffDef> for BuffData` 转换
- 注册表幂等：重复调用 `register_defaults()` 不会重复注册

---

## 5. BuffInstance — 运行时实例

```rust
pub struct BuffInstance {
    pub instance_id: BuffInstanceId,   // 唯一实例 ID
    pub buff_id: String,               // 对应 BuffData ID
    pub name: String,                  // 显示名称
    pub remaining_turns: u32,          // 剩余回合
    pub source_entity: Option<Entity>, // 来源实体
    pub tags: Vec<GameplayTag>,        // 标签副本
    pub is_buff: bool,                 // 增益/减益
    pub dot_damage: i32,               // DoT 伤害
    pub hot_heal: i32,                 // HoT 治疗
}
```

---

## 6. ActiveBuffs — 活跃 Buff 列表

```rust
#[derive(Component)]
pub struct ActiveBuffs {
    pub instances: Vec<BuffInstance>,
    next_id: u64,
}
```

### 6.1 核心操作

| 方法 | 说明 |
|------|------|
| `add(buff)` | 添加实例，同源同 buff_id 刷新持续时间 |
| `remove(instance_id)` | 移除指定实例 |
| `tick()` | 先移除 remaining=0 的，再递减所有 |
| `next_instance_id()` | 生成唯一实例 ID |

### 6.2 查询方法

| 方法 | 说明 |
|------|------|
| `is_stunned()` | 是否有活跃晕眩 |
| `consume_stun()` | 消耗晕眩，返回是否原本晕眩 |
| `dot_damage()` | 汇总所有活跃 DoT 伤害 |
| `hot_heal()` | 汇总所有活跃 HoT 治疗 |
| `remove_debuffs()` | 移除所有 Debuff |

### 6.3 同源刷新规则

```
添加 Buff 时：
  如果 source_entity 相同 && buff_id 相同 → 刷新 remaining_turns（不新增实例）
  否则 → 新增实例
```

**规则**：不同来源的同 ID Buff 可以共存。

---

## 7. 施加与移除

### 7.1 apply_buff — 施加 Buff

```
1. Cleanse 特殊处理：立即驱散所有 Debuff，返回
2. 生成 instance_id
3. 构建 BuffInstance
4. 添加修饰符到 Attributes（ModifierSource::buff_source）
5. 添加标签到 GameplayTags
6. 添加实例到 ActiveBuffs
```

### 7.2 remove_buff — 移除 Buff

```
1. 从 ActiveBuffs 移除实例
2. 移除修饰符（ModifierSource::buff_source）
3. 移除标签（仅当没有其他 Buff 提供相同标签时）
```

**标签安全移除规则**：移除 Buff 时，检查剩余活跃 Buff 是否仍提供该标签，只有无人提供时才从 GameplayTags 移除。

### 7.3 remove_all_debuffs — 移除所有 Debuff

遍历所有 `is_debuff()` 的实例，逐个调用 `remove_buff()`。

---

## 8. 持续效果结算

### 8.1 resolve_status_effects — 回合开始结算

**触发时机**：`OnEnter(TurnPhase::SelectUnit)`，通过 `NeedsResolve` 标记确保每回合只结算一次。

**结算顺序**：

```
1. 晕眩结算
   - 标记 unit.acted = true
   - 移除所有 STUN 标签的 Buff
   - 发送 StunApplied Message

2. DoT 结算
   - 汇总 dot_damage
   - 扣血：hp = max(0, hp - dot)
   - 发送 DotApplied Message
   - DoT 致死判定：hp <= 0 → 插入 Dead + CharacterDied

3. HoT 结算
   - 汇总 hot_heal
   - 回血：hp = min(max_hp, hp + hot)
   - 发送 HotApplied Message

4. tick_buffs
   - 递减所有 Buff 持续时间
   - 清理过期 Buff 的修饰符
   - 重建 GameplayTags

5. 技能冷却 tick
```

### 8.2 tick_buffs — 持续时间递减

```
1. 收集即将过期的 Buff 的 ModifierSource（remaining_turns <= 1）
2. 调用 ActiveBuffs::tick()（先移除 remaining=0，再递减）
3. 清理过期 Buff 的修饰符
4. 重建 GameplayTags
```

### 8.3 rebuild_tags — 标签重建

标签三层架构（按优先级叠加）：

```
第一层：Trait 授予（persistent.from_traits）
第二层：装备授予（persistent.from_equipment）
第三层：Buff 授予（活跃 Buff 的 tags，跳过 remaining_turns=0）
```

**规则**：
- 每次 tick 后完全重建 GameplayTags，不增量更新
- Trait 和装备授予的标签不受 Buff 过期影响
- 过期 Buff（remaining_turns=0）的标签不参与重建

---

## 9. Buff 生命周期

```
施加（apply_buff）
  ↓
每回合结算（resolve_status_effects）
  ├─ 晕眩：消耗后移除
  ├─ DoT：扣血 + 死亡判定
  ├─ HoT：回血
  └─ tick：递减持续时间
  ↓
过期（remaining_turns = 0）
  ├─ 修饰符清理
  ├─ 标签重建
  └─ 下次 tick 移除实例
```

---

## 10. 关键约束

1. **Cleanse 不创建实例**：净化 Buff 立即驱散所有 Debuff，返回 `BuffInstanceId(0)`
2. **同源刷新不新增**：同 source_entity + 同 buff_id 只刷新持续时间
3. **不同源可共存**：不同来源的同 ID Buff 各自独立
4. **晕眩消耗后移除**：`consume_stun()` 移除所有 STUN 标签 Buff
5. **DoT 可致死**：DoT 伤害可导致角色死亡
6. **HoT 不超过 MaxHp**：`min(max_hp, hp + hot)`
7. **标签安全移除**：移除 Buff 时检查共享标签
8. **结算每回合一次**：通过 `NeedsResolve` 标记保证
9. **tick 两步走**：先移除 remaining=0 的，再递减所有
10. **修饰符来源追踪**：通过 `BuffInstanceId.to_modifier_source()` 关联
