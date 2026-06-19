---
id: 09-planning.Phase-post-M1-execution-plan
title: "Phase Post-M1 执行计划 — M1 后未完成任务与下一步行动"
status: active
owner: feature-developer
created: 2026-06-17
updated: 2026-06-18 (D-9 ✅ + C-4 ✅ Replay 桥接层 + C-5 ✅ Save 桥接层 + 全角色评审)
tags:
  - planning
  - implementation
  - post-m1
  - backlog
  - phase-c
  - phase-d
  - phase-e
---

# Phase Post-M1 执行计划 — M1 后未完成任务与下一步行动

> **本文档承接 `Phase-C-D-execution-plan.md`**
> 撰写: Sisyphus | 日期: 2026-06-17
> 范围: M1 里程碑之后的所有未开始/骨架状态任务

---

## 0. 当前状态摘要（M1 完成后）

| 维度 | 状态 | 详情 |
|------|------|------|
| Phase A (Shared) | ✅ 完成 | 1,695 行 |
| Phase B (Capabilities) | ✅ ≈85% | 21,431 行，已注册但零业务域调用 |
| Phase C-1 (Pipeline) | ✅ 完成 | ~250 行，3 测试，review PASS |
| Phase C-2 (Input) | ✅ 完成 | ~356 行，22 测试，review PASS |
| Phase D-1 (Tactical) | ✅ 完成 | ~782 行，36 测试，review PASS |
| **M1 里程碑** | ✅ 通过 | 742 lib tests pass |
| **Capabilities 集成验证** | ✅ 完成 | Observer System 打通 Tag/Attribute/Modifier 管线，742 tests pass |
| **Phase C-3 — Registry** | ✅ 完成 | ~750 行，23 测试，build+test PASS |
| **Phase C-4 — Replay** | ✅ 完成 | 桥接层实现：resources/systems/events/api + plugin，5 文件 ~145 行 |
| **Phase C-5 — Save** | ✅ 完成 | 桥接层实现：resources/events/systems + plugin + tests，5 文件 ~260 行，19 tests |
| **Phase D-2 — Terrain** | ✅ 完成 | ~700 行，9 测试，build+test PASS |
| **Phase D-3 ~ D-15** | 🟡 骨架 | 13 个 Domain 均为 `mod.rs` + `plugin.rs` + 空 `tests/` |
| **Phase E ~ H** | 🔴 未开始 | — |
| **@refactor-guardian 技术债扫描** | ✅ 完成 | `docs/11-refactor/debt-inventory-2026-06-17.md`，P0 可见性修复已完成，P1 待处理 |

---

## 1. 核心未解决风险

**Capabilities 系统 21k 行代码从未被任何业务域在运行态调用过。**

Tactical 的 `integration.rs` 定义了 `MovementType → TagId` 映射，但没有任何 System 实际通过 Capabilities 管线做任何事情。这意味着：

- Tag 模块的查询/过滤逻辑未验证
- Attribute 的读/写/消耗管线未验证
- Modifier 的应用/聚合/清除逻辑未验证
- Execution/Effect 的完整管线未验证
- GameplayContext 的载荷构建/传递未验证

**这是整个项目最大的技术风险。** 如果 Capabilities 存在设计缺陷，越晚发现修复成本越高。

---

## 2. 战略选项分析

| 选项 | 内容 | 工作量 | 风险降低 | 业务价值 | 推荐度 |
|------|------|--------|---------|---------|-------|
| **A: Capabilities 集成验证** | 编写 Tactical → Capabilities 集成测试或验证 System，实际调用 Tag/Attribute/Modifier/Execution 管线 | ~1 次会话 | ⭐⭐⭐ 最高 | 低（无用户可见功能） | 🥇 **首选** |
| **B: C-3 Registry** | 实现 GenericRegistry + 冲突检测，为内容加载做准备 | ~1 次会话 | ⭐（不影响 Capabilities） | 中（基础设施） | 🥉 |
| **C: D-2 Terrain** | 实现地形系统（自然地形的扩展），会自然用到 Capabilities Modifier | ~2 次会话 | ⭐⭐（间接验证） | 高（直接影响游戏性） | 🥈 |
| **D: E-1 Combat** | 第一个战斗域，重度依赖 Capabilities | ~3+ 次会话 | ⭐⭐ | 高 | 时机未到 |
| **E: 技术债清理** | ✅ @refactor-guardian 已扫描完成，输出债务清单 | ~1 次会话 | ⭐（代码健康） | 低 | ✅ 已完成 |

### 2.1 推荐路径

