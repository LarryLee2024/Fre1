//! 战役模块 Feature Test (Part A: 内容加载)
//!
//! 测试 RON 配置文件的正确反序列化与数据转换:
//! - A1: enemy_goblin_leader 模板反序列化
//! - A2: tutorial 关卡 RON + LevelConfig.from_def 转换
//! - A3: player_archer 模板 skill_ids 包含 pierce

// ================================================
// AI Self-Check (test_spec.md §13.1)
// ================================================
// ✅ 测试行为，不是实现 —— 测试的是 RON 配置数据的正确性（数据驱动约定）
// ✅ 符合领域规则 —— 不修改 Definition 配置，只验证数据完整性
// ✅ 测试是确定性的 —— RON 文件内容不变则测试结果不变
// ✅ 使用标准数据 —— 使用项目仓库中的实际 RON 配置文件
// ✅ 没有测试私有实现 —— 通过公开的 *Def 类型和 from_def 方法测试
// ✅ 没有生成不在范围内的测试 —— 仅测试配置数据加载
// ================================================

use std::collections::HashMap;

use tactical_rpg::core::attribute::AttributeKind;
use tactical_rpg::core::character::template::{FactionDef, UnitTemplateDef};
use tactical_rpg::core::map::{LevelConfig, LevelConfigDef, TerrainRegistry};

// ══════════════════════════════════════════════════════════════
// A1: enemy_goblin_leader RON 反序列化
// ══════════════════════════════════════════════════════════════

/// Test ID: FT-CAMP-001
/// Title: enemy_goblin_leader RON 反序列化成功且字段正确
///
/// Given: assets/units/enemy_goblin_leader.ron 文件
/// When:  按 UnitTemplateDef 格式反序列化
/// Then:  所有字段值与 RON 定义一致
///
/// Assertions: id, name, faction, class, race, skill_ids, trait_ids, ai_behavior
#[test]
fn 敌方哥布林队长_ron反序列化正确() {
    // Given
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/content/characters/enemy_goblin_leader.ron"
    );
    let bytes = std::fs::read(path).expect("读取 enemy_goblin_leader.ron 失败");

    // When
    let def: UnitTemplateDef =
        ron::de::from_bytes(&bytes).expect("反序列化 enemy_goblin_leader.ron 失败");

    // Then
    assert_eq!(def.id, "enemy_goblin_leader", "单位 ID 应正确");
    assert_eq!(def.name, "哥布林队长", "单位名称应正确");
    assert_eq!(def.faction, FactionDef::Enemy, "阵营应为 Enemy");

    // 基本信息
    assert_eq!(def.race, "goblin", "种族应正确");
    assert_eq!(def.class, "warrior", "职业应正确");
    assert_eq!(def.background, "raider", "背景应正确");
    assert_eq!(def.base_attack_range, 1, "基础攻击范围应为 1");
    assert_eq!(def.ai_behavior, "cautious", "AI 行为应为 cautious");

    // 技能 ID
    assert!(
        def.skill_ids.contains(&"basic_attack".to_string()),
        "应包含 basic_attack 技能"
    );
    assert!(
        def.skill_ids.contains(&"charge".to_string()),
        "应包含 charge 技能"
    );
    assert_eq!(def.skill_ids.len(), 2, "应恰好有 2 个技能");

    // Trait ID
    assert!(
        def.trait_ids.contains(&"warrior_mastery".to_string()),
        "应包含 warrior_mastery trait"
    );
    assert_eq!(def.trait_ids.len(), 1, "应恰好有 1 个 trait");

    // 核心属性
    assert!(
        (def.base_attributes
            .get(&AttributeKind::Might)
            .copied()
            .unwrap_or(0.0)
            - 5.0)
            .abs()
            < f32::EPSILON
    );
    assert!(
        (def.base_attributes
            .get(&AttributeKind::Vitality)
            .copied()
            .unwrap_or(0.0)
            - 5.0)
            .abs()
            < f32::EPSILON
    );
    assert!(
        (def.base_attributes
            .get(&AttributeKind::Agility)
            .copied()
            .unwrap_or(0.0)
            - 5.0)
            .abs()
            < f32::EPSILON
    );
    assert!(
        (def.base_attributes
            .get(&AttributeKind::Intelligence)
            .copied()
            .unwrap_or(0.0)
            - 2.0)
            .abs()
            < f32::EPSILON
    );

    // 初始装备
    assert_eq!(def.initial_equipment.len(), 1, "应有一件初始装备");
}

