---
id: foundation.save-architecture.v1
title: Save Architecture Deep Dive — 存档架构详述
status: draft
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: persistence
replay-safe: false
---

# Save Architecture — 存档架构详述

> **总纲引用**: `docs/04-data/README.md` §6 — Save 架构
> **本文档是存档架构的深度展开**，覆盖文件格式、序列化策略、版本迁移、校验恢复和增量存档。

---

## 1. 存档分层结构

### 1.1 完整 Save File 布局

```
Save File (二进制, 扩展名 .fresave)
┌────────────────────────────────────────────────────────────┐
│ Magic Header (8 bytes)                                     │
│   magic: [0x46, 0x52, 0x45, 0x53, 0x41, 0x56, 0x45, 0x00] │
│   → "FRESAVE\0"                                            │
├────────────────────────────────────────────────────────────┤
│ File Header (variable)                                     │
│   save_format_version: u32     # 存档格式版本号             │
│   game_version: String         # 创建存档的游戏版本         │
│   timestamp: u64               # 创建时间戳（仅显示用途）   │
│   checksum: [u8; 32]           # SHA-256 (header + body)   │
│   compression: u8              # 0=none, 1=zstd, 2=lz4     │
│   body_size: u64               # 压缩后的 body 大小         │
├────────────────────────────────────────────────────────────┤
│ Body (compressed)                                           │
│   ├── Metadata (JSON)                                       │
│   │   ├── player_name: String                               │
│   │   ├── playtime_seconds: u64                             │
│   │   ├── location: String                                  │
│   │   ├── level: u32                                        │
│   │   └── screenshot: Option<[u8]>  # 缩略图               │
│   │                                                         │
│   ├── GlobalState (MessagePack)                             │
│   │   ├── game_time: GameTime                               │
│   │   ├── story_flags: Vec<StoryFlag>                       │
│   │   └── world_state: WorldState                           │
│   │                                                         │
│   ├── EntityStates (MessagePack, 差分编码)                  │
│   │   ├── party: Vec<EntityState>                           │
│   │   ├── npcs: Vec<EntityState>                            │
│   │   ├── world_objects: Vec<EntityState>                   │
│   │   └── deleted_entities: Vec<EntityId>  # 增量标记       │
│   │                                                         │
│   ├── DomainStates (MessagePack)                            │
│   │   ├── progression: ProgressionState                     │
│   │   ├── quest_log: QuestLog                               │
│   │   ├── inventory: InventoryState                         │
│   │   ├── economy: EconomyState                             │
│   │   ├── faction: FactionState                             │
│   │   ├── narrative: NarrativeState                         │
│   │   ├── party: PartyState                                 │
│   │   ├── camp_rest: CampRestState                          │
│   │   ├── crafting: CraftingState                           │
│   │   ├── terrain: Option<TerrainState>                     │
│   │   └── combat: Option<CombatSnapshot>                    │
│   │                                                         │
│   └── Signature (optional)                                  │
│       └── ed25519_signature: [u8; 64]  # 防止篡改           │
└────────────────────────────────────────────────────────────┘
```

### 1.2 序列化格式选择

| 部分 | 格式 | 理由 |
|------|------|------|
| Magic Header | 固定 8 字节 | 快速辨识文件类型 |
| File Header | 二进制 (bincode) | 固定偏移，随机访问 |
| Metadata | JSON | 人类可读、工具友好 |
| Global / Entity / Domain | MessagePack | 紧凑、Schema 灵活、跨语言 |
| Signature | ed25519 二进制 | 固定 64 字节，验证快速 |

---

## 2. 序列化策略

### 2.1 Entity 序列化

不是所有 ECS Component 都持久化。每个 Component 显式标记是否参与存档：

```rust
/// 标记该 Component 需要存档
trait Persistable: Serialize + Deserialize {
    type SaveData: Serialize + Deserialize;

    /// Component → 存档数据
    fn to_save_data(&self) -> Self::SaveData;

    /// 存档数据 → Component
    fn from_save_data(data: &Self::SaveData) -> Self;
}
```

