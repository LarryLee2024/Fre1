---
id: 01-architecture.ADR-042
title: ADR-042 — Save & Persistence Architecture
status: approved
owner: architect
created: 2026-06-16
updated: 2026-06-16
supersedes: none
---

# ADR-042: 存档持久化策略

## 状态

**Approved** — 依赖 ADR-040（Data Ownership）和 `docs/04-data/foundation/save_architecture.md`，本架构决策正式生效。

## 背景

存档系统需要持久化所有需要保存的游戏状态，并支持版本迁移。根据 Data Law 011，所有 Persistence 层 Schema 必须带版本号并支持前向/后向兼容。

## 引用的领域规则与数据架构

- `docs/04-data/foundation/save_architecture.md` — Save 架构详述
- `docs/04-data/foundation/id_strategy.md` — ID 策略
- `docs/04-data/README.md` — Data Law 011（Schema 版本化）
- `.trae/rules/SRPG专项规则.md` §九 — 存档与兼容性

## 决策

### 1. 存档架构

```
SaveGame
├── Header
│   ├── save_version:   u32              # 存档 Schema 版本
│   ├── game_version:   String           # 创建版本
│   ├── timestamp:      u64             # 创建时间（仅显示）
│   ├── checksum:       [u8; 32]        # SHA-256 内容校验
│   └── metadata:       SaveMetadata     # 玩家可见元数据
│
├── Entities: Vec<EntityState>           # 所有持久化的 Entity
│   └── [Entity 1]
│       ├── id:          EntityId        # 持久化 ID
│       ├── components:  Vec<u8>         # Owner 序列化的 Component 数据
│       └── archetype:   ArchetypeMarker # 反序列化辅助
│
├── Globals: GlobalState                 # 全局 Resource 状态
│   ├── story:          StoryState
│   ├── party:          Party
│   ├── progression:    ProgressionState
│   ├── turn_queue:     TurnQueue
│   └── world_state:    WorldState
│
└── Registry: SaveRegistry               # ID 映射表
    ├── entity_map:     HashMap<EntityId, PersistentEntityId>
    └── def_versions:   HashMap<DefId, Version>
```

### 2. 各 Feature 的序列化责任

使用 **Per-Feature Serialization** 模式——每个 Owner Feature 负责序列化自己的状态：

```rust
/// 每个 Feature 实现此 Trait
pub trait SaveLoad {
    /// 序列化该 Feature 拥有的数据到存档
    fn save(&self, world: &World, writer: &mut SaveWriter) -> Result<(), SaveError>;

    /// 从存档反序列化并恢复状态
    fn load(&mut self, world: &mut World, reader: &mut SaveReader) -> Result<(), SaveError>;
}

/// SaveWriter — 每个 Feature 写入自己的部分
pub struct SaveWriter {
    pub entities: Vec<SerializedEntity>,
    pub globals: HashMap<TypeId, Vec<u8>>,
}

/// 存档 Plugin 负责协调各 Feature 的 SaveLoad
pub struct SavePlugin;
impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        // 注册各 Feature 的 SaveLoad 实现
        app.register_save_load::<Party>()
           .register_save_load::<StoryState>()
           .register_save_load::<QuestTracker>()
           // ...
           .add_systems(PreUpdate, auto_save_system)
           .add_systems(OnTrigger::<SaveRequest>, execute_save)
           .add_systems(OnTrigger::<LoadRequest>, execute_load);
    }
}
```

### 3. Entity ID 重映射

存档中的 Entity ID 在加载时需要重映射（因为新运行的 Entity ID 不同）：

```rust
/// 持久化 Entity ID — 存档中的稳定 ID
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct PersistentEntityId(u64);

/// Entity 映射表 — 存档 → 运行时
#[derive(Resource)]
pub struct EntityRemapper {
    /// 存档 ID → 运行时 Entity
    persistent_to_entity: HashMap<PersistentEntityId, Entity>,
    /// 运行时 Entity → 存档 ID
    entity_to_persistent: HashMap<Entity, PersistentEntityId>,
}
```

### 4. 版本迁移

```rust
/// 迁移链 — 从旧版本逐步升级到当前版本
pub struct MigrationChain {
    migrations: Vec<Box<dyn Migration>>,
}

pub trait Migration {
    fn from_version(&self) -> u32;
    fn to_version(&self) -> u32;
    fn migrate(&self, data: &mut Vec<u8>) -> Result<(), MigrationError>;
}

/// 迁移流程
fn load_and_migrate(
    mut data: Vec<u8>,
    save_version: u32,
    current_version: u32,
    chain: &MigrationChain,
) -> Result<Vec<u8>, SaveError> {
    let mut version = save_version;
    while version < current_version {
        let migration = chain.find(version, version + 1)?;
        migration.migrate(&mut data)?;
        version += 1;
    }
    Ok(data)
}
```

### 5. 自动保存策略

