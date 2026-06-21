---
id: 04-data.foundation.migration-policy
title: Migration Policy — 数据迁移策略
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-21
layer: persistence
replay-safe: true
---

# Migration Policy — 数据迁移策略

> **总纲引用**: `docs/04-data/README.md` §6.2 — 版本迁移策略
> **本文档是数据迁移的完整策略定义**，覆盖 Content Def 迁移、Save 文件迁移、Replay 文件迁移三类场景，定义版本号方案、迁移链工程实践、回滚策略和测试规范。

---

## 1. Domain Ownership

### 1.1 归属领域

| 领域 | 迁移类型 | 职责 | 所有权 |
|------|---------|------|--------|
| **Content Platform** | Content Def 迁移 (`ContentMigration<T>`) | Def Schema 版本升级时提供迁移函数；注册到 MigrationRegistry | `@content-architect` |
| **Save System** | Save 文件迁移 (`save_version`) | 存档文件从旧格式升级到新格式；管理 Entity/Global/Domain 各 section 的版本 | `@data-architect` + `@feature-developer` |
| **Replay System** | Replay 文件迁移 (`replay_version`) | 回放文件从旧格式升级到新格式；保证回放确定性不受版本影响 | `@data-architect` |
| **各业务 Domain** | Domain SaveData 迁移 | 每个领域负责自己的 `DomainSaveData` 结构的版本演进（字段增删改） | 各 Domain Owner |

### 1.2 涉及的数据类别

```
数据迁移全景
├── Content Def (配置层)
│   ├── TagDef, EffectDef, AbilityDef, ... (每个 Def 有自己的 schema_version)
│   └── 迁移发生在注册时：旧版本 Def → 迁移 → 当前版本 Def
│
├── Save File (存档层)
│   ├── save_format_version (文件格式版本)
│   ├── EntityState (每个 Entity 的 Component 数据)
│   └── DomainStates (每个领域的持久化数据)
│
├── Replay File (回放层)
│   ├── replay_format_version (回放格式版本)
│   ├── SyncCheckpoint (同步检查点结构)
│   └── ReplayFrame (帧结构 / Command 枚举)
│
└── Migration Infrastructure
    ├── MigrationRegistry (全局迁移注册表)
    ├── MigrationChain (链式迁移执行器)
    └── BackupManager (回滚备份管理)
```

### 1.3 三层迁移的责任边界

- **Content Def 迁移**：配置文件的版本升级。旧的 RON 文件加载后立即迁移到当前版本，对运行时透明。
- **Save 文件迁移**：存档的格式升级。读档时按 `save_format_version` 逐级迁移到当前版本，涉及 Entity ID 重映射、Domain 数据结构变更。
- **Replay 文件迁移**：回放文件的格式升级。回放时按 `replay_format_version` 迁移，确保旧回放在新版引擎上可执行。

---

## 2. Problem

### 2.1 核心问题

项目生命周期内，数据 Schema 必然经历数百次变更。每次变更是对兼容性的挑战：

| 问题 | 描述 | 影响 |
|------|------|------|
| **Schema 演进** | 新增字段、修改类型、重命名字段、删除字段 | 旧存档/旧配置无法被新版读取 |
| **多版本并存** | 玩家存档可能跨越多个大版本（如 v1.0 → v3.0） | 需要链式迁移而非两两转换矩阵 |
| **领域独立演进** | 15 个业务领域各自独立修改 Schema | 迁移不可集中在单一点，需分布式 |
| **Content Def 版本化** | 每种 Def（AbilityDef / EffectDef 等）独立版本号 | 迁移逻辑分散，需要统一注册机制 |
| **大版本升级** | 1.x → 2.x 可能包含架构级重构 | 迁移路径可能断裂，需要专用迁移工具 |
| **回放兼容** | 旧回放文件需要在新版本引擎上精确复现 | 回放数据格式变更可能导致 desync |
| **Mod 兼容** | Mod 可能包含自定义 Def 和存档数据 | 迁移框架必须支持第三方扩展 |

