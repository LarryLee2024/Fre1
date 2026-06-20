# Pipeline 管线系统深度解析——从通用引擎到四条业务管线的编排哲学

> Fre 项目里"Pipeline"这个词出现得很频繁：Ability Pipeline、Modifier Pipeline、Combat Pipeline、Content Loading Pipeline。拿到一起看容易糊涂——它们到底是一个东西伪装成不同名字，还是完全不同的东西恰巧都叫 Pipeline？答案是：**有一个通用引擎，然后四条业务管线在上面长出了各自的形状。** 本文把这四条管线的设计和实现拆开讲清楚，顺便解释那个通用引擎到底管什么。

---

## 1. Pipeline 是什么——一个比 System 粗、比 State Machine 细的执行单元

在解释具体管线之前，先理解为什么这个项目需要 Pipeline。

Bevy 本身提供了两种"编排"模式：
- **System + Schedule**：每帧运行一次，适合持续进行的逻辑（移动、AI 决策）
- **State + OnEnter/OnExit**：状态切换时运行，适合粗粒度的流程（战斗开始/结束）

但游戏里有一种流程，既不是每帧持续跑，也不是一锤子状态切换——它是一个**多步骤的、带顺序的、中间可能暂停等待外部输入的执行计划**。比如"一个单位的完整回合"：开始 → 检查阶段 → 等待玩家操作 → 结算效果 → 结束。这种流程用 Schedule 写会变得支离破碎（每个步骤一个 System，靠 Global Resource 沟通状态），用 State Machine 写会变得异常膨胀（每个步骤一个 State）。

Pipeline 就是给这类流程设计的。它把"执行计划"抽象成了四个层次：

```
PipelineDefinition（管线定义）
  └── PipelineStage（阶段——一组有序步骤的集合）
        └── PipelineStep（步骤——一个可执行的原子动作）
              └── StepExecutor（执行函数——实际做事的代码）
```

这个抽象的好处是：管线本身是一个数据结构（`PipelineDefinition`），你可以注册它、录制它、序列化它、在测试里重放它。而执行引擎是通用的，不关心你在跑哪个业务管线。

---

## 2. 通用 Pipeline 引擎——不包含业务逻辑的编排器

通用引擎位于 `src/core/capabilities/runtime/pipeline/`，infra 层有一个 ECS 集成包装在 `src/infra/pipeline/`。两者分工明确：

| 层 | 路径 | 职责 |
|----|------|------|
| 纯逻辑层（Core） | `capabilities/runtime/pipeline/` | 类型定义、执行引擎、注册中心、Hook Trait |
| ECS 集成层（Infra） | `infra/pipeline/` | PipelinePlugin、PipelineRegistry Resource、内置 Hook 实现 |

### 2.1 核心类型

**PipelineDefinition**——整条管线的蓝图：

```rust
pub struct PipelineDefinition {
    pub id: String,              // 管线标识，如 "combat.turn"
    pub stages: Vec<PipelineStage>,  // 有序阶段列表
}
```

**PipelineStage**——执行阶段，包含多个 Step：

```rust
pub struct PipelineStage {
    pub name: String,
    pub steps: Vec<PipelineStep>,
    pub on_failure: FailureStrategy,  // 本阶段步骤失败时的策略
    pub skippable: bool,              // 空步骤时是否可跳过
}
```

**PipelineStep**——四种步骤类型：

```rust
pub enum PipelineStep {
    System(String),          // 执行一个命名 System
    Rule(String),            // 执行一个命名规则
    SubPipeline(String),     // 执行一个子管线
    Conditional {            // 条件分支
        condition: String,
        if_true: Box<PipelineStep>,
        if_false: Box<PipelineStep>,
    },
}
```

**FailureStrategy**——失败处理策略：

```rust
pub enum FailureStrategy {
    Abort,                // 失败时立即终止整条管线
    SkipAndContinue,      // 跳过失败的步骤，继续后续步骤
    Retry { max_retries: u8 },  // 最多重试 N 次
}
```

**PipelineContext**——步骤间传递数据的容器：

```rust
pub struct PipelineContext {
    pub pipeline_id: String,
    pub stage_data: HashMap<String, String>,  // 阶段间数据传递
    pub execution_log: Vec<ExecutionLogEntry>,
    pub aborted: bool,
    pub abort_reason: Option<String>,
}
```

### 2.2 执行引擎——`execute_pipeline()`

核心执行函数（`executor.rs`）的逻辑非常直白：

```
execute_pipeline(definition, context, executor):
    for stage in definition.stages:
        if context.aborted:  → 返回 Err(Aborted)
        for step in stage.steps:
            result = executor(step.name, context, stage.name)
            match result:
                Success → 继续
                Failure(err):
                    Abort 策略 → 返回 Err(StepFailed)
                    SkipAndContinue 策略 → 记录失败，继续
                    Retry 策略 → 重试，耗尽后返回 Err
                Skipped → 跳过
```

`StepExecutor` 是一个函数指针，由业务管线提供：

```rust
pub type StepExecutor =
    fn(step_name: &str, context: &mut PipelineContext, stage_name: &str) -> StepResult;
```

引擎不关心这个函数里是什么——它只负责"调用你，把结果告诉你，按失败策略处理"。

### 2.3 PipelineHook——只观察不阻断

```rust
pub trait PipelineHook: Send + Sync {
    fn name(&self) -> &str;
    fn on_stage_start(&self, stage: &str, context: &PipelineContext) {}
    fn on_stage_end(&self, stage: &str, context: &PipelineContext, result: &StepResult) {}
    fn on_step_start(&self, stage: &str, step: &str, context: &PipelineContext) {}
    fn on_step_end(&self, stage: &str, step: &str, context: &PipelineContext, result: &StepResult) {}
}
```

