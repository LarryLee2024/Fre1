---
id: 09-planning.infrastructure-integration-plan
title: 基础设施接入规划 — 消灭死代码，让每行代码都有消费者
status: active
owner: feature-developer
created: 2026-06-18
updated: 2026-06-20

> **Phase pipeline→combat review + test coverage**: ✅ Complete (2026-06-20)
> - H1: PipelineRegistry moved to core — dependency direction restored
> - M2: CombatPipelineDriver fields made private
> - M3: COMBAT_TURN_PIPELINE_ID constant eliminated duplication
> - M4: check_team_elimination extracted as standalone function
> - Test: 6 unit tests for pipeline_def, 14 integration tests for pipeline_step
> - All 1451 tests pass
tags:
  - planning
  - infrastructure
  - integration
  - technical-debt
---

# 基础设施接入规划 — 消灭死代码，让每行代码都有消费者

> **审计依据**: 2026-06-18 基础设施使用审计
> **原则**: 高度工程化项目，所有已设计的基础设施必须有消费者。第一选项是接入，不是删除。
> **目标**: 将基础设施使用率从 19% 提升到 80%+

---

## 0. 审计结果速览

| 层 | 系统总数 | 已使用 | 未使用 | 使用率 |
|----|---------|--------|--------|--------|
| Shared (L0) | 4 | 1 | 3 | 25% |
| Capabilities (L1) | 15 | 4 | 11 | 27% |
| Infrastructure (L2) | 5 | 0 | 5 | 0% |
| Cross-cutting | 3 | 0 | 3 | 0% |
| **总计** | **27** | **5** | **22** | **19%** |

---

## 1. Shared 层 (L0) 接入规划

### 1.1 `shared::ids` — 统一 ID 类型

**现状**: 定义了 14 个 ID 类型（TagId, AbilityId, SpellId 等），但各域自建了 FactionId, SpellDefId, QuestDefId 等。

**接入策略**: 统一 ID 来源，各域使用 shared::ids 中的类型。

| 域 | 当前自定义 ID | 应迁移到 | 优先级 |
|----|--------------|---------|--------|
| faction | `faction::components::FactionId` | `shared::ids::FactionId` | P0 |
| spell | `spell::components::SpellDefId` | `shared::ids::SpellId` | P0 |
| quest | `quest::components::QuestDefId` | `shared::ids::QuestId` | P0 |
| combat | `combat::components::TeamId` | `shared::ids` 新增 `TeamId` | P1 |
| progression | `ClassId, TalentId, SubclassId` | `shared::ids` 新增对应类型 | P1 |
| party | `BondDefId, FormationDefId` | `shared::ids` 新增对应类型 | P1 |
| camp_rest | `CampEventId` | `shared::ids` 新增 `CampEventId` | P1 |
| terrain | 已用 `infra::registry::DefinitionId` | 保持（registry 专用） | — |
| crafting | 使用 `String` 作为 ID | `shared::ids` 新增 `RecipeId` | P1 |
| economy | 使用 `String` 作为 ID | `shared::ids` 新增 `ShopId` | P1 |

**执行步骤**:
1. 为缺失的 ID 类型在 `shared/ids/types.rs` 中新增 `define_string_id!` 声明
2. 各域 components.rs 中删除自定义 ID，改为 `use crate::shared::ids::*`
3. 更新所有引用处（含测试）
4. 确保 `cargo test` 无回归

**预计工作量**: ~2 天（机械性替换，需仔细测试）

---

### 1.2 `shared::random::SeededRng` — 确定性 RNG

**现状**: 定义了基于 ChaCha12 的确定性 RNG，但无任何消费者。

**接入策略**: 接入到需要确定性随机的系统。