### 2.2 约束条件

```
1. 迁移不可逆 — 一旦升级到新版本，不可自动降级
2. 迁移不可跳过 — 必须逐版本迁移，禁止从 v1 直接跳到 v3
3. 迁移必须幂等 — 多次执行同一迁移应产生相同结果
4. 迁移失败必须可回滚 — 原始数据必须完整保留
5. 迁移必须是纯函数 — 不依赖外部状态、系统时间、RNG
```

### 2.3 非目标

- 不处理数据库级别的迁移（本游戏不使用外部数据库）
- 不处理网络同步的数据迁移（使用状态同步而非数据同步）
- 不处理跨平台的存档格式转换（各平台统一使用 MessagePack）

---

## 3. Migration Strategy

### 3.1 链式增量迁移（Chain-based Incremental Migration）

三种迁移类型统一采用相同的链式模式，只是迁移目标和粒度不同：

```
v0 ──→ Migration_v0→v1 ──→ v1 ──→ Migration_v1→v2 ──→ v2 ──→ ... ──→ vCurrent

每个 Migration 是独立可测试的纯函数：
  fn migrate(input: Data) -> Result<Data, MigrationError>
```

**核心规则**：

| 规则 | 说明 |
|------|------|
| 不可跳过中间版本 | 禁止 `v0 → v2` 的直接迁移跳板 |
| 不可合并迁移 | 每个版本号对应一个迁移函数，未来不合并 |
| 迁移方向唯一 | 只支持 `vN → vN+1` 正向迁移，不支持反向降级 |
| 迁移幂等 | 对已迁移的数据再次执行迁移，结果不变 |
| 迁移无副作用 | 迁移不写文件、不修改外部状态、不发送网络请求 |

### 3.2 Content Def 迁移策略

#### 3.2.1 迁移时机

Def 加载管线如下：

```
读入 RON → 反序列化为 OldDef → 检查 schema_version
  ├── 匹配当前版本 → 直接使用
  └── 不匹配（旧版本）→ 查找 ContentMigration<T> 执行迁移
       ├── 迁移成功 → 使用 NewDef
       └── 迁移失败 → 拒绝该 Def，记录错误
```

#### 3.2.2 迁移注册

```rust
// 每个 Def 类型实现 ContentMigration trait（定义于 Content Platform）
pub trait ContentMigration<T: ContentDef> {
    fn from_version(&self) -> u32;
    fn to_version(&self) -> u32;
    fn migrate(&self, def: T) -> Result<T, MigrationError>;
}

// 注册示例（在 Content Plugin 中）
registry.register_migration(EffectDefV2toV3);
registry.register_migration(AbilityDefV1toV2);
```

#### 3.2.3 迁移类型示例

| Def 类型 | 迁移 | 说明 |
|---------|------|------|
| EffectDef v1→v2 | `damage_type: String` → `damage_type: DamageType` 枚举 | 字符串改为强类型枚举 |
| EffectDef v2→v3 | 新增 `conditions: Vec<ConditionDef>` 字段 | 条件系统接入（默认空列表） |
| AbilityDef v1→v2 | 新增 `cooldown_group: Option<CooldownGroupId>` | 冷却分组支持 |
| CharacterDef v1→v2 | `attributes: Vec<AttributeMod>` → `attributes: HashMap<AttributeId, i32>` | 更高效的属性存储 |

#### 3.2.4 Content Def 版本兼容原则

```
加字段: ✅ 向前兼容（须有默认值）
删字段: ❌ 破坏性变更（必须经 deprecation 周期）
改类型: ❌ 破坏性变更（必须写新 Migration）
改语义: ❌ 破坏性变更（必须更新文档 + 迁移）
```

### 3.3 Save 文件迁移策略

#### 3.3.1 完整加载 + 迁移流程

