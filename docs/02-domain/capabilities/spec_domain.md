---
id: 02-domain.spec
title: Spec（规格/配置）领域规则 v1.0
status: stable
owner: domain-designer
created: 2026-06-16
updated: 2026-06-19
tags:
  - domain
  - spec
  - capabilities
---


## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Def（Definition） | 模板定义层，内容配置文件中定义的不可变数据模板 | 负责：提供能力的原始定义（如 Fireball 的伤害骰/范围/等级）；不负责：角色身上该能力的具体配置 |
| Spec（Specification） | 配置层，角色/实体身上的能力配置实例，在 Def 基础上叠加角色定制数据 | 负责：承载 Def 在特定实体上的定制数据（等级/强化/冷却缩减），Spec 的 LocalizationKey（name_key/desc_key）；不负责：能力的运行时执行状态 |
| Instance（Runtime Instance） | 运行时实例层，激活中的能力/效果的运行态数据 | 负责：承载运行时状态（当前冷却进度/施法进度/持续效果倒计时）；不负责：能力的定义与配置 |
| AbilitySpec | 角色身上的技能配置实例，包含技能等级、输入绑定、冷却覆盖等定制数据 | 负责：技能在角色维度的个性化配置；不负责：技能的运行状态 |
| EffectSpec | 效果应用后的实例，包含效果来源、持续时间、堆叠计数等运行时数据 | 负责：效果在目标身上的个性化实例；不负责：效果的定义模板 |
| SpecRegistry | Spec 注册中心，提供 Def→Spec 的工厂转换 | 负责：基于 Def 创建 Spec 实例；不负责：Spec 的运行时生命周期管理 |

### 三层分离映射

```
Def（模板/资源）          Spec（配置/槽位）          Instance（运行时）

AbilityDef               AbilitySpec               AbilityInstance
  ├ 技能ID                  ├ 模板引用                  ├ 所属 Spec 引用
  ├ 基础消耗（20法力）       ├ 等级（Lv3）               ├ 当前阶段（Casting）
  ├ 基础冷却（3回合）        ├ 冷却缩减（-1回合）         ├ 施法进度（2/3）
  ├ 基础伤害（8d6）         ├ 强化数据（+2d6）          ├ 目标列表
  └ 基础范围（20尺）        └ 输入绑定（快捷栏3）        └ 状态（激活中）

EffectDef                EffectSpec                EffectInstance
  ├ 效果ID                  ├ 模板引用                   ├ 所属 Spec 引用
  ├ 类型（Periodic）        ├ 来源上下文                  ├ 剩余持续时间（2回合）
  ├ 基础持续时间（3回合）    ├ 持续时间修正（+1回合）       ├ 下一跳时间戳
  ├ 基础间隔（1回合）        ├ 堆叠数（2层）               ├ 已触发跳数（3）
  └ 基础数值（1d6/跳）      └ 快照属性值                  └ 状态（Ticking）
```

### 已对齐项目术语

- **Tag**：Spec 的 Tag 相关字段引用 TagId 进行条件过滤（定义在 Tag 领域）
- **Attribute**：Spec 中需要快照的属性值引用 Attribute 领域的数据
- **Ability**：AbilityInstance 是 Spec 的运行时表现（Ability 领域管理）
- **Effect**：EffectInstance 是 Spec 的运行时表现（Effect 领域管理）
- **Content**：Def 定义在 content/schema/ 中，Spec 领域负责 Def→Spec 的转换

---

## 2. Spec 状态机

### AbilitySpec 生命周期

```
Empty（无 Spec）
   │  [SpecGranted]
   ▼
Idle（已配置/未激活）
   │  [等级变更]  ───────── 等级变更后回到 Idle
   │  [冷却结束]  ←────────
   │      │                   
   │      ▼ [技能激活触发]
   │  Activated（已激活——转 Ability 领域管理）
   │      │
   │      ▼ [执行完毕]
   ├──→ Idle（回到待命）
   │
   │  [SpecRemoved]
   ▼
Removed（已移除）
```

### EffectSpec 生命周期

```
Empty（无 Spec）
   │  [EffectDef → 通过 SpecRegistry 创建]
   ▼
Pending（待应用——条件检查中）
   │  [条件通过]
   ▼
Applied（已应用——Effect 领域接管）
   │  [Effect 生命周期结束]
   ▼
Expired（已过期——Effect 已移除）
   │  [SpecRemoved]
   ▼
Removed（已移除）
```

