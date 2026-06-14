---
id: 01-architecture.performance-budget
title: Performance Budget
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
---

# Performance Budget — 性能预算与优化基线

Version: 1.1
Status: Proposed

来源：`docs/其他/31遗漏.md` 第三节 — 性能预算与优化基线

本文档定义游戏各模块的帧预算、优化规则和性能门禁标准。

### 宪法条款映射

| 本文档规则 | 宪法条款 | 强制等级 |
|-----------|---------|---------|
| §0 基础原则 | 🟥 15.0.1 正确性优先 | 最高优先级 |
| §5.3 优化决策 | 🟥 15.0.2 测量优先（禁止凭直觉优化） | 必须遵循 |
| §5.3 优化决策 | 🟥 15.0.3 优化热点代码（禁止全局重构） | 必须遵循 |
| §7 禁止事项 | 🟥 15.0.4 可读性优先（除非 Profile 证明） | 必须遵循 |
| §4.2 变更检测 | 🟥 15.0.5 ECS 优化（优先使用 Changed 过滤器） | 必须遵循 |
| §7 禁止事项 | 🟥 15.0.6 Reflect 限制（禁止在热路径使用） | 必须遵循 |
| §6.2 缓存 Key 设计 | 🟥 15.0.7 缓存通用规范 | 必须遵循 |
| §0 基础原则 | 🟥 15.0.9 复杂度优先于性能 | 必须遵循 |

交叉引用：
- `docs/01-architecture/README.md` — 性能总纲（先正确再优化、先 Profile 再优化）
- `docs/01-architecture/infrastructure-design.md` — Profiler 和 Diagnostics 模块设计

---

## 0. 基础原则（宪法 §15 最高优先级）

🟥 **正确性优先于性能（§15.0.1）**：必须先保证代码正确，再考虑性能优化。

🟥 **测量优先（§15.0.2）**：所有性能问题必须通过 Profile 测量确认。禁止凭直觉进行性能优化。

🟥 **热点优化（§15.0.3）**：必须优先优化热点代码，禁止为了性能进行全局重构。

🟥 **可读性优先（§15.0.4）**：禁止为了性能牺牲代码可读性，除非有明确的 Profile 数据证明该部分是性能瓶颈。

🟥 **Reflect 限制（§15.0.6）**：绝对禁止在高频计算路径中使用 Reflect。Reflect 仅用于工具链支持（编辑器、Inspector），不用于运行时计算。

🟥 **复杂度优先（§15.0.9）**：大多数独立游戏死于复杂度，而非性能。架构复杂度预算优先级高于性能优化预算。

🟥 **缓存规范（§15.0.7）**：所有缓存必须明确定义失效条件与重建方式。缓存永远不是事实源，必须允许随时删除且不影响正确性。禁止缓存成为数据的唯一存储位置。

---

## 1. 目标帧率

### 1.1 基准目标

| 指标 | 目标值 | 说明 |
|------|--------|------|
| 目标帧率 | 60 FPS | 稳定 60 帧，允许偶发掉帧 |
| 单帧预算 | 16.67ms | 60 FPS 的理论帧间隔 |
| 可用预算 | ≤14ms | 预留 2.67ms 给系统调度和渲染 |
| 最低可接受 | 30 FPS | 低于此帧率视为性能问题 |

### 1.2 帧时间构成

```
单帧 16.67ms
├── 输入处理          ≤1ms
├── ECS 调度/执行     ≤9ms（游戏逻辑）
├── 渲染              ≤5ms
├── 系统开销          ≤1.67ms
└── 余量              ~0ms
```

---

## 2. 模块预算

### 2.1 游戏逻辑模块

| 模块 | 预算 | 说明 |
|------|------|------|
| 战斗逻辑（Effect Pipeline） | <5ms | 伤害计算、Buff 结算 |
| AI 决策 | <2ms | 策略选择、目标评估 |
| 寻路计算 | <2ms | BFS + 地形消耗 |
| 回合管理 | <1ms | 状态机转换、队列操作 |
| 属性计算 | <1ms | Modifier Stack 求值 |

