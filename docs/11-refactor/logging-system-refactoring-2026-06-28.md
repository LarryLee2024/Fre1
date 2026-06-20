# 日志/可观测系统激进重构计划

> 发现于 2026-06-28 全面日志系统 + 可观测性架构评审
> 基于 ADR-052、宪法 §11.1-§11.9、.trae/rules/日志规则.md、docs/08-knowledge/logging-overview.md、docs/04-data/infrastructure/logging_schema.md、docs/ai_ignore_this_dir/14可观测.md
> 优先级：P0（绝对禁止违规）→ P1（治理规则违规）→ P2（设计债）→ P3（工具化/自动化）

---

## 评审发现的所有问题

### 文档问题

| ID | 严重度 | 描述 | 位置 |
|----|--------|------|------|
| D01 | P1 | ADR-052 Module Design 只列出 7 个 Observer 模块，实际代码有 20 个模块、71 个 listener | `ADR-052.md §Module Design` |
| D02 | P1 | CNT 前缀的 target 不一致：ADR-052 写 `infra.content`，代码 LogCode::target() 返回 `"content"` | `ADR-052.md §Target层级规范` vs `log_code.rs:696` |
| D03 | P1 | ADR-052 不包含 `14可观测.md` 的核心观点（ObservableEvent trait、Observability Facade） | `ADR-052.md` 全篇 |
| D04 | P2 | logging_schema.md 将 LogCode/LogCategory 列为 Infrastructure 层，实际在 Shared 层 | `logging_schema.md §1` vs `src/shared/diagnostics/` |
| D05 | P2 | overview.md §7 统计 domain 层违规为 "13 个文件"，实际审计发现 17 个 domain 文件 + 2 个 capability 文件 | `logging-overview.md §7` |
| D06 | P3 | Plan 文档(logging-architecture-plan.md)使用已废弃的前缀名 BTL/ABI，实际代码用 BAT/ABL | `logging-architecture-plan.md` |

### 代码问题

| ID | 严重度 | 描述 | 位置 |
|----|--------|------|------|
| C01 | P0 | `on_shield_used` 和 `on_guardian_used` 复用 `LogCode::RCT005`（counterspell_executed），LogCode 与事件不匹配，是数据错误 | `reaction_logger.rs:113,130` |
| C02 | P0 | `on_shield_used` 和 `on_guardian_used` 已定义但未在 plugin.rs 注册——废弃代码 | `reaction_logger.rs` vs `plugin.rs:58-62` |
| C03 | P0 | plugin.rs 硬编码 "56 个 observer" 与实际 71 个 listener 不符，谁改代码谁忘更新数字 | `plugin.rs:143` |
| C04 | P1 | 缺少 `ObservableEvent` trait——领域事件与可观测系统之间没有正式契约，Observer 需要手动提取 event 字段 | `shared/diagnostics/` 不存在此 trait |
| C05 | P1 | `telemetry::emit(LogCode)` 只接受编码，不接受事件——Observer 仍需要手动调用 `metrics::record(code)` + `info!(target, fields...)`，三个动作没有统一 | `telemetry.rs:33-35` |
| C06 | P1 | `DiagnosticContext` 完全定义（含 log_info/log_warn/log_error + Builder）但 0 个 Observer 使用，已变冷存储 | `context.rs` 全文件 |
| C07 | P1 | 缺少可观测性门面（Observability Facade）——当前 Observer 模式是 "做三件事"（#[instrument] + emit + info!），而非 "做一件事"（emit event） | 全部 20 个 observer 文件 |
| C08 | P2 | Observer 中 `target` 在两处重复（`#[instrument]` + `info!()`），每个 Observer 多 2 行冗余代码 | 全部 observer 文件 |
| C09 | P2 | `FileSink` 未接入 tracing-subscriber Layer，目前无人调用，纯手工触发模式 | `sinks/file_sink.rs` |
| C10 | P2 | `warn_once!`/`error_once!` 宏定义存在，全项目 0 处实际使用，rate_limit 基础设施闲置 | `rate_limit/mod.rs` |
| C11 | P2 | `attribute/content.rs` 和 `tag/content.rs` 的 `info!` 直接调用——Capability 层未受宪法 §11.4 约束，无事件驱动 | `attribute/content.rs:29,65`、`tag/content.rs:30,66` |
| C12 | P2 | Domain 层 26 处 `warn!` + 24 处 `debug!` + 16 处 `trace!` 通过 tracing 直接输出，event 字段命名不一致（有 "combat.pipeline.not_found" 点分隔式、有 "combat_input.skill_slot" 混合式，无统一规范） | `core/domains/` 共 17 个文件 |
| C13 | P2 | Reaction Logger 有 7 个函数但只有 5 个 LogCode（RCT001-RCT005），缺少 RCT006/RCT007 | `reaction_logger.rs` + `log_code.rs` |
| C14 | P3 | `metrics::record(LogCode)` 是手动调用——未做到 "emit event 时 metrics 自动派生" | `metrics/mod.rs` + 全部 observer |
| C15 | P3 | `format_json()` 使用 Unix 时间戳，logging_schema.md §6.1 要求 ISO 8601 | `file_sink.rs:158-168` |
| C16 | P3 | MetricsCollector 依赖 `FrameCounter`（来自 infra/replay），logging 与 replay 非必要耦合 | `metrics/mod.rs:15` |
| C17 | P3 | 缺少日志/可观测系统自动化检查脚本 | `tools/` 目录 |

