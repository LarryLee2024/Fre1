# Effect 领域 — 铃兰之剑数据提取

> 领域：Effect | 来源：78铃兰.md §四、补充3、补充4、补充5、补充6、补充7、补充10 | 数据层：Definition + Instance

---

## 一、数据实体清单

### 1.1 EffectDefinition（Definition层）

| 字段名 | 类型 | 约束 | 说明 | 来源 |
|--------|------|------|------|------|
| `id` | EffectId | PK | 效果唯一标识 | §三.2 |
| `name_key` | String | - | 效果名称本地化Key（ApplyBuff类型使用） | §四 + ADR-017 |
| `desc_key` | String | - | 效果描述本地化Key（ApplyBuff类型使用） | §四 + ADR-017 |
| `effect_type` | EffectType | - | 效果类型 | 全文 |
| `duration` | Option<DurationDef> | - | 持续时间定义 | §四.2 |
| `stacking` | Option<StackingId> | FK | 堆叠策略引用 | §五 |
| `cue` | Option<CueId> | FK | 表现引用 | §三.4 |

### 1.2 EffectInstance（Instance层）

| 字段名 | 类型 | 说明 | 来源 |
|--------|------|------|------|
| `entity` | Entity | 挂载目标实体 | - |
| `effect_id` | EffectId | 引用EffectDefinition | - |
| `source_entity` | Option<Entity> | 来源实体 | §四 |
| `remaining_duration` | Option<u32> | 剩余持续回合 | §四.2 |
| `current_stack` | u32 | 当前层数 | §四.3 |
| `lifecycle_phase` | LifecyclePhase | 当前生命周期阶段 | §四.2 |

---

## 二、Effect类型分类

### 2.1 EffectType 枚举

| 类型 | 说明 | 产出 | 来源 |
|------|------|------|------|
| `Damage` | 伤害效果 | 扣减HP | §二 |
| `Heal` | 治疗效果 | 恢复HP | 补充6 |
| `ApplyBuff` | 施加Buff | 添加Tag + Modifier | §四 |
| `Dispel` | 驱散效果 | 移除Tag + Modifier | 补充3 |
| `Displacement` | 位移效果 | 改变位置 | 补充4 |
| `ApplyShield` | 施加护盾 | 添加护盾实例 | 补充6 |
| `Summon` | 召唤效果 | 创建召唤物实体 | 补充10 |
| `Kill` | 死亡效果 | 标记死亡 | 补充7 |

---

## 三、Buff生命周期（Data Law 007: Duration属于Effect）

所有持续状态遵循统一生命周期：

```
Apply（施加）→ Tick（周期触发）→ Expire（到期）→ Remove（移除）
```

| 阶段 | 触发时机 | 行为 | 来源 |
|------|----------|------|------|
| Apply | 效果施加时 | 添加到目标，触发入场效果，刷新持续时间/层数 | §四.2 |
| Tick | 按回合/按行动 | 周期效果触发（如中毒每回合掉血） | §四.2 |
| Expire | 持续时间归零 | 触发到期效果 | §四.2 |
| Remove | 被驱散/覆盖 | 执行清理逻辑 | §四.2 |

### 3.1 Duration定义

| 字段名 | 类型 | 说明 | 来源 |
|--------|------|------|------|
| `duration_type` | Enum | turns/actions | §四.2 |
| `value` | u32 | 持续数值 | §四.2 |
| `tick_timing` | Enum | turn_start/turn_end/action_end | §四.2 |

---

## 四、各类Effect详细数据

### 4.1 Damage Effect

| 字段名 | 类型 | 说明 | 来源 |
|--------|------|------|------|
| `damage_type` | DamageTypeTag | 物理/魔法/穿透/真实 | §二.1 |
| `skill_multiplier` | f32 | 技能倍率 | §二.2第三段 |
| `can_crit` | bool | 是否可暴击 | §二.2 |
| `is_multi_hit` | bool | 是否多段伤害 | 补充5 |
| `hit_count` | Option<u32> | 多段伤害段数 | 补充5 |

多段伤害规则：
- 每段独立判定暴击、命中、闪避
- 「受到伤害时触发」被动：每段触发一次
- 「被技能命中时触发」被动：整个技能只触发一次
- 护盾：每段单独扣减
- **「技能命中」和「造成伤害」是两个独立事件**

