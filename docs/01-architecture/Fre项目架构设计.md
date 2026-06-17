# Fre 项目架构设计文档

> **引擎**: Bevy 0.18+ | **类型**: SRPG | **规模**: 50万行+ | **周期**: 10-20年
> **开发模式**: 单人开发 + AI 协作 | **对标**: D&D 5e / BG3 / 铃兰之剑

---

## 一、设计原则

| 原则 | 说明 |
|------|------|
| **领域优先** | 游戏领域概念（Tag/Attribute/Ability）与架构组件（Container/Manager）严格分离 |
| **组合优于创建** | 新玩法优先组合已有机制（Cost=Effect, Cooldown=Tag+Effect），不造新系统 |
| **三层分离** | Def（模板）→ Spec（配置）→ Instance（运行时），贯穿能力系统全链路 |
| **内聚优于分层** | 同一领域的代码放在一起（内聚），而非按抽象层级拆散到不同目录（分层） |
| **单向依赖** | Domains → Capabilities → Shared，禁止反向；Domain 间仅通过事件通信 |
| **数据驱动** | 玩法规则下沉 Domain/rules/，数值配置归 content/，代码只提供机制 |
| **可换可测** | 每层可独立替换与测试，纯函数规则零 ECS 依赖 |
| **个人开发友好** | 避免团队级复杂流程，优先轻量方案 |

---

## 二、顶层目录结构

```
Fre/
├── Cargo.toml                  # 工作空间根配置，定义 feature 与依赖
├── rustfmt.toml                # 格式化规则
├── clippy.toml                 # Clippy 规则配置
├── .gitignore
│
├── src/                        # 源码主目录（DDD三层 + 横切四层）
├── tests/                      # 分层测试体系
├── assets/                     # 游戏资源与配置数据
├── docs/                       # 架构文档与规范
├── scripts/                    # 工程脚本
└── crates/                     # 可独立拆分的通用库（可选，初期合并到 src）
```

---

## 三、源码架构总览

```
src/
├── main.rs                   # 程序入口，根据 feature 启动 game/editor/headless
├── lib.rs                    # 库根，导出各层公共接口
│
│                           ┌─ DDD 纵向三层 ─┐
├── shared/                   # L0：底层原子层。零业务语义的通用工具
├── core/                     # L1：领域规则层。能力机制 + 业务子系统
├── infra/                    # L2：技术实现层。渲染/持久化/网络等"脏活"
│                           └─────────────────┘
│                           ┌─ 横切四层 ─┐
├── app/                      # 横切1：启动装配层（Composition Root）
├── content/                  # 横切2：内容桥接层（数据驱动核心）
├── tools/                    # 横切3：开发工具层（feature-gated）
└── modding/                  # 横切4：Mod 扩展层（跨层聚合）
                            └─────────────┘
```

### 依赖方向（严格单向，禁止反向）

```
Domains ──→ Capabilities ──→ Shared
  │              │              │
  │              ↓              │
  │       (C1 Foundation →     │
  │        C2 Mechanism →      │
  │        C3 Runtime)         │
  │                              │
  └──── 事件通信 ────┘
       (禁止直接引用)

Shared ──→ Core ──→ Infra     # 依赖方向：高层依赖低层
App   ──→ 知道所有层（唯一 Composition Root）
Content ──→ Core + Infra（只做加载/校验/注册）
```

---

## 四、Shared 层（L0）

> 零业务语义、零技术语义、零框架语义的通用编程原子工具

```
src/shared/
├── mod.rs
│
├── ids/                        # 强类型 ID
│   ├── mod.rs
│   ├── unit_id.rs              # 单位唯一标识
│   ├── skill_id.rs             # 技能唯一标识
│   ├── buff_id.rs              # Buff 唯一标识
│   ├── item_id.rs              # 物品唯一标识
│   └── quest_id.rs             # 任务唯一标识
│
├── error/                      # 错误工具
│   ├── mod.rs
│   ├── error_context.rs        # 错误上下文附加信息
│   └── log_if_error.rs         # 错误条件日志输出
│
├── result/                     # 通用 Result 辅助
│   ├── mod.rs
│   └── result_ext.rs           # Result 类型别名与扩展方法
│
├── math/                       # 纯数学工具
│   ├── mod.rs
│   ├── distance.rs             # 距离计算（曼哈顿/切比雪夫/欧几里得）
│   ├── interpolation.rs        # 插值算法（线性/贝塞尔/缓动曲线）
│   └── grid_math.rs            # 网格坐标变换与邻域计算
│
├── random/                     # 确定性随机基础接口
│   ├── mod.rs
│   └── rng_trait.rs            # 确定性随机数生成器 Trait（支持回放）
│
├── time/                       # 时间基础类型
│   ├── mod.rs
│   ├── game_time.rs            # 游戏内时间（秒/帧/时间缩放）
│   └── turn_count.rs           # 回合计数器
│
├── collections/                # 通用集合扩展
│   ├── mod.rs
│   └── sparse_set.rs           # 稀疏集（ECS 友好的高速查找结构）
│
├── hashing/                    # 哈希工具
│   ├── mod.rs
│   └── fast_hash.rs            # 非加密高速哈希（FnvHash/xxHash 封装）
│
├── validation/                 # 通用校验工具
│   ├── mod.rs
│   └── validate_ext.rs         # 链式校验器与范围检查
│
├── testing/                    # 测试基础工具（test 配置下启用）
│   ├── mod.rs
│   ├── test_builder.rs         # 测试数据构建器
│   └── test_world.rs            # 最小 Bevy World 构建工具
│
├── traits/                     # 横切能力抽象
│   ├── mod.rs
│   ├── logging_trait.rs         # 日志抽象接口（详见第二十一节）
│   ├── audit_trait.rs           # 审计追踪 Trait
│   └── transaction_trait.rs     # 事务 Trait（原子操作回滚）
│
├── path/                       # 路径工具
└── prelude/                    # 统一导出
```

---

## 五、Core 层（L1）—— 项目核心

Core 层是整个项目的领域规则层，包含三大子系统：

