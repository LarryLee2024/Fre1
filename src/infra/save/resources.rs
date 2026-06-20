use std::collections::HashMap;
use std::path::PathBuf;

use bevy::prelude::*;

/// 存档元数据（玩家可见）。
/// 存档/读档系统 Resource & Observer
#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct SaveMetadata {
    pub label: String,
    pub location: String,
    pub playtime_seconds: u64,
    pub player_level: u32,
}

impl Default for SaveMetadata {
    fn default() -> Self {
        Self {
            label: "Untitled Save".into(),
            location: "Unknown".into(),
            playtime_seconds: 0,
            player_level: 1,
        }
    }
}

/// 存档管理器 — 当前存档会话状态。
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct SaveManager {
    pub current_save_path: Option<PathBuf>,
    pub metadata: SaveMetadata,
    pub is_dirty: bool,
    pub save_version: u32,
}

impl Default for SaveManager {
    fn default() -> Self {
        Self {
            current_save_path: None,
            metadata: SaveMetadata::default(),
            is_dirty: false,
            save_version: 1,
        }
    }
}

/// 自动保存配置。
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct AutoSaveConfig {
    pub enabled: bool,
    pub interval_minutes: u32,
    pub on_battle_start: bool,
    pub on_battle_end: bool,
    pub on_camp_enter: bool,
    pub max_auto_saves: u32,
}

impl Default for AutoSaveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_minutes: 15,
            on_battle_start: true,
            on_battle_end: true,
            on_camp_enter: true,
            max_auto_saves: 5,
        }
    }
}

/// 持久化 Entity ID — 存档中使用的稳定 ID。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct PersistentEntityId(pub u64);

/// Entity 重映射表 — 存档 ↔ 运行时的双向映射。
///
/// 使用 HashMap 实现 O(1) 查询，遵循 ADR-042 §3 规范。
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct EntityRemapper {
    persistent_to_entity: HashMap<PersistentEntityId, Entity>,
    entity_to_persistent: HashMap<Entity, PersistentEntityId>,
    next_id: u64,
}

impl Default for EntityRemapper {
    fn default() -> Self {
        Self {
            persistent_to_entity: HashMap::new(),
            entity_to_persistent: HashMap::new(),
            next_id: 1,
        }
    }
}

impl EntityRemapper {
    /// 为运行时 Entity 分配持久化 ID（首次序列化时调用）；已分配的返回已有 ID。
    pub fn assign(&mut self, entity: Entity) -> PersistentEntityId {
        let id = PersistentEntityId(self.next_id);
        self.next_id += 1;
        self.persistent_to_entity.insert(id, entity);
        self.entity_to_persistent.insert(entity, id);
        id
    }

    /// 持久化 ID → 运行时 Entity（加载时重建引用）。
    pub fn lookup(&self, pid: PersistentEntityId) -> Option<Entity> {
        self.persistent_to_entity.get(&pid).copied()
    }

    /// 运行时 Entity → 持久化 ID（保存时序列化引用）。
    pub fn lookup_persistent(&self, entity: Entity) -> Option<PersistentEntityId> {
        self.entity_to_persistent.get(&entity).copied()
    }

    /// 清空所有映射（开始新存档或加载时调用）。
    pub fn clear(&mut self) {
        self.persistent_to_entity.clear();
        self.entity_to_persistent.clear();
        self.next_id = 1;
    }

    /// 当前是否有映射条目（用于断言和调试）。
    pub fn is_empty(&self) -> bool {
        self.persistent_to_entity.is_empty()
    }

    /// 当前映射条目数。
    pub fn len(&self) -> usize {
        self.persistent_to_entity.len()
    }

    /// 运行时 Entity 是否已分配持久化 ID。
    pub fn contains_entity(&self, entity: &Entity) -> bool {
        self.entity_to_persistent.contains_key(entity)
    }

    /// 持久化 ID 是否已被映射（防御性检查）。
    pub fn contains_pid(&self, pid: &PersistentEntityId) -> bool {
        self.persistent_to_entity.contains_key(pid)
    }
}
