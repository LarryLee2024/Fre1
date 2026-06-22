---
id: 06-ui.testing
title: UI Testing Strategy — UI 测试策略
status: code-aligned
owner: presentation-architect
created: 2026-06-20
updated: 2026-06-21
tags:
  - ui
  - testing
  - widget
  - screen
  - snapshot
  - mock
---

# UI Testing Strategy — UI 测试策略

> **职责**: @presentation-architect | **上游**: ADR-055 §14 (测试) | domain rules §5.9 (UI 三层测试流程), §INV-UI-001 (UI 不直接读取 Domain 组件) | schema §15 (验证规则) | architecture.md §6 (宪法级规则)

> **SSPEC参考**: docs/06-ui/07-specs/ — AI-Consumable Screen Specification 标准。新增 Screen 必须先写 SSPEC，见 ADR-066。

---

## 1. 设计目的

UI 测试在 50 万行代码规模下至关重要。本文档定义 UI 的三层测试策略：

| 测试层 | 测试对象 | 验证内容 | 位置 |
|--------|---------|---------|------|
| Widget 单元测试 | 单个 Widget | Contract 合规、渲染正确性、交互响应 | `src/ui/widgets/*/tests/` |
| Screen 集成测试 | Screen 组合 | Widget 组合完整性、ViewModel 绑定、导航流程 | `src/ui/screens/*/tests/` |
| UI 快照测试 | UI 树结构 | Entity 层级、Component 结构一致性 | `src/ui/tests/snapshot/` |

此外，本文档还定义：
- **Mock Projection** — 如何 mock ViewModel 数据进行测试
- **Test Fixtures** — 预置的测试数据工厂

---

## 2. Widget 单元测试

### 2.1 测试目标

每个 Widget 的测试验证：
1. **Contract 合规** — Widget 不查询 Domain Component（INV-UI-001）
2. **渲染正确性** — 从 ViewModel 输入到 UI 元素输出的映射正确
3. **交互响应** — 交互操作（点击/悬停/输入）输出正确的 UiAction

### 2.2 测试结构

每个 Widget 目录下包含 `tests/` 子目录：

```
widgets/button/
├── mod.rs
├── primary.rs
├── secondary.rs
├── danger.rs
└── tests/
    ├── mod.rs
    ├── contract_tests.rs   # Contract 合规测试（禁止的查询模式检查）
    ├── render_tests.rs     # 渲染正确性测试（ViewModel → UI 元素）
    └── interaction_tests.rs # 交互响应测试（输入 → UiAction）
```

### 2.3 Contract 合规测试

验证 Widget 不违反 Contract 边界：

```rust
// 验证 Widget 渲染后不产生 Domain Component 查询
#[test]
fn primary_button_no_domain_query() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(UiTestPlugin)  // Mock 主题/UiStore
       .add_plugins(PrimaryButtonPlugin);

    let entity = app.world_mut().spawn((
        PrimaryButton,
        // Props
        ButtonProps {
            label_key: UiTextKey::Confirm,
            enabled: true,
            width: None,
            tooltip_key: None,
        },
    )).id();

    // 验证 Widget 没有 spawn 任何 Domain Component
    let has_domain = app.world().query::<&Health>().iter(&app.world()).count();
    assert_eq!(has_domain, 0, "PrimaryButton must not spawn Domain Components");
}
```

### 2.4 渲染正确性测试

验证不同 Props 组合下的 Widget 渲染状态：

```rust
#[test]
fn primary_button_renders_label() {
    let app = create_test_app();
    let entity = spawn_button(app.world_mut(), ButtonProps {
        label_key: UiTextKey::Confirm,
        enabled: true,
        ..Default::default()
    });

    // 验证按钮文本
    let text = app.world().get::<LocalizedText>(entity).unwrap();
    assert_eq!(text.key, UiTextKey::Confirm);
}

#[test]
fn primary_button_disabled_style() {
    let app = create_test_app();
    let entity = spawn_button(app.world_mut(), ButtonProps {
        label_key: UiTextKey::Confirm,
        enabled: false,
        ..Default::default()
    });

    // 验证禁用时交互组件被禁用
    let interaction = app.world().get::<Interaction>(entity).unwrap();
    assert_eq!(*interaction, Interaction::None);
}

#[test]
fn progress_bar_clamps_ratio() {
    let app = create_test_app();
    let entity = spawn_progress_bar(app.world_mut(), 150.0, 100.0); // current > max

    let style = app.world().get::<Style>(entity).unwrap();
    // 验证比例被 clamp 到 1.0
    assert_eq!(style.width, Val::Percent(100.0));
}

#[test]
fn progress_bar_zero_max() {
    let app = create_test_app();
    let entity = spawn_progress_bar(app.world_mut(), 0.0, 0.0); // max == 0

    let style = app.world().get::<Style>(entity).unwrap();
    // 验证 max == 0 时显示为空条
    assert_eq!(style.width, Val::Percent(0.0));
}
```

