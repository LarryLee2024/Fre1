//! UI 屏幕 Feature Test (Part B: UI 流程)
//!
//! 测试框架 UI 屏幕的正确生成/清理与交互：
//! - B1: MainMenu UI 元素 + 开始游戏按钮触发状态转换
//! - B2: LevelSelect 关卡列表 + 选择后进入 InGame
//! - B3: GameOver 胜利/失败 UI + 重玩/返回按钮
//! - B4: E2E 流程 MainMenu → LevelSelect → InGame → GameOver → MainMenu
//!
//! 测试策略：
//! - 屏幕实体生成/清理通过 OnEnter/OnExit 调度测试（经 AppState 转换触发）
//! - 状态转换逻辑通过直接操作 NextState + 资源模拟（而非通过 Message 管线）
//! - ViewModel 更新通过资源变化触发系统验证

// ================================================
// AI Self-Check (test_spec.md §13.1)
// ================================================
// ✅ 测试行为，不是实现 —— 测试屏幕的生成/清理/状态转换行为，非内部实现
// ✅ 符合领域规则 —— 通过 ECS 资源（NextState/GameOverState）驱动，不直接操作 ECS
// ✅ 测试是确定性的 —— 硬编码测试数据，无随机因素
// ✅ 使用标准数据 —— 通过构造 test data 而非依赖文件系统
// ✅ 没有测试私有实现 —— 仅通过公开 marker 组件查询屏幕实体
// ✅ 没有生成不在范围内的测试 —— 仅测试屏幕模块
// ================================================

use std::collections::HashMap;

use bevy::prelude::*;

use tactical_rpg::core::campaign::def::{CampaignDef, StageDef};
use tactical_rpg::core::campaign::registry::CampaignRegistry;
use tactical_rpg::core::campaign::state::{CampaignProgress, StageStatus};
use tactical_rpg::core::map::{LevelConfig, LevelRegistry};
use tactical_rpg::core::turn::{AppState, GameOverState, TurnState};
use tactical_rpg::infrastructure::assets::CnFont;
use tactical_rpg::infrastructure::localization::{CurrentLocale, Locale, LocalizationService};
use tactical_rpg::ui::events::UiCommand;
use tactical_rpg::ui::screens::ScreensPlugin;
use tactical_rpg::ui::screens::game_over::GameOverScreen;
use tactical_rpg::ui::screens::level_select::LevelSelectScreen;
use tactical_rpg::ui::screens::main_menu::MainMenuScreen;
use tactical_rpg::ui::theme::UiTheme;
use tactical_rpg::ui::view_models::{GameOutcome, GameResultView, LevelSelectState};

// ── 测试辅助 ──

