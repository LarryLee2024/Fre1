# Contiguous Query 连续查询与性能优化

## 1. 新特性概述

Bevy 0.19 新增 `contiguous_iter` 和 `contiguous_iter_mut`，暴露底层连续内存布局，让 LLVM 自动向量化或手动 SIMD 优化成为可能。

传统 `Query` 迭代器按实体逐个访问，每次都需要通过 Archetype 跳转，无法保证内存连续性。而 `contiguous_iter` 直接返回底层 Table 中连续存储的 Component 切片（`&[T]` / `&mut [T]`），使得：

- **LLVM 可以自动进行 SIMD 向量化**：连续内存 + 无分支 = 理想的向量化条件
- **CPU 缓存命中率大幅提升**：顺序访问比跳跃访问快得多
- **手动 SIMD 优化成为可能**：拿到 `&mut [T]` 后可以直接用 `std::simd` 或 `packed_simd`

对于拥有大量同类型 Component 的战棋项目（Health/Attribute/Buff/Effect 等），这意味着批量运算场景下可获得 **2-3 倍性能提升**（AVX2 开启时更可达 **3-4 倍**）。

---

## 2. API 详解

### 2.1 contiguous_iter

返回只读连续切片迭代器，适用于纯读取场景。

```rust
fn query_health(query: Query<&Health>) {
    for healths in query.contiguous_iter().unwrap() {
        for h in healths {
            println!("{}", h.0);
        }
    }
}
```

**返回类型**：`ContiguousIter<'_, T>`，每次迭代产出 `&[T]`（一个 Archetype 内所有匹配实体的连续切片）。

**关键点**：
- 外层 `for` 遍历不同的 Archetype（相同 Component 组合的实体集合）
- 内层 `for` 遍历同一 Archetype 内的连续 Component 数据
- 如果所有实体属于同一 Archetype，外层循环只执行一次

### 2.2 contiguous_iter_mut

返回可变连续切片迭代器，适用于批量写入场景。

```rust
fn apply_health_decay(mut query: Query<(&mut Health, &HealthDecay)>) {
    for (mut health, decay) in query.contiguous_iter_mut().unwrap() {
        for (h, d) in health.iter_mut().zip(decay) {
            h.0 *= d.0;
        }
    }
}
```

**返回类型**：`ContiguousIterMut<'_, T>`，每次迭代产出 `Mut<[T]>`（可变切片，带 Change Detection）。

**关键点**：
- 多个 `&mut` 切片同时可变借用，安全因为它们来自不同 Component 列
- `zip` 组合多个切片进行逐元素运算，这是最典型的使用模式
- 仍然触发 Change Detection（每次写入都会标记）

### 2.3 bypass_change_detection

跳过变更检测，进一步减少写入开销。

```rust
for (mut health, decay) in query.contiguous_iter_mut().unwrap() {
    for (h, d) in health.bypass_change_detection().iter_mut().zip(decay) {
        h.0 *= d.0;
    }
}
```

**关键点**：
- `Mut<[T]>` 上的 `bypass_change_detection()` 返回 `&mut [T]`，彻底跳过 Change Detection
- 适用于高频更新、不需要被其他 System 观察变化的场景
- 性能提升显著（见下方基准数据）

### 2.4 性能数据

官方基准（10000 实体 `position += velocity`）：

| 方法 | 时间 | AVX2 |
|------|------|------|
| Normal iteration | 5.58 µs | 5.51 µs |
| Contiguous iteration | 4.88 µs | 1.87 µs |
| Contiguous + no change detection | 4.40 µs | 1.58 µs |

**解读**：

| 对比 | 提升幅度 | 说明 |
|------|----------|------|
| Normal → Contiguous | 12.5% (无AVX2) / 66% (AVX2) | 连续内存使 SIMD 向量化生效 |
| Contiguous → +bypass | 9.8% (无AVX2) / 15.5% (AVX2) | 消除 Change Detection 开销 |
| Normal → Contiguous+bypass | 21.1% (无AVX2) / 71.3% (AVX2) | 综合提升 |

**结论**：AVX2 开启时，Contiguous 迭代 + bypass_change_detection 可获得 **约 3.5 倍** 的性能提升。对于战棋项目中大量 Buff/DOT/Regen 的批量运算，收益极为可观。