### 状态转换规则

| Spec 类型 | 转换 | 触发条件 | 动作 |
|-----------|------|---------|------|
| Ability | Empty → Idle | SpecGranted | 注册 AbilitySpec 到实体容器 |
| Ability | Idle → Activated | 技能激活请求 | 移交 Ability 领域管理 |
| Ability | Activated → Idle | 技能执行完毕/取消 | 从 Ability 领域回收，更新冷却数据 |
| Ability | Idle/Activated → Removed | SpecRemoved | 从实体容器移除，触发取消事件 |
| Effect | Empty → Pending | EffectDef 创建 EffectSpec | 注册 EffectSpec，等待条件检查 |
| Effect | Pending → Applied | 条件通过 | 移交 Effect 领域管理 |
| Effect | Applied → Expired | Effect 生命周期结束 | 从 Effect 领域回收 |
| Effect | Pending/Applied/Expired → Removed | SpecRemoved | 从实体容器完全移除 |

---

## 3. 不变量（Invariants）

### 3.1 Def 不可变性
- **条件**：Def 从 content/ 加载完成后
- **不变量**：Def（AbilityDef/EffectDef）在运行时不可修改
- **违反后果**：运行时的 Def 修改会导致上下游 Spec/Instance 数据不一致

### 3.2 Def→Spec 一致性
- **条件**：Spec 通过 SpecRegistry 创建时
- **不变量**：Spec 必须完全基于一个已注册的 Def 创建，不允许"无模板的 Spec"
- **违反后果**：无模板的 Spec 无法确定基础属性，导致运行时异常

### 3.3 Spec 等级范围
- **条件**：AbilitySpec 等级变更时
- **不变量**：AbilitySpec 的等级必须在 [1, MaxLevel] 范围内（MaxLevel 由 AbilityDef 定义）
- **违反后果**：超出范围的等级变更被拒绝

### 3.4 Spec 与 Instance 生命周期独立
- **条件**：Instance 由 Spec 创建时
- **不变量**：Spec 被移除时，由其创建的所有 Instance 必须被级联终止
- **违反后果**：Spec 移除后仍有孤儿 Instance 在运行，导致数据不一致

### 3.5 Spec 数据快照
- **条件**：EffectSpec 从 Pending 进入 Applied 时
- **不变量**：EffectSpec 必须快照当前时刻相关属性值（如施法者属性、目标属性），后续属性变化不影响该效果的计算
- **违反后果**：不做快照时，Effect 持续期间属性变化不可预测，回放不一致

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：运行时直接修改 Def 数据 — 理由：Def 是只读模板，修改应通过 Content 层的配置更新流程
- 🟥 禁止：跳过 Spec 层直接由 Def 创建 Instance — 理由：跳过 Spec 层会丢失角色维度的定制数据（等级/强化/冷却缩减）
- 🟥 禁止：同一实体的同一 AbilityDef 生成多个 AbilitySpec — 理由：一个实体对一个技能只能有一个配置实例（堆叠等特殊规则归 Stacking 领域）
- 🟥 禁止：Spec 直接持有运行时状态（如冷却进度条） — 理由：运行状态归 Instance 层，Spec 只做配置
- 🟥 禁止：SpecDef 中直接存储用户可见文本的自然语言 — 理由：必须使用 name_key/desc_key: LocalizationKey 引用。违反宪法 §22 Localization First。

---

## 5. 流程定义

### 5.1 Spec 授予

- **输入**：目标实体、Def Id、可选定制参数（等级/强化等）
- **处理**：
  1. 校验 Def Id 是否已注册（不变量 3.2）
  2. 校验该实体是否已有同 Def 的 Spec（不变量 4.3）
  3. 通过 SpecRegistry 基于 Def 创建 Spec 实例
  4. 应用定制参数（等级、冷却缩减、强化等）
  5. 注册 Spec 到实体的 SpecContainer
  6. 发布 SpecGranted 事件
- **输出**：Spec 实例，SpecGranted 事件
- **失败处理**：Def 不存在或重复授予时返回错误

### 5.2 Spec 等级变更

- **输入**：目标实体的 AbilitySpec、新等级
- **处理**：
  1. 校验新等级在 [1, MaxLevel] 范围内（不变量 3.3）
  2. 如果该 Spec 当前有活跃 Instance，检查是否允许等级变更（通常需等待 Instance 完成）
  3. 更新 Spec 等级
  4. 重新计算等级相关属性（如伤害骰、范围、消耗）
  5. 发布 SpecLevelChanged 事件
