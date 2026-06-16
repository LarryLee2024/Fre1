---
id: 01-architecture.plugin-contract-rules
title: Plugin Contract Rules
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
  - rules
  - layer
---

# Plugin 边界与依赖契约

Version: 1.1
Status: Proposed
Source: `docs/其他/31遗漏.md` 第一节
Related: `docs/01-architecture/README.md` 插件注册顺序、`docs/01-architecture/00-overview/plugin-design.md`
> **宪法依据**：`docs/AI开发宪法完整版.md` v1.6 — 第3.0节模块化与Plugin边界、第2.2节四级通信机制、第1.5节复杂度预算

---

## 概述

本文档定义 Bevy Plugin 之间的依赖规则、通信边界、初始化顺序和禁止事项。

核心问题：你已有 `plugin-design.md` 讲设计，但缺少插件间的依赖规则与通信边界。插件是 Bevy 的模块化单元，插件边界破了，整个分层架构就形同虚设。

本规范扩展 `plugin-design.md`，补充插件间的契约约束。

---

## 显式依赖规则

### 规则

🟥 **每个 Plugin 必须显式声明其依赖的其他 Plugin。禁止隐式依赖。** — 〔宪法 3.0.2 Plugin是唯一对外入口〕

```rust
// 🟥 禁止：隐式依赖（依赖其他 Plugin 注册的 Resource，但未声明）
pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        // 隐式依赖 SkillRegistry（由 SkillPlugin 注册）
        // 隐式依赖 BuffRegistry（由 BuffPlugin 注册）
        // 但未声明依赖关系
        app.add_systems(Update, battle_system);
    }
}

// ✅ 正确：显式声明依赖
pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        // 声明依赖
        app.add_plugins((SkillPlugin, BuffPlugin));

        // 现在可以安全使用 SkillRegistry 和 BuffRegistry
        app.add_systems(Update, battle_system);
    }
}
```

### 依赖声明方式

```rust
// 方式1：在 build() 中添加依赖 Plugin
impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DependencyA, DependencyB));
    }
}

// 方式2：在 App 层统一注册（推荐）
// main.rs 中按顺序注册所有 Plugin，依赖关系一目了然
App::new()
    .add_plugins((
        SharedPlugin,        // 零依赖
        LogPlugin,           // 依赖 Shared
        EffectPlugin,        // 依赖 Shared
        SkillPlugin,         // 依赖 Shared
        BattlePlugin,        // 依赖 Skill, Buff
        TurnPlugin,          // 依赖 Core
        CharacterPlugin,     // 依赖 Core, Content
        UiPlugin,            // 依赖 Core (只读)
    ))
```

---

## 公共 API 契约

### 规则

🟥 **Plugin 只能通过公共 Event + 公共 Resource + 公共 Component 对外暴露能力。禁止调用其他 Plugin 的私有系统。** — 〔宪法 3.0.1 接口最小化原则〕

```rust
// ✅ 正确：通过公共 API 暴露能力
pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app
            // 公共 Message：其他 Plugin 可以监听
            .add_message::<DamageApplied>()
            .add_message::<HealApplied>()
            .add_message::<CharacterDied>()

            // 公共 Resource：其他 Plugin 可以读取
            .init_resource::<BattleRecord>()

            // 效果管线使用自定义 Schedule（非 .chain()）
            // 详见 docs/01-architecture/00-overview/app-bootstrap.md EffectPipelineSchedule
            // 私有 System：只在本 Plugin 内部执行
            .add_systems(Update, (
                generate_effects,
                modify_effects,
                execute_effects,
            ).run_if(in_state(AppState::InGame)));
    }
}

// 🟥 禁止：调用其他 Plugin 的私有系统
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // 🟥 禁止：直接调用 BattlePlugin 的私有系统
        app.add_systems(Update, battle::execute_effects);  // 私有系统

        // ✅ 正确：监听公共 Message
        app.add_systems(Update, combat_vfx_handler);  // 消费 DamageApplied
    }
}
```

### 公共 API 清单