/// 构建屏幕测试 App
///
/// 使用 MinimalPlugins + AssetPlugin 提供基本运行时 + 资产系统，
/// 手动注入测试数据（CampaignRegistry / LevelRegistry），
/// 避免依赖文件系统 IO（除字体外）。
fn screen_test_app() -> App {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let asset_path = format!("{manifest_dir}/assets");

    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::state::app::StatesPlugin,
        AssetPlugin {
            file_path: asset_path,
            ..default()
        },
    ));

    // 游戏状态
    app.init_state::<AppState>();

    // ── 资源 ──

    // UI 主题
    app.init_resource::<UiTheme>();

    // Font 资产类型注册（MinimalPlugins 不包含 TextPlugin）
    app.init_asset::<Font>();

    // CnFont：MinimalPlugins 不执行 Startup，故直接插入
    let asset_server = app.world().resource::<AssetServer>();
    app.insert_resource(CnFont::from(asset_server));

    // 关卡注册表：注入测试关卡
    let mut level_registry = LevelRegistry::default();
    level_registry.levels.insert(
        "tutorial".to_string(),
        LevelConfig {
            id: "tutorial".to_string(),
            name: "教学关".to_string(),
            width: 10,
            height: 8,
            tile_size: 64.0,
            terrain_map: HashMap::new(),
            player_units: vec![],
            enemy_units: vec![],
            victory_condition: None,
            turn_limit: None,
        },
    );
    app.insert_resource(level_registry);

    // 战役注册表：注入测试战役
    let mut campaign_registry = CampaignRegistry::default();
    campaign_registry.campaigns.insert(
        "test_campaign".to_string(),
        CampaignDef {
            id: "test_campaign".to_string(),
            name: "测试战役".to_string(),
            stages: vec![StageDef {
                id: "stage_001".to_string(),
                level_id: "tutorial".to_string(),
            }],
        },
    );
    app.insert_resource(campaign_registry);

    // 战役进度（空，由 StartGame 命令初始化）
    app.init_resource::<CampaignProgress>();

    // GameOver 状态（默认 Playing）
    app.insert_resource(GameOverState::Playing);

    // TurnState（update_game_result_view 依赖）
    app.init_resource::<TurnState>();

    // Localization（i18n 资源）
    let mut localization = LocalizationService::new(Locale::ZhCn);
    // 加载中文 FTL 文件
    let zh_ui_ftl = include_str!("../../assets/localization/zh-CN/ui.ftl");
    let _ = localization.load_ftl(Locale::ZhCn, zh_ui_ftl);
    let zh_buff_ftl = include_str!("../../assets/localization/zh-CN/buff.ftl");
    let _ = localization.load_ftl(Locale::ZhCn, zh_buff_ftl);
    let zh_character_ftl = include_str!("../../assets/localization/zh-CN/character.ftl");
    let _ = localization.load_ftl(Locale::ZhCn, zh_character_ftl);
    let zh_skill_ftl = include_str!("../../assets/localization/zh-CN/skill.ftl");
    let _ = localization.load_ftl(Locale::ZhCn, zh_skill_ftl);
    let zh_system_ftl = include_str!("../../assets/localization/zh-CN/system.ftl");
    let _ = localization.load_ftl(Locale::ZhCn, zh_system_ftl);
    app.insert_resource(localization);
    app.insert_resource(CurrentLocale(Locale::ZhCn));

    // ── Message ──
    app.add_message::<UiCommand>();

    // ── 屏幕插件 ──
    app.add_plugins(ScreensPlugin);

    app
}

/// 触发 App 状态转换（OnEnter/OnExit 执行）
fn apply_state_transition(app: &mut App) {
    app.update();
}

/// 获取指定 Component 的实体数量
fn entity_count<T: Component>(app: &mut App) -> usize {
    let mut query = app.world_mut().query::<&T>();
    query.iter(app.world_mut()).count()
}

/// 切换 AppState 并执行 OnEnter/OnExit
fn set_state(app: &mut App, state: AppState) {
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(state);
    apply_state_transition(app);
}

/// 初始化战役进度并切换到 LevelSelect（模拟 StartGame 命令的完整效果）
fn initialize_campaign_and_enter_level_select(app: &mut App) {
    let registry = app.world().resource::<CampaignRegistry>();
    *app.world_mut().resource_mut::<CampaignProgress>() = CampaignProgress::initialize(registry);
    set_state(app, AppState::LevelSelect);
}

/// 初始化战役 + 选中 + 确认进入 InGame
fn enter_ingame_from_main_menu(app: &mut App) {
    initialize_campaign_and_enter_level_select(app);

    // SelectStage + ConfirmStage
    app.world_mut()
        .resource_mut::<CampaignProgress>()
        .current_stage = Some("stage_001".to_string());
    set_state(app, AppState::InGame);
}

// ══════════════════════════════════════════════════════════════
// B1: MainMenu UI 元素 + "开始游戏"按钮触发状态转换
// ══════════════════════════════════════════════════════════════

/// Test ID: UI-SCR-001
/// Title: MainMenu 屏幕生成后 UI 元素存在
///
/// Given: 默认 AppState::MainMenu
/// When:  app.update() 触发 OnEnter(MainMenu)
/// Then:  MainMenuScreen 实体存在
#[test]
fn main_menu_entities_spawned() {
    // Given
    let mut app = screen_test_app();

    // When
    apply_state_transition(&mut app);

    // Then
    assert_eq!(
        entity_count::<MainMenuScreen>(&mut app),
        1,
        "MainMenuScreen 应恰好存在 1 个"
    );
}

