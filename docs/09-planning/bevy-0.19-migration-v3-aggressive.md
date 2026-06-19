---
id: 09-planning.bevy-0-19-migration-v3
title: Bevy 0.19 → 0.19 激进迁移总纲 v3.0
status: active
owner: architect
created: 2026-06-19
updated: 2026-06-19
tags:
  - migration
  - planning
  - bevy-0.19
---

# Bevy 0.19 → 0.19 激进迁移总纲 v3.0

> **版本**: v3.0 | **角色**: @architect | **当前引擎**: `bevy = "0.19.0-rc.3"` | **目标**: 全面生产就绪 + 文档对齐
> **风格**: 激进重构 — 全面采用 0.19 ECS 模型，不留技术债
> **预计周期**: 2–3 周 | **并行 Agent**: 2–3 同时工作

---

## 0. 现状快照（截至 2026-06-19）

**好消息：大部分迁移工作已经完成。** 代码库的实际状态远好于预期：

### ✅ 已完成（无需处理）

| 检查项 | 状态 | 说明 |
|--------|------|------|
| 主依赖升级 | ✅ | `bevy = "0.19.0-rc.3"` |
| EventReader/EventWriter | ✅ | 零残留，全面使用 Observer + trigger 模式 |
| `#[derive(Bundle)]` | ✅ | 零残留，全部替换 |
| `Input<T>` → `ButtonInput<T>` | ✅ | 已全面使用 |
| `AppExit` 返回类型 | ✅ | `fn main() -> AppExit` |
| `Res<T>`/`ResMut<T>` → `Single<T>` | ✅ | 全局 Resource 注入已迁移 |
| Observer 注册 | ✅ | 大量 `add_observer()` 使用中 |

### ❌ 剩余工作

| 检查项 | 严重度 | 位置 |
|--------|--------|------|
| dev-dependencies 版本不一致 | 🔴 阻塞 | `Cargo.toml` dev-deps 仍为 `bevy = "0.18.1"` |
| Timer → Delayed Commands | 🟡 中 | `localization/audit.rs`, `narrative/cutscene_system.rs`, `content/hot_reload.rs` |
| `font_size: u8` → `FontSize` | 🟡 中 | `cue/foundation/types.rs` |
| BSN 场景系统 | 🟡 中 | `app/scenes/plugin.rs` 使用 `commands.spawn(SceneRoot)` |
| Reflect 全覆盖 | 🟡 中 | 分散在所有 Component/Event/Resource 类型 |
| TextFont/FontSize/FontSource | 🟢 低 | 尚未使用，涉及所有 Text/UI 代码 |
| DiagnosticsOverlay | 🟢 低 | 未注册 |
| User Settings | 🟢 低 | 未引入 |
| Relationship | 🟢 低 | 未使用 |
| Contiguous Query | 🟢 低 | 未使用 |
| constitution 引擎版本声明 | 🟡 中 | 仍写 "Bevy 0.19+" |
| 架构文档通信机制 | 🟡 中 | 需移除 EventReader 描述，提升 Observer 地位 |
| .trae/rules ECS 规则 | 🟡 中 | 需新增 Delayed/BSN/Relationship 规则 |

---

## 1. 迁移策略：两阶段 + 文档同步

### Phase 1：代码补齐（第 1 周）

修复所有编译阻断和未完成的迁移项，确保 `cargo check` + `cargo nextest run` 全绿。

| Batch | 任务 | Agent | 文件数 |
|-------|------|-------|--------|
| 1.1 | 修复 dev-dependencies 版本 + cargo check | 手动 | 1 |
| 1.2 | Timer → Delayed Commands（3 处） | @feature-developer | 3 |
| 1.3 | font_size: u8 → FontSize 枚举 + TextFont 迁移 | @feature-developer | 3–5 |
| 1.4 | `commands.spawn(SceneRoot)` → BSN 或 spawn_scene | @feature-developer | 1 |
| 1.5 | Reflect 全量补齐 | @feature-developer | 50–80 |
| 1.6 | DiagnosticsOverlay + User Settings 引入 | @feature-developer | 2–5 |
| 1.7 | 批量测试修复 + nextest 全绿 | @test-guardian | ~30 |

