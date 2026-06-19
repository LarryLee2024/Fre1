---
id: logging-architecture-plan
title: 日志架构实施计划
status: proposed
owner: architect
created: 2026-06-25
based-on: 8logging.md + 宪法§11 + 项目现状
---

# 日志架构实施计划

## 现状分析

### 宪法已定义（§11.1-11.5）
- ✅ 领域事件触发 → 统一 Log Observer 监听 → 输出 tracing 日志
- ✅ 领域层禁止直接 `info!`，必须走事件链路
- ✅ 结构化日志，`event` 字段与事件名一致
- ✅ Log Observer 统一在基础设施层

### 8logging.md 补充的 50 万行级设计
- LogCode（BTL001、ABI001 等编码体系）
- CorrelationId（BattleId → TurnId → ActionId 链路）
- DiagnosticContext（自动携带上下文）
- 内容配置日志单独分类
- 日志风暴保护（warn_once!、RateLimitedLogger）
- Span 链路（#[instrument]）
- ERROR 预算 = 0
- 领域前缀分配（CHR/ABI/EFF/BUF/TAG/TGT/BAT/MAP/CNT）

### 当前代码违规（189+ 处）
| 位置 | 违规数 | 类型 |
|------|--------|------|
| `content/hot_reload.rs` | 40+ | `info!`/`warn!` 直接在业务代码 |
| `content/content_plugin.rs` | 20+ | `info!` 直接在加载逻辑 |
| `infra/save/` | 8 | `info!`/`error!` |
| `combat/systems/input_system.rs` | 7 | `info!` 在系统中 |
| `tactical/systems/input_system.rs` | 3 | `info!` 在系统中 |
| `combat/integration/replay/` | 5 | `debug!` |
| `terrain/systems/` | 1 | `info!` |

### 缺失的基础设施
- ❌ `src/infra/logging/` — 不存在（宪法要求存在）
- ❌ `src/shared/diagnostics/` — 不存在（LogCode/LogCategory/LogContext）
- ❌ 无 LogObserver — 没有监听领域事件生成日志的机制
- ❌ 无结构化日志工具 — 无 LogCode、无 DiagnosticContext、无限流保护

---

## 实施计划

### Phase 1: 文档先行（架构 + 领域 + 数据）

#### 1.1 新增 ADR：`docs/01-architecture/40-cross-cutting/ADR-052-logging-architecture.md`

**内容**：
- 引用宪法 §11 + 8logging.md 核心设计
- 定义 Logging 架构：领域事件 → Log Observer → tracing 输出
- 定义 LogCode 编码体系（域前缀 + 三位编号）
- 定义 CorrelationId 链路（BattleId → TurnId → ActionId）
- 定义 DiagnosticContext 结构
- 定义日志风暴保护规则
- Forbidden 清单

#### 1.2 新增 Schema：`docs/04-data/infrastructure/logging_schema.md`

**内容**：
- LogCode 枚举定义（按域分组）
- LogCategory 分类（Battle/Ability/Effect/Content/Infra）
- DiagnosticContext 数据结构
- CorrelationId 类型定义
- 日志输出格式规范（JSON + 人类可读）

#### 1.3 更新领域文档：各域事件消费表

