# Logging Design — 日志系统设计

Version: 1.0
Status: Proposed

来源：`docs/其他/31遗漏.md` 第四节 — 日志系统设计

本文档定义结构化、分层的日志架构，涵盖日志级别、基础设施、记录规则和输出策略。

交叉引用：
- `docs/architecture.md` — 日志总纲（tracing 统一、禁止 println）
- `docs/architecture/infrastructure-design.md` — logging 模块设计

---

## 1. 日志级别

### 1.1 五级日志体系

| 级别 | 用途 | 典型场景 | 构建行为 |
|------|------|---------|---------|
| TRACE | 每帧细节 | ECS 查询结果、系统执行顺序 | Release 编译出 |
| DEBUG | 系统执行 | 系统进入/退出、状态转换 | Release 编译出 |
| INFO | 重要事件 | 战斗开始、关卡完成、角色升级 | 保留 |
| WARN | 可恢复异常 | 资源加载失败（已降级）、配置校验警告 | 保留 |
| ERROR | 致命错误 | 数据损坏、状态机非法转换、校验失败 | 保留 |

### 1.2 级别使用规范

#### TRACE

```rust
// ❌ 禁止：每帧系统中使用 TRACE
fn movement_system(query: Query<&GridPosition>) {
    for pos in &query {
        trace!("Position: {:?}", pos);  // 每帧每单位都打印
    }
}

// ✅ 允许：仅在调试时手动启用
fn movement_system(query: Query<(&GridPosition, Changed<GridPosition>)>) {
    for pos in &query {
        trace!(x = pos.x, y = pos.y, "Unit moved");
    }
}
```

#### DEBUG

```rust
// ✅ 允许：系统执行追踪
fn apply_damage_system() {
    debug!("apply_damage_system: processing 3 pending effects");
    // ... 逻辑 ...
    debug!("apply_damage_system: completed, 2 units damaged");
}
```

#### INFO

```rust
// ✅ 允许：重要游戏事件
fn on_battle_start(stage: &StageConfig) {
    info!(
        stage = %stage.name,
        player_units = stage.player_count,
        enemy_units = stage.enemy_count,
        "Battle started"
    );
}
```

#### WARN

```rust
// ✅ 允许：可恢复异常
fn load_texture(path: &Path) -> Handle<Texture> {
    match asset_server.load(path) {
        handle if handle.is_some() => handle,
        _ => {
            warn!(
                path = %path.display(),
                "Texture load failed, using fallback"
            );
            asset_server.load("fallback/placeholder.png")
        }
    }
}
```

#### ERROR

```rust
// ✅ 允许：致命错误
fn validate_game_state(world: &World) {
    let hp = world.resource::<Attributes>().current_hp;
    if hp < 0 {
        error!(
            hp = hp,
            "Negative HP detected — data corruption"
        );
        // 触发崩溃报告
    }
}
```

---

## 2. 日志基础设施

### 2.1 使用 tracing crate

🟥 **统一使用 `tracing` crate，禁止 `println!`、`dbg!`、`log` crate**。

```rust
// Cargo.toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### 2.2 结构化字段

🟩 日志必须使用结构化字段，不使用字符串拼接。

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

### 2.4 LogObserver

```rust
// 监听领域事件，生成日志记录
fn log_observer(
    trigger: Trigger<DomainEvent>,
    mut trail: ResMut<AuditTrail>,
) {
    let event = trigger.event();
    trail.record(event.clone());
    
    match event {
        DomainEvent::DamageDealt { source, target, amount } => {
            info!(
                source = source.0,
                target = target.0,
                amount = amount,
                "Damage dealt"
            );
        }
        DomainEvent::BuffApplied { target, buff_id } => {
            info!(
                target = target.0,
                buff_id = %buff_id,
                "Buff applied"
            );
        }
        // ...
    }
}
```

---

## 3. 记录规则

### 3.1 必须记录的内容

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

### 3.2 禁止记录的内容

- 🟥 **每帧 ECS 查询结果**（性能杀手）
- 🟥 **内部系统状态 dump**（应使用 Debug Panel）
- 🟥 **完整 Entity 数据每个 tick**（应使用 Inspector）
- 🟥 **循环内日志**（应批量输出或使用采样）
- 🟥 **通过堆砌日志进行调试**（应使用 Replay/Debug Panel）

### 3.3 每帧系统日志规则

🟥 **每帧系统中仅允许 ERROR 级别日志**。

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

### 4.3 日志过滤

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

## 7. 禁止事项

- 🟥 **使用 `println!` 或 `eprintln!`**（必须使用 tracing 宏）
- 🟥 **使用 `dbg!` 宏**（必须使用 tracing 宏）
- 🟥 **使用 `log` crate**（必须使用 `tracing` crate）
- 🟥 **每帧系统中打印 INFO/DEBUG 日志**
- 🟥 **循环内日志**（应批量输出或采样）
- 🟥 **通过堆砌日志进行调试**（应使用 Inspector/Replay/Debug Panel）
- 🟥 **日志中包含 PII（个人身份信息）**
- 🟥 **日志中包含完整 Entity 数据每个 tick**
- 🟥 **日志文件无限增长**（必须有滚动和清理策略）
- 🟥 **日志格式化开销影响帧率**（应使用结构化字段）

---

## 8. 实现备注

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

## 9. 与其他文档的关系

| 文档 | 关系 |
|------|------|
| `architecture.md` | 本文档是"Logging"章节的详细补充 |
| `infrastructure-design.md` | logging 模块的技术设计 |
| `performance_budget.md` | 日志性能影响是性能预算的子集 |
| `audit_design.md` | 日志是审计的下游消费者 |
| `crash_report_rules.md` | 崩溃前日志记录 |