---

## 核心设计决策

### 决策 1：吸收 Single Source of Observability，但不创建 infra/observability/

`14可观测.md` 的核心洞见是：**日志、指标、追踪、审计本质都是"观测数据输出"，属于同一能力**。它建议 `infra/observability/` 统一管理。

我们的决策：**不重构目录，进化 telemetry.rs**。原因：
- 当前 `infra/logging/` 目录结构已经稳定运行，大量代码引用此路径
- `telemetry.rs` 天然就是可观测性门面（Facade）的宿主——扩展它而非重命名它
- 保持目录稳定减少 churn，让价值体现在接口设计上

```
// 当前：
infra/logging/telemetry.rs  ← 只包装 metrics::record

// 目标：
infra/logging/telemetry.rs  ← 真正的 Observability Facade
  emit(impl ObservableEvent) → Log + Metrics 自动分发
```

### 决策 2：创建 ObservableEvent trait，但不创建 core/events/

`14可观测.md` 建议 `core/events/` 存放共享领域事件。我们的决策：**不创建新目录，trait 放 shared/diagnostics/**。原因：
- 领域事件已按 DDD 分布在 `core/domains/*/events/` 中，这是正确的
- `shared/diagnostics/` 天然是可观测契约（Contract）的宿主——它被所有层引用
- ObservableEvent trait 是纯接口，不包含业务逻辑，放在 Shared 层符合架构规则

```
shared/diagnostics/
├── log_code.rs
├── log_category.rs
├── correlation.rs
├── context.rs
└── observable.rs       ← NEW: ObservableEvent trait
```

### 决策 3：Observer 从"做三件事"变为"做一件事"

当前每个 Observer 的模式：
```
1. #[instrument] span
2. telemetry::emit(LogCode)  → metrics
3. info!(target, fields...)  → log
```

目标模式：
```
1. #[instrument] span
2. telemetry::emit(LogCode, &event)  → metrics + log 自动分发
```

关键变化：
- `emit()` 内部从 LogCode 的 `target()` 派生 target，消除重复
- `emit()` 内部自动 metrics::record(code)，Observer 不再手动调用
- 事件的结构化字段通过 `ObservableEvent::emit_to()` 反射式提取

### 决策 4：保留 LogCode 数值编码，不改为命名枚举

`14可观测.md` 建议用 `AbilityActivated` 替代 `ABL001`。我们保留现有编码体系，原因：
- 150+ 个编码已部署，全项目一致使用
- 数值编码的搜索优势明确：`grep "BAT007"` 唯一匹配，用名字 `DamageDealt` 可能有同名歧义
- LogCode 已有 `event_name()`（英文名）+ `description()`（中文描述），两者兼具
- 如果未来需要语义引用，可以通过 `event_name()` 派生而非放弃编码

---

## 重构阶段

### Phase 0: 文档统一（P1-P2 文档问题）

**目的**：消除所有文档间的矛盾，吸收 `14可观测.md` 的核心观点到官方文档体系中。

| 任务 | 文件 | 操作 |
|------|------|------|
| 0.1 | ADR-052 §Module Design | 更新 Observer 模块列表为 20 个模块、71 个 listener，删除硬编码数字改为 "见 plugin.rs 实际注册" |
| 0.2 | ADR-052 §Target 层级规范 | 统一 CNT target 为 `"content"`（与代码一致），修正 ADR 表 |
| 0.3 | ADR-052 | 新增 "可观测性门面（Observability Facade）" 节：基于 `14可观测.md` 提炼的 Single Source of Observability 原则 |
| 0.4 | ADR-052 | 新增 ObservableEvent trait 设计决策，说明其位于 shared/diagnostics |
| 0.5 | logging_schema.md §1 | 修正 LogCode/LogCategory 归属层为 Shared（非 Infrastructure） |
| 0.6 | logging_schema.md | 新增 ObservableEvent trait 的 Schema 定义 |
| 0.7 | overview.md §7 | 更新 domain 层违规统计，标记 `warn!`/`debug!`/`trace!` 为"规则例外已评估" |
| 0.8 | overview.md | 新增 §10 "未来演进" 节，描述 ObservableEvent + Observability Facade 路线图 |

**验证**：
- ADR-052 不再提到 "56 个 observer"
- ADR-052 包含 ObservableEvent 和 Observability Facade 的设计
- LogCode::target() 与 ADR-052 target 表一致

---

### Phase 1: 修复 P0 代码违规

**目的**：消除所有运行时错误和废弃代码。

**任务 1.1: LogCode 扩展**

```rust
// log_code.rs 新增变体：
RCT006, // shield_used              护盾术
RCT007, // guardian_used            援护格挡