当前各域文档（02-domain/domains/*.md）已列出"日志"作为事件消费者，但未明确 LogCode。需要补充：
- 每个域事件对应的 LogCode
- 每个域的前缀分配

域前缀分配：
```
BAT — Combat
TAC — Tactical
TER — Terrain
ABL — Ability (capability)
EFF — Effect (capability)
TAG — Tag (capability)
MOD — Modifier (capability)
AGG — Aggregator (capability)
TRG — Trigger (capability)
SPR — Spell
RCT — Reaction
QST — Quest
PRG — Progression
INV — Inventory
ECO — Economy
CRF — Crafting
FAC — Faction
PRY — Party
CNR — CampRest
NAR — Narrative
SUM — Summon
CNT — Content (infra)
SAV — Save (infra)
RPL — Replay (infra)
```

#### 1.4 更新 `docs/00-governance/coding-rules.md`

在 §14 Logging 章节补充：
- LogCode 使用规范
- 结构化日志示例
- 日志风暴保护规则

---

### Phase 2: 共享基础设施（shared/diagnostics）

#### 2.1 新建 `src/shared/diagnostics/`

```
src/shared/diagnostics/
├── mod.rs
├── log_code.rs        — LogCode 枚举（按域分组）
├── log_category.rs    — LogCategory 分类
├── correlation.rs     — CorrelationId（BattleId, TurnId, ActionId）
└── context.rs         — DiagnosticContext
```

#### 2.2 LogCode 设计

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogCode {
    // Battle
    BTL001, // battle_started
    BTL002, // battle_ended
    BTL003, // turn_started
    BTL004, // turn_ended
    // Ability
    ABI001, // ability_activated
    ABI002, // ability_completed
    ABI003, // ability_not_found
    // Effect
    EFF001, // effect_applied
    EFF002, // effect_expired
    EFF003, // effect_immunity
    // ... 完整列表在 ADR 中定义
}
```

每个 LogCode 实现 `Display` 和 `Into<&'static str>`。

#### 2.3 DiagnosticContext

```rust
pub struct DiagnosticContext {
    pub battle_id: Option<BattleId>,
    pub turn_id: Option<TurnId>,
    pub action_id: Option<ActionId>,
    pub entity: Option<Entity>,
}
```

实现 `info!`/`warn!`/`debug!` 的便捷方法，自动携带上下文。

---

### Phase 3: 基础设施实现（infra/logging）

#### 3.1 新建 `src/infra/logging/`

```
src/infra/logging/
├── mod.rs
├── plugin.rs           — LoggingPlugin
├── observers/
│   ├── mod.rs
│   ├── battle_logger.rs    — 监听战斗事件
│   ├── ability_logger.rs   — 监听技能事件
│   ├── effect_logger.rs    — 监听效果事件
│   ├── spell_logger.rs     — 监听法术事件
│   ├── quest_logger.rs     — 监听任务事件
│   ├── content_logger.rs   — 监听内容加载事件
│   └── turn_logger.rs      — 监听回合事件
├── rate_limit/
│   ├── mod.rs
│   └── once_guard.rs       — warn_once!/error_once! 实现
└── sinks/                  — 预留（console/file/telemetry）
    └── mod.rs
```

#### 3.2 LoggingPlugin

```rust
pub struct LoggingPlugin;
impl Plugin for LoggingPlugin {
    fn build(&self, app: &mut App) {
        // 注册所有 Logger Observer
        app.add_observer(BattleLogger::on_battle_started);
        app.add_observer(AbilityLogger::on_ability_activated);
        // ...
    }
}
```

#### 3.3 Observer 示例

```rust
fn on_battle_started(trigger: Trigger<BattleStarted>, ctx: Option<Res<DiagnosticContext>>) {
    let e = trigger.event();
    info!(
        event = "battle_started",
        battle_id = ?e.battle_id,
        participants = e.participant_count,
        "battle_started"
    );
}
```

---

### Phase 4: 代码迁移（清理违规）

#### 4.1 清理 domain 层直接 `info!`（优先级 P0）

| 文件 | 违规数 | 处理 |
|------|--------|------|
| `combat/systems/input_system.rs` | 7 | 改为触发 InputAction 事件，由 LogObserver 监听 |
| `tactical/systems/input_system.rs` | 3 | 同上 |
| `terrain/systems/terrain_effect_system.rs` | 1 | 改为触发 TerrainEffect 事件 |

#### 4.2 清理 content 层直接 `info!`/`warn!`（优先级 P1）

| 文件 | 违规数 | 处理 |
|------|--------|------|
| `content/hot_reload.rs` | 40+ | 改为触发 ContentReloaded/ContentWarning 事件 |
| `content/content_plugin.rs` | 20+ | 改为触发 ContentLoaded 事件 |

#### 4.3 清理 infra 层直接 `info!`/`error!`（优先级 P2）

| 文件 | 违规数 | 处理 |
|------|--------|------|
| `infra/save/` | 8 | infra 层可保留直接日志（属于基础设施日志，非业务事件） |
| `infra/replay/` | 5 | 同上 |

**注意**：infra 层的 `info!`/`error!` 属于基础设施自身的日志（如"存档成功"），不是业务事件日志，可以保留。但需统一格式（结构化 + LogCode）。

---

### Phase 5: 验证 + 归档

#### 5.1 验证清单
- [ ] `cargo build` 编译通过
- [ ] `cargo nextest run` 所有测试通过
- [ ] `cargo clippy` 无新增 warning
- [ ] domain 层零 `info!`/`warn!` 直接调用（仅事件触发）
- [ ] infra/logging/ 模块可独立运行
- [ ] LogCode 枚举完整覆盖所有域事件

#### 5.2 归档
- 更新 `docs/01-architecture/README.md` — 添加 Logging 模块
- 更新 `docs/04-data/README.md` — 添加 logging_schema.md
- 更新 `docs/09-planning/README.md` — 记录本次计划

---

## 执行顺序

```
Phase 1 (文档)  ← 可并行
  ├── ADR-052
  ├── logging_schema.md
  ├── 领域文档更新
  └── coding-rules 更新

Phase 2 (shared/diagnostics)  ← 依赖 Phase 1
  └── LogCode + DiagnosticContext + CorrelationId

Phase 3 (infra/logging)  ← 依赖 Phase 2
  └── LoggingPlugin + Observers + RateLimit

Phase 4 (代码迁移)  ← 依赖 Phase 3
  ├── P0: domain 层清理
  ├── P1: content 层清理
  └── P2: infra 层格式统一

Phase 5 (验证)  ← 依赖 Phase 4
  └── 编译 + 测试 + clippy
```

## 预估工时

| Phase | 工时 | 角色 |
|-------|------|------|
| Phase 1 | 2-3h | @architect + @data-architect |
| Phase 2 | 2-3h | @feature-developer |
| Phase 3 | 3-4h | @feature-developer |
| Phase 4 | 4-6h | @feature-developer |
| Phase 5 | 1h | @test-guardian + @code-reviewer |
| **总计** | **12-17h** | |

## 风险

1. **content/hot_reload.rs 改动量大**（40+ 处），需谨慎
2. **infra 层日志是否保留**需确认（基础设施日志 vs 业务事件日志的边界）
3. **LogCode 枚举完整性**需与所有域事件对齐