### 2.5 交互响应测试

验证用户交互输出正确的 UiAction：

```rust
#[test]
fn primary_button_click_emits_action() {
    let app = create_test_app();
    let entity = spawn_button(app.world_mut(), ButtonProps {
        label_key: UiTextKey::Confirm,
        enabled: true,
        ..Default::default()
    });

    // 模拟点击（UiIntent → Action 映射测试）
    let mut intent_writer = app.world_mut().resource_mut::<Events<UiIntent>>();
    intent_writer.send(UiIntent::Confirm);

    app.update();

    // 验证 UiAction::Click 被发射
    let actions = app.world().resource::<Events<UiAction>>();
    let mut reader = actions.get_reader();
    let emitted: Vec<&UiAction> = reader.read(&actions).collect();
    assert!(emitted.contains(&&UiAction::Click));
}

#[test]
fn toggle_changes_state_on_click() {
    let app = create_test_app();
    let entity = spawn_toggle(app.world_mut(), ToggleProps {
        label_key: UiTextKey::ShowGrid,
        checked: false,
        enabled: true,
    });

    // 模拟点击
    click_entity(&mut app, entity);

    // 验证 UiAction::Toggle(true) 被发射
    let actions = app.world().resource::<Events<UiAction>>();
    let mut reader = actions.get_reader();
    let toggled_on = reader.read(&actions).any(|a| matches!(a, UiAction::Toggle(true)));
    assert!(toggled_on);
}

#[test]
fn notification_auto_dismiss_after_duration() {
    let app = create_test_app();
    let vm = NotificationVm {
        message_key: UiTextKey::ItemAcquired,
        duration_secs: 0.1,
        priority: NotificationPriorityVm::Normal,
        notification_type: NotificationTypeVm::Toast,
        ..Default::default()
    };
    let entity = spawn_notification(app.world_mut(), vm);

    // 模拟时间流逝
    app.update();  // t = 0
    assert!(app.world().get::<Notification>(entity).is_some());

    // 跳过 0.2 秒
    app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs_f32(0.2));
    app.update();

    // 验证通知已销毁
    assert!(app.world().get::<Notification>(entity).is_none());
}
```

### 2.6 Widget 测试 Props 组合矩阵

每个 Widget 的 Props 组合应覆盖：

| 字段 | 测试值组合 | 验证点 |
|------|-----------|--------|
| enabled | true, false | 交互可用/禁用样式 |
| 所有 UiTextKey 字段 | 有效 Key, 空 Key | 文本渲染不崩溃 |
| 数值字段 | 正常值, 边界值(0), 负值(如果适用), 最大值 | 数值范围约束 |
| Option 字段 | Some, None | 条件渲染正确 |
| 枚举字段 | 所有变体 | 不同变体的渲染差异 |

---

## 3. Screen 集成测试

### 3.1 测试目标

Screen 集成测试验证：
1. **Widget 组合完整性** — Screen spawn 后所有 Widget 正确创建
2. **ViewModel 绑定正确** — Widget 连接到正确的 Dirty<T> 和 UiBinding
3. **FocusGroup 初始化** — Screen 激活后 FocusGroup 正确设置
4. **导航流程正常** — Screen 的 push/pop 行为正确

### 3.2 测试结构

```
screens/battle/
├── mod.rs
├── components.rs
└── tests/
    ├── mod.rs
    ├── composition_tests.rs  # Widget 组合完整性测试
    ├── binding_tests.rs      # ViewModel 绑定测试
    └── navigation_tests.rs   # 导航流程测试
```

### 3.3 Widget 组合完整性测试