| 接入点 | 用途 | 优先级 |
|--------|------|--------|
| `infra::replay` rng_sync_system | 回放时同步 RNG 状态 | P0 |
| `infra::save` 加载时 | 存档恢复后重建 RNG | P0 |
| `combat` 伤害浮动 | 暴击判定、伤害随机偏移 | P1 |
| `terrain` 地形生成 | 程序化地形变化 | P2 |
| `crafting` 制作成功率 | 技能检定的骰子投掷 | P2 |

**执行步骤**:
1. 在 `replay` 模块的 `rng_sync_system` 中使用 `SeededRng` 替代 `rand::thread_rng()`
2. 在 `save` 模块的加载流程中重建 `SeededRng` 状态
3. 在 combat 域的伤害计算中注入 `SeededRng` 作为 Resource
4. 确保所有随机调用通过 `SeededRng` 而非 `thread_rng()`（Replay 一致性要求）

**预计工作量**: ~1 天

---

### 1.3 `shared::time::GameTime` — 游戏时间

**现状**: 定义了 frame/turn 双轴时间，但仅 camp_rest 在注释中提及。

**接入策略**: 作为 Resource 注入到所有需要时间感知的系统。

| 接入点 | 用途 | 优先级 |
|--------|------|--------|
| `combat` 回合计时 | Turn 状态机的 frame 计数 | P0 |
| `effect` 持续时间 | Effect 的 tick_durations 依赖帧计数 | P0 |
| `ability` 冷却计时 | CooldownEntry.tick() 需要当前 turn | P0 |
| `spell` 专注维持 | Concentration 的 elapsed_rounds | P1 |
| `terrain` 地形效果持续 | HazardZone 的持续帧数 | P1 |
| `replay` 帧同步 | ReplayFrame 中记录 GameTime | P1 |

**执行步骤**:
1. 在 `AppPlugin` 中初始化 `GameTime` 为 Resource
2. 在 `combat::turn_system` 中每回合调用 `game_time.advance_turn()`
3. 在 `effect::tick_durations` 中使用 `game_time.frame()` 判断过期
4. 在 `ability::tick_cooldowns` 中使用 `game_time.turn()` 减少冷却

**预计工作量**: ~1 天

---

### 1.4 `shared::error::ErrorContext` — 错误上下文

**现状**: 定义了错误追踪上下文，但无消费者。

**接入策略**: 接入到所有可能失败的跨域操作。

| 接入点 | 用途 | 优先级 |
|--------|------|--------|
| `infra::pipeline` 执行失败 | Pipeline 执行失败时附加上下文 | P1 |
| `infra::save` 读写失败 | 存档操作失败时附加上下文 | P1 |
| `combat` Effect 应用失败 | Effect 被免疫/抵抗时记录原因 | P2 |
| `spell` 施法失败 | SpellCastRequest 被拒绝时记录原因 | P2 |

**执行步骤**:
1. 在 `infra::pipeline` 的 `StepResult::Failed` 中携带 `ErrorContext`
2. 在 `infra::save` 的 `SaveError` 事件中携带 `ErrorContext`
3. 在 combat/spell 的错误事件中携带 `ErrorContext`

**预计工作量**: ~0.5 天

---

## 2. Capabilities 层 (L1) 接入规划

### 2.1 已使用的 4 个 Capability

| Capability | 当前使用者 | 接入深度 |
|-----------|-----------|---------|
| attribute | tactical (movement) | 🟡 仅 AttributeContainer |
| modifier | tactical (movement) | 🟡 仅 ModifierContainer |
| tag | tactical (movement + facade) | 🟡 仅 TagHierarchy + TagSet |
| effect | combat (integration/effect) | 🟢 较完整（facade + system_param + tests） |

**增强计划**: 将这 4 个 capability 的使用扩展到更多域。

---

### 2.2 未使用的 11 个 Capability — 接入规划

#### P0: 核心战斗管线必须接入