### Phase 2：架构现代化（第 2 周）

采用 0.19 新特性优化架构，更新所有治理文档。

| Batch | 任务 | Agent | 文件数 |
|-------|------|-------|--------|
| 2.1 | BSN UI 层试点（app/scenes） | @feature-developer | 3–5 |
| 2.2 | Relationship 核心关系（CasterOf/TargetOf） | @feature-developer | 5–8 |
| 2.3 | Resource → Singleton Entity（BattleState/TurnState） | @feature-developer | 10–15 |
| 2.4 | Contiguous Query 热点替换 | @feature-developer | 5–8 |
| 2.5 | Vignette/Lens Distortion 战斗特效 | @feature-developer | 2–3 |

### Phase 3：文档全面对齐（与 Phase 1&2 并行）

同步更新所有治理文档，确保与 0.19 代码状态一致。

| 文档 | 变更内容 | 优先级 |
|------|---------|--------|
| `Cargo.toml` | dev-dependencies `0.18.1` → `0.19` | 🔴 P0 |
| `docs/00-governance/ai-constitution-complete.md` | §1.1 引擎版本 "0.19+" → "0.19"；新增 Observer/Delayed/Relationship/BSN 规则 | 🔴 P0 |
| `docs/01-architecture/README.md` | §4.2 通信机制：移除 EventReader，提升 Observer 地位；新增 §11 0.19 迁移 ADR | 🔴 P0 |
| `docs/01-architecture/ADR-002-ecs-communication.md` | 更新通信机制优先级 | 🟡 P1 |
| `docs/02-domain/` | 检查所有领域规则，确保无 0.18 模式引用 | 🟡 P1 |
| `docs/04-data/README.md` | 检查数据层映射是否与 0.19 一致 | 🟡 P1 |
| `docs/05-testing/test-spec.md` | 新增 Observer/Delayed/Relationship 测试模式 §5 | 🟡 P1 |
| `docs/03-technical/bevy-0.19-migration/*.md` | 归档至 `done/`（迁移完成后） | 🟢 P2 |
| `.trae/rules/ECS规则.md` | 新增 Delayed Commands / BSN / Relationship 规则 | 🟡 P1 |
| `.trae/rules/编码规则.md` | 更新 EventReader → Observer 规则 | 🟡 P1 |
| `.trae/rules/架构规则.md` | 更新通信机制优先级 | 🟡 P1 |

---

## 2. 详细执行计划

### Phase 1.1：修复 dev-dependencies（30 分钟）

```bash
# Cargo.toml [dev-dependencies]
# bevy = { version = "0.18.1", features = ["2d"] }
# → bevy = { version = "0.19.0-rc.3", features = ["2d"] }
```

完成后立即 `cargo check` 验证。

### Phase 1.2：Timer → Delayed Commands（3 处）

#### 2a. `infra/localization/audit.rs`

```rust
// 当前：Timer 轮询
audit_timer.timer.tick(time.delta());
if !audit_timer.timer.just_finished() { return; }

// 目标：Delayed Commands + Observer
commands.delayed().secs(AUDIT_INTERVAL).trigger(LocalizationAuditTick);
fn on_audit_tick(trigger: Trigger<LocalizationAuditTick>, ...) { ... }
```

#### 2b. `core/domains/narrative/systems/cutscene_system.rs`

```rust
// 当前：Timer tick
state.tick(time.delta().as_secs_f32());

// 目标：Delayed Commands + Observer
commands.delayed().secs(cutscene_duration).trigger(CutsceneFinished);
```

#### 2c. `content/hot_reload.rs`

```rust
// 当前：Timer 轮询
hr_state.timer.tick(time.delta());
if !hr_state.timer.just_finished() { return; }

// 目标：Delayed Commands
commands.delayed().secs(HOT_RELOAD_INTERVAL).trigger(HotReloadCheck);
```

