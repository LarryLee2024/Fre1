# 技能领域规则 (Skill Rules)

## 1. 领域概述

技能系统是战斗系统的核心驱动，负责定义技能数据、管理技能槽位、追踪冷却状态、校验使用条件，以及预览技能效果。遵循 **Definition / Instance 分离**和 **Rule / Content 分离**原则。

### 核心原则

- **Definition / Instance 分离**：`SkillData`（静态定义）与 `SkillCooldowns`（运行时状态）分离
- **Rule / Content 分离**：代码负责规则（条件校验、冷却逻辑），RON 配置负责内容（技能数值）
- **数据驱动**：新增技能优先修改 RON 配置，不修改逻辑代码
- **纯函数校验**：`can_use()` 是纯函数，不修改任何状态

---

## 2. SkillData — 技能定义

```rust
pub struct SkillData {
    pub id: String,                    // 技能唯一标识
    pub name: String,                  // 显示名称
    pub description: String,           // 描述文本
    pub cost_mp: i32,                  // MP 消耗
    pub range: u32,                    // 射程（0 = 使用单位基础攻击范围）
    pub targeting: SkillTargeting,     // 目标类型
    pub effects: Vec<EffectDef>,       // 效果列表
    pub tags: Vec<GameplayTag>,        // 技能标签
    pub conditions: Vec<SkillCondition>, // 使用条件
    pub cooldown: u32,                 // 冷却回合数
    pub priority: u32,                 // AI 决策优先级
}
```

### 2.1 SkillTargeting — 目标类型

| 类型 | 说明 | 需要选择目标 |
|------|------|:---:|
| `SingleEnemy` | 单个敌方单位 | Yes |
| `SingleAlly` | 单个友方单位 | Yes |
| `SelfOnly` | 自身 | No |
| `AoeEnemies` | 自身周围敌方（范围由 range 决定） | No |
| `AoeAllies` | 自身周围友方 | No |
| `NoTarget` | 无需目标 | No |

### 2.2 EffectDef — 效果定义

| 类型 | 字段 | 说明 |
|------|------|------|
| `Damage` | `multiplier, ignore_def_percent` | 伤害效果 |
| `Heal` | `amount` | 治疗效果 |
| `ApplyBuff` | `buff_id, duration` | 施加 Buff |
| `Cleanse` | — | 净化所有 Debuff |

**规则**：
- 一个技能可以有多个效果（如火球 = 伤害 + 施加灼烧）
- 效果通过 `EffectHandlerRegistry` 分发处理

---

## 3. SkillCondition — 使用条件

### 3.1 运行时条件

| 条件 | 参数 | 说明 |
|------|------|------|
| `MpCost(cost)` | i32 | MP 不足时不可使用 |
| `RequireTag(tag)` | GameplayTag | 自身缺少标签时不可使用 |
| `TargetRequireTag(tag)` | GameplayTag | 目标缺少标签时不可使用 |
| `HpBelow(pct)` | f32 (0.0~1.0) | HP 不低于阈值时不可使用 |
| `HpAbove(pct)` | f32 (0.0~1.0) | HP 不高于阈值时不可使用 |

### 3.2 SkillConditionDef — RON 反序列化用

使用 `TagName` 替代 `GameplayTag`，通过 `From<SkillConditionDef> for SkillCondition` 转换。

### 3.3 can_use() — 条件校验

```rust
pub fn can_use(&self, source_attrs, source_tags, target_tags, current_cooldown) -> Result<(), SkillUseError>
```

**校验顺序**：
1. 冷却检查（current_cooldown > 0 → OnCooldown）
2. 逐条检查 conditions（按定义顺序，短路返回第一个失败条件）
3. 全部通过 → Ok(())

**规则**：
- 纯函数，不修改任何状态
- TargetRequireTag 在 target_tags 为 None 时跳过检查

### 3.4 SkillUseError — 失败原因

| 错误 | 包含信息 |
|------|----------|
| `OnCooldown` | remaining: u32 |
| `InsufficientMp` | required, current |
| `MissingTag` | tag: GameplayTag |
| `TargetMissingTag` | tag: GameplayTag |
| `HpNotBelow` | threshold: f32 |
| `HpNotAbove` | threshold: f32 |

---

## 4. SkillSlots — 技能槽位

```rust
#[derive(Component)]
pub struct SkillSlots {
    pub skill_ids: Vec<String>,
}
```

**规则**：
- 每个单位通过 `SkillSlots` 持有可用技能 ID 列表
- 第一个技能为默认攻击（`basic_attack`）
- `default_attack()` 空列表时回退到 `BASIC_ATTACK_ID` 常量
- `special_skill()` 返回第二个技能（如有）

### 4.1 有效射程计算

```rust
pub fn effective_skill_range(skill_data: &SkillData, base_attack_range: u32) -> u32
```

- 技能 `range > 0`：使用技能自身射程
- 技能 `range == 0`：使用单位基础攻击范围

---

## 5. SkillCooldowns — 冷却追踪

```rust
#[derive(Component)]
pub struct SkillCooldowns {
    cooldowns: HashMap<String, u32>,  // skill_id → 剩余回合数
}
```

| 方法 | 说明 |
|------|------|
| `get(skill_id)` | 获取当前冷却（未记录返回 0） |
| `set(skill_id, turns)` | 设置冷却（turns > 0 才记录） |
| `tick()` | 回合结束递减所有冷却，归零后移除 |
| `clear()` | 清除所有冷却 |