| 类型 | 可见性 | 用途 |
|------|--------|------|
| Message | `pub` | 跨 Plugin 广播 |
| Resource | `pub` | 全局状态共享 |
| Component | `pub` | Entity 数据 |
| System | 私有（默认） | 只在本 Plugin 内部执行 |
| SystemSet | `pub` | 系统排序约束 |
| State / SubState | `pub` | 状态机定义 |

---

## 分层禁令

### 规则

🟥 **领域 Plugin 禁止依赖 UI Plugin。基础设施 Plugin 禁止反向依赖领域 Plugin。** — 〔宪法 1.3.2 依赖方向铁则〕

```
依赖方向图：

Shared Plugin
    ↑
Infrastructure Plugin ← ✅ 可以依赖 Shared
    ↑
Core Plugin ← ✅ 可以依赖 Shared, Infrastructure
    ↑
Content Plugin ← ✅ 可以依赖 Core, Shared, Infrastructure
    ↑
UI Plugin ← 🟥 禁止依赖 Core（只读 ViewModel）
    ↑
Debug Plugin ← 🟥 禁止依赖 UI
```

### 具体禁令

| Plugin 类型 | 禁止依赖 | 理由 |
|-------------|----------|------|
| Core Plugin | UI Plugin | 领域逻辑不依赖表现层 |
| Core Plugin | Debug Plugin | 领域逻辑不依赖调试工具 |
| Infrastructure Plugin | Core Plugin | 技术实现不反向依赖领域逻辑 |
| Infrastructure Plugin | UI Plugin | 技术实现不依赖表现层 |
| Content Plugin | UI Plugin | 内容加载不依赖表现层 |
| UI Plugin | Debug Plugin | 表现层不依赖调试工具 |

### 违反示例

```rust
// 🟥 禁止：领域 Plugin 依赖 UI Plugin
pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        // 🟥 禁止：依赖 UI Plugin
        app.add_plugins(UiPlugin);

        // ✅ 正确：只依赖领域 Plugin
        app.add_plugins((SkillPlugin, BuffPlugin));
    }
}

// 🟥 禁止：基础设施 Plugin 反向依赖领域 Plugin
pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        // 🟥 禁止：依赖 BattlePlugin
        app.add_plugins(BattlePlugin);

        // ✅ 正确：只依赖 Shared
        app.add_plugins(SharedPlugin);
    }
}
```

---

## Plugin 初始化顺序

### 规则

🟥 **Plugin 必须按以下顺序注册。违反顺序会导致 Resource 未就绪或循环依赖。** — 〔宪法 3.0.2 Plugin是唯一对外入口〕

```
1. Shared Plugin       → 零依赖（最早）
2. Infrastructure Plugin → 依赖 Shared
3. Core Plugin          → 依赖 Shared, Infrastructure
4. Content Plugin       → 依赖 Core, Shared, Infrastructure
5. UI Plugin            → 依赖 Core（只读）
6. Debug Plugin         → 仅开发模式，依赖 Core（只读）
7. Modding Plugin       → 依赖所有
```

### 详细注册表

| 顺序 | 分组 | Plugin | 所在层 | 依赖 |
|------|------|--------|--------|------|
| 1 | Shared | `SharedPlugin` | shared | 无 |
| 2 | Infra | `LogPlugin` | infrastructure | shared |
| 3 | Infra | `AuditPlugin` | infrastructure | shared |
| 4 | Core | `EffectPlugin` | core | shared |
| 5 | Core | `ModifierRulePlugin` | core | shared |
| 6 | Core | `AttributeDefPlugin` | core | shared |
| 7 | Core | `TagDefPlugin` | core | shared |
| 8 | Content | `SkillPlugin` | core | shared |
| 9 | Content | `BuffPlugin` | core | shared |
| 10 | Content | `AiBehaviorPlugin` | core | shared |
| 11 | Content | `EquipmentPlugin` | core | shared |
| 12 | Content | `InventoryPlugin` | core | shared |
| 13 | Content | `AssetsPlugin` | content | core, infra |
| 14 | Logic | `TurnPlugin` | core | core |
| 15 | Logic | `MapPlugin` | core | core |
| 16 | Logic | `CharacterPlugin` | core | core, content |
| 17 | Logic | `BattlePlugin` | core | core, content |
| 18 | Logic | `AiPlugin` | core | core, content |
| 19 | Logic | `CampaignPlugin` | core | content |
| 20 | UI | `UiPlugin` | ui | core (只读) |
| 21 | UI | `InputPlugin` | ui | core (只读) |
| 22 | Debug | `DebugPlugin` | debug | core (只读) |

