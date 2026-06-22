---
id: 02-domain.tactical
title: Tactical（战术/网格）领域规则 v1.0
status: stable
owner: domain-designer
created: 2026-06-16
updated: 2026-06-23
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

---

## 11. 单位选择规则（Unit Selection）

> **版本**: 1.0
> **状态**: Draft
> **适用**: 玩家通过 UI 与战场单位进行交互时的选择逻辑
> **前置依赖**: ADR-PICK-000

---

### 11.1 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Selection | 单位选择机制，定义玩家如何从战场中选择一个单位作为当前操控对象 | 负责：选择状态的维护、可选性判定、选择状态切换；不负责：选择后的行动执行 |
| PickTarget | 当前鼠标/光标指向的内容，描述玩家"正在指向什么" | 负责：PickTarget 的类型定义与有效性判定；不负责：选择状态转移后的动作编排 |
| PickContext | 选择上下文模式，定义当前交互阶段下哪些 PickTarget 有效及其语义 | 负责：模式定义与模式间转换规则；不负责：具体的目标校验逻辑 |
| Hovered | 鼠标悬停状态——光标单位经过，纯 UI 预览无领域状态变更 | 负责：UI 层面的高亮 / 信息预览；不负责：选择状态管理 |
| Focused | 导航焦点状态——键盘/手柄模式下的光标位置，纯 UI 导航无领域状态变更 | 负责：非鼠标输入模式的导航焦点；不负责：选择状态管理 |
| Selected | 选中状态——玩家已决定由该单位执行下一步行动 | 负责：当前行动单位的标识；不负责：行动的选择与执行 |
| Targeted | 目标锁定状态——在行动上下文中，某个单位/位置被标记为潜在目标 | 负责：目标候选的标记；不负责：目标合法性的完整校验（委托 Targeting 能力） |
| Activated | 已确认执行状态——玩家已确认行动与目标，进入不可回退的执行阶段 | 负责：执行锁定标志；不负责：执行过程本身 |
| Selectable | 可选中标志，标记当前帧中玩家可以选中的单位集合 | 负责：可选性判定结果的缓存；不负责：选择状态的持久化 |
| CrosshairMode | 准星模式——正在选择目标的交互阶段，视野受限（只显示有效目标/区域） | 负责：目标选择的交互约束；不负责：目标校验 |
| SelectionHistory | 选择历史，记录当前交互循环中的选择路径（用于返回/取消操作） | 负责：选择路径的记录与回溯；不负责：选择的业务含义 |

### 11.2 五级状态层级（核心原则）

以下五个状态严格区分语义，层级不可混淆：

```
层级 1: Hovered（悬停）   —— 光标经过，瞬时 UI 状态（无领域承诺）
层级 2: Focused（焦点）   —— 键盘/手柄导航焦点（无领域承诺）
层级 3: Selected（选中）  —— 当前行动单位（有领域承诺）
层级 4: Targeted（目标）  —— 当前行动的潜在目标（待确认）
层级 5: Activated（激活） —— 行动已确认执行（不可回退）
```

**层级间的关系**：

- Hovered 和 Focused 是互斥的输入模式（鼠标 vs 键盘/手柄），同一时间只有一个活跃
- Selected 始终有且只有一个单位（Idle 状态下为零）
- Targeted 可以为零到多个（AoE 技能可以有多个目标同时被锁定）
- Activated 是最终的 Commit 状态，一旦进入不可回退到更低层级
- 高层级的状态变更必须从相邻低层级进入（禁止"瞬跳"）

**职责归属**：

```
┌──────────────────────────────────────────────────────────────────────┐
│ Presentation 层（UI）                                                 │
│   - 管理 Hovered / Focused 状态                                      │
│   - 将鼠标点击 / 手柄按键 发送为输入事件                              │
│   - 查询 Tactical 领域的 Selected / Targeted / Activated 状态用于显示  │
└──────────────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────────────┐
│ Tactical 领域（业务规则）                                              │
│   - 管理 Selected / Targeted / Activated 状态                        │
│   - 判定哪些单位可以被选中 / 可以被目标锁定                            │
│   - 控制选择状态机的所有状态转换规则                                   │
└──────────────────────────────────────────────────────────────────────┘
```

---

### 11.3 选择状态机（Selection State Machine）

#### 主状态机