// event_name:
Self::RCT006 => "shield_used",
Self::RCT007 => "guardian_used",

// description:
Self::RCT006 => "护盾术",
Self::RCT007 => "援护格挡",
```

更新 `log_category.rs`：RCT006/RCT007 → `LogCategory::Battle`。

**任务 1.2: 修复 Reaction Logger 错误**

`reaction_logger.rs` 中两处 LogCode 纠错 + 注册：
- `on_shield_used`：`LogCode::RCT005` → `LogCode::RCT006`，`event = "counterspell_executed"` → `event = "shield_used"`
- `on_guardian_used`：`LogCode::RCT005` → `LogCode::RCT007`，`event = "counterspell_executed"` → `event = "guardian_used"`
- `plugin.rs`：注册这两个函数

**任务 1.3: 修复 Observer 数量硬编码**

```rust
// plugin.rs:143 从：
tracing::info!(target: "logging", "[LoggingPlugin] 已初始化（Metrics + {} 个 observer）", 56);
// 改为：
// 使用编译期计数，确保代码真实：
const OBSERVER_COUNT: usize = count_of_registered_observers();  // 或手动更新为 71
tracing::info!(target: "logging", "[LoggingPlugin] 已初始化（Metrics + {} 个 observer）", OBSERVER_COUNT);
```

并添加注释 `// 增删 Observer 后请更新此数字与 observers/mod.rs 模块声明`。

**验证**：
- `cargo build` 通过
- `cargo nextest run` 通过
- RCT006/RCT007 的 LogCode 编码有效
- Shield/Guardian 不再误用 RCT005

---

### Phase 2: ObservableEvent trait（Shared 层契约）

**目的**：在 `shared/diagnostics/` 中定义可观测性契约，让领域事件与可观测系统之间建立正式的类型约束。

这是吸收 `14可观测.md` 核心观点后的新增阶段——先有契约，再改实现。

