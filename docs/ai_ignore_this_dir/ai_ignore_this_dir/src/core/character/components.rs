// 角色组件定义：单位身份、阵营、位置、标记等

use super::traits::TraitCollection;
use crate::core::ability::{SkillCooldowns, SkillSlots};
use crate::core::attribute::Attributes;

use crate::core::equipment::EquipmentSlots;
use crate::core::inventory::container::Container;
use crate::core::tag::{GameplayTags, PersistentTags};
use bevy::ecs::lifecycle::{Add, HookContext};
use bevy::ecs::observer::On;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;

/// 阵营
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect, serde::Serialize, serde::Deserialize,
)]
#[reflect(Serialize, Deserialize)]
pub enum Faction {
    Player,
    Enemy,
}

/// 战斗单位组件（身份与回合状态）
/// Required Components：生成 Unit 时自动插入默认组件，防止遗漏
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(
    Attributes,
    SkillSlots,
    SkillCooldowns,
    GameplayTags,
    PersistentTags,
    TraitCollection,
    EquipmentSlots,
    Container,
    GridPosition
)]
pub struct Unit {
    pub faction: Faction,
    pub acted: bool,
}

/// 单位名称（UI 显示用）
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct UnitName(pub String);

/// 单位业务 ID（逻辑标识，如 "knight_001"）
/// Name 用于 Inspector 调试，UnitId 用于业务逻辑，UnitName 用于 UI 显示
#[derive(
    Component, Reflect, Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[reflect(Component, Serialize, Deserialize)]
pub struct UnitId(pub String);

/// 单位种族
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct UnitRace(pub String);

/// 单位职业
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct UnitClass(pub String);

/// 单位所在格子坐标
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct GridPosition {
    pub coord: IVec2,
}

impl Default for GridPosition {
    fn default() -> Self {
        Self { coord: IVec2::ZERO }
    }
}

/// 选中标记
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Selected;

/// 死亡标记：HP 降为 0 时添加，Hook 自动清理固有状态
#[derive(Component, Reflect)]
#[reflect(Component)]
#[component(on_add = Dead::on_add_dead)]
pub struct Dead;

impl Dead {
    /// 死亡 Hook：标记已行动，移除选中状态
    /// Hook = 固有行为（宪法 5.0），只处理 Dead 组件自身的固有逻辑
    fn on_add_dead(mut world: DeferredWorld, context: HookContext) {
        let entity = context.entity;
        let unit_id = world
            .get::<UnitId>(entity)
            .map(|id| id.0.as_str())
            .unwrap_or("?");
        bevy::log::trace!(target: "character", entity=?entity, unit_id=%unit_id, "Dead hook triggered");
        // 标记已行动，防止死亡单位继续行动
        if let Some(mut unit) = world.get_mut::<Unit>(entity) {
            unit.acted = true;
        }
        // 移除选中标记
        world.commands().entity(entity).remove::<Selected>();
    }
}

/// Dead Observer：响应 Dead Tag 添加，发送 CharacterDied Message
/// Observer = 局部响应（宪法 5.0），负责死亡事件的跨 Feature 广播
/// 规则3：HP ≤ 0 时只添加 Dead Tag，死亡通知由 Observer 统一发送
pub fn on_dead_added(
    trigger: On<Add, Dead>,
    mut commands: Commands,
    mut died_writer: bevy::ecs::message::MessageWriter<crate::core::battle::CharacterDied>,
    units: Query<(&Unit, &UnitName, Option<&UnitId>), With<Dead>>,
) {
    if let Ok((unit, name, unit_id)) = units.get(trigger.entity) {
        bevy::log::trace!(
            target: "character",
            entity = ?trigger.entity,
            name = %name.0,
            "CharacterDied 消息发送(Dead Observer)"
        );
        died_writer.write(crate::core::battle::CharacterDied {
            entity: trigger.entity,
            name: name.0.clone(),
            faction: unit.faction,
        });
        // TODO(future): Remove fallback once all entities carry &UnitId component
        let shared_uid = unit_id
            .map(|uid| crate::shared::ids::UnitId::new(&uid.0))
            .unwrap_or_else(|| {
                crate::shared::ids::UnitId::new(trigger.entity.to_bits().to_string())
            });
        commands.write_message(crate::shared::event::battle::CharacterDied {
            unit_id: shared_uid,
            name: name.0.clone(),
            killed_by: None,
            faction: format!("{:?}", unit.faction),
        });
    }
}