```
开始加载存档
  │
  ├── 1. 读取 Magic Header → 验证 magic bytes
  ├── 2. 读取 File Header → 获取 save_format_version
  ├── 3. 读取 Body → 验证 SHA-256 checksum
  │
  ├── 4. 检查版本号
  │      ├── save_format_version == CURRENT_VERSION → 直接反序列化
  │      └── save_format_version < CURRENT_VERSION
  │           ├── 执行链式迁移 (v → v+1 → ... → vCurrent)
  │           ├── 迁移后重新计算 checksum
  │           └── 反序列化
  │
  ├── 5. 执行 Entity ID 重映射（persistent_id → runtime Entity）
  ├── 6. 执行各 Domain 的领域级校验（引用完整性、值范围）
  └── 7. 完成加载
```

#### 3.3.2 迁移注册

```rust
// Save 迁移器注册（在 save 模块的 plugin.rs 中）
pub fn register_save_migrations(app: &mut App) {
    let mut registry = SaveMigrationRegistry::default();
    registry.push(SaveV1toV2);
    registry.push(SaveV2toV3);
    registry.push(SaveV3toV4);
    app.insert_resource(registry);
}
```

#### 3.3.3 Save 迁移类型示例

| 迁移 | 变更内容 | 数据影响 |
|------|---------|---------|
| v1→v2 | 新增 `FactionState.reputation` 字段 | 各 Entity 的 Faction Component 新增字段，默认 Neutral |
| v2→v3 | `ItemDef.max_stack` 从 `u8`(255) 改为 `u32`(无上限) | 字段类型扩大，无损转换 |
| v3→v4 | 拆分 `CombatState.entity_states` 为 `CombatParticipant` 列表 | 结构性重构，需要遍历每个 CombatState 重建 |
| v4→v5 | 新增 `CraftingState.in_progress_crafts: Vec<CraftProgress>` | 加字段，默认空列表 |
| v5→v6 | `EntityRemapper` 引入 UUID 类型持久化 ID | 所有 Entity 引用需要重新映射 |

### 3.4 Replay 文件迁移策略

#### 3.4.1 迁移流程

```
加载 Replay 文件
  │
  ├── 1. 读取 Magic Header → 验证
  ├── 2. 读取 Header → 获取 replay_format_version
  │
  ├── 3. 版本检查
  │      ├── replay_format_version == CURRENT → 直接播放
  │      └── replay_format_version < CURRENT
  │           ├── 执行链式迁移（replay_version 链）
  │           └── 迁移过程中保持所有 Command 语义不变
  │
  ├── 4. 开始确定性回放
  └── 5. 每帧执行 SyncCheckpoint 校验
```

#### 3.4.2 Replay 迁移的特殊约束

| 约束 | 说明 |
|------|------|
| **语义保持** | 迁移后 Command 的含义必须与录制时完全相同 |
| **确定性不受影响** | 迁移本身必须是确定性纯函数，不影响 RNG 序列 |
| **帧结构不变性** | `frame_number` / `rng_seed` / `rng_consumed` 字段不得修改 |
| **向后兼容优先** | Replay 迁移优先于功能演进——旧回放必须永远可播放 |

#### 3.4.3 Replay 迁移类型示例

| 迁移 | 变更内容 | 说明 |
|------|---------|------|
| v1→v2 | `Command::UseAbility` 新增 `spec_overrides` 字段 | 新增字段，旧回放使用空列表 |
| v2→v3 | `AIDecision` 新增 `reasoning` 字段 | 可选字段，旧回放使用 None |
| v3→v4 | `TargetingSnapshot` 新增 `secondary_targets` 字段 | 新增字段，旧回放使用空列表 |

### 3.5 N-2 兼容策略

存档和回放系统保证至少支持当前版本及之前 2 个大版本的迁移：