### 4.2 Heal Effect

| 字段名 | 类型 | 说明 | 来源 |
|--------|------|------|------|
| `heal_base` | Enum | atk_based/fixed | 补充6 |
| `heal_multiplier` | f32 | 治疗倍率 | 补充6 |
| `can_crit` | bool | 是否可暴击 | 补充6 |
| `aoe_decay` | Option<f32> | AOE治疗递减率 | 补充6 |
| `overheal_to_shield` | Option<f32> | 过量治疗转护盾比例 | 补充6 |

### 4.3 Shield Effect

| 字段名 | 类型 | 说明 | 来源 |
|--------|------|------|------|
| `shield_type` | Enum | physical/magical/universal | 补充6 |
| `shield_value` | f32 | 护盾吸收量 | 补充6 |
| `is_regen_shield` | bool | 是否每回合回复 | 补充6 |
| `regen_value` | Option<f32> | 每回合回复值 | 补充6 |
| `damage_type_filter` | Option<Vec<DamageTypeTag>> | 只吸收特定伤害类型 | 补充6 |

护盾结算顺序：先扣通用护盾 → 再扣物理/魔法专属护盾

### 4.4 Dispel Effect

| 字段名 | 类型 | 说明 | 来源 |
|--------|------|------|------|
| `dispel_type` | Enum | buff/debuff/all | 补充3 |
| `dispel_count` | u32 | 驱散数量 | 补充3 |
| `dispel_priority` | Enum | newest/oldest/strongest | 补充3 |

### 4.5 Displacement Effect

| 字段名 | 类型 | 说明 | 来源 |
|--------|------|------|------|
| `displacement_type` | Enum | active/forced | 补充4 |
| `direction` | Option<Direction> | 强制位移方向 | 补充4 |
| `distance` | u32 | 位移格数 | 补充4 |
| `can_cross_obstacle` | bool | 是否可穿越障碍 | 补充4 |
| `wall_damage_pct` | Option<f32> | 撞墙伤害（最大生命值%） | 补充4 |
| `can_push_units` | bool | 是否推开路径上单位 | 补充4 |

位移规则：
- 位移路径上每经过1格地形效果触发1次
- 位移结束位置不可通行时回退到最近合法格子
- 定身：禁止主动位移，不禁止强制位移
- 石化：禁止所有位移

### 4.6 Summon Effect

| 字段名 | 类型 | 说明 | 来源 |
|--------|------|------|------|
| `summon_template` | CharacterId | 召唤物模板 | 补充10 |
| `inherit_ratio` | f32 | 属性继承比例 | 补充10 |
| `max_count` | u32 | 最大召唤数量 | 补充10 |
| `duration` | Option<u32> | 召唤持续回合 | 补充10 |

召唤物规则：
- 属性按百分比继承召唤者**施加时的快照属性**
- 召唤者后续Buff变化不影响召唤物
- 召唤物击杀算召唤者本体击杀
- 召唤物死亡不算友军死亡
- 召唤者死亡 → 所有召唤物消失
- 召唤物数量超上限 → 顶替最早的

### 4.7 Kill Effect（死亡链路）

死亡结算4步链路：

```
伤害结算完成 → 血量≤0判定
  ↓
濒死窗口（回血/免死/假死）
  ↓ 血量仍≤0
「即将死亡」事件（最后一次触发被动）
  ↓
清除可驱散Buff/Debuff（保留机制Tag）
  ↓
「单位死亡」事件（击杀者触发击杀被动）
  ↓
标记死亡，移除实体控制权
```

复活规则：复活后仅恢复基础血量，不恢复之前的Buff、冷却、能量

---

## 五、状态三大分类

| 分类 | 颜色标识 | 可驱散 | 子类 | 来源 |
|------|----------|--------|------|------|
| 增益 | 橙色 | 是 | 属性类/功能类/特殊类 | §四.1 |
| 减益 | 红色/紫色 | 是 | 属性类/控制类/持续伤害类/功能类 | §四.1 |
| 特殊 | 蓝色/灰色 | 否 | 濒死/不可驱散减伤/召唤物归属/地形效果 | §四.1 |

### 层数型Buff

