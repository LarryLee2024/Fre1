# UI-Domain Boundary Rules — UI 与领域层交互边界

Version: 1.0
Status: Proposed

来源：`docs/其他/31遗漏.md` 第四节 — UI-领域交互边界

本文档定义 UI 层与领域层（Core）之间的严格分离契约，防止 UI 代码污染业务逻辑。

交叉引用：
- `docs/domain/ui_architecture_rules.md` — UI 架构领域完整规则
- `docs/architecture.md` — UI 架构总纲（Logic/Presentation 分离）
- `docs/architecture/layer-contracts.md` — 层间依赖规则

---

## 1. 核心原则

### 1.1 单向数据流

```
用户输入 → UI 系统 → UiCommand Message → CommandHandler → Core 系统 → ViewModel → UI 重绘
```

**数据只沿一个方向流动**：
- Core → UI：通过 ViewModel（只读 Resource）
- UI → Core：通过 UiCommand Message（单向意图传递）

### 1.2 三大不变量

1. **UI 只读**：UI 层只能读取 ViewModel Resource，不能直接读取 ECS 组件
2. **UI 只写 Message**：UI 层只能通过 UiCommand Message 表达意图，不能直接修改 ECS
3. **Core 无 UI 知觉**：Core 层不知道 UI 的存在，不引用任何 UI 类型

---

## 2. 通信图

### 2.1 完整通信流程

```
┌─────────────────────────────────────────────────────────┐
│  UI 层                                                   │
│                                                          │
│  UserInput → UI System → UiCommand Message               │
│                                                          │
│  ViewModel Resource → UI System → Panel/Widget 渲染       │
│                                                          │
└─────────────────────┬───────────────────────┬────────────┘
                      │                       │
                      ▼                       ▲
┌─────────────────────┴───────────────────────┴────────────┐
│  Core 层                                                  │
│                                                          │
│  UiCommand → CommandHandler → Core System                │
│                                                          │
│  Core System → ViewModel Resource (更新)                  │
│                                                          │
│  Core System → DomainEvent Message (广播)                 │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### 2.2 箭头含义

| 箭头 | 类型 | 说明 |
|------|------|------|
| UserInput → UI System | 用户事件 | 鼠标点击、键盘输入 |
| UI System → UiCommand | Message | 用户操作意图 |
| UiCommand → CommandHandler | Message | 命令分发 |
| CommandHandler → Core System | 函数调用 | 执行游戏逻辑 |
| Core System → ViewModel | Resource 写入 | 填充显示数据 |
| ViewModel → UI System | Resource 只读 | 读取显示数据 |
| Core System → DomainEvent | Message | 广播游戏事件 |
| DomainEvent → UI System | Message | UI 响应事件 |

---

## 3. UI 层只读规则

### 3.1 只读 ViewModel

UI 层只能通过 ViewModel Resource 获取显示数据。

**允许**：
```rust
fn update_unit_panel(
    view: Res<SelectedUnitView>,
    mut panel: Query<&mut Text>,
) {
    // ✅ 正确：从 ViewModel 读取数据
    for mut text in &mut panel {
        text.sections[0].value = view.name.clone();
    }
}
```

**禁止**：
```rust
fn update_unit_panel_bad(
    query: Query<(&UnitName, &Attributes)>,  // ❌ 直接查询 ECS 组件
    mut panel: Query<&mut Text>,
) {
    for (name, attrs) in &query {
        // ❌ UI 直接访问 Core 组件
    }
}
```

### 3.2 只读 DomainEvent

UI 层只能通过 MessageReader 消费 Core 广播的事件。

**允许**：
```rust
fn handle_damage_event(
    mut events: MessageReader<DamageApplied>,
    mut floating_text: Query<&mut FloatingText>,
) {
    // ✅ 正确：从 Message 读取事件
    for event in events.read() {
        spawn_floating_text(event.target, event.damage);
    }
}
```

### 3.3 禁止的读取方式

- 🟥 `Query<(&Attributes, ...)>` 直接查询 Core 组件
- 🟥 `Res<Attributes>` 直接读取 Core Resource
- 🟥 `World::entity()` 直接访问 Entity
- 🟥 `Res<SkillRegistry>` 等直接读取 Core Registry

---

## 4. UI 层写入规则

### 4.1 只写 UiCommand

UI 层只能通过发送 UiCommand Message 表达用户操作意图。

**允许**：
```rust
fn handle_attack_button(
    interaction: Query<&Interaction, Changed<Interaction>>,
    mut commands: Commands,
) {
    for interaction in &interaction {
        if *interaction == Interaction::Pressed {
            // ✅ 正确：发送 UiCommand Message
            commands.write_message(UiCommand::Attack);
        }
    }
}
```

**禁止**：
```rust
fn handle_attack_button_bad(
    interaction: Query<&Interaction, Changed<Interaction>>,
    mut commands: Commands,
    mut query: Query<&mut CombatIntent>,  // ❌ 直接修改 ECS 组件
) {
    for interaction in &interaction {
        if *interaction == Interaction::Pressed {
            // ❌ UI 直接修改游戏状态
            if let Ok(mut intent) = query.get_single_mut() {
                *intent = CombatIntent::Attack;
            }
        }
    }
}
```

### 4.2 禁止的写入方式

- 🟥 `commands.entity(e).insert(Selected)` 直接修改组件
- 🟥 `commands.entity(e).remove::<Dead>()` 直接移除组件
- 🟥 `next_state.set(AppState::GameOver)` 直接设置状态
- 🟥 直接调用 Core 系统函数

### 4.3 CommandHandler 是唯一的写入入口

所有用户操作必须经过 `command_handler.rs` 中的 `handle_ui_commands` 系统处理。

```rust
// command_handler.rs（Core 层）
fn handle_ui_commands(
    mut commands: Commands,
    mut reader: MessageReader<UiCommand>,
    // ... Core 层 Resource 和 Query ...
) {
    for cmd in reader.read() {
        match cmd {
            UiCommand::Attack => {
                // Core 层执行攻击逻辑
                // 不包含任何 UI 代码
            }
            UiCommand::MoveUnit(target) => {
                // Core 层执行移动逻辑
            }
            // ...
        }
    }
}
```

---

## 5. UI 本地状态分离

### 5.1 UI State 定义

UI 本地状态（面板开关、选中项、悬停实体等）只存在于 UI 层。

```rust
// src/ui/state.rs（UI 层）
#[derive(Resource, Default)]
pub struct UiLocalState {
    pub selected_panel: Option<PanelType>,
    pub is_unit_panel_open: bool,
    pub hovered_grid: Option<IVec2>,
}

