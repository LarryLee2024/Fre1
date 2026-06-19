---
id: 02-domain.execution
title: Execution（执行计算）领域规则 v1.0
status: stable
owner: domain-designer
created: 2026-06-16
updated: 2026-06-19
tags:
  - domain
  - execution
  - capabilities
---


## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Execution | 技能/效果执行计算的核心，负责将技能描述转化为实际数值变化 | 负责：执行计算的调度与分发，Execution 的 LocalizationKey（name_key/desc_key）；不负责：技能的生命周期管理 |
| ExecutionType | 执行计算类型枚举，定义计算的业务类别 | 负责：计算分类（Damage/Heal/Custom）；不负责：具体的计算公式 |
| ExecutionContext | 执行计算上下文，从 GameplayContext 派生，携带计算所需的全部输入 | 负责：提供计算需要的所有数据（来源属性/目标属性/技能参数/环境因素）；不负责：计算结果的存储 |
| DamageExecution | 伤害计算执行，调用领域伤害公式计算最终伤害值 | 负责：伤害公式的调用与参数传递；不负责：伤害公式本身（归 Domains/rules/formulas.rs） |
| HealExecution | 治疗计算执行，调用领域治疗公式计算最终治疗值 | 负责：治疗公式的调用与参数传递；不负责：治疗公式本身 |
| CustomExecution | 自定义执行 Trait，允许 Domain 注册特定计算逻辑 | 负责：领域特有计算逻辑的扩展点；不负责：内置计算类型的实现 |
| CalculationFormula | 计算公式的引用标识，指向 Domains/rules/ 中的纯函数计算结果 | 负责：标识使用哪个公式；不负责：公式的计算实现 |

### Execution 与 Domains/rules/ 的协作关系

```
Execution 领域（capabilities/execution/）      Domains/rules/（业务域规则）
                        │                                      │
                        │  调用/引用                            │  定义
                        ▼                                      ▼
  ┌──────────────────┐                    ┌─────────────────────────┐
  │ ExecutionType    │ ──→ 路由到 ──→     │ damage_formula.rs       │
  │  Damage          │                    │   D&D 5e 伤害公式       │
  │  Heal            │                    │   weapon_dice + mod     │
  │  Custom          │                    │   + bonus               │
  └──────────────────┘                    ├─────────────────────────┤
        │                                 │ heal_formula.rs         │
        │                                 │   基础治疗量 + 属性修正   │
        │                                 ├─────────────────────────┤
        │                                 │ critical_rules.rs       │
        │                                 │   暴击骰翻倍规则          │
        │                                 ├─────────────────────────┤
        │                                 │ advantage_rules.rs      │
        │                                 │   优势/劣势判定规则       │
        └──── Execution 不包含任何公式 ──→ └─────────────────────────┘
```

### 已对齐项目术语

- **Ability**：Execution 是技能 Active 阶段的核心环节，由 Ability 领域调用
- **Effect**：Execution 产生 Effect 作为技能的执行结果
- **GameplayContext**：ExecutionContext 从 GameplayContext 派生，继承来源/目标/技能等数据
- **Modifier**：Execution 计算过程中可引用 Modifier 数据计算最终数值
- **Aggregator**：Execution 计算时需要读取属性当前值（通过 Aggregator 获取 FinalValue）

---

## 2. 执行状态机

```
Pending（待执行）
   │  [ExecutionContext 就绪]
   ▼
Executing（执行中——计算进行中）
   │  [分发到对应 ExecutionType 的处理逻辑]
   │
   ├──→ [DamageExecution] → 调用 damage_formula → 计算最终伤害
   │
   ├──→ [HealExecution] → 调用 heal_formula → 计算最终治疗
   │
   └──→ [CustomExecution] → 调用 Domain 注册的自定义计算
           │
           ▼
Completed（已完成——计算结果产出）
   │  [结果封装为 Effect]
   ▼
EffectCreated（效果已产生——移交 Effect 领域）
```

### 状态转换规则

| 转换 | 触发条件 | 动作 |
|------|---------|------|
| Pending → Executing | Execution 被调用 | 按 ExecutionType 分发到对应计算逻辑 |
| Executing → Completed | 计算公式返回结果 | 封装计算结果 |
| Completed → EffectCreated | 计算结果包装为 Effect | 发布事件，移交 Effect 领域 |
| 禁止 | 跳过 Executing 直接返回 Completed | 计算必须经过公式计算 |

---

## 3. 不变量（Invariants）

### 3.1 Execution 不包含公式
- **条件**：任何 Execution 计算时
- **不变量**：Execution 领域本身不包含任何计算公式，所有公式归 Domains/rules/ 管理
- **违反后果**：公式硬编码在 Execution 领域导致业务域无法定制数值规则

### 3.2 计算结果可追踪
- **条件**：任何 Execution 完成后
- **不变量**：每个 Execution 结果必须携带完整的计算过程日志（输入参数、中间值、最终结果）
- **违反后果**：计算结果不可追溯，数值平衡问题无法排查

### 3.3 Execution 上下文完整性
- **条件**：Execution 开始前
- **不变量**：ExecutionContext 必须包含计算所需的全部输入数据（来源属性/目标属性/技能参数/环境等）
- **违反后果**：上下文数据缺失导致计算错误

### 3.4 结果数值范围
- **条件**：任何 Execution 返回数值结果后
- **不变量**：伤害/治疗值必须在合理的数值范围内（伤害 >= 0，治疗 >= 0），负数应归零
- **违反后果**：负伤害导致目标反而回血，负治疗导致目标受伤