- **输出**：更新后的 Spec，SpecLevelChanged 事件
- **失败处理**：等级越界或 Instance 活跃中不允许变更时返回错误

### 5.3 Spec 移除

- **输入**：目标实体、Spec Id
- **处理**：
  1. 检查 Spec 当前是否有活跃 Instance
  2. 如果有活跃 Instance，级联终止所有关联 Instance（不变量 3.4）
  3. 从 SpecContainer 移除 Spec
  4. 发布 SpecRemoved 事件
- **输出**：移除确认，SpecRemoved 事件
- **失败处理**：要移除的 Spec 不存在时忽略（幂等移除）

### 5.4 Def→Spec 工厂转换

- **输入**：Def Id、目标实体、可选参数
- **处理**：
  1. 加载 Def 模板数据
  2. 应用目标实体的相关定制数据（从实体属性/容器中读取）
  3. 计算等级映射数据（如 3 级火球 → 伤害骰从 8d6 升为 10d6）
  4. 生成 Spec 实例
- **输出**：Spec 实例
- **失败处理**：Def 加载失败或数据不完整时创建失败

---

## 6. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| SpecGranted | Spec 成功授予到实体时 | entity_id, spec_type（ability/effect）, spec_id, def_id | Ability（注册可激活技能列表）、UI（更新技能栏）、日志（LogCode: CNT005） |
| SpecRemoved | Spec 从实体移除时 | entity_id, spec_id, reason（manual/expired/replaced） | Ability（清理关联）、UI（更新技能栏）、日志（LogCode: CNT006） |
| SpecLevelChanged | AbilitySpec 等级变更时 | entity_id, spec_id, old_level, new_level | Ability（更新能力参数）、Progression（确认升级效果）、日志（LogCode: CNT007） |
| SpecSnapshotTaken | EffectSpec 快照属性值时 | entity_id, spec_id, snapshot_data | 回放系统、回滚系统、日志（LogCode: CNT008） |

### 事件订阅关系图

```
SpecGranted
    │
    ├──→ Ability：将新技能加入实体的可激活技能列表
    ├──→ UI：技能栏刷新显示新技能
    └──→ Trigger：检查是否有新技能触发的自动激活条件

SpecRemoved
    │
    ├──→ Ability：从可激活技能列表移除，级联终止活跃 Instance
    ├──→ UI：技能栏移除技能图标
    └──→ Effect：级联终止关联的 EffectInstance

SpecLevelChanged
    │
    ├──→ Ability：更新技能参数（伤害/范围/消耗等）
    └──→ Progression：确认升级流程完整执行
```

---

## 7. 与已有架构的对齐校验

- ✅ 架构边界：Spec 能力领域位于 `core/capabilities/spec/`，foundation/ 定义 ability_spec.rs 和 effect_spec.rs，mechanism/ 定义 components.rs（SpecContainer）、spec_registry.rs，符合 C1→C2 分层
- ✅ 术语一致：AbilitySpec、EffectSpec、SpecRegistry 与架构文档第六节完全一致
- ✅ 三层分离：Def（content/schema/）→ Spec（core/capabilities/spec/）→ Instance（core/capabilities/ability/ + core/capabilities/effect/），严格遵循架构文档的三层分离原则
- ✅ 职责明确：Spec 只做"配置"，不做"运行"（Ability/Effect 领域的职责），不做"定义"（Content 层的职责）
- ✅ 解决核心问题：区分了"技能定义"和"角色身上的技能配置"——同一火球术在不同角色身上可以有不同等级
- ✅ LocalizationKey：Spec 使用 LocalizationKey 而非硬编码文本（宪法 §22）

---

## 8. 自检清单

- [x] 所有术语有唯一定义，与项目已有术语一致
- [x] 业务规则无"可能"、"也许"等模糊表述
- [x] 已检查 `docs/02-domain/` 下相关文档，无冲突
- [x] 未涉及代码实现细节（函数名、trait 名等）
- [x] 领域模型能完整覆盖 Spec 授予、等级变更、移除、Def→Spec 转换等全场景
- [x] 所有不变量和约束条件已识别（5 条不变量）
- [x] 禁止事项已明确列出（4 条禁止）
- [x] Def→Spec→Instance 三层分离定义清晰
- [x] 每个操作有完整的流程定义（授予、等级变更、移除、工厂转换）