```
当前 (M1 完成)
    │
    ├── ✅ 并行任务 A: @refactor-guardian 技术债扫描（已完成）
    │   └── 输出: docs/11-refactor/debt-inventory-2026-06-17.md
    │
    ├── ✅ 已完成: Capabilities 集成验证 (D-1.5)
    │   ├── 成果: `on_compute_move` Observer System 已实现
    │   ├── 验证管线: Tag 查询 → Attribute 读取 → Modifier 应用
    │   ├── 设计模式: `integration.rs` Facade 封装所有 Capabilities 交互
    │   ├── 退出条件: ✅ 通过 — TagHierarchy + TagSet / AttributeContainer / ModifierContainer 全管线触及
    │   ├── 代码: 4 files, ~280 行新增 (integration.rs/events.rs/movement_system.rs/plugin.rs)
    │   └── 下一站: 建议 @code-reviewer 审查 + @test-guardian 补测试
    │
    ├── ✅ 已完成: C-3 Registry 注册中心
    │   ├── 成果: `RegistryBucket<T>` 泛型版本化存储 + `DefinitionRegistry` Resource
    │   ├── `IdAllocator` 前缀分配/回收复用 + `RegistryValidation` 交叉引用校验
    │   ├── 代码: 3 files, ~750 行新增 (registry.rs/resolver.rs/plugin.rs)
    │   └── 退出条件: ✅ 通过 — 774 tests, 0 failed, 0 build errors
    │
    ├── ✅ 已完成: D-2 Terrain 地形域
    │   ├── 成果: TerrainType/Concealment/SurfaceType 枚举 + TileProperties/SurfaceOverride Component
    │   ├── rules: 移动消耗(6 移动类别×10 地形) + 遮蔽度修正纯函数
    │   ├── systems: terrain_effect/surface/hazard 三个 Observer System
    │   ├── 代码: ~12 files, ~700 行新增
    │   └── 退出条件: ✅ 通过 — 774 tests, 0 failed, 0 build errors
    │
    └── 第三阶段（审查通过后）:
        ├── C-4 Replay（回放系统骨架）
        ├── C-5 Save（存档系统骨架）
        └── 或 D-3 ~ D-9（剩余业务域，按优先级）
```

### 2.2 立即行动项

| # | 行动 | 执行人 | 前置 | 建议工期 |
|---|------|--------|------|---------|
| 1 | **决定是否采纳此方案** | 项目负责人 | 无 | — |
| 2 | ~~@refactor-guardian 技术债扫描~~ | ~~@refactor-guardian~~ | ~~决定后立即启动~~ | ✅ 已完成 |
| 3 | ~~Capabilities 集成验证设计~~ | ~~@architect + @feature-developer~~ | ~~方案确定~~ | ✅ 已完成 |
| 4 | ~~Capabilities 集成验证实现~~ | ~~@feature-developer~~ | ~~设计完成~~ | ✅ 已完成 |
| 5 | 验证结果评审 (Capabilities 集成) | @code-reviewer | 实现完成 | ✅ 已完成 |
| 6 | 测试覆盖 (Capabilities 集成) | @test-guardian | 验证通过 | ✅ 已完成 |
| 7 | ~~C-3 Registry 实现~~ | ~~@feature-developer~~ | ~~C-3 Registry 完成~~ | ✅ 已完成 |
| 8 | ~~D-2 Terrain 实现~~ | ~~@feature-developer~~ | ~~D-2 Terrain 完成~~ | ✅ 已完成 |
| 9 | 代码审查 (C-3 Registry + D-2 Terrain) | @code-reviewer | Registry + Terrain 完成 | ✅ 已完成 |
| 10 | 测试审查 (D-2 Terrain) | @test-guardian | Terrain 完成 | ✅ 已完成 |
| 11 | ~~C-4 Replay 桥接层实现~~ | ~~@feature-developer~~ | ~~ADR-041 + Core replay 已就绪~~ | ✅ 已完成 |
| 12 | 测试覆盖 (C-4 Replay 桥接层) | @test-guardian | C-4 实现完成 | ⏳ 待执行 |
| 13 | 代码审查 (C-4 Replay 桥接层) | @code-reviewer | C-4 测试通过 | ⏳ 待执行 |
| 14 | 技术债增量扫描 | @refactor-guardian | C-4 审查完成 | ⏳ 待执行 |

---

## 3. 未开始任务详情

### 3.1 Phase C-3: Registry 注册中心

> **目标**: 实现 GenericRegistry，为 Content 层配置加载做准备
> **估算**: ~750 行，3-4 个文件
> **主要执行者**: @feature-developer
> **状态**: ✅ 已完成 (build+test PASS, 774 tests)