```
当前版本: v5
支持的旧版本: v3, v4, v5
不支持的旧版本: v2 及更早 → 拒绝加载，提示"存档版本太旧，请使用中间版本升级"
```

**迁移链维护规则**：
- 每次发布新版本时，如果 N-2 范围内的旧版本仍有迁移链，**必须在发布前验证所有迁移路径**
- 版本号达到 N-2 边界时（如 v5 发布时，v2 即将被淘汰），在 Changelog 中公告废弃
- 废弃版本由显式的 `VersionDeprecation` 事件标记，不在运行时静默跳过

---

## 4. Versioning Scheme

### 4.1 版本号格式

三种数据使用独立的版本号线路：

| 数据类型 | 版本号类型 | 格式 | 起始值 | 位置 |
|---------|-----------|------|-------|------|
| Save 文件版本 | `u32` 单调递增 | `save_format_version = N` | `1` | File Header |
| Replay 文件版本 | `u32` 单调递增 | `replay_format_version = N` | `1` | Header |
| Content Def 版本 | `u32` 单调递增 | `schema_version = N` | `1` | Def 定义内字段 |

**规则**：
- 版本号从 1 开始连续递增，不留空号
- 版本号永不回退（禁止降级）
- 版本号在发布版本中不可修改（开发期可重置，但不建议）

### 4.2 当前版本常量

```rust
/// Save 格式当前版本 — 在 save/src/constants.rs 中定义
pub const CURRENT_SAVE_FORMAT_VERSION: u32 = 1;

/// Replay 格式当前版本 — 在 replay/src/constants.rs 中定义
pub const CURRENT_REPLAY_FORMAT_VERSION: u32 = 1;

/// 编译时最高支持版本
pub const MAX_SUPPORTED_SAVE_VERSION: u32 = 1;
pub const MIN_SUPPORTED_SAVE_VERSION: u32 = 1; // N-2 策略
```

### 4.3 兼容矩阵

#### 4.3.1 Save 兼容矩阵

| 存档版本 \ 游戏版本 | v1.0 | v2.0 | v3.0 |
|--------------------|------|------|------|
| save_ver=1 | ✅ 直接加载 | ✅ v1→v2 迁移 | ❌ 需先升级到 v2.0 |
| save_ver=2 | — | ✅ 直接加载 | ✅ v2→v3 迁移 |
| save_ver=3 | — | — | ✅ 直接加载 |

#### 4.3.2 Content Def 兼容矩阵

以 EffectDef 为例：

| Def schema_version | 当前游戏 | 迁移状态 |
|-------------------|---------|---------|
| `v1` | ❌ 不支持 | 已废弃（2026-06-01） |
| `v2` | ✅ 支持 | 有 v2→v3 迁移 |
| `v3` | ✅ 支持 | 当前版本 |

#### 4.3.3 版本废弃生命周期

```
v1 ──── 当前版本 ──── 支持 ──── 公告废弃 ──── 正式废弃
  ↑       ↑               ↑               ↑
  发布    N 版本         N+1 版本        N+2 版本
        (新版本发布)   (废弃公告)      (拒绝加载)
```

具体周期：
- 版本 `N` 发布 → 版本 `N+1` 发布：对版本 `N` 的迁移支持
- 版本 `N+1` 发布时公告版本 `N` 将在下一版本废弃
- 版本 `N+2` 发布时，版本 `N` 的存档被拒绝加载

### 4.4 版本号变更流程

```
开发新 Schema 变更
  │
  ├── 是否只是加字段（带默认值）？
  │      ├── 是 → 版本号不升级（向前兼容）
  │      └── 否 → 继续
  │
  ├── 是否需要改变现有字段类型/语义/删除字段？
  │      ├── 是 → 升级版本号 (vN → vN+1)
  │      │         ├── 编写 Migration N→N+1
  │      │         ├── 单元测试迁移逻辑
  │      │         └── 集成测试：从 vN 加载 + 迁移 + 验证
  │      └── 否 → 继续
  │
  └── 是否需要修改 ID 策略 / Entity 结构？
         ├── 是 → 升级版本号，可能需要 N-2 淘汰通知
         └── 否 → 无需变更
```

