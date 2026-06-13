# Plugin 组织设计

> Version: 1.0
> Status: Proposed
> 来源：`docs/其他/30.md` 第1410-1441行（Plugin 细化）、`docs/architecture.md`（七层架构、插件注册顺序）

---

## 1. Plugin 哲学

Plugin 是 Bevy 原生的模块化单元。每个 Plugin 封装一个连贯的功能片段。

```
Plugin = 模块边界 = 编译单元 = 功能封装
```

Plugin 的本质是**声明式的**——它告诉 Bevy "我需要什么 Resource、什么 System、什么事件"，但**不在 `build()` 中执行任何业务逻辑**。

---

## 2. Plugin 分类

### 2.1 按七层架构组织

Plugin 不应该放在独立的 `plugins/` 目录中。每个 Plugin 属于其所在层的业务模块。

```
src/
├── app/
│   └── app_plugin.rs          # 主 Plugin（全局组装者）
├── core/
│   ├── battle/plugin.rs       # 战斗 Plugin
│   ├── turn/plugin.rs         # 回合 Plugin
│   ├── character/plugin.rs    # 角色 Plugin
│   ├── skill/plugin.rs        # 技能 Plugin
│   ├── buff/plugin.rs         # Buff Plugin
│   ├── ai/plugin.rs           # AI Plugin
│   ├── equipment/plugin.rs    # 装备 Plugin
│   ├── inventory/plugin.rs    # 物品 Plugin
│   ├── map/plugin.rs          # 地图 Plugin
│   ├── effect/plugin.rs       # 效果管线 Plugin
│   ├── modifier_rule/plugin.rs # 修饰规则 Plugin
│   ├── attribute_def/plugin.rs # 属性定义 Plugin
│   └── tag_def/plugin.rs      # 标签定义 Plugin
├── shared/
│   └── shared_plugin.rs       # 共享工具 Plugin
├── infrastructure/
│   ├── logging/plugin.rs      # 日志 Plugin
│   ├── audit/plugin.rs        # 审计 Plugin
│   ├── save/plugin.rs         # 存档 Plugin
│   └── replay/plugin.rs       # 回放 Plugin
├── content/
│   └── content_plugin.rs      # 内容加载 Plugin（统一入口）
├── ui/
│   └── ui_plugin.rs           # UI Plugin
├── debug/
│   └── debug_plugin.rs        # 调试 Plugin
└── modding/
    └── mod_plugin.rs          # MOD 支持 Plugin
```

### 2.2 设计决策：Plugin 位置

| 方案 | 优点 | 缺点 |
|------|------|------|
| ✅ **Plugin 在层目录内**（`core/battle/plugin.rs`） | 与业务边界一致，编译单元清晰 | 跨层依赖需在 App 层协调 |
| ❌ 独立 `plugins/` 目录 | 集中管理方便 | 打破业务边界，Plugin 与实现分离 |

**决策**：Plugin 属于其所在业务模块。30.md 中的 `plugins/` 建议是组织代码的方式，但实际业务边界仍归属于 `core`、`content`、`infrastructure`、`shared`。

---

## 3. Plugin 注册顺序

注册顺序对 Bevy 至关重要——后注册的 Plugin 可以依赖先注册的 Plugin 注册的 Resource。

### 3.1 推荐顺序

```
1. Shared Plugins       (零依赖，最早)
2. Infrastructure Plugins (依赖 Shared)
3. Core Plugins          (依赖 Shared)
4. Content Plugins       (依赖 Core + Infra + Shared)
5. Logic Plugins         (依赖 Core + Content)
6. UI Plugins            (依赖 ViewModel only)
7. Debug Plugins         (仅开发模式，依赖 Core)
8. Modding Plugins       (依赖所有)
```

### 3.2 详细注册表

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

### 3.3 注册顺序规则

- 🟥 **表现层插件禁止在数据层之前注册**
- 🟥 **逻辑层插件禁止在核心层之前注册**
- 🟥 **禁止跳过任何注册顺序**
- 🟩 **每个分组内用元组批量注册**（`add_plugins((A, B, C))`）

---

## 4. Plugin 设计模式

### 4.1 标准 Plugin 结构

```rust
pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册 Message（跨模块广播事件）
            .add_message::<DamageApplied>()
            .add_message::<HealApplied>()
            .add_message::<CharacterDied>()

            // 注册 Resource（全局状态）
            .init_resource::<EffectQueue>()

            // 注册 System（业务逻辑）
            .add_systems(Update, (
                generate_effects,
                modify_effects,
                execute_effects,
            ).chain().run_if(in_state(AppState::InGame)))
    }
}
```

