---
id: 01-architecture.logging-design
title: Logging Design
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
  - design
---

# Logging Design — 日志系统设计

Version: 1.0
Status: Proposed

来源：`docs/其他/31遗漏.md` 第四节 — 日志系统设计

本文档定义结构化、分层的日志架构，涵盖日志级别、基础设施、记录规则和输出策略。

交叉引用：
- `docs/01-architecture/README.md` — 日志总纲（tracing 统一、禁止 println）
- `docs/01-architecture/09-infrastructure-migration/infrastructure-design.md` — logging 模块设计
- `docs/01-architecture/events_audit_design.md` — 领域事件系统、EventWhitelist

---

## 0. 日志架构核心原则 🟥

🟥 **日志是领域事件的消费者，而非业务代码主动调用的功能**（宪法 13.8.1）。

链路：**领域事件触发 → 统一 Log Observer 监听 → 输出 tracing 日志**

```
Core 系统 → DomainEvent Message → LogObserver 监听
                                      ↓
                               tracing 日志输出（带 event/target 字段）
                                      ↓
                               AuditTrail 收集（可选）
```

🟥 所有 INFO 级别的核心业务事件，必须通过触发领域事件的方式生成日志（宪法 13.8.2）。
🟥 复用现有业务事件，绝对禁止为了打日志单独创建事件（宪法 13.8.2）。
🟥 日志 Observer 统一放在基础设施层，绝对不侵入业务模块（宪法 13.8.2）。

### 例外范围（可直接调用 tracing 宏） 🟩

以下场景可直接调用 tracing 宏，不受此限（宪法 13.8.3）：
- ERROR / WARN 级别异常日志
- DEBUG / TRACE 级别调试日志（业务数据细化，非系统执行日志）
- 基础设施层、工具层代码
- 测试代码

### 领域事件白名单 🟥

🟥 所有正式领域事件必须收录在白名单文档中（宪法 2.2.7、13.10.2）。新增领域事件必须先更新白名单。

---

## 1. 日志级别

### 1.1 五级日志体系 🟥

🟥 必须统一使用 `tracing` 库进行日志记录，禁止 `println!`、`dbg!`（宪法 13.1.1）。
🟥 日志必须记录业务事件事实，绝对禁止记录技术执行流水（宪法 13.1.2）。

| 级别 | 用途 | 典型场景 | 构建行为 |
|------|------|---------|---------|
| TRACE | 极细粒度算法细节 | A*节点探索、单帧循环内部逻辑 | Release 编译出 |
| DEBUG | 开发调试辅助信息 | 寻路路径长度、Modifier计算过程、资源加载详情 | Release 编译出 |
| INFO | 核心业务事件边界 | 战斗开始/结束、回合开始/结束、单位移动/攻击/死亡、Buff施加/移除、任务完成 | 保留 |
| WARN | 可恢复异常 | 资源加载失败（已降级）、配置校验警告 | 保留 |
| ERROR | 理论上不应发生的程序异常 | 数据损坏、状态机非法转换、校验失败 | 保留 |

> ⚠️ **注意**：DEBUG/TRACE 级别记录的是**业务数据的细化信息**（如路径长度、计算中间值），而非系统执行日志（如"xxx_system running"）。系统执行日志属于技术流水账，在任何级别都禁止（宪法 13.4）。

### 1.2 级别使用规范

#### TRACE 🟦

```rust
// ❌ 禁止：每帧系统中使用 TRACE 输出技术流水账
fn movement_system(query: Query<&GridPosition>) {
    for pos in &query {
        trace!("Position: {:?}", pos);  // ❌ 每帧每单位都打印技术细节
    }
}

// ✅ 允许：仅在调试时手动启用，记录业务数据的极细粒度信息
fn movement_system(query: Query<(&GridPosition, Changed<GridPosition>)>) {
    for pos in &query {
        trace!(x = pos.x, y = pos.y, "Unit grid position updated");  // ✅ 业务数据，非系统执行
    }
}
```

#### DEBUG 🟦

