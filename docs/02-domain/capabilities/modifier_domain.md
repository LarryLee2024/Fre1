---
id: 02-domain.modifier
title: Modifier（修改器）领域规则 v1.0
status: stable
owner: domain-designer
created: 2026-06-16
updated: 2026-06-16
tags:
  - domain
  - modifier
  - capabilities
---


## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Modifier | 对目标属性执行算术运算的修改描述，是属性变更的最小原子单元 | 负责：定义如何修改一个属性的运算规则；不负责：属性的聚合计算顺序 |
| ModifierOp | 修改器运算类型枚举，决定修改器如何影响目标属性值 | 负责：运算类型（Add/Multiply/Override）的定义；不负责：运算的优先级排列 |
| ModifierData | 修改器的数据载体，包含运算类型、目标属性、幅度值、优先级 | 负责：携带一次修改所需的全部信息；不负责：修改器来源及上下文 |
| ModifierPriority | 修改器执行优先级，决定同类型修改器的执行顺序 | 负责：同一运算类型内修改器的顺序控制；不负责：不同运算类型的执行次序 |
| ScalableValue | 可缩放的数值类型，支持固定值、曲线缩放、属性缩放三种模式 | 负责：数值来源的灵活定义；不负责：数值的最终结算 |
| ModifierSource | 修改器的来源分类，标记该修改器由何种系统产生 | 负责：修改器起源的追溯（Buff/Equipment/Passive/Environmental）；不负责：来源的业务逻辑 |
| ModifierContainer | 挂载在实体上的活跃修改器容器，管理所有当前生效的修改器 | 负责：修改器的注册/移除/过期管理；不负责：修改器对属性的实际影响 |

### 运算类型定义

```
ModifierOp
 ├── Add       加法运算：Final = Base + Sum(Add modifiers)
 │             用途：力量加值、装备加值、临时增益
 ├── Multiply  乘法运算：Final = (Base + Add) * Product(Multiply modifiers)
 │             用途：百分比增伤、减伤、属性倍率
 └── Override  覆盖运算：Final = Override value（忽略其他运算）
                用途：变形术、特定状态锁定（如定身时敏捷=0）
```

### 数值缩放模式

```
ScalableValue
 ├── Fixed             固定值（如 +5 力量）
 ├── Curve(fn, level)  曲线值（如 等级×2 + 3，从曲线表查值）
 └── AttributeScaling(attribute_id, ratio)  属性缩放值（如 智力×0.5）
```

### 已对齐项目术语

- **Attribute**：被 Modifier 修改的目标，通过 attribute_id 引用（定义在 Attribute 领域）
- **Buff**：持续性效果，是 Modifier 最常见的来源载体
- **Equipment**：装备，穿戴时向穿戴者添加一组 Modifier
- **Effect**：效果实例，携带一组 Modifier 在施加时应用
- **Progression**：成长系统，升级时的属性成长通过 Modifier 实现

---

## 2. 修改器状态机

### 修改器生命周期

```
Created（创建）
   │  [Registered to Container]
   ▼
Active（活跃/生效中）
   │  [Duration expired]
   │  [Removed by system]
   │  [Dispelled]
   ▼
Removed（已移除）
   │  [Revert applied]
   ▼
Reverted（已回退）
```

### 状态转换规则

| 转换 | 触发条件 | 动作 |
|------|---------|------|
| Created → Active | Modifier 被注册到实体的 ModifierContainer | 加入容器，触发所属 Effect 的"已应用"状态 |
| Active → Removed | 持续时间到期/显式移除/驱散 | 从容器移除，触发 Aggregator 重算 |
| Removed → Reverted | 回滚操作执行 | 通知 Aggregator 回退该 Modifier 的影响 |
| 禁止 | Active 状态下直接修改 ModifierData | 修改器数据一旦生效即不可变，需先移除再重新添加 |

---

## 3. 不变量（Invariants）

### 3.1 运算类型互斥
- **条件**：同一属性上的多个 Override 类型修改器共存时
- **不变量**：同一属性上最多只有一个 Override 修改器生效（优先级最高的生效，其余被抑制）
- **违反后果**：多个 Override 修改器同时生效导致属性值不确定，取最高优先级覆盖，其余丢弃

### 3.2 修改器来源可追溯
- **条件**：任何 Modifier 被创建时
- **不变量**：每个 Modifier 必须有明确的来源（BuffId/EquipmentId/AbilityId/EffectId）
- **违反后果**：来源不明的 Modifier 不允许注册到容器

### 3.3 属性引用存在性
- **条件**：Modifier 被注册到容器时
- **不变量**：Modifier 引用的目标属性（通过 attribute_id）必须在全局属性注册表中已注册
- **违反后果**：注册失败，返回 ModifierRegistrationError

### 3.4 修改器幂等性
- **条件**：同一 Modifier 被重复应用到同一实体时
- **不变量**：相同 ModifierData（同源、同目标属性、同运算）第二次应用时不得叠加——应覆盖旧实例或按堆叠规则处理
- **违反后果**：重复应用导致属性值异常倍乘