只持久化实现了 `Persistable` 的 Component。瞬时 Component（如 `CombatIntent`、`DamageResult`）不参与。

### 2.2 差分编码（Delta Encoding）

为避免每次全量保存所有 Entity，采用差分策略：

```
初始存档：
  EntityState { id: A, components: [HP(100), Position(0,0)] }

后续存档：
  Delta { id: A, changed: [HP(75)], removed: [] }
  未列出的字段 (= Position) → 使用前一次值
```

规则：
- **初始存档**：保存所有 Entity 的完整状态
- **增量存档**：只保存自上次存档以来发生变化和新增的 Entity
- **删除标记**：自上次存档以来被销毁的 Entity 以 ID 列表记录
- **压缩**：定期执行全量重写（防碎片化）

### 2.3 写入策略

| 触发时机 | 类型 | 说明 |
|---------|------|------|
| 玩家手动存档 | 全量 + 增量可选 | 默认全量 |
| 自动存档（进入新区域） | 增量 | 快速写入 |
| 自动存档（战斗前） | 增量 | 用于战斗失败回退 |
| 自动存档（长休时） | 增量 | 防止长休中断丢失进度 |
| 自动存档（关闭游戏） | 全量 | 确保完整退出 |

---

## 3. 版本迁移

### 3.1 迁移链架构

```
存档版本 1 ──→ 迁移器 v1→v2 ──→ 存档版本 2 ──→ 迁移器 v2→v3 ──→ ... ──→ 当前版本
```

每个迁移器是独立可测试的纯函数：

```rust
trait Migration {
    /// 源版本号
    fn from_version(&self) -> u32;

    /// 目标版本号
    fn to_version(&self) -> u32;

    /// 执行迁移（返回新存档数据）
    fn migrate(&self, data: &[u8]) -> Result<Vec<u8>, MigrationError>;
}
```

### 3.2 迁移注册

```rust
/// 全局迁移注册表。按 (from, to) 顺序排列。
struct MigrationRegistry {
    migrations: Vec<Box<dyn Migration>>,
}

impl MigrationRegistry {
    /// 将存档从 old_version 逐级迁移到 current_version
    fn migrate_to_current(&self, data: &[u8], old_version: u32) -> Result<Vec<u8>> {
        let mut current = old_version;
        let mut result = data.to_vec();
        while current < CURRENT_VERSION {
            let migrator = self.migrations.iter()
                .find(|m| m.from_version() == current)
                .ok_or(MigrationError::MissingMigration(current))?;
            result = migrator.migrate(&result)?;
            current = migrator.to_version();
        }
        Ok(result)
    }
}
```

### 3.3 迁移类型示例

| 迁移 | 说明 |
|------|------|
| v1 → v2 | 新增 `FactionState.reputation` 字段（默认值 Neutral） |
| v2 → v3 | `ItemDef.max_stack` 类型从 u8 改为 u32 |
| v3 → v4 | 拆分 `CombatState.entity_states` 为独立的 `CombatParticipant` 列表 |
| v4 → v5 | 新增 `CraftingState.in_progress_crafts` 字段 |

### 3.4 迁移失败处理

```
迁移失败
    │
    ├── 1. 保留原始存档不变
    ├── 2. 记录错误日志（存档路径、版本号、迁移步骤、异常信息）
    └── 3. 向玩家显示 "存档加载失败，原始存档未修改"（含错误报告选项）
```

禁止：部分迁移、静默跳过、修改原始存档后失败。

---

## 4. 完整性校验

### 4.1 写入时

```
1. 序列化所有数据 → Body
2. 计算 Body 的 SHA-256 → checksum
3. 写入 Header (含 checksum)
4. 写入 Body
5. 写入 Signature (可选)
6. 文件 fsync 确保写入磁盘
```

### 4.2 加载时