所有方法都有默认空实现——你只需要覆写关心的方法。Hook 不是为了阻断执行而存在的（不能返回 Result），它是"在管线执行时顺便干点别的"：

| Hook | 谁用 | 干什么 |
|------|------|--------|
| ExecutionLogHook | `infra/pipeline/hooks.rs` | trace! 级别日志 |
| ReplayRecorder | `infra/replay` | 录制每个 Step 的输入/输出 |
| MetricsCollector | `tools/` (dev) | Step 耗时统计 |
| ValidationHook | `tests/` | 测试时验证执行路径 |

### 2.4 PipelineRegistry——注册中心

```rust
#[derive(Resource)]
pub struct PipelineRegistry {
    pipelines: HashMap<String, PipelineDefinition>,
    hooks: Vec<Box<dyn PipelineHook>>,
}
```

管线定义通过 `register()` 注册，执行时通过 `get()` 获取。禁止重复注册（panic 防止静默覆盖，破坏 Replay 确定性）。

### 2.5 管线事件

执行过程中会触发四个事件，供外部的 Observer 系统响应：

```rust
PipelineStarted      // 管线开始执行
PipelineStepCompleted// 每个步骤完成
PipelineFailed       // 管线失败
PipelineCompleted    // 管线全部完成
```

### 2.6 两条红线

```
🟥 Pipeline 引擎禁止包含任何业务逻辑（只编排不执行）
🟥 禁止运行时动态增删 Stage（破坏 Replay 确定性）
```

---

## 3. Pipeline 1：Ability Pipeline——技能执行六阶段（ADR-010）

### 3.1 定位

通用引擎是一个"编排架子"，Ability Pipeline 是这个架子的第一个使用者。它定义了"一个技能被激活后，怎么完整执行一遍"的六个阶段。

### 3.2 六阶段定义

```
Phase 1: Validate     ── 目标校验、条件检查、冷却检查
Phase 2: PreCost      ── 扣消耗（SP/MP/物品）
Phase 3: Targeting    ── 确定最终目标集合
Phase 4: Execute      ── 生成 Effect 列表
Phase 5: Resolve      ── Effect 逐一执行（走 Effect Pipeline）
Phase 6: PostCost     ── 冷却开始、充能消耗
```

### 3.3 每个阶段的输入输出

| 阶段 | 输入 | 输出 | 失败策略 |
|------|------|------|---------|
| Validate | ActivationRequest | TargetSet 或错误 | Abort（校验不通过就没必要继续） |
| PreCost | TargetSet, CostEntry | 消耗后的资源状态 | Abort（扣费失败是程序错误） |
| Targeting | 候选目标 | TargetData | Abort（无有效目标则技能失败） |
| Execute | TargetData | Vec<EffectInstance> | SkipAndContinue（部分效果失败不阻断） |
| Resolve | Vec<EffectInstance> | 执行报告 | SkipAndContinue（逐个 Effect 独立） |
| PostCost | 执行报告 | CooldownEntry | SkipAndContinue |

### 3.4 与通用引擎的关系

Ability Pipeline 的六阶段对应通用引擎的 `PipelineStage`，每个阶段内的子操作对应 `PipelineStep`。但截至当前代码，Ability Pipeline 还没有强制使用通用引擎来编排——六个阶段的执行是通过 ECS System + Event 链（`commands.trigger()` + Observer）隐式完成的：

```
try_activate() → AbilityActivated 事件
  → Phase 1 (Validate) 由 Observer 处理
    → Phase 2 (PreCost) 由 Observer 处理
      → Phase 3 (Targeting) 由 Observer 处理
        → ...
```

通用引擎的注册式编排（`PipelineDefinition` → `PipelineRegistry`）是设计目标（ADR-044），实际迁移可能是渐进式的。

### 3.5 代码入口

| 文件 | 内容 |
|------|------|
| `ability/mechanism/lifecycle.rs` | `try_activate()`——激活入口 |
| `ability/events.rs` | AbilityActivated, AbilityCompleted 等事件 |
| `ability/foundation/def.rs` | AbilityDef 定义 |

---

## 4. Pipeline 2：Modifier Pipeline——属性聚合四阶段（ADR-011）

### 4.1 定位

Ability Pipeline 管"技能怎么执行"，Modifier Pipeline 管"数值怎么被改"。它负责把一堆散布在 Effect 里的 Modifier，变成最终写回 Attribute 的数值。

### 4.2 四阶段定义

```
Phase 1: Collect   ── 从实体的 ModifierSet 收集所有活跃 Modifier
Phase 2: Aggregate ── Add → Multiply → Override → Clamp 四步计算
Phase 3: Resolve   ── 输入 AttributeResolver 计算最终属性值
Phase 4: Publish   ── 发布 AggregationComplete 事件
```

### 4.3 Aggregate 阶段的四步计算（最关键的细节）

这一步在 `aggregator/mechanism/pipeline.rs` 里实现，是一个**纯函数**——没有副作用，没有外部状态：

