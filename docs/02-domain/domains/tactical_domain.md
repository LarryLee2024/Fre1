---
id: 02-domain.tactical
title: Tactical（战术/网格）领域规则 v1.0
status: stable
owner: domain-designer
created: 2026-06-16
updated: 2026-06-19
tags:
  - domain
  - tactical
  - business-domain
---


## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| GridPosition | 战场网格坐标，定义单位在战场上的空间位置 | 负责：网格坐标的表示与转换，领域的 LocalizationKey（name_key/desc_key）；不负责：通行性/地形代价计算 |
| MovementPoints | 行动力，定义单位在当前回合/行动中的移动能力 | 负责：行动力的当前值/最大值/已消耗管理；不负责：行动力的消耗计算 |
| Facing | 单位朝向，影响背刺/夹击等战术判定 | 负责：朝向的维护与查询；不负责：朝向变化的条件判定 |
| FlankingState | 夹击状态，检测两个友方单位是否从对侧夹击一个敌方单位 | 负责：夹击的几何判定；不负责：夹击的效果计算（归 Combat 领域） |
| HighgroundState | 高地状态，检测单位是否处于有利/不利高度位置 | 负责：高度差的判定；不负责：高地的数值影响 |
| CoverState | 掩体状态，检测单位是否处于掩体保护中 | 负责：掩体的几何判定；不负责：掩体的 AC 加成计算 |
| BackstabState | 背刺状态，检测攻击者是否在目标的背后方向 | 负责：背刺的朝向判定；不负责：背刺的额外伤害计算 |
| PathData | 路径数据，从 A* 寻路结果转换而来的移动路径 | 负责：路径的格子序列和总消耗；不负责：寻路算法的实现 |

### 战术判定与领域职责边界

```
┌──────────────────────────────────────────────────────────────────┐
│ Tactical 领域（空间判定）                                           │
│                                                                  │
│  Tactical 负责以下判定（几何/空间层面）：                           │
│    - 判定"两个友方是否在目标的对侧" → 输出 FlankingState          │
│    - 判定"攻击者是否在目标的背后" → 输出 BackstabState            │
│    - 判定"目标与攻击者之间是否有障碍物" → 输出 CoverState          │
│    - 判定"单位 A 与单位 B 的高度差" → 输出 HighgroundState        │
│                                                                  │
│  Tactical 不负责以下计算（数值层面）：                              │
│    - 夹击的具体加成效果 → Combat 领域                             │
│    - 背刺的额外伤害骰 → Combat 领域                               │
│    - 掩体的 AC 加值 → Combat 领域                                 │
│    - 高地的命中优势 → Combat 领域                                 │
└──────────────────────────────────────────────────────────────────┘
```

### 已对齐项目术语

- **Unit**：每个战场单位拥有 GridPosition、Facing、MovementPoints 等组件
- **Tile**：地图格子，Terrain 领域控制格子的通行性与地形效果
- **Combat**：Combat 领域消费 Tactical 的判定结果计算伤害/命中修正
- **Targeting**：Targeting 计算射程时需要读取 GridPosition 和障碍数据

---

## 2. 战术状态机

### 单位移动状态

```
Idle（待命）
   │  [移动命令]
   ▼
Moving（移动中）
   │  [格消耗行动力]
   │  [路径上逐格前进]
   │
   ├──→ [到达目的地] → 停止，消耗行动力
   │
   └──→ [行动力耗尽/路径被阻] → 停在当前可达的最远格
           │
           ▼
      Stopped（已停止——位置已更新）
           │
           ▼
      Idle（回到待命）
```

### 朝向状态

```
CurrentFacing（当前朝向）
   │  [移动完成/攻击时自动调整]
   ▼
UpdatedFacing（朝向已更新）
   │  [背刺/夹击判定依赖朝向]
   ▼
FacingLocked（朝向被锁定——如被擒拿、石化等控制效果）
   │  [控制效果解除]
   ▼
CurrentFacing（恢复可转向）
```