/// HP 条背景
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct HpBarBg;

/// HP 条前景
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct HpBarFg;

/// AI 行为 ID（敌方单位使用）
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct AiBehaviorId(pub String);

/// 导航箭头标记（路径上的小圆点）
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PathArrow;

/// 移动动画组件：挂在正在移动的单位上，系统逐格插值
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovingUnit {
    /// 路径坐标序列（含终点）
    pub path: Vec<IVec2>,
    /// 当前正在前往的路径索引
    pub current_index: usize,
    /// 每格移动耗时（秒）
    pub speed: f32,
    /// 当前格内已用时间
    pub elapsed: f32,
    /// 移动完成后的回调阶段
    pub next_phase: crate::core::turn::TurnPhase,
}

impl MovingUnit {
    /// 当前目标坐标
    pub fn target_coord(&self) -> Option<IVec2> {
        self.path.get(self.current_index).copied()
    }

    /// 是否已到达终点
    pub fn is_finished(&self) -> bool {
        self.current_index >= self.path.len()
    }
}

#[cfg(test)]
mod tests {
    // ================================================
    // AI Self-Check (test_spec.md §13.1)
    // ================================================
    // ✅ 测试行为，不是实现
    // ✅ 符合领域规则
    // ✅ 测试是确定性的
    // ✅ 使用标准测试数据
    // ✅ 没有测试私有实现
    // ✅ 没有生成不在范围内的测试
    // ================================================

    use super::*;
    use crate::core::ability::SkillSlots;
    use crate::core::attribute::Attributes;
    use crate::core::turn::TurnPhase;

    fn make_moving_unit(path: Vec<IVec2>, current_index: usize) -> MovingUnit {
        MovingUnit {
            path,
            current_index,
            speed: 0.1,
            elapsed: 0.0,
            next_phase: TurnPhase::SelectUnit,
        }
    }

    // ── target_coord 测试 ──

    /// Test ID: CHR-MOV-001
    /// Title: MovingUnit.target_coord 返回路径中当前索引处的坐标
    ///
    /// Given: 路径 [(0,0), (1,0), (2,0)]，当前索引 1
    /// When: 调用 target_coord()
    /// Then: 返回 (1,0)
    ///
    /// Assertions: target_coord == Some(IVec2::new(1, 0))
    #[test]
    fn 移动单位_目标坐标在路径内() {
        // Given
        let unit = make_moving_unit(
            vec![IVec2::new(0, 0), IVec2::new(1, 0), IVec2::new(2, 0)],
            1,
        );

        // When
        let result = unit.target_coord();

        // Then
        assert_eq!(result, Some(IVec2::new(1, 0)));
    }

    /// Test ID: CHR-MOV-002
    /// Title: MovingUnit.target_coord 空路径返回 None
    ///
    /// Given: 空路径，当前索引 0
    /// When: 调用 target_coord()
    /// Then: 返回 None
    ///
    /// Assertions: target_coord == None
    #[test]
    fn 移动单位_目标坐标空路径() {
        // Given
        let unit = make_moving_unit(vec![], 0);

        // When
        let result = unit.target_coord();

        // Then
        assert_eq!(result, None);
    }