```
Browsing（浏览）
  │ [点击 Selectable 己方单位]
  ├──→ UnitSelected（选中单位——"谁行动"）
  │
  │ [点击非 Selectable 单位（敌方/已行动/死亡）]
  └──→ 查看信息，不转移状态（信息预览）
  │
  │ [点击空地]
  └──→ 取消选中（无操作/关闭信息面板）

UnitSelected（单位已选中）
  │ [打开行动菜单] ──→ ActionSelect（"什么行动"）
  │ [选择另一 Selectable 单位] ──→ UnitSelected（切换选中单位）
  │ [点击空地 / Escape] ──→ Browsing（取消选中 / Deselect）
  │ [右键取消] ──→ Browsing

ActionSelect（行动选择中）
  │ [选择攻击 / 技能 / 物品] ──→ TargetSelect（上下文=攻击/技能/物品）
  │ [选择待命 / 结束回合] ──→ AwaitingExecution（无目标，直接执行）
  │ [取消 / Escape] ──→ UnitSelected（返回单位选择）

TargetSelect（目标选择中——"目标是谁/在哪"）
  │ [点击有效目标（单位/格子）] ──→ TargetLocked（目标已锁定，待确认）
  │ [移动光标/悬停不同目标] → 仍在 TargetSelect（切换候选目标）
  │ [取消 / Escape] ──→ ActionSelect（返回行动选择）

TargetLocked（目标已锁定，等待玩家确认）
  │ [确认 Commit] ──→ AwaitingExecution（进入执行）
  │ [点击另一有效目标] ──→ TargetLocked（切换锁定目标）
  │ [取消 / Escape] ──→ TargetSelect（回到选择阶段）

AwaitingExecution（待执行）
  │ [执行完成] ──→ Browsing（回到浏览）
  │ [执行失败] ──→ Browsing（回到浏览，按失败类型处理消耗回退）
```

#### 状态转换规则表

| 转换 | 触发条件 | 动作 | 不可违反的不变量 |
|------|---------|------|----------------|
| Browsing → UnitSelected | 点击 Selectable 的己方单位 | 设置当前选中单位，发布 UnitSelected 事件 | 11.6.1, 11.6.2 |
| UnitSelected → Browsing | 点击空地 / Escape / 右键 | 清除选中单位，发布 SelectionChanged 事件 | 11.6.4 |
| UnitSelected → ActionSelect | 从行动菜单中选择行动 | 记录选中的 Action（Skill/Item/Wait），发布 ActionPicked 事件 | 11.6.3 |
| ActionSelect → UnitSelected | 取消/Escape | 清除行动选择，回到单位选择 | — |
| ActionSelect → TargetSelect | 选择需要目标的行动 | 初始化 TargetSelect 上下文（PickContext=TargetSelect），设置范围约束 | 11.6.5 |
| TargetSelect → TargetLocked | 点击有效目标 | 锁定目标，发布 TargetLocked 事件，等待玩家确认 | 11.6.7 |
| TargetLocked → AwaitingExecution | 玩家确认 Commit | 发布 TargetConfirmed 事件，过渡到执行阶段 | 11.6.8, 11.6.9 |
| TargetLocked → TargetSelect | 取消/Escape | 解锁目标，回到选择阶段 | — |
| AwaitingExecution → Browsing | 执行完成 | 清理选择状态，发布 UnitActionCompleted 事件 | 11.6.9 |

**禁止的转换**：

- 🟥 Browsing → TargetSelect（跳过单位选择和行动选择，直接进入目标选择）
- 🟥 Browsing → AwaitingExecution（跳过所有中间选择步骤）
- 🟥 TargetSelect → AwaitingExecution（跳过 TargetLocked 的确认步骤）
- 🟥 AwaitingExecution → TargetSelect（执行不可回退）

---

### 11.4 可选单位规则（Selectability Rules）

#### 11.4.1 可作为"选中单位"的条件（谁可以被选中作为行动者）

单位可以被选中（进入 UnitSelected 状态）当且仅当 **全部** 以下条件满足：