```rust
pub fn execute_aggregation(
    attribute_id: &str,
    base_value: f32,
    modifiers: &[ModifierEntry],
    pipeline: &CalcPipeline,
    min_value: f32,
    max_value: f32,
    frame: u64,
) -> Result<AggregationResult, PipelineError> {
    // Step 1: Add  — 把所有 Add modifier 加起来
    let add_sum = sum_of_op(modifiers, ModifierOp::Add);
    let after_add = base_value + add_sum;

    // Step 2: Multiply — 连乘！1.2 × 1.3 = 1.56，不是 1 + 0.2 + 0.3
    let mul_product = product_of_op(modifiers, ModifierOp::Multiply);
    let after_mul = after_add * mul_product;

    // Step 3: Override — 取最高优先级的 Override 值
    let after_override = highest_priority_override(modifiers)
        .map(|m| m.magnitude)
        .unwrap_or(after_mul);

    // Step 4: Clamp — 边界保障
    let final_val = after_override.clamp(min_value, max_value);

    AggregationResult {
        final_value,
        stage_values: [after_add, after_mul, after_override, final_val],
        was_overridden: override_mod.is_some(),
        base_value,
    }
}
```

关键理解：
- **Add 和 Multiply 不是并列关系，是串联关系**——先加后乘。所以一个 +5 的 Add 和一个 ×1.2 的 Multiply，结果是 `(base + 5) × 1.2`。
- **Multiply 是连乘**——两个 ×1.2 的 Modifier 结果是 ×1.44（不是 ×1.4）。
- **Override 是"我最特殊"**——取优先级最高的 Override 值，直接替换掉前面的计算结果。
- **Clamp 是安全网**——防止溢出或负数。

### 4.4 怎么触发聚合

Modifier 的添加和移除会触发 `ModifierApplied` / `ModifierRemoved` 事件，Aggregator 的 Observer 响应后标记对应属性为 `Dirty`。下一次读取属性时自动触发重算：

```
Effect: apply → 注入 Modifier → commands.trigger(ModifierApplied)
  → Aggregator Observer 响应 → 标记属性 Dirty
    → 下次属性读取 → 自动触发 execute_aggregation()
      → 结果写入 AttributeComponent → commands.trigger(AggregationComplete)
```

### 4.5 HP/MP 直接修改例外

宪法规定了一个例外：HP、MP 这类资源型属性允许专用系统直接修改。这是因为 HP 变化太频繁（每次攻击、每次治疗），每次都走 Modifier → Aggregator 管线开销太大。但即使是直接修改，也必须通过 Effect 作为业务入口——你不能在 System 里写 `hp.current -= 50`。

### 4.6 代码入口

| 文件 | 内容 |
|------|------|
| `modifier/foundation/types.rs` | ModifierOp（Add/Multiply/Override） |
| `modifier/mechanism/lifecycle.rs` | create_modifier, validate_modifier_data |
| `aggregator/mechanism/pipeline.rs` | execute_aggregation——四阶段纯函数 |
| `aggregator/foundation/values.rs` | CalcPipeline, ModifierEntry, AggregationResult |

---

## 5. Pipeline 3：Combat Turn Pipeline——回合流程五阶段（ADR-020/021）

### 5.1 定位

这是整个项目里最"像 Pipeline"的一条管线——它是通过通用引擎的 `PipelineRegistry` 注册的，用 `PipelineDefinition` 定义的，有一个专门的"驾驶员" Resource 来驱动执行。

### 5.2 五阶段定义

定义在 `combat/pipeline/definition.rs`：

```rust
pub fn build_turn_pipeline() -> PipelineDefinition {
    PipelineDefinition::new("combat.turn")
        .stage(PipelineStage::new("turn_start")     // 回合开始初始化
            .step(PipelineStep::System("turn_start"))
            .on_failure(FailureStrategy::Abort))
        .stage(PipelineStage::new("phase_check")     // 检查单位能否行动
            .step(PipelineStep::System("phase_check"))
            .on_failure(FailureStrategy::Abort))
        .stage(PipelineStage::new("unit_action")     // 单位执行操作（暂停等待输入）
            .step(PipelineStep::System("unit_action"))
            .on_failure(FailureStrategy::Abort))
        .stage(PipelineStage::new("turn_settlement") // 结算（Tick Effect、冷却）
            .step(PipelineStep::System("turn_settlement"))
            .on_failure(FailureStrategy::Abort))
        .stage(PipelineStage::new("turn_end")        // 完毕，切到下一个单位
            .step(PipelineStep::System("turn_end"))
            .on_failure(FailureStrategy::Abort))
}
```

### 5.3 驾驶员模式——普通的 Pipeline 不够用

这里出现了一个通用引擎没有考虑的场景：**等待外部输入**。

`UnitAction` 阶段执行后，管线需要停下来，等玩家选择技能、选择目标、确认执行。通用引擎的 `execute_pipeline()` 是同步的——一口气执行完所有步骤。但在回合制游戏里，一个单位的行动需要等待人类操作。

解决方案是 **CombatPipelineDriver**——一个 Resource，逐帧驱动管线：

```rust
pub struct CombatPipelineDriver {
    state: PipelineState,   // 当前执行位置（阶段索引、步骤索引）
    paused: bool,           // 是否暂停等待外部输入
}
```

每一帧，`combat_pipeline_driver` System 检查驾驶员状态：

```
if driver.is_driving():
    1. 从 PipelineRegistry 获取管线定义
    2. 读取当前阶段 + 当前步骤
    3. 按步骤名称分发执行
    4. 检查本步骤是否导致暂停

    PhaseCheck 阶段：
      → 单位有行动点 → 跳转到 UnitAction 阶段，暂停
      → 单位无行动点 → 跳转到 TurnSettlement 阶段

    UnitAction 阶段：
      → 暂停等待 UnitActionComplete 事件
        → 外部系统（Ability/Execution）在行动完成后触发此事件
          → on_unit_action_complete Observer 恢复驾驶员
            → 跳转到 TurnSettlement 阶段
```

### 5.4 完整的回合执行流程