---

## 5. Rollback Policy

### 5.1 迁移失败回滚原则

```
迁移失败
  │
  ├── 1. 立即中止迁移，不执行后续操作
  ├── 2. 保留原始数据不变（不删除、不覆盖、不修改）
  ├── 3. 记录详细错误日志：
  │      ├── 失败类型（Content / Save / Replay）
  │      ├── 源版本号 / 目标版本号
  │      ├── 迁移步骤 ID
  │      ├── 异常堆栈
  │      └── 数据位置（文件路径 / Def ID）
  └── 4. 向用户显示友好的错误提示 + 错误报告选项
```

### 5.2 各类数据的回滚策略

#### 5.2.1 Save 文件回滚

| 场景 | 回滚方式 | 说明 |
|------|---------|------|
| 迁移过程中程序崩溃 | 原子写入保证 | 存档先写临时文件 `.tmp`，成功后才 `rename`；崩溃不影响原始文件 |
| 迁移后数据校验失败 | 保留 `.bak` 备份 | 写入前自动创建 `.bak`，校验失败时恢复 |
| 迁移成功但游戏内出现异常 | 手动回滚 | 玩家可通过 "恢复备份" 功能从 `.bak` 文件还原 |
| 新版本 Bug 导致存档损坏 | 版本降级工具 | 提供独立 CLI 工具用于回退存档版本（仅限开发/支持团队） |

#### 5.2.2 Content Def 回滚

| 场景 | 回滚方式 |
|------|---------|
| 单个 Def 迁移失败 | 拒绝该 Def，不影响其他 Def 加载 |
| 批量 Def 迁移失败 | 拒绝整个 Content Package，使用前一次有效加载的 Def 集合 |
| Def 迁移后引用了不存在的 ID | 迁移后校验失败 → 回退到迁移前状态 |

#### 5.2.3 Replay 文件回滚

| 场景 | 回滚方式 |
|------|---------|
| Replay 迁移失败 | 拒绝回放，提示文件损坏或不兼容 |
| Replay 迁移后 desync | 允许用户切换为 "尽力回放" 模式（跳过校验，不保证精确） |

### 5.3 回滚代码示例

```rust
/// Save 加载 + 迁移 + 回滚的完整流程
pub fn load_save_with_rollback(
    path: &Path,
    registry: &SaveMigrationRegistry,
    current_version: u32,
) -> Result<SaveGame, SaveError> {
    // 1. 读取原始数据
    let mut data = std::fs::read(path).map_err(SaveError::Io)?;
    let backup = data.clone(); // ← 回滚点

    // 2. 解析 Header 获取版本号
    let (header, _) = parse_header(&data)?;

    // 3. 执行迁移（仅在需要时）
    if header.save_format_version < current_version {
        match migrate_save(&mut data, header.save_format_version, current_version, registry) {
            Ok(()) => {},
            Err(e) => {
                // 迁移失败：保留原始数据，报告错误
                data = backup; // ← 回滚
                log::error!("Save migration failed: {:?}", e);
                return Err(SaveError::MigrationFailed {
                    from: header.save_format_version,
                    to: current_version,
                    reason: e.to_string(),
                });
            }
        }
    }

    // 4. 校验 + 反序列化
    validate_checksum(&data, &header)?;
    let save_game = deserialize_save(&data)?;
    Ok(save_game)
}
```

### 5.4 原子写入实现