### 3.5 自定义执行的标识唯一性
- **条件**：注册 CustomExecution 时
- **不变量**：每个自定义执行必须有全局唯一的标识符，禁止重复注册
- **违反后果**：自定义执行标识冲突导致计算路由错误

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：Execution 领域包含业务计算公式 — 理由：公式归 Domains/rules/，Execution 只做调用分发
- 🟥 禁止：跳过 Execution 直接修改目标属性值 — 理由：所有伤害/治疗等数值变更必须经过 Execution 计算，确保可追踪性
- 🟥 禁止：在 ExecutionContext 中混入计算无关的数据（如 UI 状态） — 理由：上下文应保持纯净，只包含计算所需数据
- 🟥 禁止：Execution 产生副作用（如直接播放音效、修改界面） — 理由：副作用归 Cue 领域，Execution 只做数值计算
- 🟥 禁止：ExecutionDef 中直接存储用户可见文本的自然语言文本 — 理由：必须使用 name_key/desc_key: LocalizationKey 引用。违反宪法 §22 Localization First。

---

## 5. 流程定义

### 5.1 执行计算（通用）

- **输入**：ExecutionContext（从 GameplayContext 派生）、ExecutionType、CalculationFormula 引用
- **处理**：
  1. 根据 ExecutionType 分发到对应计算路由
  2. 加载 CalculationFormula 引用的公式（从 Domains/rules/ 加载）
  3. 将 ExecutionContext 数据传递给公式函数
  4. 公式执行计算，返回结果
  5. 验证结果数值范围（不变量 3.4）
  6. 记录计算过程日志（不变量 3.2）
  7. 封装计算结果
- **输出**：ExecutionResult（数值结果 + 计算过程日志）
- **失败处理**：公式引用不存在或 ExecutionContext 数据缺失时计算失败

### 5.2 伤害计算

- **输入**：ExecutionContext（含攻击方属性/防御方属性/武器数据/技能参数/暴击率/优势/劣势等）
- **处理**：
  1. 加载伤害公式（Domains/rules/damage_formula.rs 或 Domain 自定义公式）
  2. 读取攻击方属性（力量/敏捷/施法属性等）
  3. 读取武器/技能伤害骰
  4. 计算命中判定（攻击骰 vs 防御等级 AC）
  5. 如果命中，计算伤害值（伤害骰 + 属性调整值 + 其他加值）
  6. 计算暴击（如适用，暴击时伤害骰翻倍）
  7. 应用目标减伤/抗性
  8. 验证结果 >= 0（不变量 3.4）
- **输出**：DamageResult（最终伤害值、命中/暴击标志、公式展开明细）
- **失败处理**：关键数据缺失时返回计算失败错误

### 5.3 治疗计算

- **输入**：ExecutionContext（含治疗者属性/受治疗者数据/技能参数）
- **处理**：
  1. 加载治疗公式
  2. 读取治疗者属性（施法属性/治疗加成等）
  3. 计算治疗量（治疗骰 + 属性调整值 + 其他加值）
  4. 验证结果 >= 0（不变量 3.4）
- **输出**：HealResult（最终治疗值、是否暴击治疗、公式展开明细）
- **失败处理**：关键数据缺失时返回计算失败错误

### 5.4 自定义执行

- **输入**：ExecutionContext、CustomExecution 标识符
- **处理**：
  1. 通过标识符查找已注册的 CustomExecution
  2. 将 ExecutionContext 传递给自定义执行逻辑
  3. 自定义执行返回计算结果
- **输出**：自定义 ExecutionResult
- **失败处理**：标识符未注册时返回错误

---

## 6. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| ExecutionCompleted | 执行计算完成时 | execution_type, context, result, formula_trace | Ability（继续后续流程）、Effect（创建效果实例）、日志（LogCode: EFF005） |
| ExecutionFailed | 执行计算失败时 | execution_type, context, fail_reason | Ability（技能执行失败）、日志（LogCode: EFF006） |
| CustomExecutionRegistered | 自定义执行注册时 | execution_id, description | 注册中心、调试工具、日志（LogCode: EFF007） |

### 事件订阅关系图

```
ExecutionCompleted
    │
    ├──→ Ability：获取计算结果→创建 Effect 列表
    ├──→ Effect：将计算结果封装为 Effect（立即/持续）
    ├──→ 日志：记录完整计算过程用于平衡分析
    └──→ Cue：触发伤害数字/治疗数字表现

ExecutionFailed
    │
    ├──→ Ability：标记技能执行失败
    └──→ 日志：记录失败原因用于调试
```

---

## 7. 与已有架构的对齐校验

- ✅ 架构边界：Execution 能力领域位于 `core/capabilities/execution/`，foundation/ 定义 execution_type.rs、execution_context.rs、custom_execution.rs，mechanism/ 定义 damage_execution.rs、heal_execution.rs 和 execution_system.rs，符合 C1→C2 分层
- ✅ 术语一致：ExecutionType、ExecutionContext、DamageExecution、HealExecution、CustomExecution 与架构文档第六节完全一致
- ✅ 公式分离：所有业务公式归 Domains/rules/，Execution 只做调度分发，符合架构文档"数据驱动"原则
- ✅ 职责明确：Execution 只做"计算"，不做"生命周期"（Ability）、不做"效果管理"（Effect）
- ✅ LocalizationKey：Execution 使用 LocalizationKey 而非硬编码文本（宪法 §22）

---

## 8. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖执行计算、伤害/治疗计算、自定义执行等全场景
- [x] 所有不变量和约束条件已识别（5 条不变量）
- [x] 禁止事项已明确列出（4 条禁止）
- [x] Execution 与 Domains/rules/ 的协作关系定义清晰
- [x] 每个操作有完整的流程定义（通用计算、伤害、治疗、自定义）