// ══════════════════════════════════════════════════════════════
// A2: tutorial 关卡 RON + LevelConfig.from_def 转换
// ══════════════════════════════════════════════════════════════

/// Test ID: FT-CAMP-002
/// Title: tutorial 关卡 RON 反序列化成功且 LevelConfig.from_def 转换正确
///
/// Given: assets/maps/tutorial.ron 文件 + 默认 TerrainRegistry
/// When:  先反序列化为 LevelConfigDef，再通过 from_def 转换为 LevelConfig
/// Then:  反序列化字段正确，转换后数据完整
///
/// Assertions: id, name, dimensions, units, victory_condition, turn_limit
#[test]
fn 教学关卡_ron反序列化并转换正确() {
    // Given
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/content/stages/tutorial.ron");
    let bytes = std::fs::read(path).expect("读取 tutorial.ron 失败");

    // When - 反序列化 LevelConfigDef
    let def: LevelConfigDef = ron::de::from_bytes(&bytes).expect("反序列化 tutorial.ron 失败");

    // Then - LevelConfigDef 字段验证
    assert_eq!(def.id, "tutorial", "关卡 ID 应正确");
    assert_eq!(def.name, "教学关", "关卡名称应正确");
    assert_eq!(def.width, 10, "地图宽度应为 10");
    assert_eq!(def.height, 8, "地图高度应为 8");

    // 地形网格
    assert_eq!(def.terrain_grid.len(), 8, "地形网格应有 8 行");
    assert_eq!(def.terrain_grid[0].len(), 10, "地形网格每行应有 10 列");
    assert!(
        def.char_map.is_empty(),
        "不应有自定义 char_map（使用默认映射）"
    );

    // 单位部署
    assert_eq!(def.player_units.len(), 3, "应有 3 个玩家单位");
    assert_eq!(def.enemy_units.len(), 3, "应有 3 个敌方单位");
    assert_eq!(def.player_units[0].template, "player_warrior");
    assert_eq!(def.player_units[1].template, "player_archer");
    assert_eq!(def.player_units[2].template, "player_mage");
    assert_eq!(def.enemy_units[0].template, "enemy_goblin");
    assert_eq!(def.enemy_units[1].template, "enemy_goblin");
    assert_eq!(def.enemy_units[2].template, "enemy_dark_knight");

    // 胜负条件
    assert!(def.victory_condition.is_some(), "应有胜负条件配置");
    assert_eq!(def.turn_limit, Some(20), "回合上限应为 20");

    // 保存 def 的值用于后续比较（因为 def 将被移动到 from_def）
    let expected_height = def.height;
    let expected_width = def.width;

    // When - 转换为 LevelConfig
    let mut terrain_registry = TerrainRegistry::default();
    terrain_registry.register_defaults();
    let config = LevelConfig::from_def(def, &terrain_registry);

    // Then - LevelConfig 字段验证
    assert_eq!(config.id, "tutorial", "转换后 ID 应保持不变");
    assert_eq!(config.width, 10, "转换后宽度应保持不变");
    assert_eq!(config.height, 8, "转换后高度应保持不变");
    assert_eq!(config.player_units.len(), 3, "转换后应有 3 个玩家单位");
    assert_eq!(config.enemy_units.len(), 3, "转换后应有 3 个敌方单位");
    assert!(config.victory_condition.is_some(), "转换后应有胜负条件");
    assert_eq!(config.turn_limit, Some(20), "转换后回合上限应为 20");

    // 地形转换：验证部分已知格子的地形是否正确
    // Grid row 1: "MPPFPPPWPM" → col 0=M, 1=P, 2=P, 3=F, 7=W
    // Grid row 7: "MMMMMMMMMM" → all mountain
    assert_eq!(
        config.terrain_map.get(&(0, 0)),
        Some(&"mountain".to_string()),
        "(0,0) 应为 mountain"
    );
    assert_eq!(
        config.terrain_map.get(&(1, 1)),
        Some(&"plain".to_string()),
        "(1,1) 应为 plain"
    );
    assert_eq!(
        config.terrain_map.get(&(3, 1)),
        Some(&"forest".to_string()),
        "(3,1) 应为 forest (row 1 col 3 = 'F')"
    );
    assert_eq!(
        config.terrain_map.get(&(7, 1)),
        Some(&"water".to_string()),
        "(7,1) 应为 water (row 1 col 7 = 'W')"
    );

    // 验证 terrain_map 条目数 = width * height
    assert_eq!(
        config.terrain_map.len(),
        (expected_height * expected_width) as usize,
        "terrain_map 应包含所有格子"
    );
}

