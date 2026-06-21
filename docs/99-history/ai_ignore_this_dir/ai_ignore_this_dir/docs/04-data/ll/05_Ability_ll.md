# Ability 领域 — 铃兰之剑数据提取

> 领域：Ability | 来源：78铃兰.md §三、补充8 | 数据层：Definition + Instance

---

## 一、数据实体清单

### 1.1 AbilityDefinition（Definition层）

| 字段名 | 类型 | 约束 | 说明 | 来源 |
|--------|------|------|------|------|
| `id` | AbilityId | PK | 技能唯一标识 | §三.2 |
| `name_key` | String | - | 技能名称本地化Key | §三.2 + ADR-017 |
| `desc_key` | String | - | 技能描述本地化Key | §三.2 + ADR-017 |
| `ability_type` | AbilityType | - | 技能类型 | §三.1 |
| `cost` | CostDef | - | 消耗定义 | §三.2 |
| `cooldown` | Option<u32> | - | 冷却回合数 | §三.2 |
| `range` | u32 | ≥1 | 射程 | §三.2 |
| `targeting` | TargetingId | FK | 目标选择规则引用 | §三.3 |
| `effects` | Vec<EffectId> | FK | 效果列表引用 | §三.2 |
| `tags_required` | Vec<TagId> | - | 需具备的Tag | §三.2 |
| `tags_forbidden` | Vec<TagId> | - | 禁止具备的Tag | §三.2 |
| `special_rules` | Vec<SpecialRule> | - | 特殊规则 | §三.2 |

### 1.2 AbilityInstance（Instance层）

| 字段名 | 类型 | 说明 | 来源 |
|--------|------|------|------|
| `entity` | Entity | 所属实体 | - |
| `ability_id` | AbilityId | 引用AbilityDefinition | - |
| `cooldown_remaining` | u32 | 剩余冷却回合 | 补充8 |
| `current_energy_cost` | f32 | 当前能量消耗 | 补充8 |

---

## 二、技能五大分类

### 2.1 AbilityType 枚举

| 类型 | 行动消耗 | 触发方式 | 典型例子 | 来源 |
|------|----------|----------|----------|------|
| `NormalAttack` | 消耗1次行动 | 玩家主动释放 | 普通攻击、远程射击 | §三.1 |
| `ActiveSkill` | 消耗能量+行动 | 玩家主动释放 | 火球、冲锋、治疗 | §三.1 |
| `ReactionSkill` | 不消耗行动 | 事件触发 | 反击、援护、回击 | §三.1 |
| `SupportSkill` | 瞬发，保留行动 | 玩家主动释放 | 加Buff、驱散、自身回血 | §三.1 |
| `PassiveSkill` | 无消耗 | 常驻/条件触发 | 个性、天赋、装备被动 | §三.1 |

> **Data Law 004 注意**：反应技(ReactionSkill)在铃兰中归为技能分类，但按Data Law 004，Ability不拥有行为，反应技应归属Trigger领域。建议实现时将ReactionSkill映射为Trigger+Ability组合。

---

## 三、技能标准化构成（5字段）

### 3.1 基础信息

| 字段 | 类型 | 说明 | 来源 |
|------|------|------|------|
| `id` | AbilityId | 技能ID | §三.2 |
| `name_key` | String | 技能名称本地化Key | §三.2 + ADR-017 |
| `desc_key` | String | 技能描述本地化Key | §三.2 + ADR-017 |
| `ability_type` | AbilityType | 技能类型 | §三.2 |
| `cost` | CostDef | 消耗 | §三.2 |
| `cooldown` | Option<u32> | 冷却回合 | §三.2 |
| `range` | u32 | 射程 | §三.2 |

### 3.2 目标规则

| 字段 | 类型 | 说明 | 来源 |
|------|------|------|------|
| `targeting_id` | TargetingId | 目标选择规则引用 | §三.3 |

### 3.3 前置校验

| 校验项 | 类型 | 说明 | 来源 |
|--------|------|------|------|
| 消耗检查 | CostDef | AP/CP是否足够 | §三.4 |
| 冷却检查 | u32 | 冷却是否为0 | §三.4 |
| 标签检查 | (Vec<TagId>, Vec<TagId>) | 需具备/禁止具备的Tag | §三.4 |

### 3.4 效果列表

| 字段 | 类型 | 说明 | 来源 |
|------|------|------|------|
| `effects` | Vec<EffectId> | 按顺序执行的效果列表 | §三.2 |

执行顺序：伤害 → Buff → 位移

### 3.5 特殊规则

| 规则 | 类型 | 说明 | 来源 |
|------|------|------|------|
| `is_instant` | bool | 瞬发 | §三.1 |
| `can_cast_after_move` | bool | 移动后可释放 | §三.2 |
| `backstab_bonus` | bool | 背击加成 | §三.2 |
| `is_multi_hit` | bool | 多段攻击 | §三.2 |

---

## 四、资源管控三层体系（补充8）

### 4.1 行动点（AP）

| 字段 | 类型 | 约束 | 说明 | 来源 |
|------|------|------|------|------|
| `base_ap` | u32 | =1 | 每回合默认行动点 | 补充8 |
| `current_ap` | u32 | ≤max_ap | 当前行动点 | 补充8 |
| `max_ap` | u32 | ≥1 | 单回合行动点上限 | 补充8 |

规则：
- 普攻/主动技能消耗1点
- 支援技能不消耗
- 「再行动」= 额外增加1点AP，不是直接再动一次