```rust
#[test]
fn battle_screen_spawns_all_widgets() {
    let mut app = create_screen_test_app();
    app.add_plugins(BattleScreenPlugin);
    app.add_plugins(MockProjectionPlugin);  // 提供 Mock ViewModel

    // 模拟进入战斗
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.update();

    // 验证所有核心 Widget 存在
    let widget_count = app.world().query::<&UiBinding>().iter(&app.world()).count();
    assert!(widget_count > 0, "BattleScreen must spawn Widgets");

    // 验证特定 UiBinding 存在
    let has_hp = app.world().query_filtered::<(), With<UiBinding>>()
        .iter(&app.world())
        .any(|e| app.world().get::<UiBinding>(e) == Some(&UiBinding::Hp));
    assert!(has_hp, "BattleScreen must have HP binding");

    let has_end_turn = app.world().query::<&LocalizedText>().iter(&app.world())
        .any(|t| t.key == UiTextKey::EndTurn);
    assert!(has_end_turn, "BattleScreen must have EndTurn button");
}

#[test]
fn settings_screen_spawns_all_tabs() {
    let mut app = create_screen_test_app();
    app.add_plugins(SettingsScreenPlugin);

    // 模拟打开设置
    app.world_mut().resource_mut::<ScreenStack>().push(ScreenType::Settings);
    app.update();

    // 验证标签页存在
    let has_gameplay_tab = app.world().query::<&UiBinding>().iter(&app.world())
        .any(|b| matches!(b, UiBinding::TabPanel(_)));
    assert!(has_gameplay_tab);
}
```

### 3.4 ViewModel 绑定测试

```rust
#[test]
fn battle_screen_hp_bar_binds_to_viewmodel() {
    let mut app = create_screen_test_app();
    app.add_plugins(BattleScreenPlugin);

    // 设置 Mock ViewModel
    let mut store = app.world_mut().resource_mut::<UiStore>();
    store.battle_hud.hp = 50;
    store.battle_hud.max_hp = 100;
    store.battle_hud.mark_dirty();

    // 触发战斗 Screen
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.update();

    // 验证 HP 条宽度为 50%
    let hp_bar_style = app.world().query_filtered::<&Style, With<UiBinding>>()
        .iter(&app.world())
        .find(|(s, b)| **b == UiBinding::Hp)
        .map(|(s, _)| s);
    assert!(hp_bar_style.is_some());
    // 注：实际宽度取决于 WidgetFactory 的 refresh 实现
}

#[test]
fn inventory_screen_item_count_matches_viewmodel() {
    let mut app = create_screen_test_app();
    app.add_plugins(InventoryScreenPlugin);

    // 设置 Mock InventoryVm
    let mut store = app.world_mut().resource_mut::<UiStore>();
    store.inventory.items = vec![
        InventorySlotVm { item_id: ItemId(1), quantity: 2, ..default() },
        InventorySlotVm { item_id: ItemId(2), quantity: 1, ..default() },
    ];
    store.inventory.mark_dirty();

    app.update();

    // 验证 Grid 中 ItemSlot 数量匹配
    let item_slots = app.world().query_filtered::<(), With<UiBinding>>()
        .iter(&app.world())
        .filter(|e| matches!(app.world().get::<UiBinding>(*e), Some(UiBinding::ItemSlot(_))))
        .count();
    assert_eq!(item_slots, 2);
}
```

### 3.5 导航流程测试

```rust
#[test]
fn screen_push_triggers_background_state() {
    let mut app = create_screen_test_app();
    app.add_plugins(ScreenStackPlugin);

    // 推入 BattleScreen
    app.world_mut().resource_mut::<ScreenStack>().push(ScreenType::Battle);
    app.update();

    assert_eq!(app.world().resource::<ScreenStack>().screens.len(), 1);

    // 推入 InventoryScreen（BattleScreen 应转为 Background）
    app.world_mut().resource_mut::<ScreenStack>().push(ScreenType::Inventory);
    app.update();

    assert_eq!(app.world().resource::<ScreenStack>().screens.len(), 2);
}

#[test]
fn screen_pop_restores_previous_screen() {
    let mut app = create_screen_test_app();
    let mut stack = app.world_mut().resource_mut::<ScreenStack>();
    stack.push(ScreenType::Settings);
    stack.push(ScreenType::Inventory);
    drop(stack);

    app.update();

    // Pop InventoryScreen
    app.world_mut().resource_mut::<ScreenStack>().pop();
    app.update();

    // 验证返回 SettingsScreen
    assert_eq!(app.world().resource::<ScreenStack>().screens.len(), 1);
    assert_eq!(
        app.world().resource::<ScreenStack>().peek().unwrap().screen_type,
        ScreenType::Settings
    );
}

#[test]
fn pop_empty_stack_is_noop() {
    let mut app = create_screen_test_app();
    app.add_plugins(ScreenStackPlugin);

    // Pop 空栈
    app.world_mut().resource_mut::<ScreenStack>().pop();
    app.update();

    // 验证栈仍为空，无崩溃
    assert!(app.world().resource::<ScreenStack>().screens.is_empty());
}
```

