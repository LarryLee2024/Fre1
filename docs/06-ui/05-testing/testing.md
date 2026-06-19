---
id: 06-ui.testing
title: UI Testing Strategy — UI 测试策略
status: draft
owner: presentation-architect
created: 2026-06-20
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
| BattleHudVm.hp | 75 | 0, max_hp | 受伤、满血、空血 |
| BattleHudVm.max_hp | 100 | 0, u32::MAX | 正常、无血条、大数值 |
| BattleHudVm.phase | PlayerTurn | None, Animation | 各阶段 UI 差异 |
| SkillPanelVm.skills | 8 个技能 | 0 个, 32 个 | 空技能面板、长列表滚动 |
| InventoryVm.items | 12 个物品 | 0 个, 99 个 | 空背包、大量物品翻页 |
| NotificationVm.priority | Normal | Critical | 优先级颜色差异 |

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
            hp: 75,
            max_hp: 100,
            mp: 40,
            max_mp: 50,
            current_turn: 3,
            active_character: Some(CharacterId(1)),
            phase: BattlePhaseVm::PlayerTurn,
            action_points: 3,
            max_action_points: 5,
            ..Default::default()
        }
    }

    /// 濒死战斗 ViewModel
    pub fn near_death_battle_hud() -> BattleHudVm {
        BattleHudVm {
            hp: 5,
            max_hp: 100,
            mp: 0,
            max_mp: 50,
            current_turn: 7,
            action_points: 1,
            max_action_points: 5,
            phase: BattlePhaseVm::EnemyTurn,
            ..Default::default()
        }
    }

    /// 标准技能面板（4 个技能）
    pub fn four_skill_panel() -> SkillPanelVm {
        SkillPanelVm {
            skills: vec![
                SkillSlotVm {
                    skill_id: SkillId(1),
                    name_key: UiTextKey::SkillSlash,
                    icon_key: AssetKey("icon_skill_slash".into()),
                    cooldown_remaining: 0.0,
                    is_usable: true,
                    mp_cost: 5,
                    ap_cost: 1,
                    ..Default::default()
                },
                SkillSlotVm {
                    skill_id: SkillId(2),
                    name_key: UiTextKey::SkillFireball,
                    icon_key: AssetKey("icon_skill_fireball".into()),
                    cooldown_remaining: 2.0,
                    is_usable: false,
                    mp_cost: 15,
                    ap_cost: 2,
                    ..Default::default()
                },
                // Skill 3, 4...
            ],
            selected: None,
            ap_remaining: 3,
            max_ap: 5,
        }
    }

    /// 标准背包（3 个物品）
    pub fn three_item_inventory() -> InventoryVm {
        InventoryVm {
            items: vec![
                InventorySlotVm {
                    item_id: ItemId(1),
                    name_key: UiTextKey::ItemPotion,
                    icon_key: AssetKey("icon_potion".into()),
                    quantity: 5,
                    rarity: RarityVm::Common,
                    is_equipped: false,
                },
                InventorySlotVm {
                    item_id: ItemId(2),
                    name_key: UiTextKey::ItemSword,
                    icon_key: AssetKey("icon_sword".into()),
                    quantity: 1,
                    rarity: RarityVm::Rare,
                    is_equipped: true,
                },
                InventorySlotVm {
                    item_id: ItemId(3),
                    name_key: UiTextKey::ItemRing,
                    icon_key: AssetKey("icon_ring".into()),
                    quantity: 1,
                    rarity: RarityVm::Epic,
                    is_equipped: false,
                },
            ],
            gold: 9999,
            filter: InventoryFilterVm::All,
            selected: None,
            sort_order: InventorySortOrder::ByType,
        }
    }

    /// 空背包（测试空状态 UI）
    pub fn empty_inventory() -> InventoryVm {
        InventoryVm {
            items: vec![],
            gold: 0,
            filter: InventoryFilterVm::All,
            selected: None,
            sort_order: InventorySortOrder::ByType,
        }
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
        app.register_type::<Dirty<InventoryVm>>();
        app.register_type::<Dirty<ShopVm>>();
        app.register_type::<Dirty<QuestLogVm>>();
    }
}
```

### 7.3 测试辅助函数

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

/// 创建带有 N 个技能槽的 Mock SkillPanelVm
fn mock_skill_panel_with_n_skills(n: usize) -> SkillPanelVm {
    SkillPanelVm {
        skills: (0..n).map(|i| SkillSlotVm {
            skill_id: SkillId(i as u32),
            name_key: UiTextKey::Custom(format!("ui.test.skill_{}", i)),
            icon_key: AssetKey(format!("icon_skill_{}", i)),
            cooldown_remaining: 0.0,
            is_usable: i % 2 == 0,  // 隔一个可用
            mp_cost: 10,
            ap_cost: 1,
        }).collect(),
        selected: None,
        ap_remaining: 5,
        max_ap: 5,
    }
}

/// 捕获 UI 树结构（用于快照测试）
fn capture_ui_tree<T: Component>(world: &World) -> UiTreeSnapshot {
    // 实现：遍历 Entity 层级，收集 Component 类型和 UiBinding 值
    // 返回可序列化的 UiTreeSnapshot 结构
    todo!("Implement in src/ui/tests/helpers.rs")
}
```

---

## 8. 测试覆盖要求

| 测试类型 | 最低覆盖率 | 关键验证点 |
|---------|----------|-----------|
| Widget 单元测试 | 每个 Widget 至少 5 个测试用例 | Contract 合规、渲染正确、交互响应 |
| Screen 集成测试 | 每个 Screen 至少 3 个测试用例 | Widget 组合、ViewModel 绑定、导航流程 |
| UI 快照测试 | 每个 Screen 至少 1 个快照 | UI 树结构一致性 |
| 错误/边界测试 | Widget 每个 Props 字段的边界值 | 渲染不崩溃 |
| 空状态测试 | 每个列表/网格 Widget 的空状态 | 空列表显示"空"提示 |

---

## 9. 测试错误处理

### 9.1 常见测试失败场景

| 失败场景 | 原因 | 修复方式 |
|---------|------|---------|
| Widget 渲染后缺失 Component | WidgetFactory::create 未 spawn 预期 Component | 检查 create 实现 |
| Interaction 事件未触发 UiAction | UiActionHandler 未注册 | 检查 Plugin 注册顺序 |
| Dirty<T> consume 后 Widget 不刷新 | Binding System 未运行 | 检查 System 调度 |
| 快照测试失败 | UI 树结构变更 | 审查变更后更新 insta 快照 |
| Screen 集成测试 Entity 数量不对 | Widget 组合逻辑变更 | 检查 Screen 的 bsn! 组合 |

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
| T-VAL-01 | 测试不模拟 Domain 行为 | 测试使用 Mock Projection，不运行真实 Domain System |
| T-VAL-02 | 测试使用 UiTextKey | 测试数据中的文本引用使用 UiTextKey，不硬编码字符串 |
| T-VAL-03 | 测试 ViewModel 数据有效 | Fixture 数据遵循 ViewModel 验证规则（hp ≤ max_hp 等） |
| T-VAL-04 | 快照测试确定性 | 快照测试使用固定 Mock 数据，不依赖随机值 |

---

*本文档由 @presentation-architect 维护。新增 Widget 或 Screen 必须同步补充测试用例。*
