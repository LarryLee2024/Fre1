# Stacking 领域 — 铃兰之剑数据提取

> 领域：Stacking | 来源：78铃兰.md §五、§四.3 | 数据层：Definition

---

## 一、数据实体清单

### 1.1 StackingDefinition（Definition层）

| 字段名 | 类型 | 约束 | 说明 | 来源 |
|--------|------|------|------|------|
| `id` | StackingId | PK | 堆叠策略唯一标识 | §五 |
| `stack_type` | StackType | - | 堆叠类型 | §五 |
| `max_stack` | u32 | ≥1 | 最大层数 | §五 |
| `on_max_action` | OnMaxAction | - | 达到最大层数时的行为 | §五 |
| `duration_refresh` | bool | - | 叠加时是否刷新持续时间 | §五 |

---

## 二、基础堆叠规则

### 2.1 同名效果 vs 不同名效果

| 规则 | 说明 | 来源 |
|------|------|------|
| 同名效果 | 按堆叠策略处理（刷新/叠层/取强） | §五.1 |
| 不同名效果 | 独立共存，互不影响 | §五.1 |

### 2.2 叠加时持续时间处理

| 策略 | 说明 | 来源 |
|------|------|------|
| 刷新持续 | 重置为新的持续时间 | §五.1 |
| 取最大持续 | 保留较长的持续时间 | §五.1 |
| 独立计时 | 每层独立计时 | §五.1 |

---

## 三、堆叠类别分类（8种）

### 3.1 StackType 枚举

| 类型 | 说明 | 典型场景 | 来源 |
|------|------|----------|------|
| `RefreshDuration` | 刷新持续时间，不叠加层数 | 同一Buff重复施加 | §五.2 |
| `StackIndependent` | 独立叠加，每层效果独立 | 中毒（每层独立掉血） | §五.2 |
| `StackDecay` | 衰减叠加，层数增加但效果递减 | 燃烧（每层效果减半） | §五.2 |
| `StackUndispellable` | 不可驱散叠加，层数决定驱散难度 | 某些Boss Buff | §五.2 |
| `TakeStrongest` | 取最强效果，弱效果被覆盖 | 同类增伤取最大值 | §五.2 |
| `CounterStack` | 反击类堆叠，只触发最高优先级 | 多个反击效果 | §五.3 |
| `ShieldMaxRefresh` | 护盾取最大值+刷新时长 | 同类护盾 | 补充6 |
| `ShieldIndependent` | 不同类护盾独立共存 | 物理/魔法/通用护盾 | 补充6 |

### 3.2 各类型详细规则

#### RefreshDuration
```yaml
stacking: RefreshDuration
max_stack: 1
duration_refresh: true
on_max_action: None
```

#### StackIndependent
```yaml
stacking: StackIndependent
max_stack: 9  # 中毒最多9层
duration_refresh: false
on_max_action: None
# 每层独立计算效果
```

#### StackDecay
```yaml
stacking: StackDecay
max_stack: 5
duration_refresh: true
on_max_action: None
# 每层效果 = base_effect * (0.5 ^ (stack - 1))
```

#### StackUndispellable
```yaml
stacking: StackUndispellable
max_stack: 3
duration_refresh: true
on_max_action: None
# 层数决定驱散难度：1层可驱散，2层需强驱散，3层不可驱散
```

#### TakeStrongest
```yaml
stacking: TakeStrongest
max_stack: 1
duration_refresh: true
on_max_action: None
# 保留效果值更大的
```

#### CounterStack
```yaml
stacking: CounterStack
max_stack: 8  # 反击次数上限
duration_refresh: false
on_max_action: None
# 多个反击只触发最高优先级
# 反击次数增加对每个反击单独生效
```

---

## 四、特殊机制堆叠

### 4.1 反击类堆叠（§五.3）

| 规则 | 说明 |
|------|------|
| 多个反击共存 | 只触发优先级最高的一个 |
| 反击次数增加 | 对每个反击单独生效，分别加次数 |
| 示例 | 回击+强力回击+反击次数+3，单回合最多8次反击 |

### 4.2 护盾类堆叠（补充6）

| 规则 | 说明 |
|------|------|
| 同类护盾 | 取最大值+刷新时长 |
| 不同类护盾 | 独立共存 |
| 吸收顺序 | 先通用→再物理/魔法专属 |

### 4.3 控制类堆叠（补充1）

| 规则 | 说明 |
|------|------|
| 高级控制覆盖低级 | 强控>硬控>软控 |
| 同级控制刷新 | 刷新持续时间，不叠加 |
| 控制递减 | 连续同类控制→持续时间衰减 |

---

## 五、Schema草案

```yaml
# stacking_config.ron
(
  stacking_rules: [
    (id: "refresh_duration", stack_type: RefreshDuration,
     max_stack: 1, on_max_action: None, duration_refresh: true),
    (id: "stack_independent", stack_type: StackIndependent,
     max_stack: 9, on_max_action: None, duration_refresh: false),
    (id: "stack_decay", stack_type: StackDecay,
     max_stack: 5, on_max_action: None, duration_refresh: true),
    (id: "take_strongest", stack_type: TakeStrongest,
     max_stack: 1, on_max_action: None, duration_refresh: true),
    (id: "counter_stack", stack_type: CounterStack,
     max_stack: 8, on_max_action: None, duration_refresh: false),
    (id: "shield_max_refresh", stack_type: ShieldMaxRefresh,
     max_stack: 1, on_max_action: None, duration_refresh: true),
  ],
)
```

---

## 六、依赖关系

| 依赖领域 | 依赖方向 | 说明 |
|----------|----------|------|
| Modifier | Stacking → Modifier | Stacking决定Modifier叠加方式 |
| Effect | Stacking → Effect | Stacking决定Effect叠加行为 |
| Tag | Stacking ← Tag | Tag决定互斥/叠加规则 |

---

## 七、Data Laws合规

| Law | 状态 | 说明 |
|-----|------|------|
| 006 | ✅ | Modifier叠加行为由Stacking策略决定，Modifier自身不处理 |
| 008 | ✅ | 所有堆叠行为统一归属Stacking，不散落在Ability/Effect/Modifier |
| 010 | ✅ | 堆叠规则确定性，无随机因素 |