### Phase 1.3：font_size + TextFont 迁移

#### 3a. `cue/foundation/types.rs`

```rust
// 当前
pub font_size: u8,       // 16

// 目标
pub font_size: FontSize,  // FontSize::Px(16.0)
```

导入路径：`bevy::text::FontSize`

#### 3b. 所有 Text 相关代码

```rust
// 当前：旧模式
Text::new("...")

// 目标：新模式
Text::new("..."),
TextFont { font_size: FontSize::Px(16.0), ..default() }
```

### Phase 1.4：commands.spawn → spawn_scene + BSN

`app/scenes/plugin.rs` 中 `commands.spawn(SceneRoot)` 迁移为：

```rust
// 方案 A：直接 spawn
commands.spawn(SceneRoot);

// 方案 B：BSN 场景（如需要声明子结构）
commands.spawn_scene(bsn! { SceneRoot });
```

优先方案 A（简单场景），BSN 仅在需要声明式子实体结构时使用。

### Phase 1.5：Reflect 全覆盖

搜索策略：

```bash
# 找出所有缺少 Reflect 的 Component/Event/Resource
grep -rn "#\[derive(Component)\]" src/ | grep -v "Reflect"
grep -rn "#\[derive(Event)\]" src/ | grep -v "Reflect"
grep -rn "#\[derive(Resource)\]" src/ | grep -v "Reflect"
```

对每个匹配：
1. `#[derive(Component, Reflect)]`
2. `#[reflect(Component)]`
3. `app.register_type::<T>()`（在对应 Plugin 中）

### Phase 1.6：DevTools + Settings

```rust
// tools/dev_tools_plugin.rs
#[cfg(feature = "dev")]
fn setup_dev_tools(app: &mut App) {
    app.add_plugins(DiagnosticsOverlayPlugin);
}

// infra/settings/ 新建模块
mod settings {
    #[derive(Resource, SettingsGroup, Reflect)]
    struct AudioSettings { ... }
    #[derive(Resource, SettingsGroup, Reflect)]
    struct VideoSettings { ... }
    #[derive(Resource, SettingsGroup, Reflect)]
    struct GameplaySettings { ... }
}
```

---

## 3. 文档更新详细方案

### 3.1 `ai-constitution-complete.md` 更新

| 位置 | 当前内容 | 更新目标 |
|------|---------|---------|
| §1.1 适用范围 | 引擎：Bevy 0.19 及以上版本 | 引擎：Bevy 0.19 及以上版本 |
| §6.2 ECS 允许清单 | `EventWriter/Reader` 在允许列表中 | 移除 `EventWriter/Reader`，明确 `trigger()` + `On<T>` Observer |
| §6.3 四级通信 | Observer = 局部状态变化响应 | Observer = **跨领域通信首选**，Message 退居领域内部 |
| §6.4 运行条件 | `run_if()` 优先 | 补充：Observer 也支持 `run_if()` |
| §6.4 Timer | 无提及 | 新增：短生命周期效果优先使用 Delayed Commands |
| §6.5 新增 | 无 | BSN 使用规范（UI 层使用，核心玩法层禁用） |
| §6.5 新增 | 无 | Relationship 使用规范（从属关系使用，非关系字段不用） |
| 附则 | v5.1 (Bevy 0.19+) | v5.2 (Bevy 0.19) |

### 3.2 `docs/01-architecture/README.md` 更新

| 位置 | 变更 |
|------|------|
| §4.2 通信矩阵 | 移除 Message (Event) 中的 `EventWriter/EventReader` 示例；将 Observer 提升到"跨 Feature 状态变更"首选 |
| §4.2 注释 | 将注释从"Message 是跨 Feature 广播"改为"Observer 是跨领域首选，Message 是备选" |
| §6.1 Plugin 注册 | 确认 `add_plugins()` 兼容 0.19 API |
| §9 ADR 索引 | 新增 ADR-054: Bevy 0.19 迁移决策 |