**任务 2.1: 新增 `shared/diagnostics/observable.rs`**

```rust
//! 可观测事件契约 — 领域事件与可观测系统之间的正式接口。
//!
//! 任何需要被日志/指标/追踪系统监听的领域事件都应实现此 trait。
//! 这保证了 Observer 可以通过统一方式提取事件的结构化字段。
//!
//! 详见 ADR-052 §ObservableEvent

use super::LogCode;
use std::fmt::Debug;

/// 可观测事件 — 领域事件实现此 trait 后，Observability Facade
/// 可以自动将事件分发到日志、指标、追踪等所有 sink。
///
/// # 实现示例
///
/// ```ignore
/// impl ObservableEvent for LevelUp {
///     fn log_code(&self) -> LogCode {
///         LogCode::PRG002
///     }
/// }
/// ```
pub trait ObservableEvent: Debug + Send + Sync + 'static {
    /// 返回该事件对应的 LogCode。
    fn log_code(&self) -> LogCode;

    /// 将事件的结构化字段写入 FieldCollector。
    /// Observer 可以通过此方法获取事件的动态字段，无需反射。
    ///
    /// 默认实现不收集任何字段，事件类型可覆盖此方法提供字段。
    fn record_fields(&self, _collector: &mut FieldCollector) {}
}

/// 结构化字段收集器——Observer 通过此结构收集事件字段。
#[derive(Debug, Default)]
pub struct FieldCollector {
    fields: Vec<(&'static str, String)>,
}

impl FieldCollector {
    /// 添加一个结构化字段（用于 emit_to 内部）。
    pub fn add_field(&mut self, key: &'static str, value: impl std::fmt::Display) {
        self.fields.push((key, value.to_string()));
    }

    /// 获取所有收集的字段。
    pub fn fields(&self) -> &[(&'static str, String)] {
        &self.fields
    }
}
```

**任务 2.2: 为 1-2 个核心事件实现 ObservableEvent 作为示范**

在 `core/domains/progression/events/` 中选择 `LevelUp` 事件实现示范：

```rust
impl ObservableEvent for LevelUp {
    fn log_code(&self) -> LogCode {
        LogCode::PRG002
    }