### 违反顺序的后果

```rust
// 🟥 禁止：UI Plugin 在 Core Plugin 之前注册
App::new()
    .add_plugins(UiPlugin)           // 🟥 UI 在 Core 之前
    .add_plugins(BattlePlugin);      // Core Plugin

// 后果：UiPlugin 依赖的 ViewModel Resource 未注册
```

---

## Plugin 间通信

### 规则

🟥 **Plugin 之间只能通过注册的 Message/Observer 或共享 Resource 通信。禁止绕过 Plugin 边界直接注册资源。** — 〔宪法 3.0.4 跨模块交互规范〕

```rust
// ✅ 正确：通过 Message 通信
// BattlePlugin 发送 DamageApplied
fn execute_damage_system(
    mut writer: MessageWriter<DamageApplied>,
) {
    writer.write(DamageApplied { /* ... */ });
}

// UiPlugin 消费 DamageApplied
fn combat_vfx_handler(
    mut reader: MessageReader<DamageApplied>,
) {
    for msg in reader.read() {
        // 播放飘字动画
    }
}

// ✅ 正确：通过共享 Resource 通信
// BattlePlugin 写入 BattleRecord
fn record_battle_event_system(
    mut record: ResMut<BattleRecord>,
) {
    record.add_event(/* ... */);
}

// DebugPlugin 读取 BattleRecord
fn debug_battle_panel(
    record: Res<BattleRecord>,
) {
    // 显示战斗记录
}

// 🟥 禁止：绕过 Plugin 边界直接注册资源
impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        // 🟥 禁止：在 App 层直接注册 UI 相关资源
        app.insert_resource(UiState::default());  // UI 资源属于 UiPlugin

        // ✅ 正确：只注册本 Plugin 的资源
        app.init_resource::<BattleRecord>();
    }
}
```

### 通信方式选择

| 方式 | 用途 | 适用场景 |
|------|------|----------|
| Message | 跨 Plugin 广播 | DamageApplied, CharacterDied |
| Observer | 同 Plugin 局部响应 | 死亡动画、UI 刷新 |
| Resource | 全局状态共享 | BattleRecord, TurnOrder |
| Hook | 组件固有行为 | Dead 标签添加时清理 |

---

## 禁止循环依赖

### 规则

🟥 **禁止 Plugin 间循环依赖（A 依赖 B，B 又依赖 A）。** — 〔宪法 1.3.2 依赖方向铁则〕

```rust
// 🟥 禁止：循环依赖
pub struct PluginA;
pub struct PluginB;

impl Plugin for PluginA {
    fn build(&self, app: &mut App) {
        app.add_plugins(PluginB);  // A 依赖 B
    }
}

impl Plugin for PluginB {
    fn build(&self, app: &mut App) {
        app.add_plugins(PluginA);  // B 依赖 A → 循环！
    }
}

// ✅ 正确：提取公共依赖到 Shared
pub struct SharedPlugin;

impl Plugin for PluginA {
    fn build(&self, app: &mut App) {
        app.add_plugins(SharedPlugin);  // A 依赖 Shared
    }
}

impl Plugin for PluginB {
    fn build(&self, app: &mut App) {
        app.add_plugins(SharedPlugin);  // B 依赖 Shared
    }
}
```

### 检测方法

```
Plugin 依赖图检测：
├─ 是否存在环？→ 重构模块边界
├─ 是否超过 3 层依赖？→ 考虑提取 Shared
└─ 是否有隐式依赖？→ 添加显式声明
```

---

## 测试隔离

### 规则

🟩 **每个 Plugin 必须可以独立测试，仅需 Mock 其声明的依赖。** — 〔宪法 18.2.1 三层测试体系〕