### 2.2 表现层模块

| 模块 | 预算 | 说明 |
|------|------|------|
| UI 渲染 | <3ms | 面板更新、飘字动画 |
| 动画系统 | <2ms | 精灵动画播放 |
| 音频系统 | <1ms | 音效触发 |

### 2.3 后台模块

| 模块 | 预算 | 说明 |
|------|------|------|
| 资源加载（异步） | <5ms/帧 | 后台加载不阻塞主线程 |
| 日志记录 | <0.5ms | 结构化日志输出 |
| ViewModel 更新 | <2ms | 数据提取和格式化 |

### 2.4 预算总览

```
游戏逻辑总计：  5 + 2 + 2 + 1 + 1 = 11ms
表现层总计：    3 + 2 + 1 = 6ms
后台总计：      5 + 0.5 + 2 = 7.5ms
────────────────────────────────────
最大单帧占用：  11 + 6 + 7.5 = 24.5ms（超出预算！）
```

**说明**：后台模块与游戏逻辑/表现层并行执行，实际单帧占用为 max(11, 6) + 后台增量 ≈ 13ms，在预算内。

### 2.5 场景分级预算

不同战斗场景的复杂度差异巨大，需要分级预算：

| 场景类型 | 条件 | 战斗逻辑预算 | AI 预算 | 寻路预算 | 说明 |
|---------|------|------------|---------|---------|------|
| 小规模战斗 | ≤ 10 单位 | < 5ms | < 2ms | < 2ms | 标准预算 |
| 中规模战斗 | 11-30 单位 | < 6ms | < 3ms | < 3ms | 允许轻微超支 |
| 大规模团战 | > 30 单位 | < 8ms | < 4ms | < 4ms | 分帧计算兜底 |
| UI 模式 | 无战斗实体 | < 1ms | — | — | 仅 UI 渲染和数据展示 |

**分帧执行触发条件**：
- 单帧战斗逻辑预算超支时，自动将非关键计算延迟到下一帧
- 使用 `FrameCounter` 和 `Timer` 控制分帧节奏

> **优化来源**: `docs/其他/60.md`（场景分级预算建议）

### 2.6 设备分级预算表

不同设备的性能差异需要差异化预算：

| 设备级别 | 目标帧率 | 单帧预算 | 战斗逻辑 | AI 决策 | 寻路 | UI 渲染 |
|---------|---------|---------|---------|---------|------|---------|
| PC 高配 | 120 FPS | 8.33ms | < 4ms | < 1.5ms | < 1.5ms | < 2ms |
| PC 标准 | 60 FPS | 16.67ms | < 5ms | < 2ms | < 2ms | < 3ms |
| 移动端 | 60 FPS | 16.67ms | < 5ms | < 2ms | < 2ms | < 3ms |
| 低配机 | 30 FPS | 33.33ms | < 8ms | < 4ms | < 4ms | < 5ms |

```rust
/// 设备级别枚举
pub enum DeviceTier {
    HighEnd,    // PC 高配：120FPS 目标
    MidRange,   // PC 标准/移动端：60FPS 目标
    LowEnd,     // 低配机：30FPS 目标
}

impl DeviceTier {
    pub fn frame_budget_ms(&self) -> f64 {
        match self {
            Self::HighEnd => 8.33,
            Self::MidRange => 16.67,
            Self::LowEnd => 33.33,
        }
    }
}
```

> **优化来源**: `docs/其他/60.md`（设备分级预算表建议）

---

## 3. 堆内存分配规则

### 3.1 主更新循环规则

🟥 **主更新循环中禁止堆内存分配**。

