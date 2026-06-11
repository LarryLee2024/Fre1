# 效果管线领域规则 (Effect Pipeline)

## 1. 领域概述

效果管线是战斗系统的数据流核心，负责从技能定义生成效果、修饰效果、执行效果。采用 **Generate → Modify → Execute** 三步管线，通过 Handler trait 分发实现扩展。

### 核心原则

- **ECS 是数据流，不是调用链**：效果通过 EffectQueue 流转
- **Handler 分发**：新增效果类型只需实现 EffectHandler 并注册
- **Logic / Presentation 分离**：管线只产生数据和 Message，不调用 UI/VFX
- **数据驱动**：效果定义来自 RON 配置，代码解释配置

---

## 2. 管线总览

```
EffectDef (技能定义中的效果)
    ↓ Generate (EffectHandler.generate)
PendingEffectData (待处理效果数据)
    ↓ 组装 PendingEffect
EffectQueue.pending (效果队列)
    ↓ Modify (ModifierRuleRegistry)
修饰后的 PendingEffect
    ↓ Execute (系统消费)
属性变化 + Messages
```

---

## 3. EffectDef — 效果定义

```rust
pub enum EffectDef {
    Damage { multiplier: f32, ignore_def_percent: f32 },
    Heal { amount: i32 },
    ApplyBuff { buff_id: String, duration: u32 },
    Cleanse,
}
```

| 类型 | 参数 | 说明 |
|------|------|------|
| `Damage` | multiplier, ignore_def_percent | 伤害（倍率 × 净攻击力） |
| `Heal` | amount | 固定治疗量 |
| `ApplyBuff` | buff_id, duration | 施加 Buff |
| `Cleanse` | — | 净化所有 Debuff |

**type_name**：每个变体返回对应的类型名字符串，用于 Handler 查找。

---

## 4. PendingEffect — 待处理效果

```rust
pub struct PendingEffect {
    pub source: Entity,              // 攻击者
    pub target: Entity,              // 目标
    pub data: PendingEffectData,     // 效果数据
    pub source_tags: Vec<GameplayTag>, // 技能标签
    pub terrain_id: String,          // 地形 ID
}
```

### 4.1 PendingEffectData — 效果数据

| 类型 | 字段 | 说明 |
|------|------|------|
| `Damage` | amount, is_skill, base_amount, modifiers | 伤害（含修饰明细） |
| `Heal` | amount, base_amount | 治疗 |
| `ApplyBuff` | buff_id, duration | 施加 Buff |
| `Cleanse` | — | 净化 |

**base_amount**：Generate 阶段的原始值，Modify 阶段首次记录，后续不覆盖。

**modifiers**：Modify 阶段记录的修饰步骤列表（`Vec<ModifierEntry>`）。

---

## 5. EffectQueue — 效果队列

```rust
#[derive(Resource)]
pub struct EffectQueue {
    pub pending: Vec<PendingEffect>,
}
```

| 方法 | 说明 |
|------|------|
| `push(effect)` | 推入效果 |
| `is_empty()` | 是否为空 |
| `clear()` | 清空队列 |

**规则**：
- 管线三步共享同一个 EffectQueue
- Execute 使用 `drain(..)` 消费所有效果
- Trait 触发器（OnAttack/OnHit/OnKill）也会推入效果

---

## 6. EffectHandler — 效果处理器

### 6.1 Handler Trait

```rust
pub trait EffectHandler: Send + Sync + 'static {
    fn type_name(&self) -> &'static str;
    fn generate(&self, def: &EffectDef, ctx: &GenerateContext) -> Option<PendingEffectData>;
    fn preview(&self, def: &EffectDef, ctx: &PreviewContext) -> Option<EffectPreview>;
}
```

### 6.2 内置 Handler

| Handler | type_name | generate | preview |
|---------|-----------|----------|---------|
| `DamageHandler` | "Damage" | 计算伤害 | 预览伤害 + 致死判定 |
| `HealHandler` | "Heal" | 固定治疗量 | 预览治疗（不超过 MaxHp） |
| `BuffHandler` | "ApplyBuff" | 透传 buff_id/duration | 预览 Buff 名称 |
| `CleanseHandler` | "Cleanse" | 透传 | 预览净化 |

### 6.3 DamageHandler 伤害计算

```rust
calculate_damage_from_effect(
    effective_atk,    // 攻击者 Attack（含修饰符）
    effective_def,    // 目标 Defense（含修饰符）
    base_def,         // 目标基础 Defense（Vitality）
    multiplier,       // 技能倍率
    ignore_def_percent, // 无视防御百分比
    terrain_defense_bonus, // 地形防御加成
) -> i32
```

**公式**：

```
def_ignored = base_def × (ignore_def_percent / 100)
final_def = effective_def - def_ignored
base_damage = effective_atk - final_def
result = (base_damage - terrain_defense_bonus) × multiplier
result = max(1, result)
```

