# Post-Migration：迁移完成后的架构展望

> **目标**：迁移完成后，项目已全面采用 Bevy 0.19 ECS 模型。本文档定义完成状态、后续维护规则、以及未来 0.20 的接入准备。
> **前置条件**：Phase A + B + C 全部完成并验证通过

---

## 1. 迁移完成标志

当以下状态全部满足时，Bevy 0.18.1 → 0.19 迁移正式完成：

### 编译与测试

- [x] `cargo check` 零错误
- [x] `cargo clippy -- -D warnings` 零警告
- [x] `cargo nextest run` 全部通过
- [x] `cargo build --features dev` 正常
- [x] `cargo build --release` 正常

### 代码模式

- [x] 零 `EventReader<T>` / `EventWriter<T>` 残留（全部替换为 `On<T>` Observer + `trigger()`）
- [x] 零 `timer.tick()` / `just_finished()` 残留（全部替换为 `Delayed<T>` / `FreDelayed<T>`）
- [x] 零 `#[derive(Bundle)]` 残留（全部替换为 `bsn!()` 场景或 `spawn_*()` 工厂函数）
- [x] 零 `Res<BattleState>` / `Res<TurnState>` 等核心 Resource 注入（全部替换为 `Single<&T>`）
- [x] 零 `Input<T>` 残留（全部替换为 `ButtonInput<T>`）
- [x] 零 `font_size: f32` 残留（全部替换为 `TextFont { font_size: FontSize::Px(..) }`）
- [x] 4 种核心关系使用 `Relationship<T>`
- [x] 全部组件/事件/资源类型包含 `#[derive(Reflect)]`
- [x] User Settings 三组定义 + `init_settings()` 完成
- [x] DiagnosticsOverlay 在 dev 模式下可用

### 文档

- [x] `README.md` 引擎版本已更新
- [x] `ai-constitution-complete.md` 已更新（0.19 规则）
- [x] `01-architecture/README.md` 通信机制已更新
- [x] `.trae/rules/ECS规则.md` 新增 Delayed Commands / Observer / BSN 规则
- [x] 新增 Bevy 0.19 迁移 ADR

---

## 2. 新架构规则（宪法级）

迁移完成后，以下规则加入 `ai-constitution-complete.md`，永久生效：

### 规则 1：Observer 是默认跨域通信机制

```
Observer 是跨 Feature 通信的首选机制。Event 类型仍然使用 #[derive(Event)] 定义，
但发送/接收必须使用 trigger(T) + On<T> Observer 模式，而非 EventWriter/EventReader。

适用范围：
- 跨 Feature 事件 → Observer（如 TurnEnded → Quest 检查）
- 同 Feature 内逻辑 → 直接 System 调用（不绕过 Observer）

禁止：
- 新的 EventWriter<X> + EventReader<X> 代码模式
- 用 Observer 模拟函数调用（A Observer B → B Observer A 循环）
```

### 规则 2：Delayed Commands 替代 Timer

```
所有"一次性延迟效果"必须使用 Delayed<T> 或 FreDelayed<T>。
Timer 仅用于：
- 需要暂停/恢复的长周期效果（如可驱散 Buff）
- 需要手动 Advance 的帧动画序列

禁止：
- 新的 Timer 轮询 System
- 单纯"等 X 秒后执行 Y"用 Timer 实现
```

### 规则 3：BSN 用于实体结构

```
BSN 负责描述"实体长什么样"（组件组合），System 负责"实体做什么"。
BSN 使用范围：
- UI 层（src/app/scenes/）默认使用 BSN
- 核心玩法层实体生成使用 spawn_*() 工厂函数（内部可自由选择 BSN 或传统 spawn）
- 新增 Feature 的预制体使用 BSN 定义

禁止：
- BSN 描述业务逻辑
- BSN 引用 System/Observer
- 核心玩法层（src/core/）直接 import bsn! 宏
```

### 规则 4：Resource 默认使用 Singleton Entity