```rust
// ✅ 正确：Plugin 独立测试
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_battle_plugin_registers_required_resources() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Mock 依赖
        app.init_resource::<SkillRegistry>();
        app.init_resource::<BuffRegistry>();

        // 添加被测 Plugin
        app.add_plugins(BattlePlugin);

        // 验证 Resource 已注册
        assert!(app.world().get_resource::<BattleRecord>().is_some());
    }

    #[test]
    fn test_battle_plugin_registers_required_messages() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(BattlePlugin);

        // 验证 Message 已注册
        // （具体验证方式取决于 Bevy 版本）
    }
}
```

### Mock 规则

- 🟩 Mock 的 Resource 必须与真实 Resource 接口一致
- 🟩 Mock 不应包含业务逻辑，只返回固定值
- 🟥 禁止通过修改业务逻辑让测试通过

---

## 禁止事项总览

> **宪法依据**：〔宪法 3.0.1-3.0.7 模块化与Plugin边界〕

| 禁止项 | 理由 | 替代方案 | 宪法条款 |
|--------|------|----------|----------|
| 🟥 Plugin 隐式依赖其他 Plugin | Resource 未就绪时运行会 panic | 显式声明依赖 | 3.0.2 |
| 🟥 调用其他 Plugin 的私有系统 | 破坏模块边界，无法独立测试 | 通过 Message/Resource 通信 | 3.0.1 |
| 🟥 领域 Plugin 依赖 UI Plugin | 领域逻辑不依赖表现层 | 通过 ViewModel 只读通信 | 1.3.2 |
| 🟥 基础设施 Plugin 反向依赖领域 Plugin | 技术实现不依赖业务逻辑 | 通过 Message 监听领域事件 | 1.3.2 |
| 🟥 Plugin 注册顺序错误 | 后注册的 Plugin 依赖先注册的 Resource | 按 Shared → Infra → Core → Content → UI 顺序注册 | 3.0.2 |
| 🟥 绕过 Plugin 边界直接注册资源 | 破坏 Plugin 的封装性 | 在 Plugin 的 build() 中注册 | 3.0.1 |
| 🟥 Plugin 间循环依赖 | 编译错误、难以维护 | 提取公共依赖到 Shared | 1.3.2 |
| 🟥 Plugin::build() 中执行业务逻辑 | Plugin 只负责声明，不负责执行 | 业务逻辑放在 System 中 | 3.0.2 |
| 🟥 跨层注册（UI Plugin 注册 Core 系统的 Message） | 跨层注册破坏分层架构 | 跨层 Message 在 App 层注册 | 1.3.2 |
| 🟥 为单个实现创建 Plugin | 过度拆分 | 按业务领域拆分 | 1.5.2 |

---

## 允许的模式

### 模式1：Plugin 显式声明依赖

```rust
pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app
            // 显式依赖
            .add_plugins((SkillPlugin, BuffPlugin))

            // 公共 API
            .add_message::<DamageApplied>()
            .add_message::<CharacterDied>()
            .init_resource::<BattleRecord>()

            // 私有系统
            .add_systems(Update, (
                generate_effects,
                modify_effects,
                execute_effects,
            ).chain().run_if(in_state(AppState::InGame)));
    }
}
```

### 模式2：App 层统一注册

```rust
// main.rs：唯一允许全局视野的层
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Shared
        .add_plugins(SharedPlugin)
        // Infrastructure
        .add_plugins((LogPlugin, AuditPlugin))
        // Core
        .add_plugins((EffectPlugin, ModifierRulePlugin, AttributeDefPlugin, TagDefPlugin))
        // Content
        .add_plugins((SkillPlugin, BuffPlugin, AiBehaviorPlugin, EquipmentPlugin, InventoryPlugin))
        .add_plugins(AssetsPlugin)
        // Logic
        .add_plugins((TurnPlugin, MapPlugin, CharacterPlugin, BattlePlugin, AiPlugin))
        .add_plugins(CampaignPlugin)
        // UI
        .add_plugins((UiPlugin, InputPlugin))
        // Debug（仅开发模式）
        .add_plugins(DebugPlugin)
        .run();
}
```

### 模式3：Plugin 间 Message 通信