**规则**：
- 伤害最低为 1
- ignore_def_percent 基于基础防御（Vitality），不是修饰后的防御
- 地形防御加成在最后减去

### 6.4 HealHandler 治疗预览

```
actual = min(amount, MaxHp - Hp).max(0)
```

---

## 7. GenerateContext — 生成上下文

```rust
pub struct GenerateContext {
    pub source_entity: Entity,
    pub target_entity: Entity,
    pub source_attrs: Attributes,    // 攻击者属性快照
    pub target_attrs: Attributes,    // 目标属性快照
    pub defense_bonus: i32,          // 地形防御加成
    pub skill_id: String,            // 技能 ID
    pub source_tags: Vec<GameplayTag>, // 技能标签
    pub terrain_id: String,          // 地形 ID
}
```

**规则**：纯数据快照，避免 ECS 借用冲突。

---

## 8. PreviewContext — 预览上下文

```rust
pub struct PreviewContext {
    pub source_attrs: Attributes,
    pub target_attrs: Attributes,
    pub terrain_defense_bonus: i32,
    pub buff_registry: BuffRegistry,
}
```

---

## 9. EffectPreview — 效果预览

| 类型 | 字段 | 说明 |
|------|------|------|
| `Damage` | amount, lethal | 伤害预览（lethal = 是否致死） |
| `Heal` | amount | 治疗预览 |
| `BuffApplied` | buff_name | Buff 名称 |
| `Cleanse` | — | 净化 |

**规则**：预览是纯函数，不修改任何状态。

---

## 10. EffectHandlerRegistry — 处理器注册表

```rust
#[derive(Resource)]
pub struct EffectHandlerRegistry {
    handlers: HashMap<String, Box<dyn EffectHandler>>,
}
```

| 方法 | 说明 |
|------|------|
| `find(type_name)` | O(1) 查找处理器 |
| `register(handler)` | 注册处理器（不重复注册） |
| `register_defaults()` | 注册 4 个内置处理器 |

**规则**：
- 默认注册 Damage/Heal/ApplyBuff/Cleanse 四个处理器
- 重复注册同一 type_name 会跳过并警告
- 新增效果类型只需实现 Handler 并注册

---

## 11. EffectResult — 执行结果

```rust
pub struct EffectResult {
    pub source: Entity,
    pub target: Entity,
    pub data: EffectResultData,
}

pub enum EffectResultData {
    Damage { amount: i32, killed: bool },
    Heal { amount: i32 },
    BuffApplied { buff_id: String },
    CleanseApplied,
}
```

---

## 12. 管线三步详解

### 12.1 Generate — 效果生成

```
输入：CombatIntent + SkillData + 攻击者/目标属性
处理：
  1. 前置检查（晕眩、冷却）
  2. 遍历 skill_data.effects
  3. 通过 EffectHandlerRegistry 分发
  4. handler.generate() 生成 PendingEffectData
  5. 组装 PendingEffect 推入 EffectQueue
  6. 触发 OnAttack Trait
输出：EffectQueue.pending
```

### 12.2 Modify — 效果修饰

```
输入：EffectQueue + ModifierRuleRegistry
处理：
  1. 遍历 queue.pending
  2. Damage → apply_damage_modifiers_with_breakdown()
  3. Heal → apply_heal_modifiers()
  4. ApplyBuff/Cleanse → 不修饰
  5. 记录 base_amount（首次）和 modifiers
输出：修饰后的 EffectQueue
```

### 12.3 Execute — 效果执行

```
输入：修饰后的 EffectQueue
处理：
  1. drain(..) 消费所有效果
  2. Damage → 扣血 + 死亡判定 + DamageApplied Message
  3. Heal → 回血 + HealApplied Message
  4. ApplyBuff → 施加 Buff
  5. Cleanse → 驱散 Debuff
  6. 死亡 → Dead 组件 + CharacterDied Message
输出：属性变化 + Messages
```

---

## 13. 关键约束

1. **三步严格顺序**：Generate → Modify → Execute，不可跳步
2. **Handler 分发扩展**：新增效果类型只需实现 Handler 并注册
3. **伤害最低为 1**：`max(1.0, result)`
4. **base_amount 首次记录**：Modify 阶段首次设置，后续不覆盖
5. **EffectQueue 每轮清空**：Execute 使用 `drain(..)` 消费
6. **Generate 是纯计算**：不修改 ECS 状态，只推入队列
7. **Preview 是纯函数**：不修改任何状态
8. **类型不匹配返回 None**：Handler 收到错误类型时返回 None
9. **注册表不重复注册**：同一 type_name 只注册一次
10. **terrain_defense_bonus 在 Generate 传入**：不在 Modify 阶段处理