### 3.5 优先级唯一性
- **条件**：同一属性、同一运算类型的多个 Modifier 排序时
- **不变量**：ModifierPriority 决定同一运算类型内 Modifier 的执行顺序，优先级数值越小越先执行
- **违反后果**：优先级冲突导致执行顺序不确定，使用默认优先级（中）排序

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：修改器直接修改属性值 — 理由：Modifier 只描述"如何修改"，实际修改由 Aggregator 管线执行。直接修改违反单向数据流
- 🟥 禁止：Modifier 携带业务逻辑条件 — 理由：条件检查归 Condition 领域，Modifier 只做数值运算描述
- 🟥 禁止：同一 Modifier 同时影响多个属性 — 理由：一个 Modifier 只描述对一个属性的修改，多属性影响应由多个 Modifier 组合实现
- 🟥 禁止：运行时动态修改 Active 状态 Modifier 的数值 — 理由：修改器效力在应用时确定（快照原则），如需变更应移除旧 Modifier 并注册新实例
- 🟥 禁止：Modifier 引用自身来源形成循环依赖 — 理由：如 Modifier 来源于 Buff A，Buff A 又依赖该 Modifier 的生效结果，形成依赖循环

---

## 5. 流程定义

### 5.1 修改器创建

- **输入**：来源类型及标识、目标属性 Id、运算类型、幅度值（ScalableValue）、优先级、可选持续时间
- **处理**：
  1. 校验目标属性是否已注册（不变量 3.3）
  2. 校验来源是否可追溯（不变量 3.2）
  3. 如果运算类型为 Override，检查同一属性上是否已有活跃 Override 修改器（不变量 3.1）
  4. 解析 ScalableValue 为具体数值（固定值直接取值，曲线值查表，属性缩放读取当前属性值）
  5. 生成 ModifierData 实例
- **输出**：ModifierData 实例
- **失败处理**：校验不通过时创建失败，返回具体错误原因

### 5.2 修改器注册到容器

- **输入**：目标实体、ModifierData 实例
- **处理**：
  1. 检查 ModifierContainer 是否存在，不存在则创建
  2. 检查是否重复注册（不变量 3.4）— 同源同属性同运算的 Modifier 存在时触发展叠规则（归 Stacking 领域）
  3. 注册 ModifierData 到容器，按属性 Id + 运算类型分组存储
  4. 发布 ModifierApplied 事件
  5. 触发 Aggregator 重算受影响属性
- **输出**：注册确认，ModifierApplied 事件
- **失败处理**：容器满（如有上限）时注册失败

### 5.3 修改器移除

- **输入**：目标实体、要移除的 Modifier 标识（来源标识/ModifierData Id）
- **处理**：
  1. 在 ModifierContainer 中定位目标 Modifier
  2. 从容器中移除
  3. 发布 ModifierRemoved 事件
  4. 触发 Aggregator 重算受影响属性
- **输出**：移除确认，ModifierRemoved 事件
- **失败处理**：要移除的 Modifier 不存在时忽略（幂等移除）并记录警告

### 5.4 修改器过期自动移除

- **输入**：时间推进事件（帧更新/回合更新）
- **处理**：
  1. 遍历所有活跃修改器，检查有时间限制的修改器的剩余持续时间
  2. 对已到期的修改器执行移除流程（同 5.3）
- **输出**：批量 ModifierRemoved 事件
- **失败处理**：单个修改器过期失败不影响其他修改器的过期处理

---

## 6. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| ModifierApplied | 修改器成功注册到容器时 | entity_id, modifier_data, source_context | Aggregator（触发重算）、Effect（追踪效果状态）、Stacking（堆叠计数更新）、日志（LogCode: MOD001） |
| ModifierRemoved | 修改器从容器移除时 | entity_id, modifier_data, reason（expired/dispelled/manual） | Aggregator（触发重算）、Effect（效果到期处理）、日志（LogCode: MOD002） |
| ModifierSuppressed | Override 类型修改器因更高优先级被抑制时 | entity_id, suppressed_modifier, dominant_modifier | 日志（LogCode: MOD003）、调试工具 |
| ModifierStaleDetected | 检测到与当前属性快照不一致的修改器时 | entity_id, modifier_data, expected_value, actual_value | 回滚系统、完整性校验、日志（LogCode: MOD004） |

### 事件订阅关系图

```
ModifierApplied
    │
    ├──→ Aggregator：触发属性聚合重算（Modifier 生效→属性变化）
    ├──→ Effect：标记修改器关联效果为"已生效"状态
    └──→ Stacking：更新堆叠计数

ModifierRemoved
    │
    ├──→ Aggregator：触发属性聚合重算（Modifier 移除→属性回退）
    └──→ Effect：标记修改器关联效果为"已到期/已移除"
```

---

## 7. 与已有架构的对齐校验

- ✅ 架构边界：Modifier 能力领域位于 `core/capabilities/modifier/`，foundation/ 定义 modifier_op.rs、modifier_data.rs、scalable_value.rs，mechanism/ 定义 components.rs（ModifierContainer）和 modifier_lifecycle_system.rs，符合 C1→C2 分层
- ✅ 术语一致：ModifierOp（Add/Multiply/Override）、ModifierData、ScalableValue、ModifierPriority 与架构文档第六节完全一致
- ✅ 职责明确：Modifier 只描述"如何修改"，不执行修改（Aggregator 的职责）、不判断修改条件（Condition 的职责）
- ✅ 与 Attribute 领域对齐：Modifier 引用 attribute_id 作为目标，Attribute 领域提供属性的存在性校验
- ✅ 不造新系统：为复用性，Modifier 设计为通用描述语言——法力消耗 = Attribute(Mana) + Modifier(Add, -20)，冷却 = Tag(Cooldown) + Effect(Duration)，不需要独立的 CostSystem/CooldownSystem

---

## 8. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖修改器创建、注册、移除、过期的全生命周期
- [x] 所有不变量和约束条件已识别（5 条不变量）
- [x] 禁止事项已明确列出（5 条禁止）
- [x] 所有运算类型定义清晰（Add/Multiply/Override）
- [x] 每个操作有完整的流程定义（创建、注册、移除、过期处理）