---

## 3. 不变量（Invariants）

### 3.1 单位不可重叠
- **条件**：任何移动或单位放置时
- **不变量**：同一格内最多只能有一个单位（特殊规则如"堆叠"需显式声明）
- **违反后果类型**：🔴 规则失败
- **违反后果**：单位重叠导致选择/目标判定/碰撞检测异常

### 3.2 行动力非负
- **条件**：任何移动消耗后
- **不变量**：MovementPoints 当前值 >= 0，消耗超过剩余值时禁止移动
- **违反后果类型**：🔴 规则失败
- **违反后果**：行动力负值导致单位可以无限移动

### 3.3 路径连贯性
- **条件**：单位沿路径移动时
- **不变量**：路径中相邻格必须在网格上相邻（四连通或六连通，取决于地图类型）
- **违反后果类型**：🔴 程序错误
- **违反后果**：路径中存在跳跃格，单位"瞬移"越过中间格子

### 3.4 朝向与移动方向一致
- **条件**：单位完成移动后
- **不变量**：移动后单位的朝向必须更新为最后移动方向（除非被控制效果锁定朝向）
- **违反后果类型**：🔴 程序错误
- **违反后果**：移动后面朝错误方向，背刺/夹击判定失准

### 3.5 判定数据的帧一致性
- **条件**：单次战术判定（夹击/背刺/掩体）过程中
- **不变量**：判定使用的所有空间数据（位置/朝向/高度）必须来自同一时刻的快照
- **违反后果类型**：🔴 程序错误
- **违反后果**：判定数据跨帧导致结果不一致（如判定夹击时一方已移动）

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：Tactical 领域计算战术加成的数值（如夹击+2 命中） — 理由：数值加成归 Combat 领域，Tactical 只做几何判定
- 🟥 禁止：Tactical 领域修改单位的属性值 — 理由：属性修改通过 Modifier 管线，不走空间判定
- 🟥 禁止：移动过程中无视格子的通行性 — 理由：所有移动必须校验目标格的通行性（由 Terrain 领域提供）
- 🟥 禁止：单位在未消耗行动力的情况下移动 — 理由：任何格子的移动必须消耗对应的行动力
- 🟥 禁止：TacticalDef 中直接存储用户可见文本的自然语言 — 理由：必须使用 name_key/desc_key: LocalizationKey 引用。违反宪法 §22 Localization First。

---

## 5. 流程定义

### 5.1 移动

- **输入**：移动单位、目标网格坐标
- **处理**：
  1. 校验目标格是否可通行（委托 Terrain 领域）
  2. 计算移动路径（通过 A* 寻路，委托 Infra 层 pathfinding/）
  3. 累加路径总消耗（基础消耗 × 地形倍率）
  4. 检查单位当前 MovementPoints 是否足够
  5. 逐格移动：
     a. 消耗当前格的行动力
     b. 更新 GridPosition
     c. 触发 Terrain 效果（如进入毒池→触发中毒效果）
  6. 更新朝向为最后移动方向
  7. 发布 UnitMoved 事件
- **输出**：移动完成确认，UnitMoved 事件
- **失败处理**：行动力不足或路径受阻时移动失败，单位位置不变 → 这是**规则失败**（预期业务分支，移动资源或通行条件不满足）

### 5.2 夹击判定

- **输入**：目标单位、两个候选夹击友方单位
- **处理**：
  1. 获取三者的 GridPosition
  2. 计算两个友方单位与目标单位之间的角度差
  3. 如果角度差接近 180°（在阈值范围内，如 ±30°），判定为夹击
  4. 发布 FlankingDetected 事件
- **输出**：FlankingState（是否夹击、参与夹击的单位列表）
- **失败处理**：角度差超出阈值时返回"非夹击"状态 → 这是**规则失败**（预期业务分支，非所有位置关系都构成夹击）

### 5.3 背刺判定