**规则**：
- 🟥 禁止在 `Update` Schedule 的 System 中使用 `Vec::new()`、`HashMap::new()`、`String::from()` 等堆分配操作
- 🟥 禁止在 System 中调用 `Box::new()`、`Rc::new()` 等堆分配
- 🟥 禁止在高频 System（每帧执行）中创建临时集合

### 3.2 允许的替代方案

| 场景 | 禁止 | 允许 |
|------|------|------|
| 临时收集结果 | `Vec::new()` + push | 预分配的 `Local<Vec<T>>` |
| 字符串拼接 | `format!()` 创建新 String | 预分配的 `Local<String>` |
| 错误消息 | `format!()` 构造错误 | 使用 `&'static str` 错误描述 |
| 日志消息 | `format!()` 构造日志 | 结构化字段 + `tracing` 宏 |

### 3.3 延迟分配

🟩 需要分配内存的操作应使用 Bevy Commands 的延迟分配：

```rust
// ✅ 正确：延迟分配
fn my_system(mut commands: Commands) {
    commands.spawn(MyComponent { /* ... */ });
}

// ❌ 错误：立即分配
fn my_system() {
    let v = Vec::new(); // 堆分配！
}
```

### 3.4 内存预算管理

堆内存分配规则之外，需要整体内存预算管控：

| 指标 | 预算 | 说明 |
|------|------|------|
| 堆内存总预算 | 512MB | 游戏运行时最大堆内存占用 |
| Resource 总数上限 | 2000 | Bevy World 中 Resource 组件总数 |
| Entity 上限 | 50,000 | 单场景最大 Entity 数量 |
| 资源回收阈值 | 128MB | 累计分配超过此阈值时触发资源回收（Despawn 无用 Entity、卸载未使用资产） |

```rust
/// 内存预算常量
const HEAP_BUDGET: usize = 512 * 1024 * 1024;       // 512MB
const RESOURCE_RECLAIM_THRESHOLD: usize = 128 * 1024 * 1024; // 128MB 触发资源回收
const RESOURCE_COUNT_LIMIT: usize = 2000;
const ENTITY_COUNT_LIMIT: usize = 50_000;

/// 内存监控系统（每 N 帧执行一次）
fn monitor_memory_usage(world: &World) {
    let entity_count = world.iter_entities().count();
    if entity_count > ENTITY_COUNT_LIMIT {
        warn!(
            count = entity_count,
            limit = ENTITY_COUNT_LIMIT,
            "Entity count approaching limit"
        );
    }
}
```

> **优化来源**: `docs/其他/60.md`（内存预算管理建议）

---

## 4. ECS 查询优化

### 4.1 查询粒度

🟥 **禁止大型多组件查询**。

```rust
// ❌ 错误：查询过多组件
fn bad_system(query: Query<(&Unit, &Attributes, &ActiveBuffs, &SkillSlots, 
    &EquipmentSlots, &GridPosition, &Faction, &UnitName)>) { ... }

// ✅ 正确：小而专注的查询
fn good_system(query: Query<(&Attributes, &ActiveBuffs), Changed<Attributes>>) { ... }
```

**规则**：
- 🟩 单个 System 的查询组件数不超过 6 个
- 🟩 优先使用 `Changed<T>` 和 `Added<T>` 过滤器
- 🟩 将大型查询拆分为多个小查询
- 🟥 禁止每帧查询所有 Entity（使用过滤器缩小范围）

### 4.2 变更检测

🟥 **必须优先使用 Bevy 原生变更检测**。

```rust
// ✅ 正确：使用 Changed 过滤器
fn update_view(
    query: Query<(&Attributes, &UnitName), Changed<Attributes>>,
    mut view: ResMut<SelectedUnitView>,
) {
    for (attrs, name) in &query {
        view.update(name, attrs);
    }
}

// ❌ 错误：每帧全量更新
fn update_view_bad(
    query: Query<(&Attributes, &UnitName)>,
    mut view: ResMut<SelectedUnitView>,
) {
    for (attrs, name) in &query {
        view.update(name, attrs); // 每帧都执行，浪费性能
    }
}
```