1. **己方单位** — 属于当前操控玩家/阵营（由 Faction 领域判定）
2. **存活** — 未被标记为死亡/阵亡（不具有 Dead Tag）
3. **可行动** — 在当前回合尚未消耗完所有行动机会（未执行"待命"或同等结束回合操作）
4. **未被完全控制** — 不处于眩晕/石化/魅惑等导致无法行动的控制状态（不具有对应的 Control Tag）
5. **在当前回合的行动序列中** — 在当前 Round 中轮到该单位行动或行动顺序已到（由 Combat 领域的 TurnOrder 判定）

#### 11.4.2 不可选中的情况

以下情况下单位 **不可被选中**（即使尝试点击也不发生 Browsing → UnitSelected 转换）：

| 情况 | 原因 | 反馈类型 |
|------|------|---------|
| 单位已死亡/销毁 | 实体不存在或无 Dead Tag 但血量为零 | 🔴 规则失败（不应该出现在符合条件中） |
| 单位是敌方阵营 | 玩家不能操控敌方单位 | 规则失败（预期业务分支） |
| 单位是中立方/不可操控 | 不属于任何玩家阵营 | 规则失败（预期业务分支） |
| 单位已行动完毕（已结束回合） | 本回合该单位已完成所有行动 | 规则失败（预期业务分支） |
| 单位被眩晕/石化 | 控制状态阻止行动 | 规则失败（预期业务分支） |
| 单位不在当前操控者的回合 | 回合未到或已过 | 规则失败（预期业务分支） |

#### 11.4.3 可作为"目标"的条件（谁可以被锁定为行动目标）

当单位处于 TargetSelect 状态时，可以作为有效目标的条件取决于当前行动的 Targeting 规则。通用的目标可锁定的最低条件（任何行动都必须满足）：

1. **单位存活** 或 **行动允许锁定尸体**（如复活术允许死去的单位作为目标）
2. **满足阵营要求** — 与 TargetType（Self/Ally/Enemy/Any）一致
3. **在射程范围内** — 以 hex_distance 计算的距离不超过行动声明的最大射程
4. **在视线范围内** — 施法者到目标之间无完全阻挡的障碍物（除非行动声明"无视视线"）
5. **单位未被隐藏/消失** — 不具有不可见/潜行状态（除非行动声明"可探测隐藏目标"）
6. **不是行动者自身** — 除非 TargetType 允许 Self 或行动声明可作用自身

> 以上条件的具体校验委托给 **Targeting 能力领域**，Tactical 领域只负责在 TargetSelect 状态下接收校验结果并反馈给玩家。

---

### 11.5 PickTarget 领域语义

`PickTarget` 枚举定义玩家当前"指向/选择"的内容类型：

| 变体 | 含义 | 有效使用场景 | 无效场景示例 |
|------|------|-------------|-------------|
| `Unit(UnitId)` | 指向一个具体单位 | Browsing（选中）、TargetSelect（锁定目标） | ActionSelect（行动菜单中不选单位） |
| `Tile(TilePos)` | 指向一个地图格子 | TargetSelect（AoE 放置位置） | ActionSelect（行动菜单中不选格子） |
| `Skill(SkillId)` | 指向一个技能（从行动栏选取） | ActionSelect（选择要使用的技能） | TargetSelect（目标选择中不选技能） |
| `Item(ItemId)` | 指向一个物品（从背包选取） | ActionSelect（选择要使用的物品） | TargetSelect（目标选择中不选物品） |

**PickTarget 有效性判定规则**：

```
PickTarget 在以下条件下为"有效"：
  Browsing 下: Unit(己方可选中) → 有效选中
               Unit(敌方/不可选中) → 只查看信息，不选中
               Tile(任何) → 取消选中
  ActionSelect 下: Skill(可用) → 有效行动
                   Item(可用) → 有效行动
                   Unit/Tile → 无效（忽略）
  TargetSelect 下: Unit(符合目标条件) → 有效目标
                   Tile(符合位置条件) → 有效位置（AoE）
                   Skill/Item → 无效（忽略）
```

---

### 11.6 PickContext 状态机（选择上下文模式）

PickContext 定义当前交互阶段的选择行为模式，决定哪些 PickTarget 变体被视为有效以及选择后的行为：

