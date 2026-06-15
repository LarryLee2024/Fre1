# Tag 领域 — 铃兰之剑数据提取

> 领域：Tag | 来源：78铃兰.md §补充1、补充2、§四 | 数据层：Definition + Instance

---

## 一、数据实体清单

### 1.1 TagDefinition（Definition层）

| 字段名 | 类型 | 约束 | 说明 | 来源 |
|--------|------|------|------|------|
| `id` | TagId | PK | 标签唯一标识 | 补充2 |
| `name_key` | String | - | 标签名称本地化Key | 补充2 + ADR-017 |
| `category` | Enum | damage_type/status/faction/mechanism | 标签分类 | 补充2 |
| `priority_weight` | u32 | ≥0 | 优先级权重（控制类使用） | 补充1 |
| `dispellable` | bool | - | 是否可驱散 | §四.1 |
| `reflectable` | bool | - | 是否可反弹 | 补充2 |
| `mutually_exclusive_with` | Vec<TagId> | - | 互斥标签列表 | 补充2 |

---

## 二、Tag四大分类

### 2.1 伤害类型Tag（category = damage_type）

用于伤害判定、抗性匹配。

| TagId | 名称 | 说明 | 来源 |
|-------|------|------|------|
| `dmg_physical` | 物理伤害 | 受物防减免，物理护盾可挡 | §二.1 |
| `dmg_magical` | 魔法伤害 | 受魔防减免，魔法护盾可挡 | §二.1 |
| `dmg_pierce` | 穿透伤害 | 无视防御，部分护盾可挡 | §二.1 |
| `dmg_true` | 真实伤害 | 无视防御无视护盾 | §二.1 |
| `dmg_fire` | 火焰 | 元素子类型 | 补充2 |
| `dmg_ice` | 寒冰 | 元素子类型 | 补充2 |

### 2.2 状态Tag（category = status）

用于状态判定、互斥判断。

| TagId | 名称 | 控制层级 | 可驱散 | 说明 | 来源 |
|-------|------|----------|--------|------|------|
| `buff` | 增益 | - | 是 | 橙色状态 | §四.1 |
| `debuff` | 减益 | - | 是 | 红色/紫色状态 | §四.1 |
| `special_state` | 特殊状态 | - | 否 | 蓝色/灰色，不可驱散 | §四.1 |
| `control_soft` | 软控 | 1 | 是 | 减速、命中降低、攻击降低 | 补充1 |
| `control_hard` | 硬控 | 2 | 是 | 定身、束缚、嘲讽 | 补充1 |
| `control_full` | 强控 | 3 | 是 | 眩晕、冰冻、石化 | 补充1 |
| `invincible` | 无敌 | - | 否 | 最高权限机制Tag | 补充2 |
| `untargetable` | 不可选中 | - | 否 | 屏蔽选中类效果 | 补充2 |

### 2.3 阵营/身份Tag（category = faction）

用于目标筛选、效果判定。

| TagId | 名称 | 说明 | 来源 |
|-------|------|------|------|
| `ally` | 友方 | 己方单位 | 补充2 |
| `enemy` | 敌方 | 对方单位 | 补充2 |
| `summon` | 召唤物 | 召唤产生的单位 | 补充2 |
| `boss` | Boss | Boss级单位 | 补充2 |
| `mechanical` | 机械 | 机械类单位 | 补充2 |

### 2.4 机制Tag（category = mechanism）

用于底层规则判定。

| TagId | 名称 | 说明 | 来源 |
|-------|------|------|------|
| `dispellable` | 可驱散 | 可被驱散效果移除 | 补充2 |
| `undispellable` | 不可驱散 | 只能靠时间衰减 | §四.3 |
| `reflectable` | 可反弹 | 可被反弹效果反射 | 补充2 |
| `untriggerable` | 不可触发 | 不触发被动 | 补充2 |
| `flying` | 飞行 | 不触发地面地形效果 | 补充2 |
| `grounded` | 地面 | 触发地面地形效果 | 补充2 |
| `no_cooldown_refresh` | 禁止刷新冷却 | 屏蔽冷却刷新效果 | 补充8 |