### 4.3 查询优化检查表

- [ ] 是否使用了 `Changed<T>` / `Added<T>` 过滤器？
- [ ] 查询的组件数量是否 ≤ 6 个？
- [ ] 是否避免了全 Entity 扫描？
- [ ] 是否使用了 `Without<T>` 过滤器排除不需要的 Entity？
- [ ] 查询结果是否被缓存（如果多处使用）？

### 4.4 ECS 进阶技巧

#### ParQuery 使用场景

对于无共享数据的大规模查询，使用 `par_iter()` 并行处理：

```rust
/// 并行处理所有单位的属性计算（无共享可变状态）
fn parallel_attribute_calculation(
    query: Query<(&mut Attributes, &ActiveBuffs), Changed<Attributes>>,
) {
    query.par_iter_mut().for_each(|(mut attrs, buffs)| {
        // 每个单位独立计算，无数据竞争
        attrs.recalculate(buffs);
    });
}
```

**适用条件**：
- 🟩 查询结果之间无数据依赖
- 🟩 无共享的可变状态（`ResMut` / `&mut`）
- 🟩 实体数量 > 100（小规模并行开销大于收益）
- 🟥 有共享资源写入时禁止并行

#### Table vs SparseSet 选择

| 组件类型 | 推荐存储 | 原因 |
|---------|---------|------|
| 高频查询组件（如 `GridPosition`） | Table（默认） | 连续内存，缓存友好 |
| 低频/稀疏组件（如 `DebugMarker`） | SparseSet | 避免 Table 碎片化 |
| 大型组件（如 `SpriteBundle`） | SparseSet | 避免 Table 行过宽 |
| 标签组件（如 `Player`） | Table | 查询频繁，需连续遍历 |

```rust
/// 标记组件存储方式
#[derive(Component, SparseSet)]  // 低频访问，使用 SparseSet
pub struct DebugMarker;

#[derive(Component)]  // 高频查询，使用默认 Table
pub struct GridPosition;
```

#### SystemSet 调度优化

通过 `SystemSet` 分组减少依赖阻塞，提升并行度：

```rust
/// 将无依赖的 System 分组并行执行
app.add_systems(Update, (
    (calculate_ranges, update_pathfinding_stats).in_set(PathfindingSet),
    (update_ui, update_animations).in_set(RenderSet),
    (process_ai_decisions).in_set(AiSet),
).chain());  // Set 之间串行，Set 内部并行
```

> **优化来源**: `docs/其他/60.md`（ECS 进阶技巧建议）

---

## 5. 性能门禁

### 5.1 新增系统审查

🟥 **任何新增系统如果使帧时间增加 >1ms，需要显式审批**。

**审查流程**：

```
1. 新增 System 开发完成
2. 运行性能基准测试
3. 如果帧时间增加 ≤1ms → 自动通过
4. 如果帧时间增加 >1ms → 需要架构审查
5. 审查通过后才能合并
```

### 5.2 性能回归检测

🟩 CI 中集成性能基准测试：

- 每次 PR 运行性能基准
- 帧时间回归超过 10% 时自动标记
- 关键 System 的执行时间监控

### 5.3 性能优化决策

```
发现性能问题
  ↓
Profile 定位瓶颈
  ↓
├── 是热点代码 → 优化热点
├── 不是热点 → 不优化
└── 不确定 → 先 Profile 再决定
```

### 5.4 性能告警处理闭环

告警触发后必须形成完整的处理闭环：

```
检测（puffin/tracy）
    ↓ 帧时间超阈值
告警（WARN log + 指标记录）
    ↓ 自动触发
定位（performance profile 快照）
    ↓ 分析瓶颈
修复（针对性优化代码）
    ↓ 提交 PR
验证（benchmark 回归测试）
    ↓ 通过
关闭告警
```