| 子系统 | 职责 | 模块数 |
|--------|------|--------|
| **capabilities/** | 15个核心能力领域，通用机制骨架，与玩法无关 | 15 |
| **domains/** | 业务子系统，承载全部玩法复杂度 | 15+ |
| **mod_api/** | Mod 稳定 API，唯一对外暴露的核心接口 | - |

```
src/core/
├── mod.rs
├── core_plugin.rs              # Core 层总 Plugin，注册所有能力与领域
│
├── capabilities/               # 15个核心能力领域（详见第六节）
├── domains/                    # 业务子系统（详见第七节）
└── mod_api/                    # Mod 稳定 API（详见第八节）
```

---

## 六、Capabilities —— 15个核心能力领域

> 融合 UE GAS 设计哲学与 Bevy ECS 原生优势的通用机制骨架。
> 玩法无关，提供原子能力；所有业务复杂度向 Domain 沉淀。

### 6.1 领域总览

| 分组 | 领域 | 职责 | GAS 对应 |
|------|------|------|----------|
| **核心基石** | Tag | 标签定义、位掩码、层级关系 | FGameplayTag |
| | Attribute | 属性定义、基础值、当前值 | FGameplayAttribute |
| | Modifier | 修改器定义（Add/Mul/Override） | FGameplayModifier |
| | Aggregator | 属性聚合管线（Base→Add→Mul→Override→Clamp→Final） | FAttributeAggregator |
| **逻辑骨架** | GameplayContext | 统一上下文/载荷，贯穿全链路的数据载体 | FGameplayEffectContextHandle |
| | Spec | 模板→配置→实例三层分离 | AbilitySpec / EffectSpec |
| | Ability | 技能逻辑与生命周期 | UGameplayAbility |
| | Trigger | 技能激活条件 | EGameplayAbilityTriggerSource |
| | Condition | 统一条件检查（含免疫/限制/激活条件） | ApplicationReqs / BlockedTags |
| **行为表现** | Targeting | 目标选择（含 Grid 范围） | FTargetData |
| | Execution | 执行计算（伤害/治疗/自定义） | UExecutionCalculation |
| | Effect | 效果（含 Period/Duration/Instant） | UGameplayEffect |
| | Stacking | 堆叠规则 | FGameplayEffectStackingDef |
| | Event | 系统间结构化通信 | GameplayEvent |
| | Cue | 表现层信号（VFX/SFX/动画触发） | UGameplayCue |

### 6.2 核心数据流

```
[Spec] 配置了 [Ability]
  → 满足 [Condition] 且被 [Trigger] 激活
  → 消耗 [GameplayContext] 资源，通过 [Targeting] 选定目标
  → 进入 [Execution] 计算，生成带有 [GameplayContext] 的 [Effect]
  → [Effect] 携带 [Modifier]，经过 [Aggregator] 改变 [Attribute]
  → 检查 [Stacking] 和 [Tag] 冲突，触发 [Event] 和 [Cue]
```

### 6.3 三层分离原则

```
Def（模板/资源）  →  Spec（配置/槽位）  →  Instance（运行时实例）

AbilityDef        →  AbilitySpec         →  AbilityInstance
  技能定义            角色身上的技能        激活中的技能实例
  (content/)         (Spec 领域)          (Ability 领域)

EffectDef         →  EffectSpec          →  EffectInstance
  效果定义            应用后的实例          运行中的效果
  (content/)         (Spec 领域)          (Effect 领域)
```

### 6.4 组合优于创建——GAS 核心设计哲学

| 常见需求 | 错误做法 | 正确做法（组合已有机制） |
|----------|----------|--------------------------|
| 法力消耗 | ManaSystem | Attribute(Mana) + Effect(Cost) |
| 冷却时间 | CooldownSystem | Tag(Cooldown.X) + Effect(Duration) |
| 怒气/体力/行动点 | 多套资源系统 | Attribute + ResourcePipeline |
| 免疫 | ImmunitySystem | Tag(Immune.X) + Condition(Check) |
| 光环 | AuraSystem | Targeting(Area) + Effect(Infinite) |
| DoT/HoT | DoTSystem | Effect(Duration + Period) |
| 暴击/闪避 | CritSystem | Execution(CustomCalc) + Modifier |
| 反击 | CounterAttackSystem | Trigger(OnAttacked) + Ability(Counter) |

### 6.5 Capabilities 内部三层架构

> # Capabilities 内部三层架构说明
> # 设计思路：将原5层架构（atom/rule/runtime/model/frame）精简为3层，
> # 核心理由是"内聚优于分层"——同一领域的代码应放在一起，而非按抽象层级拆散。
> # 原L0(atom)+L1(rule)的人为分裂导致同一领域代码跨层修改，违反内聚性；
> # 原L3(model)与Domain的components.rs职责重叠，造成数据结构双份维护；
> # 原L4(frame)与Domain的plugin.rs高度重复，增加"放L4还是放Domain"的决策负担。
> # 调整后：每个能力领域内部自包含（Foundation+Mechanism），跨领域运行时基础设施独立成层。

#### 三层职责划分

| 层 | 名称 | 职责 | 禁止事项 | 对应原5层 |
|------|------|------|----------|----------|
| **C1** | Foundation | 纯数据定义、类型枚举、值对象 | 禁止包含任何行为逻辑、系统、ECS组件 | L0 atom |
| **C2** | Mechanism | 规则组件、查询系统、生命周期管理、ECS组件与System | 禁止包含具体玩法内容 | L1 rule + L2 runtime(部分) |
| **C3** | Runtime | 跨领域能力的运行时编排底座 | 禁止包含具体业务逻辑 | L2 runtime(部分) + L4 frame(事件总线) |

#### 层间交互规范

```
# 依赖方向：C3 → C2 → C1 → Shared（严格单向）
#
# C1 Foundation：被 C2 引用，提供类型定义
#   - 各领域内的 *_id.rs, *_type.rs, *_value.rs
#   - 无 System、无 Component、无 Event
#
# C2 Mechanism：引用 C1 + Shared，提供规则与系统
#   - 各领域内的 components.rs, systems/, query, lifecycle
#   - 可定义 Component、System、Event
#
# C3 Runtime：引用 C2 + C1 + Shared，提供跨领域编排
#   - pipeline/, scheduler/, registry/, command/, replay/
#   - 只提供执行机制，不感知具体业务内容
#
# 原5层中 L3(model) → 归入对应 Domain 的 components.rs
# 原5层中 L4(frame) → 归入对应 Domain 的 plugin.rs + integration.rs
# 原5层中 L4(event_bus) → 归入 capabilities/event/
```

#### C2↔C3 交互判定规则

> # 何时用 C2（领域内聚）vs C3（跨领域编排）
> # 核心判定：逻辑是否只涉及单一领域？
> #   - 是 → C2 Mechanism（领域内 systems/ 处理）
> #   - 否 → C3 Runtime（pipeline/scheduler 编排多领域）

| 场景 | 归属 | 理由 |
|------|------|------|
| 标签同步（Tag→Tag） | C2 tag/systems | 单领域内部逻辑 |
| 属性聚合（Modifier→Aggregator→Attribute） | C2 各领域 + C3 pipeline | Modifier/Aggregator/Attribute 分属不同领域，需 C3 pipeline 编排执行顺序 |
| 技能激活（Condition→Trigger→Ability→Targeting→Execution→Effect） | C3 pipeline | 跨6个领域的链式执行，必须由 pipeline 编排 |
| 效果施加（Effect→Stacking→Tag→Event） | C2 effect + C3 scheduler | Effect 施加是单领域，但后续 Stacking/Tag 检查需 C3 调度 |
| 回合调度（TurnScheduler） | C3 scheduler | 跨所有战斗参与者的时序编排 |
| 堆叠规则检查 | C2 stacking/systems | 单领域内部逻辑 |

#### 错误传播模式

```
# 跨层错误传播规则：
#
# 1. C1 Foundation → C2 Mechanism
#    - C1 不返回错误（纯数据定义，校验由 C2 负责）
#    - C2 对 C1 数据做合法性校验，不合法时返回 DomainError
#
# 2. C2 Mechanism → C3 Runtime
#    - C2 返回 Result<T, DomainError>，C3 负责处理
#    - C3 pipeline 中单步骤失败不中断整条管线：
#      - 可恢复错误（如条件不满足）→ 跳过当前步骤，继续后续步骤
#      - 不可恢复错误（如数据损坏）→ 中断管线，向上层返回 PipelineError
#
# 3. C3 Runtime → Domain
#    - Domain 的 integration.rs 捕获 C3 返回的 PipelineError
#    - 转换为领域特定的业务错误（如 CombatError::PipelineFailed）
#    - 通过 Event 通知其他 Domain
#
# 4. Domain → Domain
#    - 禁止直接传播错误，仅通过 Event 通信
#    - 错误事件（如 SpellCastFailed）携带错误码与上下文
#    - 消费方 Domain 自行决定如何响应
#
# 5. 错误日志规范
#    - 所有 Result::Err 必须在返回前用 log_error! 记录
#    - C2 内部错误用 log_warn!（可恢复）或 log_error!（不可恢复）
#    - C3 pipeline 错误用 log_error! 并附带 pipeline 阶段信息
```

#### 各能力领域内部结构（遵循 C1→C2 内聚）

每个能力领域内部自包含 Foundation + Mechanism，不再跨目录拆分：

```
capabilities/<domain>/
├── plugin.rs              # 领域 Plugin（C2）
├── foundation/            # C1：纯数据定义层
│   ├── mod.rs
│   ├── types.rs           # 基础类型与枚举
│   └── values.rs          # 值对象定义
├── mechanism/             # C2：规则与系统层
│   ├── mod.rs
│   ├── components.rs      # ECS 组件
│   ├── query.rs           # 查询/匹配/条件逻辑
│   ├── lifecycle.rs       # 生命周期管理
│   └── systems/            # Bevy Systems
│       ├── mod.rs
│       └── xxx_system.rs
└── events.rs              # 领域事件（C2）
```

#### C3 Runtime 独立模块

```
capabilities/runtime/       # C3：跨领域运行时编排底座
├── plugin.rs
├── pipeline/              # 通用执行管线框架
│   ├── mod.rs
│   ├── pipeline_stage.rs  # 阶段化执行与钩子扩展点
│   └── pipeline_context.rs # 执行上下文通用抽象
├── scheduler/             # 时序调度器
│   ├── mod.rs
│   ├── turn_scheduler.rs  # 回合调度通用框架
│   └── effect_scheduler.rs # 效果时序调度
├── registry/               # 统一注册中心
│   ├── mod.rs
│   └── registry_trait.rs  # 注册中心通用接口
├── command/                # 通用命令模式框架
│   ├── mod.rs
│   └── command_trait.rs   # 命令接口与队列
└── replay/                 # 回放基础框架
    ├── mod.rs
    ├── recorder.rs        # 命令录制与种子管理
    └── replay_frame.rs    # 回放重放通用流程
```

### 6.6 各领域详细结构

#### Tag（标签）

```
capabilities/tag/
├── plugin.rs              # Tag Plugin，注册标签系统与同步逻辑
├── foundation/            # C1：标签基础数据定义
│   ├── mod.rs
│   ├── tag_id.rs           # TagId 强类型，标签唯一标识
│   ├── tag_set.rs          # TagSet 位掩码集合，O(1) 包含检查
│   └── tag_hierarchy.rs    # 标签层级关系（父标签自动包含子标签）
├── mechanism/             # C2：标签规则与系统
│   ├── mod.rs
│   ├── components.rs      # GameTagContainer 组件，挂载于每个有标签的 Entity
│   ├── tag_query.rs        # Tag 条件查询（Any/All/None 三种匹配模式）
│   └── systems/
│       ├── mod.rs
│       ├── tag_sync_system.rs   # 标签同步与层级推导（父标签变化时更新子标签状态）
│       └── tag_query_system.rs  # 标签条件评估（供 Condition 领域调用）
└── events.rs               # TagAdded / TagRemoved 事件（供其他领域订阅）
```

#### Attribute（属性）

```
capabilities/attribute/
├── plugin.rs              # Attribute Plugin
├── foundation/            # C1：属性基础数据定义
│   ├── mod.rs
│   ├── attribute_id.rs     # AttributeId 强类型，属性唯一标识
│   ├── attribute_value.rs  # AttributeValue { base, current }，基础值与当前值分离
│   └── attribute_category.rs # 属性分类枚举（Primary/Secondary/Derived/Resource）
├── mechanism/             # C2：属性规则与系统
│   ├── mod.rs
│   ├── components.rs      # AttributeContainer 组件，挂载于每个有属性的 Entity
│   └── systems/
│       ├── mod.rs
│       └── attribute_init_system.rs  # 属性初始化（从 Def 加载基础值）
└── events.rs               # AttributeChanged 事件（供 Aggregator/UI 订阅）
```

#### Modifier（修改器）

```
capabilities/modifier/
├── plugin.rs              # Modifier Plugin
├── foundation/            # C1：修改器基础数据定义
│   ├── mod.rs
│   ├── modifier_op.rs      # 修改操作枚举（Add/Multiply/Override），决定 Aggregator 计算顺序
│   ├── modifier_data.rs    # ModifierData { op, attribute_id, magnitude, priority }
│   └── scalable_value.rs   # ScalableValue（支持固定值/曲线/属性缩放），防止硬编码数值
├── mechanism/             # C2：修改器规则与系统
│   ├── mod.rs
│   ├── components.rs      # ModifierContainer 组件，存储 Entity 上所有活跃修改器
│   └── systems/
│       ├── mod.rs
│       └── modifier_lifecycle_system.rs  # 修改器生命周期管理（添加/移除/过期）
└── events.rs               # ModifierApplied / ModifierRemoved 事件
```

#### Aggregator（聚合器）

```
capabilities/aggregator/
├── plugin.rs              # Aggregator Plugin
├── foundation/            # C1：聚合器基础数据定义
│   ├── mod.rs
│   ├── calc_stage.rs       # 计算阶段枚举与优先级（Add→Multiply→Override→Clamp）
│   └── snapshot.rs         # 快照数据结构（记录某一时刻的 Modifier 状态，用于回滚）
├── mechanism/             # C2：聚合器规则与系统
│   ├── mod.rs
│   ├── components.rs      # AggregatorState 组件，缓存聚合结果
│   ├── calc_pipeline.rs   # 属性计算管线
│   │                       # Base → [+Add] → [*Multiply] → [Override] → [Clamp] → Final
│   │                       # 设计思路：GAS 的 FAttributeAggregator 处理修饰符优先级排序、
│   │                       # 不同运算类型执行顺序、Clamp 限制、快照机制。独立为领域防止属性系统爆炸。
│   └── systems/
│       ├── mod.rs
│       ├── aggregate_system.rs  # 核心聚合计算（每帧检测 Modifier 变化时重算）
│       └── snapshot_system.rs   # 快照管理（战斗开始/关键节点拍摄快照）
└── events.rs               # AggregationComplete 事件
```

#### GameplayContext（上下文/载荷）

```
capabilities/gameplay_context/
├── plugin.rs              # GameplayContext Plugin
├── foundation/            # C1：上下文基础数据定义
│   ├── mod.rs
│   └── context_data.rs     # GameplayContextData 统一数据载体
│                           # { source, target, ability, weapon, element, crit, ... }
│                           # 设计思路：所有跨系统传递的结构化数据统一走此载体，
│                           # 避免各 Event 重复定义 source/target/ability 等字段，
│                           # 防止几年后字段膨胀（如 DamageEvent 加 summon_owner/is_reflect 等）
├── mechanism/             # C2：上下文规则与系统
│   ├── mod.rs
│   ├── components.rs      # GameplayContext 组件
│   ├── context_builder.rs # Builder 模式构建上下文（链式调用设置字段）
│   ├── context_chain.rs   # 上下文链（反击/连锁/伤害转移的溯源链）
│   │                       # 设计思路：A打B→B反伤→反弹给A，需要溯源链防止无限循环
│   └── systems/
│       ├── mod.rs
│       └── context_cleanup_system.rs  # 上下文生命周期清理
└── events.rs               # ContextCreated 事件
```

#### Spec（规格/配置）

```
capabilities/spec/
├── plugin.rs              # Spec Plugin
├── foundation/            # C1：Spec 基础数据定义
│   ├── mod.rs
│   ├── ability_spec.rs     # AbilitySpec { def_id, level, input_binding, cooldown_override, ... }
│   │                       # 角色学会火球 Lv3 后的运行时配置，不属于 AbilityDef
│   └── effect_spec.rs      # EffectSpec { def_id, source_context, duration, stack_count, ... }
│                           # 效果应用后的实例，包含快照与上下文
├── mechanism/             # C2：Spec 规则与系统
│   ├── mod.rs
│   ├── components.rs      # AbilitySpecContainer / EffectSpecContainer 组件
│   ├── spec_registry.rs   # Spec 注册中心（Def → Spec 的工厂）
│   └── systems/
│       ├── mod.rs
│       ├── spec_grant_system.rs    # 授予/移除 Spec（角色获得/失去技能时调用）
│       └── spec_level_system.rs    # Spec 等级变更（升级/降级时更新属性）
└── events.rs               # SpecGranted / SpecRemoved 事件
```

#### Ability（技能逻辑）

```
capabilities/ability/
├── plugin.rs              # Ability Plugin
├── foundation/            # C1：技能基础数据定义
│   ├── mod.rs
│   ├── ability_state.rs    # 技能状态枚举（Ready/Casting/Active/Cooldown/Blocked）
│   ├── ability_instance.rs # 运行时技能实例（激活中的技能数据）
│   ├── cost.rs             # 消耗定义（复用 Effect，不造新系统）
│   │                       # 设计思路：法力消耗 = Attribute(Mana) + Effect(Cost)，
│   │                       # 怒气/体力/行动点同理，统一走 Attribute + Effect 组合
│   └── cooldown.rs         # 冷却定义（复用 Tag+Effect，不造新系统）
│                           # 设计思路：冷却 = Tag(Cooldown.Fireball) + Effect(Duration)，
│                           # Condition 检查 Tag 存在即阻止激活，Effect 到期移除 Tag
├── mechanism/             # C2：技能规则与系统
│   ├── mod.rs
│   ├── components.rs      # ActiveAbilityContainer 组件
│   ├── ability_task.rs     # 异步任务编排
│   │                       # 设计思路：Bevy 原生 State/Event/Observer/Timer 替代 UE AbilityTask。
│   │                       # WaitTagAdded → On<TagAdded>，WaitEvent → EventReader<T>。
│   │                       # 不作为顶级领域，因为 Bevy 已天然提供异步能力。
│   └── systems/
│       ├── mod.rs
│       ├── ability_activate_system.rs  # 技能激活流程（检查Condition→消耗Cost→进入Casting）
│       ├── ability_update_system.rs    # 技能状态更新（Casting→Active→Cooldown）
│       ├── ability_cancel_system.rs    # 技能取消/打断（被打断时触发事件）
│       └── ability_cooldown_system.rs  # 冷却管理（Effect到期→移除Cooldown Tag）
└── events.rs               # AbilityActivated / AbilityCompleted / AbilityCancelled 事件
```

#### Trigger（触发器）

```
capabilities/trigger/
├── plugin.rs              # Trigger Plugin
├── foundation/            # C1：触发器基础数据定义
│   ├── mod.rs
│   ├── trigger_type.rs     # 触发类型枚举（OnTagAdded/OnDamaged/OnTurnStart/OnDeath/Custom）
│   └── trigger_condition.rs # 触发条件定义（什么条件下激活技能）
├── mechanism/             # C2：触发器规则与系统
│   ├── mod.rs
│   ├── components.rs      # TriggerContainer 组件
│   └── systems/
│       ├── mod.rs
│       └── trigger_eval_system.rs   # 触发条件评估（检测触发条件是否满足）
└── events.rs               # TriggerFired 事件
```

> # Trigger 与 Event 的区别
> # Trigger 解决"什么条件下激活技能"——如受到攻击→激活反击
> # Event 解决"系统间如何传递结构化数据"——如 DamageDealt→触发连锁闪电
> # 两者职责不同，必须分离。Trigger 消费者是 Ability 系统，Event 消费者是全系统。

#### Condition（条件/限制/免疫）

```
capabilities/condition/
├── plugin.rs              # Condition Plugin
├── foundation/            # C1：条件基础数据定义
│   ├── mod.rs
│   ├── condition_type.rs   # 条件类型枚举
│   │                       # TagRequirement / AttributeCheck / ResourceCheck / Custom
│   ├── tag_requirement.rs  # Tag 条件（Require Has / Require Not / Require Any）
│   │                       # 免疫本质：Tag(Immune.Fire) + Condition(Require Not Tag)
│   ├── attribute_check.rs  # 属性阈值检查（如力量≥15才能装备）
│   └── resource_check.rs   # 资源充足检查（如法力≥20才能施法）
├── mechanism/             # C2：条件规则与系统
│   ├── mod.rs
│   ├── components.rs      # ConditionContainer 组件
│   └── systems/
│       ├── mod.rs
│       └── condition_eval_system.rs  # 条件评估（统一入口，返回 Pass/Fail）
└── events.rs               # ConditionPassed / ConditionFailed 事件
```

> # 为什么 Condition 替代 Immunity
> # 免疫本质是 Tag(Immune.X) + Condition(Require Not Tag) 的组合。
> # 如果独立 Immunity 领域，以后 Resistance/Block/Restriction 都要独立，导致领域爆炸。
> # Condition 统一处理 Tag需求/免疫/激活条件/应用条件，更优雅。

#### Targeting（目标选择）

```
capabilities/targeting/
├── plugin.rs              # Targeting Plugin
├── foundation/            # C1：目标选择基础数据定义
│   ├── mod.rs
│   ├── target_type.rs      # 目标类型枚举（Self/Ally/Enemy/Area/Line/Cone/Chain）
│   └── target_data.rs      # TargetData { entities, positions, context }
├── mechanism/             # C2：目标选择规则与系统
│   ├── mod.rs
│   ├── components.rs      # TargetingComponent 组件
│   ├── selector.rs         # 范围筛选、优先级排序通用实现
│   ├── grid_targeting.rs   # 网格目标选择（SRPG 核心需求，六角/四角网格适配）
│   └── systems/
│       ├── mod.rs
│       ├── target_select_system.rs    # 目标选择（根据 TargetType 筛选合法目标）
│       └── target_validate_system.rs  # 目标合法性校验（射程/视野/障碍）
└── events.rs               # TargetSelected / TargetChanged 事件
```

#### Execution（执行计算）

```
capabilities/execution/
├── plugin.rs              # Execution Plugin
├── foundation/            # C1：执行计算基础数据定义
│   ├── mod.rs
│   ├── execution_type.rs   # 执行类型枚举（Damage/Heal/Custom）
│   ├── execution_context.rs # 执行上下文（从 GameplayContext 派生，携带计算所需全部信息）
│   └── custom_execution.rs # 自定义执行 Trait（Domain 可注册自定义计算逻辑）
├── mechanism/             # C2：执行计算规则与系统
│   ├── mod.rs
│   ├── components.rs      # ExecutionState 组件
│   ├── damage_execution.rs # 伤害计算执行（调用 Domain 的 damage_formula）
│   ├── heal_execution.rs   # 治疗计算执行
│   └── systems/
│       ├── mod.rs
│       └── execution_system.rs  # 执行管线（分发到对应 ExecutionType 的计算逻辑）
└── events.rs               # ExecutionCompleted 事件
```

#### Effect（效果）

```
capabilities/effect/
├── plugin.rs              # Effect Plugin
├── foundation/            # C1：效果基础数据定义
│   ├── mod.rs
│   ├── effect_duration.rs  # 持续时间定义（Instant/HasDuration/Infinite）
│   ├── effect_period.rs    # 周期 Tick 定义（DoT/HoT 的每跳间隔）
│   │                       # 设计思路：Period 是 Effect 的属性而非独立领域。
│   │                       # 光环 = Targeting(Area) + Effect(Infinite)，不造 AuraSystem。
│   ├── effect_modifiers.rs # 效果携带的修改器列表
│   └── effect_tags.rs     # 效果授予/需要的标签
├── mechanism/             # C2：效果规则与系统
│   ├── mod.rs
│   ├── components.rs      # ActiveEffectContainer 组件
│   ├── effect_lifecycle.rs # 生命周期（施加→持续→到期→移除 四阶段）
│   └── systems/
│       ├── mod.rs
│       ├── effect_apply_system.rs     # 效果施加（检查Condition→应用Modifier→授予Tag）
│       ├── effect_duration_system.rs  # 持续时间管理（倒计时→到期→触发移除）
│       ├── effect_period_system.rs    # 周期 Tick（DoT每跳→触发Execution）
│       └── effect_remove_system.rs    # 效果移除（回退Modifier→移除Tag→触发Cue）
└── events.rs               # EffectApplied / EffectRemoved / EffectTicked 事件
```

#### Stacking（堆叠规则）

```
capabilities/stacking/
├── plugin.rs              # Stacking Plugin
├── foundation/            # C1：堆叠基础数据定义
│   ├── mod.rs
│   ├── stacking_type.rs    # 堆叠类型枚举（None/Aggregate/Replace/RefreshDuration）
│   ├── stacking_rule.rs    # 堆叠规则定义（同源/异源/按Tag分组）
│   └── stacking_limit.rs   # 堆叠上限与溢出处理（溢出时刷新持续时间/移除最早层）
├── mechanism/             # C2：堆叠规则与系统
│   ├── mod.rs
│   ├── components.rs      # StackingState 组件
│   └── systems/
│       ├── mod.rs
│       └── stacking_system.rs  # 堆叠规则处理（新Effect应用时检查Stacking规则）
└── events.rs               # StackAdded / StackRemoved / StackOverflow 事件
```

#### Event（系统通信）

```
capabilities/event/
├── plugin.rs              # Event Plugin
├── foundation/            # C1：事件基础数据定义
│   ├── mod.rs
│   ├── gameplay_event.rs   # GameplayEvent { tag, context, magnitude }
│   │                       # 统一事件结构，context 携带 GameplayContextData
│   └── event_type.rs       # 事件类型枚举
├── mechanism/             # C2：事件规则与系统
│   ├── mod.rs
│   ├── event_bus.rs        # 全局领域事件总线（Domain 间通信的唯一通道）
│   ├── event_subscription.rs # 事件订阅管理（自动注册/注销）
│   └── systems/
│       ├── mod.rs
│       └── event_dispatch_system.rs  # 事件分发（按订阅关系投递）
└── events.rs               # （Event 领域自身不发布事件，避免循环）
```

> # Event 与 Trigger 的区别
> # Trigger 解决"什么条件下激活技能"——消费者是 Ability 系统
> # Event 解决"系统间如何传递结构化数据"——消费者是全系统
> # 链式闪电/反弹伤害/击杀奖励等高级玩法依赖事件传递

#### Cue（表现层信号）

```
capabilities/cue/
├── plugin.rs              # Cue Plugin
├── foundation/            # C1：表现信号基础数据定义
│   ├── mod.rs
│   ├── cue_type.rs         # 信号类型枚举（VFX/SFX/Animation/Shake/Popup）
│   ├── cue_data.rs         # CueData { cue_id, context, parameters }
│   └── cue_tag.rs          # Cue 关联的 Tag（OnApply/OnRemove/OnTick 三种触发时机）
├── mechanism/             # C2：表现信号规则与系统
│   ├── mod.rs
│   ├── components.rs      # CueContainer 组件
│   └── systems/
│       ├── mod.rs
│       └── cue_dispatch_system.rs  # 信号分发到 Infra 表现层（解耦逻辑与表现）
└── events.rs               # CueTriggered 事件（Infra 层订阅此事件执行实际渲染/音效）
```

### 6.7 Capabilities 内部依赖关系

```
                    ┌─────────────────────────────────────┐
                    │          核心基石（无依赖）           │
                    │   Tag  Attribute  Modifier          │
                    └──────────┬──────────────────────────┘
                               │
                    ┌──────────▼──────────────────────────┐
                    │          聚合层（依赖基石）            │
                    │   Aggregator  GameplayContext        │
                    └──────────┬──────────────────────────┘
                               │
                    ┌──────────▼──────────────────────────┐
                    │          逻辑骨架（依赖基石+聚合）    │
                    │   Spec  Condition  Trigger  Event    │
                    └──────────┬──────────────────────────┘
                               │
                    ┌──────────▼──────────────────────────┐
                    │          行为表现（依赖全部）          │
                    │   Ability  Targeting  Execution      │
                    │   Effect  Stacking  Cue              │
                    └──────────┬──────────────────────────┘
                               │
                    ┌──────────▼──────────────────────────┐
                    │          运行时（跨领域编排）          │
                    │   Pipeline  Scheduler  Registry      │
                    │   Command   Replay                   │
                    └─────────────────────────────────────┘
```

---

## 七、Domains —— 业务子系统

> # 所有业务复杂度下沉到对应 Domain，是玩法迭代的唯一载体。
> # Domain 只能向下调用 Capabilities 能力，禁止反向依赖；
> # Domain 之间禁止直接引用，只能通过全局事件通信。
> # 原5层架构中 L3(model) 的数据模型归入各 Domain 的 components.rs；
> # 原5层架构中 L4(frame) 的玩法框架归入各 Domain 的 plugin.rs + integration.rs。

### 7.1 领域总览（15个核心业务域）

| # | 领域 | 职责 | 对标 |
|---|------|------|------|
| 1 | **Combat** | 战斗全流程编排、伤害结算、回合规则、胜负判定 | BG3 回合制 |
| 2 | **Spell** | 法术释放、法术位消耗、反应法术、施法组件、专注维持、法术豁免 | D&D 5e 法术 |
| 3 | **Reaction** | 机会攻击、援护格挡、反击、法术反应、回合外触发 | BG3 反应系统 |
| 4 | **Progression** | 经验等级、职业成长、子职、天赋解锁、属性成长、技能升级 | D&D 多职 |
| 5 | **Inventory** | 背包管理、物品使用、装备穿戴、物品交互、掉落规则 | BG3 背包 |
| 6 | **Quest** | 任务流程控制、条件判定、奖励发放、任务状态管理 | BG3 任务 |
| 7 | **Narrative** | 剧情对话、分支选项、条件对话、演出控制、阵营对话变体 | BG3 对话 |
| 8 | **Tactical** | 网格移动、站位管理、高地优势、掩体、背刺、夹击 | 铃兰战术 |
| 9 | **Terrain** | 毒池、冰面、灼烧地面、高地加成、掩体、地形交互 | BG3 地形 |
| 10 | **Faction** | 阵营声望、友好/敌对关系、势力影响、声望奖励 | BG3 阵营 |
| 11 | **Party** | 战前编队、队伍换人、小队羁绊、队伍全局buff、队伍资源共享 | 铃兰羁绊 |
| 12 | **CampRest** | 长休短休、资源恢复、营地NPC交互、营地商人、营地剧情、法术刷新 | BG3 营地 |
| 13 | **Crafting** | 装备锻造、词条附魔、武器改造、分解、材料合成 | BG3 制作 |
| 14 | **Economy** | 货币、商店买卖、物价波动、交易折扣、赃物交易 | BG3 商店 |
| 15 | **Summon** | 召唤物生命周期、幻象消失、召唤物AI、召唤物占用格子 | D&D 召唤 |

### 7.2 Domain 标准内部结构（强制统一）

```
domain_name/
├── plugin.rs          # 唯一对外入口，注册组件、系统、事件
├── components.rs      # 本系统专属 ECS 组件（含原 L3 model 的数据模型）
├── systems/           # 本系统业务系统（按子模块拆分）
│   ├── mod.rs
│   ├── xxx_system.rs
│   └── yyy_system.rs
├── events.rs          # 本系统对外发布的领域事件
├── error.rs           # 本系统专属错误枚举
├── rules/             # 纯业务规则（优先纯函数，无 ECS 依赖）
│   ├── formulas.rs    # 业务计算公式
│   └── rules.rs       # 玩法规则判定
└── integration.rs     # 集成层：唯一调用 Capabilities 能力的入口
                        # 含原 L4 frame 的玩法编排逻辑
```

**示例：Combat Domain**

```
domains/combat/
├── plugin.rs
├── components.rs           # CombatState, TurnOrder, CombatParticipant, Initiative
├── events.rs               # CombatStarted, TurnBegin, TurnEnd, CombatEnded
├── error.rs
├── integration.rs          # 对接 Ability/Effect/Attribute/Aggregator 等机制
├── systems/
│   ├── mod.rs
│   ├── turn_system.rs      # 回合阶段流转（先攻→行动→结束）
│   ├── initiative_system.rs # 先攻检定与排序
│   ├── damage_system.rs    # 伤害结算
│   ├── death_system.rs     # 单位死亡与尸体处理
│   ├── victory_system.rs   # 胜负判定
│   └── reaction_trigger_system.rs  # 反应触发检测
└── rules/
    ├── mod.rs
    ├── damage_formula.rs   # D&D 伤害公式：武器骰+属性调整+其他加值
    ├── critical_rules.rs   # 暴击规则（自然20暴击，暴击骰翻倍）
    ├── cover_rules.rs      # 掩体AC加成规则
    └── advantage_rules.rs  # 优势/劣势判定规则
```

### 7.3 各 Domain 详细结构

#### Combat（战斗）

```
domains/combat/
├── plugin.rs              # 战斗 Plugin，注册战斗系统与事件
├── components.rs           # CombatState, TurnOrder, CombatParticipant, Initiative
│                           # CombatState: 战斗状态机（Initiative→PlayerTurn→EnemyTurn→Resolution）
│                           # TurnOrder: 先攻排序队列
│                           # CombatParticipant: 战斗参与者标记与阵营
├── events.rs               # CombatStarted, TurnBegin, TurnEnd, CombatEnded
├── error.rs                # CombatError（无效目标/非战斗状态/回合已结束等）
├── integration.rs          # 对接 Ability/Effect/Attribute/Aggregator 等机制
├── systems/
│   ├── turn_system.rs      # 回合阶段流转（先攻→行动→结束）
│   ├── initiative_system.rs # 先攻检定与排序（D&D d20+敏捷调整值）
│   ├── damage_system.rs    # 伤害结算（调用 Execution + Aggregator）
│   ├── death_system.rs     # 单位死亡与尸体处理（触发死亡事件→移除战斗参与者）
│   ├── victory_system.rs   # 胜负判定（一方全灭→战斗结束）
│   └── reaction_trigger_system.rs  # 反应触发检测（离开威胁区→机会攻击）
└── rules/
    ├── damage_formula.rs   # D&D 伤害公式：武器骰+属性调整+其他加值
    ├── critical_rules.rs   # 暴击规则（自然20暴击，暴击骰翻倍）
    ├── cover_rules.rs      # 掩体AC加成规则（半掩+2/四分之三掩+5）
    └── advantage_rules.rs  # 优势/劣势判定规则（掷两个d20取高/低）
```

#### Spell（法术）

```
domains/spell/
├── plugin.rs
├── components.rs           # SpellSlots, Concentration, Spellbook, CastingComponent
│                           # SpellSlots: 法术位（1-9环），每环有上限与已用数
│                           # Concentration: 专注状态（维持的法术、专注来源Entity）
│                           # Spellbook: 已知/已准备法术列表
├── events.rs               # SpellCast, SpellResolved, ConcentrationBroken
├── error.rs
├── integration.rs
├── systems/
│   ├── cast_system.rs      # 法术释放流程（检查法术位→施法→消耗→生成Effect）
│   ├── slot_system.rs      # 法术位消耗与恢复（长休恢复全部）
│   ├── concentration_system.rs  # 专注维持与打断（受伤时DC=10/2+伤害一半）
│   ├── save_system.rs      # 法术豁免检定（DC=8+熟练+施法属性调整值）
│   └── reaction_spell_system.rs  # 反应法术（护盾术+5AC、法术反制环阶判定）
└── rules/
    ├── spell_level_rules.rs  # 法术环阶规则（戏法/1-9环）
    ├── concentration_rules.rs  # 专注检定DC规则（受伤/环境干扰）
    ├── save_dc_rules.rs     # 法术豁免DC计算（8+熟练+施法属性调整值）
    └── upcast_rules.rs     # 升环施法规则（3环火球用5环位施放→伤害+2d6）
```

#### Reaction（反应/援护）

```
domains/reaction/
├── plugin.rs
├── components.rs           # ReactionState, ReactionQueue, OpportunityAttack, CounterSpell
│                           # ReactionState: 本回合是否已使用反应（D&D每回合1次）
│                           # ReactionQueue: 待处理反应队列（多人同时触发时按先攻排序）
├── events.rs               # ReactionTriggered, ReactionExecuted, ReactionDeclined
├── error.rs
├── integration.rs
├── systems/
│   ├── opportunity_attack_system.rs  # 机会攻击（离开威胁区→近战攻击）
│   ├── counterspell_system.rs        # 法术反制（反应消耗法术位→打断对方施法）
│   ├── shield_system.rs              # 护盾反应（+5AC直到下回合开始）
│   ├── guardian_system.rs            # 援护格挡（替相邻友方承受攻击）
│   └── reaction_queue_system.rs      # 反应队列管理（按优先级弹出→执行→标记已用）
└── rules/
    ├── opportunity_rules.rs  # 机会攻击触发条件（离开威胁区/站立起身）
    ├── reaction_limit_rules.rs  # 每回合反应次数限制（D&D 5e: 1次/回合）
    └── counterspell_rules.rs  # 法术反制环阶判定（3环反制≤3环，+1环检定DC=10+环差×2）
```

#### Progression（成长养成）

```
domains/progression/
├── plugin.rs
├── components.rs           # Level, Experience, ClassLevels, TalentTree, SubclassChoice
│                           # ClassLevels: 多职等级映射（如战士5/法师3）
│                           # TalentTree: 天赋选择状态（已解锁/可选/锁定）
├── events.rs               # LevelUp, ClassGained, TalentUnlocked, SubclassChosen
├── error.rs
├── integration.rs
├── systems/
│   ├── exp_system.rs       # 经验获取与升级（D&D 5e 经验表/里程碑）
│   ├── class_system.rs     # 职业等级管理（多职规则：总等级=各职等级之和）
│   ├── talent_system.rs    # 天赋解锁与选择（满足前置条件→解锁→选择）
│   ├── subclass_system.rs  # 子职选择与特性（3级选择子职→获得子职特性）
│   └── asi_system.rs       # 属性提升（ASI: 4/8/12/16/19级各+2点属性）
└── rules/
    ├── level_table_rules.rs  # 等级经验表（1-20级所需XP）
    ├── multiclass_rules.rs   # 多职规则（D&D 5e 多职：熟练加值按总等级计算）
    ├── asi_rules.rs          # 属性提升时机与规则（4级/8级/... 或取代为专长）
    └── proficiency_rules.rs  # 熟练加值规则（+2@1-4, +3@5-8, +4@9-12, +5@13-16, +6@17-20）
```

#### Inventory（背包物品）

```
domains/inventory/
├── plugin.rs
├── components.rs           # Inventory, EquipmentSlots, ItemInstance
│                           # EquipmentSlots: 装备槽位（主手/副手/头盔/铠甲/靴子/饰品×2）
│                           # ItemInstance: 物品实例（含附魔/耐久/自定义属性）
├── events.rs               # ItemAcquired, ItemUsed, EquipmentChanged
├── error.rs
├── integration.rs
├── systems/
│   ├── inventory_system.rs  # 背包管理（增删查改，堆叠合并）
│   ├── equip_system.rs     # 装备穿戴/卸下（检查Condition→穿戴→应用Modifier）
│   ├── use_system.rs       # 物品使用（消耗品→生成Effect，卷轴→施放法术）
│   └── loot_system.rs     # 掉落与拾取（LootTable随机→生成ItemInstance→入背包）
└── rules/
    ├── slot_rules.rs       # 装备槽位规则（双手武器占主+副手，戒指限2个）
    ├── weight_rules.rs     # 负重规则（力量×15=携带上限，超重降速）
    └── rarity_rules.rs     # 稀有度规则（普通/非凡/稀有/史诗/传说）
```

#### Quest（任务）

```
domains/quest/
├── plugin.rs
├── components.rs           # QuestLog, QuestState, ObjectiveProgress
│                           # QuestState: 任务状态（Available/Active/Completed/Failed/Expired）
│                           # ObjectiveProgress: 目标进度（击杀3/5哥布林）
├── events.rs               # QuestAccepted, ObjectiveCompleted, QuestTurnedIn
├── error.rs
├── integration.rs
├── systems/
│   ├── quest_system.rs     # 任务生命周期管理（接受→追踪→完成→交付）
│   ├── objective_system.rs # 目标进度追踪（监听Event→更新进度→检查完成）
│   └── reward_system.rs    # 奖励发放（经验/物品/声望/解锁）
└── rules/
    ├── quest_prereq_rules.rs  # 任务前置条件（等级/阵营/完成某任务）
    └── reward_rules.rs        # 奖励规则（经验按等级缩放）
```

#### Narrative（叙事/对话）

```
domains/narrative/
├── plugin.rs
├── components.rs           # DialogueState, StoryFlag, CutsceneState
│                           # DialogueState: 对话状态（当前节点/可选分支/已选分支）
│                           # StoryFlag: 故事标记（二值/枚举，如"救了村民"/"杀了商人"）
├── events.rs               # DialogueStarted, ChoiceMade, StoryFlagSet
├── error.rs
├── integration.rs
├── systems/
│   ├── dialogue_system.rs  # 对话流程控制（节点跳转/分支选择/条件过滤）
│   ├── choice_system.rs    # 分支选项评估（检查Condition→过滤可选分支→展示）
│   ├── cutscene_system.rs  # 演出控制（镜头/动画/音效/等待时间）
│   └── flag_system.rs     # 故事标记管理（设置/检查/持久化）
└── rules/
    ├── condition_rules.rs  # 对话条件判定（阵营/任务状态/物品持有/属性值）
    └── branch_rules.rs     # 分支走向规则（条件满足→跳转对应节点）
```

#### Tactical（战术/网格）

```
domains/tactical/
├── plugin.rs
├── components.rs           # GridPosition, MovementPoints, Facing, FlankingState
│                           # GridPosition: 网格坐标（行/列/层高）
│                           # MovementPoints: 行动力（当前/最大/已消耗）
│                           # Facing: 朝向（六向/八向，影响背刺判定）
├── events.rs               # UnitMoved, PositionChanged, FlankingDetected
├── error.rs
├── integration.rs
├── systems/
│   ├── movement_system.rs  # 移动与路径（消耗行动力→A*寻路→更新位置）
│   ├── flanking_system.rs  # 夹击检测（两个友方在目标对侧→优势攻击）
│   ├── highground_system.rs # 高地优势（高度差≥2→远程优势+2命中）
│   ├── backstab_system.rs  # 背刺判定（攻击者在目标背后→额外伤害骰）
│   └── cover_system.rs    # 掩体检测（目标与攻击者间有障碍物→AC加成）
└── rules/
    ├── movement_rules.rs   # 移动规则（行动力/距离/困难地形消耗翻倍）
    ├── flanking_rules.rs   # 夹击规则（铃兰之剑：两个友方夹击→无视防御）
    ├── highground_rules.rs # 高地规则（BG3：高度差≥2→远程优势）
    └── backstab_rules.rs   # 背刺规则（铃兰之剑：背后攻击→暴击率+25%）
```

#### Terrain（地形）

```
domains/terrain/
├── plugin.rs
├── components.rs           # TileProperties, TerrainEffect, HazardZone
│                           # TileProperties: 瓦片属性（类型/高度/通行性/遮蔽度）
│                           # TerrainEffect: 地形效果（灼烧/冰滑/毒池/油面）
│                           # HazardZone: 陷阱区域（触发条件/伤害/范围）
├── events.rs               # TerrainEntered, TerrainEffectTriggered
├── error.rs
├── integration.rs
├── systems/
│   ├── terrain_effect_system.rs  # 地形效果施加（进入格子→检查效果→应用Effect）
│   ├── hazard_system.rs          # 陷阱/毒池/灼烧地面（每回合Tick→触发Execution）
│   ├── surface_system.rs         # 冰面/油面/水面（冰面→移动消耗翻倍，油面→火焰扩散）
│   └── interaction_system.rs     # 地形交互（点火/冻结/推落高地）
└── rules/
    ├── damage_rules.rs     # 地形伤害规则（灼烧地面1d6/回合，毒池1d4/回合）
    ├── movement_rules.rs   # 困难地形移动惩罚（消耗2倍行动力）
    └── interaction_rules.rs  # 地形交互规则（火焰+油面→范围爆炸2d8）
```

#### Faction（阵营关系）

```
domains/faction/
├── plugin.rs
├── components.rs           # FactionMembership, Reputation, FactionRelation
│                           # FactionMembership: 角色所属阵营列表
│                           # Reputation: 声望值（-100~+100，分段：仇恨/敌对/中立/友好/崇敬）
│                           # FactionRelation: 阵营间关系（盟友/中立/敌对/战争）
├── events.rs               # ReputationChanged, FactionHostile, FactionAllied
├── error.rs
├── integration.rs
├── systems/
│   ├── reputation_system.rs  # 声望增减（击杀敌对阵营成员→+声望，杀友方→-声望）
│   ├── relation_system.rs    # 关系演变（声望阈值→关系变化→触发事件）
│   └── reward_system.rs      # 声望奖励（友好→解锁商店折扣，崇敬→专属装备）
└── rules/
    ├── reputation_rules.rs  # 声望阈值与等级（0中立，+50友好，+100崇敬）
    └── relation_rules.rs    # 友好/敌对/中立判定（影响NPC攻击/商店价格/对话选项）
```

#### Party（队伍）

```
domains/party/
├── plugin.rs
├── components.rs           # PartyRoster, PartyBuff, BondState
│                           # PartyRoster: 队伍成员列表（战斗中/预备）
│                           # BondState: 羁绊状态（已激活羁绊/羁绊等级）
├── events.rs               # MemberJoined, MemberSwapped, BondActivated
├── error.rs
├── integration.rs
├── systems/
│   ├── roster_system.rs    # 队伍成员管理（加入/移除/排序）
│   ├── swap_system.rs      # 战斗中换人（消耗行动力→替换战斗成员）
│   ├── bond_system.rs      # 小队羁绊激活（特定角色组合→触发羁绊Effect）
│   └── party_buff_system.rs # 队伍全局buff（如"全员+1AC"的营地休息效果）
└── rules/
    ├── bond_rules.rs       # 羁绊触发条件与效果（铃兰：同阵营3人→全属性+10%）
    ├── swap_rules.rs       # 换人规则（战斗中换人消耗1行动力，每回合限1次）
    └── party_size_rules.rs # 队伍人数限制（战斗4人，预备不限）
```

#### CampRest（营地休息）

```
domains/camp_rest/
├── plugin.rs
├── components.rs           # RestState, CampNPC, CampEvent
│                           # RestState: 休息状态（短休/长休进行中）
│                           # CampNPC: 营地NPC（商人/剧情角色/队友对话）
│                           # CampEvent: 营地事件（篝火对话/营地攻击/特殊剧情）
├── events.rs               # LongRestStarted, ShortRestCompleted, SpellSlotsRestored
├── error.rs
├── integration.rs
├── systems/
│   ├── rest_system.rs      # 长休/短休流程（长休8h→全恢复，短休1h→恢复HitDice）
│   ├── recovery_system.rs  # 资源恢复（HP→HitDice，法术位→全恢复，能力→重置）
│   ├── camp_npc_system.rs  # 营地NPC交互（买卖/对话/任务）
│   └── camp_event_system.rs # 营地剧情触发（好感度/阵营/任务状态条件）
└── rules/
    ├── rest_rules.rs       # 长休/短休规则（D&D 5e：长休恢复全部HP+法术位，短休恢复HitDice）
    ├── recovery_rules.rs   # 恢复量规则（长休HP=全部，短休HP=HitDice×等级）
    └── camp_rules.rs       # 营地规则（长休前必须安全区域，短休可在战斗间）
```

#### Crafting（制作）

```
domains/crafting/
├── plugin.rs
├── components.rs           # RecipeBook, CraftingStation, EnchantmentSlot
│                           # RecipeBook: 已解锁配方列表
│                           # CraftingStation: 制作台类型（锻造台/附魔台/炼金台）
│                           # EnchantmentSlot: 附魔槽位（武器最多3个词条）
├── events.rs               # ItemCrafted, EnchantmentApplied, ItemDisenchanted
├── error.rs
├── integration.rs
├── systems/
│   ├── craft_system.rs     # 制作流程（检查材料→消耗→生成ItemInstance）
│   ├── enchant_system.rs   # 附魔/词条（消耗材料→添加Modifier到装备）
│   ├── upgrade_system.rs   # 武器改造（+1→+2→+3，每级+1命中/伤害）
│   └── disenchant_system.rs # 分解（销毁物品→获得材料，稀有度越高材料越多）
└── rules/
    ├── recipe_rules.rs     # 配方规则（需要对应制作台+材料+熟练度）
    ├── enchant_rules.rs    # 附魔规则（同类型词条互斥，最多3个槽位）
    └── material_rules.rs   # 材料需求规则（稀有装备需要稀有材料）
```

#### Economy（经济）

```
domains/economy/
├── plugin.rs
├── components.rs           # Wallet, ShopInventory, PriceModifier
│                           # Wallet: 钱包（金币/银币/铜币，1金=10银=100铜）
│                           # ShopInventory: 商店库存（物品列表+库存量+补货周期）
│                           # PriceModifier: 价格修正（声望折扣/赃物惩罚/供需波动）
├── events.rs               # TransactionCompleted, PriceChanged
├── error.rs
├── integration.rs
├── systems/
│   ├── shop_system.rs      # 商店买卖（检查钱包→交易→更新库存→触发事件）
│   ├── price_system.rs     # 物价计算（基础价×声望折扣×供需波动×赃物惩罚）
│   └── trade_system.rs     # 交易流程（选择物品→确认价格→扣款→获得物品）
└── rules/
    ├── price_rules.rs      # 定价规则（基础价由物品稀有度决定）
    ├── discount_rules.rs   # 折扣规则（友好声望-10%，崇敬-25%）
    └── stolen_rules.rs     # 赃物交易规则（赃物售价-50%，部分商人拒收）
```

#### Summon（召唤）

```
domains/summon/
├── plugin.rs
├── components.rs           # SummonBond, SummonDuration, SummonAI
│                           # SummonBond: 召唤者-召唤物绑定（召唤者死亡→召唤物消失）
│                           # SummonDuration: 召唤持续时间（专注维持/固定回合/永久）
│                           # SummonAI: 召唤物AI模式（跟随施法者指令/自主攻击）
├── events.rs               # SummonCreated, SummonExpired, SummonDismissed
├── error.rs
├── integration.rs
├── systems/
│   ├── summon_create_system.rs  # 召唤物生成（创建Entity→复制模板→绑定召唤者）
│   ├── summon_expire_system.rs  # 召唤物到期/消失（专注打断→立即消失）
│   ├── summon_control_system.rs # 召唤物控制（施法者消耗附赠动作下达指令）
│   └── summon_death_system.rs   # 召唤者死亡→召唤物消失（监听DeathEvent→级联移除）
└── rules/
    ├── concentration_rules.rs  # 专注召唤规则（同时只能维持1个专注召唤）
    ├── duration_rules.rs       # 召唤持续时间（专注维持=施法者专注期间，固定=N回合）
    └── slot_rules.rs          # 召唤物占位规则（大型召唤物占2×2格子）
```

---

## 八、Mod API

> # 唯一对外暴露的核心接口，Gateway 模式，保证 Mod 兼容性
> # 每个 Gateway 只暴露查询和受控修改操作，禁止直接操作 ECS World

```
src/core/mod_api/
├── mod.rs
├── core_facade.rs            # 核心门面，统一入口（Mod 通过此入口访问所有 Gateway）
├── combat_gateway.rs         # 战斗 API（查询战斗状态/触发反应/强制结束战斗）
├── character_gateway.rs     # 角色 API（查询属性/标签/修改 Spec 等级）
├── spell_gateway.rs         # 法术 API（查询法术位/施放法术/打断专注）
├── quest_gateway.rs         # 任务 API（接受/完成/查询任务状态）
├── party_gateway.rs         # 队伍 API（换人/查询羁绊/修改队伍配置）
├── camp_gateway.rs          # 营地 API（触发长休/短休/查询营地事件）
├── summon_gateway.rs        # 召唤 API（创建/移除召唤物/查询召唤状态）
├── terrain_gateway.rs       # 地形 API（修改地形/查询地形效果/触发地形交互）
├── craft_gateway.rs         # 制作 API（注册配方/执行制作/查询材料）
├── economy_gateway.rs       # 经济 API（修改物价/查询钱包/执行交易）
├── inventory_gateway.rs     # 物品 API（添加/移除物品/装备穿戴）
├── faction_gateway.rs       # 阵营 API（修改声望/查询关系/注册阵营）
├── progression_gateway.rs   # 成长 API（给予经验/升级/解锁天赋）
└── narrative_gateway.rs     # 叙事 API（设置故事标记/触发对话/查询剧情状态）
```

---

## 九、Infra 层（L2）

> # 内部分三层，严格单向依赖，禁止反向调用
> # foundation → services → presentation（单向）
> # foundation：被所有 infra 模块依赖的底层技术工具
> # services：通用服务，依赖 foundation，被 presentation 和上层调用
> # presentation：表现层，依赖 services + foundation，是 ECS 与引擎的桥梁

```
src/infra/
├── mod.rs
├── infra_plugin.rs             # 基础设施总 Plugin，注册所有服务与表现层
│
├── foundation/                 # 基础技术层（被所有 infra 模块依赖）
│   # 职责：提供序列化/资源加载/线程/内存等底层技术能力
│   # 禁止：包含任何业务逻辑、依赖 Core 层
│   ├── mod.rs
│   ├── ecs/                      # ECS 通用扩展
│   │   # 功能：对 Bevy ECS 的 Component/Query 提供项目级扩展方法
│   │   ├── mod.rs
│   │   ├── component_ext.rs      # 组件扩展方法（如批量插入/删除）
│   │   └── query_ext.rs          # 查询扩展方法（如安全遍历/过滤）
│   ├── serialization/            # 序列化框架
│   │   # 功能：统一 RON/JSON 序列化配置，确保全项目序列化行为一致
│   │   ├── mod.rs
│   │   ├── ron_ext.rs             # RON 格式扩展（游戏配置首选格式）
│   │   └── json_ext.rs            # JSON 格式扩展（Mod 互操作格式）
│   ├── asset/                    # 资源加载框架
│   │   # 功能：统一资源加载入口，支持热重载与异步加载
│   │   ├── mod.rs
│   │   ├── asset_loader.rs        # 统一资源加载器（支持 RON/JSON/图片/音频）
│   │   └── hot_reload_base.rs     # 热重载基础（开发期自动检测文件变更）
│   ├── threading/                # 线程池与任务调度
│   │   # 功能：封装 Bevy TaskPool，提供项目级异步任务调度
│   │   ├── mod.rs
│   │   └── task_pool.rs           # Bevy TaskPool 封装
│   └── memory/                   # 内存管理
│       # 功能：对象池与内存复用，减少高频 Entity 创建/销毁的分配开销
│       ├── mod.rs
│       └── object_pool.rs         # 对象池（频繁创建/销毁的 Entity 复用）
│
├── services/                   # 通用服务层
│   # 职责：提供日志/存档/寻路/输入/分析等通用服务
│   # 依赖：foundation 层
│   # 禁止：包含任何业务逻辑、直接操作 ECS World
│   ├── mod.rs
│   ├── logging/                   # 日志系统（详见第二十一节）
│   │   # 功能：统一日志管理，异步写入，级别过滤，结构化输出
│   │   ├── mod.rs
│   │   ├── log_manager.rs          # 日志管理器（级别过滤/格式化/输出路由）
│   │   ├── log_formatter.rs        # 日志格式化器（统一输出格式）
│   │   ├── log_writer.rs           # 日志写入器（控制台/文件/远程，异步通道写入）
│   │   └── log_config.rs           # 日志配置（级别/输出目标/滚动策略，RON 配置文件加载）
│   ├── save/                     # 存档系统
│   │   # 功能：存档序列化/反序列化/版本迁移/完整性校验
│   │   # 设计思路：版本迁移采用「当前版本→目标版本」链式转换，而非 N² 转换矩阵
│   │   ├── mod.rs
│   │   ├── save_loader.rs          # 存档加载（反序列化→校验→注册）
│   │   ├── save_migration.rs       # 存档版本迁移（旧版本→新版本自动转换）
│   │   └── save_error.rs           # 存档错误枚举
│   ├── localization/             # 多语言框架
│   │   # 功能：按 Key 查询当前语言文本，支持运行时切换语言
│   │   ├── mod.rs
│   │   └── locale_manager.rs       # 多语言管理器（按 Key 查询当前语言文本）
│   ├── physics/                  # 物理碰撞
│   │   # 功能：SRPG 简化为格子碰撞+射线检测，不做完整物理模拟
│   │   ├── mod.rs
│   │   └── collision.rs           # 碰撞检测（SRPG 简化为格子碰撞+射线检测）
│   ├── pathfinding/              # 寻路算法实现
│   │   # 功能：A* 寻路，支持六角/四角网格+地形代价+障碍物
│   │   # 设计思路：寻路结果是 Domain 层的 MovementPoints 消耗依据
│   │   ├── mod.rs
│   │   └── a_star.rs              # A* 寻路（支持六角/四角网格+地形代价）
│   ├── navmesh/                  # 导航网格
│   │   # 功能：非战斗场景的导航网格构建（城镇/营地等开放区域）
│   │   ├── mod.rs
│   │   └── navmesh_builder.rs     # 导航网格构建（非战斗场景使用）
│   ├── input/                    # 输入处理
│   │   # 功能：按键/手柄→游戏动作映射，UI 操作→游戏命令转换
│   │   # 设计思路：输入层只产生 Command，不直接修改 ECS 状态
│   │   ├── mod.rs
│   │   ├── input_mapping.rs        # 输入映射（按键/手柄→游戏动作）
│   │   └── command_converter.rs    # 输入→命令转换（UI操作→游戏命令）
│   ├── analytics/                # 数据埋点与遥测
│   │   # 功能：采集战斗时长/技能使用频率/死亡原因等指标，供数值平衡分析
│   │   ├── mod.rs
│   │   └── metrics.rs              # 指标采集（战斗时长/技能使用频率/死亡原因）
│   ├── hot_reload/               # 内容热重载
│   │   # 功能：开发期自动检测 assets/ 目录变化并触发重载
│   │   # 注意：仅 dev-tools feature 启用，永不进入发布构建
│   │   ├── mod.rs
│   │   └── file_watcher.rs         # 文件变更监听（检测 assets/ 目录变化→触发重载）
│   ├── encryption/               # 加密能力
│   │   # 功能：存档加密/资源完整性校验，防止存档篡改
│   │   ├── mod.rs
│   │   └── crypto.rs               # 加密/解密（存档加密/资源完整性校验）
│   ├── anti_cheat/               # 反作弊
│   │   # 功能：存档/配置文件哈希校验，检测非法修改
│   │   ├── mod.rs
│   │   └── integrity_check.rs     # 完整性校验（存档/配置文件哈希校验）
│   ├── networking/               # 网络通信
│   │   # 功能：预留网络协议层，单机暂不实现
│   │   ├── mod.rs
│   │   └── net_protocol.rs         # 网络协议（预留，单机暂不实现）
│   ├── cloud/                    # 云服务对接
│   │   # 功能：预留云存档同步，单机暂不实现
│   │   ├── mod.rs
│   │   └── cloud_save.rs           # 云存档同步（预留）
│   └── mod_loader/               # Mod 加载器
│       # 功能：Mod 扫描/验证/排序/沙箱隔离
│       # 设计思路：沙箱限制 Mod 的 API 调用范围和执行时间，防止恶意 Mod
│       ├── mod.rs
│       ├── mod_scanner.rs          # Mod 扫描（发现/验证/排序）
│       └── sandbox.rs              # Mod 沙箱（隔离运行/限制API调用）
│
└── presentation/               # 表现层
    # 职责：渲染/音频/动画/粒子/UI，是 ECS 与引擎的桥梁
    # 依赖：services + foundation 层
    # 禁止：包含任何业务逻辑、直接修改 Core 层数据
    # 设计思路：表现层通过订阅 Core 层 Event（如 CueTriggered/AttributeChanged）驱动视觉更新
    ├── mod.rs
    ├── rendering/                # 渲染管线
    │   # 功能：材质/Shader 管理，自定义渲染管线
    │   ├── mod.rs
    │   ├── materials.rs            # 材质管理
    │   └── shaders.rs             # Shader 管理
    ├── ui/                       # UI 系统
    │   # 功能：页面视图/通用组件/路由/数据绑定/HUD
    │   # 设计思路：数据绑定（binding.rs）将 ECS Component 映射到 UI 显示，
    │   #          UI 操作通过 Command 修改 ECS 状态，不直接操作 World
    │   ├── mod.rs
    │   ├── view/                   # 页面视图（战斗UI/背包UI/对话UI）
    │   ├── widget/                 # 通用组件（按钮/列表/进度条/弹窗）
    │   ├── navigation.rs           # 页面路由（状态驱动页面切换）
    │   ├── binding.rs              # 数据绑定（ECS Component → UI 显示）
    │   └── hud.rs                  # 战斗 HUD（血条/行动条/状态图标）
    ├── audio/                      # 音频系统
    │   # 功能：3D空间音效/2D UI音效/BGM管理
    │   # 设计思路：订阅 CueTriggered 事件播放对应音效
    │   ├── mod.rs
    │   ├── sfx_player.rs            # 音效播放（3D空间音效/2D UI音效）
    │   └── bgm_manager.rs          # 背景音乐管理（淡入淡出/战斗切换）
    ├── animation/                # 动画系统
    │   # 功能：状态机驱动动画切换，接收 Cue 事件触发动画
    │   ├── mod.rs
    │   └── anim_controller.rs      # 动画控制器（状态机驱动动画切换）
    ├── particles/                # 粒子特效
    │   # 功能：接收 Cue 事件→播放对应特效
    │   # 设计思路：VFX 与逻辑解耦，Cue 是唯一桥梁
    │   ├── mod.rs
    │   └── vfx_player.rs            # 特效播放器（接收 Cue 事件→播放对应特效）
    └── platform/                 # 平台适配
        # 功能：Steam/Epic SDK 集成（成就/云存档/Workshop）
        # 设计思路：平台差异通过 feature flag 隔离，不污染核心代码
        ├── mod.rs
        ├── steam.rs                 # Steam SDK 集成（成就/云存档/Workshop）
        └── epic.rs                  # Epic SDK 集成（预留）
```

---

## 十、App 层（横切1：启动装配）

> # Composition Root，唯一知道所有层存在的模块
> # 职责：组装 Plugin、配置 Schedule、管理全局状态
> # 禁止：包含任何业务逻辑

```
src/app/
├── mod.rs
├── app_plugin.rs               # 主Plugin，汇总注册所有子Plugin（唯一知道全部模块的地方）
├── game_app.rs                 # 游戏模式启动入口（注册游戏专用Plugin/禁用编辑器Plugin）
├── editor_app.rs               # 编辑器模式启动入口（注册编辑器Plugin/启用dev-tools）
├── server_app.rs               # 服务器模式（预留，无渲染纯逻辑）
├── headless_app.rs             # 无头模式启动入口（战斗模拟/自动化测试）
│
├── state/                      # 全局状态机
│   ├── mod.rs
│   ├── app_state.rs            # AppState 主状态（Loading→MainMenu→Game→Editor）
│   └── game_state.rs           # 游戏内子状态（Exploration→Combat→Dialogue→Camp→Menu）
│
├── bootstrap/                  # 启动流程
│   ├── mod.rs
│   ├── load_core.rs            # 核心资源加载（Tag树/Attribute定义/注册中心初始化）
│   └── init_content.rs         # 内容初始化（加载配置→校验→注册到Core）
│
└── schedule/                   # 调度编排
    ├── mod.rs
    ├── schedules.rs            # 自定义 Schedule 定义（PreUpdate/CombatTick/PostCombat）
    └── sets.rs                 # SystemSet 定义与顺序约束（确保系统执行顺序正确）
```

---

## 十一、Content 层（横切2：内容桥接）

> # 只做「加载 → 校验 → 注册」，不含业务规则
> # 是数据驱动的核心，DSL + 配置 + gameplay 粘合
> # 所有 Def 类型定义在此层，Spec/Instance 在 Core 层
> # 设计思路：Content 层是「数据→逻辑」的桥梁，将 assets/ 中的 RON/JSON 配置
> #          转换为 Core 层可用的 Def 结构，再由 Core 层生成 Spec/Instance

```
src/content/
├── mod.rs
├── content_plugin.rs           # 内容层总 Plugin，注册所有加载器与校验器
├── mod_support/                # Mod 内容加载与覆盖
│   # 功能：扫描 Mod 目录，合并/覆盖基础配置
│   # 设计思路：Mod 配置优先级高于基础配置，后加载的 Mod 覆盖先加载的
│   ├── mod.rs
│   └── mod_content_loader.rs   # Mod 内容加载器（扫描Mod目录→合并/覆盖基础配置）
│
├── schema/                     # 配置数据结构定义（Def 类型）
│   # 功能：定义所有从配置文件反序列化的数据结构
│   # 设计思路：Def 是「模板」，Spec 是「配置」，Instance 是「运行时」
│   #          Def 在此层定义，Spec/Instance 在 Core 层
│   ├── mod.rs
│   ├── attribute_def.rs        # 属性定义（ID/基础值/分类/上限下限）
│   ├── effect_def.rs           # 效果定义（类型/持续时间/修改器列表/标签需求）
│   ├── ability_def.rs          # 技能定义（类型/消耗/冷却/触发条件/目标选择）
│   ├── modifier_def.rs        # 修改器定义（操作类型/属性ID/缩放值/优先级）
│   ├── tag_def.rs              # 标签定义（ID/父标签/描述）
│   ├── trigger_def.rs          # 触发器定义（类型/条件/关联技能）
│   └── condition_def.rs        # 条件定义（类型/参数/逻辑组合）
│
├── attributes/                   # 属性配置加载
│   # 功能：从 assets/data/attributes/ 加载 RON 配置→反序列化为 AttributeDef→注册到 Core
│   └── mod.rs
├── tags/                         # 标签配置加载
│   # 功能：从 assets/data/tags/ 加载标签树配置→构建 Tag 层级关系→注册到 Core
│   └── mod.rs
├── modifiers/                    # 修改器配置加载
│   └── mod.rs
├── effects/                      # 效果配置加载
│   └── mod.rs
├── abilities/                    # 技能配置加载
│   └── mod.rs
├── triggers/                     # 触发器配置加载
│   └── mod.rs
│
├── characters/                   # 角色模板配置
│   # 功能：角色模板定义（基础属性/标签/技能列表/装备槽位）
│   └── mod.rs
├── enemies/                      # 敌人模板配置
│   └── mod.rs
├── classes/                      # 职业配置
│   └── mod.rs
├── factions/                     # 势力配置
│   └── mod.rs
│
├── maps/                         # 地图配置
│   └── mod.rs
├── encounters/                   # 遭遇配置
│   └── mod.rs
│
├── items/                        # 物品配置
│   └── mod.rs
├── equipments/                   # 装备配置
│   └── mod.rs
├── shops/                        # 商店配置
│   └── mod.rs
├── loot_tables/                  # 战利品配置
│   └── mod.rs
├── progression/                  # 进展配置
│   └── mod.rs
│
├── quests/                       # 任务配置
│   └── mod.rs
├── stories/                      # 故事配置
│   └── mod.rs
├── dialogues/                    # 对话配置
│   └── mod.rs
│
├── localization/                 # 多语言文本数据加载
│   └── mod.rs
├── balance/                      # 全局数值平衡配置
│   └── mod.rs
├── migration/                    # 配置版本迁移（旧版本配置→新版本自动转换）
│   └── mod.rs
└── validation/                   # 内容校验规则
    ├── mod.rs
    ├── reference_check.rs        # 引用合法性校验（Effect引用的Tag是否存在）
    └── value_range.rs            # 数值范围校验（属性值是否在合理区间）
```

---

## 十二、Tools 层（横切3：开发工具）

> # `#[cfg(feature = "dev-tools")]` 控制，永不进入发布构建
> # 所有工具通过 headless 模式或编辑器模式运行
> # 设计思路：开发工具是长期项目的「倍增器」，10-20年维护中工具投入的回报远超成本

```
src/tools/
├── mod.rs                      # [cfg(feature = "dev-tools")]
├── tools_plugin.rs
│
├── replay_viewer/              # 回放查看器（可视化回放命令流）
│   # 功能：加载回放文件→逐步回放→可视化每帧命令
│   └── mod.rs
├── battle_simulator/           # 战斗模拟器（数值平衡核心工具）
│   # 功能：无渲染纯逻辑战斗模拟，蒙特卡洛采样验证数值平衡
│   # 设计思路：直接复用 Core 层 Capabilities + Domains，通过 headless 模式运行
│   ├── mod.rs
│   ├── sim_runner.rs           # 模拟运行器（无渲染，纯逻辑）
│   ├── sim_config.rs           # 模拟配置（队伍/地图/规则）
│   ├── sim_result.rs           # 模拟结果（胜率/回合数/DPS分布）
│   └── batch_runner.rs         # 批量模拟（蒙特卡洛采样）
├── balance_analyzer/           # 数值平衡分析器
│   # 功能：采集模拟数据→方差分析→识别过强/过弱→生成报告
│   ├── mod.rs
│   ├── stat_collector.rs       # 统计数据采集
│   ├── variance_analyzer.rs    # 方差分析（识别过强/过弱）
│   └── report_generator.rs     # 报告生成
├── content_validator/          # 内容合法性校验器
│   # 功能：批量校验配置数据（引用完整性/数值范围/标签存在性）
│   └── mod.rs
├── localization_tool/          # 多语言工具（缺失翻译检测/批量导入导出）
│   └── mod.rs
├── save_editor/                # 存档编辑器
│   # 功能：可视化编辑存档数据（调试用，仅 dev-tools）
│   └── mod.rs
├── schema_generator/           # 配置 Schema 生成器（导出 JSON Schema 供编辑器使用）
│   └── mod.rs
├── graph_viewer/               # 效果/依赖管线可视化
│   # 功能：可视化 Ability→Effect→Modifier→Attribute 依赖图
│   └── mod.rs
├── profiling_tool/             # 性能剖析工具
│   # 功能：帧时间分析/系统耗时排序/内存占用追踪
│   └── mod.rs
├── ai_debugger/                # AI 决策调试器（可视化决策树/权重）
│   └── mod.rs
├── map_editor/                 # 地图编辑器
│   # 功能：可视化编辑地图（瓦片/高度/地形/遭遇点）
│   └── mod.rs
├── ability_editor/             # 技能编辑器（可视化编辑 AbilityDef）
│   └── mod.rs
├── effect_editor/              # 效果编辑器（可视化编辑 EffectDef）
│   └── mod.rs
├── quest_editor/               # 任务编辑器
│   └── mod.rs
├── pipeline_inspector/         # 执行管线检查器（可视化 Ability→Effect→Modifier 链路）
│   # 功能：实时追踪技能激活→效果施加→修改器应用的全链路
│   └── mod.rs
├── log_analyzer/               # 日志分析器（详见第二十一节）
│   # 功能：过滤/聚合/对比/报告，是长期项目排查问题的核心工具
│   ├── mod.rs
│   ├── log_filter.rs           # 日志过滤（按模块/级别/时间范围）
│   ├── log_aggregator.rs       # 日志聚合统计（如"过去1小时所有伤害事件平均值"）
│   └── log_diff.rs             # 回放日志对比（两次运行差异检测）
├── data_browser/               # 数据查询器
│   # 功能：查询 ECS World 中的 Entity/Component/Resource 状态
│   └── mod.rs
├── replay_diff/                # 回放对比
│   # 功能：对比两次回放差异，检测非确定性
│   └── mod.rs
├── migration_tool/             # 数据迁移
│   # 功能：旧版本存档/配置→新版本自动转换
│   └── mod.rs
├── dependency_checker/         # 架构检查（检测违反依赖规则的模块引用）
│   # 功能：静态分析 use 语句，检测 Domain 间直接引用等架构违规
│   └── mod.rs
└── test_runner/                # 自动测试入口
    └── mod.rs
```

---

## 十三、Modding 层（横切4：Mod 扩展）

> # 跨层聚合，暴露稳定的 Modding API
> # Mod 只能通过 mod_api/ 的 Gateway 访问核心功能，禁止直接操作 ECS World

```
src/modding/
├── api/                        # Mod 开发 API
│   ├── mod.rs
│   ├── mod_trait.rs              # Mod 生命周期 Trait（on_load/on_unload/on_update）
│   ├── hook_points.rs            # 扩展点定义（如"战斗开始前""伤害计算后"）
│   └── event_api.rs              # Mod 事件 API（订阅/发布领域事件）
├── registry/                   # Mod 注册中心
│   ├── mod.rs
│   └── mod_registry.rs           # Mod 注册表（ID/版本/依赖/加载顺序）
├── loader/                     # Mod 加载器
│   ├── mod.rs
│   └── mod_loader.rs             # Mod 加载流程（扫描→校验→排序→加载→初始化）
├── sandbox/                    # Mod 沙箱
│   ├── mod.rs
│   └── mod_sandbox.rs            # Mod 运行沙箱（限制API调用/资源访问/执行时间）
├── compatibility/              # Mod 兼容性管理
│   ├── mod.rs
│   ├── version_check.rs          # 版本兼容性检查（Mod 声明兼容的游戏版本号）
│   └── conflict_resolver.rs      # 冲突解决（多个Mod修改同一配置时的合并策略）
├── documentation/              # Mod 开发文档生成
│   └── mod.rs
└── examples/                   # Mod 示例
    └── mod.rs
```

---

## 十四、测试架构

### 核心原则

> **测试跟领域走（Feature First），但不写在源码文件内部。**

- 🟥 禁止 `#[cfg(test)] mod tests` 内联测试（对 AI 上下文污染严重）
- 🟥 禁止将所有测试平铺到根 `tests/unit/`（后期变成大杂烩）
- 🟩 测试与被测领域同目录放置，形成 Feature Folder 结构
- 🟩 根 `tests/` 仅保留跨领域测试（战斗流程、存档、回归、E2E）

### 领域内聚测试结构（四层）

每个领域/能力模块内部自包含测试：

```
<domain>/
├── components/
├── systems/
├── events/
├── services/
├── tests/
│   ├── unit/                    # 单元测试：验证领域纯函数、核心规则
│   │   ├── hp_spec.rs
│   │   ├── level_spec.rs
│   │   └── ...
│   ├── integration/             # 集成测试：验证领域内多组件协作
│   │   ├── character_skill_spec.rs
│   │   ├── character_buff_spec.rs
│   │   └── ...
│   ├── invariant/               # 不变量测试：验证领域不变量（最重要）
│   │   ├── tag_invariant_spec.rs
│   │   ├── buff_invariant_spec.rs
│   │   ├── effect_invariant_spec.rs
│   │   ├── hp_invariant_spec.rs
│   │   └── ...
│   └── fixtures/                # 测试数据
│       ├── levelup_character.ron
│       └── ...
```

### 四层测试定义

| 层 | 名称 | 职责 | 示例 |
|------|------|------|------|
| **unit** | 单元测试 | 验证单个函数/纯规则的正确性 | HP 计算、Tag 包含检查、Modifier 优先级排序 |
| **integration** | 集成测试 | 验证领域内多组件协作 | 角色穿戴装备→Modifier 生效→Attribute 变化 |
| **invariant** | 不变量测试 | 验证领域不变量（**最核心**） | Tag bit 唯一、Buff 不重复叠加、Effect 不修改不存在属性、HP>=0 |
| **fixtures** | 测试数据 | Builder 模式构造的测试数据 | RON 格式的角色模板、技能配置、战斗场景 |

### 不变量测试（Invariant Test）— 最高价值

SRPG 架构核心是 Attribute / Tag / Effect / Modifier / Buff / Skill / Turn，这些都有大量**领域不变量**：

| 不变量 | 说明 | 测试文件 |
|--------|------|----------|
| Tag bit 唯一 | 同一 Tag 不能在位掩码中重复设置 | `tag_invariant_spec.rs` |
| Buff 不重复叠加 | 同源同类型 Buff 不会无限堆叠 | `buff_invariant_spec.rs` |
| Effect 不修改不存在属性 | Effect 引用的 AttributeId 必须已注册 | `effect_invariant_spec.rs` |
| HP 永远 >= 0 | HP 计算结果不能为负 | `hp_invariant_spec.rs` |
| Modifier 不改变基础值 | Modifier 只影响聚合后的当前值 | `modifier_invariant_spec.rs` |
| 回合先攻排序稳定 | 同先攻值的单位顺序确定 | `turn_invariant_spec.rs` |
| 技能消耗原子性 | 消耗失败时不产生部分效果 | `ability_invariant_spec.rs` |

> 不变量测试的价值远大于普通单元测试，是架构稳定性的最后防线。

### 跨领域测试（根 tests/）

仅保留不属于任何单一领域的跨域测试：

```
tests/
├── battle_flow/                 # 完整战斗流程（先攻→行动→伤害→死亡→胜负）
├── save_load/                   # 存档/读档完整性
├── regression/                  # 回归测试（历史 Bug 复现）
├── replay/                      # 回放确定性（同一输入→同一输出）
├── golden/                      # 金文件对比（战斗结果与标准输出）
├── simulation/                  # 战斗模拟与数值平衡
├── performance/                 # 性能回归（帧率/内存基准）
└── e2e/                         # 端到端测试（完整游戏流程）
```

---

## 十五、资源文件结构

```
assets/
├── data/                   # 配置文件（对应 src/content 层）
│   ├── attributes/
│   ├── tags/
│   ├── modifiers/
│   ├── effects/
│   ├── abilities/
│   ├── triggers/
│   ├── characters/
│   ├── classes/
│   ├── enemies/
│   ├── items/
│   ├── equipments/
│   ├── quests/
│   ├── stories/
│   ├── maps/
│   ├── balance/
│   └── schemas/
│
├── localization/             # 多语言文本
│   ├── zh-CN/
│   └── en-US/
│
├── textures/                 # 贴图
├── sprites/
├── models/                   # 模型
├── animations/
├── shaders/                  # Shader
├── audio/                    # 音频
│   ├── sfx/
│   └── bgm/
│
├── ui/                       # UI 资源
├── vfx/
├── cinematics/
└── fonts/
```

---

## 十六、文档与脚本

### docs/

```
docs/
├── architecture/               # 架构文档
│   ├── layer-contracts.md      # 分层契约（依赖规则与禁止事项）
│   ├── ecs-spec.md             # ECS 使用规范（Component/System/Resource 使用约定）
│   ├── capability-domains.md   # 15个核心能力领域说明
│   ├── logging-standard.md     # 日志规范（详见第二十一节）
│   └── mod-api.md              # Mod API 说明
├── adr/                        # 架构决策记录
│   ├── ADR-001-ecs-layer-boundary.md
│   ├── ADR-002-gas-domain-design.md
│   ├── ADR-003-three-layer-separation.md
│   ├── ADR-004-five-to-three-layer.md  # 5层→3层架构调整决策
│   └── ...
├── domain-events.md            # 领域事件白皮书（所有 Domain 间事件清单）
├── error-codes.md              # 错误码对照表
└── style-guide.md              # 代码风格指南
```

### scripts/

```
scripts/
├── dependcheck.rs              # 架构依赖检查脚本（检测违反依赖规则的 use 语句）
├── content_validate.rs         # 配置数据批量校验
├── schema_export.rs            # 导出配置 Schema
└── ci/                         # CI 流程脚本
```

---

## 十七、关键架构决策

### 决策1：为什么 Capabilities 是15个而非18个

| 候选 | 判定 | 理由 |
|------|------|------|
| Container | 不作为领域 | ASC 是架构组件，不是游戏领域概念。在 Bevy 中由 AbilityContainer 组件实现 |
| Task | 不作为领域 | Bevy 原生 State/Event/Observer/Timer 替代 UE AbilityTask，属于实现模式 |
| Period | 不作为领域 | Effect 的属性（duration/period），独立会导致领域爆炸 |
| Prediction | 不作为领域 | 网络层概念，单机 SRPG 暂不需要 |
| Immunity | 合入 Condition | 免疫 = Tag(Immune.X) + Condition(Require Not Tag)，统一条件检查更优雅 |
| GameplayContext | 晋升为领域 | 大型项目后期所有 Event 重复 source/target/ability 字段会膨胀，统一载体是必然 |

### 决策2：Trigger 与 Event 为什么分离

| 维度 | Trigger | Event |
|------|---------|-------|
| 职责 | 技能激活条件 | 系统间结构化通信 |
| 示例 | 受到攻击→激活反击 | DamageDealt→触发连锁闪电 |
| 消费者 | Ability 系统 | 全系统 |
| 数据 | 轻量（条件表达式） | 重量（GameplayContext 载荷） |

### 决策3：三层分离（Def→Spec→Instance）的必要性

大型项目必然出现：
- 同一技能在不同角色身上有不同等级/强化/冷却缩减
- 同一效果在不同上下文中有不同来源/快照/堆叠数
- 没有Spec层，无法区分"技能定义"和"角色身上的技能配置"

### 决策4：Domain 间通信规则

- 禁止 Domain 之间直接引用（`use`）
- 仅通过 `Event` 通信
- 需要跨 Domain 协调的逻辑放在 `integration.rs` 中编排 Capabilities 能力
- 新增 Domain 间交互需求时，优先考虑是否可通过 Capabilities 的 Event 机制解决

### 决策5：5层→3层架构调整

> # 原5层架构（atom/rule/runtime/model/frame）调整为3层（Foundation/Mechanism/Runtime）
> # 核心理由：内聚优于分层

| 问题 | 5层架构 | 3层架构 |
|------|---------|---------|
| **L0+L1 人为分裂** | tag_base（L0）和 tag_query（L1）拆到不同目录，修改标签系统需跨层 | tag/ 内部自包含 foundation/ + mechanism/，修改不跨目录 |
| **L3 model 与 Domain 重复** | character_data 在 L3 和 Domain 的 components.rs 各一份 | 数据模型归入 Domain 的 components.rs，消除重复 |
| **L4 frame 与 Domain 重复** | quest_trait（L4）和 domains/quest/ 做同一件事 | 玩法编排归入 Domain 的 plugin.rs + integration.rs |
| **L2 runtime 职责混杂** | pipeline/scheduler/command/replay 混在一起 | pipeline/scheduler/registry 归 C3 Runtime，command/replay 独立 |

**3层映射表**：

| 原5层 | 新3层 | 归属 |
|-------|-------|------|
| L0 atom → types/tag_base/attribute_base/... | C1 Foundation | 各能力领域内部 foundation/ |
| L1 rule → tag_query/calc_pipeline/effect_lifecycle/... | C2 Mechanism | 各能力领域内部 mechanism/ |
| L2 runtime → pipeline/scheduler/registry | C3 Runtime | capabilities/runtime/ |
| L2 runtime → command/replay | C3 Runtime | capabilities/runtime/ |
| L3 model → character/item/map/... | 归入 Domain | domains/xxx/components.rs |
| L4 frame → quest/ai/event_bus/... | 归入 Domain | domains/xxx/plugin.rs + capabilities/event/ |

### 决策6：数值模拟系统设计

```
tools/battle_simulator/        # 战斗模拟器
├── mod.rs
├── sim_runner.rs              # 模拟运行器（无渲染，纯逻辑）
├── sim_config.rs              # 模拟配置（队伍/地图/规则）
├── sim_result.rs              # 模拟结果（胜率/回合数/DPS分布）
└── batch_runner.rs            # 批量模拟（蒙特卡洛采样）

tools/balance_analyzer/        # 数值平衡分析器
├── mod.rs
├── stat_collector.rs          # 统计数据采集
├── variance_analyzer.rs       # 方差分析（识别过强/过弱）
└── report_generator.rs        # 报告生成
```

模拟系统直接复用 Core 层的 Capabilities + Domains，通过 headless 模式运行，无需渲染。

---

## 十八、Feature Flag 设计

```toml
# Cargo.toml features
[features]
default = ["game"]

# 运行模式
game = []
editor = []
headless = []
server = []

# 开发工具（永不进入发布构建）
dev-tools = []

# 调试
debug-render = []              # 调试渲染（碰撞体/寻路/网格）
debug-ai = []                  # AI 决策可视化
verbose-logging = []           # 详细日志（TRACE 级别输出）

# 平台
steam = []
epic = []

# 可选能力
networking = []
cloud-save = []
anti-cheat = []
```

---

## 十九、完整源码目录一览

```
src/
├── main.rs
├── lib.rs
│
├── shared/                           # L0：底层原子层
│   ├── mod.rs
│   ├── ids/
│   ├── error/
│   ├── result/
│   ├── math/
│   ├── random/
│   ├── time/
│   ├── collections/
│   ├── hashing/
│   ├── validation/
│   ├── testing/
│   ├── traits/
│   ├── path/
│   └── prelude/
│
├── core/                             # L1：领域规则层
│   ├── mod.rs
│   ├── core_plugin.rs
│   │
│   ├── capabilities/                 # 15个核心能力领域
│   │   ├── mod.rs
│   │   ├── tag/                      # 标签
│   │   ├── attribute/                # 属性
│   │   ├── modifier/                 # 修改器
│   │   ├── aggregator/              # 聚合器
│   │   ├── gameplay_context/        # 上下文/载荷
│   │   ├── spec/                    # 规格/配置
│   │   ├── ability/                 # 技能逻辑
│   │   ├── trigger/                 # 触发器
│   │   ├── condition/               # 条件/限制/免疫
│   │   ├── targeting/               # 目标选择
│   │   ├── execution/               # 执行计算
│   │   ├── effect/                  # 效果
│   │   ├── stacking/                # 堆叠规则
│   │   ├── event/                   # 系统通信
│   │   ├── cue/                     # 表现层信号
│   │   └── runtime/                 # C3：跨领域运行时编排
│   │
│   ├── domains/                      # 业务子系统
│   │   ├── mod.rs
│   │   ├── combat/                   # 战斗
│   │   ├── spell/                    # 法术
│   │   ├── reaction/                 # 反应/援护
│   │   ├── progression/             # 成长养成
│   │   ├── inventory/               # 背包物品
│   │   ├── quest/                   # 任务
│   │   ├── narrative/               # 叙事/对话
│   │   ├── tactical/                # 战术/网格
│   │   ├── terrain/                 # 地形
│   │   ├── faction/                 # 阵营关系
│   │   ├── party/                   # 队伍
│   │   ├── camp_rest/              # 营地休息
│   │   ├── crafting/               # 制作
│   │   ├── economy/                # 经济
│   │   └── summon/                  # 召唤
│   │
│   └── mod_api/                      # Mod 稳定 API
│       ├── mod.rs
│       ├── core_facade.rs
│       ├── combat_gateway.rs
│       ├── character_gateway.rs
│       ├── spell_gateway.rs
│       ├── quest_gateway.rs
│       ├── party_gateway.rs
│       ├── camp_gateway.rs
│       ├── summon_gateway.rs
│       ├── terrain_gateway.rs
│       ├── craft_gateway.rs
│       ├── economy_gateway.rs
│       ├── inventory_gateway.rs
│       ├── faction_gateway.rs
│       ├── progression_gateway.rs
│       └── narrative_gateway.rs
│
├── infra/                            # L2：技术实现层
│   ├── mod.rs
│   ├── infra_plugin.rs
│   ├── foundation/
│   │   ├── ecs/
│   │   ├── serialization/
│   │   ├── asset/
│   │   ├── threading/
│   │   └── memory/
│   ├── services/
│   │   ├── logging/                  # 日志系统
│   │   ├── save/
│   │   ├── localization/
│   │   ├── physics/
│   │   ├── pathfinding/
│   │   ├── navmesh/
│   │   ├── input/
│   │   ├── analytics/
│   │   ├── hot_reload/
│   │   ├── encryption/
│   │   ├── anti_cheat/
│   │   ├── networking/
│   │   ├── cloud/
│   │   └── mod_loader/
│   └── presentation/
│       ├── rendering/
│       ├── ui/
│       ├── audio/
│       ├── animation/
│       ├── particles/
│       └── platform/
│
├── app/                              # 横切1：启动装配
│   ├── mod.rs
│   ├── app_plugin.rs
│   ├── game_app.rs
│   ├── editor_app.rs
│   ├── server_app.rs
│   ├── headless_app.rs
│   ├── state/
│   ├── bootstrap/
│   └── schedule/
│
├── content/                          # 横切2：内容桥接
│   ├── mod.rs
│   ├── content_plugin.rs
│   ├── mod_support/
│   ├── schema/
│   ├── attributes/
│   ├── tags/
│   ├── modifiers/
│   ├── effects/
│   ├── abilities/
│   ├── triggers/
│   ├── characters/
│   ├── enemies/
│   ├── classes/
│   ├── factions/
│   ├── maps/
│   ├── encounters/
│   ├── items/
│   ├── equipments/
│   ├── shops/
│   ├── loot_tables/
│   ├── progression/
│   ├── quests/
│   ├── stories/
│   ├── dialogues/
│   ├── localization/
│   ├── balance/
│   ├── migration/
│   └── validation/
│
├── tools/                            # 横切3：开发工具
│   ├── mod.rs
│   ├── tools_plugin.rs
│   ├── replay_viewer/
│   ├── battle_simulator/
│   ├── balance_analyzer/
│   ├── content_validator/
│   ├── localization_tool/
│   ├── save_editor/
│   ├── schema_generator/
│   ├── graph_viewer/
│   ├── profiling_tool/
│   ├── ai_debugger/
│   ├── map_editor/
│   ├── ability_editor/
│   ├── effect_editor/
│   ├── quest_editor/
│   ├── pipeline_inspector/
│   ├── log_analyzer/
│   ├── data_browser/
│   ├── replay_diff/
│   ├── migration_tool/
│   ├── dependency_checker/
│   └── test_runner/
│
└── modding/                          # 横切4：Mod 扩展
    ├── api/
    ├── registry/
    ├── loader/
    ├── sandbox/
    ├── compatibility/
    ├── documentation/
    └── examples/
```

---

## 二十、架构演进策略

### 阶段规划

| 阶段 | 目标 | 实现范围 |
|------|------|----------|
| **P0 骨架** | 可编译运行的空壳 | shared + capabilities 4基石(Tag/Attribute/Modifier/Aggregator) + app |
| **P1 能力** | 能力系统可用 | capabilities 全15领域 + runtime + content/schema + 基础测试 |
| **P2 战斗** | 核心玩法闭环 | domains/combat + spell + reaction + tactical + terrain |
| **P3 养成** | 角色成长体系 | domains/progression + inventory + party + camp_rest |
| **P4 叙事** | 剧情与任务 | domains/narrative + quest + faction |
| **P5 经济** | 交易与制作 | domains/economy + crafting + summon |
| **P6 工具** | 编辑器与调试 | tools 全部 + editor_app + log_analyzer |
| **P7 Mod** | Mod 生态 | modding 全部 + mod_api |

### 扩展守则

- 新增玩法优先组合已有 Capabilities，不造新系统
- 新增 Domain 遵循标准内部结构（plugin/components/systems/events/error/rules/integration）
- Capabilities 15领域封顶，不再增加；新机制以子模块形式归入已有领域
- Domain 间禁止直接依赖，仅通过 Event 通信
- 所有数值配置归 content/，代码只提供机制

---

## 二十一、Logging 统一管理机制

> # 日志是长期项目可维护性的基石
> # 10-20年维护周期中，日志是排查问题、分析平衡、回放调试的核心工具

### 21.1 日志级别划分

| 级别 | 用途 | 输出目标 | 性能影响 | 示例 |
|------|------|----------|----------|------|
| **TRACE** | 极细粒度，每帧系统执行细节 | 仅文件（dev-tools） | 高 | `tag_sync_system: 检查 Entity#42 标签变更` |
| **DEBUG** | 调试信息，关键逻辑节点 | 控制台+文件（dev-tools） | 中 | `ability_activate: 火球术激活，消耗法力20` |
| **INFO** | 关键业务节点 | 控制台+文件 | 低 | `combat: 战斗开始，先攻顺序 [战士, 法师, 哥布林]` |
| **WARN** | 异常但可恢复 | 控制台+文件 | 极低 | `stacking: 效果堆叠溢出，移除最早层` |
| **ERROR** | 功能性错误 | 控制台+文件 | 极低 | `spell_cast: 法术位不足，施法失败` |
| **FATAL** | 致命错误 | 控制台+文件+弹窗 | 无 | `save_load: 存档损坏，无法加载` |

### 21.2 日志输出格式

```
# 统一格式：
# [时间戳][级别][模块路径][帧号/回合号] 消息 {结构化数据}
#
# 时间戳：毫秒精度，格式 HH:MM:SS.mmm
# 级别：6级，左对齐5字符
# 模块路径：从 src/ 根开始的完整路径，如 combat::damage_system
# 帧号/回合号：Frame:NNNN/Turn:N（非战斗时只显示帧号）
# 消息：人类可读的简短描述
# 结构化数据：JSON 格式的键值对，便于日志分析器解析

# 示例输出：
[12:34:56.789][INFO ][combat::damage_system][Frame:1234/Turn:3] 伤害结算 {source: "战士", target: "哥布林", damage: 15, crit: true}
[12:34:56.790][DEBUG][ability::activate_system][Frame:1234/Turn:3] 技能激活 {ability: "火球术", cost: 20, target_count: 3}
[12:34:57.001][WARN ][stacking::stacking_system][Frame:1235/Turn:3] 堆叠溢出 {effect: "灼烧", limit: 5, current: 6, action: "RemoveOldest"}
```

### 21.3 日志存储策略

| 场景 | 控制台输出 | 文件输出 | 远程输出 |
|------|-----------|---------|---------|
| **开发期** | DEBUG+ | TRACE+（滚动） | - |
| **发布期** | INFO+ | WARN+（滚动） | - |
| **测试期** | WARN+ | TRACE+（全量） | - |
| **回放调试** | - | TRACE+（独立文件） | - |

**文件滚动策略**：

```
# 日志文件命名：fre_YYYYMMDD_HHMMSS.log
# 示例：fre_20260616_123456.log
#
# 滚动规则：
# - 单文件上限：50MB
# - 保留数量：最近 20 个文件
# - 按日期+会话ID分割（每次启动游戏创建新文件）
# - 压缩归档：超过保留数量的旧文件 gzip 压缩后删除原文件
#
# 存储位置：
# - 开发期：项目根目录/logs/
# - 发布期：系统日志目录（macOS: ~/Library/Logs/Fre/, Windows: %APPDATA%/Fre/logs/）
```

**回放日志（独立于普通日志）**：

```
# 回放日志记录确定性随机种子 + 命令流，用于回放复现
# 存储位置：项目根目录/replays/
# 命名：replay_YYYYMMDD_HHMMSS.rep
# 格式：二进制（种子 + 帧号 + 命令ID + 命令参数）
# 与普通日志分离，避免影响性能
```

### 21.4 日志分析方案

```
# tools/log_analyzer/ 提供以下能力：
#
# 1. 过滤查询
#    - 按模块路径过滤：--module combat::damage_system
#    - 按级别过滤：--level WARN+
#    - 按时间范围过滤：--from 12:30:00 --to 12:35:00
#    - 按关键字过滤：--grep "暴击"
#
# 2. 聚合统计
#    - 统计某模块的日志频率：--stats combat::damage_system
#    - 统计某类事件的平均值：--avg damage
#    - 统计错误分布：--error-distribution
#
# 3. 回放对比
#    - 对比两次运行的日志差异：--diff log1.log log2.log
#    - 检测非确定性：同一输入不同输出→高亮差异行
#
# 4. 报告生成
#    - 生成 HTML 报告：--report output.html
#    - 包含：错误统计/性能热点/异常模式
```

### 21.5 实现架构

```
# 日志系统分层：
#
# shared/traits/logging_trait.rs  → 日志抽象接口（所有层通过此接口使用日志）
#   - trait GameLog: fn log(level, module, message, data)
#   - 不依赖具体实现，Core/Infra/Content 都通过此接口写日志
#
# infra/services/logging/        → 日志实现（只有 Infra 知道具体实现）
#   - log_manager.rs: 实现 GameLog trait，管理级别过滤/输出路由
#   - log_formatter.rs: 格式化日志（统一输出格式）
#   - log_writer.rs: 写入器（控制台/文件/远程，可扩展）
#   - log_config.rs: 配置（级别/输出目标/滚动策略，通过 RON 配置文件加载）
#
# 与 Bevy 集成：
#   - 基于 Bevy 的 tracing 生态（bevy_log 基于 tracing）
#   - 自定义 tracing Subscriber 实现结构化日志
#   - 通过 Resource<LogConfig> 暴露日志配置（运行时可调级别）
#   - 通过 Event<LogEntry> 实现日志事件（UI 可订阅显示日志面板）
```

### 21.6 各层日志使用规范

| 层 | 允许级别 | 禁止事项 |
|------|---------|----------|
| **Shared** | 不写日志 | 禁止引入日志依赖 |
| **Core/Capabilities** | DEBUG+ | 禁止 TRACE（性能敏感），使用 `log_if_debug!` 宏 |
| **Core/Domains** | INFO+ | 业务关键节点必须 INFO，规则判定用 DEBUG |
| **Infra** | WARN+ | 内部实现细节用 TRACE（仅 dev-tools） |
| **Content** | INFO+ | 加载/校验/注册结果必须 INFO |
| **App** | INFO+ | 启动/状态切换必须 INFO |
| **Tools** | 不限 | 开发工具可自由使用任何级别 |

### 21.7 日志宏定义

```rust
// shared/traits/logging_trait.rs 中定义的日志宏
// 设计思路：统一日志入口，编译期级别检查，零开销（Release 中 TRACE/DEBUG 宏展开为空）
// 注意事项：
//   - log_debug! 和 log_trace! 使用不同 feature gate，区分「开发工具日志」和「极细粒度日志」
//   - dev-tools：开发期通用调试日志（编辑器/模拟器/调试UI均需要）
//   - verbose-logging：仅用于排查特定问题的每帧级日志，性能影响大，默认关闭
//   - Release 构建中两者均展开为空，零运行时开销

/// INFO 级别日志，记录关键业务节点
/// 参数：模块路径, 消息, 结构化数据键值对
/// 示例：log_info!("combat::damage_system", "伤害结算", source => "战士", damage => 15);
#[macro_export]
macro_rules! log_info {
    ($module:expr, $msg:expr, $($key:ident => $val:expr),* $(,)?) => { ... };
}

/// DEBUG 级别日志，仅在 dev-tools feature 启用时输出
/// 用途：开发期调试信息，如技能激活、效果施加等关键逻辑节点
/// 注意：不用于每帧高频输出（每帧输出用 log_trace!）
#[cfg(feature = "dev-tools")]
#[macro_export]
macro_rules! log_debug {
    ($module:expr, $msg:expr, $($key:ident => $val:expr),* $(,)?) => { ... };
}

/// WARN 级别日志，记录异常但可恢复的情况
/// 用途：堆叠溢出、配置缺失、降级处理等
#[macro_export]
macro_rules! log_warn {
    ($module:expr, $msg:expr, $($key:ident => $val:expr),* $(,)?) => { ... };
}

/// ERROR 级别日志，记录功能性错误
/// 用途：施法失败、存档损坏、配置校验不通过等
#[macro_export]
macro_rules! log_error {
    ($module:expr, $msg:expr, $($key:ident => $val:expr),* $(,)?) => { ... };
}

/// TRACE 级别日志，仅 verbose-logging feature 启用时输出
/// 用途：每帧系统执行细节，如标签同步遍历、属性重算过程
/// 注意：性能影响大，仅排查特定问题时启用，排查完毕后关闭
#[cfg(feature = "verbose-logging")]
#[macro_export]
macro_rules! log_trace {
    ($module:expr, $msg:expr, $($key:ident => $val:expr),* $(,)?) => { ... };
}

/// 非 dev-tools 构建中，log_debug! 展开为空（编译期消除）
#[cfg(not(feature = "dev-tools"))]
#[macro_export]
macro_rules! log_debug {
    ($module:expr, $msg:expr, $($key:ident => $val:expr),* $(,)?) => {};
}

/// 非 verbose-logging 构建中，log_trace! 展开为空（编译期消除）
#[cfg(not(feature = "verbose-logging"))]
#[macro_export]
macro_rules! log_trace {
    ($module:expr, $msg:expr, $($key:ident => $val:expr),* $(,)?) => {};
}
```

### 21.8 异步写入与性能防护

```
# 日志写入策略：
#
# 1. 异步通道写入（避免阻塞主线程）
#    - 日志宏 → 无锁 mpsc Sender → 独立写入线程 → 文件/控制台
#    - 主线程只做序列化+发送，不等待IO完成
#    - 通道缓冲区：8192条（溢出时丢弃最旧日志并计数）
#    - 写入线程每 100ms 或缓冲区半满时 flush 一次
#
# 2. 每帧日志数量防护（防止日志风暴）
#    - 每帧上限：100条（超过后本帧剩余日志降级为 TRACE 级别，仅写入文件）
#    - 每秒上限：500条（超过后 WARN 级别以下日志丢弃）
#    - 防护计数器每帧重置，不影响下一帧
#    - 丢弃计数在帧末以 WARN 级别输出：「本帧丢弃 N 条日志」
#
# 3. 结构化日志 Schema（便于 log_analyzer 解析）
#    {
#      "ts": "HH:MM:SS.mmm",       // 时间戳，毫秒精度
#      "lvl": "INFO",              // 日志级别
#      "mod": "combat::damage",     // 模块路径
#      "frame": 1234,              // 帧号
#      "turn": 3,                  // 回合号（非战斗时为 null）
#      "msg": "伤害结算",           // 人类可读消息
#      "data": {                   // 结构化数据键值对
#        "source": "战士",
#        "target": "哥布林",
#        "damage": 15,
#        "crit": true
#      }
#    }
#
# 4. 性能基准目标
#    - 单条日志序列化：< 500ns
#    - 通道发送（无竞争）：< 100ns
#    - 每帧日志总开销（100条）：< 0.1ms
#    - 写入线程不影响帧率（独立线程，不抢占主线程时间片）
```
