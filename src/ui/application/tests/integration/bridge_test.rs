//! Bridge 集成测试 — process_ui_commands Observer 接线验证
//!
//! 测试验证 `process_ui_commands` Observer 正确地将 UiCommand
//! 映射到 GameCommand 并推入 CommandQueue。
//!
//! 这些是 ECS 集成测试 — 使用带 MinimalPlugins 的 Bevy App。

use bevy::prelude::*;

use crate::core::capabilities::runtime::command::foundation::{CommandQueue, GameCommand};
use crate::ui::application::UiCommand;
use crate::ui::application::bridge::process_ui_commands;
use crate::ui::navigation::ScreenType;

// ── 辅助函数 ─────────────────────────────────────────────────────────────

/// 构建注册了 process_ui_commands observer 的最小 App。
fn bridge_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<CommandQueue>();
    app.add_observer(process_ui_commands);
    app
}

/// 触发 UiCommand 并返回被推入 CommandQueue 的命令列表。
fn trigger_and_drain(app: &mut App, cmd: UiCommand) -> Vec<GameCommand> {
    app.world_mut().trigger(cmd);
    app.world_mut().resource_mut::<CommandQueue>().drain()
}

/// 验证 UiCommand 映射并产生预期的 GameCommand。
fn assert_command_mapping(ui_cmd: UiCommand, expected: GameCommand) {
    let mut app = bridge_app();
    let commands = trigger_and_drain(&mut app, ui_cmd);
    assert_eq!(commands.len(), 1, "expected exactly 1 command in queue");
    assert_eq!(commands[0], expected, "command mapping mismatch");
}

/// 验证 UiCommand 不会被推入 CommandQueue。
fn assert_no_command_enqueued(ui_cmd: UiCommand) {
    let mut app = bridge_app();
    let commands = trigger_and_drain(&mut app, ui_cmd);
    assert!(
        commands.is_empty(),
        "expected no commands in queue for UI-internal command"
    );
}

// ── 战斗命令 ─────────────────────────────────────────────────────────────

#[test]
fn process_ui_commands_enqueues_end_turn() {
    assert_command_mapping(
        UiCommand::EndTurn {
            unit_id: String::new(),
        },
        GameCommand::EndTurn {
            unit_id: String::new(),
        },
    );
}

#[test]
fn process_ui_commands_enqueues_cast_skill() {
    assert_command_mapping(
        UiCommand::CastSkill {
            skill_def_id: "spl_001".into(),
            target_id: "unit_2".into(),
            caster_id: "unit_1".into(),
        },
        GameCommand::CastSpell {
            caster_id: "unit_1".into(),
            spell_def_id: "spl_001".into(),
            target_id: "unit_2".into(),
        },
    );
}

#[test]
fn process_ui_commands_enqueues_move_to_position() {
    assert_command_mapping(
        UiCommand::MoveToPosition {
            unit_id: "u1".into(),
            x: 5,
            y: 3,
        },
        GameCommand::MoveUnit {
            unit_id: "u1".into(),
            path: vec!["5,3".into()],
        },
    );
}

// ── 背包命令 ─────────────────────────────────────────────────────────────

#[test]
fn process_ui_commands_enqueues_use_item() {
    assert_command_mapping(
        UiCommand::UseItem {
            item_instance_id: "potion_1".into(),
            user_id: "u1".into(),
            target_id: None,
        },
        GameCommand::UseItem {
            user_id: "u1".into(),
            item_instance_id: "potion_1".into(),
            target_id: None,
        },
    );
}

#[test]
fn process_ui_commands_enqueues_equip_item() {
    assert_command_mapping(
        UiCommand::EquipItem {
            unit_id: "u1".into(),
            item_instance_id: "sword_1".into(),
            slot_index: 0,
        },
        GameCommand::EquipItem {
            unit_id: "u1".into(),
            item_instance_id: "sword_1".into(),
            slot_index: 0,
        },
    );
}

// ── 任务命令 ─────────────────────────────────────────────────────────────

#[test]
fn process_ui_commands_enqueues_accept_quest() {
    assert_command_mapping(
        UiCommand::AcceptQuest {
            unit_id: "u1".into(),
            quest_def_id: "q_001".into(),
        },
        GameCommand::AcceptQuest {
            unit_id: "u1".into(),
            quest_def_id: "q_001".into(),
        },
    );
}