```rust
// ❌ 禁止：系统执行日志（技术流水账，宪法 13.4）
fn apply_damage_system() {
    debug!("apply_damage_system: processing 3 pending effects");  // ❌ 系统名称 = 技术流水
    // ...
    debug!("apply_damage_system: completed, 2 units damaged");    // ❌ 系统执行追踪
}

// ✅ 允许：业务数据的开发调试信息
fn apply_damage_system(query: Query<(&UnitId, &DamagePending)>) {
    for (unit_id, pending) in &query {
        debug!(
            unit_id = %unit_id,
            raw_damage = pending.raw,
            mitigated = pending.mitigated,
            "Damage calculation detail"  // ✅ 业务数据细化，非系统执行
        );
    }
}
```

#### INFO 🟥

🟥 所有 INFO 级别日志必须携带 `event` 字段，值与事件名完全一致（宪法 13.3.1）。
🟥 日志 `target` 必须与所属 Feature 目录名完全一致（宪法 13.3.2）。
🟥 绝对禁止业务代码直接调用 `info!` 输出核心业务事件，必须走领域事件链路（宪法 13.8.2）。

```rust
// ❌ 禁止：业务代码直接调用 info!（违反 13.8.2）
fn on_battle_start(stage: &StageConfig) {
    info!(
        stage = %stage.name,
        player_units = stage.player_count,
        enemy_units = stage.enemy_count,
        "Battle started"
    );
}

// ❌ 禁止：缺少 event 字段（违反 13.3.1）
info!(unit_id = %unit_id, "Unit attacked");

// ✅ 正确：通过领域事件链路触发，LogObserver 统一输出
// 业务代码只发送领域事件：
fn some_battle_system(mut events: EventWriter<BattleStarted>) {
    events.send(BattleStarted { stage_name, player_count, enemy_count });
}

// LogObserver 监听事件并输出日志：
fn log_battle_started(event: &BattleStarted, target: &str) {
    info!(
        event = "BattleStarted",                          // 宪法 13.3.1: event 字段
        target = target,                                   // 宪法 13.3.2: target = Feature 目录名
        stage = %event.stage_name,
        player_units = event.player_count,
        enemy_units = event.enemy_count,
        "Battle started"
    );
}
```

#### WARN 🟩

```rust
// ✅ 允许：可恢复异常（宪法 13.2）
fn load_texture(path: &Path) -> Handle<Texture> {
    match asset_server.load(path) {
        handle if handle.is_some() => handle,
        _ => {
            warn!(
                path = %path.display(),
                target = "asset",  // 宪法 13.3.2: target 匹配 Feature 目录名
                "Texture load failed, using fallback"
            );
            asset_server.load("fallback/placeholder.png")
        }
    }
}
```

#### ERROR 🟥

```rust
// ✅ 允许：致命错误（宪法 13.2，必须附带完整复现上下文）
fn validate_game_state(world: &World) {
    let hp = world.resource::<Attributes>().current_hp;
    if hp < 0 {
        error!(
            hp = hp,
            target = "battle",  // 宪法 13.3.2
            "Negative HP detected — data corruption"  // 宪法 13.2: 理论上不应发生
        );
        // 触发崩溃报告
    }
}
```

---

## 2. 日志基础设施

### 2.1 使用 tracing crate 🟥

🟥 **统一使用 `tracing` crate，禁止 `println!`、`dbg!`、`log` crate**（宪法 13.1.1）。

```rust
// Cargo.toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### 2.2 结构化字段 🟩

🟥 日志必须使用结构化字段，不使用字符串拼接。日志、错误信息中必须输出业务可读的字符串 ID，禁止直接打印 `Entity(xxx)` 原生格式（宪法 1.2.2）。

```rust
// ❌ 错误：字符串拼接
error!("Unit {} died at position ({}, {})", unit_id, x, y);

// ✅ 正确：结构化字段
error!(
    unit_id = unit_id.0,
    x = pos.x,
    y = pos.y,
    "Unit died"
);
```

### 2.3 日志初始化

```rust
// src/infrastructure/logging/plugin.rs
pub struct LoggingPlugin;