```rust
#[derive(Resource)]
pub struct AutoSaveConfig {
    pub enabled: bool,
    pub interval_minutes: u32,       // 定时自动保存
    pub on_battle_start: bool,       // 战斗开始时
    pub on_battle_end: bool,         // 战斗结束时
    pub on_camp_enter: bool,         // 进入营地时
    pub max_auto_saves: u32,         // 保留的自动存档数量
}
```

### 6. 存档格式

```
Save File (save_*.bin)
├── Magic: [0x46, 0x52, 0x45, 0x53]  // "FRES"
├── Header (固定大小)
│   ├── header_size: u32
│   ├── save_version: u32
│   ├── game_version_len: u16
│   ├── game_version: [u8; game_version_len]
│   ├── timestamp: u64
│   ├── metadata_size: u32
│   ├── metadata: SaveMetadata (bincode)
│   └── checksum: [u8; 32]
│
├── Entity Table
│   ├── entity_count: u32
│   └── [Entity 1..N]
│       ├── persistent_id: u64
│       ├── component_count: u8
│       └── [Component 1..M]
│           ├── type_id: u32
│           ├── data_len: u32
│           └── data: [u8; data_len]
│
├── Global Table
│   ├── global_count: u32
│   └── [Global 1..N]
│       ├── type_id: u32
│       ├── data_len: u32
│       └── data: [u8; data_len]
│
└── Footer
    └── checksum: [u8; 32]  // 重复校验
```

## Module Design

```
src/infra/save/
  ├── plugin.rs              — SavePlugin
  ├── components.rs          — (可能需要)
  ├── resources.rs           — SaveManager, AutoSaveConfig, EntityRemapper
  ├── systems.rs             — save_game, load_game, auto_save, migrate
  ├── events.rs              — SaveRequest, LoadRequest, SaveCompleted
  ├── integration/           — 跨域访问 ACL（ADR-046）
  └── internal/
      ├── format.rs          — 二进制格式定义
      ├── serializer.rs      — 序列化/反序列化
      ├── migration.rs       — 迁移链管理
      └── remapper.rs        — Entity ID 重映射
```

## Communication Design

| 通信 | 机制 | 方向 |
|------|------|------|
| 保存请求 | Event (`SaveRequest`) | 任何 → save |
| 加载请求 | Event (`LoadRequest`) | 任何 → save |
| Feature 注册 | `register_save_load()` API | save ↔ Feature |
| 自动保存触发 | Observer (`OnBattleEnd`, `OnCampEnter`) | battle/camp → save |

## 边界定义

### 允许
- Save Feature 调用各 Feature 的 `SaveLoad` 实现
- 存档包含所有持久化需要的 Entity/Resource 数据
- 加载时 EntityRemapper 重建 Entity 引用
- 版本迁移链对用户透明

### 🟥 禁止
- Save Feature 直接读取其他 Feature 的内部 Component（通过 SaveLoad trait 访问）
- 存档格式不兼容的版本加载（必须迁移）
- 保存过程中游戏继续执行业务逻辑
- 自动存档覆盖手动存档

## Forbidden

| 禁止行为 | 理由 |
|---------|------|
| 跳过迁移直接加载旧存档 | 数据不一致 |
| 保存非持久化数据（UI 状态、临时标记） | 浪费空间 |
| Entity ID 不重映射直接使用 | 运行时 ID 冲突 |
| 存档损坏后静默忽略 | 必须检测并报告 |
| 在战斗中自动保存 | 战斗状态复杂，恢复不可靠 |

## Definition / Instance Design

- **Definition**: `SaveFormat` (config: 二进制格式版本), `AutoSaveConfig` (config)
- **Instance**: `SaveManager` (Resource), `EntityRemapper` (Resource), `MigrationChain` (Resource)
- **Persistence**: 存档文件本身（.bin 格式）

## 后果

### 正面
- Per-Feature 序列化让每个 Feature 自行管理数据完整性
- Entity ID 重映射机制隔离了运行时和持久化 ID
- 链式迁移支持跨版本兼容
- 二进制格式紧凑高效

### 负面
- 每个 Feature 需要实现 SaveLoad trait（模板代码）
- 迁移链需要维护线性版本序列
- Entity 引用在跨存档时需要额外注意（EntityRemapper 仅对当前加载有效）

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| Bevy DynamicScene 直接序列化 World | 无法控制序列化内容，版本迁移困难 |
| JSON/YAML 文本格式 | 体积大，解析慢，不适合完整存档 |
| 中心化 Save Schema（所有数据在一个 struct 中） | 耦合严重，每次新增 Feature 需要修改 Save 代码 |

## 评审要点

- [ ] EntityRemapper 如何处理跨存档的 Entity 引用（如"上一个存档的任务 NPC"）？
- [ ] 自动保存的时机——是否在战斗每回合结束时保存？
- [ ] 迁移失败的回退策略——保留原始存档还是清除？
- [ ] 多存档槽位的管理策略（手动 slot / 自动 slot 分离）？