    fn record_fields(&self, collector: &mut FieldCollector) {
        collector.add_field("entity", format_args!("{:?}", self.entity));
        collector.add_field("old", self.old_level);
        collector.add_field("new", self.new_level);
    }
}
```

**任务 2.3: 更新 shared/diagnostics/mod.rs**

```rust
mod observable;
pub use observable::{FieldCollector, ObservableEvent};
```

**任务 2.4: 更新 ADR-052 和 logging_schema.md**

在 ADR-052 中新增 "ObservableEvent 契约" 节，描述此类及其在架构中的角色。

**验证**：
- `cargo build` 通过
- `cargo nextest run` 通过
- `LevelUp` 实现了 `ObservableEvent`
- 可以在 `emit(LogCode, &impl ObservableEvent)` 的测试中调用

---

### Phase 3: Observability Facade — telemetry::emit 真正的统一入口（P1-P2 设计债）

**目的**：将 `telemetry::emit` 从"包装 metrics::record 的薄壳"升级为真正的可观测性门面——**Observer 只做一件事，emit 做剩下的事**。

**核心变化**：

```
当前 Observer 做 3 件事：
  1. #[instrument] span（声明不变量）
  2. telemetry::emit(LogCode)          → metrics
  3. info!(target, field1, field2, ...) → log

目标 Observer 做 1 件事：
  1. #[instrument] span（声明不变量）
  2. telemetry::emit(LogCode::PRG002, &event)  → metrics + log 自动分发
```

**任务 3.1: 新增 emit_with_fields**

```rust
// telemetry.rs

/// 发射一次可观测事件。
///
/// # 职责
/// 1. 记录 metrics::record(code)
/// 2. 从 LogCode::target() 派生 tracing target
/// 3. 输出结构化日志（字段来自 FieldCollector）
///
/// Observer 调用此方法后不需要再手动 info!/warn! 或 metrics::record。
///
/// # 注意
/// 此变体接受显式字段列表，不依赖 ObservableEvent trait。
/// 适用于现有 Observer 平滑迁移。
pub fn emit_with_fields(code: LogCode, event: &'static str, fields: &[(&'static str, String)], level: LogLevel) {
    // 1. metrics
    metrics::record(code);

    // 2. 结构化字段构建
    let target = code.target();

    // 3. 日志输出
    match level {
        LogLevel::Info => {
            // 使用 tracing::info! 的变长参数方式
            // 但 Rust 宏限制，通过 span 的 record 或直接 info! 实现
            // 简化方案：使用 tracing::event! 宏
            let msg = event;
            match fields.len() {
                0 => tracing::info!(target: target, code = ?code, event = event, "{msg}"),
                _ => {
                    // 构建动态 event
                    // 由于 tracing 宏需要编译期字段名，这里使用 span 的 record
                    // 或通过 tracing_subscriber 的 Visitor 模式
                    // 简化实现：逐个添加到 span 中
                    let span = tracing::span!(target: target, tracing::Level::INFO, "emit", code = ?code, event = event);
                    let _guard = span.enter();
                    for (key, value) in fields {
                        tracing::info!(target: target, key = %key, value = %value, "{msg}");
                    }
                }
            }
        }
        // ... 类似处理 Warn, Debug, Error
    }
}
```

注：由于 Rust 宏的限制，无法在运行时动态构造 `tracing::info!(target, field1 = val1, field2 = val2)` 的字段列表。因此 `emit_with_fields` 有两种实现方向：

**方案 A（推荐）**：每个字段单独 `trace!` 或 `info!` 一行——简单可靠，但日志输出多行。
**方案 B**：使用 `tracing::span!` + 字段在 span 中 record——结构化但耦合 span。
**方案 C（最实用）**：宏方案 `emit_info!` 保留编译期字段名优势，`emit_with_fields` 用于动态场景。

考虑到 Observer 代码最需要的是消除 target 重复 + 自动 metrics，**最实用的方式是宏方案 + 保留现有字段语法**：

```rust
// telemetry.rs — 新增宏

/// 统一 INFO 日志 + 度量入口。
/// 自动从 LogCode 派生 target，消除 info!() 中的 target 重复。
///
/// # 用法
/// ```ignore
/// emit_info!(LogCode::PRG002, entity = ?e.entity, old = e.old_level, "角色升级");
/// ```
#[macro_export]
macro_rules! emit_info {
    ($code:expr, $($key:tt = $value:expr),* $(,)? $msg:literal $(,)?) => {
        {
            $crate::infra::logging::metrics::record($code);
            ::tracing::info!(
                target: $code.target(),
                code = ?$code,
                $($key = $value,)*
                $msg,
            );
        }
    };
}
/// 统一 WARN 日志 + 度量入口。
#[macro_export]
macro_rules! emit_warn {
    ($code:expr, $($key:tt = $value:expr),* $(,)? $msg:literal $(,)?) => {
        {
            $crate::infra::logging::metrics::record($code);
            ::tracing::warn!(
                target: $code.target(),
                code = ?$code,
                $($key = $value,)*
                $msg,
            );
        }
    };
}
```

注意：宏内调用 `$code.target()` 派生 target，Observer 调用时不再需要写 `target = "domain.xxx"`。

**任务 3.2: 迁移 Observer 到 emit_info!/emit_warn!**

迁移模式：
```rust
// 之前：
#[tracing::instrument(skip_all, target = "domain.progression", fields(
    code = ?LogCode::PRG002,
    event = "level_up",
))]
pub(crate) fn on_level_up(trigger: On<LevelUp>) {
    telemetry::emit(LogCode::PRG002);
    let event = trigger.event();
    info!(
        target = "domain.progression",
        entity = ?event.entity,
        old = event.old_level,
        new = event.new_level,
        "角色升级",
    );
}