```text
CombatPipelineDriver
  │
  ├── turn_start
  │    → round.evaluate_triggers()
  │    → tick_effects()
  │    → tick_cooldowns()
  │    ↓
  ├── phase_check
  │    → 检查当前单位的 ActionPoints
  │    → 有行动点？→ 跳转到 unit_action
  │    → 无行动点？→ 跳转到 turn_settlement
  │    ↓
  ├── unit_action
  │    → 暂停，等待玩家/AI 输入
  │    → 玩家操作 → commands.trigger(UnitActionComplete)
  │    → on_unit_action_complete Observer → 恢复驾驶员
  │    ↓
  ├── turn_settlement
  │    → 后处理（某些效果在此结算）
  │    ↓
  └── turn_end
       → 移除当前单位，推入下一位
       → 战斗结束？→ 设置 BattlePhase::Victory
       → 继续？→ driver.start_turn() 重置驾驶员

  上述流程每帧执行一个步骤。
  在 unit_action 处可能暂停多帧（等待玩家操作）。
```

### 5.5 代码入口

| 文件 | 内容 |
|------|------|
| `combat/pipeline/definition.rs` | `build_turn_pipeline()`——管线定义 |
| `combat/pipeline/driver.rs` | `CombatPipelineDriver` Resource + `combat_pipeline_driver` System + `on_unit_action_complete` Observer |
| `combat/plugin.rs` | 注册管线定义到 PipelineRegistry + 注册 Observer |
| `combat/components.rs` | ActionPoints, TurnQueue, BattlePhase |

---

## 6. Pipeline 4：Content Loading Pipeline——配置文件的旅程（ADR-047）

### 6.1 定位

前面三条管线管的是"运行时逻辑"，Content Loading Pipeline 管的是"启动时怎么把 RON 配置文件变成可用的运行时数据结构"。它和前面三条不是一个性质的 Pipeline——它使用 Bevy Asset 系统，而不是通用 Pipeline 引擎。

### 6.2 五阶段流程

```
Phase 1: Discovery（发现）
  ── 扫描 assets/config/ 下所有 .ron 文件
  ── 按目录名映射到对应的 Bucket（abilities/ → abilities bucket）

Phase 2: Loading（加载）
  ── 每个 .ron 文件调用 AssetServer::load()
  ── Bevy 自动反序列化为 typed Asset

Phase 3: Validation（校验）
  ── 加载完成后触发 Observer: OnAdd<T>
  ── 检查：ID 格式、必填字段、引用完整性、数值范围

Phase 4: Registration（注册）
  ── 将 Handle<T> 注册到 RegistryBucket<T>
  ── 更新索引（tag/category 查询）
  ── 冲突检测（重复 ID 报错）

Phase 5: Notification（通知）
  ── 触发 OnDefinitionReloaded 事件
  ── 下游 Observer 响应（如 UI 刷新）
```

### 6.3 与三条运行时管线的区别

| 特性 | Content Pipeline | 三条运行时管线 |
|------|-----------------|---------------|
| 执行时机 | 启动时 + 热重载 | 游戏运行时 |
| 执行引擎 | Bevy Asset 系统 | 通用 Pipeline 引擎 |
| 是否可暂停 | 否（一次性执行） | 是（Combat Turn 可暂停） |
| 输入 | RON 文件 | GameCommand / ModifierSet / Event |
| 输出 | Registry 更新 | 属性变更 / 事件链 |

### 6.4 content.rs 桥接

Content Pipeline 的终点是各个 Capability 的 `content.rs` 模块。这些模块把加载好的 Raw 数据翻译成运行时结构：

```rust
// tag/content.rs — RON → TagHierarchy
pub(crate) fn register_tags_from_content(
    mut hierarchy: ResMut<TagHierarchy>,
    mut loaded_tags: ResMut<LoadedTagDefs>,
    mut commands: Commands,
) {
    let defs = std::mem::take(&mut loaded_tags.defs);
    for def in defs {
        hierarchy.register(def, &mut commands);
    }
}
```

### 6.5 代码入口

| 文件 | 内容 |
|------|------|
| `docs/01-architecture/40-cross-cutting/ADR-047-content-loading-pipeline.md` | 完整设计 |
| `src/content/content_plugin.rs` | ContentPlugin——Asset 注册 + 加载触发 |
| `src/core/capabilities/{tag,attribute,ability,effect}/content.rs` | 各域的注册桥接 |

---

## 7. 四条管线的关系——它们怎么拼接在一起

四条管线不是孤立的。在实际游戏运行中，它们串联成了完整的执行路径：

```text
[外圈] Content Loading Pipeline
  启动时加载 Def → 注册到 Registry
  │
  ├── 游戏运行时 ──→ [内圈] 四条管线协作
  │
  │  玩家操作 → GameCommand
  │    │
  │    ▼
  │  [Combat Turn Pipeline] 驱动回合流程
  │    │  UnitAction 阶段
  │    │    │
  │    │    ▼
  │    │  激活技能 → [Ability Pipeline] 六阶段执行
  │    │    │  Resolve 阶段
  │    │    │    │
  │    │    │    ▼
  │    │    │  Effect 施加 → [Modifier Pipeline] 四阶段聚合
  │    │    │    │  → 属性变化 → AggregationComplete 事件
  │    │    │    │  → Trigger 检查 → 连锁反应
  │    │    │    │  → Cue 信号 → 表现层
  │    │    │    │
  │    │    │    └── 所有 Effect 执行完毕 → AbilityCompleted
  │    │    │
  │    │    └── UnitActionComplete → 恢复 Combat Turn Pipeline
  │    │
  │    └── turn_end → 下一个单位的回合
  │
  └── 热重载 → Content Pipeline 重新加载 → 增量注册
```

三种运行时管线的协作时序：

