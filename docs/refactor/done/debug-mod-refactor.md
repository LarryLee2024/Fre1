# Debug mod.rs 重构建议

## 当前状态

文件: `src/debug/mod.rs` (489 lines)

包含内容:
- DebugView 枚举 + all() 方法
- DebugPanelState 资源
- WorldInspectorState 资源
- debug_hotkey_system
- DebugPlugin 实现
- setup_egui_font
- world_inspector_ui
- unified_debug_panel (~100 lines)

## 问题分析

| 问题 | 影响 |
|------|------|
| 多个独立关注点混合 | 难以单独测试/修改 |
| unified_debug_panel 参数过多 | 12+ 参数，维护困难 |
| 快捷键逻辑分散 | hotkey_system + unified_debug_panel 都处理 F3 |
| egui 字体初始化逻辑 | 应作为独立功能 |

## 重构方案

### 方案: 按职责拆分 (推荐)

```
src/debug/
├── mod.rs              # Plugin 注册 (≤80 lines)
├── state.rs            # DebugView, DebugPanelState, WorldInspectorState
├── hotkeys.rs          # debug_hotkey_system
├── egui_setup.rs       # setup_egui_font
├── panel.rs            # unified_debug_panel, world_inspector_ui
├── overlay.rs          # (已存在)
├── stepping_control.rs # (已存在)
├── gizmos_viz.rs       # (已存在)
└── viewers/            # (已存在)
```

### 各文件职责

**state.rs** (~60 lines)
```rust
pub enum DebugView { ... }
impl DebugView { pub fn all() -> ... }
pub struct DebugPanelState { ... }
pub struct WorldInspectorState { ... }
```

**hotkeys.rs** (~70 lines)
```rust
pub fn debug_hotkey_system(
    state: ResMut<DebugPanelState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    stepping: ResMut<Stepping>,
    stepping_state: ResMut<DebugSteppingState>,
    world_inspector: ResMut<WorldInspectorState>,
    overlay: ResMut<DebugOverlay>,  // F3 逻辑统一到这里
) { ... }
```

**egui_setup.rs** (~40 lines)
```rust
pub fn setup_egui_font(...) { ... }
```

**panel.rs** (~120 lines)
```rust
pub fn unified_debug_panel(
    state: ResMut<DebugPanelState>,
    egui_ctx: ...,
    units: Query<...>,
    // 只保留面板渲染必需的参数
) { ... }

pub fn world_inspector_ui(world: &mut World) { ... }
```

**mod.rs** (~80 lines)
```rust
mod state;
mod hotkeys;
mod egui_setup;
mod panel;
// ... 其他已有模块

pub use state::{DebugView, DebugPanelState, WorldInspectorState};

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultInspectorConfigPlugin)
            .insert_resource(state::DebugPanelState::default())
            // ...
            .add_systems(PreUpdate, hotkeys::debug_hotkey_system)
            .add_systems(EguiPrimaryContextPass, (
                egui_setup::setup_egui_font,
                panel::unified_debug_panel,
                panel::world_inspector_ui,
            ).chain())
            // ...
    }
}
```

## unified_debug_panel 参数优化

当前参数过多的原因: 每个 viewer 需要不同的数据。

**方案 A: 使用 Context 结构体**
```rust
pub struct DebugContext<'a> {
    pub battle_record: &'a BattleRecord,
    pub turn_order: &'a TurnOrder,
    pub units: &'a UnitsQuery<'a>,
    // ...
}
```

**方案 B: 各 viewer 自行查询 (推荐)**
```rust
// viewer 不接收数据参数，自己查询
pub fn render(ui: &mut egui::Ui) {
    // viewer 内部使用 Res<> 和 Query<> 自行获取数据
}
```

方案 B 更符合 ECS 模式，但需要将 viewer 改为系统而非纯函数。

## 实施步骤

1. 创建 `state.rs`，移入 DebugView、DebugPanelState、WorldInspectorState
2. 创建 `hotkeys.rs`，移入 debug_hotkey_system (包括 F3 overlay 逻辑)
3. 创建 `egui_setup.rs`，移入 setup_egui_font
4. 创建 `panel.rs`，移入 unified_debug_panel 和 world_inspector_ui
5. 精简 mod.rs 为 Plugin 注册入口
6. 运行 `cargo test` 验证

## 风险

- 低风险: 纯重构，不改变行为
- 需确保 pub use 导出路径不变
