---
id: 02-domain.targeting
title: Targeting（目标选择）领域规则 v1.0
status: stable
owner: domain-designer
created: 2026-06-16
updated: 2026-06-16
tags:
  - domain
  - targeting
  - capabilities
---


## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Targeting | 目标选择机制，定义技能/效果作用于哪些目标及如何筛选 | 负责：合法目标的筛选规则；不负责：目标选择后的效果执行 |
| TargetType | 目标类型枚举，定义技能可以选择的何种目标 | 负责：目标类别（Self/Ally/Enemy/Dead/Any）的定义；不负责：范围形状 |
| TargetShape | 目标范围形状枚举，定义技能的影响区域形状 | 负责：范围形状（Single/Area/Line/Cone/Chain/Burst）的定义；不负责：范围内的目标筛选逻辑 |
| TargetData | 目标选择的结果数据，包含选中的实体列表、位置列表和上下文信息 | 负责：封装选择结果供下游消费；不负责：选择过程 |
| Selector | 范围筛选与优先级排序的通用实现 | 负责：基于距离/朝向/阵营/属性等条件筛选目标；不负责：具体 TargetType 的含义 |
| TargetValidator | 目标合法性校验器，验证候选目标是否满足技能的各项条件 | 负责：合法性检查（射程/视野/障碍/阵营）；不负责：范围形状的计算 |
| GridTargeting | 网格目标选择实现，支持六角/四角网格的坐标转换与范围计算 | 负责：网格坐标下的范围计算（符合 SRPG 核心需求）；不负责：网格的地形数据 |

### TargetType 与 TargetShape 的组合

```
TargetType（选定何种目标）     TargetShape（以何种范围选择）
 ├── Self                          ├── Single（单体）
 ├── Ally                          ├── Area（区域，自定义半径）
 ├── Enemy                         ├── Line（直线，穿透）
 ├── Dead                          ├── Cone（锥形）
 ├── Any                           ├── Chain（链式弹射）
 └── Custom                        ├── Burst（爆炸，以某格为中心）
                                   └── Wall（墙体/连线）

 组合示例：
   Enemy + Single    = 单体伤害技能（火球术单体版）
   Enemy + Area      = 范围伤害技能（火球术范围版）
   Ally + Single     = 单体治疗技能
   Dead + Single     = 复活术
   Enemy + Line      = 直线穿透技能（闪电束）
   Enemy + Chain     = 链式弹射技能（连锁闪电）
   Ally + Cone       = 锥形治疗/增益
   Any + Burst       = 无差别范围爆炸
```

### 已对齐项目术语

- **Ability**：Targeting 是技能流程的关键环节，Ability 激活后调用 Targeting 选择目标
- **Execution**：TargetData 传递给 Execution 作为伤害/治疗计算的目标依据
- **Effect**：持续性效果可能需要定期重新评估目标（如光环的进出范围检测）
- **Condition**：TargetValidator 可委托 Condition 领域执行复杂的目标合法性检查
- **GameplayContext**：TargetData 作为 GameplayContext 的一部分传递给下游系统

---

## 2. 目标选择状态机

```
Pending（待选择）
   │  [Targeting 被调用]
   ▼
Selecting（筛选中）
   │  [TargetType 过滤 → TargetShape 计算 → Validator 校验]
   ├──→ [有合法目标] → 选择结果
   │
   └──→ [无合法目标] → 空结果
           │
           ▼
      NoValidTarget（无合法目标）
           │
           ▼
      Pending（等待重新选择或取消）
```

### 状态转换规则

| 转换 | 触发条件 | 动作 |
|------|---------|------|
| Pending → Selecting | 目标选择被调用 | 开始筛选流程 |
| Selecting → 有结果 | 找到至少一个合法目标 | 封装 TargetData，返回结果 |
| Selecting → NoValidTarget | 无合法目标 | 通知调用方（技能激活失败） |
| NoValidTarget → Pending | 重新选择请求 | 重置选择状态 |
| 禁止 | Selecting 状态下被重复调用 | 目标选择完成后才可重新调用 |

---

## 3. 不变量（Invariants）

### 3.1 目标合法性不可绕过
- **条件**：任何目标选择流程中
- **不变量**：所有选中的目标必须通过 TargetValidator 校验（射程/视野/阵营/障碍检查）
- **违反后果**：不合法目标被选中，技能作用于不应作用的目标

### 3.2 射程限制
- **条件**：选择距离类目标时
- **不变量**：目标与施法者之间的距离不得超过技能声明的最大射程
- **违反后果**：超射程目标被选中，技能在实际执行时失败

### 3.3 阵营一致性
- **条件**：TargetType 为 Ally/Enemy 时
- **不变量**：选中的目标必须与 TargetType 声明的阵营关系一致
- **违反后果**：友方技能作用到敌方，或敌方技能作用到友方

### 3.4 目标数量限制
- **条件**：TargetShape 确定范围后
- **不变量**：选中的目标数量不得超过技能声明的最大目标数上限
- **违反后果**：过多目标导致技能效果被稀释或性能问题