```
时间 →
┌─────────────────────────────────────────────┐
│  Combat Turn Pipeline  (慢节奏, 逐帧驱动)    │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌─────┐ ┌────┐│
│  │ T.S. │ │ P.C. │ │ U.A. │ │ T.S.│ │T.E.││
│  └──┬───┘ └──┬───┘ └──┬───┘ └──┬──┘ └──┬─┘│
│     │        │        │        │       │   │
│     ▼        ▼        ▼        ▼       ▼   │
│  Ability Pipeline  (快节奏, Event 链)       │
│  ┌──┐ ┌────┐ ┌──────┐ ┌────┐ ┌────┐ ┌──┐ │
│  │V.│ │PC. │ │ Targ │ │ Ex │ │ R. │ │PC.│ │
│  └──┘ └────┘ └──────┘ └────┘ └────┘ └──┘ │
│     │                  │                  │
│     ▼                  ▼                  ▼
│  Modifier Pipeline  (即时, Dirty 触发)      │
│  ┌────┐ ┌──────┐ ┌────┐ ┌────┐            │
│  │Coll│ │Aggr.│ │Res.│ │Pub.│            │
│  └────┘ └──────┘ └────┘ └────┘            │
└─────────────────────────────────────────────┘
```

---

## 8. 为什么是"Pipeline"——对比 Bevy 原生方案

| 方案 | 适合场景 | 不适合 |
|------|---------|--------|
| **Schedule + System** | 每帧持续执行的逻辑 | 多步骤顺序流程（需要大量状态变量） |
| **State + OnEnter/OnExit** | 粗粒度状态切换 | 细粒度步骤编排（State 爆炸） |
| **Event + Observer** | 松散耦合的事件响应 | 严格顺序的执行链路（Event 是广播的） |
| **Pipeline** | 有序多步骤 + 暂停恢复 + 确定性 | 高频率无状态的数值计算 |

Pipeline 不替代 Bevy 的方案——它在 Bevy 方案的缝隙里填补了一个特定的生态位。

---

## 9. 所有 Pipeline 相关源文件索引

### 通用引擎（Core 层）

| 文件 | 说明 |
|------|------|
| `capabilities/runtime/pipeline/foundation/types.rs` | PipelineStage, PipelineStep, FailureStrategy, StepResult, PipelineContext, ExecutionLogEntry |
| `capabilities/runtime/pipeline/foundation/values.rs` | PipelineDefinition, PipelineState |
| `capabilities/runtime/pipeline/foundation/error.rs` | PipelineError 枚举（4 个变体） |
| `capabilities/runtime/pipeline/mechanism/executor.rs` | execute_pipeline(), validate_pipeline() |
| `capabilities/runtime/pipeline/registry.rs` | PipelineRegistry Resource |
| `capabilities/runtime/pipeline/hooks.rs` | PipelineHook trait |
| `capabilities/runtime/pipeline/events.rs` | PipelineStarted, PipelineStepCompleted, PipelineFailed, PipelineCompleted |

### 通用引擎（Infra 层）

| 文件 | 说明 |
|------|------|
| `infra/pipeline/mod.rs` | 重导出 Core 层所有 public 类型 |
| `infra/pipeline/plugin.rs` | PipelinePlugin（init PipelineRegistry） |
| `infra/pipeline/hooks.rs` | ExecutionLogHook 实现 |

### Ability Pipeline

| 文件 | 说明 |
|------|------|
| `capabilities/ability/mechanism/lifecycle.rs` | try_activate() 激活入口 |
| `capabilities/ability/events.rs` | AbilityActivated, AbilityCompleted 等 |
| `capabilities/ability/foundation/def.rs` | AbilityDef 定义 |
| `docs/01-architecture/10-capability-system/ADR-010-ability-pipeline.md` | 六阶段设计 |

### Modifier Pipeline + Aggregator

| 文件 | 说明 |
|------|------|
| `capabilities/modifier/foundation/types.rs` | ModifierOp 原子操作 |
| `capabilities/modifier/mechanism/lifecycle.rs` | create_modifier |
| `capabilities/modifier/events.rs` | ModifierApplied, ModifierRemoved |
| `capabilities/aggregator/mechanism/pipeline.rs` | execute_aggregation 四阶段纯函数 |
| `capabilities/aggregator/foundation/types.rs` | CalcStage, ModifierOp, PipelineError |
| `capabilities/aggregator/foundation/values.rs` | CalcPipeline, ModifierEntry, AggregationResult |
| `capabilities/aggregator/events.rs` | AggregationComplete, AggregateDirty |
| `docs/01-architecture/10-capability-system/ADR-011-modifier-pipeline.md` | 四阶段设计 |

### Combat Turn Pipeline

| 文件 | 说明 |
|------|------|
| `domains/combat/pipeline/definition.rs` | build_turn_pipeline() |
| `domains/combat/pipeline/driver.rs` | CombatPipelineDriver + 逐帧驱动 + Observer |
| `domains/combat/pipeline/steps.rs` | 各步骤实现函数 |
| `domains/combat/plugin.rs` | 注册管线、Observer |
| `docs/01-architecture/20-tactical-combat/ADR-020-combat-pipeline.md` | Combat Pipeline 七阶段 |
| `docs/01-architecture/20-tactical-combat/ADR-021-turn-state-machine.md` | 回合状态机设计 |

### Content Loading Pipeline

| 文件 | 说明 |
|------|------|
| `src/content/content_plugin.rs` | ContentPlugin + Asset 注册 |
| `src/core/capabilities/tag/content.rs` | Tag RON → TagHierarchy 桥接 |
| `docs/01-architecture/40-cross-cutting/ADR-047-content-loading-pipeline.md` | 五阶段加载设计 |