### 4.2 Plugin 组成要素

| 要素 | 作用 | 禁止 |
|------|------|------|
| `Message` | 跨模块广播事件 | 禁止在 build 中触发事件 |
| `Resource` | 全局状态 | 禁止在 build 中修改 Resource |
| `System` | 业务逻辑 | 禁止在 build 中直接执行业务逻辑 |
| `SystemSet` | 系统排序约束 | 禁止在 build 中执行跨 Set 逻辑 |
| `SubState` | 层级状态机 | 禁止在 build 中切换状态 |

### 4.3 run_if 条件

Plugin 注册的 System 应该使用 `run_if` 控制执行条件：

```rust
// 回合相关系统只在 InGame 时执行
.add_systems(Update, (
    turn_system,
    movement_system,
).chain().run_if(in_state(AppState::InGame)))

// UI 系统只在对应状态时执行
.add_systems(Update, ui_system.run_if(in_state(AppState::MainMenu)))
```

### 4.4 SubState 注册

```rust
// 注册 TurnPhase 为 AppState::InGame 的 SubState
app.add_sub_state::<TurnPhase>();

// 系统只在特定 TurnPhase 执行
app.add_systems(Update, (
    select_unit_system.run_if(in_state(TurnPhase::SelectUnit)),
    move_unit_system.run_if(in_state(TurnPhase::MoveUnit)),
    execute_action_system.run_if(in_state(TurnPhase::ExecuteAction)),
));
```

---

## 5. Plugin 粒度规则

### 5.1 粒度标准

| 层 | Plugin 粒度 | 示例 |
|----|-------------|------|
| Core | 每个业务模块一个 Plugin | `BattlePlugin`, `TurnPlugin`, `SkillPlugin` |
| Shared | 一个 `SharedPlugin` 统一管理 | `SharedPlugin` |
| Infrastructure | 每个基础设施一个 Plugin | `LogPlugin`, `AuditPlugin`, `SavePlugin` |
| Content | 一个 `ContentPlugin` 统一入口 | `ContentPlugin`（内部注册所有加载器） |
| UI | 一个 `UiPlugin` 统一管理 | `UiPlugin`（内部注册所有面板） |
| Debug | 一个 `DebugPlugin` 统一管理 | `DebugPlugin`（内部注册所有调试面板） |
| Modding | 一个 `ModPlugin` 统一管理 | `ModPlugin` |

### 5.2 拆分原则

- 🟩 **Plugin 职责过大时必须拆分**
- 🟩 **按业务领域拆分，不按代码数量拆分**
- 🟥 **禁止为单个实现创建 Trait**（Plugin 本身不是 Trait）

### 5.3 粒度判断

```
一个 Plugin 是否需要拆分？
├─ 超过 3 个不同的业务领域？→ 拆分
├─ 超过 50 个 System？→ 考虑拆分
├─ 不同的 System 需要不同的 run_if？→ 拆分
└─ 否则 → 保持单一 Plugin
```

---

## 6. 条件编译与 Feature Flags

### 6.1 Debug Plugin

```rust
// main.rs 或 app_plugin.rs
#[cfg(feature = "dev")]
app.add_plugins(DebugPlugin);
```

### 6.2 Modding Plugin

```rust
#[cfg(feature = "modding")]
app.add_plugins(ModdingPlugin);
```

### 6.3 Feature Flag 规则

| Feature | 说明 | 默认 |
|---------|------|------|
| `dev` | 开发工具（调试面板、World Inspector） | ❌ 关闭 |
| `modding` | MOD 支持 | ❌ 关闭 |
| `replay` | 战斗回放 | ⚠️ 可选 |

- 🟥 **Debug Plugin 禁止在生产构建中注册**
- 🟩 **Feature Flag 应在 Cargo.toml 中统一管理**

---

## 7. Plugin 间通信

### 7.1 通信方式

Plugin 之间**禁止直接访问内部组件或状态**。通信方式：

| 方式 | 用途 | 示例 |
|------|------|------|
| `Message` | 跨模块广播 | `DamageApplied` → battle → UI |
| `Observer` | 同模块局部响应 | 死亡动画播放 |
| `Hook` | 组件固有行为 | `Dead` 标签添加时移除移动组件 |
| `Command` | UI → 业务 | `UiCommand` → command_handler |