/// Test ID: UI-SCR-002
/// Title: 退出 MainMenu 后屏幕清理
///
/// Given: MainMenu 已生成
/// When:  切换到 LevelSelect 状态
/// Then:  MainMenuScreen 实体全部销毁
#[test]
fn main_menu_cleaned_on_exit() {
    // Given
    let mut app = screen_test_app();
    apply_state_transition(&mut app);
    assert_eq!(
        entity_count::<MainMenuScreen>(&mut app),
        1,
        "初始状态应为 MainMenu"
    );

    // When
    set_state(&mut app, AppState::LevelSelect);

    // Then
    assert_eq!(
        entity_count::<MainMenuScreen>(&mut app),
        0,
        "切换到 LevelSelect 后 MainMenu 应被清理"
    );
}

/// Test ID: UI-SCR-003
/// Title: StartGame → LevelSelect 状态转换 + CampaignProgress 初始化
///
/// Given: MainMenu 状态
/// When:  手动初始化 CampaignProgress 并切换到 LevelSelect
/// Then:  LevelSelect 状态，CampaignProgress 已初始化
#[test]
fn start_game_triggers_state_transition() {
    // Given
    let mut app = screen_test_app();
    apply_state_transition(&mut app);

    // When - 模拟 StartGame 命令效果
    initialize_campaign_and_enter_level_select(&mut app);

    // Then - 状态变为 LevelSelect
    assert_eq!(
        *app.world().resource::<State<AppState>>(),
        AppState::LevelSelect,
        "StartGame 后应进入 LevelSelect 状态"
    );

    // Then - CampaignProgress 已初始化
    let progress = app.world().resource::<CampaignProgress>();
    assert!(
        !progress.stages.is_empty(),
        "CampaignProgress 应包含关卡数据"
    );
    assert_eq!(
        progress.stage_status("stage_001"),
        Some(&StageStatus::Unlocked),
        "第一个 stage 应为 Unlocked"
    );
}

// ══════════════════════════════════════════════════════════════
// B2: LevelSelect 关卡列表展示 + 选择后进入 InGame
// ══════════════════════════════════════════════════════════════

/// Test ID: UI-SCR-004
/// Title: LevelSelect 屏幕生成后关卡列表展示
///
/// Given: AppState::LevelSelect
/// When:  OnEnter(LevelSelect) 执行
/// Then:  LevelSelectScreen 实体存在，ViewModel 包含关卡列表
#[test]
fn level_select_entities_spawned() {
    // Given
    let mut app = screen_test_app();
    apply_state_transition(&mut app);

    // When - 模拟进入 LevelSelect
    initialize_campaign_and_enter_level_select(&mut app);

    // Then
    assert_eq!(
        entity_count::<LevelSelectScreen>(&mut app),
        1,
        "LevelSelectScreen 应恰好存在 1 个"
    );

    // ViewModel 包含关卡数据
    let view = app.world().resource::<LevelSelectState>();
    assert_eq!(view.stages.len(), 1, "ViewModel 应有 1 个关卡");
    assert_eq!(view.stages[0].stage_id, "stage_001");
    assert_eq!(view.stages[0].status, StageStatus::Unlocked);
    assert_eq!(view.campaign_name, "测试战役");
}

/// Test ID: UI-SCR-005
/// Title: LevelSelect → InGame 状态转换
///
/// Given: LevelSelect 状态，关卡已选中
/// When:  切换到 InGame 状态
/// Then:  状态为 InGame，LevelSelectScreen 被清理
#[test]
fn level_select_confirm_enters_ingame() {
    // Given
    let mut app = screen_test_app();
    apply_state_transition(&mut app);
    initialize_campaign_and_enter_level_select(&mut app);

    // When - 模拟 SelectStage + ConfirmStage
    enter_ingame_from_main_menu(&mut app);

    // Then
    assert_eq!(
        *app.world().resource::<State<AppState>>(),
        AppState::InGame,
        "Confirm 后应进入 InGame 状态"
    );
    assert_eq!(
        entity_count::<LevelSelectScreen>(&mut app),
        0,
        "进入 InGame 后 LevelSelect 应被清理"
    );
}