```
Normal（浏览模式）
  │ [玩家选中己方单位]
  ├──→ UnitSelected → PickContext 变为 UnitSelected
  │
  │ [玩家点击敌方/不可选单位]
  └──→ 停留在 Normal（仅信息预览）
  
UnitSelected（单位已选中模式）
  │ [玩家打开行动菜单]
  ├──→ PickContext 变为 ActionSelect
  │
  │ [玩家选择另一单位]
  └──→ 切换选中的单位（仍在 UnitSelected 模式）
  │
  │ [玩家取消]
  └──→ 回到 Normal

ActionSelect（行动选择模式）
  │ [选择需要目标的行动]
  ├──→ PickContext 变为 TargetSelect（带约束）
  │    约束包括：有效目标类型、范围形状、最大/最小目标数
  │
  │ [选择不需要目标的行动（待命/自我增益）]
  └──→ PickContext 变为 AwaitingExecution
  
TargetSelect（目标选择模式——Crosshair 模式）
  │ [玩家在有效范围内移动]
  ├──→ 实时更新目标预览（高亮影响范围内的单位和区域）
  │
  │ [玩家锁定目标]
  └──→ PickContext 变为 TargetLocked
  
TargetLocked（目标已锁定）
  │ [玩家确认]
  ├──→ PickContext 变为 AwaitingExecution
  │
  │ [玩家取消]
  └──→ 回到 TargetSelect
```

**PickContext 模式约束表**：

| PickContext | 有效 PickTarget | 视觉预览 | 行动约束 |
|-------------|----------------|---------|---------|
| Normal | Unit（查看/选中）、Tile（查看） | 悬停高亮 | 无行动约束 |
| UnitSelected | Unit（切换选中）、Tile（取消） | 选中高亮、行动菜单 | 行动选择菜单打开 |
| ActionSelect | Skill、Item | 技能/物品图标高亮 | 技能可用性/冷却/消耗决定可选列表 |
| TargetSelect | Unit（目标单位）、Tile（目标位置） | 有效目标高亮、无效目标灰色、范围预览 | 行动 TargetType + TargetShape 决定有效目标范围 |
| TargetLocked | Unit（切换锁定）、Tile（切换位置） | 锁定目标高亮、确认按钮 | 仅可在有效目标间切换 |
| AwaitingExecution | 无（不响应输入） | 执行动画/进度 | 执行过程不可打断 |

---

### 11.7 不变量（Invariants）

#### 11.7.1 选择互斥
- **条件**：任何时刻
- **不变量**：全系统最多同时只有一个单位的 Selected 状态为 true。选中单位 A 时，之前选中的单位 B 必须自动取消选中
- **违反后果**：两个单位同时处于选中状态，输入/UI 显示冲突
- **违反后果类型**：🔴 程序错误

#### 11.7.2 选择右边界
- **条件**：选中一个单位时（Browsing → UnitSelected）
- **不变量**：被选中的单位必须满足 11.4.1 全部条件（己方/存活/可行动/未受控/在回合中）
- **违反后果**：无法行动的单位被选中，玩家尝试操作后失败
- **违反后果类型**：🔴 规则失败

#### 11.7.3 选中状态与行动菜单绑定
- **条件**：UnitSelected 状态下
- **不变量**：单位被选中时必须立即展示该单位的可用行动列表（技能/物品/待命），不可无行动停留在选中状态
- **违反后果**：选中单位后玩家无任何可操作选项，交互卡死
- **违反后果类型**：🔴 程序错误

#### 11.7.4 取消选中无副作用
- **条件**：从 UnitSelected 取消选中回到 Browsing 时
- **不变量**：取消选中不会取消或回退该单位已执行的任何行动。已消耗的资源（行动点等）不可恢复
- **违反后果**：取消选中后被错误地回退行动资源，产生无限操作
- **违反后果类型**：🔴 程序错误

#### 11.7.5 目标选择与行动类型一致
- **条件**：进入 TargetSelect 时
- **不变量**：TargetSelect 模式下的有效目标范围必须与所选行动的 Targeting 规则严格一致，不可扩大或缩小
- **违反后果**：玩家可选中不符合行动规则的目标，执行阶段校验失败
- **违反后果类型**：🔴 规则失败

#### 11.7.6 TargetSelect 下的有效目标不可为空
- **条件**：进入 TargetSelect 时
- **不变量**：如果行动的 Targeting 规则下没有任何合法目标（无范围内的敌方、无可作用的单位等），则不允许进入 TargetSelect 状态，直接反馈"无可用目标"
- **违反后果**：玩家进入 TargetSelect 后无可选目标，交互卡死
- **违反后果类型**：🔴 规则失败

