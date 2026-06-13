# Performance Budget — 性能预算与优化基线

Version: 1.0
Status: Proposed

来源：`docs/其他/31遗漏.md` 第三节 — 性能预算与优化基线

本文档定义游戏各模块的帧预算、优化规则和性能门禁标准。

交叉引用：
- `docs/architecture.md` — 性能总纲（先正确再优化、先 Profile 再优化）
- `docs/architecture/infrastructure-design.md` — Profiler 和 Diagnostics 模块设计

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

### 6.2 性能分析工具

🟩 推荐使用以下工具进行性能分析：

- **Bevy Inspector**：`F12` 打开 World Inspector，查看 System 执行时间
- **tracing**：结构化日志记录 System 执行时间
- **puffin**：帧级性能分析
- **cargo bench**：基准测试

### 6.3 性能日志

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