```
新增全局状态时，优先使用 Singleton Entity Component 而非 Resource：
1. 定义 Marker 组件（BattleRoot / TimeRoot / ...）
2. 将状态作为 Component 放在 Marker Entity 上
3. 使用 Single<T> 或 Query<&T, With<Marker>> 注入

保留 Res<T> / ResMut<T> 仅用于：
- Bevy 内置 Resource（Time<Real>, Assets<T> 等）
- 第三方库期望的 Resource 类型
- 零业务语义的技术配置
```

### 规则 5：Relationship 用于实体关系

```
Entity 间的关系必须使用 Relationship<T>，而非裸 Entity 字段。
当需要表达"X 属于 Y"、"X 由 Y 创建"、"X 的目标是 Y"时：

1. 定义 #[derive(Relationship)] struct XOf(Entity)
2. 在源 Entity 上添加 Relationship<XOf>
3. 使用 query.get::<Relationship<XOf>>() 查询

例外：
- 临时引用（如"当前选中单位"）
- 值语义的关系（如"队伍 ID"）
```

---

## 3. 后续演进路线图

### 短期（迁移后 1–2 个月）

- 修复迁移中遗留的边缘 case bug
- 补全测试覆盖率（新 Observer/Delayed 路径）
- 性能调优（实测 `contiguous_iter()` 收益）
- 开发者体验优化（DiagnosticsOverlay 集成更多自定义诊断）

### 中期（2–6 个月）

- 继续观察 Bevy 0.20 发布计划
- BSN API 跟踪：如果 0.20 中 BSN 有破坏性变更，迁移工厂函数内部实现
- Relationship API 跟踪：如果 0.20 中 Relationship 有增强（级联删除、多对多），评估升级
- 考虑用 `SceneComponent` 重构编辑器数据格式（如果 0.20 稳定）

### 长期（6 个月+）

- 评估 Bevy Editor 集成
- 评估 Feathers UI 作为长期 UI 框架
- 评估 `bevy_remote` 运行时调试能力

---

## 4. 常见问题

### Q：为什么 EventReader 模式被全部替换？

EventReader/EventWriter 在 Bevy 0.19 中没有被废弃，但 Observer + trigger() 模式提供了三个关键优势：

1. **声明式注册**：Observer 自动在 `app.observe()` 时注册，不需要在 Schedule 中手动添加 System
2. **Entity 级作用域**：Observer 可以绑定到特定 Entity，实现"谁关心谁监听"
3. **RunConditions**：Observer 支持 `run_if()` 条件守卫，EventReader 需要手动在 System 入口判断

### Q：为什么 Bundles 被全部替换？

Bundle 在 0.19 中没有废弃，但 BSN 和工厂函数提供了更好的灵活性：

1. **BSN 声明式**：`bsn! { Name, Health, Faction }` 比 `struct XBundle { name: Name, ... }` 更接近数据定义
2. **工厂函数隔离**：`spawn_hero()` 可以在不改变调用方的情况下修改内部实现
3. **编辑器兼容**：BSN 场景可以直接在未来的 Bevy Editor 中编辑

### Q：Delayed Commands 不可取消怎么办？

通过 `FreDelayed<T>` 包装层解决。包装层存储 Delayed 的命令 ID，支持：

- `remove::<FreDelayed<T>>()` → 取消延迟命令
- `FreDelayed::pause()` / `FreDelayed::resume()` → 暂停/恢复
- `.remaining()` → 查询剩余时间

### Q：迁移后编译时间会变长吗？

BSN 和 Observer 模式不会显著增加编译时间。Delayed Commands 是内置功能，零开销。Relationship 可能带来少量额外泛型编译，但影响可以忽略。

---

> **维护者**: @architect
> **关联文档**: `docs/00-governance/ai-constitution-complete.md`, `docs/01-architecture/README.md`, `.trae/rules/ECS规则.md`
> **最后更新**: 2026-06-19 | **版本**: v2.0 (激进版)