impl Plugin for LoggingPlugin {
    fn build(&self, app: &mut App) {
        // 初始化 tracing subscriber
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| EnvFilter::new("info"))
            )
            .init();
        
        // 注册日志 Observer
        app.add_observer(LogObserver);
    }
}
```

### 2.4 LogObserver 🟥

🟥 所有 INFO 级别核心业务事件必须通过触发领域事件的方式生成日志（宪法 13.8.2）。
🟥 日志 Observer 统一放在基础设施层，绝对不侵入业务模块（宪法 13.8.2）。

```rust
// 监听领域事件，生成日志记录
// 注意：每个事件是独立的 Struct（非 DomainEvent 大枚举）
fn log_observer_damage(trigger: Trigger<DamageDealt>, mut trail: ResMut<AuditTrail>) {
    let event = trigger.event();
    trail.record(event.to_audit_payload(), AuditMetadata { ... });
    
    // ✅ 正确：使用 Strong ID 的 Display 格式（宪法 1.2.2）
    // ✅ 正确：携带 event 字段（宪法 13.3.1）
    // ✅ 正确：携带 target 字段（宪法 13.3.2）
    info!(
        event = "DamageDealt",                    // 宪法 13.3.1
        target = "battle",                        // 宪法 13.3.2: Feature 目录名
        source = %event.source,                   // ✅ UnitId 的 Display: "Unit(warrior_001)"
        target_unit = %event.target,              // ✅ UnitId 的 Display: "Unit(goblin_01)"
        amount = event.amount,
        is_critical = event.is_critical,
        "unit received damage"
    );
}

fn log_observer_buff(trigger: Trigger<BuffApplied>, mut trail: ResMut<AuditTrail>) {
    let event = trigger.event();
    trail.record(event.to_audit_payload(), AuditMetadata { ... });
    
    info!(
        event = "BuffApplied",                    // 宪法 13.3.1
        target = "buff",                          // 宪法 13.3.2
        target_unit = %event.target,              // ✅ Strong ID Display
        buff_id = %event.buff_id,                 // ✅ BuffId 的 Display: "Buff(poison)"
        "buff applied to unit"
    );
}
```

> ⚠️ **注意**：此处 `info!` 调用位于 LogObserver（基础设施层），属于宪法 13.8.3 例外范围。业务层代码禁止直接调用 `info!` 输出核心业务事件。

---

## 3. 记录规则

### 3.1 必须记录的内容 🟥

🟥 INFO 日志必须记录核心业务事件边界（宪法 13.2）。所有 INFO 级别核心业务事件必须通过领域事件链路生成（宪法 13.8.2）。
🟥 所有正式领域事件必须收录在白名单文档中（宪法 2.2.7、13.10.2）。

| 事件类型 | 日志级别 | 结构化字段 |
|---------|---------|-----------|
| 战斗开始/结束 | INFO | stage, player_count, enemy_count |
| 角色死亡 | INFO | unit_id, killer_id, position |
| 技能释放 | INFO | caster_id, skill_id, target_id |
| Buff 施加/移除 | INFO | target_id, buff_id, source |
| 装备穿脱 | INFO | unit_id, equipment_id, slot |
| 状态转换 | DEBUG | from_state, to_state |
| 校验违规 | WARN/ERROR | violation_type, current_value, expected |
| 资源加载失败 | WARN | path, fallback_used |

### 3.2 禁止记录的内容 🟥

🟥 **每帧 ECS 查询结果**（性能杀手，宪法 13.4）
🟥 **内部系统状态 dump**（应使用 Debug Panel，宪法 13.7.1）
🟥 **完整 Entity 数据每个 tick**（应使用 Inspector，宪法 13.4）
🟥 **循环内日志**（应批量输出或使用采样，宪法 13.4）
🟥 **通过堆砌日志进行调试**（应使用 Replay/Debug Panel，宪法 13.7.1）
🟥 **函数进入/退出日志**（技术流水账，宪法 13.4）
🟥 **INFO/DEBUG 在 Release 每帧系统中**（仅允许 ERROR，宪法 13.4）

### 3.3 批量输出与采样策略 🟥

🟥 高频事件（如每帧触发的 ECS 查询）必须使用采样或批量输出，避免日志洪流（宪法 13.4）。

> ⚠️ **注意**：采样和批量输出仅适用于基础设施层日志。核心业务事件必须通过领域事件链路生成（宪法 13.8.2），不使用此采样模式。

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

/// 帧计数器，用于 INFO 级别采样
static FRAME_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// 每 100 帧采样 1 次 INFO 日志，避免日志洪流
fn sample_info_log(message: &str) {
    let frame = FRAME_COUNTER.fetch_add(1, Ordering::Relaxed);
    if frame % 100 == 0 {
        info!(
            frame = frame,
            sample_rate = 100,
            "{}", message
        );
    }
}

/// 批量输出示例：累计 N 条事件后批量打印
pub struct BatchLogger {
    buffer: Vec<String>,
    batch_size: usize,
}

impl BatchLogger {
    pub fn new(batch_size: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(batch_size),
            batch_size,
        }
    }

    pub fn push(&mut self, msg: String) {
        self.buffer.push(msg);
        if self.buffer.len() >= self.batch_size {
            self.flush();
        }
    }

    pub fn flush(&mut self) {
        if !self.buffer.is_empty() {
            info!(
                batch_size = self.buffer.len(),
                events = ?self.buffer,
                "Batch log output"
            );
            self.buffer.clear();
        }
    }
}
```