#[derive(Resource, Default)]
pub struct HoveredEntity {
    pub entity: Option<Entity>,
}

#[derive(Resource, Default)]
pub struct UiFocusState {
    pub blocks_input: bool,
}
```

### 5.2 UI State 规则

- 🟩 UI State 只被 UI 层系统读写
- 🟥 Core 层不读写 UI State
- 🟥 UI State 丢失不影响游戏逻辑
- 🟩 UI State 默认值对游戏无副作用

### 5.3 BlocksGameInput

模态面板必须标记 `BlocksGameInput`，Input 系统读取此标记决定是否跳过游戏操作。

```rust
// 模态面板标记
#[derive(Component)]
pub struct BlocksGameInput;

// Input 系统检查
fn should_process_input(focus: Res<UiFocusState>) -> bool {
    !focus.blocks_input
}
```

---

## 6. Core 层无 UI 知觉

### 6.1 禁止引用

Core 层的 `use` 语句中不出现 `crate::ui::` 路径。

**禁止**：
```rust
// core/battle/attack_system.rs
use crate::ui::UiTheme;           // ❌ Core 引用 UI 类型
use crate::ui::SelectedUnitView;   // ❌ Core 引用 ViewModel
use crate::ui::FloatingText;       // ❌ Core 引用 UI 组件
```

### 6.2 允许的通信方式

Core 层通过以下方式间接与 UI 通信：

- 🟩 通过 ViewModel Resource 填充显示数据
- 🟩 通过 DomainEvent Message 广播游戏事件
- 🟩 通过 ECS Component 变更触发 UI 响应

### 6.3 服务器模式兼容

Core 层不依赖 UI 的设计使得游戏可以在无 UI 的服务器模式下运行：

```rust
// 服务器模式：只加载 Core + Infrastructure，不加载 UI
fn run_server() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(CorePlugin)        // ✅ Core 不依赖 UI
        .add_plugins(InfrastructurePlugin)
        // .add_plugins(UiPlugin);      // 不加载 UI
}
```

---

## 7. ViewModel 更新规则

### 7.1 更新系统

每种 ViewModel 有对应的 `update_*_view` 系统，在 `Update` 阶段运行。

```rust
fn update_selected_unit_view(
    hovered: Res<HoveredEntity>,
    query: Query<(&UnitName, &Attributes, &ActiveBuffs), Changed<Attributes>>,
    mut view: ResMut<SelectedUnitView>,
) {
    if let Some(entity) = hovered.entity {
        if let Ok((name, attrs, buffs)) = query.get(entity) {
            view.name = name.to_string();
            view.hp = attrs.current_hp;
            view.max_hp = attrs.max_hp;
            // ... 填充其他字段
        }
    }
}
```

### 7.2 更新规则

- 🟩 ViewModel 是 Bevy Resource（`#[derive(Resource)]`），不是 Component
- 🟩 ViewModel 只包含扁平化的渲染数据，不包含 Entity 引用
- 🟩 ViewModel 更新使用 `Changed<T>` 过滤器优化
- 🟥 ViewModel 不包含游戏逻辑计算
- 🟥 ViewModel 不在 Core 模块中定义