### 4.2 能量（CP）

| 字段 | 类型 | 约束 | 说明 | 来源 |
|------|------|------|------|------|
| `current_cp` | f32 | ≤max_cp | 当前能量 | 补充8 |
| `max_cp` | f32 | >0 | 能量上限 | 补充8 |
| `regen_per_turn` | f32 | ≥0 | 每回合固定回复 | 补充8 |
| `bonus_on_kill` | f32 | ≥0 | 击杀额外回复 | 补充8 |
| `bonus_on_hit` | f32 | ≥0 | 受击额外回复 | 补充8 |

规则：
- 溢出作废
- 「能量回复提升」只影响回合固定回复，不影响击杀额外回复

### 4.3 冷却回合（CD）

| 操作 | 说明 | 来源 |
|------|------|------|
| 冷却减少 | 直接减少剩余回合数，多个效果加算 | 补充8 |
| 冷却刷新 | 直接清零冷却，可被`no_cooldown_refresh`Tag屏蔽 | 补充8 |
| 冷却增加 | 延长技能冷却，属于减益效果 | 补充8 |

### 4.4 CostDef 结构

```yaml
# 消耗定义
cost:
  ap: 1           # 行动点消耗
  cp: 0           # 能量消耗（0=不消耗）
  hp_pct: 0.0     # 生命值百分比消耗（可选）
```

---

## 五、Schema草案

```yaml
# ability_config.ron
(
  abilities: [
    # 普攻
    (id: "normal_attack", name_key: "skill.s_1000.name", desc_key: "skill.s_1000.desc", ability_type: NormalAttack,
     cost: (ap: 1, cp: 0), cooldown: None, range: 1,
     targeting: "single_enemy", effects: ["phys_damage"],
     tags_required: [], tags_forbidden: ["control_full"],
     special_rules: []),
    # 火球术
    (id: "fireball", name_key: "skill.s_1001.name", desc_key: "skill.s_1001.desc", ability_type: ActiveSkill,
     cost: (ap: 1, cp: 30), cooldown: Some(3), range: 3,
     targeting: "single_enemy", effects: ["fire_damage", "apply_burn"],
     tags_required: [], tags_forbidden: ["silenced"],
     special_rules: []),
  ],
)
```

### 对应 FTL 文件示例

```ftl
# zh-CN/skill.ftl
skill.s_1000.name = 普通攻击
skill.s_1000.desc = 基础攻击，造成1.0倍物理伤害

skill.s_1001.name = 火球术
skill.s_1001.desc = 发射火球，造成{ $multiplier }倍魔法伤害并附加灼烧
```

---

## 六、依赖关系

| 依赖领域 | 依赖方向 | 说明 |
|----------|----------|------|
| Effect | Ability → Effect | Ability引用Effect列表 |
| Targeting | Ability → Targeting | Ability引用目标选择规则 |
| Trigger | Ability ← Trigger | 反应技通过Trigger触发 |
| Tag | Ability → Tag | 前置校验检查Tag |

---

## 七、Data Laws合规

| Law | 状态 | 说明 |
|-----|------|------|
| 004 | ⚠️ | 反应技在铃兰中归为技能，但按Data Law应归属Trigger。建议映射为Trigger+Ability |
| 005 | ✅ | Ability通过Effect列表执行业务，不直接调用Modifier |
| 007 | ✅ | 冷却回合属于Ability，Duration属于Effect，边界清晰 |

---

## 八、代码实现映射

| 概念 | Rust 类型 | 源码路径 | 层级 |
|------|-----------|----------|------|
| SkillDef | `SkillDef { version, id, name, description, name_key, desc_key, cost_mp, range, targeting, effects, tags, conditions, cooldown, priority }` | `src/core/ability/domain/types.rs` | Definition (RON) |
| SkillData | `SkillData { id, name, description, name_key, desc_key, cost_mp, range, targeting, effects, tags: Vec<GameplayTag>, conditions, cooldown, priority }` | `src/core/ability/domain/types.rs` | Definition (Runtime) |
| SkillRegistry | `SkillRegistry { skills: HashMap<String, SkillData> }` | `src/core/ability/domain/types.rs` | Definition (Resource) |
| SkillSlots | `SkillSlots { skill_ids: Vec<String> }` — Component | `src/core/ability/slots.rs` | Instance (Component) |
| SkillCooldowns | `SkillCooldowns { cooldowns: HashMap<String, u32> }` — Component | `src/core/ability/slots.rs` | Instance (Component) |
| SkillTargeting | `enum SkillTargeting { SingleEnemy, SingleAlly, SelfOnly, AoeEnemies, AoeAllies, NoTarget }` | `src/core/targeting/mod.rs` | Definition |
| SkillCondition | `enum SkillCondition { MpCost(i32), RequireTag(GameplayTag), TargetRequireTag(GameplayTag), HpBelow(f32), HpAbove(f32) }` | `src/core/ability/domain/types.rs` | Definition |
| SkillError | `enum SkillError { SkillNotFound { skill_id }, SkillNotReady { skill_id, reason } }` | `src/core/ability/domain/error.rs` | Error |

**SkillUseError**：`OnCooldown { remaining }` | `InsufficientMp { required, current }` | `MissingTag { tag }` | `TargetMissingTag { tag }` | `HpNotBelow { threshold }` | `HpNotAbove { threshold }`

**RON 配置**：`content/skills/*.ron`（6 个技能文件，单对象格式）