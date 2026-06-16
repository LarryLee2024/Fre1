# Aggregator（聚合器）领域规则 v1.0

Version: 1.0
Status: Draft
Applies To: Capabilities — 聚合层

---

## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Aggregator | 属性聚合管线，将 Attribute 的 BaseValue 与所有活跃 Modifier 按规则计算为 FinalValue | 负责：属性值的聚合计算流程编排；不负责：单个 Modifier 的创建与管理 |
| CalcStage | 计算阶段，定义 Modifier 运算类型的执行顺序 | 负责：运算阶段的划分（Add→Multiply→Override→Clamp）与顺序保证；不负责：各阶段内部 Modifier 的优先级排序 |
| CalcPipeline | 完整属性计算管线，从 BaseValue 到 FinalValue 的完整变换链路 | 负责：多阶段运算的串联执行；不负责：计算结果的持久化 |
| AggregationSnapshot | 聚合快照，记录某一时刻所有属性的聚合状态 | 负责：提供可恢复的状态记录；不负责：快照间的差异计算 |
| AggregatorState | 挂载在实体上的聚合状态组件，缓存当前聚合结果 | 负责：缓存计算后的属性最终值；不负责：属性的中间计算过程 |
| StagePriority | 阶段内 Modifier 执行优先级，决定同一阶段内多个 Modifier 的计算顺序 | 负责：阶段内排序规则；不负责：不同阶段间的执行顺序 |

### 计算管线阶段定义

```
CalcPipeline 执行顺序（严格单向，不可逆）：

  BaseValue（基础值）
      │
      ▼
  Stage 1: Add（加法阶段）
      │    Sum(所有 Add 类型 Modifier 的值)
      │    公式：AfterAdd = BaseValue + Σ(AddModifierValue × PriorityOrder)
      ▼
  Stage 2: Multiply（乘法阶段）
      │    Product(所有 Multiply 类型 Modifier 的值)
      │    公式：AfterMul = AfterAdd × Π(MultiplyModifierValue × PriorityOrder)
      │    注意：乘法叠加是连乘而非加法（+20% +30% = ×1.2 ×1.3 = ×1.56，非 ×1.5）
      ▼
  Stage 3: Override（覆盖阶段）
      │    取优先级最高的 Override Modifier 的值
      │    公式：AfterOverride = OverrideModifier(最高优先级).value
      │    如果无 Override Modifier，则 AfterOverride = AfterMul
      ▼
  Stage 4: Clamp（钳制阶段）
      │    将值限制在 [MinValue, MaxValue] 范围内
      │    公式：FinalValue = clamp(AfterOverride, MinValue, MaxValue)
      ▼
  FinalValue（最终值）
```

### 已对齐项目术语

- **Attribute**：属性定义与基础值，Aggregator 的计算起点（定义在 Attribute 领域）
- **Modifier**：修改器描述，Aggregator 在阶段计算中消费 Modifier 数据（定义在 Modifier 领域）
- **ModifierOp**：修改器运算类型（Add/Multiply/Override），直接对应 CalcStage 的阶段划分

---

## 2. 聚合器状态机

### 属性计算状态

```
Clean（缓存有效）
   │  [ModifierApplied / ModifierRemoved 事件到达]
   ▼
Dirty（需要重算）
   │  [重算开始]
   ▼
Computing（正在计算）
   │  [计算完成]
   ▼
Clean（缓存有效）
```

### 状态转换规则

| 转换 | 触发条件 | 动作 |
|------|---------|------|
| Clean → Dirty | 监听到 ModifierApplied 或 ModifierRemoved 事件 | 标记 AggregatorState 为脏 |
| Dirty → Computing | 下一个可计算时机（同一帧内批量合并） | 开始聚合管线计算 |
| Computing → Clean | 四项阶段计算全部完成 | 缓存 FinalValue，标记为 Clean |
| 禁止 | 同一属性在 Computing 状态下被再次标记为 Dirty | 视为并发冲突，应等待当前计算完成后重新触发 |