| Capability | 接入域 | 接入方式 | 优先级 |
|-----------|--------|---------|--------|
| **ability** | combat | CombatAbility 组件引用 AbilityDef，激活时走 capability::ability 生命周期 | P0 |
| **condition** | combat/spell | 施法前检查 Condition（沉默/束缚/专注），Effect 应用前检查 Condition | P0 |
| **trigger** | combat | TurnStarted/TurnEnded/DamageTaken 等事件触发 trigger 评估 | P0 |
| **event** | combat | 用 EventBus 替代域自定义 EventWriter，统一事件分发 | P0 |

**具体接入方案**:

**ability → combat**:
```
CombatParticipant 新增 ability_container: ActiveAbilityContainer
combat_system 在回合开始时调用 tick_cooldowns()
施法时调用 try_activate() 检查 Ready 状态
```

**condition → combat/spell**:
```
施法前: check_condition(silence_condition) → 失败则拒绝施法
Effect 应用前: check_condition(immunity_condition) → 失败则跳过
```

**trigger → combat**:
```
TurnStarted 事件 → evaluate(trigger_entries) → 触发 OnTurnStart 效果
DamageTaken 事件 → evaluate(trigger_entries) → 触发反击/护盾
```

**event → combat**:
```
替换现有的 EventWriter<CombatEvent> 为 EventBus::publish()
下游 Observer 订阅 EventBus 的 EventTag
```

---

#### P1: 完整战斗体验需要接入

| Capability | 接入域 | 接入方式 | 优先级 |
|-----------|--------|---------|--------|
| **targeting** | combat/spell | SpellDef 引用 TargetingDef，施法时执行 select_targets() | P1 |
| **execution** | combat | Effect 执行时走 ExecutionEngine（伤害/治疗/护盾/控制） | P1 |
| **gameplay_context** | combat | 每次攻击构建 GameplayContextData，贯穿全链路 | P1 |
| **aggregator** | combat | 属性聚合：base + modifier → final value，每回合重算 | P1 |

**具体接入方案**:

**targeting → spell**:
```
SpellDef 新增 targeting: TargetingDef
施法时: select_targets(&spell.targeting, candidates, context)
```

**execution → combat**:
```
Effect 应用时: ExecutionEngine::execute(effect, context) → StepResult
替代当前直接修改 AttributeValue 的方式
```

**gameplay_context → combat**:
```
攻击时: ContextBuilder::new(CombatAttack, frame).source(...).target(...).build()
全链路传递 GameplayContextData
```

**aggregator → combat**:
```
每回合开始: aggregator.recalculate_all(attribute_container, modifier_container)
输出 AggregationResult 供其他系统读取
```

---

#### P2: 高级特性需要接入

| Capability | 接入域 | 接入方式 | 优先级 |
|-----------|--------|---------|--------|
| **cue** | combat/spell | Effect 应用时触发 Cue（VFX/SFX/Popup） | P2 |
| **spec** | combat | Ability 激活时创建 AbilitySnapshot，Effect 使用 EffectSnapshot | P2 |
| **stacking** | combat | 多个相同 Buff 叠加时走 Stacking 决策 | P2 |

---

### 2.3 Capability 接入时间线

```
Phase E (✅ 已完成):
  ├── ✅ ability    → combat (P0)
  ├── ✅ condition  → combat/spell (P0)
  ├── ✅ trigger    → combat (P0)
  └── ✅ event      → combat (P0)

Phase F (✅ 已完成):
  ├── ✅ targeting  → spell (P1)
  ├── ✅ execution  → combat (P1)
  ├── ✅ gameplay_context → combat (P1)
  └── ✅ aggregator → combat (P1)

Phase G (音频/UI 后):
  ├── cue        → combat/spell (P2)
  ├── spec       → combat (P2)
  └── stacking   → combat (P2)
```

---

## 3. Infrastructure 层 (L2) 接入规划

### 3.1 `infra::pipeline` — 战斗管线引擎

**现状**: PipelineRegistry + PipelineHook + ExecutionLogHook，完全未接入。

**接入策略**: 替代 combat 域当前的手动 system 调度。