```rust
/// 保存时的原子写入流程
pub fn atomic_save(path: &Path, data: &[u8]) -> Result<(), SaveError> {
    let tmp_path = path.with_extension("fresave.tmp");
    let bak_path = path.with_extension("fresave.bak");

    // 0. 如果已有存档，创建备份
    if path.exists() {
        std::fs::rename(path, &bak_path).ok(); // 忽略备份失败
    }

    // 1. 写入临时文件
    std::fs::write(&tmp_path, data).map_err(SaveError::Io)?;

    // 2. fsync 确保写入磁盘
    let file = std::fs::File::open(&tmp_path)?;
    file.sync_all().map_err(SaveError::Io)?;
    drop(file);

    // 3. 原子 rename（tmp → 目标）
    std::fs::rename(&tmp_path, path).map_err(SaveError::Io)?;

    // 4. fsync 目录确保 rename 持久化
    if let Some(parent) = path.parent() {
        let dir = std::fs::File::open(parent)?;
        dir.sync_all().ok();
    }

    Ok(())
}
```

### 5.5 回滚禁止事项

```
❌ 迁移失败后自动重试（除非有明确的修复补丁）
❌ 迁移失败后降级到旧版本继续（跳过迁移）
❌ 迁移过程中修改原始文件（先写临时文件，成功才替换）
❌ 迁移后删除备份文件（保留至少一个版本周期的备份）
```

---

## 6. Testing Requirements

### 6.1 迁移测试金字塔

```
          ┌──────────────────────────┐
          │   Performance / Stress   │  ← 极少量
          │  (10MB 存档迁移 < 1s)    │
          ├──────────────────────────┤
          │    Integration Tests     │  ← 每个迁移 1-2 个
          │  (加载旧存档 → 验证结果)  │
          ├──────────────────────────┤
          │    Round-trip Tests      │  ← 每个迁移 1 个
          │ (vN → vN+1 → 验证字段)   │
          ├──────────────────────────┤
          │   Unit Tests (每个迁移)   │  ← 大量
          │  (纯函数测试迁移逻辑)      │
          ├──────────────────────────┤
          │    Fuzz / Edge Tests     │  ← 每个迁移 1 组
          │  (空数据、边界值、损坏数据) │
          └──────────────────────────┘
```

### 6.2 测试类型细则

#### 6.2.1 Unit Test — 每个迁移独立测试

```rust
#[test]
fn test_save_v1_to_v2_migration() {
    // Given: v1 格式的数据
    let v1_data = create_sample_v1_save();

    // When: 执行迁移
    let migrator = SaveV1toV2;
    let v2_data = migrator.migrate(v1_data).unwrap();

    // Then: 验证 v2 格式
    let v2_save = deserialize::<SaveV2>(&v2_data);
    assert_eq!(v2_save.header.save_format_version, 2);
    // v1→v2 新增的字段应有默认值
    assert!(v2_save.faction_state.is_some());
}
```

**测试清单**：

| 测试项 | 验证内容 |
|--------|---------|
| 正常数据迁移 | 标准数据正确转换 |
| 空数据 | 空 Entity 列表/空 Domain 状态不崩溃 |
| 极限值 | u8::MAX → u32 转换不溢出 |
| 缺失字段 | 旧版本不存在的字段在新版本中得到正确默认值 |
| 特殊字符 | String 字段含 Unicode/转义符时正确迁移 |

#### 6.2.2 Round-trip Test — 迁移可逆验证

虽然迁移不可逆（不支持自动降级），但必须验证迁移结果的**一致性**：

```rust
#[test]
fn test_save_v2_to_v3_roundtrip() {
    // Given
    let mut v2_data = create_sample_v2_save();
    let original_checksum = compute_checksum(&v2_data);

    // When: 执行迁移 v2→v3
    let migrator = SaveV2toV3;
    let v3_data = migrator.migrate(v2_data.clone()).unwrap();

    // Then: 验证数据完整性
    // (1) v2 中的字段在 v3 中保留原值
    let v3_save = deserialize::<SaveV3>(&v3_data);
    assert_eq!(v3_save.global_state.game_time, compute_expected_game_time());

    // (2) 迁移幂等性：再次执行相同迁移，结果不变
    let v3_data_again = migrator.migrate(v3_data.clone()).unwrap();
    assert_eq!(v3_data, v3_data_again);
}
```

