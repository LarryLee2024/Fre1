---
id: ECS-CONSTITUTION
title: ECS 宪法
status: accepted
stability: stable
layer: domain
related:
  - ai-constitution-complete.md
  - srpg-systems-constitution.md
tags:
  - ecs
  - communication
  - observer
  - execution-model
---

> **原文来源**：`ai-constitution-complete.md` 第六编（L730-L839）
> **锚定总宪法**：第六编

## 第六编 ECS 宪法
### 6.1 核心概念
- 🟩 **Entity 只是 ID**
  - 🟥 绝对禁止：在 Entity 上调用任何方法或将其当作面向对象实例使用
  - 允许将 Entity 作为纯 ID 参数传递
- 🟥 **Domain 层禁止裸 Entity**（详见 `docs/04-data/foundation/id-taxonomy.md`）
  - Domain/Application 层禁止裸 `Entity`、`u64`、`usize` 作为业务对象标识
  - 只允许使用显式命名的强类型 ID（如 `UnitId`、`AbilityId`）
  - Entity ↔ 业务 ID 映射集中在 Infrastructure 层（`integration/` 模块）
- 🟩 **数据与行为强制分离**
  - Component 只能存储纯数据状态，🟥 绝对禁止包含任何逻辑
  - System 只能包含纯逻辑，🟥 绝对禁止存储任何状态
- 🟩 **Tag Component 优先原则**
  - 实体持久状态必须优先使用空 Tag Component 实现
    - 合法：`Dead`、`Frozen`、`Stunned`
    - 非法：`is_dead: bool`、`is_frozen: bool`
  - 配置字段、临时计算值、非实体级状态允许使用 bool 类型
  - ⚠️ **重要区分**：此处的 "Tag Component"（ECS 单元结构体标记）≠ "Tag System"（语义标签系统）。两者是不同的概念：
    - **Tag Component** = ECS 模式 `struct Dead;`，用于标记实体状态，参与 Archetype 过滤
    - **Tag System** = `TagSet { bits: u128 }` + `TagHierarchy`，用于语义分类（Enemy.Boss, Ability.Fire），不参与规则计算
  - **ZST（Zero-Sized Type）概念**：Tag Component 即零大小类型（ZST），编译期零开销；ZST不仅用于实体标记，还用于泛型分类标记（如 `struct DamageTag;` 用于 `Effect<DamageTag>`）
- 🟩 **Interior Mutability 边界**
  - 只有 Resource 层和 Infra 层允许使用 RefCell/Cell/Mutex
  - Domain 层和 Capability 层禁止内部可变性
  - 与 ECS World 的交互必须通过 Commands/Events
  - 🟥 RefCell<T> 作为 Component 字段禁止（含运行时行为的"伪纯数据"）
    - 详见 `docs/02-domain/capabilities/tag_domain.md` §9 "Tag vs Type 决策指南"
- 🟨 **从属关系使用官方 Relationship**
  - 实体间从属关系优先使用 Bevy 官方 `Relationship` 机制实现
  - 适用场景：CasterOf、TargetOf、SummonedBy、OwnerOf 等实体间关系
  - 不适用场景：临时引用（如当前选中单位）、值语义关系（如队伍 ID）
- 🟩 **复杂度对比指导**：业务对象直接持有其他业务对象形成对象图，复杂度增长是 **指数级**（O(n²)）；业务对象仅持有 ID 引用，通过 Registry 查找关系，复杂度增长是 **线性级**（O(n)）。所有跨实体关系的架构决策必须以此为指导：
  - ✅ 正确：`struct Unit { id: UnitId }` + `unit_buffs.get(unit_id)`
  - ❌ 错误：`struct Unit { buffs: Vec<Buff> }` (对象图嵌套)
  - 🟥 禁止超过 2 层的对象图嵌套结构
  - 禁止手动维护父子实体 ID 字段（可用 Relationship 的场景）

### 6.2 ECS 使用边界规范
#### Core 层允许使用的 ECS 能力
- `#[derive(Component)]` 定义业务数据组件
- `Query`、`Single`、`Observer`、`Trigger`、`Delayed`
- `Schedule`、`SystemSet` 调度编排
- `Resource` 全局业务状态（优先使用 Singleton Entity Component）
- `run_if()` 系统运行条件
- `commands.trigger()` 事件链触发

#### Core 层禁止使用的 ECS/引擎能力
- `EventWriter` / `EventReader`（已废弃，使用 `trigger()` + Observer 替代）
- `AssetServer`、`Handle<Image>` 等资源加载类型
- `RenderDevice`、`Texture` 等渲染类型
- `Input`、`KeyCode` 等输入类型（已迁移为 `ButtonInput<T>`）
- `AudioPlayer`、`SpatialAudio` 等音频类型
- 所有 UI 组件与系统

