---
id: 02-domain.attribute
title: Attribute（属性）领域规则 v1.0
status: stable
owner: domain-designer
created: 2026-06-16
updated: 2026-06-16
tags:
  - domain
  - attribute
  - capabilities
---


## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Attribute | 实体的可量化数值属性，具有基础值与当前值分离的机制 | 负责：属性的定义、数值存储与变更通知；不负责：数值的计算逻辑 |
| AttributeId | 属性的唯一标识符，强类型 | 负责：属性身份的唯一定义；不负责：属性的业务含义或数值范围 |
| AttributeValue | 属性的数值表示，分为 base（基础值）和 current（当前值） | 负责：基础值与当前值的分离存储；不负责：值的合法性校验 |
| BaseValue | 属性的基础值，不受临时修改影响，是"干净"的起点值 | 负责：提供属性计算的起点；不负责：实时数值的表示 |
| CurrentValue | 属性的当前值，经所有修改器叠加后的最终结果 | 负责：反映属性的实时状态；不负责：被修改的顺序与过程 |
| AttributeCategory | 属性分类枚举，标识属性的业务角色 | 负责：属性的类型归属（Primary/Secondary/Derived/Resource）；不负责：属性值的数值范围 |
| AttributeContainer | 挂载在实体上的属性容器，管理该实体所有属性 | 负责：实体与属性的关联及批量操作；不负责：单个属性的生命周期 |

### 属性分类体系

```
AttributeCategory
 ├── Primary（主属性）    — 决定角色基本能力的核心属性，如力量、敏捷、体质、智力、感知、魅力
 ├── Secondary（副属性）   — 由主属性推算的衍生属性，如熟练加值、先攻调整值
 ├── Derived（派生属性）   — 由多属性综合计算得出，如生命值上限、防御等级（AC）
 └── Resource（资源属性）  — 可消耗的资源量，如生命值（HP）、法力值（MP）、行动力（AP）
```

### 术语映射关系

```
AttributeId         ─── 属性的"身份证"
AttributeCategory   ─── 属性的"户口分类"
BaseValue           ─── 属性的"出生体重"
CurrentValue        ─── 属性的"实时体重"
AttributeContainer  ─── 实体的"健康档案袋"
```

### 已对齐项目术语

- **Unit**：战场上可操作单位，拥有完整的 AttributeContainer
- **Skill**：主动技能，可能消耗 Resource 类属性（如法力值），伤害受 Primary/Secondary 属性调整
- **Buff**：持续性效果，通过 Modifier 改变 CurrentValue
- **Equipment**：装备，通过 Modifier 对 Primary/Secondary 属性进行加值
- **Progression**：成长系统，升级时提升 BaseValue

---

## 2. 属性状态机

### 属性值生命周期

```
BaseValue（基础值）
    │  [Modifier Applied]
    ▼
ModifiedValue（修改后中间值）
    │  [Aggregation Pipeline]
    ▼
CurrentValue（聚合后最终值）
    │  [Runtime Consumption]
    ▼
ConsumedValue（消费后值）
    │  [Effect Expired / Rest]
    ▼
BaseValue（恢复/重置到基础值起点）
```

### 关键转换规则

| 转换 | 触发条件 | 动作 |
|------|---------|------|
| BaseValue → 调整 | 升级、获得永久性增益/减益 | BaseValue = BaseValue ± 调整量 |
| CurrentValue → 更新 | Modifier 应用或移除 | 通过 Aggregator 管线重算 |
| CurrentValue → 消耗 | 技能释放、受到伤害 | CurrentValue = CurrentValue - 消耗量 |
| CurrentValue → 恢复 | 休息、治疗、自然恢复 | CurrentValue = min(CurrentValue + 恢复量, Derived MaxValue) |
| 禁止 | 跳过 Aggregator 直接修改 CurrentValue | 视为非法操作，触发完整性校验 |

---

## 3. 不变量（Invariants）

### 3.1 基础值不可变性（运行时）
- **条件**：同一战斗/场景中的运行时
- **不变量**：BaseValue 在运行时不可变（仅升级、永久装备等"架构变更"时才可修改）
- **违反后果**：BaseValue 的运行时变更被视为数据不一致，触发错误日志

### 3.2 当前值不能越界
- **条件**：任何 CurrentValue 变更后
- **不变量**：CurrentValue 必须在 [最小允许值, 最大允许值] 范围内（0 ≤ HP ≤ MaxHP，0 ≤ MP ≤ MaxMP）
- **违反后果**：CurrentValue 自动 clamp 到边界值，并记录警告

### 3.3 资源属性非负
- **条件**：任何 Resource 类属性变更后
- **不变量**：Resource 属性的 CurrentValue 必须 ≥ 0
- **违反后果**：消费超过当前值时触发"资源不足"错误，消费失败

### 3.4 派生属性必须有明确公式
- **条件**：定义任何 Derived/Category 属性时
- **不变量**：每个 Derived 属性必须有明确的计算公式（由哪些 Primary/Secondary 属性组合得出）
- **违反后果**：无公式的派生属性不允许注册，配置校验失败

### 3.5 属性值快照一致性
- **条件**：战斗开始/关键节点创建属性快照时
- **不变量**：同一时刻所有属性的快照必须来自同一次 Aggregator 管线重算结果
- **违反后果**：快照数据不一致导致回放或回滚时状态错乱

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：跳过 Modifier/Aggregator 管线直接修改 CurrentValue — 理由：Bypass 修改会导致属性变更不可追踪，回放/回滚不可靠
- 🟥 禁止：Primary 属性的 CurrentValue 被直接修改（应通过 BaseValue→Aggregator→CurrentValue 链路） — 理由：Primary 属性是角色的本质数值，不应被临时效果直接修改
- 🟥 禁止：属性值和标签混合（如用 Tag 携带属性数值） — 理由：违反标签的职责边界，标签只做标识不做数据
- 🟥 禁止：同一 AttributeId 在不同实体上具有不同的基础定义 — 理由：AttributeId 必须全局一致，角色间差异体现在 BaseValue 的数值上而非定义上
- 🟥 禁止：运行时动态注册新 AttributeId — 理由：所有属性定义应在内容加载阶段完成，运行时只做数值变更