---

## 10. 设计红线汇总

| 规则 | 来源 | 违反后果 |
|------|------|---------|
| Pipeline 引擎禁止包含任何业务逻辑 | ADR-044 | 架构审查不通过 |
| 禁止运行时动态增删 Stage | ADR-044 | 破坏 Replay 确定性 |
| Hook 禁止修改 PipelineContext 中的业务数据 | ADR-044 | 调试/回放不一致 |
| 禁止绕过 Effect Pipeline 直接修改战斗数值 | Data Law 005 + ADR-010 | 运行时断言失败 |
| 禁止绕过 Modifier Pipeline 直接修改属性值 | ADR-011 | 运行时断言失败 |
| HP/MP 直接修改例外——但必须经过 Effect | ADR-011 | 不可审计的数值变更 |

---

## 11. PipelineError——管线领域错误的四种类型

`PipelineError` 枚举定义在 `foundation/error.rs`，共 4 个变体：

```rust
pub enum PipelineError {
    /// 按名称查找阶段时未找到
    StageNotFound { stage: String },
    /// 步骤执行失败（携带阶段名、步骤名、错误详情）
    StepFailed { stage: String, step: String, detail: String },
    /// 管线被外部中止（携带原因）
    Aborted { reason: String },
    /// 上下文数据缺失（携带缺失的 Key）
    MissingContext { key: String },
}
```

- `StageNotFound` — 在 `find_stage()` 或管线定义校验时使用，防止引用了不存在的阶段
- `StepFailed` — 执行器返回 `StepResult::Failure` 时由 `execute_pipeline` 自动构造，同时记录到 `execution_log`
- `Aborted` — `PipelineContext::abort()` 设置 `aborted = true`，`execute_pipeline` 在每次循环入口检查此标志，如果被外部设置则停止执行并返回此错误
- `MissingContext` — 当执行器/步骤试图读取上下文中的某个 Key 但不存在时使用

所有错误都实现了 `thiserror::Error` + `Display`，支持格式化输出。

---

## 12. FailureStrategy 与 Conditional Branching

### FailureStrategy——步骤失败后怎么办

`PipelineStage` 的 `on_failure` 字段决定该阶段中某个步骤失败后的行为：

```rust
pub enum FailureStrategy {
    /// 失败时立即终止整条管线（默认）
    Abort,
    /// 跳过失败的步骤，继续后续步骤
    SkipAndContinue,
    /// 重试 N 次后仍失败才终止
    Retry { max_retries: u8 },
}
```

- `Abort`（默认）— 一个步骤失败整条管线终止。适用于 precondition 检查、资源消耗等不可跳过的步骤
- `SkipAndContinue` — 记录失败到日志后继续执行后续步骤。适用于可选步骤（如特效播放失败不影响核心逻辑）
- `Retry` — 在遇到瞬时性失败时自动重试最多 N 次。每次重试会重新调用执行器

### Conditional Branching——条件分支

`PipelineStep::Conditional` 让管线在运行时根据条件选择不同执行路径：

```rust
PipelineStep::Conditional {
    condition: "has_shield",               // 条件名称，由执行器评估
    if_true: Box::new(PipelineStep::Rule("apply_shield".into())),
    if_false: Box::new(PipelineStep::Rule("apply_damage".into())),
}
```

执行器解释规则：先执行名为 `__condition__` 的虚拟步骤，如果返回 `Success` 则走 `if_true` 分支，否则走 `if_false` 分支。测试中这是怎么模拟的：

```rust
fn condition_true_executor(step_name: &str, ...) -> StepResult {
    if step_name == "__condition__" {
        StepResult::Success   // 条件为真
    } else if step_name == "apply_shield" {
        StepResult::Success   // 走 if_true 分支
    }
}
```

这种设计将条件判别和分支执行都交给执行器（而非引擎），保持引擎通用。

---

## 13. Abort 机制与早期终止

Pipeline 支持两种早期终止机制：

### 1. 通过 PipelineContext 中止

任何持有 `&mut PipelineContext` 的代码可以调用：

```rust
ctx.abort("external cancellation"); // 设置 aborted = true
```

`execute_pipeline` 在每轮循环入口检查 `ctx.aborted`，如果为真则立即返回 `Err(PipelineError::Aborted { .. })`。

### 2. 通过 FailureStrategy::Abort 终止

当一个阶段设置为 `FailureStrategy::Abort`（默认值），且执行器返回 `StepResult::Failure`，引擎立即停止执行并返回 `Err(PipelineError::StepFailed { .. })`。

### 与驾驶员模式的外部队中止

在 CombatPipelineDriver 中，暂停状态（`paused = true`）是一种可控的中止——管线不执行任何步骤但保存状态。当外部输入到达时（`UnitActionComplete` 事件），Observer 恢复驾驶员并跳转到下一个阶段。这是一种"等待式暂停"，不是"异常终止"。

---

## 14. Combat Pipeline Steps 实现详解

`domains/combat/pipeline/steps.rs` 实现战斗回合管线五个步骤的纯函数。每个步骤都只操作数据，不直接与 ECS 调度器连接：

### Step 1: turn_start（重置行动资源）

```rust
pub(crate) fn step_turn_start(
    commands: &mut Commands,
    turn_queue: &TurnQueue,
    ap_query: &mut Query<&mut ActionPoints>,
)
```

- 获取当前单位（`turn_queue.current()`）
- 重置 `ActionPoints`（`ap.reset()`）
- 触发 `OnTurnStart` 领域事件（供 UI/Ability 监听）

### Step 2: phase_check（判定单位行动能力）

