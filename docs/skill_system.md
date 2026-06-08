# 技能系统技术文档

## 1. 系统架构

技能系统遵循 UE GAS (Gameplay Ability System) 核心思想，采用 5 层架构设计：

```
Layer 5: Runtime ECS          ← Unit / SkillSlots / SkillCooldowns / ActiveBuffs
Layer 4: Observer 事件流       ← OnEnter(ExecuteAction) 链式执行
Layer 3: Effect Pipeline      ← Generate → Modify → Execute 三步管道
Layer 2: Tag + Attribute      ← GameplayTags + Attributes 修饰栈
Layer 1: Data Asset           ← SkillData / BuffData 注册表
```

### 核心设计原则

- **数据驱动**：技能和 Buff 定义在注册表中，棋子只持有 `skill_ids`
- **完全解耦**：技能定义独立于棋子，新增技能无需修改棋子代码
- **纯函数优先**：伤害计算、条件检查、效果预览均为纯函数，易于测试
- **修饰栈模式**：属性通过 base → Add 叠加 → Multiply 叠加计算，Buff 修饰符可安全添加/移除

## 2. 核心类型

### 2.1 SkillData（技能数据定义）

```rust
pub struct SkillData {
    pub id: String,              // 唯一标识
    pub name: String,            // 显示名称
    pub description: String,     // 技能描述
    pub cost_mp: i32,            // MP 消耗
    pub range: u32,              // 射程（0 = 使用单位自身攻击范围）
    pub targeting: SkillTargeting, // 目标类型
    pub effects: Vec<EffectDef>,   // 效果列表
    pub tags: Vec<GameplayTag>,    // 技能标签
    pub conditions: Vec<SkillCondition>, // 使用条件
    pub cooldown: u32,           // 冷却回合数
    pub priority: u32,           // AI 优先级
}
```

### 2.2 SkillTargeting（目标类型）

| 变体 | 说明 | 需要选择目标 |
|------|------|:---:|
| `SingleEnemy` | 对单个敌方单位 | 是 |
| `SingleAlly` | 对单个友方单位 | 是 |
| `SelfOnly` | 对自身 | 否 |
| `AoeEnemies` | 对周围敌方 | 否 |
| `AoeAllies` | 对周围友方 | 否 |
| `NoTarget` | 无需目标 | 否 |

### 2.3 SkillCondition（使用条件）

| 条件 | 说明 |
|------|------|
| `MpCost(n)` | 需要 MP ≥ n |
| `RequireTag(tag)` | 施法者需拥有指定标签 |
| `TargetRequireTag(tag)` | 目标需拥有指定标签 |
| `HpBelow(pct)` | 施法者 HP < pct% |
| `HpAbove(pct)` | 施法者 HP > pct% |

### 2.4 SkillUseError（使用失败原因）

```rust
pub enum SkillUseError {
    OnCooldown { remaining: u32 },
    InsufficientMp { required: i32, current: i32 },
    MissingTag { tag: GameplayTag },
    TargetMissingTag { tag: GameplayTag },
    HpNotBelow { threshold: f32 },
    HpNotAbove { threshold: f32 },
}
```

### 2.5 EffectDef（效果定义）

| 效果 | 说明 |
|------|------|
| `Damage { multiplier, ignore_def_percent }` | 伤害：ATK × multiplier，无视 ignore_def_percent% 基础防御 |
| `Heal { amount }` | 治疗：恢复 amount HP（不超过 MaxHP） |
| `ApplyBuff { buff_id, duration }` | 施加 Buff |
| `Cleanse` | 驱散所有 Debuff |

## 3. 运行时组件

### 3.1 SkillSlots（技能槽）

棋子持有的技能 ID 列表，与技能定义完全解耦：

```rust
#[derive(Component)]
pub struct SkillSlots {
    pub skill_ids: Vec<String>,
}
```

- 第一个技能为默认攻击
- 通过 `SkillSlots::new(vec!["basic_attack", "fireball"])` 构建

### 3.2 SkillCooldowns（冷却追踪）

```rust
#[derive(Component)]
pub struct SkillCooldowns {
    cooldowns: HashMap<String, u32>,  // skill_id → 剩余回合
}
```

- `get(skill_id)` 查询冷却
- `set(skill_id, turns)` 设置冷却
- `tick()` 回合结束时递减所有冷却
- 在 `status.rs` 的 `resolve_status_effects` 中自动 tick

### 3.3 SkillExecutionContext（执行上下文）

封装一次技能释放的所有信息快照，避免 ECS 借用冲突：