| 接入点 | 用途 | 优先级 |
|--------|------|--------|
| `combat` 战斗流程 | 定义 CombatPipeline（输入→判定→执行→结算） | P1 |
| `spell` 施法流程 | 定义 SpellPipeline（选择→校验→施放→效果） | P1 |

**执行步骤**:
1. 在 `combat/plugin.rs` 中注册 CombatPipeline 定义
2. 将当前 combat_system 拆分为 Pipeline 的 Step
3. 使用 PipelineHook 记录执行日志（调试用）
4. Replay 模式下重放 Pipeline 执行

**预计工作量**: ~2 天

---

### 3.2 `infra::replay` — 回放系统

**现状**: RecordingSession + PlaybackSession + FrameCounter，仅 camp_rest 引用了 FrameCounter。

**接入策略**: 接入到所有战斗流程。

| 接入点 | 用途 | 优先级 |
|--------|------|--------|
| `combat` 战斗录制 | 每场战斗创建 RecordingSession，记录输入 | P0 |
| `combat` 战斗回放 | 加载存档时创建 PlaybackSession，重放输入 | P0 |
| `shared::random` | SeededRng 与 Replay 同步 | P0 |

**执行步骤**:
1. 在 combat 启动时根据模式选择 Recording/Playback
2. 将 combat 的输入（玩家决策）录制到 ReplayFrame
3. 回放时从 ReplayFrame 读取输入，替换真实输入
4. 每帧计算 checksum，验证一致性

**预计工作量**: ~2 天

---

### 3.3 `infra::save` — 存档系统

**现状**: SaveManager + EntityRemapper + SaveRequest/LoadRequest，完全未接入。

**接入策略**: 接入到游戏状态持久化。

| 接入点 | 用途 | 优先级 |
|--------|------|--------|
| `combat` 战斗状态 | 保存/恢复战斗场景（单位位置、血量、Buff） | P1 |
| `party` 队伍状态 | 保存/恢复队伍配置（成员、装备、物品） | P1 |
| `progression` 成长 | 保存/恢复角色等级、技能、天赋 | P1 |
| `quest` 任务 | 保存/恢复任务进度 | P1 |

**执行步骤**:
1. 定义 SaveFormat schema（哪些 Component 需要序列化）
2. 在 SaveManager 中注册所有需要持久化的 Component 类型
3. 实现 on_save_request 系统：遍历标记 Component，序列化到文件
4. 实现 on_load_request 系统：从文件反序列化，用 EntityRemapper 恢复 Entity 引用

**预计工作量**: ~3 天

---

### 3.4 `infra::input` — 输入抽象

**现状**: InputAction + InputMap + InputState，完全未接入。

**接入策略**: 接入到游戏输入处理。

| 接入点 | 用途 | 优先级 |
|--------|------|--------|
| `tactical` 移动输入 | WASD/方向键移动光标 | P1 |
| `combat` 操作输入 | 确认/取消/菜单按键 | P1 |
| `spell` 目标选择 | 鼠标点击选择目标 | P2 |

**执行步骤**:
1. 在 AppPlugin 中配置 InputMap（键盘/鼠标映射）
2. 在 tactical_system 中读取 InputState 处理移动
3. 在 combat_system 中读取 InputState 处理操作确认
4. Replay 模式下用 ReplayFrame 中的 InputAction 替代真实输入

**预计工作量**: ~1 天

---

## 4. Cross-cutting 层接入规划

### 4.1 `content` — 内容加载

**现状**: 基础管线已实现（ADR-047），SpellDef/CueDef 已 Asset 化。

**接入计划**: 扩展到所有 Def 类型，创建完整 RON 配置。

| 步骤 | 内容 | 优先级 |
|------|------|--------|
| 1 | 其他 Def 类型添加 Asset+Serialize+Deserialize | P0 |
| 2 | ContentPlugin 注册所有 Asset 类型 | P0 |
| 3 | 创建 assets/config/ 下所有桶的示例 RON 文件 | P1 |
| 4 | 添加热重载 Observer | P1 |