#### 前置文档确认

| 文档 | 位置 | 责任人 | 状态 |
|------|------|--------|------|
| ADR-013 | `docs/01-architecture/10-capability-system/ADR-013-registry-hotreload.md` | @architect | ✅ 已批准 |
| registry_schema | `docs/04-data/infrastructure/registry_schema.md` | @architect → @data-architect | ✅ v2 已对齐 ADR-013 (Handle 方案，@architect 完成) |

> **schema 对齐说明**: v1 使用直接值存储 (`HashMap<Id, T>`)，v2 迁移至 Handle 间接存储 (`HashMap<Id, Handle<T>>`)，对齐 ADR-013 的两层架构设计。变更内容：Asset 层注册、RegistryBucket 泛型设计、Snapshot 热重载保护、OnDefinitionReloaded 事件。

#### 交付清单

| # | 任务 | 输出文件 | 责任人 |
|---|------|---------|--------|
| C3-1 | Registry trait + GenericRegistry | `src/infra/registry/registry.rs` | @feature-developer |
| C3-2 | RegistryPlugin 更新 | `src/infra/registry/plugin.rs` | @feature-developer |
| C3-3 | 冲突检测 + ID 分配 | `src/infra/registry/resolver.rs` | @feature-developer |
| C3-T | 单元测试 | `src/infra/registry/tests/` | @test-guardian |
| C3-R | 代码审查 | `docs/10-reviews/registry-review.md` | @code-reviewer |

---

### 3.2 Phase C-4: Replay 回放桥接层

> **目标**: 将 Core 层 replay 能力（RecordingSession、PlaybackSession、DeterministicRng）桥接到 Bevy ECS
> **当前状态**: ✅ 已完成 — 5 个文件 ~145 行，cargo check 零错误零警告，914 tests passed
> **估算**: ~145 行，5 个文件
> **主要执行者**: @feature-developer
> **设计依据**: ADR-041（回放确定性与架构）— 已由 @architect 批准

#### 前置文档确认

| 文档 | 位置 | 责任人 | 状态 |
|------|------|--------|------|
| ADR-041 | `docs/01-architecture/40-cross-cutting/ADR-041-replay-determinism.md` | @architect | ✅ 已批准 |
| replay_schema | `docs/04-data/infrastructure/replay_schema.md` | @data-architect | ✅ 已稳定 |
| replay_architecture | `docs/04-data/foundation/replay_architecture.md` | @data-architect | ✅ 已稳定 |
| Core replay 模块 | `src/core/capabilities/runtime/replay/` | @feature-developer | ✅ 已实现（24 tests） |

#### 架构差异说明（相比初步设计）

实际实现基于 Capabilities 层已存在的完整 replay 模块（~800 行 + 24 tests），采用**桥接模式**而非重新实现：

| 初步设计 | 实际实现 | 原因 |
|----------|---------|------|
| 独立实现 ReplayEvent/Recorder/Player | 包装 Core 层类型为 Bevy Resource | 避免重复实现已存在的能力 |
| 4 个独立文件（event/recorder/player/plugin） | 4 个功能文件（resources/systems/events/api）+ plugin | 结构更清晰，API 统一出口 |
| 使用 `add_event` 注册 | Observer-based Events + `commands.trigger()` | 对齐 Bevy 0.19 事件系统 |

#### 交付清单

| # | 任务 | 输出文件 | 责任人 | 状态 |
|---|------|---------|--------|------|
| C4-1 | Bevy Resource 包装（DeterministicRng, ReplayModeGuard, 会话资源, FrameCounter） | `src/infra/replay/resources.rs` | @feature-developer | ✅ 完成 |
| C4-2 | Infra 层事件 re-export | `src/infra/replay/events.rs` | @feature-developer | ✅ 完成 |
| C4-3 | 录制/回放生命周期 System（帧管理、RNG 同步） | `src/infra/replay/systems.rs` | @feature-developer | ✅ 完成 |
| C4-4 | 对外统一 re-export（已合并到 mod.rs，ADR-046） | `src/infra/replay/mod.rs` | @feature-developer | ✅ 完成 |
| C4-5 | ReplayPlugin 注册 + mod 声明 | `src/infra/replay/plugin.rs`, `mod.rs` | @feature-developer | ✅ 完成 |
| C4-T | 单元/集成测试 | `src/infra/replay/tests/` | @test-guardian | ⏳ 待编写 |
| C4-R | 代码审查 | `docs/10-reviews/replay-bridge-review.md` | @code-reviewer | ⏳ 待审查 |