```rust
pub struct SkillExecutionContext {
    pub source: Entity,
    pub target: Entity,
    pub skill_id: String,
    pub source_attrs: Attributes,
    pub target_attrs: Attributes,
    pub source_tags: GameplayTags,
    pub target_tags: GameplayTags,
    pub terrain: Terrain,
}
```

## 4. 效果预览

`preview_skill_effects()` 是纯函数，不修改任何状态，用于 UI 显示伤害预览：

```rust
pub fn preview_skill_effects(
    ctx: &SkillExecutionContext,
    skill_data: &SkillData,
    buff_registry: &BuffRegistry,
) -> SkillPreview
```

返回的 `SkillPreview` 包含每个效果的预测结果：

| 预览类型 | 说明 |
|----------|------|
| `Damage { amount, lethal }` | 预计伤害值，是否致死 |
| `Heal { amount }` | 预计治疗量（不超过 MaxHP - 当前HP） |
| `BuffApplied { buff_name }` | 将施加的 Buff 名称 |
| `Cleanse` | 将驱散所有 Debuff |

## 5. 技能执行流程

```
玩家选择技能 → 设置 CombatIntent
     ↓
OnEnter(ExecuteAction)
     ↓
generate_combat_effects()  ← 从 SkillData 生成 PendingEffect
     ↓
modify_effects()           ← 标签增伤等被动修饰
     ↓
execute_effects()          ← 扣血/加Buff/特效/日志/击杀
     ↓
execute_action_on_enter()  ← 设置冷却 + 标记已行动
```

## 6. 条件检查

`SkillData::can_use()` 在技能使用前检查所有条件：

```rust
let result = skill_data.can_use(
    &source_attrs,    // 施法者属性
    &source_tags,     // 施法者标签
    Some(&target_tags), // 目标标签（可选）
    cooldowns.get(skill_id), // 当前冷却
);
```

返回 `Ok(())` 或 `Err(SkillUseError)`。

## 7. 注册表与动态扩展

### SkillRegistry

```rust
let mut registry = SkillRegistry::default();
registry.populate();  // 加载默认技能
registry.register(custom_skill);  // 动态注册新技能
```

### 当前内置技能

| ID | 名称 | 类型 | 效果 | 冷却 |
|----|------|------|------|:----:|
| `basic_attack` | 普通攻击 | SingleEnemy | 1.0× 伤害 | 0 |
| `charge` | 冲锋 | SingleEnemy | 1.5× 伤害 | 0 |
| `pierce` | 穿透箭 | SingleEnemy | 1.3× 伤害，无视50%防御 | 2 |
| `fireball` | 火球 | SingleEnemy | 1.8× 伤害 + 灼烧2回合 | 3 |
| `heal` | 治疗 | SingleAlly | 恢复8 HP | 2 |
| `cleanse_skill` | 净化 | SingleAlly | 驱散所有 Debuff | 3 |

## 8. 添加新技能示例

```rust
let new_skill = SkillData {
    id: "ice_lance".into(),
    name: "冰枪".into(),
    description: "冰系远程攻击，2倍伤害并冻结1回合".into(),
    cost_mp: 5,
    range: 3,
    targeting: SkillTargeting::SingleEnemy,
    effects: vec![
        EffectDef::Damage { multiplier: 2.0, ignore_def_percent: 0.0 },
        EffectDef::ApplyBuff { buff_id: "stun".into(), duration: 1 },
    ],
    tags: vec![GameplayTag::ICE, GameplayTag::SKILL_ACTIVE],
    conditions: vec![SkillCondition::MpCost(5)],
    cooldown: 2,
    priority: 25,
};
registry.register(new_skill);
```

然后在 `spawn_units` 中给棋子添加技能 ID：

```rust
SkillSlots::new(vec!["basic_attack".into(), "ice_lance".into()])
```

## 9. 测试覆盖

| 模块 | 测试数 | 覆盖内容 |
|------|:------:|----------|
| skill_data | 22 | SkillSlots、SkillTargeting、SkillCooldowns、can_use 条件、SkillRegistry、效果预览 |
| buff_data | 17 | ActiveBuffs CRUD、tick、晕眩、DoT/HoT、apply/remove_buff、cleanse、注册表 |
| attribute | 11 | 基础值、加法/乘法修饰符、混合、移除、批量添加 |
| effect | 12 | 伤害计算（基础/倍率/无视防御/地形）、EffectQueue |
| tag | 5 | 位掩码查询、has_any/has_all、TagName 转换 |
| **总计** | **67** | |

运行测试：`cargo test`