---

## 5. 流程定义

### 5.1 属性注册

- **输入**：属性定义（AttributeId、分类 Category、基础值 BaseValue、允许范围 [min, max]、可选计算公式）
- **处理**：
  1. 校验 AttributeId 全局唯一性
  2. 校验 Category 必须为 Primary/Secondary/Derived/Resource 之一
  3. 如果 Category = Derived，校验计算是否存在且引用属性已注册（不变量 3.4）
  4. 校验 BaseValue 在允许范围内
  5. 注册属性定义到全局属性注册表
- **输出**：注册确认 或 AttributeRegistrationError
- **失败处理**：校验不通过时注册失败，不破坏已有注册表

### 5.2 属性初始化（实体创建时）

- **输入**：实体、属性列表（含各属性的 BaseValue）
- **处理**：
  1. 从全局注册表获取属性定义
  2. 设置 BaseValue = 定义中的基础值（可被角色模板覆盖）
  3. 设置 CurrentValue = BaseValue（初始时两者相等）
  4. 对于 Resource 属性，CurrentValue = BaseValue（满状态）
  5. 注册 AttributeContainer 组件到实体
- **输出**：初始化完成的 AttributeContainer
- **失败处理**：引用了未注册的属性时初始化失败

### 5.3 属性值变更通知

- **输入**：属性变更事件（属性 Id、旧值、新值、变更原因、上下文）
- **处理**：
  1. 校验变更后的值是否在允许范围内（不变量 3.2）
  2. 更新 CurrentValue
  3. 如果 CurrentValue 超过边界，clamp 到边界
  4. 发布 AttributeChanged 事件
- **输出**：AttributeChanged 事件
- **失败处理**：值越界时 clamp 并记录警告，不阻止变更

### 5.4 属性快照拍摄

- **输入**：快照触发事件（战斗开始/关键决策节点）
- **处理**：
  1. 遍历目标实体的 AttributeContainer 中所有属性
  2. 记录当前 CurrentValue
  3. 记录当前所有活跃 Modifier 列表及状态
  4. 封装为不可变快照存储
  5. 验证快照一致性（不变量 3.5）
- **输出**：属性快照实例
- **失败处理**：快照一致性校验失败时触发全量重算

---

## 6. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| AttributeChanged | 属性 CurrentValue 发生变更时 | entity_id, attribute_id, old_value, new_value, source_context | Aggregator（触发下游派生属性重算）、UI（数据绑定更新）、Effect（检查 Effect 条件变化）、日志（LogCode: AGG005） |
| AttributeInitialized | 实体完成属性初始化时 | entity_id, attribute_list (attribute_id → initial_value) | Progression（检查是否需基于属性计算派生值）、日志（LogCode: AGG006） |
| SnapshotTaken | 属性快照拍摄完成时 | entity_id, snapshot_id, timestamp | 回放系统、回滚系统、日志（LogCode: AGG007） |
| AttributeClamped | 属性值被 clamp 到边界时 | entity_id, attribute_id, attempted_value, clamped_value | 日志（LogCode: AGG008）、调试工具 |

### 事件订阅关系图

```
AttributeChanged
    │
    ├──→ Aggregator：触发依赖此属性的派生属性重算（如力量改变→触发负重上限重算）
    ├──→ UI：更新 HUD 显示（如血量变化→血条动画）
    ├──→ Condition：检查属性阈值条件是否有变化（如力量≥13 的装备条件）
    ├──→ Trigger：检查属性触发的技能条件是否满足
    └──→ Cue：触发数值变化表现（如掉血闪红）

AttributeInitialized
    │
    └──→ Progression：基于基础属性计算等级相关派生值

SnapshotTaken
    │
    ├──→ 回放系统：记录关键节点的状态作为回放校验点
    └──→ 回滚系统：提供恢复到快照状态的能力
```

---

## 7. 与已有架构的对齐校验

- ✅ 架构边界：Attribute 能力领域位于 `core/capabilities/attribute/`，foundation/ 层定义 AttributeId/AttributeValue/Category，mechanism/ 层定义 AttributeContainer 组件和初始化系统，符合 C1→C2 分层
- ✅ 术语一致：AttributeId、AttributeValue、BaseValue/CurrentValue、AttributeCategory 与架构文档第六节完全一致
- ✅ 职责明确：Attribute 只定义"有什么属性、值是多少"，不涉及"值如何计算"（Aggregator 的职责）、"值如何被改变"（Modifier 的职责）
- ✅ 三层分离：属性定义（content/ 中的 AttributeDef）→ Spec 层（AttributeSpec，角色的属性配置）→ Instance 层（AttributeContainer，运行时的属性值），与 Spec 领域衔接
- ✅ 数据驱动：属性定义下沉到 content/schema/attribute_def.rs 作为配置数据，领域规则只规定属性的行为约束

---

## 8. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖属性注册、初始化、变更通知、快照等全生命周期场景
- [x] 所有不变量和约束条件已识别（5 条不变量）
- [x] 禁止事项已明确列出（5 条禁止）
- [x] 属性分类体系清晰（Primary/Secondary/Derived/Resource）
- [x] 每个操作有完整的流程定义（注册、初始化、变更通知、快照）