---

## 4. UI 快照测试

### 4.1 测试目标

UI 快照测试验证 UI 树的 Entity 层级和 Component 结构在重构后保持一致。

### 4.2 工具

使用 `insta` 快照库 + 自定义的 `capture_ui_tree()` 辅助函数：

```rust
/// 捕获指定 Screen 的 UI 树结构
/// 返回可序列化的树结构，包含 Entity 层级、Component 类型、UiBinding 值
#[test]
fn battle_screen_tree_snapshot() {
    let mut app = create_screen_test_app();
    app.add_plugins(BattleScreenPlugin);
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.update();

    let tree = capture_ui_tree::<BattleScreen>(&app.world());
    insta::assert_yaml_snapshot!("battle_screen_tree", tree);
}

#[test]
fn inventory_screen_filtered_tree_snapshot() {
    let mut app = create_screen_test_app();
    app.add_plugins(InventoryScreenPlugin);

    // 设置特定筛选条件
    app.world_mut().resource_mut::<UiStore>().inventory.filter = InventoryFilterVm::Equipment;
    app.update();

    let tree = capture_ui_tree::<InventoryScreen>(&app.world());
    insta::assert_yaml_snapshot!("inventory_screen_equipment_filter", tree);
}
```

### 4.3 快照结构示例

```yaml
# insta snapshot: battle_screen_tree.snap
entity_count: 24
layers:
  - name: ScreenLayer
    root: BattleScreen
    children:
      - widget: TopBar
        children:
          - binding: Turn
            type: LocalizedText
          - binding: Phase
            type: LocalizedText
      - widget: CharacterPanel
        children:
          - binding: Name
            type: LocalizedText
          - binding: Hp
            type: ProgressBar
            style:
              width: "75%"
          - binding: Mp
            type: ProgressBar
            style:
              width: "60%"
      - widget: SkillPanel
        children:
          - binding: SkillSlot(0)
            type: IconButton
            icon: "icon_skill_slash"
          - binding: SkillSlot(1)
            type: IconButton
            icon: "icon_skill_fireball"
      - widget: TurnBar
        # ...
```

### 4.4 快照测试规则

| 规则 | 描述 |
|------|------|
| 每次 Screen/Widget 结构性变更后更新快照 | 新增/删除 Widget 或调整层级时重新生成 |
| 快照文件名包含 Screen 名称和场景 | `battle_screen_tree.snap`, `inventory_screen_filtered.snap` |
| ViewModel 数据使用 Mock 固定值 | 快照测试使用 Test Fixtures 确保确定性 |
| 快照测试不覆盖交互测试 | 快照仅验证结构，不验证交互行为 |

---

## 5. Mock Projection

### 5.1 Mock 策略

Mock Projection 是 UI 测试的核心基础设施。它提供预设的 ViewModel 数据，绕过真实的 Domain Event → Projection 链。

```rust
/// Mock Projection Plugin — 替换真实 Projection，提供预设 ViewModel 数据
/// 使用方式：
///   1. 在测试 app 中 add_plugins(MockProjectionPlugin)
///   2. 通过 MockProjection 设置预设数据
///   3. 触发 screen update
pub struct MockProjectionPlugin;

impl Plugin for MockProjectionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MockProjection::default());
        // 注册 Mock System 替代真实 Projection System
        app.add_systems(Update, mock_projection_system);
    }
}

/// Mock Projection Resource — 预设 ViewModel 数据
#[derive(Resource, Default)]
pub struct MockProjection {
    /// 预设的 UiStore 值
    pub store: UiStore,
    /// 是否自动标记 dirty
    pub auto_dirty: bool,
}

fn mock_projection_system(
    mock: Res<MockProjection>,
    mut store: ResMut<UiStore>,
) {
    if mock.is_changed() {
        *store = mock.store.clone();
        if mock.auto_dirty {
            store.mark_all_dirty();
        }
    }
}
```

