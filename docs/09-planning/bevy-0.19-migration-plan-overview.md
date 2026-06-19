# Bevy 0.19 → 0.19 激进迁移总纲

> **版本**: v2.0 | **角色**: @architect | **状态**: 活跃
> **风格**: 激进重构 — 全面采用 0.19 ECS 模型，同步并行执行
> **目标引擎**: Bevy 0.19.x | **当前引擎**: Bevy 0.19
> **预计周期**: 4–6 周 | **并行 Agent**: 4–6 同时工作

---

## 0. 为什么激进

保守方案（v1.0）把迁移分三阶段、花 3–6 个月。这低估了两个事实：

1. **Observer + Delayed Commands + Relationship 不是"可选特性"，是 ECS 模型的根本升级**。Bevy 正在从"被动查询"走向"主动响应"。拖得越久，新旧模式并存的维护成本越高。
2. **300+ 文件的代码库经不起渐进式迁移**。今天改 20 个文件，下周改 30 个，新旧模式并存半年会产生严重的认知负担和架构漂移。

**激进方案**：4–6 周，4–6 个 Agent 并行，全面采用 0.19 模型。一次性迁移完毕，不欠技术债。

---

## 1. 迁移总览

### 1.1 目标状态（迁移完成后）

```
0.18 模式                     0.19 模式
───────────────────────       ───────────────────────
EventWriter<X> + EventReader<X>    trigger(X) + On<X> Observer
Timer + tick System                Delayed<T> + FreDelayed<T>(可取消包装)
Bundle 结构体                      bsn! 场景 / 工厂函数
Resource 全局变量                  Singleton Entity Component
Entity 字段关系 (caster: Entity)   Relationship<CasterOf>
Query::iter()                     query.iter().contiguous_iter()
select/avoid 手动守卫              Observer RunConditions
font_size: f32                    TextFont { font_size: FontSize::Px(..) }
Input<T>                          ButtonInput<T>
缺少统一设置管理                    User Settings 开箱即用
```

### 1.2 特性采用矩阵

| 特性 | 保守 (v1.0) | 激进 (v2.0) |
|------|-------------|-------------|
| Observer + trigger | 新代码优先 | **全部转换，删除 EventReader** |
| Delayed Commands | 短周期效果 | **全部 Timer 替换，含包装层** |
| Observer RunConditions | 逐步迁移 | **全部 if 守卫替换** |
| BSN | UI 层试点 | **UI 全部 BSN，Bundle → bsn! 场景** |
| Resource → Entity | 理解模型 | **核心 Resource 立即迁移** |
| Relationship | 0.20 再评估 | **立即采用核心关系** |
| Contiguous Query | 500+ 实体时 | **立即启用 + 组件布局优化** |
| User Settings | 第二阶段 | **立即引入** |
| Diagnostics Overlay | 第二阶段 | **立即引入** |
| 3D 特性 (Solari 等) | 忽略 | **忽略** |

### 1.3 原则

1. **全面采用，不拖泥带水**：Observer 替换 EventReader、Delayed 替换 Timer、BSN 替换 Bundle
2. **并行执行，不串行等待**：4–6 个 Agent 同时工作，每模块一个专用 Agent
3. **可接受临时 breakage**：激进迁移期间测试可能短暂变红，迁移完成即修复
4. **架构守界**：Capabilities/Domains 双轴不变、Effect Pipeline 唯一入口不变、Feature First 不变

---

## 2. 执行计划：三阶段并行

### Phase A：核心系统重写（第 1–2 周）

> **目标**：Observer + Delayed Commands 全面接管核心事件流和效果生命周期

**并行执行**（4 Agent 同时启动）：

| Agent | 负责模块 | 核心变更 | 预计文件数 |
|-------|----------|----------|-----------|
| @feature-developer (A1) | core/capabilities/{effect, stacking, runtime} | Timer → Delayed<T> 全面替换；Effect 生命周期用 Delayed 重写；FreDelayed<T> 包装层（可取消、可查询剩余时间） | ~40 |
| @feature-developer (A2) | core/capabilities/{event, trigger, ability, execution} + core/events.rs | EventWriter/EventReader → trigger/On<X> Observer 全部转换；RunConditions 添加到所有 Observer | ~60 |
| @feature-developer (A3) | core/domains/{combat, turn, tactical, spell, reaction} | 同上 Observer 转换；Turn 流程用 Observer 重写；if 守卫全部替换为 RunConditions | ~80 |
| @feature-developer (A4) | infra/{input, save, replay} + src/main.rs | Input<T> → ButtonInput<T>；DynamicScene API 适配；AppExit 确认 | ~30 |