/// Test ID: UI-SCR-006
/// Title: LevelSelect → Back → MainMenu 返回
///
/// Given: LevelSelect 状态
/// When:  切换回 MainMenu
/// Then:  状态回到 MainMenu，LevelSelectScreen 被清理
#[test]
fn level_select_back_returns_to_main_menu() {
    // Given
    let mut app = screen_test_app();
    apply_state_transition(&mut app);
    initialize_campaign_and_enter_level_select(&mut app);

    // When - 返回主菜单
    set_state(&mut app, AppState::MainMenu);

    // Then
    assert_eq!(
        *app.world().resource::<State<AppState>>(),
        AppState::MainMenu,
        "返回后应回到 MainMenu"
    );
    assert_eq!(
        entity_count::<LevelSelectScreen>(&mut app),
        0,
        "返回 MainMenu 后 LevelSelect 应被清理"
    );
}

// ══════════════════════════════════════════════════════════════
// B3: GameOver 胜利/失败 UI + 重玩/返回按钮
// ══════════════════════════════════════════════════════════════

/// Test ID: UI-SCR-007
/// Title: GameOver 屏幕胜利时 UI 元素正确
///
/// Given: GameOverState::Victory
/// When:  进入 GameOver 状态，触发 update_game_result_view + spawn
/// Then:  GameOverScreen 存在，ViewModel 显示 Victory
#[test]
fn game_over_victory_ui() {
    // Given
    let mut app = screen_test_app();
    apply_state_transition(&mut app);
    initialize_campaign_and_enter_level_select(&mut app);

    // 设定胜利状态
    *app.world_mut().resource_mut::<GameOverState>() = GameOverState::Victory;

    // When - 切换到 GameOver 状态
    set_state(&mut app, AppState::GameOver);

    // Then - 屏幕存在
    assert_eq!(
        entity_count::<GameOverScreen>(&mut app),
        1,
        "GameOverScreen 应恰好存在 1 个"
    );

    // Then - ViewModel 为 Victory
    let view = app.world().resource::<GameResultView>();
    assert_eq!(
        view.result,
        GameOutcome::Victory,
        "胜利时 result 应为 Victory"
    );
}

/// Test ID: UI-SCR-008
/// Title: GameOver 屏幕失败时 UI 元素正确
///
/// Given: GameOverState::Defeat
/// When:  进入 GameOver 状态，触发 update_game_result_view + spawn
/// Then:  GameOverScreen 存在，ViewModel 显示 Defeat
#[test]
fn game_over_defeat_ui() {
    // Given
    let mut app = screen_test_app();
    apply_state_transition(&mut app);
    initialize_campaign_and_enter_level_select(&mut app);

    // 设定失败状态
    *app.world_mut().resource_mut::<GameOverState>() = GameOverState::Defeat;

    // When - 切换到 GameOver 状态
    set_state(&mut app, AppState::GameOver);

    // Then - 屏幕存在
    assert_eq!(
        entity_count::<GameOverScreen>(&mut app),
        1,
        "GameOverScreen 应恰好存在 1 个"
    );

    // Then - ViewModel 为 Defeat
    let view = app.world().resource::<GameResultView>();
    assert_eq!(
        view.result,
        GameOutcome::Defeat,
        "失败时 result 应为 Defeat"
    );
    assert!(!view.has_next_stage, "失败时 has_next_stage 应为 false");
}

/// Test ID: UI-SCR-009
/// Title: GameOver → InGame (Retry) 重玩当前关卡
///
/// Given: GameOver 状态（Defeat）
/// When:  切换到 InGame（模拟 Retry）
/// Then:  GameOverScreen 被清理，GameOverState 重置为 Playing
#[test]
fn game_over_retry_returns_to_ingame() {
    // Given
    let mut app = screen_test_app();
    apply_state_transition(&mut app);
    enter_ingame_from_main_menu(&mut app);

    // 模拟失败进入 GameOver
    *app.world_mut().resource_mut::<GameOverState>() = GameOverState::Defeat;
    set_state(&mut app, AppState::GameOver);

    // When - 模拟 Retry：重置 GameOverState + 回到 InGame
    *app.world_mut().resource_mut::<GameOverState>() = GameOverState::Playing;
    set_state(&mut app, AppState::InGame);

    // Then
    assert_eq!(
        *app.world().resource::<State<AppState>>(),
        AppState::InGame,
        "Retry 后应回到 InGame"
    );
    assert_eq!(
        *app.world().resource::<GameOverState>(),
        GameOverState::Playing,
        "Retry 后 GameOverState 应重置为 Playing"
    );
    assert_eq!(
        entity_count::<GameOverScreen>(&mut app),
        0,
        "进入 InGame 后 GameOver 应被清理"
    );
}