**告警级别定义**：

| 级别 | 条件 | 处理方式 | SLA |
|------|------|---------|-----|
| WARN | 帧时间超预算 10%-50% | 记录日志，下次迭代修复 | 3 天内 |
| ERROR | 帧时间超预算 50%-100% | 立即定位，当前迭代修复 | 1 天内 |
| CRITICAL | 帧时间超预算 >100% | 阻止合并，立即修复 | 当天 |

```rust
/// 性能告警系统
fn performance_alert_system(
    diagnostics: Res<FrameTimeDiagnosticsPlugin>,
    time: Res<Time>,
) {
    let avg_frame_ms = diagnostics.average_frame_time()
        .map(|d| d.as_secs_f64() * 1000.0)
        .unwrap_or(0.0);
    
    if avg_frame_ms > 33.33 {  // 30FPS 以下
        error!(frame_ms = avg_frame_ms, "CRITICAL: Frame time severely over budget");
    } else if avg_frame_ms > 25.0 {  // 超预算 50%
        error!(frame_ms = avg_frame_ms, "ERROR: Frame time significantly over budget");
    } else if avg_frame_ms > 18.34 {  // 超预算 10%
        warn!(frame_ms = avg_frame_ms, "WARN: Frame time over budget");
    }
}
```

> **优化来源**: `docs/其他/60.md`（性能告警处理闭环建议）

---

## 6. 瓶颈检测

### 6.1 常见瓶颈模式

| 瓶颈类型 | 表现 | 解决方案 |
|---------|------|---------|
| AI 评估过重 | AI 回合帧时间 >5ms | 分帧评估、异步计算 |
| 范围计算过频 | 每帧重算移动/攻击范围 | 缓存范围结果 |
| 链式查询 | 多个 System 串行执行 | 合并 System、优化依赖 |
| 大量 Entity 查询 | Entity 数量多时帧时间飙升 | 使用过滤器、分组处理 |
| 变更检测缺失 | 每帧全量更新 UI | 添加 Changed 过滤器 |

### 6.2 常见瓶颈代码级案例

#### AI 分帧 Timer

避免 AI 回合一次性评估所有单位导致帧时间飙升：

```rust
/// AI 分帧评估 — 每帧最多处理 N 个 AI 单位
#[derive(Resource)]
pub struct AiScheduler {
    pub timer: Timer,
    pub units_per_frame: usize,
}

impl Default for AiScheduler {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.016, TimerMode::Repeating),  // 每帧触发
            units_per_frame: 4,  // 每帧最多处理 4 个 AI 单位
        }
    }
}

fn process_ai_decisions(
    time: Res<Time>,
    mut scheduler: ResMut<AiScheduler>,
    mut query: Query<(Entity, &mut AiDecision), With<Enemy>>,
) {
    scheduler.timer.tick(time.delta());
    if !scheduler.timer.just_finished() { return; }
    
    let mut processed = 0;
    for (entity, mut decision) in &mut query {
        if processed >= scheduler.units_per_frame { break; }
        decision.evaluate();
        processed += 1;
    }
}
```

#### 寻路缓存 Key 设计

缓存 Key 的设计直接影响命中率：

```rust
/// ❌ 错误：Key 过于宽泛，导致无效缓存
type BadCacheKey = Entity;

/// ✅ 正确：Key 包含影响范围的所有因素
#[derive(Hash, Eq, PartialEq)]
struct MoveRangeCacheKey {
    entity: Entity,
    position: IVec2,       // 单位当前位置
    move_points: u32,      // 当前剩余移动力
    terrain_version: u64,  // 地形版本号（地形变化时递增）
    blocker_version: u64,  // 阻挡版本号（单位移动时递增）
}
```

#### UI 脏标记避免每帧重绘