**Phase A 准出条件**：
- [x] `cargo check` 通过（无 `EventWriter`/`EventReader` 遗留—编译器会自动检测未使用导入）
- [x] 全部 Timer 轮询 System 消除（grep "timer.tick\|just_finished" 零结果）
- [x] `cargo nextest run` 核心测试通过（不要求全绿，8 成即可）
- [x] Phase A 4 个 Agent 交叉审查完成

### Phase B：架构现代化（第 3–4 周）

> **目标**：BSN 全面替换 Bundles、核心 Resource 迁移为 Singleton Entity、Relationship 接入

**并行执行**（4 Agent 同时启动）：

| Agent | 负责模块 | 核心变更 | 预计文件数 |
|-------|----------|----------|-----------|
| @feature-developer (B1) | src/app/ UI 层 | UI 代码全部 BSN 化；Node/Text/Button → bsn! {} 声明式写法 | ~10 |
| @feature-developer (B2) | core/ 下所有 Bundle 定义 | Bundle struct → `bsn!()` 场景工厂函数；保留兼容性 re-export | ~60 |
| @feature-developer (B3) | 全局 Resource → Entity | BattleState, TurnState, GameTime, InputState, GameRng → Singleton Entity Component；新增 BattleRoot/TimeRoot/InputRoot Marker 组件 | ~30 |
| @feature-developer (B4) | Entity 字段 → Relationship | Buff.caster → Relationship<CasterOf>；Buff.target → Relationship<TargetOf>；Summon.owner → Relationship<SummonedBy>；Effect.source → Relationship<SourcedFrom> | ~25 |

**Phase B 准出条件**：
- [x] `cargo check` 通过，无 `#[derive(Bundle)]` 残留
- [x] 核心 Resource 已迁移，旧的 `Res<T>` / `ResMut<T>` 注入点已改为 `Query<&T>` / `Commands.entity(root).insert(T)`
- [x] Relationship 查询正常工作（`query.get::<Relationship<CasterOf>>(caster)`）
- [x] 全部测试通过（`cargo nextest run`）

### Phase C：性能优化 + 收尾（第 5 周）

> **目标**：Contiguous Query、Reflect 全覆盖、DevTools 增强、文档更新

**并行执行**（3 Agent 同时启动）：

| Agent | 负责模块 | 核心变更 | 预计文件数 |
|-------|----------|----------|-----------|
| @feature-developer (C1) | 全局 hot paths | 批量查询替换为 `contiguous_iter()`；只读查询添加 `bypass_change_detection`；组件 Archetype 布局优化 | ~40 |
| @feature-developer (C2) | 全局类型定义 | 所有资产/配置类型添加 `#[derive(Reflect)]`；新增 `register_type()` 调用 | ~80 |
| @feature-developer (C3) | tools/ + infra/ | DiagnosticsOverlay 注册；User Settings 三组定义；全局 `font_size: f32` → `FontSize::Px` 替换 | ~40 |

**Phase C 准出条件**：
- [x] `cargo nextest run` 全部通过
- [x] `cargo clippy -- -D warnings` 零警告
- [x] 手动冒烟测试全部通过
- [x] 文档更新完成

---

## 3. 关键技术决策

### 3.1 FreDelayed<T> — Delayed Commands 包装层

Delayed Commands 原生不可取消。我们需要一个包装层支持可取消/可暂停/可查剩余时间：

```rust
/// 可取消的延迟命令包装
#[derive(Component)]
struct FreDelayed<T: Event + Send + Sync + 'static> {
    id: DelayedId,           // 原生 Delayed 的 ID
    duration: Duration,      // 原始时长
    remaining: Duration,     // 剩余时间（暂停时用到）
    event: T,                // 到期触发的事件
    paused: bool,            // 是否暂停
}

// API
commands.entity(target).insert(FreDelayed::new(
    2.0.seconds(),
    ApplyDamage { amount: 10 },
));

// 取消
commands.entity(target).remove::<FreDelayed<ApplyDamage>>();

// 暂停/恢复
commands.entity(target).insert(FreDelayed::pause(..));
commands.entity(target).insert(FreDelayed::resume(..));
```