#### 6.2.3 Integration Test — 加载旧存档验证

```
测试场景：
  1. 准备一个已知内容的 v1 存档（golden file）
  2. 使用当前版本的游戏加载该存档
  3. 验证所有数据正确迁移并加载
  4. 验证游戏可以正常操作（移动、战斗、保存）

Golden file 管理：
  每个版本保留一个 golden save file，存放在 tests/fixtures/saves/ 目录中
  格式: save_v{N}_{description}.fresave
  示例: save_v1_pre_faction_state.fresave
```

**Golden file 清单**：

```
tests/fixtures/saves/
├── save_v1_minimal.fresave          # 最小存档（仅必要数据）
├── save_v1_full.fresave             # 完整存档（所有领域数据）
├── save_v2_full.fresave             # v2 格式完整存档
├── save_v1_corrupted.fresave        # 损坏存档（校验迁移失败处理）
└── README.md                        # 各 golden file 的创建说明
```

#### 6.2.4 Fuzz Test — 边界和异常数据

```rust
#[test]
fn test_save_v1_to_v2_fuzz() {
    // 边界值测试
    let edge_cases = vec![
        ("empty", vec![]),
        ("max_entities", vec![0u8; 1_000_000]), // 大量 Entity
        ("negative_values", /* ... */),
        ("corrupted_header", /* ... */),
    ];

    for (name, data) in edge_cases {
        let migrator = SaveV1toV2;
        // 不 panic，优雅地返回错误
        let result = migrator.migrate(data);
        assert!(result.is_err(), "Fuzz case '{}' should fail", name);
    }
}
```

#### 6.2.5 Performance Test — 迁移性能基准

```rust
#[test]
fn test_save_v1_to_current_performance() {
    // 创建模拟的 10MB 存档
    let v1_data = generate_large_v1_save(10 * 1024 * 1024);

    let start = std::time::Instant::now();
    let chain = build_full_migration_chain();
    let result = chain.migrate_to_current(v1_data, 1).unwrap();
    let elapsed = start.elapsed();

    // 迁移必须在 1000ms 内完成
    assert!(
        elapsed.as_millis() < 1000,
        "Migration of 10MB save took {}ms (limit: 1000ms)",
        elapsed.as_millis()
    );
}
```

**性能基准**：

| 存档大小 | 迁移版本跨度 | 最大耗时 |
|---------|-------------|---------|
| 1 MB | v1→vCurrent | 200ms |
| 10 MB | v1→vCurrent | 1,000ms |
| 100 MB | v1→vCurrent | 5,000ms |

### 6.3 测试数据管理

所有迁移测试的**旧版本数据**必须通过代码生成，而非手动构造：

```rust
/// 测试助手：生成 v1 格式的 SaveGame
fn create_sample_v1_save() -> Vec<u8> {
    let save_v1 = SaveGameV1 {
        header: SaveHeaderV1 {
            save_version: 1,
            game_version: "1.0.0".to_string(),
            // ...
        },
        entities: vec![],
        globals: GlobalStateV1::default(),
    };
    serialize(&save_v1)
}
```

禁止将序列化的二进制文件（`.bin`/`.fresave`）提交到版本库，除非是 golden file。

### 6.4 CI 集成规则

```
CI 迁移测试套件：
  ├── 单元测试 — 每次提交执行
  ├── 集成测试（Golden file 验证）— 每次提交执行
  ├── 性能测试 — 每日定时执行（标记为 #[bench]）
  └── Fuzz 测试 — 每周定时执行

当新版本发布时：
  ├── 1. 在 CI 中生成当版本 golden file
  ├── 2. 执行所有跨版本迁移测试（v1→vCurrent, v2→vCurrent, ...）
  ├── 3. 如果任何迁移路径断裂 → 阻塞发布
  └── 4. 废弃版本号在 Changelog 中公告
```