- **输入**：攻击者、目标单位
- **处理**：
  1. 获取目标单位的当前 Facing
  2. 计算攻击者位置相对于目标朝向的方向
  3. 如果攻击者在目标的背面方向（180° ± 45°），判定为背刺
- **输出**：BackstabState（是否背刺）
- **失败处理**：目标朝向未知时默认判定为非背刺 → 这是**程序错误**（系统异常，朝向数据缺失应记 Bug）

### 5.4 掩体判定

- **输入**：攻击者、目标单位
- **处理**：
  1. 计算攻击者到目标的视线线
  2. 检测视线线上是否有障碍物格子（由 Terrain 领域提供）
  3. 根据障碍物覆盖程度判定掩体等级（无掩体/半掩/全掩）
- **输出**：CoverState（无掩体/半掩/四分之三掩）
- **失败处理**：视线计算异常时默认返回"无掩体" → 这是**程序错误**（系统异常，视线计算失败应记 Bug）

### 5.5 高地判定

- **输入**：单位 A、单位 B
- **处理**：
  1. 获取两者的网格层高
  2. 计算高度差
  3. 如果高度差 >= 2（层），判定为单位 A 有高地优势
- **输出**：HighgroundState（谁有高地优势、高度差数值）
- **失败处理**：高度数据缺失时返回"无高地差异" → 这是**程序错误**（系统异常，高度数据缺失应记 Bug）

---

## 6. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| UnitMoved | 单位完成移动时 | entity_id, from_position, to_position, path, remaining_mp | Combat（更新战斗范围）、Terrain（触发地形效果）、Trigger（触发移动相关触发器）、日志（LogCode: TAC001） |
| FlankingDetected | 夹击判定完成时 | target_id, flankers[2], angle | Combat（应用夹击加成）、UI（显示夹击指示器）、日志（LogCode: TAC002） |
| BackstabDetected | 背刺判定完成时 | attacker_id, target_id, position_relation | Combat（应用背刺加成）、UI（显示背刺指示）、日志（LogCode: TAC003） |
| CoverEvaluated | 掩体判定完成时 | target_id, cover_level（None/Half/ThreeQuarter） | Combat（应用掩体 AC 加成）、UI（显示掩体图标）、日志（LogCode: TAC004） |
| PositionChanged | 单位位置变更时（每格移动时） | entity_id, old_pos, new_pos | Tactical（更新战场布局）、Targeting（更新射程范围）、日志（LogCode: TAC005） |

### 事件订阅关系图

```
UnitMoved
    │
    ├──→ Terrain：检查新位置的地形效果
    ├──→ Combat：更新战斗距离/范围状态
    ├──→ Trigger：检查 借机攻击/移动触发 等触发器
    ├──→ Tactical：重新计算夹击/背刺/掩体状态
    ├──→ UI：更新单位位置显示
    └──→ Faction：检查进入友方/敌方领地

FlankingDetected / BackstabDetected / CoverEvaluated
    │
    ├──→ Combat：应用战术加成到伤害/命中计算
    └──→ UI：显示战斗预览（夹击/背刺/掩体指示器）
```

---

## 7. 与已有架构的对齐校验

- ✅ 架构边界：Tactical 域位于 `core/domains/tactical/`，components.rs 定义 GridPosition/MovementPoints/Facing/FlankingState，systems/ 实现移动/夹击/背刺/掩体系统，rules/ 定义移动/夹击/高地/背刺规则
- ✅ 职责明确：Tactical 只做空间判定，不做数值计算（Combat 的职责）。夹击判定 vs 夹击加成分离清晰
- ✅ 空间与数值分离：所有"判定"归 Tactical，所有"加成"归 Combat，防止领域膨胀
- ✅ 协作紧密：Tactical 的输出（FlankingState/BackstabState/CoverState/HighgroundState）是 Combat 伤害/命中计算的直接输入
- ✅ LocalizationKey：本领域涉及的用户可见文本使用 LocalizationKey 而非硬编码文本（宪法 §22）

---