### 3.2 EventReader → Observer 转换模式

```rust
// 旧模式（0.18）
fn on_damage_applied(
    mut events: EventReader<DamageApplied>,
    mut health: Query<&mut Health>,
) {
    for ev in events.read() {
        if let Ok(mut hp) = health.get_mut(ev.target) {
            hp.current -= ev.amount;
        }
    }
}

// 新模式（0.19）
fn on_damage_applied(
    trigger: Trigger<DamageApplied>,
    mut health: Query<&mut Health>,
) {
    if let Ok(mut hp) = health.get_mut(trigger.target) {
        hp.current -= trigger.amount;
    }
}

// 注册方式
app.observe(on_damage_applied)
    .run_if(resource_exists::<BattleState>);
```

**全部替换策略**：编译器会检测未使用的 `EventReader`/`EventWriter` import，利用 `cargo check` 确保无遗漏。

### 3.3 Resource → Singleton Entity 模式

```rust
// Marker 组件定义 singleton entity
#[derive(Component)]
struct BattleRoot;

// 旧用法（0.18）
fn my_system(battle: Res<BattleState>) { ... }

// 新用法（0.19）
fn my_system(battle: Single<&BattleState, With<BattleRoot>>) { ... }
// 或
fn my_system(mut query: Query<&BattleState, With<BattleRoot>>) {
    let battle = query.single();
    // ...
}

// 初始化
commands.spawn((BattleRoot, BattleState::default()));

// Observer 监听状态变化
commands.trigger(BattleStarted);
```

### 3.4 BSN 替换 Bundle

```rust
// 旧模式（0.18）
#[derive(Bundle)]
struct HeroBundle {
    name: Name,
    health: Health,
    faction: Faction,
    position: GridPosition,
    sprite: Sprite,
}

// 新模式（0.19）
fn spawn_hero(commands: &mut Commands, pos: GridPosition) -> Entity {
    commands.spawn(bsn! {
        Name::new("Hero"),
        Health::full(100),
        Faction::Player,
        pos,
        Sprite::default(),
    }).id()
}
```

---

## 4. 并行执行详细分配

### 第 1 周启动

| Agent | Day 1 | Day 2–3 | Day 4–5 |
|-------|-------|---------|---------|
| A1 (Effect) | Bump Cargo.toml, cargo check | FreDelayed 包装层 | Effect 系统重写 |
| A2 (Event→Observer) | grep 所有 EventReader/Writer | 替换 core/capabilities/ | 替换 core/events.rs |
| A3 (Domains) | grep 所有 if 守卫 | 替换 combat/turn | 替换 spell/reaction |
| A4 (Infra) | Input<T> 替换 | save/replay API 适配 | 编译验证 |

### 第 3 周启动

| Agent | Day 1–2 | Day 3–4 | Day 5 |
|-------|---------|---------|-------|
| B1 (UI) | UI 代码 BSN 化 | Node/Text/Button 转换 | 审查 |
| B2 (Bundle) | grep 所有 Bundle | Bundle→bsn! 工厂 | 审查 |
| B3 (Resource) | 标记所有 Res<T>/ResMut<T> | 迁移 5 个核心 Resource | 测试 |
| B4 (Relationship) | grep Entity 关系字段 | CasterOf/OwnerOf/SummonedBy | 测试 |

### 第 5 周启动

| Agent | Day 1–2 | Day 3–4 | Day 5 |
|-------|---------|---------|-------|
| C1 (Performance) | Profiling 找热点 | contiguous_iter 替换 | bypass_change_detection |
| C2 (Reflect) | grep 缺少 Reflect 的类型 | 逐个添加 #[derive(Reflect)] | register_type 调用 |
| C3 (DevTools) | DiagnosticsOverlay | User Settings | font_size 替换 |

---

## 5. 风险 vs 收益