// 之后（推荐 — 保留 event 在 span 中，Observer body 干净）：
// 方案 A：保留 #[instrument] 的 event 做日志索引，emit_info 只放变量
#[tracing::instrument(skip_all, target = "domain.progression", fields(
    code = ?LogCode::PRG002,
    event = "level_up",
))]
pub(crate) fn on_level_up(trigger: On<LevelUp>) {
    let event = trigger.event();
    emit_info!(
        LogCode::PRG002,
        entity = ?event.entity,
        old = event.old_level,
        new = event.new_level,
        "角色升级",
    );
}

// 方案 B（未来 — target 完全从 #[instrument] 移除，只用 emit_info）：
// 但注意：#[instrument] 的 target 影响 span 名，需要保留 span 链路追踪
// 所以暂时保留 #[instrument(target = "...")]，只消除 info! 内的重复 target
```

**任务 3.3: 迁移全部 71 个 Observer**

分批次迁移，每批验证编译和测试。

```
Batch 1: progression, battle, turn    (10 observers)   — 先验证模式
Batch 2: ability, effect, spell       (9 observers)
Batch 3: inventory, economy, crafting (10 observers)
Batch 4: quest, faction, party        (14 observers)
Batch 5: camp_rest, narrative, summon (14 observers)
Batch 6: tactical, terrain, content   (7 observers)
Batch 7: reaction                     (7 observers)
```

**验证**：
- `cargo build` 通过（每批）
- `cargo nextest run` 通过（每批）
- 每个 Observer 不再有 `target = "domain.xxx"` 在 `info!`/`warn!` 中（但 `#[instrument]` 中仍保留）
- 每个 Observer 不再有 `telemetry::emit(LogCode::XXX)` 单独调用（已被宏自动包含）

---

### Phase 4: DiagnosticContext 清理（P1 冷存储）

**目的**：结束 `DiagnosticContext` "定义了但无人使用" 的状态。

**任务 4.1**: 标记 context.rs 为 deprecated

```rust
// DEPRECATED: 当前 Observer 采用 emit_info!/emit_warn! 传结构化字段模式。
// DiagnosticContext 保留为工具类，供需要 CorrelationId 链路追踪的
// 复杂调试场景使用，但不再是 Observer 的强制模式。
//
// 原因：emit_info! 直接传字段在 90% 场景中更具可读性，
// 且不需要 Builder 模式的开销。
```

**任务 4.2**: 更新文档中 DiagnosticContext 的状态

- overview.md §7：`DiagnosticContext 实际使用` → ⚠️ 已弃用（保留为工具类）
- ADR-052 §DiagnosticContext：说明当前状态和适用范围

**验证**：context.rs 文件头有 deprecation 注释。

---

### Phase 5: FileSink 接入 tracing-subscriber Layer（P2 架构债）

**目的**：让 FileSink 自动捕获 tracing 事件，输出 JSONL 文件。

**任务 5.1**: 实现 FileSinkLayer

```rust
// sinks/file_sink.rs
pub struct FileSinkLayer {
    sink: FileSink,
}

// 实现 tracing_subscriber::Layer
// 注意：需要与 Bevy 的 DefaultPlugins.LogPlugin 共存
```

**任务 5.2**: 在 LoggingPlugin 中注册 Layer

```rust
// plugin.rs
pub fn build(&self, app: &mut App) {
    // ... 现有代码 ...
    
    // 初始化文件日志（默认仅 debug 模式启用）
    app.insert_resource(FileSinkConfig::default());
    // 使用 startup_system 注册 Layer（app 构建完成后）
}
```

> **注意**：Bevy 已经在 DefaultPlugins 中初始化 tracing-subscriber。添加 FileSinkLayer 需要通过 `app.set_global_tracing_subscriber()` 或在首次 init 前追加 Layer。如果 Bevy 的 LogPlugin 不支持扩展 Layer，可能需要自定义 LogPlugin 替代方案。

**任务 5.3**: 修复 format_json 时间戳格式