### 7.2 Message 注册

每个 Plugin 在 `build()` 中注册自己发送的 Message：

```rust
// battle/plugin.rs
impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DamageApplied>()   // battle 发送
          .add_message::<HealApplied>()      // battle 发送
          .add_message::<CharacterDied>();   // battle 发送
    }
}

// ui/plugin.rs
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            combat_vfx_handler,      // 接收 DamageApplied
            combat_log_handler,      // 接收 DamageApplied, HealApplied, CharacterDied
            buff_panel_handler,      // 接收 BuffApplied, BuffRemoved
        ).run_if(in_state(AppState::InGame)));
    }
}
```

### 7.3 通信规则

- 🟥 **禁止 Plugin 间循环依赖**（A 依赖 B，B 又依赖 A）
- 🟥 **禁止跨层注册**（UI Plugin 禁止注册 Core 系统的 Message）
- 🟩 **跨层 Message 注册在 App 层做**（App 是唯一允许全局视野的层）
- 🟩 **模块内部优先函数调用**（不需要走 Message）

---

## 8. Plugin 测试

### 8.1 独立测试

每个 Plugin 必须可以独立测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_battle_plugin_registers_required_resources() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(BattlePlugin);

        // 验证 Resource 已注册
        assert!(app.world().get_resource::<EffectQueue>().is_some());
    }

    #[test]
    fn test_battle_plugin_registers_required_messages() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(BattlePlugin);

        // 验证 Message 已注册
        // （具体验证方式取决于 Bevy 版本）
    }

    #[test]
    fn test_battle_plugin_systems_run_in_correct_state() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(BattlePlugin);

        // 设置状态为 InGame
        app.world_mut().insert_resource(NextState::<AppState>::default());
        app.world_mut().get_mut::<AppState>() = Some(AppState::InGame);

        // 验证系统在正确状态下执行
    }
}
```

### 8.2 测试规则

- 🟩 **每个 Plugin 必须有独立测试**
- 🟩 **验证 Plugin 注册了所有预期的 Resource/Message/System**
- 🟩 **Mock 依赖进行隔离测试**
- 🟥 **禁止通过修改业务逻辑让测试通过**

---

## 9. 禁止事项

| 禁止项 | 理由 | 替代方案 |
|--------|------|----------|
| 🟥 Plugin 内部包含跨模块注册逻辑 | 每个 Plugin 只管自己 | 跨层注册在 App 层做 |
| 🟥 Plugin::build 中执行业务逻辑 | 只注册不执行 | 业务逻辑放在 System 中 |
| 🟥 Plugin 间循环依赖 | 编译错误、难以维护 | 重新设计模块边界 |
| 🟥 UI Plugin 注册 Core 系统的 Message | 跨层注册 | 跨层 Message 在 App 层注册 |
| 🟥 跳过 Plugin 直接注册 System | 破坏模块边界 | 通过 Plugin 注册 |
| 🟥 为单个实现创建 Plugin | 过度拆分 | 按业务领域拆分 |

---

## 10. 交叉引用

| 文档 | 关系 |
|------|------|
| `docs/architecture.md` | 七层架构、插件注册顺序（第724-747行） |
| `docs/architecture/app-bootstrap.md` | App 层启动序列与 Plugin 组装 |
| `docs/architecture/layer-contracts.md` | 各层 Plugin 职责边界 |
| `docs/architecture/project-structure.md` | Plugin 目录结构（第80-94行） |
| `docs/其他/30.md` | 原始 Plugin 建议（第1410-1441行） |

---

## 附录 A：当前实现与设计对照

| 设计原则 | 当前 main.rs 状态 | 差距 |
|----------|-------------------|------|
| Plugin 在层目录内 | ⚠️ 部分实现（`battle/`, `turn/` 有 plugin.rs） | 需统一所有模块 |
| 注册顺序分组 | ⚠️ 已有分组但注释不清晰 | 需更明确的分层注释 |
| Debug 条件编译 | ❌ DebugPlugin 无条件注册 | 需加 `#[cfg(feature = "dev")]` |
| ContentPlugin 统一入口 | ❌ Content 模块 Plugin 分散注册 | 需创建 ContentPlugin |
| SharedPlugin 统一入口 | ❌ 无 SharedPlugin | 需创建 SharedPlugin |
| AppPlugin 统一入口 | ❌ main.rs 直接注册所有 | 需重构为 AppPlugin |