> **宪法条款**: 🟥 §2.3.3 禁止手写 bool 脏标记检测组件变化（必须使用 Bevy 原生 Added/Changed/Removed 过滤器）。
> 以下 UI 脏标记用于优化 UI 渲染频率，**不是**用于检测组件生命周期变化，两者用途不同。

使用脏标记（dirty flag）避免 UI 每帧全量重绘：

```rust
/// ✅ 正确：脏标记驱动的 UI 更新
#[derive(Resource)]
pub struct UiDirtyFlags {
    pub stats_panel: bool,
    pub minimap: bool,
    pub action_menu: bool,
}

fn update_stats_panel(
    mut dirty: ResMut<UiDirtyFlags>,
    query: Query<&Attributes, Changed<Attributes>>,
    mut panel: ResMut<StatsPanel>,
) {
    if !dirty.stats_panel { return; }  // 未标记脏，跳过
    
    for attrs in &query {
        panel.update(attrs);
    }
    dirty.stats_panel = false;  // 清除脏标记
}
```

> **优化来源**: `docs/其他/60.md`（常见瓶颈代码级案例建议）

### 6.3 性能分析工具

🟩 推荐使用以下工具进行性能分析：

- **Bevy Inspector**：`F12` 打开 World Inspector，查看 System 执行时间
- **tracing**：结构化日志记录 System 执行时间
- **puffin**：帧级性能分析
- **cargo bench**：基准测试

### 6.4 性能日志

🟩 关键 System 应记录执行时间：

```rust
fn expensive_system(start: Res<Time>) {
    let start_time = start.elapsed();
    // ... 逻辑 ...
    let elapsed = start.elapsed() - start_time;
    if elapsed > Duration::from_millis(1) {
        warn!(
            system = "expensive_system",
            elapsed_ms = elapsed.as_secs_f64() * 1000.0,
            "System exceeded 1ms budget"
        );
    }
}
```

---

## 7. 禁止事项

- 🟥 **主循环堆内存分配**（使用预分配或延迟分配）
- 🟥 **无过滤器的全 Entity 查询**（必须使用 Changed/Added/Without 等过滤器）
- 🟥 **寻路每帧重算**（缓存范围结果，仅在状态变化时重算）
- 🟥 **凭直觉进行性能优化**（必须先 Profile）
- 🟥 **未 Profile 就全局重构**（优先优化热点代码）
- 🟥 **新增 System 使帧时间增加 >1ms 而不审批**
- 🟥 **为了性能牺牲代码可读性**（除非有 Profile 数据证明该部分是瓶颈）
- 🟥 **在每帧 System 中打印 Info/Debug 日志**

---

## 8. 实现备注

### 8.1 性能基准测试

```rust
// benches/game_logic_bench.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_damage_calculation(c: &mut Criterion) {
    c.bench_function("damage_calculation_100_units", |b| {
        b.iter(|| {
            // 100 个单位的伤害计算
        })
    });
}

criterion_group!(benches, bench_damage_calculation);
criterion_main!(benches);
```

### 8.2 帧时间监控

```rust
fn frame_time_monitor(time: Res<Time>, diagnostics: Res<FrameTimeDiagnosticsPlugin>) {
    let avg = diagnostics.average_frame_time().unwrap_or_default();
    if avg > Duration::from_millis(16.67) {
        warn!(
            avg_frame_ms = avg.as_secs_f64() * 1000.0,
            "Frame time exceeds budget"
        );
    }
}
```

---

## 9. 与其他文档的关系

| 文档 | 关系 |
|------|------|
| `architecture.md` | 本文档是"性能"章节的详细补充 |
| `infrastructure-design.md` | Profiler/Diagnostics 模块实现性能监控 |
| `asset_lifecycle_rules.md` | 资源卸载策略影响内存性能 |
| `ui_architecture_rules.md` | ViewModel 更新频率影响 UI 性能 |
| `validation_rules.md` | 校验逻辑的执行频率影响性能 |