```rust
// 从 Unix epoch 改为 ISO 8601
fn format_timestamp() -> String {
    // 使用 time 或 chrono 依赖（或简单 UTC 格式化）
}
```

**验证**：
- `cargo build` 通过
- 启动后 `logs/game.jsonl` 自动生成
- 文件内容为合法 JSON，有 ISO 8601 时间戳

---

### Phase 6: rate_limit 推广 + Domain 调用清理（P2 长期债）

**目的**：让闲置的 rate_limit 基础设施实际使用 + 统一 domain 直接 tracing 调用的 event 命名。

**任务 6.1: 高频 warn! 点迁移到 warn_once!**

| 位置 | 风险 | 处理 |
|------|------|------|
| `combat/pipeline/driver.rs:117` | 每帧 pipeline 执行时可能触发 | 改用 `warn_once!` |
| `combat/effect_tick_system.rs:47` | Effect Tick 循环内 | 改用 `warn_once!` |
| `inventory/inventory_system.rs:21,45,67` | 每帧遍历物品时可能触发 | 改用 `warn_once!` |

**任务 6.2: 统一 domain 直接 tracing 调用的 event 命名**

从混合命名（`"combat.pipeline.not_found"`、`"combat_input.skill_slot"`）统一为 LogCode 映射的 snake_case 名。注意这些直接调用不经过 Observer，所以没有自己的 LogCode。决策：**按"日志规则.md"例外保留直接调用，但将 event 值统一为 domain 前缀的 snake_case**。

```
"combat.pipeline.not_found"     → "combat_pipeline_not_found"  (点转下划线)
"combat_input.skill_slot"       → "combat_input_skill_slot"
"party.add_member.failed"       → "party_add_member_failed"
"faction.relationship_eval..."  → "faction_relationship_eval..."
```

这不是强制要求而是规范性建议——直接 tracing 调用的 event 字段本质上是自由文本，一致性提升在调试时体现价值。

**验证**：
- `cargo build` 通过
- `cargo nextest run` 通过
- 高频 warn! 点首次触发仍输出日志，后续不再重复

---

### Phase 7: 工具脚本与最终验证（P3 工具化）

**目的**：创建自动化检查脚本，防止架构退化。

**任务 7.1**: 创建 `tools/check-logging-invariants.sh`

```bash
#!/usr/bin/env bash
# Logging Invariant Lint

# 1. 检查 Observer 中是否还有残留的 telemetry::emit + info! 分离模式
#    （Phase 3 迁移后，应全部使用 emit_info!/emit_warn!）

# 2. 检查 plugin.rs observer 计数与实际注册数一致
#    grep "add_observer" plugin.rs | wc -l VS 硬编码数字

# 3. 检查 RCT LogCode 复用（reaction_logger.rs 的坑）
#    确保不同 observer 函数使用不同 LogCode

# 4. 检查所有 observer 函数都有 #[instrument]（格式规范）

# 5. 检查 emit_info!/emit_warn! 中没有 target 参数（必须由宏自动派生）
```

**任务 7.2**: 更新 overview.md

更新 §7 现状表反映所有 Phase 完成后状态。

**任务 7.3**: 最终验证

```bash
cargo build
cargo nextest run
cargo clippy -- -D warnings
tools/check-logging-invariants.sh --ci
```

---

## 关于 `14可观测.md` 哪些观点我们吸收、哪些没吸收的解释

### 吸收的观点