**采样规则**：
- TRACE/DEBUG：每帧系统中**禁止**（已通过 Feature Gate 编译出）
- INFO：高频事件采样 1/100，低频事件全量输出
- WARN/ERROR：全量输出，不采样

> **优化来源**：`docs/其他/56.md` — 批量输出/采样「每 100 帧 INFO 采样 1 次，避免日志洪流」

### 3.4 每帧系统日志规则 🟥

🟥 **每帧系统中仅允许 ERROR 级别日志**（宪法 13.4）。

```rust
// ❌ 禁止：每帧系统中使用 INFO/DEBUG
fn every_frame_system() {
    info!("Processing...");  // ❌ 每帧打印
    debug!("Step 1");        // ❌ 每帧打印
}

// ✅ 允许：每帧系统中只用 ERROR
fn every_frame_system() {
    if let Err(e) = do_something() {
        error!(error = %e, "Critical failure in frame system");
    }
}
```

---

## 4. 日志输出

### 4.1 输出目标

| 构建类型 | 输出目标 | 格式 |
|---------|---------|------|
| 开发构建 | stdout | 彩色格式化 |
| 测试构建 | stdout | 紧凑格式 |
| 发发构建 | 文件 | JSON 结构化 |
| Release 构建 | 文件 + 可选遥测 | JSON 结构化 |

### 4.2 日志文件管理

```
logs/
├── game_2026-06-13.log         # 当天日志
├── game_2026-06-12.log         # 昨天日志
└── game_2026-06-11.log         # 前天日志
```

- 🟩 日志文件按天滚动
- 🟩 保留最近 7 天日志
- 🟩 单个日志文件最大 10MB
- 🟩 超过大小限制时压缩旧日志

### 4.3 日志写入失败兜底策略

生产环境可能出现磁盘满、日志文件权限不足等异常场景。日志系统必须提供降级策略，避免日志丢失或程序崩溃：

| 异常场景 | 兜底策略 | 日志标记 |
|---------|---------|---------|
| 磁盘满 | 降级到 stdout 输出 | WARN 级别标记 `log_degraded=true` |
| 文件权限不足 | 降级到 stdout 输出 | WARN 级别标记 `log_degraded=true` |
| 格式化失败 | 输出原始消息（无结构化字段） | WARN 级别标记 `format_degraded=true` |
| 全部输出失败 | 静默丢弃（不阻塞业务逻辑） | 内存计数器记录丢弃数量 |