    /// Test ID: CHR-MOV-003
    /// Title: MovingUnit.target_coord 索引越界返回 None
    ///
    /// Given: 路径 [(0,0)]，当前索引 5（越界）
    /// When: 调用 target_coord()
    /// Then: 返回 None
    ///
    /// Assertions: target_coord == None
    #[test]
    fn 移动单位_目标坐标索引越界() {
        // Given
        let unit = make_moving_unit(vec![IVec2::new(0, 0)], 5);

        // When
        let result = unit.target_coord();

        // Then
        assert_eq!(result, None);
    }

    // ── is_finished 测试 ──

    /// Test ID: CHR-MOV-004
    /// Title: MovingUnit.is_finished 未完成时返回 false
    ///
    /// Given: 路径 [(0,0), (1,0)]，当前索引 0
    /// When: 调用 is_finished()
    /// Then: 返回 false
    ///
    /// Assertions: is_finished == false
    #[test]
    fn 移动单位_未完成() {
        // Given
        let unit = make_moving_unit(vec![IVec2::new(0, 0), IVec2::new(1, 0)], 0);

        // When
        let result = unit.is_finished();

        // Then
        assert!(!result);
    }

    /// Test ID: CHR-MOV-005
    /// Title: MovingUnit.is_finished 已完成时返回 true
    ///
    /// Given: 路径 [(0,0)]，当前索引 1（超出路径长度）
    /// When: 调用 is_finished()
    /// Then: 返回 true
    ///
    /// Assertions: is_finished == true
    #[test]
    fn 移动单位_已完成() {
        // Given
        let unit = make_moving_unit(vec![IVec2::new(0, 0)], 1);

        // When
        let result = unit.is_finished();

        // Then
        assert!(result);
    }

    /// Test ID: CHR-MOV-006
    /// Title: MovingUnit.is_finished 空路径返回 true
    ///
    /// Given: 空路径，当前索引 0
    /// When: 调用 is_finished()
    /// Then: 返回 true
    ///
    /// Assertions: is_finished == true
    #[test]
    fn 移动单位_空路径() {
        // Given
        let unit = make_moving_unit(vec![], 0);

        // When
        let result = unit.is_finished();

        // Then
        assert!(result);
    }

    /// Test ID: CHR-MOV-007
    /// Title: MovingUnit.is_finished 刚到达终点返回 true
    ///
    /// Given: 路径 [(0,0), (1,0)]，当前索引 2（等于路径长度）
    /// When: 调用 is_finished()
    /// Then: 返回 true
    ///
    /// Assertions: is_finished == true
    #[test]
    fn 移动单位_刚到达() {
        // Given
        let unit = make_moving_unit(vec![IVec2::new(0, 0), IVec2::new(1, 0)], 2);

        // When
        let result = unit.is_finished();

        // Then
        assert!(result);
    }

    // ── Dead Hook 测试 ──

    /// Test ID: CHR-DEAD-001
    /// Title: Dead Hook 标记已行动
    ///
    /// Given: 一个未行动的 Unit (acted=false)
    /// When: 插入 Dead 组件
    /// Then: Unit.acted 变为 true
    ///
    /// Assertions: unit.acted == true
    #[test]
    fn dead_hook_标记单位已行动() {
        // Given
        let mut world = World::new();
        let entity = world
            .spawn((Unit {
                faction: Faction::Player,
                acted: false,
            },))
            .id();

        // When
        world.entity_mut(entity).insert(Dead);

        // Then
        let unit = world.get::<Unit>(entity).unwrap();
        assert!(unit.acted);
    }

    /// Test ID: CHR-DEAD-002
    /// Title: Dead Hook 移除 Selected 组件
    ///
    /// Given: 一个带有 Selected 组件的 Unit
    /// When: 插入 Dead 组件
    /// Then: Selected 组件被移除
    ///
    /// Assertions: world.get::<Selected>(entity).is_none()
    #[test]
    fn dead_hook_移除选中() {
        // Given
        let mut world = World::new();
        let entity = world
            .spawn((
                Unit {
                    faction: Faction::Player,
                    acted: false,
                },
                Selected,
            ))
            .id();

        // When
        world.entity_mut(entity).insert(Dead);

        // Then
        assert!(world.get::<Selected>(entity).is_none());
    }

