use std::path::PathBuf;

use bevy::prelude::*;

/// 存档元数据（玩家可见）。
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Resource)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PersistentEntityId(pub u64);

/// Entity 重映射表 — 存档 ↔ 运行时的双向映射。
#[derive(Resource)]
pub struct EntityRemapper {
    pub persistent_to_entity: Vec<(PersistentEntityId, Entity)>,
    next_id: u64,
}

impl Default for EntityRemapper {
    fn default() -> Self {
        Self {
            persistent_to_entity: Vec::new(),
            next_id: 1,
        }
    }
}

impl EntityRemapper {
    pub fn assign(&mut self, entity: Entity) -> PersistentEntityId {
        let id = PersistentEntityId(self.next_id);
        self.next_id += 1;
        self.persistent_to_entity.push((id, entity));
        id
    }

    pub fn lookup(&self, pid: PersistentEntityId) -> Option<Entity> {
        self.persistent_to_entity
            .iter()
            .find(|(id, _)| *id == pid)
            .map(|(_, e)| *e)
    }

    pub fn clear(&mut self) {
        self.persistent_to_entity.clear();
        self.next_id = 1;
    }
}