```
1. 读取 Magic Header → 验证 magic bytes
2. 读取 File Header
3. 读取 Body
4. 解压 Body
5. 验证 SHA-256 checksum
6. 版本号检查 → 如果需要迁移，执行迁移链
7. 迁移后重新计算 checksum 并验证
8. 反序列化各 section
9. 执行领域级校验规则（引用完整性、值范围检查）
```

### 4.3 校验失败处理

| 失败类型 | 处理 |
|---------|------|
| Magic header 不匹配 | 拒绝加载，提示"不是有效的存档文件" |
| Checksum 不匹配 | 尝试从备份恢复；无备份时拒绝加载 |
| 版本号不识别（太新） | 拒绝加载，提示"需要更高版本的游戏" |
| 版本号不识别（太旧） | 尝试链式迁移；缺少迁移器时拒绝加载 |
| 反序列化失败 | 报告字段错误，拒绝加载 |
| 引用完整性失败 | 报告 dangling reference 具体列表，拒绝加载 |

---

## 5. 存档文件管理

### 5.1 文件命名

```
<slot>_<timestamp>.<ext>
```

| 示例 | 含义 |
|------|------|
| `save_00_20260616_123045.fresave` | 槽位 0 手动存档 |
| `autosave_20260616_120000.fresave` | 自动存档 |
| `quicksave.fresave` | 快速存档（单槽覆盖） |
| `backup_save_00_20260616_123045.fresave.bak` | 写入前备份 |

### 5.2 保存策略

- **写前备份**：覆写已有存档前，先将旧文件重命名为 `.bak`
- **原子写入**：先写入临时文件，成功后 `rename` 到目标路径
- **写后验证**：写入完成后读取验证 checksum
- **自动清理**：自动存档保留最近 N 个（默认 5），旧存档自动删除

### 5.3 存档目录结构

```
saves/
├── save_00_20260616_123045.fresave
├── save_01_20260616_124500.fresave
├── save_02_20260616_130000.fresave.bak   # 写入前的备份
├── autosave_20260616_120000.fresave
├── autosave_20260616_121500.fresave
├── quicksave.fresave
├── .temp_write_abc123.fresave.tmp        # 写入中的临时文件
└── metadata.json                         # 存档槽位元数据缓存
```

---

## 6. Save 与各领域 Schema 的接口

每个领域暴露自己的 SaveData 类型，由 Save 系统统一收集：

```rust
/// Save 系统期望每个领域实现此 trait
trait DomainSave {
    type SaveData: Serialize + Deserialize;

    /// 收集需要持久化的数据
    fn capture(&self, world: &World) -> Self::SaveData;

    /// 从持久化数据恢复
    fn restore(&mut self, world: &mut World, data: &Self::SaveData);
}
```

| 领域 | SaveData 类型 | 在存档中的位置 |
|------|-------------|--------------|
| Progression | `ProgressionState` | DomainStates.progression |
| Quest | `QuestLog` | DomainStates.quest_log |
| Inventory | `InventoryState` | DomainStates.inventory |
| Economy | `EconomyState` | DomainStates.economy |
| Faction | `FactionState` | DomainStates.faction |
| Narrative | `NarrativeState` | DomainStates.narrative |
| Party | `PartyState` | DomainStates.party |
| CampRest | `CampRestState` | DomainStates.camp_rest |
| Crafting | `CraftingState` | DomainStates.crafting |
| Terrain | `Option<TerrainState>` | DomainStates.terrain |
| Combat | `Option<CombatSnapshot>` | DomainStates.combat |

---

## 7. Future Extension

- **云存档**: SaveData 包可序列化为 Base64，通过 Cloud Save API 上传
- **跨平台同步**: 统一的 SaveData 格式支持 PC/Console/Mobile 互读
- **Mod 存档隔离**: Mod 数据单独 section，卸载 Mod 时保留或安全剥离
- **压缩等级选择**: 快速存档用 LZ4（快），手动存档用 zstd（小）
- **校验和离线验证**: 外部工具可独立验证存档完整性