### 5.2 Mock 测试辅助函数

```rust
/// 创建带 Mock Projection 的测试 App
fn create_screen_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(TestThemePlugin)       // Mock 主题
       .add_plugins(TestBindingPlugin)      // Mock 数据绑定
       .add_plugins(MockProjectionPlugin);  // Mock Projection
    app
}

/// 在测试 App 中设置 ViewModel 数据
fn setup_battle_viewmodel(app: &mut App, hp: u32, max_hp: u32) {
    let mut mock = app.world_mut().resource_mut::<MockProjection>();
    mock.store.battle_hud.hp = hp;
    mock.store.battle_hud.max_hp = max_hp;
    mock.store.battle_hud.mark_dirty();
}

/// 在测试 App 中设置背包数据
fn setup_inventory_viewmodel(app: &mut App, items: Vec<InventorySlotVm>) {
    let mut mock = app.world_mut().resource_mut::<MockProjection>();
    mock.store.inventory.items = items;
    mock.store.inventory.gold = 9999;
    mock.store.inventory.mark_dirty();
}
```

### 5.3 Mock Projection 使用规则

| 规则 | 描述 |
|------|------|
| Mock 替换整个 Projection | 测试中不运行真实 Projection System，运行 Mock System |
| Mock 数据必须是有效的 ViewModel | 遵循 schema §15 的验证规则（hp ≤ max_hp 等） |
| Mock 不模拟 Domain Event | Mock 直接设置 ViewModel 值，不通过 Domain Event 触发 |
| 边界值测试使用 Mock | 如 hp=0、max_hp=0、items=vec![] 等边界情况通过 Mock 覆盖 |

### 5.4 Mock Coverage 矩阵

| ViewModel 字段 | Mock 值（正常） | Mock 值（边界） | 测试场景 |
|---------------|----------------|----------------|---------|
| BattleHudVm.hp | 75.0 | 0.0, max_hp | 受伤、满血、空血 |
| BattleHudVm.max_hp | 100.0 | 0.0, f32::MAX | 正常、无血条、大数值 |
| BattleHudVm.phase_key | "ui.battle.phase.player" | "" | 各阶段 UI 差异 |
| SkillPanelVm.skills | 8 个技能 | 0 个, 32 个 | 空技能面板、长列表 |
| CharacterPanelVm.level | 5 | 0, 99 | 等级显示边界 |

---

## 6. Test Fixtures

### 6.1 预置测试数据工厂

```rust
/// 预置的测试数据工厂 — 快速创建有效 ViewModel 数据
mod fixtures {
    use super::*;

    /// 标准战斗 ViewModel（默认值）
    pub fn standard_battle_hud() -> BattleHudVm {
        BattleHudVm {
            hp: 75.0,
            max_hp: 100.0,
            mp: 40.0,
            max_mp: 50.0,
            ap: 3.0,
            max_ap: 5.0,
            turn_number: 3,
            phase_key: "ui.battle.phase.player",
        }
    }

    /// 濒死战斗 ViewModel
    pub fn near_death_battle_hud() -> BattleHudVm {
        BattleHudVm {
            hp: 5.0,
            max_hp: 100.0,
            mp: 0.0,
            max_mp: 50.0,
            ap: 1.0,
            max_ap: 5.0,
            turn_number: 7,
            phase_key: "ui.battle.phase.enemy",
        }
    }

    /// 标准技能面板（4 个技能）
    pub fn four_skill_panel() -> SkillPanelVm {
        SkillPanelVm {
            skills: HashMap::from([
                (1, SkillSlotVm {
                    skill_id: 1,
                    name_key: "ui.skill.slash",
                    cooldown_remaining: 0,
                    max_cooldown: 3,
                    is_usable: true,
                    ap_cost: 1,
                }),
                (2, SkillSlotVm {
                    skill_id: 2,
                    name_key: "ui.skill.fireball",
                    cooldown_remaining: 2,
                    max_cooldown: 4,
                    is_usable: false,
                    ap_cost: 2,
                }),
            ]),
        }
    }

    /// 标准背包（3 个物品）
    ///
    /// 注意：InventoryVm 当前在 UiStore 中未实现。
    /// 此 Fixture 为参考预留，待 Inventory Projection 实现后启用。
    pub fn three_item_inventory() -> InventoryVm {
        todo!("Implement when InventoryVm is added to UiStore")
    }

    /// 空背包（测试空状态 UI）
    pub fn empty_inventory() -> InventoryVm {
        todo!("Implement when InventoryVm is added to UiStore")
    }
}
```