### 7.3 ViewModel 不包含 Entity

🟥 **ViewModel 禁止包含 Entity 引用**。

```rust
// ❌ 错误：ViewModel 包含 Entity
pub struct SelectedUnitView {
    pub entity: Entity,  // ❌ 包含 Entity
    pub name: String,
    pub hp: i32,
}

// ✅ 正确：ViewModel 只包含渲染数据
pub struct SelectedUnitView {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    // ... 其他扁平化数据
}
```

---

## 8. 主题管理

### 8.1 UiTheme 统一管理

所有 UI 样式常量集中在 `UiTheme` Resource 中。

```rust
#[derive(Resource)]
pub struct UiTheme {
    pub colors: ColorPalette,
    pub sizes: SizePalette,
    pub fonts: FontPalette,
}
```

### 8.2 禁止硬编码

- 🟥 UI 面板代码中禁止硬编码颜色值
- 🟥 UI 面板代码中禁止硬编码字号值
- 🟥 UI 面板代码中禁止硬编码间距值

**允许**：
```rust
fn create_button(theme: Res<UiTheme>) -> ButtonBundle {
    ButtonBundle::default()
        .with_background_color(theme.colors.primary)
        .with_style(Style {
            padding: UiRect::all(theme.sizes.padding_medium),
            ..default()
        })
}
```

---

## 9. 禁止事项

- 🟥 **UI 直接查询 ECS 组件**（必须通过 ViewModel）
- 🟥 **UI 直接修改 ECS 组件**（必须通过 UiCommand）
- 🟥 **Core 引用 UI 类型**（Core 不知道 UI 存在）
- 🟥 **ViewModel 包含 Entity 引用**（ViewModel 是只读快照）
- 🟥 **UI State 丢失影响游戏逻辑**（UI State 是纯表现层状态）
- 🟥 **Notification 中包含 UI 特定数据**（Core 不应知道 UI 实现细节）
- 🟥 **UI 面板硬编码颜色/字号/间距**（必须通过 UiTheme）
- 🟥 **command_handler 中执行 UI 渲染逻辑**（command_handler 是 Core 行为）
- 🟥 **handle_ui_commands 在非玩家回合执行**（只有玩家回合响应 UI 操作）
- 🟥 **UI 系统绕过 UiCommand 直接设置 NextState**（必须通过 CommandHandler）

---

## 10. 违反检测

### 10.1 编译期检测

- 🟩 使用 Rust 模块可见性确保 Core 不能 `use` UI 类型
- 🟩 UI 模块的 `pub` 接口只暴露 ViewModel 和 UiCommand

### 10.2 运行期检测

- 🟩 Debug 构建中检测 UI 系统是否直接查询了 Core 组件
- 🟩 调试面板中显示所有 UiCommand 的发送和消费记录

### 10.3 架构审查检查表

- [ ] UI 系统的 `use` 语句是否只引用 UI 内部类型和 ViewModel？
- [ ] UI 系统是否通过 UiCommand 传递用户操作？
- [ ] Core 系统的 `use` 语句是否不包含 `crate::ui::`？
- [ ] ViewModel 是否只包含扁平化渲染数据？
- [ ] ViewModel 是否不包含 Entity 引用？
- [ ] 模态面板是否标记了 `BlocksGameInput`？

---

## 11. 与其他文档的关系

| 文档 | 关系 |
|------|------|
| `ui_architecture_rules.md` | 本文档是 UI 架构的边界规则补充 |
| `architecture.md` | 本文档是"UI 架构"章节的详细补充 |
| `layer-contracts.md` | UI 层的依赖规则在层间契约中定义 |
| `ecs_communication_rules.md` | Message 和 Observer 通信规则 |
| `validation_rules.md` | UI 数据一致性校验 |