**预计工作量**: ~2 天

---

### 4.2 `tools` — 开发工具

**现状**: 骨架空壳（DevToolsPlugin）。

**接入计划**: 集成 bevy-inspector-egui 调试面板。

| 步骤 | 内容 | 优先级 |
|------|------|--------|
| 1 | 在 DevToolsPlugin 中初始化 bevy-inspector-egui | P2 |
| 2 | 注册所有 Component/Resource 到 Inspector | P2 |
| 3 | 添加自定义调试面板（战斗状态、属性值、Effect 列表） | P2 |

**预计工作量**: ~1 天

---

### 4.3 `modding` — Mod 支持

**现状**: 骨架空壳（ModdingPlugin）。

**接入计划**: 依赖 Content 系统，支持外部 RON 配置覆盖。

| 步骤 | 内容 | 优先级 |
|------|------|--------|
| 1 | 定义 Mod 加载优先级（内置 < Mod） | P2 |
| 2 | 在 Content 加载管线中支持 Mod 目录扫描 | P2 |
| 3 | 实现 Mod 配置覆盖逻辑 | P3 |

**预计工作量**: ~2 天

---

## 5. 执行优先级总表

| 优先级 | 任务 | 预计工时 | 前置依赖 |
|--------|------|---------|---------|
| **P0** | shared::ids 统一 ID 策略 | ✅ 已完成 | 无 |
| **P0** | SeededRng → GameRng Resource + SharedPlugin 注册 | ✅ 已完成 | 无 |
| **P0** | GameTime → Resource + PreUpdate system | ✅ 已完成 | 无 |
| **P0** | ability/condition/trigger/event → combat | ✅ 已完成 | 无 |
| **P0** | replay → combat 录制/回放 | ⏳ 待执行（依赖 P0 capability） | P0 capability |
| **P0** | content: 其他 Def 类型 Asset 化 | 2 天 | 无 |
| **P1** | ErrorContext → pipeline/save | 0.5 天 | 无 |
| **P1** | targeting/execution/gameplay_context/aggregator → combat | ✅ 已完成 | P0 capability |
| **P1** | pipeline → combat 流程 (CombatPipelineDriver) | ✅ 已完成 | P0 capability |
| **P1** | save → combat/party/progression | 3 天 | shared::ids |
| **P1** | input → tactical/combat | 1 天 | 无 |
| **P1** | content: 完整 RON 配置 + hot-reload | 2 天 | P0 content |
| **P2** | cue/spec/stacking → combat | 2 天 | P1 capability |
| **P2** | tools: Inspector 调试面板 | 1 天 | 无 |
| **P2** | modding: Mod 加载支持 | 2 天 | P1 content |
| | **总计** | **~28 天** | |

---

## 6. 预期成果

| 指标 | 当前 | 目标 |
|------|------|------|
| 基础设施使用率 | 19% (5/27) | 85%+ (23/27) |
| Shared L0 使用率 | 25% (1/4) | 100% (4/4) |
| Capabilities L1 使用率 | 27% (4/15) | 100% (15/15) |
| Infrastructure L2 使用率 | 0% (0/5) | 100% (5/5) |
| Cross-cutting 使用率 | 0% (0/3) | 100% (3/3) |

---

## 7. 风险与缓解

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| capability 接入后域逻辑需要大幅重构 | 中 | 高 | 渐进式接入，每次只接入一个 capability |
| shared::ids 迁移导致大量编译错误 | 低 | 中 | 机械性替换，cargo check 逐步验证 |
| pipeline 接入后性能下降 | 低 | 中 | Benchmark 对比，必要时优化 |
| replay 接入后确定性被破坏 | 中 | 高 | 每次接入后运行 roundtrip 测试验证 |

---

*本文档是基础设施接入的 single source of truth。每个接入任务完成后，对应章节应更新为 ✅ 状态。*