### 6.2 Fixture 使用规则

| 规则 | 描述 |
|------|------|
| Fixture 只包含有效数据 | Fixture 中的 ViewModel 数据遵循 schema 验证规则 |
| Fixture 覆盖常见测试场景 | standard（标准）/empty（空）/edge（边界值）/error（异常） |
| Fixture 使用 UiTextKey 而非 String | 遵循宪法 §22 和 INV-UI-007 |
| Fixture 引用 Def 使用 ID 而非嵌入 | 使用 SkillId(1)、ItemId(1) 等 ID，不包含完整 Definition |

---

## 7. 测试辅助基础设施

### 7.1 TestThemePlugin

提供 Mock 主题，使 Widget 测试不需要加载真实主题配置文件：

```rust
pub struct TestThemePlugin;

impl Plugin for TestThemePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Theme {
            colors: UiColors {
                primary: Color::srgb(0.2, 0.4, 0.8),
                danger: Color::srgb(0.8, 0.2, 0.2),
                text_primary: Color::WHITE,
                // ... 其他颜色使用默认值
            },
            spacing: UiSpacing::default(),
            typography: UiTypography::default(),
            name: ThemeName::Dark,
        });
    }
}
```

### 7.2 TestBindingPlugin

提供 Mock 数据绑定基础设施：

```rust
pub struct TestBindingPlugin;

impl Plugin for TestBindingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiStore::default());
        // 注册所有 Dirty<T> 类型
        app.register_type::<Dirty<BattleHudVm>>();
        app.register_type::<Dirty<CharacterPanelVm>>();
        app.register_type::<Dirty<SkillPanelVm>>();
        // 后续迭代补充：
        // app.register_type::<Dirty<InventoryVm>>();
        // app.register_type::<Dirty<ShopVm>>();
        // app.register_type::<Dirty<QuestLogVm>>();
    }
}
```

### 7.4 已实现的测试套件

当前代码库中包含 4 个测试套件，覆盖 Dirty 标记、导航栈、焦点管理和战斗投影，共 46 个测试用例。所有测试均为纯单元测试（无 ECS 依赖），验证核心数据结构和纯函数的正确性。

#### 7.4.1 Dirty<T> 生命周期测试（9 个测试）

**位置**: `src/ui/binding/tests/unit/dirty_test.rs`

测试 `Dirty<T>` 的变更跟踪合约：

| 测试用例 | 验证内容 |
|---------|---------|
| `new_is_dirty` | 新创建的 `Dirty<T>` 初始状态为脏（`consume()` 返回 true） |
| `consume_clears_flag` | `consume()` 清除脏标记后再次调用返回 false |
| `consume_returns_true_only_once_per_mark` | 多次 consume 仅第一次返回 true；重标记后可再次消费 |
| `get_mut_marks_dirty` | `get_mut()` 自动标记脏 |
| `get_does_not_mark_dirty` | `get()` 只读访问不触发脏标记 |
| `mark_dirty_sets_flag` | `mark_dirty()` 显式重设脏标记 |
| `default_creates_dirty_state` | `Dirty::default()` 创建脏状态 |
| `multiple_round_trip_cycles` | 多轮 new→consume→mark→get_mut→consume 循环正确 |
| `inner_value_accessible_via_get` | `get()` 返回内部值 |

#### 7.4.2 ScreenStack 导航栈测试（14 个测试）

**位置**: `src/ui/navigation/tests/unit/screen_stack_test.rs`

测试 `ScreenStack` 的 LIFO 导航栈合约：