**冷却生命周期**：

```
使用技能 → set(skill_id, skill_data.cooldown)
  ↓
每回合结束 → tick() 递减
  ↓
冷却归零 → 自动移除，技能可用
```

---

## 6. SkillRegistry — 技能注册表

```rust
#[derive(Resource)]
pub struct SkillRegistry {
    pub skills: HashMap<String, SkillData>,
}
```

| 方法 | 说明 |
|------|------|
| `get(id)` | 查找技能定义 |
| `register(skill)` | 注册技能 |
| `register_defaults()` | 注册内置默认技能（幂等） |

### 6.1 数据加载

通过 `RegistryLoader` trait 实现 RON 文件加载：

- 加载目录：`assets/skills/`
- 每个 RON 文件反序列化为 `SkillDef`
- `SkillDef` 通过 `From<SkillDef> for SkillData` 转换为运行时类型
- `TagName` → `GameplayTag` 转换在此时完成

### 6.2 内置默认技能

| ID | 名称 | 射程 | 目标 | 效果 | 冷却 |
|----|------|------|------|------|------|
| `basic_attack` | 普通攻击 | 0 | SingleEnemy | Damage(1.0x) | 0 |
| `charge` | 冲锋 | 0 | SingleEnemy | Damage(1.5x) | 2 |
| `pierce` | 穿刺 | 0 | SingleEnemy | Damage(1.2x, 无视50%防御) | 3 |
| `fireball` | 火球 | 3 | SingleEnemy | Damage(1.5x) + ApplyBuff(burn, 2) | 2 |
| `heal` | 治疗 | 2 | SingleAlly | Heal(8) | 2 |
| `cleanse_skill` | 净化 | 2 | SingleAlly | Cleanse | 3 |

---

## 7. 技能预览

### 7.1 SkillExecutionContext — 执行上下文

```rust
pub struct SkillExecutionContext {
    pub source: Entity,
    pub target: Entity,
    pub skill_id: String,
    pub source_attrs: Attributes,      // 快照
    pub target_attrs: Attributes,      // 快照
    pub source_tags: GameplayTags,     // 快照
    pub target_tags: GameplayTags,     // 快照
    pub terrain_defense_bonus: i32,
}
```

**规则**：
- 纯数据快照，避免 ECS 借用冲突
- `from_query()` 从 ECS 查询构建

### 7.2 SkillPreview — 预览结果

```rust
pub struct SkillPreview {
    pub skill_id: String,
    pub skill_name: String,
    pub predictions: Vec<EffectPreview>,
}
```

### 7.3 EffectPreview — 效果预览

| 类型 | 字段 | 说明 |
|------|------|------|
| `Damage` | `amount, lethal` | 伤害预览（lethal = 是否致死） |
| `Heal` | `amount` | 治疗预览（不超过 MaxHp） |
| `BuffApplied` | `buff_name` | Buff 施加预览 |
| `Cleanse` | — | 净化预览 |

### 7.4 预览规则

```rust
pub fn preview_skill_effects(ctx, skill_data, buff_registry) -> SkillPreview
```

- 纯函数，不修改任何状态
- 通过 `EffectHandlerRegistry` trait 分发
- 新增效果类型只需注册 Handler，无需修改预览逻辑
- 伤害预览：`Attack - Defense - terrain_bonus`，最低 1
- 治疗预览：`min(amount, MaxHp - Hp)`
- 致死判定：预览伤害 >= 目标当前 HP

---

## 8. SkillDef — RON 配置格式

```ron
(
    version: 1,                    // 配置版本（可选，默认 0）
    id: "fireball",                // 技能 ID
    name: "火球",                  // 显示名称
    description: "远程火属性攻击",  // 描述
    cost_mp: 5,                    // MP 消耗
    range: 3,                      // 射程
    targeting: SingleEnemy,        // 目标类型
    effects: [
        Damage(multiplier: 1.5, ignore_def_percent: 0.0),
        ApplyBuff(buff_id: "burn", duration: 2),
    ],
    tags: [FIRE, SKILL_ACTIVE],    // 标签（TagName 枚举）
    conditions: [
        MpCost(5),
        RequireTag(MAGE),
    ],
    cooldown: 2,                   // 冷却回合
    priority: 10,                  // AI 优先级
)
```

**规则**：
- `version` 字段可选，缺失时默认为 0（向后兼容）
- `tags` 使用 `TagName` 枚举，反序列化时转为 `GameplayTag`
- `conditions` 使用 `SkillConditionDef`，反序列化时转为 `SkillCondition`

---

## 9. 关键约束

1. **can_use() 是纯函数**：不修改任何状态，可安全在 UI 和 AI 中调用
2. **冷却 set > 0 才记录**：`set(skill_id, 0)` 不产生记录
3. **冷却 tick 自动清理**：归零后从 HashMap 移除
4. **range = 0 使用基础范围**：技能射程为 0 时回退到单位攻击范围
5. **预览不修改状态**：`preview_skill_effects()` 是纯函数
6. **TargetRequireTag 无目标跳过**：`target_tags` 为 None 时不检查目标标签
7. **注册表幂等**：`register_defaults()` 重复调用不会重复注册
8. **RON 加载优先**：`assets/skills/` 目录有文件时使用文件数据，否则使用内置默认
9. **技能标签传递**：`skill_data.tags` 作为 `source_tags` 传入效果管线修饰阶段
10. **多效果顺序执行**：技能效果按 `effects` 列表顺序依次处理
