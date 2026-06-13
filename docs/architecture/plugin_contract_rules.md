# Plugin 边界与依赖契约

Version: 1.0
Status: Proposed
Source: `docs/其他/31遗漏.md` 第一节
Related: `docs/architecture.md` 插件注册顺序、`docs/architecture/plugin-design.md`

---

## 概述

本文档定义 Bevy Plugin 之间的依赖规则、通信边界、初始化顺序和禁止事项。

核心问题：你已有 `plugin-design.md` 讲设计，但缺少插件间的依赖规则与通信边界。插件是 Bevy 的模块化单元，插件边界破了，整个分层架构就形同虚设。

本规范扩展 `plugin-design.md`，补充插件间的契约约束。

---

## 显式依赖规则

### 规则

🟥 **每个 Plugin 必须显式声明其依赖的其他 Plugin。禁止隐式依赖。**

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

🟥 **Plugin 只能通过公共 Event + 公共 Resource + 公共 Component 对外暴露能力。禁止调用其他 Plugin 的私有系统。**

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

            // 私有 System：只在本 Plugin 内部执行
            .add_systems(Update, (
                generate_effects,
                modify_effects,
                execute_effects,
            ).chain());
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

🟥 **领域 Plugin 禁止依赖 UI Plugin。基础设施 Plugin 禁止反向依赖领域 Plugin。**

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

🟥 **Plugin 必须按以下顺序注册。违反顺序会导致 Resource 未就绪或循环依赖。**

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

🟥 **Plugin 之间只能通过注册的 Message/Observer 或共享 Resource 通信。禁止绕过 Plugin 边界直接注册资源。**

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

🟥 **禁止 Plugin 间循环依赖（A 依赖 B，B 又依赖 A）。**

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

🟩 **每个 Plugin 必须可以独立测试，仅需 Mock 其声明的依赖。**

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

| 禁止项 | 理由 | 替代方案 |
|--------|------|----------|
| 🟥 Plugin 隐式依赖其他 Plugin | Resource 未就绪时运行会 panic | 显式声明依赖 |
| 🟥 调用其他 Plugin 的私有系统 | 破坏模块边界，无法独立测试 | 通过 Message/Resource 通信 |
| 🟥 领域 Plugin 依赖 UI Plugin | 领域逻辑不依赖表现层 | 通过 ViewModel 只读通信 |
| 🟥 基础设施 Plugin 反向依赖领域 Plugin | 技术实现不依赖业务逻辑 | 通过 Message 监听领域事件 |
| 🟥 Plugin 注册顺序错误 | 后注册的 Plugin 依赖先注册的 Resource | 按 Shared → Infra → Core → Content → UI 顺序注册 |
| 🟥 绕过 Plugin 边界直接注册资源 | 破坏 Plugin 的封装性 | 在 Plugin 的 build() 中注册 |
| 🟥 Plugin 间循环依赖 | 编译错误、难以维护 | 提取公共依赖到 Shared |
| 🟥 Plugin::build() 中执行业务逻辑 | Plugin 只负责声明，不负责执行 | 业务逻辑放在 System 中 |
| 🟥 跨层注册（UI Plugin 注册 Core 系统的 Message） | 跨层注册破坏分层架构 | 跨层 Message 在 App 层注册 |
| 🟥 为单个实现创建 Plugin | 过度拆分 | 按业务领域拆分 |

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

## 交叉引用

| 文档 | 关系 |
|------|------|
| `docs/architecture.md` | 插件注册顺序（第 724-747 行） |
| `docs/architecture/plugin-design.md` | Plugin 设计模式、粒度规则 |
| `docs/architecture/layer-contracts.md` | 各层 Plugin 职责边界 |
| `docs/architecture/component_design_rules.md` | Plugin 注册的 Component 设计规范 |
| `docs/architecture/system_design_rules.md` | Plugin 注册的 System 编写规范 |
| `docs/domain/ecs_communication_rules.md` | Plugin 间通信方式 |
| `docs/其他/31遗漏.md` | 本文档的原始需求来源（第 202-208 行） |