```rust
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// 带兜底的日志初始化
pub fn init_logging_with_fallback() {
    let file_layer = match init_file_logging() {
        Ok(layer) => Some(layer),
        Err(e) => {
            // 磁盘满或权限不足时降级到 stdout
            eprintln!("⚠️ [LOG DEGRADED] 日志文件初始化失败: {e}, 降级到 stdout");
            None
        }
    };

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_filter(tracing_subscriber::EnvFilter::from_default_env());

    let subscriber = tracing_subscriber::registry()
        .with(stdout_layer);

    let subscriber = match file_layer {
        Some(fl) => subscriber.with(fl),
        None => subscriber,
    };

    subscriber.init();
}

/// 磁盘满时的降级写入器
pub struct DegradedWriter {
    fallback_stdout: bool,
}

impl std::io::Write for DegradedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.fallback_stdout {
            // 降级到 stdout，附加 WARN 标记
            let mut output = b"[DEGRADED] ".to_vec();
            output.extend_from_slice(buf);
            std::io::stdout().write_all(&output)?;
            return Ok(buf.len());
        }
        Ok(0)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stdout().flush()
    }
}
```

> **优化来源**：`docs/其他/56.md` — 异常场景「磁盘满时降级到 stdout + WARN 标记」

### 4.4 日志过滤

```rust
// 开发环境：显示所有级别
RUST_LOG=debug cargo run

// 生产环境：只显示 INFO 及以上
RUST_LOG=info cargo run

// 调试特定模块
RUST_LOG=bevy_srpg::battle=debug cargo run
```

---

## 5. Feature Gate

### 5.1 级别编译控制

```rust
// Trace 和 Debug 日志在 Release 构建中编译出
#[cfg(debug_assertions)]
macro_rules! trace_log {
    ($($arg:tt)*) => {
        tracing::trace!($($arg)*)
    };
}

#[cfg(not(debug_assertions))]
macro_rules! trace_log {
    ($($arg:tt)*) => {};
}
```

### 5.2 性能影响

| 级别 | Debug 构建性能影响 | Release 构建性能影响 |
|------|-------------------|---------------------|
| TRACE | 高（格式化开销） | 无（编译出） |
| DEBUG | 中（格式化开销） | 无（编译出） |
| INFO | 低 | 低 |
| WARN | 低 | 低 |
| ERROR | 低 | 低 |

---

## 6. 审计集成

### 6.1 日志与审计的关系

日志是审计的下游消费者：

```
Core 系统 → DomainEvent Message → AuditTrail 收集
                                     ↓
                              LogObserver 监听
                                     ↓
                              Tracing 日志输出
```

### 6.2 审计日志 vs 调试日志

| 特征 | 审计日志 | 调试日志 |
|------|---------|---------|
| 目的 | 记录游戏事件供回放/分析 | 辅助开发调试 |
| 级别 | INFO/WARN/ERROR | TRACE/DEBUG |
| 持久化 | 永久保留 | 按天滚动 |
| 格式 | 结构化 JSON | 彩色格式化 |
| 性能影响 | 低（批量写入） | 中（实时格式化） |

---

## 7. 遥测接入与数据脱敏

### 7.1 tracing 对接 OpenTelemetry

Release 构建可选接入 OpenTelemetry，实现分布式追踪和远程监控：

```rust
// Cargo.toml
[dependencies]
tracing-opentelemetry = { version = "0.24", optional = true }
opentelemetry = { version = "0.22", optional = true }
opentelemetry-stdout = { version = "0.1", optional = true }

[features]
telemetry = ["tracing-opentelemetry", "opentelemetry", "opentelemetry-stdout"]
```

```rust
// 遥测初始化（feature gate 控制）
#[cfg(feature = "telemetry")]
pub fn init_telemetry() {
    use opentelemetry::sdk::trace::TracerProvider;
    use opentelemetry::sdk::export::trace::stdout::SpanExporter;

    let exporter = SpanExporter::default();
    let provider = TracerProvider::builder()
        .with_simple_exporter(exporter)
        .build();

    let tracer = provider.tracer("bevy_srpg");
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(telemetry)
        .init();
}

/// 未启用遥测时的空实现
#[cfg(not(feature = "telemetry"))]
pub fn init_telemetry() {
    // 遥测未启用，跳过
}
```

### 7.2 数据脱敏规则

日志和遥测数据必须脱敏，避免泄露敏感信息：