#### 11.7.7 目标锁定不可跳过确认
- **条件**：TargetSelect → AwaitingExecution 转换中
- **不变量**：必须经过 TargetLocked 状态（确认步骤），禁止从 TargetSelect 直接跳转到 AwaitingExecution
- **违反后果**：玩家选择目标的意图未确认，导致误操作
- **违反后果类型**：🔴 规则失败

#### 11.7.8 执行阶段不可回退
- **条件**：进入 AwaitingExecution 后
- **不变量**：一旦进入 AwaitingExecution 状态，任何操作（除执行完成/执行失败回调外）不可将状态回退到 TargetSelect 或更低层级
- **违反后果**：执行阶段被回退导致行动重复执行或状态不一致
- **违反后果类型**：🔴 规则失败

#### 11.7.9 目标确认后目标集合不可变
- **条件**：TargetConfirmed 事件发布后
- **不变量**：TargetConfirmed 事件携带的目标列表在执行完成前不可修改。执行系统收到的目标列表必须与确认时完全一致
- **违反后果**：执行时的目标与确认时的目标不一致，技能效果作用到错误目标
- **违反后果类型**：🔴 规则失败

#### 11.7.10 PickTarget 有效性检查必须实时
- **条件**：玩家移动光标/指针时
- **不变量**：Hovered 状态下的 PickTarget 有效性检查必须基于当前帧的最新 Tactical 状态（位置/朝向/状态），不能使用缓存在上一帧的判断结果
- **违反后果**：玩家看到有效高亮但点击时目标已无效（位置移动/状态变化）
- **违反后果类型**：🔴 程序错误

---

### 11.8 禁止事项（Forbidden）

- 🟥 禁止：跳过 UnitSelected 或 ActionSelect 直接进入 TargetSelect — 理由：必须经过"谁行动"→"什么行动"→"目标是谁"的完整选择链，不可跳跃
- 🟥 禁止：TargetSelect 状态下将不可选目标显示为可选 — 理由：无效目标必须用灰色/禁用状态明确区分；视觉欺骗导致玩家误操作
- 🟥 禁止：TargetSelect 状态下允许选择不符合 PickContext 约束的 PickTarget — 理由：不同模式下 PickTarget 的有效性不同，必须根据 PickContext 过滤
- 🟥 禁止：执行阶段（AwaitingExecution）响应任何选择输入 — 理由：执行阶段锁定所有输入，直到执行完成回调
- 🟥 禁止：选中状态穿越回合边界 — 理由：当前单位执行完毕后，其选中状态必须在回合结束前清除。新回合开始后必须从 Browsing 重新开始
- 🟥 禁止：在 Browsing 模式下直接选取技能/物品作为 PickTarget — 理由：技能/物品选择必须发生在 UnitSelected → ActionSelect 状态下
- 🟥 禁止：Hovered/Focused 状态下触发任何领域状态变更 — 理由：Hovered 和 Focused 是纯 UI 状态，不产生领域承诺。将所有悬停/聚焦操作限制在 Presentation 层
- 🟥 禁止：未持有 Dead Tag 但血量归零的单位被选中 — 理由：血量归零的单位应立即添加 Dead Tag，不可处于"灰色地带"被误选中
- 🟥 禁止：TacticalDef 中直接存储用户可见文本的自然语言 — 理由：必须使用 name_key/desc_key: LocalizationKey 引用。违反宪法 §22 Localization First。

---

### 11.9 流程定义

#### 11.9.1 单位选中流程

- **输入**：点击事件（物理点击位置 → 转换为 PickTarget::Unit(UnitId)）
- **处理**：
  1. Presentation 层将物理点击转换为 PickTarget::Unit(UnitId)
  2. Tactical 域收到 PickTarget，开始校验（不变量 11.7.2 — 检查 Selectability 条件）
  3. 校验通过：执行 Browsing → UnitSelected 转换
  4. 校验不通过：停留在 Browsing，返回不可选原因（用于 UI 提示）
  5. 发布 UnitSelected 事件
- **输出**：选中状态确认，UnitSelected 事件
- **失败处理**：选中条件不满足时状态不变，系统记录失败原因但不报错 → 这是 **规则失败**（预期业务分支，不是所有点击都产生选中）

#### 11.9.2 目标选择流程