### 6.3 四级通信机制（Hook / Trigger / Observer / Message）
优先级从高到低，优先使用最小粒度的通信方式，禁止事件滥用。
1. **Hook = 组件生命周期固有行为**
   - 组件添加/移除时的轻量固有副作用，通过 `#[component(on_add=..., on_remove=...)]` 声明
   - 仅用于简单状态联动，禁止承载复杂业务逻辑
2. **Trigger = Feature 内事件链载体**
   - 同一 Feature 内的多段响应逻辑、战斗事件链，使用 `commands.trigger()` 机制实现
   - 典型场景：伤害触发护盾、吸血、反击等连锁效果
3. **Observer = 跨领域通信首选机制**
   - 跨 Feature、跨 Domain 的事件响应使用 `On<T>` Observer 实现
   - 支持 `run_if()` 条件守卫，替代手动 if 判断
   - 典型场景：伤害 → 护盾 → 吸血 → 死亡判定（跨 Ability/Effect/Combat 领域）
4. **Message = 跨 Feature 全局广播（备选）**
   - 当 Observer 不适用时的全局事件备选方案
   - 典型场景：回合结束 → 同时通知 Quest/Progression/AI 多个领域

#### 补充规则
- 🟩 模块内部优先函数调用，🟥 绝对禁止将同一模块内的普通逻辑全部事件化
- 🟩 领域事件是唯一业务事实源，日志、战斗回放、UI 履历、成就、任务均为事件的下游消费者
- 🟩 领域事件白名单管理，新增事件必须先更新白名单，🟥 绝对禁止为临时副作用随意新增领域事件
- 🟥 **Observer 深度限制**：Observer 递归深度必须限制在 `MAX_OBSERVER_DEPTH`（默认 10）以内，禁止 Observer 触发同一事件类型的递归循环无保护
- 🟥 **系统互调禁令**：绝对禁止系统函数直接互相调用（如 `fn system_a()` 内部调用 `fn system_b()`）
  系统间通信必须通过四级通信机制（Hook→Trigger→Observer→Message）进行
  架构扫描已确认当前代码零违规，此条款为前瞻性防护

### 6.4 ECS 执行模型
- 🟩 **ECS 是数据流，不是调用链**
  - 🟥 绝对禁止模拟面向对象的调用方式，如 `player.attack(enemy)`
  - 正确方式：创建命令组件，由对应系统统一处理
- 🟩 **组件依赖声明**：组件依赖必须通过 `#[require(Component)]` 属性声明，🟥 绝对禁止手动检查并补全缺失的组件
- 🟩 **状态变更检测**
  - 组件生命周期变化优先使用 Bevy 原生 `Added`、`Changed`、`Removed` 过滤器
  - 🟥 绝对禁止手写 bool 脏标记字段检测组件变化
  - 业务状态机允许使用显式状态组件，自行管理状态流转
- 🟩 **Resource 使用规范**
  - Resource 只能存储真正的全局唯一状态
  - 🟥 绝对禁止将 Resource 当作全局变量仓库使用
- 🟩 **性能约束**
  - 🟥 绝对禁止在高频逻辑中使用 Observer 造成事件风暴
  - 每帧执行 10 次以上的逻辑必须直接使用 System 处理
- 🟩 **状态管理规范**
  - 全局流程状态优先使用 `States`、`SubStates`、`ComputedStates` 实现
  - 禁止手动维护全局状态枚举与切换逻辑
- 🟨 **运行条件优先**
  - 系统的执行前置判断优先使用 `run_if()` 条件表达
  - Observer 也支持 `run_if()`，用于条件守卫
  - 避免在系统或 Observer 内部写大段 if 判断是否执行
- 🟨 **Delayed Commands 优先于 Timer**
  - 一次性延迟效果使用 `Delayed<T>` 或 `FreDelayed<T>` 实现
  - Timer 仅用于基础设施层的周期性任务（热重载、审计）或需要暂停/恢复的长周期 Buff
  - 🟥 禁止单纯的"等 X 秒后执行 Y"用 Timer 轮询实现
- 🟩 **Schedule 权责划分**
  - `PreUpdate`：输入处理、命令执行、状态同步
  - `Update`：核心业务逻辑、规则结算
  - `PostUpdate`：事件响应、表现层更新、UI 刷新
  - 禁止跨 Schedule 乱放系统，禁止在错误阶段执行业务逻辑