#[test]
fn process_ui_commands_enqueues_abandon_quest() {
    assert_command_mapping(
        UiCommand::AbandonQuest {
            unit_id: "u1".into(),
            quest_def_id: "q_001".into(),
        },
        GameCommand::AbandonQuest {
            unit_id: "u1".into(),
            quest_def_id: "q_001".into(),
        },
    );
}

// ── 经济命令 ─────────────────────────────────────────────────────────────

#[test]
fn process_ui_commands_enqueues_buy_item() {
    assert_command_mapping(
        UiCommand::BuyItem {
            item_def_id: "potion".into(),
            quantity: 2,
            shop_id: "shop_1".into(),
        },
        GameCommand::BuyItem {
            buyer_id: String::new(),
            item_def_id: "potion".into(),
            quantity: 2,
            shop_id: "shop_1".into(),
        },
    );
}

#[test]
fn process_ui_commands_enqueues_sell_item() {
    assert_command_mapping(
        UiCommand::SellItem {
            item_def_id: "potion".into(),
            quantity: 1,
            shop_id: "shop_1".into(),
        },
        GameCommand::SellItem {
            seller_id: String::new(),
            item_def_id: "potion".into(),
            quantity: 1,
            shop_id: "shop_1".into(),
        },
    );
}

// ── 存档命令 ─────────────────────────────────────────────────────────────

#[test]
fn process_ui_commands_enqueues_save_game() {
    assert_command_mapping(UiCommand::SaveGame(1), GameCommand::SaveGame);
}

#[test]
fn process_ui_commands_enqueues_load_game() {
    assert_command_mapping(UiCommand::LoadGame(2), GameCommand::LoadGame);
}

// ── 系统命令 ─────────────────────────────────────────────────────────────

#[test]
fn process_ui_commands_enqueues_new_game() {
    assert_command_mapping(UiCommand::NewGame, GameCommand::NewGame);
}

#[test]
fn process_ui_commands_enqueues_toggle_pause_as_open_menu() {
    assert_command_mapping(UiCommand::TogglePause, GameCommand::OpenMenu);
}

// ── UI 内部命令（不入队）────────────────────────────────────────────────

#[test]
fn process_ui_commands_does_not_enqueue_open_screen() {
    assert_no_command_enqueued(UiCommand::OpenScreen(ScreenType::Settings));
}

#[test]
fn process_ui_commands_does_not_enqueue_close_screen() {
    assert_no_command_enqueued(UiCommand::CloseScreen);
}

#[test]
fn process_ui_commands_does_not_enqueue_select_target() {
    assert_no_command_enqueued(UiCommand::SelectTarget(42));
}

// ── 队列行为 ─────────────────────────────────────────────────────────────

#[test]
fn process_ui_commands_accumulates_multiple_commands() {
    let mut app = bridge_app();

    app.world_mut().trigger(UiCommand::NewGame);
    app.world_mut().trigger(UiCommand::TogglePause);

    let commands = app.world_mut().resource_mut::<CommandQueue>().drain();
    assert_eq!(
        commands.len(),
        2,
        "multiple triggers must accumulate multiple commands in queue"
    );
    assert_eq!(commands[0], GameCommand::NewGame);
    assert_eq!(commands[1], GameCommand::OpenMenu);
}

#[test]
fn process_ui_commands_skips_ui_internal_but_processes_others() {
    let mut app = bridge_app();

    // Mix of internal and domain commands — internal ones should be skipped
    app.world_mut()
        .trigger(UiCommand::OpenScreen(ScreenType::Settings));
    app.world_mut().trigger(UiCommand::EndTurn {
        unit_id: String::new(),
    });
    app.world_mut().trigger(UiCommand::CloseScreen);

    let commands = app.world_mut().resource_mut::<CommandQueue>().drain();
    assert_eq!(
        commands.len(),
        1,
        "only EndTurn should be enqueued; OpenScreen and CloseScreen are UI-internal"
    );
    assert_eq!(
        commands[0],
        GameCommand::EndTurn {
            unit_id: String::new()
        }
    );
}
