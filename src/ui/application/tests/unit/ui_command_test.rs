//! UiCommand → GameCommand 映射单元测试
//!
//! 测试 `UiCommand::into_game_command()` 的契约：
//! - 所有战斗/背包/任务/经济/系统命令正确映射到 GameCommand
//! - UI 内部导航命令（OpenScreen、CloseScreen）返回 None
//! - 当前无等价命令的（SelectTarget）返回 None
//!
//! 这些是纯函数测试 — 不需要 ECS 设置。

use crate::core::capabilities::runtime::command::foundation::GameCommand;
use crate::ui::application::UiCommand;
use crate::ui::navigation::ScreenType;

// ── 战斗命令映射 ─────────────────────────────────────────────────────────

#[test]
fn end_turn_maps_to_game_command() {
    let cmd = UiCommand::EndTurn {
        unit_id: String::new(),
    };
    let result = cmd.into_game_command();

    assert_eq!(
        result,
        Some(GameCommand::EndTurn {
            unit_id: String::new(),
        }),
        "EndTurn must map to GameCommand::EndTurn"
    );
}

#[test]
fn cast_skill_maps_to_cast_spell() {
    let cmd = UiCommand::CastSkill {
        skill_def_id: "spl_001".into(),
        target_id: "unit_2".into(),
        caster_id: "unit_1".into(),
    };
    let result = cmd.into_game_command();

    assert_eq!(
        result,
        Some(GameCommand::CastSpell {
            caster_id: "unit_1".into(),
            spell_def_id: "spl_001".into(),
            target_id: "unit_2".into(),
        }),
        "CastSkill must map to GameCommand::CastSpell with matching fields"
    );
}

#[test]
fn move_to_position_maps_to_move_unit() {
    let cmd = UiCommand::MoveToPosition {
        unit_id: "u1".into(),
        x: 5,
        y: 3,
    };
    let result = cmd.into_game_command();

    assert_eq!(
        result,
        Some(GameCommand::MoveUnit {
            unit_id: "u1".into(),
            path: vec!["5,3".into()],
        }),
        "MoveToPosition must map to GameCommand::MoveUnit with path encoding x,y"
    );
}

#[test]
fn select_target_returns_none() {
    let cmd = UiCommand::SelectTarget(42);
    let result = cmd.into_game_command();

    assert_eq!(
        result, None,
        "SelectTarget must return None as there is no GameCommand equivalent"
    );
}

// ── 背包命令映射 ─────────────────────────────────────────────────────────

#[test]
fn use_item_maps_to_game_command() {
    let cmd = UiCommand::UseItem {
        item_instance_id: "potion_1".into(),
        user_id: "u1".into(),
        target_id: None,
    };
    let result = cmd.into_game_command();

    assert_eq!(
        result,
        Some(GameCommand::UseItem {
            user_id: "u1".into(),
            item_instance_id: "potion_1".into(),
            target_id: None,
        }),
        "UseItem must map to GameCommand::UseItem with matching fields"
    );
}

#[test]
fn use_item_with_target_maps_correctly() {
    let cmd = UiCommand::UseItem {
        item_instance_id: "heal_pot".into(),
        user_id: "u1".into(),
        target_id: Some("u2".into()),
    };
    let result = cmd.into_game_command();

    assert_eq!(
        result,
        Some(GameCommand::UseItem {
            user_id: "u1".into(),
            item_instance_id: "heal_pot".into(),
            target_id: Some("u2".into()),
        }),
        "UseItem must preserve optional target_id"
    );
}

#[test]
fn equip_item_maps_to_game_command() {
    let cmd = UiCommand::EquipItem {
        unit_id: "u1".into(),
        item_instance_id: "sword_1".into(),
        slot_index: 0,
    };
    let result = cmd.into_game_command();

    assert_eq!(
        result,
        Some(GameCommand::EquipItem {
            unit_id: "u1".into(),
            item_instance_id: "sword_1".into(),
            slot_index: 0,
        }),
        "EquipItem must map to GameCommand::EquipItem with matching fields"
    );
}

#[test]
fn drop_item_maps_to_game_command() {
    let cmd = UiCommand::DropItem {
        unit_id: "u1".into(),
        item_instance_id: "potion_1".into(),
        quantity: 1,
    };
    let result = cmd.into_game_command();

    assert_eq!(
        result,
        Some(GameCommand::DropItem {
            unit_id: "u1".into(),
            item_instance_id: "potion_1".into(),
            quantity: 1,
        }),
        "DropItem must map to GameCommand::DropItem with matching fields"
    );
}

// ── 任务命令映射 ─────────────────────────────────────────────────────────

#[test]
fn accept_quest_maps_to_game_command() {
    let cmd = UiCommand::AcceptQuest {
        unit_id: "u1".into(),
        quest_def_id: "q_001".into(),
    };
    let result = cmd.into_game_command();

    assert_eq!(
        result,
        Some(GameCommand::AcceptQuest {
            unit_id: "u1".into(),
            quest_def_id: "q_001".into(),
        }),
        "AcceptQuest must map to GameCommand::AcceptQuest with matching fields"
    );
}