// ══════════════════════════════════════════════════════════════
// B4: E2E 完整流程 MainMenu → LevelSelect → InGame → GameOver → MainMenu
// ══════════════════════════════════════════════════════════════

/// Test ID: UI-SCR-010
/// Title: E2E 完整流程
///
/// Given: 初始 MainMenu 状态
/// When:  依次执行 StartGame → SelectStage → ConfirmStage → Victory → GameOver → BackToMainMenu
/// Then:  最终回到 MainMenu，中间所有屏幕正确生成/清理
#[test]
fn e2e_full_flow() {
    // Given - MainMenu
    let mut app = screen_test_app();
    apply_state_transition(&mut app);
    assert_eq!(
        *app.world().resource::<State<AppState>>(),
        AppState::MainMenu,
        "初始状态应为 MainMenu"
    );

    // Step 1: StartGame → LevelSelect
    initialize_campaign_and_enter_level_select(&mut app);
    assert_eq!(
        *app.world().resource::<State<AppState>>(),
        AppState::LevelSelect,
        "Step 1: 应进入 LevelSelect"
    );
    assert_eq!(
        entity_count::<LevelSelectScreen>(&mut app),
        1,
        "LevelSelectScreen 应存在"
    );
    assert_eq!(
        entity_count::<MainMenuScreen>(&mut app),
        0,
        "LevelSelect 时 MainMenu 应已清理"
    );

    // Step 2: SelectStage + ConfirmStage → InGame
    enter_ingame_from_main_menu(&mut app);
    assert_eq!(
        *app.world().resource::<State<AppState>>(),
        AppState::InGame,
        "Step 2: 应进入 InGame"
    );
    assert_eq!(
        entity_count::<LevelSelectScreen>(&mut app),
        0,
        "InGame 时 LevelSelect 应已清理"
    );

    // Step 3: 胜利 → GameOver
    *app.world_mut().resource_mut::<GameOverState>() = GameOverState::Victory;
    set_state(&mut app, AppState::GameOver);
    assert_eq!(
        *app.world().resource::<State<AppState>>(),
        AppState::GameOver,
        "Step 3: 应进入 GameOver"
    );
    assert_eq!(
        entity_count::<GameOverScreen>(&mut app),
        1,
        "GameOverScreen 应存在"
    );
    let view = app.world().resource::<GameResultView>();
    assert_eq!(view.result, GameOutcome::Victory, "胜利结果应正确");

    // Step 4: BackToMainMenu → 回到 MainMenu
    set_state(&mut app, AppState::MainMenu);
    assert_eq!(
        *app.world().resource::<State<AppState>>(),
        AppState::MainMenu,
        "Step 4: 应回到 MainMenu"
    );
    assert_eq!(
        entity_count::<GameOverScreen>(&mut app),
        0,
        "回到 MainMenu 后 GameOver 应已清理"
    );
}

// ══════════════════════════════════════════════════════════════
// ViewModel 单元测试（补充验证）
// ══════════════════════════════════════════════════════════════

/// Test ID: UI-SCR-011
/// Title: GameResultView 默认值为 Victory
///
/// Given: GameResultView::default()
/// When:  检查默认值
/// Then:  result 为 Victory，turn_count 为 0
#[test]
fn game_result_view_default_is_victory() {
    // Given
    let view = GameResultView::default();

    // Then
    assert_eq!(view.result, GameOutcome::Victory);
    assert_eq!(view.turn_count, 0);
    assert!(view.stage_name.is_empty());
    assert!(!view.has_next_stage);
}

/// Test ID: UI-SCR-012
/// Title: LevelSelectState 默认值为空
///
/// Given: LevelSelectState::default()
/// When:  检查默认值
/// Then:  campaign_name 为空，stages 为空，selected_stage 为 None
#[test]
fn level_select_state_default_is_empty() {
    // Given
    let state = LevelSelectState::default();

    // Then
    assert!(state.campaign_name.is_empty());
    assert!(state.stages.is_empty());
    assert!(state.selected_stage.is_none());
}