---

## 3. 使用条件

`contiguous_iter` / `contiguous_iter_mut` 并非在所有 Query 上都可用，需要满足以下条件：

### 3.1 存储策略

所有查询的 Component 必须使用默认的 **"table" 存储策略**。

```rust
// ✅ 可用：默认 table 存储
#[derive(Component)]
struct Health(f32);

// ❌ 不可用：sparse-set 存储
#[derive(Component)]
#[component(storage = "SparseSet")]
struct RareComponent;
```

**原因**：`contiguous_iter` 依赖 Table 的连续内存布局。SparseSet 使用哈希表 + 稀疏数组，天然不连续。

### 3.2 查询过滤器限制

查询过滤器不能包含 `Changed<T>` 或 `Added<T>`：

```rust
// ✅ 可用：With/Without 是 Archetypal filter
Query<&Health, With<Alive>>

// ❌ 不可用：Changed/Added 需要逐实体检查
Query<&Health, Changed<Health>>
Query<&Health, Added<Health>>
```

**原因**：`Changed<T>` / `Added<T>` 需要检查每个实体的 tick 值，无法用连续切片表达。

**允许的过滤器**：`With<T>`、`Without<T>`、`Or<...>`（仅含 Archetypal filter）等。

### 3.3 unwrap 的安全性

条件是查询类型的**固定属性**（编译期确定），不是运行时状态。因此可以安全 `unwrap()`：

```rust
// 安全：Health 是 table 存储，没有 Changed/Added 过滤器
for healths in query.contiguous_iter().unwrap() {
    // ...
}
```

**唯一例外**：泛型代码中，Component 的存储策略可能在编译期未知，此时需要处理 `None` 情况：

```rust
// 泛型场景：T 的存储策略未知
fn generic_process<T: Component>(query: Query<&T>) {
    if let Some(slices) = query.contiguous_iter() {
        for slice in slices {
            // 连续路径
        }
    } else {
        // 回退到普通迭代
        for val in &query {
            // ...
        }
    }
}
```

---

## 4. 对 SRPG 项目的适用场景

### 4.1 Attribute 批量运算

生命回复、法力恢复等每帧执行的属性运算：

```rust
fn apply_regen(mut query: Query<(&mut Health, &RegenRate)>) {
    for (mut health, regen) in query.contiguous_iter_mut().unwrap() {
        for (h, r) in health.iter_mut().zip(regen) {
            h.0 = (h.0 + r.0).min(h.max);
        }
    }
}
```

**适用原因**：
- 回复逻辑是纯数值运算，无分支
- 每帧所有拥有 `Health + RegenRate` 的实体都需处理
- 不需要 `Changed<Health>` 过滤

### 4.2 Buff Tick 批量处理

Buff 计时器递减、DOT 伤害应用：

```rust
fn tick_buffs(mut query: Query<(&mut BuffTimer, &BuffDuration)>) {
    for (mut timer, duration) in query.contiguous_iter_mut().unwrap() {
        for (t, d) in timer.iter_mut().zip(duration) {
            t.0 -= d.0;
        }
    }
}
```

**适用原因**：
- Buff 数量可能很大（1000+ Buff/DOT/Regen 场景）
- 计时器递减是简单的减法运算
- 批量处理后，过期检测可单独用普通迭代处理

### 4.3 位置批量更新

移动系统中的位置更新：

```rust
fn apply_velocity(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in query.contiguous_iter_mut().unwrap() {
        for (p, v) in pos.iter_mut().zip(vel) {
            p.0 += v.0;
        }
    }
}
```

**适用原因**：
- `position += velocity` 是最经典的 SIMD 友好运算
- 官方基准就是用这个场景测试的
- 战棋中移动动画插值也适用

### 4.4 不适用场景

以下场景**不适合**使用 contiguous_iter：

| 场景 | 原因 | 替代方案 |
|------|------|----------|
| 需要 `Changed<T>` 过滤 | 不满足使用条件 | 普通 `iter()` |
| 包含 `SparseSet` 组件 | 不满足使用条件 | 普通 `iter()` |
| 每个实体有复杂分支逻辑 | 无法向量化 | 普通 `iter()` |
| 实体数量极少（< 100） | 收益可忽略 | 普通 `iter()` |
| 需要访问 Entity ID | 切片不包含 Entity | 普通 `iter()` |