```rust
pub(crate) fn step_phase_check(
    turn_queue: &TurnQueue,
    ap_query: &Query<&mut ActionPoints>,
) -> PhaseCheckResult
```

返回枚举：
- `PhaseCheckResult::HasActions` — 单位有可用 AP，进入 UnitAction 等待输入
- `PhaseCheckResult::Idle` — 单位无可用 AP，跳过到 TurnSettlement

PhaseCheck 的结果决定驾驶员的"跳转路径"——不是简单地推进到下一步，而是根据结果跳转到不同的阶段。

### Step 3: unit_action（暂停点——等待输入）

```rust
pub(crate) fn step_unit_action(_commands: &mut Commands, turn_queue: &TurnQueue)
```

这是一个"纯占位步骤"——只记录日志，不执行任何实质性逻辑。驾驶员在此处设置 `paused = true` 并等待外部事件 `UnitActionComplete` 恢复。

### Step 4: turn_settlement（回合结算）

```rust
pub(crate) fn step_turn_settlement(commands: &mut Commands, turn_queue: &TurnQueue)
```

- 触发 `OnTurnEnd` 领域事件（同步触发 Effects tick、Ability cooldowns 等 Observer）
- 触发全局 `TurnEnded` 事件（供其它 Domain 无依赖订阅）

### Step 5: turn_end（单位切换与胜负判定）

```rust
pub(crate) fn step_turn_end(
    commands: &mut Commands,
    turn_queue: &mut TurnQueue,
    combatant_query: &Query<&CombatParticipant>,
    dead_query: &Query<&CombatParticipant, With<Dead>>,
) -> TurnEndResult
```

返回枚举：
- `TurnEndResult::BattleOver` — 仅剩 ≤1 个存活队伍，触发 `OnBattleEnd`
- `TurnEndResult::Continue` — `turn_queue.advance()` 切换到下一个单位，新一轮循环

通过 `Without<Dead>` 过滤器判定存活（而非检查 `is_alive` 字段），这是 ECS 风格的"通过组件存在性推断状态"。

---

## 15. Pipeline 测试全景

Pipeline 引擎的单元测试集中位于 `runtime/pipeline/tests/unit/`，分为 3 个测试文件：

### types_test.rs（111 行）

| 测试 | 验证内容 |
|------|---------|
| `stage_constructed_correctly` | PipelineStage 构造和步数 |
| `stage_default_type_correct` | skippable 标记 |
| `step_constructed_correctly` | System/Rule/SubPipeline 三种类型 |
| `configured_step_constructed_correctly` | Conditional 步骤 |
| `failure_handler_constructed_correctly` | 3 种 FailureStrategy |
| `context_constructed_correctly` | PipelineContext 初始状态 |
| `context_stage_data_read_write` | set_stage_data / get_stage_data |
| `context_halt_state` | abort() 设置 |
| `context_log_recording` | execution_log 追加 |
| `error_message_format_correct` | PipelineError Display |
| `step_failure_error_message` | StepFailed 格式化 |
| `execution_log_count_correct` | ExecutionLogEntry |

### values_test.rs（51 行）

| 测试 | 验证内容 |
|------|---------|
| `definition_constructed_correctly` | PipelineDefinition 构造 |
| `definition_adds_stage` | find_stage 查找 |
| `state_initial_state_correct` | PipelineState 初始状态 |
| `state_advances_step_and_stage` | advance_step / advance_stage |
| `state_reset` | mark_completed |

### executor_test.rs（227 行）

| 测试 | 验证内容 |
|------|---------|
| `execute_empty_pipeline` | 空管线 OK |
| `execute_single_stage_pipeline` | 单阶段管线 |
| `execute_multi_stage_pipeline` | 三阶段管线 |
| `execute_logs_to_context` | 执行日志记录 |
| `failure_stops_pipeline` | 失败终止 + 后续阶段不执行 |
| `failure_returns_error_message` | SkipAndContinue 不失败 |
| `execute_passes_context_parameters` | set_stage_data / get_stage_data |
| `execute_early_termination_pipeline` | abort() 提前终止 |
| `execute_skips_disabled_stage` | skippable 阶段跳过 |
| `execute_conditional_branch` | Conditional 条件分支 |
| `validate_*`（5 个） | validate_pipeline 校验（空 ID/空名/重复/有效） |

### 测试覆盖率特点

- **无 ECS 依赖** — 所有测试都是纯函数测试，不需要 Bevy App 启动
- **模拟执行器** — 通过注入 `fn(&str, &mut PipelineContext, &str) -> StepResult` 来模拟不同行为
- **覆盖 4 条 PipelineError 路径** — StepFailed / Aborted / StageNotFound / MissingContext
- **覆盖所有 FailureStrategy** — Abort / SkipAndContinue / Retry（通过 `mock_selective_executor`）

---

## 16. RuntimePlugin——当前状态与接下来的集成

`capabilities/runtime/plugin.rs` 中的 `RuntimePlugin` 当前是一个"桩"（stub）：

```rust
pub struct RuntimePlugin;

impl Plugin for RuntimePlugin {
    fn build(&self, _app: &mut App) {
        // ── 当前 Phase B 状态 ──────────────────────────────────
        // Runtime 子模块（pipeline/scheduler/registry/command/replay）
        // 领域层纯函数实现已完成...
        // ECS 集成注册时机（将在后续 Phase 实现）：
        //   1. PipelineState Resource → app.init_resource::<PipelineState>()
        //   2. app.add_observer(on_pipeline_started)
        //   3. app.add_observer(on_pipeline_completed)
        //   4. Scheduler Resource + 帧推进 System
    }
}
```