### 3.3 `docs/02-domain/` 更新

逐文件检查是否引用了已废弃的 0.18 模式：
- `EventReader`/`EventWriter` 引用 → 改为 `trigger()`/Observer
- `Timer` 引用 → 改为 Delayed Commands
- `font_size: f32` 引用 → 改为 `FontSize`
- 无上述引用的文件：仅更新元数据（`status`, `updated`）

### 3.4 `docs/05-testing/test-spec.md` 更新

新增 §5（或 §17 之后）Observer/Delayed/Relationship 测试规范：

```markdown
### §X Observer 测试规范

- Observer 集成测试必须使用最小 App + `app.add_observer()`
- 验证 Observer 响应：使用 `app.world_mut().trigger(EventType)`
- 验证 Observer 不响应：使用 `run_if` 条件
- 🟥 禁止在单元测试中直接调用 Observer 函数体

### §X Delayed Commands 测试规范

- 延迟命令测试使用 `app.update()` 多次迭代模拟时间流逝
- 验证延迟触发：使用 DelayedId 追踪
- 验证取消：`entity.remove::<FreDelayed<T>>()` 确认未触发
```

### 3.5 `.trae/rules/` 更新

| 文件 | 变更 |
|------|------|
| `ECS规则.md` | 新增 Observer 优先于 EventReader、Delayed 优先于 Timer、BSN 使用范围、Relationship 使用条件 |
| `编码规则.md` | 更新 EventReader/Writer 编码禁令 |
| `架构规则.md` | 更新四级通信机制优先级 |

---

## 4. 特性采用矩阵（最终决策）

| 0.19 特性 | 决策 | 理由 | 执行阶段 |
|-----------|------|------|---------|
| Observer + trigger | ✅ **已采用** | 代码库已全面使用 | Phase 0 |
| `ButtonInput<T>` | ✅ **已采用** | 代码库已全面使用 | Phase 0 |
| `AppExit` | ✅ **已采用** | 代码库已使用 | Phase 0 |
| `Single<T>` | ✅ **已采用** | 代码库已使用 | Phase 0 |
| Run Conditions | ✅ **全量迁移** | 剩余的 if 守卫替换 | Phase 1.2 |
| Delayed Commands | ✅ **全量替换 Timer** | 3 处 Timer 迁移到 Delayed | Phase 1.2 |
| `FontSize` 枚举 | ✅ **全量采用** | 破坏性编译变更 | Phase 1.3 |
| TextFont/FontSource | ✅ **全量采用** | 新 Text API 标准化 | Phase 1.3 |
| Reflect 全覆盖 | ✅ **全量补齐** | 编辑器/序列化基础 | Phase 1.5 |
| DiagnosticsOverlay | ✅ **立即引入** | Dev 模式诊断工具 | Phase 1.6 |
| User Settings | ✅ **立即引入** | 统一设置管理 | Phase 1.6 |
| BSN (bsn!) | 🟡 **UI 层采用** | app/scenes 试点，核心层不碰 | Phase 2.1 |
| Relationship | 🟡 **核心关系采用** | CasterOf/TargetOf 等有限引入 | Phase 2.2 |
| Resource → Entity | 🟡 **评估后决定** | BattleState/TurnState 观察 0.19 稳定版 | Phase 2.3 |
| Contiguous Query | 🟢 **热点场景使用** | 性能不足时启用，不做提前优化 | Phase 2.4 |
| Render Graph as Systems | ⬜ **忽略** | 与本项目 2D SRPG 无关 | — |
| Solari / Skinned Mesh / Bindless | ⬜ **忽略** | 3D 特性，与本项目无关 | — |
| SceneComponent | ⬜ **暂不采用** | 等待 BSN asset loader 稳定 | 未来 |

---