---

## 5. Change Detection 成本显性化

### 5.1 官方信号

0.19 专门提到 `bypass_change_detection()`，说明官方开始承认 **Change Detection 不是免费午餐**。

过去 Bevy 社区有一种倾向：所有 `Mut<T>` 都带 Change Detection，开发者无需关心。但基准数据清楚表明：

- Change Detection 在 AVX2 场景下额外增加 **15.5%** 开销
- 对于高频批量运算，这个开销不可忽视

官方提供 `bypass_change_detection` 是一种"成本显性化"策略：让开发者明确知道什么时候在为 Change Detection 付费，什么时候可以跳过。

### 5.2 项目规范建议

根据数据特征，将组件分为三类：

| 数据类别 | 特征 | Change Detection 策略 | 示例 |
|----------|------|----------------------|------|
| **状态数据** | 低频变化，变化需被观察 | 允许 `Changed<T>` | `BattlePhase`, `TurnState`, `Faction` |
| **高频数据** | 每帧更新，变化不需观察 | 慎用 `Changed<T>` | `Health`, `Mana`, `Position` |
| **批量运算数据** | 批量处理，纯数值运算 | `contiguous_iter` + `bypass_change_detection` | `BuffTimer`, `RegenRate`, `Velocity` |

**具体建议**：

- **状态数据**：允许 `Changed<T>`（如 `BattlePhase` 变化触发 UI 更新）
- **高频数据**：慎用 `Changed<Attribute>` / `Changed<Health>` / `Changed<Position>`——它们每帧都在变，`Changed` 几乎总是返回 true，等于白白付出开销
- **批量运算**：优先 `contiguous_iter` + `bypass_change_detection`，如果需要通知其他 System，用 Event 代替 Change Detection

### 5.3 组件设计影响

Contiguous Query 的性能优势与组件粒度直接相关：

```rust
// ❌ 反模式：巨型组件
#[derive(Component)]
struct Character {
    health: f32,
    max_health: f32,
    mana: f32,
    max_mana: f32,
    level: u32,
    faction: Faction,
    position: Vec3,
    velocity: Vec3,
    // ... 50 个字段
}

// ✅ 正确：细粒度组件
#[derive(Component)]
struct Health { current: f32, max: f32 }

#[derive(Component)]
struct Mana { current: f32, max: f32 }

#[derive(Component)]
struct Level(u32);

#[derive(Component)]
struct Faction(FactionType);

#[derive(Component)]
struct Position(Vec3);

#[derive(Component)]
struct Velocity(Vec3);
```

**原因**：
- `Query<&Health>` 只需加载 Health 列的连续内存
- `Query<&Character>` 需要加载整个巨型组件，包含大量不需要的字段
- 细粒度组件让 `contiguous_iter` 的切片更紧凑，缓存利用率更高

**项目现状**：本项目的组件设计已遵循细粒度原则（Health/Mana/Level/Faction 独立），天然适合 Contiguous Query 优化。

---

## 6. 迁移策略

### 6.1 不要现在就用

**核心原则**：等性能分析发现热点后再使用，否则属于过早优化。

理由：
- 当前实体规模可能不足以体现性能差异
- 过早使用会增加代码复杂度
- `contiguous_iter` 的 API 与普通迭代器不同，需要额外的思维负担

### 6.2 提前准备

虽然不立即使用，但可以提前做好基础设施：

1. **组件设计遵循细粒度原则**（已在做）
   - 保持 Health/Mana/Level/Faction 等独立组件
   - 避免创建巨型"上帝组件"

2. **区分"状态数据"和"高频数据"**
   - 在组件文档中标注数据类别
   - 审查现有 `Changed<T>` 使用是否合理

3. **建立性能基准测试**
   - 使用 `criterion` 为关键 System 建立基准
   - 记录当前性能数据作为对比基线
   - 基准测试示例：

```rust
// 基准测试框架（未来建立）
#[cfg(test)]
mod benches {
    use criterion::*;

    fn bench_health_regen(c: &mut Criterion) {
        // 对比普通迭代 vs contiguous_iter 的性能
        // 在 100/1000/10000 实体规模下测试
    }
}
```

### 6.3 未来应用路线

