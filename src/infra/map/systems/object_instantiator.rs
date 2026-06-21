//! ObjectInstantiator — MapObject → ECS Entity 实例化
//!
//! 将 MapAsset 中的 ObjectLayer 和 SpawnPoint 转换为 ECS Entity。
//! Object 的 class 字段决定实例化策略。
//!
//! V1 实现：
//! - unit_spawn → 标记为 SpawnPointMarker（未来接入 SpawnGroupDef）
//! - decoration → 标记为 DecorationMarker（渲染帧占位）
//! - interactive → 标记为 InteractiveMarker（未来接入 InteractionSystem）
//! - trigger → 标记为 TriggerZoneMarker（未来接入 TriggerSystem）
//! - 未知 class → 仅创建 Entity + ObjectMarker（不报错）
//!
//! 详见 ADR-065 §6 (Object 实例化)

use bevy::prelude::*;

use super::super::asset::{MapObject, ObjectLayer, SpawnPoint};

// ─── 实例化标记组件 ──────────────────────────────────────────────

/// 标记所有由 MapObject 实例化的 Entity。
#[derive(Component, Debug, Clone)]
pub struct ObjectMarker {
    /// 稳定 GUID（内容哈希，跨存档稳定）
    pub guid: u64,
    /// Tiled 原始 ID（调试追溯用）
    pub tiled_id: u32,
    /// 对象名称
    pub name: String,
    /// 对象类型/Custom Class
    pub class: String,
}

/// 单位生成点标记。
#[derive(Component, Debug, Clone)]
pub struct SpawnPointMarker {
    /// 生成组 ID（引用 L3 SpawnGroupDef）
    pub spawn_group_id: String,
}

/// 装饰物标记。
#[derive(Component, Debug, Clone)]
pub struct DecorationMarker;

/// 可交互对象标记。
#[derive(Component, Debug, Clone)]
pub struct InteractiveMarker;

/// 触发器区域标记。
#[derive(Component, Debug, Clone)]
pub struct TriggerZoneMarker;

// ─── 实例化辅助函数 ──────────────────────────────────────────────

/// 实例化单个 MapObject 为 ECS Entity。
pub fn instantiate_object(commands: &mut Commands, obj: &MapObject, parent: Option<Entity>) {
    let mut entity_cmd = commands.spawn((
        Name::new(format!("Obj:{}", obj.name)),
        ObjectMarker {
            guid: obj.guid.get(),
            tiled_id: obj.tiled_id,
            name: obj.name.clone(),
            class: obj.class.clone(),
        },
        Transform::from_xyz(
            obj.position.x as f32 * 64.0,
            obj.position.y as f32 * 64.0,
            0.0,
        ),
    ));

    // 根据 class 添加特定标记组件
    match obj.class.as_str() {
        "decoration" | "Decoration" => {
            entity_cmd.insert(DecorationMarker);
        }
        "interactive" | "Interactive" => {
            entity_cmd.insert(InteractiveMarker);
        }
        "trigger" | "Trigger" | "TriggerZone" => {
            entity_cmd.insert(TriggerZoneMarker);
        }
        // "unit_spawn" 由 spawn_points 单独处理
        _ => {
            // 未知 class：仅创建 ObjectMarker，不添加额外标记
            trace!(target: "map",
                "[Map] 未知 Object class '{}'（name: {}），仅创建 ObjectMarker",
                obj.class, obj.name
            );
        }
    }

    if let Some(parent_entity) = parent {
        entity_cmd.set_parent_in_place(parent_entity);
    }
}

/// 实例化单个 SpawnPoint 为 ECS Entity。
pub fn instantiate_spawn_point(
    commands: &mut Commands,
    spawn: &SpawnPoint,
    parent: Option<Entity>,
) {
    let mut entity_cmd = commands.spawn((
        Name::new(format!("Spawn:{}", spawn.spawn_group_id)),
        ObjectMarker {
            guid: spawn.guid.get(),
            tiled_id: 0,
            name: format!("SpawnPoint({})", spawn.spawn_group_id),
            class: "unit_spawn".to_string(),
        },
        SpawnPointMarker {
            spawn_group_id: spawn.spawn_group_id.clone(),
        },
        Transform::from_xyz(
            spawn.position.x as f32 * 64.0,
            spawn.position.y as f32 * 64.0,
            0.0,
        ),
    ));

    if let Some(parent_entity) = parent {
        entity_cmd.set_parent_in_place(parent_entity);
    }
}

/// 实例化整个 ObjectLayer 下的所有对象。
pub fn instantiate_object_layer(
    commands: &mut Commands,
    layer: &ObjectLayer,
    parent: Option<Entity>,
) {
    for obj in &layer.objects {
        instantiate_object(commands, obj, parent);
    }
}