---

## 三、控制层级与免疫规则

### 3.1 控制层级定义（补充1）

| 层级 | 名称 | 效果 | 覆盖规则 |
|------|------|------|----------|
| 1 | 软控（削弱层） | 不限制行动，仅削弱属性 | 被硬控/强控覆盖 |
| 2 | 硬控（行动限制层） | 禁止移动/改变目标，可释放技能/普攻 | 被强控覆盖 |
| 3 | 强控（完全失能层） | 完全禁止所有行动 | 最高级 |

### 3.2 免疫分级

| 免疫类型 | 免疫范围 | 说明 |
|----------|----------|------|
| 免疫控制 | 硬控 + 强控 | 不免疫软控 |
| 免疫行动限制 | 仅硬控 | 定身类 |
| 免疫眩晕 | 仅单种状态 | 精确免疫 |

### 3.3 控制递减规则

- 连续对同一目标施加同类控制 → 持续时间衰减
- 避免无限控死

---

## 四、Tag继承与互斥规则

### 4.1 召唤物Tag继承

| 继承类型 | 继承的Tag分类 | 不继承的Tag分类 |
|----------|--------------|----------------|
| 召唤物 | 阵营Tag + 机制Tag | 状态Tag |

### 4.2 互斥Tag对

| Tag A | Tag B | 互斥效果 | 来源 |
|-------|-------|----------|------|
| `flying` | `grounded` | 飞行不触发地面地形效果 | 补充2 |
| `invincible` | 所有伤害Tag | 无敌屏蔽所有伤害 | 补充2 |
| `untargetable` | 所有选中效果 | 不可选中屏蔽选中 | 补充2 |
| `control_full` | `control_hard`/`control_soft` | 强控覆盖低级控制 | 补充1 |

### 4.3 最高权限Tag

- `invincible`：屏蔽所有伤害、选中类效果
- `untargetable`：屏蔽所有选中类效果

---

## 五、Schema草案

```yaml
# tag_config.ron
(
  tags: [
    # 伤害类型
    (id: "dmg_physical", category: DamageType, priority_weight: 0, dispellable: false, reflectable: true),
    (id: "dmg_magical", category: DamageType, priority_weight: 0, dispellable: false, reflectable: true),
    (id: "dmg_pierce", category: DamageType, priority_weight: 0, dispellable: false, reflectable: false),
    (id: "dmg_true", category: DamageType, priority_weight: 0, dispellable: false, reflectable: false),
    # 状态
    (id: "control_soft", category: Status, priority_weight: 1, dispellable: true, reflectable: false),
    (id: "control_hard", category: Status, priority_weight: 2, dispellable: true, reflectable: false),
    (id: "control_full", category: Status, priority_weight: 3, dispellable: true, reflectable: false),
    (id: "invincible", category: Mechanism, priority_weight: 99, dispellable: false, reflectable: false),
    (id: "untargetable", category: Mechanism, priority_weight: 98, dispellable: false, reflectable: false),
  ],
  mutual_exclusions: [
    (tag_a: "flying", tag_b: "grounded"),
    (tag_a: "control_full", tag_b: "control_hard"),
    (tag_a: "control_full", tag_b: "control_soft"),
  ],
)
```

---

## 六、依赖关系

| 依赖领域 | 依赖方向 | 说明 |
|----------|----------|------|
| Attribute | Tag → Attribute | Tag决定条件修正是否生效 |
| Trigger | Tag → Trigger | Tag作为触发条件 |
| Stacking | Tag → Stacking | Tag决定互斥/叠加规则 |
| Effect | Tag ← Effect | Effect可添加/移除Tag |
| Targeting | Tag → Targeting | Tag用于目标筛选 |

---

## 七、Data Laws合规

| Law | 状态 | 说明 |
|-----|------|------|
| 001 | ✅ | TagDefinition与TagInstance分离 |
| 003 | ✅ | Tag只引用ID，不重复定义 |
| 006 | ✅ | Tag不含业务逻辑，仅做判定标记 |
| 010 | ✅ | Tag优先级权重保证控制覆盖的确定性 |