## 8. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖移动、夹击、背刺、掩体、高地等 SRPG 核心战术场景
- [x] 所有不变量和约束条件已识别（5 条不变量）
- [x] 禁止事项已明确列出（4 条禁止）
- [x] 战术判定与数值加成的职责边界定义清晰
- [x] 每个操作有完整的流程定义（移动、夹击、背刺、掩体、高地判定）

---

## 9. Integration Facade 设计（Anti-Corruption Layer）

Tactical 域与 Capabilities 的交互通过 `integration/` 模块完成，采用 **Facade + SystemParam** 模式。

### 9.1 设计原则

- **Systems 不知道 Capabilities 内部类型**：通过 SystemParam + View Types 交互
- **Facade 是唯一访问 Capabilities 字段的地方**：当 AttributeContainer / ModifierContainer 内部变化时，只修改 facade.rs
- **按能力域拆分**：避免单文件膨胀为 God File

### 9.2 模块结构

```
integration/
├── mod.rs
└── movement/
    ├── mod.rs
    ├── facade.rs       # 业务语义 API
    ├── types.rs        # MovementCapabilityView, MP(newtype)
    └── system_param.rs # MovementCapabilityParam(SystemParam)
```

### 9.3 View Types

| 类型 | 说明 |
|------|------|
| `MP` | 移动力值（newtype，禁止裸 f32） |
| `MovementCapabilityView` | 移动能力评估报告（can_move, effective_points, max_points, modifier_summary） |
| `MovementModifierSummary` | 移动修正摘要（flat_bonus, multiplier, total_effect） |
| `MovementPrerequisiteError` | 移动前提条件错误 |
| `MovementCostError` | 移动成本错误 |

### 9.4 SystemParam

`MovementCapabilityParam` 封装所有 Capabilities 查询依赖：

```rust
fn movement_system(mov: MovementCapabilityParam) {
    let view = mov.build_view(entity, MovementType::Walk);
    if view.can_move { /* ... */ }
}
```

### 9.5 禁止事项

- 🟥 禁止 Systems 直接 `use` TagSet / AttributeContainer / ModifierContainer 进行字段访问
- 🟥 禁止在 `integration/` 外部访问 Capabilities 组件的内部字段
- 🟥 禁止将所有能力域塞入单个 `integration/` 文件

---

## 10. 六边形网格坐标系规则

### 10.1 坐标系统

Tactical 领域使用六边形网格坐标系统，由 `HexCoord` 类型（定义于 `shared/math/`）实现。

**轴向坐标（Axial Coordinate System）**：

- 使用 (q, r) 两个轴表示六边形网格中的位置
- q = 列坐标（沿水平方向）、r = 行坐标（沿斜向方向）
- 第三个立方体坐标 `s = -q - r` 隐式存在，用于距离计算
- 坐标类型为 `i32`，支持负值
- 方向：点顶朝向（pointy-top）

**坐标类型转换**：

| 操作 | 方法 | 说明 |
|------|------|------|
| 构造 | `HexCoord::new(q, r)` | 从 q, r 创建坐标 |
| 加法 | `a + b` | 坐标向量加法 |
| 减法 | `a - b` | 坐标向量减法 |
| 元组转换 | `HexCoord::from((q, r))` | 从 (i32, i32) 元组创建 |

### 10.2 距离公式

两个六边形坐标之间的立方体距离：

```
distance(a, b) = (|dq| + |dr| + |ds|) / 2
               = (|dq| + |dr| + |dq + dr|) / 2

其中 dq = b.q - a.q，dr = b.r - a.r，ds = -dq - dr
```

**距离示例**：

| 起点 | 终点 | 距离 |
|------|------|------|
| (0, 0) | (0, 0) | 0 |
| (0, 0) | (1, 0) | 1 |
| (0, 0) | (2, 1) | 3 |
| (-3, 2) | (1, -1) | 5 |

### 10.3 邻居方向

每个六边形有 6 个邻居，以轴向坐标偏移表示（点顶朝向）：