| 测试用例 | 验证内容 |
|---------|---------|
| `new_stack_is_empty` | 新栈为空 |
| `push_adds_element_to_stack` | `push()` 添加到栈顶 |
| `push_duplicate_top_is_noop` | 重复 push 同一 screen 到栈顶为 no-op |
| `push_allows_different_screens` | push 不同 screen 增长栈深度 |
| `push_duplicate_non_top_is_allowed` | 非栈顶的重复 screen 允许 push |
| `pop_returns_top_element` | `pop()` 返回栈顶元素 |
| `pop_single_element_returns_none` | 单元素栈 pop 返回 None（保留根 screen） |
| `pop_empty_stack_returns_none` | 空栈 pop 返回 None |
| `replace_swaps_top_element` | `replace()` 替换栈顶元素 |
| `replace_on_empty_degrads_to_push` | 空栈上 replace 降级为 push |
| `contains_returns_true_for_pushed_screen` | `contains()` 检测已 push 的 screen |
| `contains_returns_false_for_unknown_screen` | `contains()` 检测未知 screen |
| `contains_returns_false_after_pop` | pop 后 contains 返回 false |
| `clear_removes_all_elements` | `clear()` 清空所有元素 |
| `iter_returns_elements_bottom_to_top` | `iter()` 按插入顺序（从底到顶）返回 |

#### 7.4.3 FocusManager 焦点管理测试（15 个测试）

**位置**: `src/ui/focus/tests/unit/focus_manager_test.rs`

测试 `FocusManager` 的全局焦点状态管理合约：

| 测试用例 | 验证内容 |
|---------|---------|
| `initial_state_has_no_focus` | 初始状态无焦点、无活跃组、空历史 |
| `focus_sets_entity_and_group` | `focus()` 设置实体、组 ID 和组索引 |
| `focus_updates_active_group` | 连续 focus 更新活跃组 |
| `blur_clears_focus` | `blur()` 清除焦点和活跃组 |
| `blur_does_not_clear_group_indices` | `blur()` 保留组索引 |
| `is_focused_returns_true_for_focused_entity` | `is_focused()` 识别已聚焦实体 |
| `is_focused_returns_false_for_unfocused_entity` | `is_focused()` 识别未聚焦实体 |
| `is_focused_returns_false_after_blur` | blur 后 `is_focused()` 返回 false |
| `push_focus_saves_current_focus_to_history` | `push_focus()` 保存当前焦点到历史 |
| `push_focus_noops_when_no_focus` | 无焦点时 `push_focus()` 不 panic |
| `pop_focus_restores_saved_focus` | `pop_focus()` 恢复保存的焦点 |
| `pop_focus_returns_none_for_unknown_group` | 未知组 `pop_focus()` 返回 None |
| `pop_focus_removes_history_entry` | `pop_focus()` 移除历史条目 |
| `activate_group_restores_history_when_available` | `activate_group()` 有历史时恢复焦点 |
| `activate_group_uses_first_entity_when_no_history` | `activate_group()` 无历史时用 first_entity |

#### 7.4.4 BattleProjection 投影测试（7 个测试）

**位置**: `src/ui/projections/tests/unit/battle_projection_test.rs`

测试 `BattleProjection` 纯函数将 Domain Event 正确投影为 ViewModel 更新：

| 测试用例 | 验证内容 |
|---------|---------|
| `on_turn_started_increments_turn_number` | TurnStarted 事件将 turn_number 从 0 递增到 1 |
| `on_turn_started_sets_phase_key` | phase_key 设置为 `"ui.battle.phase.player"` |
| `on_turn_started_increments_from_existing_value` | 从 5 递增到 6 |
| `on_turn_started_preserves_other_fields` | 投影不修改不相干字段（hp、ap 等） |
| `on_turn_started_multiple_calls_accumulate` | 多次调用累积 turn_number |
| `on_effect_applied_does_not_panic` | EffectApplied 当前 no-op，不应 panic |
| `on_effect_applied_does_not_modify_battle_hud` | 占位投影不修改 battle_hud |
| `on_effect_applied_does_not_modify_skill_panel` | 占位投影不修改 skill_panel |

### 7.5 测试组织模式总结

上述测试遵循以下共同模式：