| 字段名 | 类型 | 说明 | 来源 |
|--------|------|------|------|
| `max_stack` | u32 | 最大层数 | §四.3 |
| `stack_effect` | StackEffectType | 独立/衰减/不可驱散 | §四.3 |
| `decay_rule` | Option<DecayRule> | 衰减规则（如触发后减半） | §四.3 |

---

## 六、Schema草案

```yaml
# effect_config.ron
(
  effects: [
    # 物理伤害效果（无玩家可见名称）
    (id: "phys_damage", effect_type: Damage,
     damage: (damage_type: "dmg_physical", can_crit: true)),
    # 中毒效果（层数型，有名称）
    (id: "poison", name_key: "buff.b_001.name", desc_key: "buff.b_001.desc", effect_type: ApplyBuff,
     duration: (duration_type: Turns, value: 3, tick_timing: ActionEnd),
     stacking: "stack_independent",
     max_stack: 9,
     tick_effect: "poison_tick"),
    # 击退效果（无玩家可见名称）
    (id: "knockback_2", effect_type: Displacement,
     displacement: (displacement_type: Forced, distance: 2,
                    can_cross_obstacle: false, wall_damage_pct: 0.1)),
  ],
)
```

### 对应 FTL 文件示例

```ftl
# zh-CN/buff.ftl
buff.b_001.name = 中毒
buff.b_001.desc = 每回合受到{ $damage }点伤害，持续{ $duration }回合
```

---

## 七、依赖关系

| 依赖领域 | 依赖方向 | 说明 |
|----------|----------|------|
| Modifier | Effect → Modifier | Effect产生Modifier |
| Tag | Effect → Tag | Effect添加/移除Tag |
| Cue | Effect → Cue | Effect触发表现 |
| Trigger | Effect → Trigger | Effect触发事件 |
| Stacking | Effect ← Stacking | Stacking决定Effect叠加行为 |
| Targeting | Effect ← Targeting | 位移Effect需要地形交互 |

---

## 八、Data Laws合规

| Law | 状态 | 说明 |
|-----|------|------|
| 004 | ✅ | Ability通过Effect列表执行，不直接调用Modifier |
| 005 | ✅ | Effect是唯一业务执行入口 |
| 007 | ✅ | Duration定义在Effect中，不属于独立Buff系统 |
| 009 | ✅ | Effect→Cue链路已建立，Cue是表现层唯一入口（见 12_Cue_ll.md） |

---

## 九、代码实现映射

| 概念 | Rust 类型 | 源码路径 | 层级 |
|------|-----------|----------|------|
| EffectDef | `enum EffectDef { Damage { multiplier, ignore_def_percent }, Heal { amount }, ApplyModifier { modifier_id, duration, stacking }, Cleanse }` | `src/core/effect/types.rs` | Definition (RON) |
| DurationDef | `enum DurationDef { Instant, TurnLimited(u32), Permanent }` | `src/core/effect/types.rs` | Definition |
| StackingDef | `enum StackingDef { Replace, RefreshDuration, StackAdd, StackMax { max_stack } }` | `src/core/effect/types.rs` | Definition |
| PendingEffect | `PendingEffect { source, target, data: PendingEffectData, source_tags, terrain_id }` | `src/core/effect/types.rs` | Runtime (Queue) |
| PendingEffectData | `enum PendingEffectData { Damage { amount, is_skill, base_amount, modifiers }, Heal { ... }, ApplyModifier { ... }, Cleanse }` | `src/core/effect/types.rs` | Runtime |
| EffectResult | `EffectResult { source, target, data: EffectResultData }` | `src/core/effect/types.rs` | Runtime (Output) |
| EffectResultData | `enum EffectResultData { Damage { amount, killed }, Heal { amount }, ModifierApplied { modifier_id }, CleanseApplied }` | `src/core/effect/types.rs` | Runtime |
| EffectQueue | `EffectQueue { pending: Vec<PendingEffect> }` — Resource | `src/core/effect/types.rs` | Runtime (Resource) |

**Effect 类型**（8 种）：Damage, Heal, ApplyBuff, Dispel, Displacement, ApplyShield, Summon, Kill
**RON 配置**：`content/effects/damage_basic.ron`（数组格式） + 技能文件中 inline 定义