也就是说：
- **领域层已就绪** — PipelineDefinition, PipelineStage, PipelineContext, execute_pipeline, validate_pipeline 等纯函数全部完成
- **Event 类型已定义** — PipelineStarted, PipelineStepCompleted, PipelineFailed, PipelineCompleted
- **ECS 集成待实现** — PipelineRegistry 初始化、PipelineState Resource、Observer 注册、Scheduler 帧推进

这与整个项目的 Phase B→C 过渡一致——Capability 机制层完成，Domain 集成待完成。

---

## 17. Event-Driven Pipeline 触发模式

管线本身是纯数据编排，但"什么启动了一条管线"是通过 Bevy Event/Observer 模式实现的：

### Pipeline 事件

```rust
PipelineStarted    // { pipeline_id, total_stages }
PipelineStepCompleted  // { pipeline_id, stage, step, success }
PipelineFailed     // { pipeline_id, stage, step, reason }
PipelineCompleted  // { pipeline_id, total_steps, failed_steps }
```

### Combat Pipeline 的 Observer 驱动链

```
1. [外部触发] commands.trigger(OnTurnStart { unit })
       ↓
2. [Observer] combat_on_turn_started → 初始化 TurnQueue
       ↓
3. [Driver System] combat_pipeline_driver → 每帧推进 CombatPipelineDriver
       ↓
4. 遇到 PhaseCheck → 决定跳转路径
       ↓
5. 遇到 UnitAction → driver.paused = true，暂停
       ↓
6. [外部输入] commands.trigger(UnitActionComplete { unit })
       ↓
7. [Observer] on_unit_action_complete → driver.paused = false，跳转 TurnSettlement
       ↓
8. Driver 继续推进 → turn_end → 下一个单位或战斗结束
```

### Modifier Pipeline 的脏标记模式

```rust
// 某个 Modifier 变更时触发
commands.trigger(AggregateDirty {
    entity,
    attribute_id: "attr_000001",
    trigger_source: "mod_000042",
});

// AggregatorSystem Observer 监听脏标记
// → 将 (entity, attribute_id) 加入重算队列
// → 下一帧执行 execute_aggregation 四阶段管线
```

这种模式的关键在于**发射事件不立即执行管线**——而是由 Observer/System 在适当的时间（下一帧或空闲时）调度执行。

---

## 18. 对比总结——Pipeline 引擎 vs 业务管线

```ascii
┌──────────────────────────────────────────────────────────────┐
│                   Pipeline 引擎                               │
│  (通用、无业务逻辑)                                            │
│                                                              │
│  PipelineDefinition    → 描述管线结构（阶段+步骤）              │
│  PipelineContext       → 跨阶段数据传递                        │
│  PipelineState         → 运行时状态（执行到哪里了）             │
│  PipelineRegistry      → 注册中心（ECS Resource）              │
│  execute_pipeline()    → 同步执行全部步骤                      │
│  validate_pipeline()   → 校验管线定义合法性                    │
│  PipelineHook trait    → 执行过程中的回调                      │
│  4 PipelineError 变体  → 领域错误类型                          │
│  3 FailureStrategy     → 失败处理策略                          │
│  Conditional step      → 条件分支                             │
└──────────────────────────────────────────────────────────────┘
          ▲                          ▲              ▲
          │ 继承                     │ 注册          │ 使用
          │                          │              │
┌─────────┴──────────┐ ┌─────────────┴──┐ ┌────────┴──────────┐
│ Ability Pipeline   │ │ Modifier       │ │ Combat Turn       │
│                    │ │ Pipeline       │ │ Pipeline          │
│ try_activate() →   │ │                │ │                   │
│ 6 阶段：           │ │ execute_       │ │ CombatPipeline-   │
│ check → cost →     │ │ aggregation() →│ │ Driver 逐帧驱动  │
│ target → exec →    │ │ 4 阶段：        │ │ 5 阶段：          │
│ apply → cue        │ │ calc →         │ │ TurnStart →       │
│                    │ │ aggregate →    │ │ PhaseCheck →      │
│                    │ │ finalize →     │ │ UnitAction →      │
│                    │ │ validate       │ │ TurnSettlement →  │
│                    │ │                │ │ TurnEnd           │
│                    │ │ 添加触发模式：  │ │                   │
│                    │ │ AggregateDirty │ │ 驾驶员模式        │
│                    │ │ 事件驱动重算   │ │ 暂停/恢复         │
└────────────────────┘ └────────────────┘ └───────────────────┘
```

---

## 19. 阅读导引——如果只想看一部分

| 你的目标 | 先看什么 | 再看什么 |
|---------|---------|---------|
| **理解 Pipeline 是什么** | §1（核心抽象） | §4（执行器） |
| **理解 Combat Turn 怎么跑** | §6（驾驶员） | §14（步骤实现）+ §7（时序图） |
| **看 Pipeline 引擎源码** | §9（文件索引）→ `types.rs` + `executor.rs` | §15（测试覆盖） |
| **理解数据 Schema** | §2（Definition/Pipeline）→ `pipeline_schema.md` | §11（PipelineError）|
| **理解三种 FailureStrategy** | §12（FailureStrategy） | executor_test.rs `failure_*` 测试 |
| **理解 Conditional 分支** | §12（Conditional Branching） | executor_test.rs `execute_conditional_branch` |
| **理解 Modifier Pipeline 的 AggregateDirty** | ADR-011 | `aggregator/mechanism/pipeline.rs` |
| **理解 Content Loading Pipeline** | ADR-047 | `content_plugin.rs`（加载流程）|
| **看测试验证范围** | §15（测试全景） | executor_test.rs（227 行） |