| 数据类型 | 脱敏方式 | 示例 |
|---------|---------|------|
| 玩家 UID | 哈希脱敏 | `uid=sha256(player_123)→a1b2c3...` |
| 设备信息 | 不记录 | 完全省略 |
| IP 地址 | 部分掩码 | `192.168.***.***` |
| 密钥/Token | 绝对禁止记录 | 一旦发现立即告警 |
| 存档路径 | 相对路径化 | `saves/game1.sav` → `saves/**` |
| MOD 配置 | 仅记录 ID | 不记录 MOD 内容细节 |

```rust
/// 脱敏工具函数
pub mod sanitization {
    use sha2::{Sha256, Digest};

    /// 哈希脱敏：将敏感 ID 转为不可逆哈希
    pub fn hash_mask(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)[..8].to_string() // 取前 8 位
    }

    /// 路径脱敏：只保留文件名
    pub fn path_mask(path: &str) -> String {
        std::path::Path::new(path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "***".to_string())
    }
}
```

> **优化来源**：`docs/其他/56.md` — 遥测接入「tracing 对接 OpenTelemetry + 数据脱敏规则」

---

## 8. 日志功能测试策略

### 8.1 业务事件日志断言

使用 `tracing_test` crate 断言业务事件触发对应级别日志：

```rust
// Cargo.toml
[dev-dependencies]
tracing-test = "0.2"
```

```rust
#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    /// 测试：战斗开始事件触发 INFO 级别日志
    #[traced_test]
    #[test]
    fn battle_start_logs_info() {
        let stage = StageConfig::new("test_stage", 2, 3);
        on_battle_start(&stage);

        // 断言：INFO 级别日志包含 "Battle started"
        assert!(logs_contain("Battle started"));
        // 断言：结构化字段包含 stage 名称
        assert!(logs_contain("test_stage"));
    }

    /// 测试：资源加载失败触发 WARN 级别日志
    #[traced_test]
    #[test]
    fn asset_load_failure_logs_warn() {
        let result = load_texture(Path::new("nonexistent.png"));
        assert!(logs_contain("Texture load failed"));
    }

    /// 测试：数据损坏触发 ERROR 级别日志
    #[traced_test]
    #[test]
    fn data_corruption_logs_error() {
        validate_negative_hp(-10);
        assert!(logs_contain("Negative HP detected"));
    }
}
```

### 8.2 日志文件滚动测试

```rust
#[test]
fn test_log_file_rotation() {
    // 写入超过 10MB 日志
    for _ in 0..100_000 {
        info!("Test log line for rotation testing");
    }

    // 验证：旧日志文件被压缩
    let logs_dir = std::fs::read_dir("logs/").unwrap();
    let gz_files: Vec<_> = logs_dir
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "gz").unwrap_or(false))
        .collect();

    assert!(!gz_files.is_empty(), "应有压缩的旧日志文件");
}
```

### 8.3 降级策略测试

```rust
#[test]
fn test_logging_fallback_on_disk_full() {
    // 模拟磁盘满场景（只读文件系统）
    let result = init_logging_to_readonly_path("/proc/nonexistent");
    assert!(result.is_err());

    // 验证：降级到 stdout 后日志仍可输出
    let subscriber = init_logging_with_fallback();
    // 降级模式下日志不应 panic
}
```

> **优化来源**：`docs/其他/56.md` — 测试策略「assert 业务事件触发对应级别日志 + tracing_test::with_default」

---

## 9. 多环境配置模板

### 9.1 环境变量配置

| 环境 | RUST_LOG 值 | 说明 |
|------|------------|------|
| 开发环境 | `debug` | 显示所有级别，彩色输出 |
| 测试环境 | `info` | 只显示 INFO 及以上 |
| 生产环境 | `info` | 只显示 INFO 及以上，JSON 格式 |
| 调试模式 | `bevy_srpg::battle=debug` | 调试特定模块 |

### 9.2 配置文件方案

大规模团队协作时，仅靠环境变量不够灵活。推荐使用配置文件：