- **输入**：当前 ActionSelect 状态下的行动类型（SkillId/ItemId）、PickTarget 选择
- **处理**：
  1. 根据行动类型确定 TargetType 和 TargetShape（委派 Targeting 能力领域）
  2. 校验行动是否有合法目标（不变量 11.7.6 — 无合法目标直接拒绝进入 TargetSelect）
  3. 进入 TargetSelect 状态，设置 PickContext 约束
  4. 显示有效目标范围（委托 Targeting 领域计算范围网格）
  5. PickTarget 实时变化时，委派 Targeting 校验目标合法性
  6. 玩家点击有效目标 → 进入 TargetLocked
  7. 玩家确认 → 发布 TargetConfirmed 事件
- **输出**：TargetConfirmed 事件（携带目标列表）
- **失败处理**：无合法目标时停留在 ActionSelect，返回"无可用目标"反馈 → 这是 **规则失败**（预期业务分支，不是所有行动都有合法目标）

#### 11.9.3 取消选择流程

- **输入**：取消操作（点击空地 / Escape / 右键）
- **处理**：
  1. 根据当前状态确定取消目标：
     - TargetLocked → TargetSelect（解锁目标）
     - TargetSelect → ActionSelect（放弃目标选择）
     - ActionSelect → UnitSelected（放弃行动选择）
     - UnitSelected → Browsing（取消选中）
  2. 清除当前状态相关的临时数据（目标锁定、范围预览等）
  3. 发布 SelectionChanged 事件
- **输出**：SelectionChanged 事件
- **失败处理**：Browsing 状态下取消操作无效果 → 这是 **规则失败**（预期业务分支，Browsing 已是最外层）

#### 11.9.4 单位行动完成流程

- **输入**：执行系统回调（执行完成或执行失败）
- **处理**：
  1. 收到执行完成/失败回调
  2. 清理该单位的选中状态（Selected = false）
  3. 清理所有关联的目标锁定状态
  4. 回到 Browsing 状态
  5. 如果该单位本回合已无行动机会，标记为"已行动"
  6. 发布 UnitActionCompleted 事件
- **输出**：UnitActionCompleted 事件，Browsing 状态
- **失败处理**：执行失败时仍清除选中状态（消耗是否回退归执行系统决策，选择系统不回退选择状态） → 这是 **程序错误**（选择系统不应依赖执行系统的成功/失败来决定状态清理）

---

### 11.10 领域事件

| 事件名 | 触发时机 | 数据载荷 | 订阅者 | 读写分类 |
|--------|---------|---------|--------|---------|
| UnitClicked | Presentation 层检测到单位被点击时 | pick_target: PickTarget, click_type: LeftClick/RightClick | Tactical（翻译为业务选择决策） | Read（Presentation → Tactical 查询式通知） |
| UnitSelected | Browsing → UnitSelected 转换成功时 | unit_id: UnitId, faction: FactionId | UI（更新选中高亮）、Trigger（检查选中相关触发条件）、日志（LogCode: TAC011） | Write（Tactical → 各订阅者） |
| UnitDeselected | UnitSelected → Browsing 转换时 | unit_id: UnitId, reason: CancelReason（Escape/ClickOther/Timeout） | UI（清除选中高亮）、日志（LogCode: TAC012） | Write（Tactical → 各订阅者） |
| SelectionChanged | 任何选择状态变更时（选中/取消/切换） | previous_state: SelectionState, current_state: SelectionState, involved_entity: Option\<EntityId\> | UI（全状态同步）、Presentation（更新输入模式） | Write（Tactical → 各订阅者） |
| ActionPicked | ActionSelect 完成，行动已选定时 | unit_id: UnitId, action_type: ActionType（Skill/Item/Wait）, action_id: Option\<Id\> | Tactical（进入 TargetSelect 或 AwaitingExecution）、UI（显示行动预览）、日志（LogCode: TAC013） | Write（Tactical → 各订阅者） |
| TargetLocked | TargetSelect → TargetLocked 时 | unit_id: UnitId, target: PickTarget, action_context: ActionContext | UI（显示确认按钮 / 锁定高亮） | Write（Tactical → UI） |
| TargetConfirmed | TargetLocked → AwaitingExecution 时 | unit_id: UnitId, action_type: ActionType, action_id: Option\<Id\>, target_list: Vec\<PickTarget\> | Execution（开始执行）、Trigger（检查目标相关触发条件）、Ability（继续技能执行流程）、日志（LogCode: TAC014） | Write（Tactical → 各订阅者） |
| UnitActionCompleted | 单位行动执行完毕时 | unit_id: UnitId, action_type: ActionType, result: ActionResult（Success/Failed） | Combat（推进回合）、Trigger（检查行动完成触发条件）、Progression（记录行动经验）、日志（LogCode: TAC015） | Write（Tactical → 各订阅者） |
| NoValidTarget | 进入 TargetSelect 但无合法目标时 | unit_id: UnitId, action_id: Option\<Id\>, reason: NoValidTargetReason | UI（显示"无可用目标"提示）、日志（LogCode: TAC016） | Write（Tactical → UI） |