// ══════════════════════════════════════════════════════════════
// A3: player_archer 模板 skill_ids 含有 pierce
// ══════════════════════════════════════════════════════════════

/// Test ID: FT-CAMP-003
/// Title: player_archer 模板 skill_ids 包含 pierce 技能
///
/// Given: assets/units/player_archer.ron 文件
/// When:  按 UnitTemplateDef 格式反序列化
/// Then:  skill_ids 包含 "pierce" 和 "basic_attack"
///
/// Assertions: id, skill_ids
#[test]
fn 玩家弓箭手_技能含穿透() {
    // Given
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/content/characters/player_archer.ron"
    );
    let bytes = std::fs::read(path).expect("读取 player_archer.ron 失败");

    // When
    let def: UnitTemplateDef =
        ron::de::from_bytes(&bytes).expect("反序列化 player_archer.ron 失败");

    // Then
    assert_eq!(def.id, "player_archer", "单位 ID 应正确");
    assert_eq!(def.name, "弓箭手", "单位名称应正确");

    // 验证 skill_ids
    assert!(
        def.skill_ids.contains(&"pierce".to_string()),
        "弓箭手 skill_ids 应包含 pierce（内容修缮后新增的穿透技能）"
    );
    assert!(
        def.skill_ids.contains(&"basic_attack".to_string()),
        "弓箭手 skill_ids 应包含 basic_attack（基础攻击）"
    );
    assert_eq!(def.skill_ids.len(), 2, "应恰好有 2 个技能");

    // 验证其他字段完整性
    assert_eq!(def.faction, FactionDef::Player, "阵营应为 Player");
    assert_eq!(def.base_attack_range, 3, "弓箭手基础攻击范围应为 3");
    assert!(
        def.trait_ids.contains(&"archer_mastery".to_string()),
        "应包含 archer_mastery trait"
    );
}

/// Test ID: FT-CAMP-004
/// Title: player_mage 模板已添加 heal 技能
///
/// Given: assets/units/player_mage.ron 文件
/// When:  按 UnitTemplateDef 格式反序列化
/// Then:  skill_ids 包含 "heal"
///
/// Assertions: id, skill_ids contains heal
#[test]
fn 玩家法师_技能含治疗() {
    // Given
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/content/characters/player_mage.ron"
    );
    let bytes = std::fs::read(path).expect("读取 player_mage.ron 失败");

    // When
    let def: UnitTemplateDef = ron::de::from_bytes(&bytes).expect("反序列化 player_mage.ron 失败");

    // Then
    assert_eq!(def.id, "player_mage", "单位 ID 应正确");
    assert!(
        def.skill_ids.contains(&"heal".to_string()),
        "法师 skill_ids 应包含 heal（内容修缮后新增的治疗技能）"
    );
    assert!(
        def.skill_ids.contains(&"basic_attack".to_string()),
        "法师 skill_ids 应包含 basic_attack（基础攻击）"
    );
}

/// Test ID: FT-CAMP-005
/// Title: campaign_001 RON 反序列化
///
/// Given: assets/campaigns/campaign_001.ron 文件
/// When:  按 CampaignDef 格式反序列化
/// Then:  字段正确，stages 引用有效
///
/// Assertions: id, name, stages
#[test]
fn 战役001_ron反序列化正确() {
    use tactical_rpg::core::campaign::def::CampaignDef;

    // Given
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/content/campaigns/campaign_001.ron"
    );
    let bytes = std::fs::read(path).expect("读取 campaign_001.ron 失败");

    // When
    let def: CampaignDef = ron::de::from_bytes(&bytes).expect("反序列化 campaign_001.ron 失败");

    // Then
    assert_eq!(def.id, "campaign_001", "战役 ID 应正确");
    assert_eq!(def.name, "边境之旅", "战役名称应正确");
    assert_eq!(def.stages.len(), 1, "应有 1 个关卡");
    assert_eq!(def.stages[0].id, "stage_001", "第一个 Stage ID 应正确");
    assert_eq!(
        def.stages[0].level_id, "tutorial",
        "第一个 Stage 应引用 tutorial 关卡"
    );
}