```toml
# config/logging.toml — 多环境配置模板

[development]
level = "debug"
output = "stdout"
format = "pretty"
color = true

[testing]
level = "info"
output = "stdout"
format = "compact"
color = false

[production]
level = "info"
output = "file"
format = "json"
color = false
file_path = "logs/game.log"
max_file_size_mb = 10
max_files = 7

[staging]
level = "info"
output = "file+stdout"
format = "json"
color = false
file_path = "logs/game.log"
```

### 9.3 CLI 覆盖

```bash
# 环境变量覆盖配置文件
RUST_LOG=bevy_srpg::battle=debug cargo run

# 配置文件 + 环境变量组合
# 1. 先加载 config/logging.toml
# 2. RUST_LOG 环境变量覆盖 level 字段
# 3. CLI 参数覆盖具体模块级别

# 示例：生产环境 + 调试战斗模块
RUST_LOG=info,bevy_srpg::battle=debug cargo run --release
```

### 9.4 运行时动态调整（开发模式）

```rust
/// 开发模式下支持运行时调整日志级别
#[cfg(debug_assertions)]
pub fn adjust_log_level(level: &str) {
    // 通过修改 LoggingConfig 资源实现运行时调整
    // 注意：仅开发模式允许，生产模式禁止
    info!(new_level = level, "日志级别已调整");
}
```

> **优化来源**：`docs/其他/56.md` — 多环境配置「开发 RUST_LOG=debug，生产 RUST_LOG=info + 配置文件 + CLI 覆盖」

## 10. 禁止事项 🟥

🟥 **使用 `println!` 或 `eprintln!`**（必须使用 tracing 宏，宪法 13.1.1）
🟥 **使用 `dbg!` 宏**（必须使用 tracing 宏，宪法 13.1.1）
🟥 **使用 `log` crate**（必须使用 `tracing` crate，宪法 13.1.1）
🟥 **每帧系统中打印 INFO/DEBUG 日志**（宪法 13.4）
🟥 **循环内日志**（应批量输出或采样，宪法 13.4）
🟥 **通过堆砌日志进行调试**（应使用 Inspector/Replay/Debug Panel，宪法 13.7.1）
🟥 **日志中包含 PII（个人身份信息）**
🟥 **日志中包含完整 Entity 数据每个 tick**（宪法 13.4）
🟥 **日志文件无限增长**（必须有滚动和清理策略）
🟥 **日志格式化开销影响帧率**（应使用结构化字段）
🟥 **业务代码直接调用 `info!` 输出核心业务事件**（必须走领域事件链路，宪法 13.8.2）
🟥 **函数进入/退出日志**（技术流水账，宪法 13.4）
🟥 **INFO/DEBUG 在循环、迭代器内部**（宪法 13.4）

---

## 11. 实现备注

### 8.1 日志模块结构

```
src/infrastructure/logging/
├── mod.rs           # 模块入口
├── events.rs        # 日志相关事件定义
└── observer.rs      # LogObserver 实现
```

### 8.2 日志配置

```rust
// 日志级别配置
pub struct LoggingConfig {
    pub level: LevelFilter,
    pub file_output: bool,
    pub file_path: PathBuf,
    pub max_file_size: u64,
    pub max_files: usize,
}
```

### 8.3 性能监控日志

```rust
// 关键 System 执行时间日志
fn timed_system(start: Res<Time>) {
    let start_time = start.elapsed();
    // ... 逻辑 ...
    let elapsed = start.elapsed() - start_time;
    if elapsed > Duration::from_millis(1) {
        warn!(
            system = "timed_system",
            elapsed_ms = elapsed.as_secs_f64() * 1000.0,
            "System exceeded 1ms budget"
        );
    }
}
```

---

## 12. 与其他文档的关系

| 文档 | 关系 |
|------|------|
| `architecture.md` | 本文档是"Logging"章节的详细补充 |
| `infrastructure-design.md` | logging 模块的技术设计 |
| `performance_budget.md` | 日志性能影响是性能预算的子集 |
| `docs/01-architecture/events_audit_design.md` | 日志是审计的下游消费者 |
| `crash_report_rules.md` | 崩溃前日志记录 |