**事件读写区分**：

- **Read 路径**（输入事件，不产生领域状态变更）：
  - `UnitClicked` — Presentation 层通知 Tactical 发生了点击，Tactical 据此决定是否状态变更。属于 Quadrant 2（查询式通知）
  
- **Write 路径**（产生领域状态变更）：
  - `UnitSelected` / `UnitDeselected` / `SelectionChanged` — 选择状态变更，下游订阅者据此更新 UI 和触发后续逻辑
  - `ActionPicked` — 行动选定，Tactical 内部消费推进状态机
  - `TargetLocked` / `TargetConfirmed` — 目标锁定/确认，是 Selection → Execution 的关键桥接事件
  - `UnitActionCompleted` — 行动完成，Combat 领域推进回合

**事件订阅关系图**：

```
UnitClicked (Presentation → Tactical)
    │
    └──→ Tactical：判断是否触发选择状态变更

UnitSelected
    │
    ├──→ UI：更新选中高亮，打开行动菜单
    ├──→ Trigger：检查是否有"被选中时触发"的效果
    └──→ 日志 (TAC011)

TargetConfirmed
    │
    ├──→ Execution：开始执行行动效果
    ├──→ Trigger：检查是否有"被选为目标时触发"的效果
    ├──→ Ability：继续技能激活流程 → 调用 Targeting → Execution
    └──→ 日志 (TAC014)

UnitActionCompleted
    │
    ├──→ Combat：推进回合顺序 / 检查胜负条件
    ├──→ Trigger：检查"行动完成后触发"的效果
    ├──→ Progression：记录技能使用次数
    ├──→ UI：清除选中状态，恢复浏览模式
    └──→ 日志 (TAC015)
```

---

### 11.11 与已有规则的关系

#### 11.11.1 与 Targeting 能力领域的关系（选择 vs 目标选择）

```
┌──────────────────────────────────────────────┐
│ Tactical 领域（单位选择 — 本节定义）            │
│                                                │
│  职责：选择"谁行动"，确定行动后选择"目标是谁"     │
│  状态管理：Browsing → UnitSelected →           │
│            ActionSelect → TargetSelect          │
│                                                │
│  TargetSelect 状态下：                           │
│   - 确定玩家正在选择目标的交互模式                 │
│   - 接收玩家的 PickTarget 输入                   │
│   - 委派 Targeting 能力校验 PickTarget 合法性    │
│   - 根据校验结果更新可视状态（有效/无效高亮）      │
│   - 玩家确认后发布 TargetConfirmed               │
└──────────────────────┬───────────────────────┘
                       │ 委派目标合法性校验
                       ▼
┌──────────────────────────────────────────────┐
│ Targeting 能力领域（目标校验）                   │
│                                                │
│  职责：定义目标筛选规则、范围计算、合法性校验      │
│  输入：施法者、行动定义（TargetType+TargetShape）、  │
│        候选目标                                  │
│  输出：目标是否合法、影响范围网格                   │
│                                                │
│  Tactical 在 TargetSelect 状态下：               │
│   - 调用 Targeting 的 Selector + Validator      │
│   - 不读取 Targeting 内部字段（通过 Facade）      │
└──────────────────────────────────────────────────┘
```

**职责划分总结**：