    /// Test ID: CHR-DEAD-003
    /// Title: Dead Hook 无 Selected 组件时不报错
    ///
    /// Given: 一个没有 Selected 组件的 Unit
    /// When: 插入 Dead 组件
    /// Then: 不报错，Unit.acted 变为 true
    ///
    /// Assertions: unit.acted == true
    #[test]
    fn dead_hook_无选中不panic() {
        // Given
        let mut world = World::new();
        let entity = world
            .spawn((Unit {
                faction: Faction::Player,
                acted: false,
            },))
            .id();

        // When
        world.entity_mut(entity).insert(Dead);

        // Then
        assert!(world.get::<Unit>(entity).unwrap().acted);
    }

    /// Test ID: CHR-REQ-001
    /// Title: Unit 生成时自动插入 Required Components
    ///
    /// Given: 一个新生成的 Unit
    /// When: 检查 Required Components
    /// Then: Attributes, SkillSlots, GridPosition, ActiveBuffs 均存在
    ///
    /// Assertions: 所有 Required Components 存在
    #[test]
    fn unit_自动插入必需组件() {
        // Given
        let mut world = World::new();
        let entity = world
            .spawn(Unit {
                faction: Faction::Player,
                acted: false,
            })
            .id();

        // When - 无需操作

        // Then
        assert!(world.get::<Attributes>(entity).is_some());
        assert!(world.get::<SkillSlots>(entity).is_some());
        assert!(world.get::<GridPosition>(entity).is_some());
        // ActiveBuffs 已移除（ADR-026：Buff 统一为 ApplyModifier）
    }

    /// Test ID: CHR-UID-001
    /// Title: UnitId 组件基本属性正确
    ///
    /// Given: UnitId("knight_001")
    /// When: 读取 .0 字段
    /// Then: 返回 "knight_001"
    ///
    /// Assertions: id.0 == "knight_001"
    #[test]
    fn unit_id_基本属性() {
        // Given
        let id = UnitId("knight_001".into());

        // When - 无需操作

        // Then
        assert_eq!(id.0, "knight_001");
    }

    /// Test ID: CHR-UID-002
    /// Title: UnitId 相等与哈希正确
    ///
    /// Given: 两个相同 ID 和一个不同 ID 的 UnitId
    /// When: 比较相等性和哈希
    /// Then: 相同 ID 相等且哈希相同，不同 ID 不等
    ///
    /// Assertions: a == b, a != c, set.contains(b), !set.contains(c)
    #[test]
    fn unit_id_相等性和哈希() {
        // Given
        let a = UnitId("knight_001".into());
        let b = UnitId("knight_001".into());
        let c = UnitId("mage_001".into());

        // When - 无需操作

        // Then
        assert_eq!(a, b);
        assert_ne!(a, c);
        let mut set = std::collections::HashSet::new();
        set.insert(a.clone());
        assert!(set.contains(&b));
        assert!(!set.contains(&c));
    }

    /// Test ID: CHR-UID-003
    /// Title: UnitId 挂载与读取正确
    ///
    /// Given: 一个带有 UnitId 组件的 Unit
    /// When: 从 ECS 读取 UnitId
    /// Then: 返回正确的 ID 值
    ///
    /// Assertions: unit_id.0 == "warrior_001"
    #[test]
    fn unit_id_挂载和读取() {
        // Given
        let mut world = World::new();
        let entity = world
            .spawn((
                Unit {
                    faction: Faction::Player,
                    acted: false,
                },
                UnitId("warrior_001".into()),
            ))
            .id();

        // When
        let unit_id = world.get::<UnitId>(entity).unwrap();

        // Then
        assert_eq!(unit_id.0, "warrior_001");
    }
}