---

### 3.3 Phase C-5: Save 存档桥接层

> **目标**: 实现最小存档系统桥接层，提供 Resource/Event/Observer 基础设施
> **当前状态**: ✅ 已完成 — 5 文件 ~260 行，19 tests，cargo build 零错误零警告
> **估算**: ~260 行，5 个文件
> **主要执行者**: @feature-developer
> **设计依据**: ADR-042（存档持久化策略）— 已由 @architect 批准

#### 前置文档确认

| 文档 | 位置 | 责任人 | 状态 |
|------|------|--------|------|
| ADR-042 | `docs/01-architecture/40-cross-cutting/ADR-042-save-persistence.md` | @architect | ✅ 已批准 |
| save_architecture | `docs/04-data/foundation/save_architecture.md` | @data-architect | ✅ 已稳定 |

#### 架构差异说明

与 C-4 Replay 不同，Save 没有现有的 Core 层模块。采用**直接实现桥接层骨架**策略：

| 初步设计 | 实际实现 | 原因 |
|----------|---------|------|
| SaveData / SaveHeader 结构 | `events.rs` 中的 Event 类型 | observer-based 事件驱动 save/load |
| SaveManager + serializer + migration | `resources.rs` 中 SaveManager + AutoSaveConfig + EntityRemapper | 最小实现，Per-Feature 序列化后续迭代 |
| Per-Feature SaveLoad trait | Observer pattern (On<SaveRequest> / On<LoadRequest>) | 对齐 Bevy 0.19 事件系统 |

#### 交付清单

| # | 任务 | 输出文件 | 责任人 | 状态 |
|---|------|---------|--------|------|
| C5-1 | SaveManager, AutoSaveConfig, EntityRemapper Resource | `src/infra/save/resources.rs` | @feature-developer | ✅ 完成 |
| C5-2 | SaveRequest, LoadRequest, SaveCompleted, LoadCompleted, SaveError 事件 | `src/infra/save/events.rs` | @feature-developer | ✅ 完成 |
| C5-3 | on_save_request, on_load_request Observer | `src/infra/save/systems.rs` | @feature-developer | ✅ 完成 |
| C5-4 | SavePlugin 注册 + mod 声明 | `src/infra/save/plugin.rs`, `mod.rs` | @feature-developer | ✅ 完成 |
| C5-T | 单元/集成/不变量测试 | `src/infra/save/tests/` | @test-guardian | ✅ 完成 (19 tests) |
| C5-R | 代码审查 | `docs/10-reviews/save-bridge-review.md` | @code-reviewer | ✅ 完成 (PASS) |

---

### 3.4 Phase D-3 ~ D-15: 13 个业务域骨架

> **当前状态**: 🟡 进步中 — D-2 Terrain ✅, D-9 Turn ✅, D-3 Faction ✅, D-13 Narrative ✅
> **说明**: 剩余 11 个域按业务优先级逐个实现

#### 业务域清单

| 编号 | 域 | 说明 | 优先级 | 依赖 | 状态 |
|------|-----|------|-------|------|------|
| D-2 | **Terrain** 地形 | 地形类型、高度、遮挡、效果 | 🔴 高 | D-1 (Tactical) | ✅ 已完成 |
| D-3 | **Faction** 阵营 | 阵营关系、外交、仇恨列表 | 🟡 中 | D-1 | ✅ 已完成 |
| D-4 | **Unit** 单位 | 单位创建、属性、状态机 | 🔴 高 | D-1, D-3 | 🟡 骨架（无领域文档） |
| D-5 | **Combat** 战斗 | 攻击、防御、伤害计算 | 🔴 高 | D-1, D-4, Capabilities 验证 | 🟡 骨架 |
| D-6 | **Spell** 法术 | 法术释放、效果、冷却 | 🟡 中 | D-4, D-5 | 🟡 骨架 |
| D-7 | **Reaction** 反应 | 反击、触发效果、机会攻击 | 🟡 中 | D-5 | 🟡 骨架 |
| D-8 | **AI** 人工智能 | 敌方决策、寻路、行为树 | 🟡 中 | D-1, D-4 | 🟡 骨架 |
| D-9 | **Turn** 回合 | 回合顺序、行动点、阶段、胜败判定 | 🔴 高 | D-1, D-4 | ✅ 已完成 |
| D-10 | **Item** 物品 | 装备、消耗品、背包 | 🟢 低 | D-4 | 🟡 骨架 |
| D-11 | **Skill** 技能 | 职业技能、专精树 | 🟢 低 | D-4, D-5 | 🟡 骨架 |
| D-12 | **Quest** 任务 | 任务状态、目标、奖励 | 🟢 低 | D-4 | 🟡 骨架 |
| D-13 | **Narrative** 叙事 | 对话树、选择分支、过场动画 | 🟢 低 | D-3 | ✅ 已完成 |
| D-14 | **Economy** 经济 | 货币、商店、交易 | 🟢 低 | D-10 | 🟡 骨架 |
| D-15 | **Progression** 成长 | 经验、等级、解锁 | 🟢 低 | D-4, D-11 | 🟡 骨架 |

