---
id: 02-domain.condition
title: Condition（条件/限制/免疫）领域规则 v1.1
status: stable
owner: domain-designer
created: 2026-06-16
updated: 2026-06-28
tags:
  - domain
  - condition
  - capabilities
---


## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Condition | 统一的业务条件检查单元，判断某个条件是否满足 | 负责：条件评估的统一入口，返回 Pass/Fail，Condition 的 LocalizationKey（name_key/desc_key）；不负责：条件不满足时的回退逻辑 |
| ConditionType | 条件类型枚举，定义条件检查的类别 | 负责：条件分类（TagRequirement/TagMatch/AttributeCheck/ResourceCheck/Custom）；不负责：各条件类型的内部逻辑 |
| TagRequirement | 基于标签的条件检查，判断实体是否拥有/不拥有特定标签 | 负责：单标签存在性、排除性检查；不负责：标签的业务含义 |
| TagMatch | 基于 TagQuery 的多标签匹配，支持 Any/All/None + 层级继承 | 负责：多标签组合查询与层级继承匹配；不负责：标签的业务含义 |
| AttributeCheck | 基于属性阈值的条件检查，判断属性值是否达到要求 | 负责：属性值数值门槛检查；不负责：属性值的计算逻辑 |
| ResourceCheck | 基于资源充足性的条件检查，判断是否有足够资源执行操作 | 负责：资源量 >= 消耗量的检查；不负责：资源的消耗执行 |
| ConditionGroup | 条件组合，支持 AND/OR/NOT 逻辑运算组合多个条件 | 负责：多个条件的逻辑编排；不负责：单个条件的评估 |
| CustomCondition | 自定义条件 Trait，允许 Domain 注册特定条件逻辑 | 负责：领域特定条件的扩展点；不负责：内置条件类型的实现 |

### 条件类型的组合与嵌套

```
Condition 支持三种逻辑组合方式，可任意嵌套：

ConditionGroup (AND)
  ├── TagRequirement: Has(Tag.Immune.Fire)
  ├── ConditionGroup (OR)
  │    ├── AttributeCheck: 力量 >= 15
  │    └── AttributeCheck: 敏捷 >= 15
  └── NOT
       └── ResourceCheck: 法力 >= 20

等价逻辑：(拥有火焰免疫) AND (力量>=15 OR 敏捷>=15) AND (法力 < 20)
```

### 已对齐项目术语

- **Tag**：TagRequirement 引用 TagId 检查实体标签状态（定义在 Tag 领域）
- **Attribute**：AttributeCheck 引用 AttributeId 检查属性值（定义在 Attribute 领域）
- **Ability**：技能激活前检查 Condition（激活条件），技能影响目标前检查 Immune Condition
- **Effect**：效果应用前检查 Condition（应用条件），可应用 Tag/Attribute 相关条件
- **Equipment**：装备穿戴前检查 Condition（属性需求/等级需求/职业需求）

---

## 2. Condition 状态机

### 条件评估的状态

```
Pending（待评估）
   │  [评估请求]
   ▼
Evaluating（评估中）
   │  [遍历 ConditionGroup 树]
   ├──→ [所有条件通过] → Passed
   │
   └──→ [任一条件不通过] → Failed
           │
           ▼
      Failed（不通过）
           │  [订阅相关事件，等待条件变化]
           ▼
      Pending（可重新评估）
```

### 状态转换规则

| 转换 | 触发条件 | 动作 |
|------|---------|------|
| Pending → Evaluating | 条件评估被触发 | 开始遍历条件树 |
| Evaluating → Passed | 所有条件均通过 | 通知调用方，可执行后续操作 |
| Evaluating → Failed | 任一条件不通过 | 通知调用方，记录失败原因 |
| Failed → Pending | 被订阅的相关状态变化（Tag/Attribute 变更） | 标记为可重新评估 |
| 禁止 | 跳过 Evaluating 直接返回 Passed | 条件评估必须全量执行 |

---

## 3. 不变量（Invariants）