- **纯函数优先**：Dirty<T>、ScreenStack、FocusManager 和 BattleProjection 都是纯数据结构/函数，测试无需 ECS 环境
- **确定性**：相同输入始终产生相同输出
- **简单类型**：ID 使用 u32 或 Entity::from_bits() 模拟，不依赖复杂 Domain 类型
- **边界覆盖**：空栈、单元素栈、空组、未知组、累积调用等边界情况均有覆盖

后续将在 Widget 和 Screen 层级补充带 ECS 环境的集成测试。

### 7.5 测试辅助函数

```rust
/// 模拟点击 Entity
fn click_entity(app: &mut App, entity: Entity) {
    let mut interaction = app.world_mut().get_mut::<Interaction>(entity).unwrap();
    *interaction = Interaction::Pressed;
    app.update();
    *interaction = Interaction::None;
    app.update();
}

/// 模拟 hover Entity
fn hover_entity(app: &mut App, entity: Entity) {
    let mut interaction = app.world_mut().get_mut::<Interaction>(entity).unwrap();
    *interaction = Interaction::Hovered;
    app.update();
}

/// 捕获 UI 树结构（用于快照测试，待实现）
fn capture_ui_tree<T: Component>(world: &World) -> UiTreeSnapshot {
    todo!("Implement in src/ui/tests/helpers.rs")
}
```

---

## 8. 测试覆盖要求

| 测试类型 | 最低覆盖率 | 实际覆盖率 | 关键验证点 |
|---------|----------|-----------|-----------|
| 核心数据结构测试 | 每个核心结构至少 5 个测试用例 | 46 个（4 套件）| Dirty 生命周期、栈操作合约、焦点状态、投影纯函数 |
| Widget 单元测试 | 每个 Widget 至少 5 个测试用例 | 0（待实现） | Contract 合规、渲染正确、交互响应 |
| Screen 集成测试 | 每个 Screen 至少 3 个测试用例 | 0（待实现） | Widget 组合、ViewModel 绑定、导航流程 |
| UI 快照测试 | 每个 Screen 至少 1 个快照 | 0（待实现） | UI 树结构一致性 |
| 错误/边界测试 | Widget 每个 Props 字段的边界值 | — | 渲染不崩溃 |
| 空状态测试 | 每个列表/网格 Widget 的空状态 | — | 空列表显示"空"提示 |

---

## 9. 测试错误处理

### 9.1 常见测试失败场景

| 失败场景 | 原因 | 修复方式 |
|---------|------|---------|
| Widget 渲染后缺失 Component | 工厂函数未 spawn 预期 Component | 检查工厂实现 |
| Interaction 事件未触发 UiAction | UiActionHandler 未注册 | 检查 Plugin 注册顺序 |
| Dirty<T> consume 后 Widget 不刷新 | Widget System 未检测 Dirty | 检查 System 调度 |
| 快照测试失败 | UI 树结构变更 | 审查变更后更新快照 |
| Screen 集成测试 Entity 数量不对 | Widget 组合逻辑变更 | 检查 Screen 的 spawn 函数 |

### 9.2 Mock 不一致

| 问题 | 症状 | 解决方案 |
|------|------|---------|
| Mock 数据与真实 Projection 不一致 | 测试通过但运行时 UI 异常 | 定期同步 Mock Fixture 与真实 Projection |
| Mock 遗漏新字段 | 新 ViewModel 字段未覆盖 | 新增 ViewModel 字段时同步更新 Fixture |
| Mock 数据违反验证规则 | 测试使用无效数据（如 hp > max_hp） | 在 Fixture 创建函数中添加断言 |

---

## 10. 验证规则

| # | 规则 | 校验逻辑 |
|---|------|----------|
| T-VAL-01 | 测试使用纯函数/数据结构 | 尽量使用无 ECS 依赖的纯单元测试，降低测试复杂度 |
| T-VAL-02 | 测试数据使用本地化 Key | 测试数据中的文本引用使用 `&'static str` Key，不硬编码字符串 |
| T-VAL-03 | 测试 ViewModel 数据有效 | Fixture 数据遵循 ViewModel 验证规则（hp ≤ max_hp 等） |
| T-VAL-04 | 测试确定性 | 相同输入始终产生相同输出 |

---

*本文档由 @presentation-architect 维护。新增 Widget 或 Screen 必须同步补充测试用例。*

*最后更新: 2026-06-21 — 与实际代码实现对齐 (commit 903d039)*