## 5. 风险与缓解

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| dev-deps 版本不一致导致 CI 失败 | 🔴 高 | 阻塞 | Phase 1.1 优先修复 |
| Timer → Delayed 丢失循环语义 | 🟡 中 | 行为 Bug | 循环 Timer 用 `DelayedCommandLoop` 包装 |
| font_size: u8 → f32 精度变化 | 🟢 低 | UI 差异 | 测试覆盖 |
| Reflect 补齐漏类型 | 🟡 中 | 运行时序列化失败 | `register_type` 检查清单 |
| 文档更新遗漏 | 🟡 中 | 知识不一致 | 逐文件 checklist |

---

## 6. 准出条件

### Phase 1 准出

- [ ] `cargo check` 零错误（dev-deps 已修复）
- [ ] `cargo nextest run` 全部通过
- [ ] `cargo clippy -- -D warnings` 零警告
- [ ] 零 Timer 残留（grep `timer.tick\|just_finished\|fn tick` 零结果，测试辅助除外）
- [ ] 零 `font_size: u8\|font_size: f32` 残留
- [ ] 已添加 Reflect derive 到所有 Component/Event/Resource
- [ ] DiagnosticsOverlay 在 dev 模式可用
- [ ] User Settings 已注册

### Phase 2 准出

- [ ] BSN 已用于 app/scenes UI 层
- [ ] Relationship 已用于核心关系（CasterOf/TargetOf）
- [ ] Contiguous Query 已用于热点场景（如有）
- [ ] 战斗特效已添加（Vignette + Lens Distortion）

### Phase 3 准出

- [ ] `ai-constitution-complete.md` 已更新版本号 + 新增 0.19 规则
- [ ] `docs/01-architecture/README.md` 通信机制已更新
- [ ] `docs/04-data/README.md` 数据层映射已校验
- [ ] `docs/05-testing/test-spec.md` 已新增 Observer/Delayed 测试规范
- [ ] `.trae/rules/ECS规则.md` 已更新
- [ ] `.trae/rules/编码规则.md` 已更新
- [ ] `.trae/rules/架构规则.md` 已更新
- [ ] 全量文档已与代码对齐

### 总准出

- [ ] **代码**：`cargo check` + `cargo nextest` + `cargo clippy` 全绿
- [ ] **文档**：宪法/架构/领域/数据/测试/规则 全部对齐 0.19
- [ ] **架构**：所有 ADR 引用与 0.19 代码一致
- [ ] **冒烟**：主菜单 → 角色移动 → 技能使用 → 回合流转 → 存档/读档 全流程正常

---

## 7. 文档影响范围总表

| 文档路径 | 变更类型 | 优先级 | 预计工作量 |
|---------|---------|--------|-----------|
| `Cargo.toml` (dev-deps) | 版本号更新 | 🔴 P0 | 1 行 |
| `docs/00-governance/ai-constitution-complete.md` | 多节修订 + 新增规则 | 🔴 P0 | 2–3 小时 |
| `docs/01-architecture/README.md` | §4.2 §6.1 §9 | 🔴 P0 | 1–2 小时 |
| `docs/01-architecture/ADR-002-ecs-communication.md` | 通信优先级更新 | 🟡 P1 | 30 分钟 |
| `docs/02-domain/README.md` | 元数据更新 + 规则校验 | 🟡 P1 | 1 小时 |
| `docs/02-domain/capabilities/*.md` | 逐文件检查 0.18 引用 | 🟡 P1 | 2 小时 |
| `docs/02-domain/domains/*.md` | 逐文件检查 0.18 引用 | 🟡 P1 | 2 小时 |
| `docs/04-data/README.md` | 校验数据层映射 | 🟡 P1 | 30 分钟 |
| `docs/05-testing/test-spec.md` | 新增 §X Observer/Delayed 测试规范 | 🟡 P1 | 1 小时 |
| `docs/03-technical/bevy-0.19-migration/*.md` | 迁移完成后归档至 `done/` | 🟢 P2 | 10 分钟 |
| `.trae/rules/ECS规则.md` | 新增 Delayed/BSN/Relationship 规则 | 🟡 P1 | 30 分钟 |
| `.trae/rules/编码规则.md` | 更新 EventReader 禁令 | 🟡 P1 | 15 分钟 |
| `.trae/rules/架构规则.md` | 更新通信机制优先级 | 🟡 P1 | 15 分钟 |