| 维度 | Tactical（选择） | Targeting（目标校验） |
|------|-----------------|---------------------|
| 谁控制"选择模式"的进入与退出 | 是 | 否 |
| 谁计算合法目标范围 | 否（委派） | 是 |
| 谁校验单个目标的合法性 | 否（委派） | 是 |
| 谁控制选中的 UI 反馈 | 是（通过 SelectionChanged 事件） | 否 |
| 谁管理确认/取消流程 | 是 | 否 |
| 谁定义 TargetType/TargetShape | 否 | 是 |
| 谁记录选择历史 | 是 | 否 |

#### 11.11.2 与 Combat 领域（TurnQueue）的关系

```
Combat 领域                          Tactical 领域
─────────────────                    ─────────────────

TurnOrder 决定 "当前是哪个玩家/       Selection 决定 "该玩家控制下的
阵营的回合"                           哪个单位被选中行动"
       │                                      │
       │ 发布 RoundStarted / UnitTurnStarted  │
       └─────────────────────────────────→ 单位被 Browsing 过滤：
                                             只有轮到行动的单位可被选中
                                             (Selectability 条件 §11.4.1-5)
                                             
                                             单位行动完毕后发布
       ←───────────────────────────────────── UnitActionCompleted
       Combat 推进到下一个单位/回合
```

**约束关系**：

- 单位选择只能在 **当前回合的当前单位** 可行动期间发生
- 单位行动完成后，Selection 状态必须清除（回到 Browsing）
- 回合结束（RoundEnd）强制清除所有 Selection 状态（包括 TargetLocked）
- 新的 RoundStart 发布后，所有单位回到未行动状态，Selection 从 Browsing 重新开始

#### 11.11.3 与行动点（ActionPoints）的关系

- **在选择阶段**：ActionSelect 展示的可用行动列表由当前单位的剩余行动点决定
  - 如果行动点不足，对应行动（技能/物品）在菜单中标记为不可用（灰色）
  - 待命（End Turn）始终可用（消耗全部剩余行动点）
- **在执行阶段**：行动执行时实际消耗行动点，消耗校验在 ActionPicked 时预先检查
- **消耗回退**：如果执行失败且行动点需要回退，由 Execution 领域决定，Selection 不参与回退逻辑

#### 11.11.4 与 Ability 能力领域的关系

```
Selection 状态机                          Ability 状态机
─────────────────                        ────────────────

ActionPicked (选择技能)
    │
    └──→ AbilityActivated (技能激活)
              │
              ▼
         Targeting (目标校验)
              │
              ▼
TargetConfirmed (确认目标)
    │
    └──→ Ability Active (执行)
              │
              ▼
         AbilityCompleted
              │
    ┌─────────┘
    ▼
UnitActionCompleted (选择完成)
```

- ActionPicked 触发技能的 AbilityActivated（技能进入 Ready → Active 流程）
- TargetConfirmed 为技能执行提供最终目标列表（Ability 流程中的 Targeting 阶段已完成）
- UnitActionCompleted 在 AbilityCompleted 后发布

---

### 11.12 自检清单

- [x] 五级状态层级（Hovered/Focused/Selected/Targeted/Activated）语义严格区分，职责归属清晰
- [x] 选择状态机覆盖完整交互流程（Browsing → UnitSelected → ActionSelect → TargetSelect → TargetLocked → AwaitingExecution → Browsing）
- [x] PickTarget 枚举语义明确（Unit/Tile/Skill/Item），与 PickContext 的约束关系定义清晰
- [x] PickContext 状态机定义了 Normal/UnitSelected/ActionSelect/TargetSelect/TargetLocked/AwaitingExecution 六种模式及约束
- [x] 可选单位条件与不可选情况完整定义（11.4.1 + 11.4.2 + 11.4.3）
- [x] 10 条不变量覆盖选择互斥、边界校验、确认流程完整性、执行不可回退等关键领域规则
- [x] 9 条禁止事项明确列出
- [x] 4 个核心流程（选中、目标选择、取消、行动完成）有完整定义
- [x] 7 个领域事件（UnitClicked/UnitSelected/SelectionChanged/ActionPicked/TargetLocked/TargetConfirmed/UnitActionCompleted）附带触发时机和订阅关系
- [x] 与 Targeting 能力领域的职责边界清晰（选择 vs 目标校验）
- [x] 与 Combat 领域（TurnQueue）和行动点的交互关系定义完整
- [x] 与 Ability 领域的选择→激活→执行的生命周期对齐
- [x] 未涉及代码实现细节，仅定义领域规则