```rust
// BattlePlugin：发送 DamageApplied
impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DamageApplied>();
    }
}

// UiPlugin：消费 DamageApplied
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, combat_vfx_handler.run_if(in_state(AppState::InGame)));
    }
}

// combat_vfx_handler 消费 DamageApplied Message
fn combat_vfx_handler(mut reader: MessageReader<DamageApplied>) {
    for msg in reader.read() {
        // 播放飘字动画
    }
}
```

---

## 动态依赖支持

> **优化来源**: `docs/其他/62.md`

### 条件依赖模式

静态依赖声明无法处理运行时条件依赖。对于可选功能，使用 `cfg(feature = "...")` 条件依赖模式：

```rust
// ✅ 条件依赖：Feature 开关 + 条件注册
impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        // 强依赖：必须注册
        app.add_plugins((SkillPlugin, BuffPlugin));

        // 弱依赖：条件注册（通过 Feature Flag 控制）
        #[cfg(feature = "advanced_ai")]
        app.add_plugins(AdvancedAiPlugin);

        #[cfg(feature = "debug_tools")]
        app.add_plugins(DebugOverlayPlugin);
    }
}

// ✅ 运行时条件依赖：检查 Resource 是否存在
impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        // 强依赖：无条件注册
        app.add_plugins((SkillPlugin, BuffPlugin));

        // 弱依赖：运行时检查（仅当 Resource 已存在时注册）
        if app.world().contains_resource::<DebugSettings>() {
            app.add_plugins(DebugPlugin);
        }
    }
}
```

### 规则

- 🟥 **强依赖必须在 `build()` 中无条件注册** — 缺失强依赖会导致 panic
- 🟩 **弱依赖通过 `#[cfg(feature = "...")]` 条件编译注册** — 缺失弱依赖时优雅降级
- 🟩 **弱依赖缺失时必须有兜底逻辑** — 不能因为缺少可选 Plugin 而导致功能异常
- 🟩 **App 层统一管理所有 Feature Flag** — 禁止各 Plugin 内部随意检查 Feature

---

## Plugin 版本兼容性契约

> **优化来源**: `docs/其他/62.md`

### Semantic Versioning 规则

Plugin API 遵循 SemVer：`MAJOR.MINOR.PATCH`

| 版本变更 | 含义 | 示例 |
|---------|------|------|
| MAJOR | 公共 API 断裂（删除/修改 Message、Resource、Component） | `DamageApplied` 字段变更 |
| MINOR | 新增公共 API（新增 Message、Resource） | 新增 `SkillCastFailed` Message |
| PATCH | 内部修复，不影响公共 API | 修复伤害计算精度 |

### 版本声明

```rust
pub struct BattlePlugin {
    version: PluginVersion, // MAJOR.MINOR.PATCH
}

pub struct PluginVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        // 声明公共 API 变更
        app.add_plugins(PluginCompatibilityCheck::new(self.version.clone()));
    }
}
```

### 规则

- 🟥 **MAJOR 版本变更必须提供迁移路径** — 旧版本 Message 的消费者必须能平滑升级
- 🟩 **MINOR 版本变更应保持向后兼容** — 新增 API 不影响现有消费者
- 🟩 **PATCH 版本变更不应影响公共 API** — 仅修复内部逻辑

---

## 强/弱依赖区分

> **优化来源**: `docs/其他/62.md`

### 强依赖

插件运行**必须**初始化的依赖。缺失时会导致 panic 或功能异常。

```rust
// 强依赖：BattlePlugin 运行必须依赖 SkillPlugin 和 BuffPlugin
impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        // 🟥 强依赖：无条件注册
        app.add_plugins((SkillPlugin, BuffPlugin));
    }
}
```

### 弱依赖

可选功能依赖的插件。缺失时优雅降级，不影响核心功能。

```rust
// 弱依赖：BattlePlugin 可选依赖 AdvancedAiPlugin
impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        // 🟩 弱依赖：条件注册
        #[cfg(feature = "advanced_ai")]
        app.add_plugins(AdvancedAiPlugin);

        // 核心功能不依赖 AdvancedAiPlugin
        app.add_systems(Update, battle_system);
    }
}
```

### 区分标准