---

## 8. 宪法级新规则（v5.2 新增）

迁移完成后，以下规则加入 `ai-constitution-complete.md`：

### 规则 A：Observer 是默认跨领域通信机制

```
🟩 Observer 是跨 Feature 通信的首选机制。
Event 类型仍然使用 #[derive(Event)] 定义，
但发送/接收必须使用 trigger(T) + On<T> Observer 模式。

适用范围：
- 跨 Feature 事件 → Observer（如 TurnEnded → Quest 检查）
- 同 Feature 内逻辑 → 直接 System 调用（不绕过 Observer）

禁止：
- 新的 EventWriter<X> + EventReader<X> 代码模式
- 用 Observer 模拟函数调用（A→B→A 循环）
```

### 规则 B：Delayed Commands 替代 Timer

```
🟩 所有"一次性延迟效果"必须使用 Delayed<T> 或 FreDelayed<T>。
Timer 仅用于：
- 需要暂停/恢复的长周期效果（如可驱散 Buff）
- 需要手动 Advance 的帧动画序列

🟥 禁止：
- 新的 Timer 轮询 System
- 单纯"等 X 秒后执行 Y"用 Timer 实现
```

### 规则 C：BSN 使用范围

```
🟩 BSN 负责描述"实体长什么样"（组件组合），System 负责"实体做什么"。
使用范围：
- UI 层（src/app/scenes/）默认使用 BSN
- 核心玩法层实体生成使用 spawn_*() 工厂函数
- 新增 Feature 的预制体优先使用 BSN 定义

🟥 禁止：
- BSN 描述业务逻辑
- BSN 引用 System/Observer
- 核心玩法层（src/core/）直接 import bsn! 宏
```

### 规则 D：Relationship 使用条件

```
🟨 实体间从属关系使用 Bevy Relationship 机制。
当需要表达"X 属于 Y"、"X 由 Y 创建"、"X 的目标是 Y"时：

1. 定义 #[derive(Relationship)] struct XOf(Entity)
2. 在源 Entity 上添加 Relationship<XOf>
3. 使用 query.get::<Relationship<XOf>>() 查询

例外（继续使用裸 Entity 字段）：
- 临时引用（如"当前选中单位"）
- 值语义的关系（如"队伍 ID"）
- 第三方库期望的字段类型
```

### 规则 E：Reflect 全覆盖

```
🟩 所有 Component/Event/Resource 类型必须 derive Reflect。
#[derive(Component, Reflect)]
#[reflect(Component)]
struct MyComponent { ... }

🟥 禁止新增无 Reflect 的 Component/Asset 类型。
```

---

## 9. 与已有方案的关系

本 v3.0 方案整合并取代 `ignore_this_dir/` 中所有 v1.0–v2.0 方案，主要差异：

| 维度 | v1.0–v2.0 假设 | v3.0 实际发现 |
|------|----------------|---------------|
| 迁移起点 | 0.18.1 全量未动 | 已在使用 0.19.0-rc.3 + Observer 模式 |
| 受影响文件数 | ~210–300 | ~30–50（剩余） |
| 执行周期 | 4–6 周 | 2–3 周 |
| EventReader/Writer | 需要全量替换 | 已替换完成 |
| Bundle | 需要全量替换 | 已替换完成 |
| Input<T> | 需要替换 | 已替换为 ButtonInput |
| Res<T>/ResMut<T> | 需要替换 | 已替换为 Single<T> |
| 最大工作量 | 代码迁移 | **文档对齐** |

---

> **维护者**: @architect | **执行**: @feature-developer + @test-guardian
> **创建日期**: 2026-06-19 | **版本**: v3.0（基于实际代码扫描数据）
> **前置知识库**: `docs/03-technical/bevy-0.19-migration/`（迁移完成后归档至 `docs/09-planning/done/`）