---

## 3. 不变量（Invariants）

### 3.1 阶段执行顺序不可逆
- **条件**：任何属性聚合计算时
- **不变量**：CalcStage 必须严格按照 Add→Multiply→Override→Clamp 顺序执行，禁止跳过或调换任何阶段
- **违反后果**：计算结果错误，聚合管线异常终止

### 3.2 乘法叠加非加法
- **条件**：多个 Multiply 类型 Modifier 共存时
- **不变量**：乘法 Modifier 以连乘方式叠加（1.2 × 1.3 = 1.56），而非加法叠加（1 + 0.2 + 0.3 = 1.5）
- **违反后果**：乘法 Modifier 叠加后效果偏差，数值平衡失控

### 3.3 Override 互斥性
- **条件**：Override 阶段执行时
- **不变量**：同一属性上最多只有一个 Override Modifier 的值被采用（优先级最高的生效）
- **违反后果**：属性值被多次覆盖，最终结果由时序决定而非规则决定

### 3.4 聚合结果可复现
- **条件**：同一组 BaseValue + 同一组 Modifier 集合
- **不变量**：任何一次聚合计算必须产生完全相同的 FinalValue（确定性）
- **违反后果**：回放时状态不一致，回放测试失败

### 3.5 快照一致性
- **条件**：AggregationSnapshot 被创建时
- **不变量**：快照必须包含快照时刻所有属性的 FinalValue 及当前 Modifier 容器的完整状态
- **违反后果**：快照用于回滚时无法恢复完整状态

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：跳过 Aggregator 管线直接使用属性值 — 理由：实时读取属性值必须经过 Aggregator 确保包含所有 Modifier 影响
- 🟥 禁止：同一属性在单次管线计算中被多次触发 — 理由：Modifier→Aggregator→AttributeChange→Modifier 形成循环触发链，必须检测并终止
- 🟥 禁止：Override Modifier 在 Add/Multiply 阶段参与计算 — 理由：Override 是独立阶段，不在前两阶段参与任何运算
- 🟥 禁止：Clamp 阶段的边界值为可变值（依赖运行时动态数据） — 理由：MinValue/MaxValue 应在属性定义时确定，运行时 Clamp 边界不应依赖实时属性值
- 🟥 禁止：聚合器感知 Modifier 的业务来源 — 理由：Aggregator 只按运算类型和优先级处理 Modifier，不区分"来自 Buff 还是来自装备"

---

## 5. 流程定义

### 5.1 属性聚合计算

- **输入**：目标属性 Id、当前 AggregatorState（含缓存的当前值）、对应 Attribute 的 BaseValue、该属性上的所有活跃 Modifier 列表
- **处理**：
  1. 标记 AggregatorState 为 Dirty
  2. 从 Attribute 获取 BaseValue
  3. 从 ModifierContainer 获取该属性关联的所有活跃 Modifier
  4. 按 ModifierOp 分组并依 StagePriority 排序
  5. **阶段 1——Add**：对所有 Add 类型 Modifier 按优先级依次求累加和
  6. **阶段 2——Multiply**：对所有 Multiply 类型 Modifier 按优先级依次求连乘积
  7. **阶段 3——Override**：选择最高优先级的 Override Modifier 的值（若无则跳过）
  8. **阶段 4——Clamp**：将结果限制在 [MinValue, MaxValue] 范围内
  9. 缓存 FinalValue 到 AggregatorState
  10. 标记 AggregatorState 为 Clean
  11. 发布 AggregationComplete 事件
- **输出**：FinalValue, AggregationComplete 事件
- **失败处理**：任一阶段中出现数据不一致（如 Modifier 引用了不存在的属性），中断管线，返回 PipelineError，AggregatorState 保持 Dirty 状态等待重试

### 5.2 快照拍摄

- **输入**：快照触发（战斗开始/关键决策节点/手动请求）
- **处理**：
  1. 确保所有属性处于 Clean 状态（先触发一次完整聚合）
  2. 遍历目标实体的所有属性，记录 FinalValue
  3. 遍历 ModifierContainer，记录所有活跃 Modifier 的数据
  4. 封装为不可变 AggregationSnapshot