---

## 7. Risks

### 7.1 风险矩阵

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| 迁移链过长（10+ 版本后性能下降） | 中 | 中 | 定期压缩存档格式；N-2 废弃策略自动淘汰旧版本 |
| 迁移过程中数据丢失 | 低 | 高 | 原子写入 + .bak 备份；每个迁移必须通过 Round-trip 测试 |
| 跨领域迁移协调失败 | 中 | 中 | Save 迁移按领域 section 独立执行；Domain 各自独立版本号 |
| 开发期忘记注册迁移 | 中 | 高 | CI 检测：当前版本号 != 注册的迁移数量-1 时告警 |
| 旧回放文件因引擎改动无法播放 | 低 | 中 | Replay 迁移保持 Command 语义；回放使用尽力模式回退 |
| Mod 开发者未提供迁移 | 中 | 低 | Mod 迁移可选；加载时告警而非拒绝 |
| 多个大版本连续发布导致迁移路径断裂 | 低 | 高 | 发布前必须通过所有 N-2 范围的迁移集成测试 |
| 浮点精度变化导致回放迁移后 desync | 低 | 中 | 回放使用整型计算；浮点值 never 出现在回放 key 路径 |

### 7.2 缓解措施优先级

```
高风险（立刻行动）
  ├── 每个迁移必须通过 Round-trip + 集成测试
  ├── CI 自动检测未注册的迁移
  └── 原子写入保证存档不损坏

中风险（持续关注）
  ├── N-2 版本废弃策略
  ├── 性能基准测试（限制迁移耗时）
  └── 跨版本 golden file 自动生成

低风险（文档记录）
  ├── Mod 迁移指南
  ├── 开发期迁移调试工具
  └── 回放尽力模式
```

---

## 附录 A：迁移开发 Checklist

每个涉及 Schema 变更的 PR 必须逐项检查：

- [ ] **变更类型确认**：新增字段（向前兼容）或破坏性变更（需升级版本号）
- [ ] **版本号升级**：如果破坏性变更，`schema_version` / `save_format_version` / `replay_format_version` 已升级
- [ ] **迁移函数**：已编写对应的 `vN→vN+1` 迁移函数并注册
- [ ] **Unit Test**：迁移函数有完整的单元测试（正常 + 边界 + 异常）
- [ ] **Round-trip Test**：幂等性验证通过
- [ ] **Golden File**：旧版本 golden file 已更新或创建
- [ ] **Integration Test**：从旧版本加载并迁移后数据正确
- [ ] **Rollback 验证**：迁移失败时原始数据被保留
- [ ] **性能验证**：迁移性能在基准范围内
- [ ] **文档更新**：Changelog 标记版本变更；本文档的兼容矩阵更新

## 附录 B：术语表

| 术语 | 定义 |
|------|------|
| Migration | 将数据从旧 Schema 版本转换为新 Schema 版本的纯函数 |
| Migration Chain | 按版本号串联的一系列 Migration |
| Migration Registry | 全局迁移注册表，管理所有已注册的 Migration |
| Content Migration | Content Def 的版本迁移（由 `ContentMigration<T>` trait 定义） |
| Save Migration | 存档文件的版本迁移 |
| Replay Migration | 回放文件的版本迁移 |
| Golden File | 已知内容的旧版本存档，用于集成测试 |
| N-2 Policy | 支持当前版本及之前 2 个大版本的迁移 |
| Atomic Write | 先写临时文件再 rename 的写入策略，保证写入原子性 |
| Rollback | 迁移失败时恢复到迁移前状态 |
| Deprecation Cycle | 版本废弃前的公告期（至少一个大版本周期） |
| Round-trip Test | 验证迁移幂等性和数据完整性的测试 |
| Compatibility Matrix | 各版本之间的兼容关系表 |