### 3.1 条件评估无副作用
- **条件**：任何 Condition 评估时
- **不变量**：条件评估不得修改任何实体状态、属性值、标签状态
- **违反后果**：副作用导致评估结果受评估顺序影响，确定性被破坏

### 3.2 条件引用对象必须存在
- **条件**：依赖 Tag 或 Attribute 的条件评估时
- **不变量**：条件引用的 TagId 和 AttributeId 必须在注册表中已存在
- **违反后果**：引用不存在的 Tag/Attribute 时条件评估失败

### 3.3 条件的确定性
- **条件**：同一组输入数据
- **不变量**：同一 Condition 在不同时间对同一实体状态的评估结果必须一致（只要实体状态不变）
- **违反后果**：同一条件在同样状态下产生不同结果，导致不可复现的行为

### 3.4 条件组合不产生歧义
- **条件**：多个条件通过 AND/OR/NOT 组合时
- **不变量**：组合条件必须唯一确定，禁止优先级歧义的表达式（如 A AND B OR C 未指定优先级）
- **违反后果**：条件评估结果取决于实现顺序，不同实现产生不同结果

### 3.5 免疫条件的最高优先级
- **条件**：评估"是否可以施加效果 X"时
- **不变量**：免疫条件（Tag.Immune.X）具有最高优先级——如果目标具有 Tag.Immune.X，无论其他条件如何，效果施加被拒绝
- **违反后果**：免疫被其他条件覆盖，角色受到本应免疫的效果

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：Condition 评估过程中修改实体状态 — 理由：条件评估是纯查询操作，不得附带任何副作用
- 🟥 禁止：用 TagRequirement 做属性阈值检查 — 理由：Tag 只做标识分类，属性值检查应使用 AttributeCheck
- 🟥 禁止：条件承载业务"处理逻辑"（如条件不满足时回退到另一种效果） — 理由：条件只做"通过/不通过"判断，回退逻辑归入调用方
- 🟥 禁止：将免疫逻辑分散到各个 Domain 中 — 理由：免疫统一归 Condition 领域处理（使用 Tag.Immune.X + TagRequirement(Not)），防止各 Domain 各自实现免疫检查
- 🟥 禁止：条件依赖运行时 ECS 状态以外的环境变量（如时间戳/随机数） — 理由：条件评估必须是确定性的，依赖环境变量会破坏重现性
- 🟥 禁止：ConditionDef 中直接存储用户可见文本的自然语言 — 理由：必须使用 name_key/desc_key: LocalizationKey 引用。违反宪法 §22 Localization First。

---

## 5. 流程定义

### 5.1 单条件评估

- **输入**：ConditionType、评估目标实体、条件参数（TagId/AttributeId/阈值等）
- **处理**：
  1. 根据 ConditionType 分发到对应检查逻辑
  2. **TagRequirement**：检查目标实体的 GameTagContainer，验证 Has/Not/Any 条件
  3. **AttributeCheck**：读取目标实体的当前属性值，验证是否满足阈值
  4. **ResourceCheck**：读取目标实体的当前资源属性值，验证是否 >= 所需量
  5. 返回 Passed 或 Failed（附带失败原因）
- **输出**：ConditionResult（Passed | Failed { reason }）
- **失败处理**：引用的 Tag/Attribute 不存在时返回 Failed（不变量 3.2）

### 5.2 条件组合评估（ConditionGroup）

- **输入**：ConditionGroup（AND/OR/NOT 组合树）、评估目标实体
- **处理**：
  1. 如果是 AND 组：遍历所有子条件，全部通过返回 Passed；任一失败返回 Failed（短路评估——一旦失败立即返回）
  2. 如果是 OR 组：遍历所有子条件，任一通过返回 Passed；全部失败返回 Failed（短路评估——一旦通过立即返回）
  3. 如果是 NOT 组：评估子条件，结果取反
  4. 递归处理嵌套的 ConditionGroup
- **输出**：ConditionResult
- **失败处理**：条件组合中存在不可评估的条件时，按该条件不通过处理