> **注**: 优先级为初步评估，实际启动顺序需结合具体业务需求和架构评审确定。

---

### 3.5 Phase E ~ H

> **当前状态**: 🔴 未开始
> **说明**: 这些阶段属于横切层和高级功能，在核心域（D-1 ~ D-9）稳定后再启动

| 阶段 | 说明 | 前置条件 |
|------|------|---------|
| Phase E | 内容系统 (Content) | C-3 Registry 完成 + 至少 3 个业务域稳定 |
| Phase F | 渲染系统 (Rendering) | D-1 Tactical 稳定 + 网格渲染需求明确 |
| Phase G | 音频系统 (Audio) | F 渲染基础完成 |
| Phase H | Modding 系统 | E 内容系统完成 |

---

## 4. 风险与缓解

| 风险 | 概率 | 影响 | 缓解措施 | 责任人 |
|------|------|------|---------|--------|
| Capabilities 21k 行存在未被发现的架构缺陷 | 中 | **高**（阻塞所有业务域） | Tactical 尽早接入，发现即修复 | @architect + @feature-developer |
| 14 个业务域全部为空骨架，启动顺序不明确 | 中 | 中 | 按业务优先级表（§3.4）逐步推进 | @architect |
| Save 模块后续 Per-Feature 序列化实现 | 低 | 中 | 桥接层骨架已完成，按需迭代 | @feature-developer |
| 单人瓶颈（所有代码变更经过同一人） | **高** | **高** | 这是一个已知限制 | — |
| @refactor-guardian 技术债扫描滞后 | 中 | 中 | ✅ 已完成，输出 `docs/11-refactor/debt-inventory-2026-06-17.md` | @refactor-guardian |

---

## 5. 工作量总估（Post-M1）

| 阶段 | 文件数 | 预估行数 | 主要执行者 | 建议工期 | 状态 |
|------|--------|---------|-----------|---------|------|
| **Capabilities 集成验证** | 4 | ~280 | @feature-developer | 1 次会话 | ✅ 已完成 |
| C-3 Registry 注册中心 | 3-4 | ~750 | @feature-developer | 1 次会话 | ✅ 已完成 |
| C-4 Replay 回放桥接层 | 5 | ~145 | @feature-developer | 1 次会话 | ✅ 已完成 |
| C-5 Save 存档桥接层 | 5 | ~260 | @feature-developer | 1 次会话 | ✅ 已完成 |
| D-2 Terrain 地形域 | ~12 | ~700 | @feature-developer | 2 次会话 | ✅ 已完成 |
| D-3 ~ D-15 (13 个域) | — | — | — | 待定 | 🔴 未开始 |
| **@refactor-guardian 技术债扫描** | — | — | @refactor-guardian | ~1 次会话 | ✅ 完成 |
| @architect — registry_schema v2 对齐 ADR-013 | 1 文件 | ~180 | @architect | 1 次会话 | ✅ 完成 |
| **总计（近期）** | **~26** | **~1,940** | **全角色协作** | **~6-8 次会话** | **C-4 Replay ✅ + C-5 Save ✅ + 全角色评审 PASS** |

---

## 6. 与上游文档的关联

| 文档 | 关系 | 说明 |
|------|------|------|
| `Phase-C-D-execution-plan.md` | 前置/基础 | M1 及之前完成的计划，本文档承接其状态 |
| `docs/01-architecture/README.md` | 架构约束 | 模块边界、Plugin 注册顺序 |
| `docs/02-domain/README.md` | 领域规则 | D-2 ~ D-15 各域的规则来源 |
| `docs/04-data/README.md` | 数据架构 | C-3 ~ C-5 的 Schema 设计依据 |
| `docs/05-testing/test-spec.md` | 测试规范 | 所有新增代码的测试标准 |

---

*本文档承接 `Phase-C-D-execution-plan.md`，聚焦 M1 里程碑之后的所有未完成任务。原计划的 M1 完成内容请参阅上游文档。*