当性能分析发现热点后，按以下优先级应用：

| 优先级 | 场景 | 预期收益 | 触发条件 |
|--------|------|----------|----------|
| P0 | AttributeSystem 批量运算 | 高 | 500+ 实体同时回复 |
| P0 | BuffTickSystem 批量处理 | 高 | 1000+ Buff/DOT/Regen |
| P1 | EffectTickSystem 批量处理 | 中 | 500+ 活跃 Effect |
| P1 | 位置批量更新 | 中 | 200+ 同时移动的实体 |
| P2 | 其他数值批量运算 | 低-中 | 按需 |

---

## 7. 批处理思维

官方开始鼓励"批处理思维"，这是 ECS 架构的核心理念在 API 层面的体现。

### 7.1 两种思维模式

| 模式 | 思考方式 | 适用场景 | API |
|------|----------|----------|-----|
| **单实体逻辑** | "对这个实体做什么" | 条件分支、状态机、AI 决策 | `for entity in &query` |
| **批量数据逻辑** | "对这批数据做什么" | 数值运算、物理模拟、属性更新 | `for slice in query.contiguous_iter()` |

### 7.2 识别批量数据逻辑

判断标准：**如果去掉实体概念，纯看数据运算，逻辑是否仍然成立？**

```rust
// 单实体逻辑：需要知道"这是谁"
fn process_ai(query: Query<(Entity, &Position, &Team), With<Alive>>) {
    for (entity, pos, team) in &query {
        // 每个实体的决策不同，依赖上下文
        let target = find_nearest_enemy(entity, pos, team);
        // ...
    }
}

// 批量数据逻辑：不需要知道"这是谁"
fn apply_regen(query: Query<(&mut Health, &RegenRate)>) {
    // Health += Regen 本质是数组运算，不是实体运算
    for (mut health, regen) in query.contiguous_iter_mut().unwrap() {
        for (h, r) in health.iter_mut().zip(regen) {
            h.0 = (h.0 + r.0).min(h.max);
        }
    }
}
```

### 7.3 未来写 System 时的思考框架

1. 这个 System 的核心逻辑是什么？
2. 它是"对每个实体做不同的事"还是"对所有实体做相同的事"？
3. 如果是后者，考虑使用 `contiguous_iter`
4. 是否需要 Change Detection？如果不需要，加上 `bypass_change_detection`

---

## 8. 注意事项

### 8.1 不要过早优化

`contiguous_iter` 是性能优化工具，不是默认选择。在实体数量较少时，普通迭代器的性能完全够用。**只在性能分析确认热点后使用**。

### 8.2 unwrap 是安全的

对于非泛型代码，`contiguous_iter().unwrap()` 是安全的——条件在编译期已确定。但如果写泛型代码，需要处理 `None` 情况并提供回退路径。

### 8.3 bypass_change_detection 的副作用

使用 `bypass_change_detection` 后，该次写入不会被 Change Detection 记录。如果其他 System 依赖 `Changed<T>` 来响应变化，会导致逻辑错误。确保你理解跳过变更检测的后果。

### 8.4 Changed/Added 过滤器不兼容

`Changed<T>` / `Added<T>` 不能与 `contiguous_iter` 一起使用。如果需要过滤变更实体，只能使用普通迭代器。考虑用 Event 代替 Change Detection 来通知变更。

### 8.5 未来展望

Bevy 可能基于 Contiguous Query 做更多优化：
- **只读 Schedule 并行编码**：编译器可以推断哪些 System 只读访问哪些 Component，实现更细粒度的并行调度
- **自动向量化提示**：编译器可能自动识别可向量化的 System 并应用 `contiguous_iter`
- **更丰富的批量操作 API**：如 `query.par_contiguous_iter()` 并行批量处理

### 8.6 与现有架构的兼容性

本项目的 ECS 架构与 Contiguous Query 完全兼容：
- 细粒度组件设计 ✓
- Table 默认存储策略 ✓
- Effect/Modifier 管线不依赖 Change Detection ✓
- 批量运算场景清晰可识别 ✓

唯一需要注意的是：**不要为了使用 `contiguous_iter` 而绕过 Effect/Modifier 管线直接修改战斗数值**。批量优化应作用于管线内部的数值运算，而非跳过管线。
