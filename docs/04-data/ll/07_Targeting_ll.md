# Targeting 领域 — 铃兰之剑数据提取

> 领域：Targeting | 来源：78铃兰.md §三.3、补充4 | 数据层：Definition

---

## 一、数据实体清单

### 1.1 TargetingDefinition（Definition层）

| 字段名 | 类型 | 约束 | 说明 | 来源 |
|--------|------|------|------|------|
| `id` | TargetingId | PK | 目标选择规则唯一标识 | §三.3 |
| `target_type` | TargetType | - | 目标类型 | §三.3 |
| `range` | u32 | ≥1 | 射程 | §三.3 |
| `aoe_shape` | Option<AoEShape> | - | AOE形状（可选） | §三.3 |
| `filters` | Vec<TargetFilter> | - | 目标筛选条件 | §三.3 |

---

## 二、目标类型分类

### 2.1 TargetType 枚举

| 类型 | 说明 | 典型技能 | 来源 |
|------|------|----------|------|
| `SingleEnemy` | 单体敌方 | 普攻、单体技能 | §三.3 |
| `SingleAlly` | 单体友方 | 治疗单体、加Buff | §三.3 |
| `Self` | 自身 | 自身增益、自我治疗 | §三.3 |
| `AoEEnemy` | 范围敌方 | 火球、旋风斩 | §三.3 |
| `AoEAlly` | 范围友方 | 群体治疗、群体增益 | §三.3 |
| `AllEnemy` | 全体敌方 | 全屏大招 | §三.3 |
| `DirectionalLine` | 方向线 | 冲锋、直线技能 | §三.3 |

---

## 三、目标筛选条件

### 3.1 TargetFilter

| 筛选维度 | 类型 | 说明 | 来源 |
|----------|------|------|------|
| 阵营筛选 | TagId | ally/enemy/neutral | §三.3 |
| 状态筛选 | TagId | 是否具备/禁止某Tag | §三.3 |
| 距离筛选 | (min, max) | 最近/最远优先 | §三.3 |

### 3.2 位移目标选择（补充4）

| 字段 | 类型 | 说明 | 来源 |
|------|------|------|------|
| `displacement_targeting` | Enum | active/forced | 补充4 |
| `path_check` | bool | 是否检查路径合法性 | 补充4 |
| `terrain_check` | bool | 是否检查地形可通行性 | 补充4 |
| `unit_collision` | Enum | block/pass_through/push | 补充4 |

位移路径规则：
- 主动位移：障碍/悬崖/水域完全阻挡，敌方阻挡路径，友方可穿越不可停留
- 强制位移：部分可穿越障碍，可推开路径上敌方单位

---

## 四、Schema草案

```yaml
# targeting_config.ron
(
  targetings: [
    (id: "single_enemy", target_type: SingleEnemy, range: 1,
     filters: [(faction: "enemy")]),
    (id: "aoe_enemy_3x3", target_type: AoEEnemy, range: 3,
     aoe_shape: Cross(radius: 1),
     filters: [(faction: "enemy")]),
    (id: "self", target_type: Self, range: 0, filters: []),
  ],
)
```

---

## 五、依赖关系

| 依赖领域 | 依赖方向 | 说明 |
|----------|----------|------|
| Ability | Targeting ← Ability | Ability引用目标选择规则 |
| Tag | Targeting ← Tag | Tag用于目标筛选 |
| Effect | Targeting ← Effect | 位移Effect需要地形交互 |

---

## 六、Data Laws合规

| Law | 状态 | 说明 |
|-----|------|------|
| 003 | ✅ | 目标选择规则通过ID引用，不重复定义 |
| 010 | ✅ | 目标选择规则确定性，无随机因素 |