| 观点 | 落地方式 |
|------|---------|
| **Single Source of Observability** — 业务代码只调用一次 emit，log/metric/trace 自动分发 | Phase 3 telemetry::emit 升级为 Observability Facade，Observer 从"做三件事"变为"做一件事" |
| **ObservableEvent trait** — 领域事件与可观测系统之间的正式契约 | Phase 2 在 shared/diagnostics/observable.rs 定义，不侵入现有领域事件 |
| **Metrics 应自动派生** — emit event 时 metrics 不应需要手动调用 | Phase 3 emit_info!/emit_warn! 宏内部自动 record(code)，Observer 不再手动 telemetry::emit |
| **Observability 是 Infra 能力** — 不是新领域，不创建顶层 telemetry/ 模块 | 保持 infra/logging/ 不变，进化 telemetry.rs 为门面 |
| **Shared/diagnostics 值得扩张** — 纯契约放这里 | Phase 2 将 ObservableEvent trait 加入 shared/diagnostics |
| **Domain Event = Observability Event** — 不另建 TelemetryEvent 层级 | 已经如此，Observer 直接监听领域事件 |
| **高基数控制** — 禁止实体名作为 metric label | 已在宪法 §11.3 和日志规则.md 中落实 |

### 未吸收的观点及原因

| 观点 | 未吸收原因 |
|------|-----------|
| **infra/observability/ 替代 infra/logging/** | 当前目录已稳定运行，telemetry.rs 的门面进化不需要目录改名。改名只有结构美观收益却带来大量 import 更新成本 |
| **core/events/ 统一存放领域事件** | 我们的领域事件按 DDD 分布在 `core/domains/*/events/` 中。单独的 core/events/ 拆散业务内聚 |
| **LogCode 用命名体替代数值编码** | 150+ 个编码已部署，数值的搜索明确性（grep "BAT007" 唯一匹配）是实际优势。LogCode 已有 event_name() 补充语义 |
| **Replay 归入 Observability** | Replay 有确定性录制/帧精确回放的特殊要求。两者共享事件来源（领域事件），不共享基础设施 |
| **Observer 中消除 #[instrument] 的 target** | `#[instrument]` 的 target 影响 span 名和链路追踪，保留它不影响 Observer body 的简洁性。Phase 3 只消除 info!() 中的 target 重复 |

---

## 优先级汇总

| 阶段 | 任务 | 级别 | 估计文件变更 | 风险 | 状态 |
|------|------|------|-------------|------|------|
| 0 | 文档统一 + 吸收 14可观测.md 观点 | P1-P2 | 4 文档 | 低 | 🟡 待开始 |
| 1 | 修复 Reaction Logger LogCode 复用 + 注册遗漏 | P0 | 4 文件 | 低 | 🟡 待开始 |
| 2 | ObservableEvent trait（shared/diagnostics 扩张） | P1 | 3 文件 | 低 | 🟡 待开始 |
| 3 | Observability Facade（emit_info!/emit_warn! 迁移） | P1-P2 | 73 文件 | 中 | 🟡 待开始 |
| 4 | DiagnosticContext 弃用声明 | P1 | 3 文件 | 低 | 🟡 待开始 |
| 5 | FileSink 接入 tracing-subscriber Layer | P2 | 2 文件 | 高 | 🟡 待开始 |
| 6 | rate_limit 推广 + Domain 调用 event 统一 | P2 | 6 文件 | 低 | 🟡 待开始 |
| 7 | 工具脚本 + 文档更新 + 最终验证 | P3 | 1 脚本 + 1 文档 | 低 | 🟡 待开始 |

### 已知剩余技术债

| 问题 | 位置 | 原因 |
|------|------|------|
| Domain 层直接 `warn!`/`debug!`/`trace!` 调用 | 17 个 domain 文件 | "日志规则.md"例外允许（WARN/DEBUG/TRACE 不走事件链路），但 event 命名格式需统一 |
| Capability 层 `info!` 直呼 | attribute/content.rs, tag/content.rs | Capability 层未明确受宪法 §11.4 约束，灰色地带 |
| FrameCounter 耦合 | metrics/mod.rs | logging 依赖 replay，遗留设计。需评估是否可用帧计数器替代 |
| Metrics 自动派生不完整 | 全部 observer | 当前 emit_info! 宏仍需要手动写结构化字段，完全的自动派生需要 macro 或 proc-macro 支持 |

### 最终验证

- `cargo build` — 🟡
- `cargo nextest run` — 🟡
- `cargo clippy -- -D warnings` — 🟡
- `tools/check-logging-invariants.sh --ci` — 🟡