| 邻居 | q 偏移 | r 偏移 | 方向描述 |
|------|--------|--------|----------|
| N1 | +1 | 0 | 右（East） |
| N2 | -1 | 0 | 左（West） |
| N3 | 0 | +1 | 右下（Southeast） |
| N4 | 0 | -1 | 左上（Northwest） |
| N5 | +1 | -1 | 右上（Northeast） |
| N6 | -1 | +1 | 左下（Southwest） |

**邻居判定规则**：

- 两个坐标相邻当且仅当 `hex_distance(a, b) == 1`
- 每个 HexCoord 可通过 `neighbors()` 方法获取 6 个邻居数组

### 10.4 战术领域 HexCoord 使用不变量

以下不变量是对战术领域已有规则的补充，专门约束六边形网格坐标系下的行为：

#### 10.4.1 移动范围以 hex_distance 衡量

- **条件**：计算单位移动范围时
- **不变量**：移动范围以 `hex_distance` 计算的步数为单位。单位可移动到的格子必须是 `distance(start, target) <= remaining_mp` 的格子
- **违反后果类型**：🔴 程序错误
- **违反后果**：移动范围计算错误导致单位移动到不可达位置

#### 10.4.2 目标范围以 hex_distance 衡量

- **条件**：计算技能/攻击的目标范围时
- **不变量**：技能/攻击的射程以 `hex_distance` 计算。目标必须满足 `distance(attacker, target) <= range`
- **违反后果类型**：🔴 程序错误
- **违反后果**：射程判定错误导致越界攻击或无法选择有效目标

#### 10.4.3 路径步进必须为相邻格

- **条件**：单位沿路径移动时（路径由 PathData 表示）
- **不变量**：路径中相邻的坐标之间必须满足 `hex_distance(path[i], path[i+1]) == 1`。不允许跳跃
- **违反后果类型**：🔴 程序错误
- **违反后果**：路径中存在非相邻跳跃，导致单位表现瞬移

#### 10.4.4 坐标运算不可溢出

- **条件**：任何 HexCoord 加减运算时
- **不变量**：坐标值必须在 i32 范围内，防止算术溢出导致坐标取绕
- **违反后果类型**：🔴 程序错误
- **违反后果**：坐标溢出导致位置错误，单位出现在非预期位置

#### 10.4.5 夹击判别使用 hex_distance 角度

- **条件**：判定夹击时
- **不变量**：夹击的角度计算基于六边形网格的几何关系，使用 `hex_distance` 和位置向量关系确定是否形成 180° 夹击
- **违反后果类型**：🔴 程序错误
- **违反后果**：夹击判定错误导致实际站位和判定结果不一致

### 10.5 HexCoord 与 GridPosition 的关系

| 概念 | HexCoord | GridPosition |
|------|----------|--------------|
| 用途 | 六边形网格的底层数学坐标 | 战场单位的空间位置抽象 |
| 坐标系统 | 轴向坐标 (q, r) | 可适配不同网格类型 |
| 距离计算 | hex_distance 公式 | 委托 HexCoord |
| 邻居 | 6 个固定偏移 | 委托 HexCoord |
| 领域职责 | 纯数学运算（无 ECS 依赖） | 战场空间管理（ECS Component） |

> **核心原则**：GridPosition 内聚 HexCoord 作为其坐标表示。所有战术领域中的距离和方向计算最终委托给 HexCoord 的数学方法。

### 10.6 与已有架构的对齐校验

- ✅ HexCoord 位于 `shared/math/`，符合"shared 层零业务语义"的架构分层
- ✅ GridPosition 通过 HexCoord 实现距离/方向计算，不重复实现数学逻辑
- ✅ 距离公式、方向判定与 SRPG 标准六边形网格算法一致
- ✅ 所有战术定位计算都有明确的数学基础（axial + cubic 坐标系统）
- ✅ 不变量覆盖移动范围、目标范围、路径连贯性、坐标安全、夹击判定