| 维度 | 强依赖 | 弱依赖 |
|------|--------|--------|
| 缺失后果 | panic / 功能异常 | 优雅降级 |
| 注册方式 | 无条件 `add_plugins()` | `#[cfg(feature = "...")]` |
| 适用场景 | 核心业务逻辑 | 扩展功能、调试工具 |
| 测试要求 | 必须 Mock 注册 | 可选 Mock |

---

## 依赖检测自动化工具

> **优化来源**: `docs/其他/62.md`

### 编译时检查（proc_macro）

开发过程宏在编译时验证 Plugin 依赖声明：

```rust
// 使用过程宏强制验证依赖声明
#[derive(PluginContract)]
#[requires(SkillPlugin, BuffPlugin)]  // 强依赖
#[optional(AdvancedAiPlugin)]         // 弱依赖
pub struct BattlePlugin;
```

### CI 依赖图分析

在 CI 中运行依赖图分析，自动检测：

```bash
# 生成依赖图
cargo plugin-graph --output=plugin-dependencies.dot

# 检测循环依赖
cargo plugin-graph --check-cycles

# 检测跨层依赖违规
cargo plugin-graph --forbidden "core:ui, core:debug"

# 输出依赖分析报告
cargo plugin-graph --report=ci-report.md
```

### 规则

- 🟥 **CI 必须运行依赖图分析** — 每次 PR 必须通过依赖检查
- 🟩 **proc_macro 编译时验证依赖声明** — 编译阶段拦截违规
- 🟩 **运行时监控 Plugin 间非法访问** — 调试模式下检测越界调用

---

## 性能契约

> **优化来源**: `docs/其他/62.md`
> ⚠️ **宪法 1.5.2 警告**：以下 `PluginPerformanceContract` trait 为预留扩展点设计，属于轻量级接口预留。在对应功能未实现前，禁止提前实现完整框架。仅允许定义 trait 接口，禁止添加复杂实现逻辑。

### 定义

每个 Plugin 定义性能预算元数据，作为架构约束的一部分：

```rust
pub trait PluginPerformanceContract {
    /// 单帧最大执行时间（毫秒）
    fn max_frame_time(&self) -> Duration;

    /// 最大内存占用（字节）
    fn max_memory_footprint(&self) -> usize;

    /// 最大 Message 队列长度
    fn max_message_queue_size(&self) -> usize;
}

impl PluginPerformanceContract for BattlePlugin {
    fn max_frame_time(&self) -> Duration {
        Duration::from_millis(2) // 战斗系统单帧不超过 2ms
    }

    fn max_memory_footprint(&self) -> usize {
        1024 * 1024 // 1MB
    }

    fn max_message_queue_size(&self) -> usize {
        256 // 最多 256 条待处理 Message
    }
}
```

### 规则

- 🟩 **关键 Plugin 必须定义性能契约** — BattlePlugin、TurnPlugin 等高频 Plugin
- 🟩 **性能契约应与 `SystemSet` 并行策略关联** — 高频系统使用并行执行
- 🟩 **高频 Message 建议批量处理** — 避免单帧触发过多事件导致性能损耗
- 🟩 **Plugin 拆分粒度参考性能预算** — 单 Plugin 超出预算时考虑拆分

---

## 交叉引用

| 文档 | 关系 |
|------|------|
| `docs/AI开发宪法完整版.md` | 宪法第3.0节模块化与Plugin边界、第2.2节四级通信机制、第1.5节复杂度预算 |
| `docs/01-architecture/README.md` | 插件注册顺序（第 724-747 行） |
| `docs/01-architecture/00-overview/plugin-design.md` | Plugin 设计模式、粒度规则 |
| `docs/01-architecture/00-overview/layer-contracts.md` | 各层 Plugin 职责边界 |
| `docs/01-architecture/00-overview/app-bootstrap.md` | App 层启动序列与 Plugin 组装、EffectPipelineSchedule |
| `docs/01-architecture/component_design_rules.md` | Plugin 注册的 Component 设计规范 |
| `docs/01-architecture/system_design_rules.md` | Plugin 注册的 System 编写规范 |
| `docs/02-domain/ecs_communication_rules.md` | Plugin 间通信方式 |
| `docs/其他/31遗漏.md` | 本文档的原始需求来源（第 202-208 行） |