#[test]
fn abandon_quest_maps_to_game_command() {
    let cmd = UiCommand::AbandonQuest {
        unit_id: "u1".into(),
        quest_def_id: "q_001".into(),
    };
    let result = cmd.into_game_command();

    assert_eq!(
        result,
        Some(GameCommand::AbandonQuest {
            unit_id: "u1".into(),
            quest_def_id: "q_001".into(),
        }),
        "AbandonQuest must map to GameCommand::AbandonQuest with matching fields"
    );
}

// ── 经济命令映射 ─────────────────────────────────────────────────────────

#[test]
fn buy_item_maps_to_game_command_with_empty_buyer_id() {
    let cmd = UiCommand::BuyItem {
        item_def_id: "potion".into(),
        quantity: 2,
        shop_id: "shop_1".into(),
    };
    let result = cmd.into_game_command();

    assert_eq!(
        result,
        Some(GameCommand::BuyItem {
            buyer_id: String::new(),
            item_def_id: "potion".into(),
            quantity: 2,
            shop_id: "shop_1".into(),
        }),
        "BuyItem must map to GameCommand::BuyItem with empty buyer_id for caller to fill"
    );
}

#[test]
fn sell_item_maps_to_game_command_with_empty_seller_id() {
    let cmd = UiCommand::SellItem {
        item_def_id: "potion".into(),
        quantity: 1,
        shop_id: "shop_1".into(),
    };
    let result = cmd.into_game_command();

    assert_eq!(
        result,
        Some(GameCommand::SellItem {
            seller_id: String::new(),
            item_def_id: "potion".into(),
            quantity: 1,
            shop_id: "shop_1".into(),
        }),
        "SellItem must map to GameCommand::SellItem with empty seller_id for caller to fill"
    );
}

// ── 存档命令映射 ─────────────────────────────────────────────────────────

#[test]
fn save_game_maps_to_save_game_dropping_slot() {
    let cmd = UiCommand::SaveGame(1);
    let result = cmd.into_game_command();

    assert_eq!(
        result,
        Some(GameCommand::SaveGame),
        "SaveGame must map to GameCommand::SaveGame, dropping the slot number"
    );
}

#[test]
fn load_game_maps_to_load_game_dropping_slot() {
    let cmd = UiCommand::LoadGame(2);
    let result = cmd.into_game_command();

    assert_eq!(
        result,
        Some(GameCommand::LoadGame),
        "LoadGame must map to GameCommand::LoadGame, dropping the slot number"
    );
}

// ── 系统命令映射 ─────────────────────────────────────────────────────────

#[test]
fn new_game_maps_to_new_game() {
    let cmd = UiCommand::NewGame;
    let result = cmd.into_game_command();

    assert_eq!(
        result,
        Some(GameCommand::NewGame),
        "NewGame must map to GameCommand::NewGame"
    );
}

#[test]
fn toggle_pause_maps_to_open_menu() {
    let cmd = UiCommand::TogglePause;
    let result = cmd.into_game_command();

    assert_eq!(
        result,
        Some(GameCommand::OpenMenu),
        "TogglePause must map to GameCommand::OpenMenu"
    );
}

// ── UI 内部命令（返回 None）─────────────────────────────────────────────

#[test]
fn open_screen_returns_none() {
    let cmd = UiCommand::OpenScreen(ScreenType::Settings);
    let result = cmd.into_game_command();

    assert_eq!(
        result, None,
        "OpenScreen must return None as it is UI-internal navigation"
    );
}

#[test]
fn open_screen_any_variant_returns_none() {
    for screen in &[
        ScreenType::MainMenu,
        ScreenType::Battle,
        ScreenType::Inventory,
        ScreenType::Shop,
        ScreenType::SaveLoad,
    ] {
        let cmd = UiCommand::OpenScreen(*screen);
        assert_eq!(
            cmd.into_game_command(),
            None,
            "OpenScreen({screen:?}) must return None"
        );
    }
}

#[test]
fn close_screen_returns_none() {
    let cmd = UiCommand::CloseScreen;
    let result = cmd.into_game_command();

    assert_eq!(
        result, None,
        "CloseScreen must return None as it is UI-internal navigation"
    );
}

#[test]
fn ui_internal_commands_are_distinct_from_no_mapping() {
    // OpenScreen/CloseScreen 和 SelectTarget 都返回 None，但原因不同。验证类型系统能区分它们。
    let open = UiCommand::OpenScreen(ScreenType::Settings);
    let close = UiCommand::CloseScreen;
    let select = UiCommand::SelectTarget(99);

    assert_eq!(open.into_game_command(), None);
    assert_eq!(close.into_game_command(), None);
    assert_eq!(select.into_game_command(), None);
}