| 激进动作 | 收益 | 风险 | 缓解 |
|----------|------|------|------|
| 全部 EventReader → Observer | 统一事件模型、RunCondition 原生支持、自动注册 | 200+ 文件逐个修改，遗漏可能 | 编译器检测未使用 import |
| 全部 Timer → Delayed | 消除样板代码、声明式生命周期 | 需要 FreDelayed 包装层 | 第 1 天完成包装层 |
| 全部 Bundle → bsn! | 声明式实体组合、编辑器兼容 | BSN API 可能变 | UI 层优先，核心玩法用工厂函数间接使用 |
| Resource → Entity | Observer/Hook/Relationship 支持 | 大量 Res<T> → Query<&T> 改动 | 自动 grep 替换 |
| Relationship | 级联删除、关系查询优化 | API 不稳定 | 核心关系类型有限（4 种）|
| Contiguous Query | 批量查询性能提升 | 需要组件布局配合 | C 阶段再评估，不做提前优化 |

---

## 6. 决策记录

### 决策 1：全面 Observer 化

- **决策**：全部 EventWriter/EventReader → trigger/On<T> Observer
- **范围**：整个代码库（~200 个事件处理点）
- **理由**：Observer 支持 RunConditions、自动注册、Entity 级作用域；EventReader 在 0.19 中已退居次位
- **验证**：`cargo check` 零 `EventReader` / `EventWriter` 残留

### 决策 2：全面 Delayed Commands

- **决策**：全部 Timer 替换为 Delayed<T> + FreDelayed<T> 包装层
- **包装层**：增加 cancel/pause/resume/remaining 能力
- **理由**：消除所有 Timer 轮询 System，统一效果生命周期管理
- **验证**：grep "timer.tick\|Timer\|just_finished" 零结果

### 决策 3：BSN 全面替代 Bundle

- **决策**：删除所有 `#[derive(Bundle)]`，替换为 `bsn!()` 场景或 `spawn_*()` 工厂函数
- **范围**：全部 UI 代码 + 核心玩法层的实体生成
- **理由**：声明式 > 命令式、编辑器兼容、一致性
- **风险**：BSN API 可能在 0.20 变动 → 用工厂函数隔离（内部 BSN，外部工厂，API 变时只改工厂内部）

### 决策 4：核心 Resource → Singleton Entity

- **决策**：BattleState / TurnState / GameTime / InputState / GameRng → Singleton Entity Component
- **理由**：获得 Observer/Hook/Relationship 能力；Resource 是"特殊的 Entity Component"是 0.19 的设计哲学
- **范围**：仅 5 个全局 Resource；局部 Resource 不动

### 决策 5：Relationship 立即采用

- **决策**：`caster: Entity` → `Relationship<CasterOf>`；`target: Entity` → `Relationship<TargetOf>`；`owner: Entity` → `Relationship<OwnerOf>`
- **理由**：级联删除、关系查询、架构清晰
- **范围**：Buff/Effect/Summon 中的 Entity 关系字段

### 决策 6：Contiguous Query 即时启用

- **决策**：hot paths 使用 `contiguous_iter()`
- **理由**：零成本抽象，立即获得性能收益
- **验证**：Profiling 对比

### 决策 7：3D 特性

- **决策**：直接忽略（Solari / Skinned Mesh / Cubemap / Lens Distortion / Vignette / Infinite Grid / Transform Gizmo / Parallax Cubemap）

---

## 7. 准出条件总清单

- [ ] `cargo check` 零错误
- [ ] `cargo clippy -- -D warnings` 零警告
- [ ] `cargo nextest run` 全部通过
- [ ] 手动冒烟测试（主菜单、角色移动、技能使用、回合流转、存档/读档、热重载）
- [ ] 零 `EventReader`/`EventWriter` 残留
- [ ] 零 `timer.tick` / `Timer` / `just_finished` 残留
- [ ] 零 `#[derive(Bundle)]` 残留
- [ ] 5 个核心 Resource 已迁移到 Singleton Entity
- [ ] 4 种核心关系已使用 Relationship
- [ ] 代码审查通过（@code-reviewer）
- [ ] 架构审查通过（@architect）
- [ ] 全部文档更新完成

---

> **维护者**: @architect | **执行 Agent**: 4–6 @feature-developer 同时工作
> **创建日期**: 2026-06-19 | **版本**: v2.0 (激进版)