### 3.5 可见性/感知要求
- **条件**：需要视野检查的技能
- **不变量**：目标必须在施法者的视线范围内（除非技能声明"无视视野"）
- **违反后果**：隔着障碍物选中目标，技能效果在逻辑上不应生效

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：跳过 TargetValidator 直接使用预选目标 — 理由：所有目标必须经过合法性校验，即使是由 UI/玩家预选的
- 🟥 禁止：TargetType 和 TargetShape 互相覆盖语义（如用 Enemy+Ally 组合替代 Any） — 理由：语义不清晰导致目标选择规则不可预测
- 🟥 禁止：目标选择逻辑中嵌入业务计算（如"选择血量最低的敌人"应是 Selector 优先级规则的一部分） — 理由：目标选择只做筛选，不做业务决策
- 🟥 禁止：选择不存在或已销毁的实体作为目标 — 理由：目标选择前必须校验实体存在性

---

## 5. 流程定义

### 5.1 目标选择

- **输入**：施法者实体、技能定义中的 TargetType + TargetShape、可选预选目标/位置
- **处理**：
  1. 根据 TargetType 筛选候选目标池（排除自身/只选敌方/只选友方等）
  2. 根据 TargetShape 计算影响范围（圆形/锥形/直线/链式）
  3. 在范围内应用 Selector 筛选（排除/保留）
  4. 对候选目标执行 TargetValidator 校验（不变量 3.1-3.5）
  5. 对通过校验的目标按优先级排序（如有）
  6. 限制最终目标数量不超过上限（不变量 3.4）
  7. 封装结果到 TargetData
- **输出**：TargetData（entity_list, position_list, context）
- **失败处理**：无合法目标时返回空 TargetData，调用方（Ability）据此判断技能激活失败

### 5.2 网格目标选择（SRPG 专用）

- **输入**：网格起始坐标、TargetShape 参数（半径/角度/方向）、施法者网格坐标
- **处理**：
  1. 将 TargetShape 转换为网格坐标范围
  2. 圆形/爆炸：枚举半径内的所有网格格
  3. 锥形：计算锥形覆盖的所有网格格
  4. 直线：沿方向逐格穿透（遇到障碍或最大距离停止）
  5. 链式：从起始目标按距离弹射到下一目标
  6. 对每个网格中的实体执行目标合法性校验
- **输出**：TargetData（含网格坐标列表）
- **失败处理**：网格坐标超出地图范围时忽略

### 5.3 目标合法性校验（TargetValidator）

- **输入**：候选目标实体、施法者实体、技能定义（射程/阵营/视野要求）
- **处理**：
  1. 实体存在性校验（不变量 4.4）
  2. 距离校验：计算施法者与目标距离，与技能射程比较（不变量 3.2）
  3. 阵营校验：检查目标阵营是否与 TargetType 声明的一致（不变量 3.3）
  4. 视野校验：检查施法者到目标之间是否有障碍物（不变量 3.5）
  5. 数量校验：检查已选数量是否已达上限（不变量 3.4）
  6. 特殊校验：委托 Condition 领域执行技能特定的条件检查
- **输出**：PASS / FAIL（附带失败原因）
- **失败处理**：任一校验不通过返回 FAIL，该目标从候选池移除

---

## 6. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| TargetSelected | 目标选择完成时 | entity_id（施法者）, target_data, ability_id | Ability（继续执行流程）、UI（高亮目标）、日志（LogCode: CNT015） |
| TargetChanged | 目标选择被玩家/系统修改时 | entity_id, old_target_data, new_target_data | Ability（更新 Execution 目标）、日志（LogCode: CNT016） |
| NoValidTarget | 没有合法目标时 | entity_id, ability_id, fail_reason | Ability（技能激活失败处理）、UI（提示无合法目标）、日志（LogCode: CNT017） |
| TargetValidated | 单个目标通过校验时（调试用） | entity_id（目标）, validator_result | 调试工具、日志（LogCode: CNT018） |

### 事件订阅关系图

```
TargetSelected
    │
    ├──→ Ability：获得目标数据→继续 Execution
    ├──→ UI：高亮选中的目标单位
    └──→ Cue：目标选择音效/特效

NoValidTarget
    │
    ├──→ Ability：技能激活失败（无合法目标）
    └──→ UI：显示"无合法目标"提示
```

---

## 7. 与已有架构的对齐校验

- ✅ 架构边界：Targeting 能力领域位于 `core/capabilities/targeting/`，foundation/ 定义 target_type.rs 和 target_data.rs，mechanism/ 定义 selector.rs、grid_targeting.rs 和多个 systems/，符合 C1→C2 分层
- ✅ 术语一致：TargetType、TargetShape、TargetData、Selector、GridTargeting 与架构文档第六节完全一致
- ✅ 网格支持：GridTargeting 明确支持六角/四角网格的 SRPG 核心需求
- ✅ 职责明确：Targeting 只做"目标筛选与校验"，不执行"技能效果"（Execution）、不"选择目标后做什么"（Ability）

---

## 8. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖目标选择、网格目标、合法性校验等全场景
- [x] 所有不变量和约束条件已识别（5 条不变量）
- [x] 禁止事项已明确列出（4 条禁止）
- [x] TargetType 与 TargetShape 的组合规则定义清晰
- [x] 每个操作有完整的流程定义（目标选择、网格目标、合法性校验）