- **输出**：AggregationSnapshot
- **失败处理**：快照时存在 Dirty 属性，强制聚合后再拍摄

### 5.3 快照恢复（回滚）

- **输入**：AggregationSnapshot 实例
- **处理**：
  1. 将属性的 FinalValue 恢复到快照记录的值
  2. 清空当前 ModifierContainer 并恢复快照中的 Modifier 列表
  3. 重新注册 Modifier 到容器
  4. 标记 AggregatorState 为 Clean
- **输出**：恢复确认
- **失败处理**：快照版本与当前属性定义不兼容时恢复失败

### 5.4 批量惰性重算

- **输入**：帧更新事件（一帧内累积的多个 ModifierApplied/Removed 事件）
- **处理**：
  1. 收集一帧内所有 Dirty 属性
  2. 去重（同一属性在一帧内被多次标记 Dirty 只重算一次）
  3. 批量执行属性聚合计算
  4. 统一发布 AggregationComplete 事件（批量）
- **输出**：批量 FinalValue 更新
- **失败处理**：批量中单个属性重算失败不影响其他属性

---

## 6. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| AggregationComplete | 属性聚合计算完成时 | entity_id, attribute_id, final_value, calc_stage_trace（调试用） | Attribute（更新 CurrentValue）、UI（刷新显示）、Condition（检查属性阈值变更） |
| AggregateDirty | 属性被标记为需要重算时 | entity_id, attribute_id, trigger_source（哪个 Modifier 变更导致的） | 调度系统（安排下一帧重算） |
| SnapshotCreated | 快照拍摄完成时 | entity_id, snapshot_id, attribute_count, timestamp | 回放系统、回滚系统 |
| PipelineCycleDetected | 检测到聚合闭环时（如 A→B→A 循环） | cycle_chain: [attribute_id1, attribute_id2, attribute_id1] | 日志、架构检查工具 |

### 事件订阅关系图

```
ModifierApplied / ModifierRemoved（由 Modifier 领域发布）
    │
    ├──→ Aggregator：标记对应属性为 Dirty
    │
    ▼
AggregateDirty（由 Aggregator 发布）
    │
    ├──→ Scheduler：安排帧末批量重算
    │
    ▼
AggregationComplete（由 Aggregator 发布）
    │
    ├──→ Attribute：更新 Attribute.CurrentValue
    ├──→ UI：刷新属性显示（如血量/法力值/攻击力面板）
    ├──→ Condition：触发属性阈值相关条件检查
    └──→ Cue：数值变化表现信号
```

---

## 7. 与已有架构的对齐校验

- ✅ 架构边界：Aggregator 能力领域位于 `core/capabilities/aggregator/`，foundation/ 定义 calc_stage.rs 和 snapshot.rs，mechanism/ 定义 calc_pipeline.rs 和 aggregate_system.rs，符合 C1→C2 分层
- ✅ 术语一致：CalcStage、CalcPipeline、AggregationSnapshot、StagePriority 与架构文档第六节完全一致
- ✅ 职责明确：Aggregator 只编排"计算顺序"，不创建 Modifier（Modifier 的职责）、不存储属性值（Attribute 的职责）
- ✅ 管线顺序与 GAS 对齐：Base→Add→Multiply→Override→Clamp 与 FAttributeAggregator 的设计一致
- ✅ 回放友好：确定性的计算管线保证同一输入产生同一输出，快照机制支持回滚

---

## 8. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖属性聚合计算、快照、回滚、批量重算等全场景
- [x] 所有不变量和约束条件已识别（5 条不变量）
- [x] 禁止事项已明确列出（5 条禁止）
- [x] 计算管线四阶段定义清晰（Add→Multiply→Override→Clamp）
- [x] 每个操作有完整的流程定义（聚合计算、快照拍摄、快照恢复、批量重算）