### 5.3 免疫检查（特殊流程）

- **输入**：目标实体、效果类型 Id、效果携带的 Tag
- **处理**：
  1. 构建免疫检查条件：TagRequirement(Has, Tag.Immune.{EffectType})
  2. 对目标实体的 GameTagContainer 执行检查
  3. 如果目标具有免疫标签，返回免疫生效（不变量 3.5）
- **输出**：免疫结果（Immune/NotImmune）
- **失败处理**：免疫检查是条件系统的最高优先级路径，失败即禁止效果施加

### 5.4 条件订阅（等待条件变化）

- **输入**：条件定义、当前评估失败的实体
- **处理**：
  1. 分析条件依赖的 Tag/Attribute 列表
  2. 订阅对应 TagAdded/TagRemoved/AttributeChanged 事件
  3. 当被订阅事件触发时，标记条件为 Pending（可重新评估）
- **输出**：订阅确认
- **失败处理**：订阅事件源不存在时忽略（条件将不再自动重评估）

---

## 6. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| ConditionPassed | 条件评估通过时 | entity_id, condition_id, condition_type, result_data | Ability（允许继续激活）、Equipment（允许穿戴）、日志（LogCode: CNT011） |
| ConditionFailed | 条件评估不通过时 | entity_id, condition_id, condition_type, fail_reason | Ability（阻止激活，显示失败原因）、UI（显示提示）、日志（LogCode: CNT012） |
| ImmunityTriggered | 免疫条件生效时 | entity_id, target_effect_type, immune_tag_id | Cue（显示免疫文字弹跳）、日志（LogCode: CNT013） |
| ConditionSubscribed | 条件进入等待变化订阅状态时 | entity_id, condition_id, subscribed_events | 调试工具、条件可视化、日志（LogCode: CNT014） |

### 事件订阅关系图

```
ConditionPassed
    │
    ├──→ Ability：条件满足→允许技能继续激活流程
    ├──→ Equipment：条件满足→允许装备穿戴
    ├──→ Effect：条件满足→允许效果应用
    └──→ Trigger：触发条件满足→激活关联技能

ConditionFailed
    │
    ├──→ Ability：条件不满足→技能激活终止
    ├──→ UI：显示"条件不满足"提示（如力量不足、法力不够）
    └──→ 日志：记录失败原因用于调试

ImmunityTriggered
    │
    ├──→ Cue：显示"IMMUNE"文字特效
    └──→ 日志：记录免疫事件用于平衡分析
```

---

## 7. 与已有架构的对齐校验

- ✅ 架构边界：Condition 能力领域位于 `core/capabilities/condition/`，foundation/ 定义 condition_type.rs、tag_requirement.rs、attribute_check.rs、resource_check.rs，mechanism/ 定义 components.rs（ConditionContainer）和 condition_eval_system.rs，符合 C1→C2 分层
- ✅ 术语一致：ConditionType、TagRequirement、AttributeCheck、ResourceCheck 与架构文档第六节完全一致
- ✅ 职责明确：Condition 只做"条件判断"，不做"能力激活"（Ability）、"效果施加"（Effect）
- ✅ 免疫统一归入 Condition：免疫 = Tag(Immune.X) + Condition(TagRequirement Not)，避免了独立的 ImmunitySystem
- ✅ Condition 统一了三个场景：技能激活条件、装备穿戴限制、效果免疫检查——Same Engine, Different Configs
- ✅ LocalizationKey：Condition 使用 LocalizationKey 而非硬编码文本（宪法 §22）

---

## 8. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖单条件评估、条件组合、免疫检查、条件订阅等全场景
- [x] 所有不变量和约束条件已识别（5 条不变量）
- [x] 禁止事项已明确列出（5 条禁止）
- [x] 条件组合的 AND/OR/NOT 逻辑定义清晰
- [x] 免疫作为条件的最高优先级路径已明确定义
- [x] 每个操作有完整的流程定义（单条件评估、组合评估、免疫检查、条件订阅）
