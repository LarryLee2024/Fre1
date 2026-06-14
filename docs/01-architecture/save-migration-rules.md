---
id: 01-architecture.save-migration-rules
title: Save Migration Rules
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
  - rules
---

# Save Migration Rules — 存档迁移与版本兼容

Version: 1.1
Status: Proposed

来源：`docs/其他/31遗漏.md` 第三节 — 存档迁移与版本兼容

本文档定义存档文件的版本管理、向前兼容策略和数据迁移规范。

### 宪法条款映射

| 本文档规则 | 宪法条款 | 强制等级 |
|-----------|---------|---------|
| §1.2 版本号位置 | 🟥 12.6.1 强制版本字段 | 必须遵循 |
| §2.1 向前兼容原则 | 🟥 12.6.2 向后兼容原则 | 必须遵循 |
| §3.1 最低支持版本 | 🟥 12.6.2 向后兼容原则 | 必须遵循 |
| §6.1 只保存 Instance | 🟥 1.1.2 定义与实例强制分离 | 必须遵循 |
| §8 禁止事项 | 🟥 12.5.1 三步删除原则 | 必须遵循 |
| §11 纯函数验证 | 🟥 1.1.4 逻辑与表现分离 | 必须遵循 |

交叉引用：
- `docs/01-architecture/migration-roadmap.md` — 项目架构迁移（不同于此文档的存档数据迁移）
- `docs/01-architecture/infrastructure-design.md` — persistence 模块设计
- `docs/01-architecture/README.md` — Save 章节（只保存 Instance，不保存 Definition）

---

## 1. 存档格式版本管理

### 1.1 SemVer 版本号

存档格式使用 SemVer 版本号：`MAJOR.MINOR.PATCH`

| 版本变更 | 含义 | 示例 |
|---------|------|------|
| MAJOR | 不兼容变更，需要迁移 | 1.0.0 → 2.0.0 |
| MINOR | 向后兼容的新增字段 | 1.0.0 → 1.1.0 |
| PATCH | Bug 修复，不影响格式 | 1.0.0 → 1.0.1 |

### 1.2 版本号位置

```rust
pub struct SaveData {
    pub version: SemVer,          // 存档版本号
    pub timestamp: u64,           // 保存时间戳
    pub instances: InstanceData,  // 运行时状态
    pub metadata: SaveMetadata,   // 存档元信息
}

pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}
```

### 1.3 版本号规则

- 🟥 每次格式变更必须递增版本号
- 🟥 MAJOR 变更必须提供迁移路径
- 🟩 MINOR 变更可以不提供迁移路径（向后兼容）
- 🟩 PATCH 变更不影响存档格式

---

## 2. 向前兼容策略

### 2.1 核心原则

🟥 **新游戏版本必须能加载旧版本存档**，通过迁移链实现。

```
新版本加载旧存档
  ↓
检测存档版本
  ↓
执行迁移链 V1 → V2 → V3 → ... → 当前版本
  ↓
加载迁移后的存档
```

### 2.2 迁移链

```rust
pub fn migrate(mut data: SaveData, target: SemVer) -> Result<SaveData, MigrationError> {
    while data.version < target {
        data = match (data.version.major, data.version.minor) {
            (1, 0) => migrate_v1_0_to_v1_1(data)?,
            (1, 1) => migrate_v1_1_to_v2_0(data)?,
            (2, 0) => migrate_v2_0_to_v2_1(data)?,
            _ => return Err(MigrationError::UnsupportedVersion(data.version)),
        };
    }
    Ok(data)
}
```

### 2.3 迁移函数规范

每个迁移函数处理一个版本跳跃：

```rust
pub fn migrate_v1_0_to_v1_1(mut data: SaveData) -> Result<SaveData, MigrationError> {
    // 1. 验证当前版本
    assert_eq!(data.version, SemVer::new(1, 0, 0));
    
    // 2. 执行数据转换
    //    - 新增字段：使用默认值填充
    //    - 删除字段：忽略
    //    - 字段重命名：映射到新名称
    
    // 3. 更新版本号
    data.version = SemVer::new(1, 1, 0);
    
    // 4. 验证迁移后数据合法性
    validate_save_data(&data)?;
    
    Ok(data)
}
```

### 2.4 迁移函数约束

- 🟥 每个迁移函数必须是纯函数（不依赖外部状态）
- 🟥 每个迁移函数必须验证输入数据合法性
- 🟥 每个迁移函数必须验证输出数据合法性
- 🟩 每个迁移函数必须有对应的单元测试
- 🟩 迁移函数应该记录 WARN 日志（表示正在执行迁移）

---

## 3. 最低支持版本

### 3.1 政策

🟥 **支持最近 3 个 MAJOR 版本的存档**。

| 当前版本 | 支持的最旧版本 |
|---------|--------------|
| 3.0.0 | 1.0.0 |
| 4.0.0 | 2.0.0 |
| 5.0.0 | 3.0.0 |

### 3.2 不支持的版本处理

当存档版本低于最低支持版本时：

```rust
fn load_save(path: &Path) -> Result<SaveData, LoadError> {
    let data = read_save_file(path)?;
    
    if data.version < MIN_SUPPORTED_VERSION {
        return Err(LoadError::UnsupportedVersion {
            found: data.version,
            minimum: MIN_SUPPORTED_VERSION,
        });
    }
    
    // 执行迁移...
}
```

**处理方式**：
- 🟥 拒绝加载
- 🟥 记录 ERROR 日志
- 🟥 提示用户更新游戏或重新开始
- 🟥 不尝试加载（避免数据损坏）

---

## 4. 迁移执行时机

### 4.1 执行流程

```
用户选择加载存档
  ↓
读取存档文件
  ↓
检测存档版本
  ↓
├── 版本匹配 → 直接加载
├── 版本较旧 → 执行迁移链 → 加载
└── 版本过旧 → 拒绝加载 + 提示
```

### 4.2 迁移在反序列化之前

🟥 **迁移是数据转换，在反序列化为游戏状态之前执行**。

```rust
pub fn load_game(path: &Path, registries: &Registries) -> Result<SaveData, LoadError> {
    // 1. 读取原始 JSON
    let raw_json = std::fs::read_to_string(path)?;
    
    // 2. 反序列化为 SaveData（粗粒度）
    let mut data: SaveData = serde_json::from_str(&raw_json)?;
    
    // 3. 执行迁移（数据转换）
    if data.version < CURRENT_VERSION {
        data = migrate(data, CURRENT_VERSION)?;
    }
    
    // 4. 验证迁移后数据
    validate_save_data(&data)?;
    
    // 5. 恢复 Definition 引用
    data.restore_definitions(registries)?;
    
    Ok(data)
}
```

### 4.3 迁移时机限制

- 🟩 迁移在游戏启动时或加载存档时执行
- 🟥 迁移不在战斗中执行
- 🟥 迁移不阻塞主线程超过 5 秒

---

## 5. 回滚策略

### 5.1 迁移失败处理

当迁移失败时：

```rust
fn migrate_with_rollback(
    data: SaveData,
    target: SemVer,
) -> Result<SaveData, MigrationError> {
    // 1. 备份原始数据
    let backup = data.clone();
    
    // 2. 尝试迁移
    match migrate(data, target) {
        Ok(migrated) => Ok(migrated),
        Err(e) => {
            // 3. 迁移失败，回滚到原始数据
            error!(
                from_version = %backup.version,
                to_version = %target,
                error = %e,
                "Migration failed, rolling back"
            );
            
            // 4. 保持原始存档文件不变
            // 5. 返回错误，不修改原文件
            Err(e)
        }
    }
}
```

### 5.2 回滚规则

- 🟥 迁移失败时保持原始存档文件不变
- 🟥 不修改原文件内容
- 🟥 不删除原文件
- 🟥 记录 ERROR 日志
- 🟥 返回错误给调用者

### 5.3 备份策略

- 🟩 迁移前创建临时备份文件
- 🟩 迁移成功后删除备份
- 🟩 迁移失败后保留备份供用户恢复

---

## 6. 存档内容规则

### 6.1 只保存 Instance

🟥 **存档只保存 Instance 数据，禁止保存 Definition**。

**必须保存**：
- 单位属性（Attributes）
- Buff 列表（ActiveBuffs）
- 装备槽位（EquipmentSlots）
- 技能冷却（SkillCooldowns）
- 背包物品（Inventory）
- 回合状态（TurnState）
- 地图状态（地图 ID、单位位置）

**禁止保存**：
- 技能定义（SkillDef）
- Buff 定义（BuffDef）
- 装备定义（EquipmentDef）
- 物品定义（ItemDef）
- 角色模板（UnitTemplate）

### 6.2 Definition 恢复

加载存档时，通过 ID 从 Registry 恢复 Definition：

```rust
impl SaveData {
    pub fn restore_definitions(&mut self, registries: &Registries) -> Result<(), LoadError> {
        // 恢复技能定义
        for skill_instance in &mut self.instances.skills {
            let skill_def = registries.skills.get(&skill_instance.skill_id)
                .ok_or(LoadError::SkillNotFound(skill_instance.skill_id))?;
            skill_instance.restore_def(skill_def);
        }
        
        // 恢复 Buff 定义
        for buff_instance in &mut self.instances.buffs {
            let buff_def = registries.buffs.get(&buff_instance.buff_id)
                .ok_or(LoadError::BuffNotFound(buff_instance.buff_id))?;
            buff_instance.restore_def(buff_def);
        }
        
        // ... 其他类型
        
        Ok(())
    }
}
```

---

## 7. 存档验证

### 7.1 加载时验证

存档加载后必须执行验证：

- 🟥 版本号在支持范围内
- 🟥 时间戳合理（不早于游戏发布日期）
- 🟥 所有 ID 引用的 Definition 存在于 Registry
- 🟥 属性值在合法范围内（HP ≥ 0，MP ≥ 0）
- 🟥 Buff 数量不超限
- 🟥 单位位置在地图范围内

### 7.2 验证失败处理

```rust
fn validate_save_data(data: &SaveData) -> Result<(), SaveValidationError> {
    // 版本检查
    if data.version < MIN_SUPPORTED_VERSION {
        return Err(SaveValidationError::UnsupportedVersion(data.version));
    }
    
    // 属性检查
    for unit in &data.instances.units {
        if unit.attributes.current_hp < 0 {
            return Err(SaveValidationError::InvalidHp {
                unit_id: unit.id,
                hp: unit.attributes.current_hp,
            });
        }
    }
    
    // ... 其他验证
    
    Ok(())
}
```

---

## 8. 禁止事项

- 🟥 **破坏存档格式而不提供迁移路径**（必须有 V_old → V_new 的迁移函数）
- 🟥 **删除旧版本的迁移函数**（迁移函数是永久资产）
- 🟥 **跳过迁移直接加载旧存档**（必须通过迁移链）
- 🟥 **迁移失败时修改原始存档文件**（必须保持原文件不变）
- 🟥 **存档保存 Definition 数据**（只保存 Instance）
- 🟥 **存档格式无版本号**（必须有 SemVer 版本号）
- 🟥 **迁移函数不验证输入输出合法性**（必须验证）
- 🟥 **迁移函数依赖外部状态**（必须是纯函数）
- 🟥 **支持超过 3 个 MAJOR 版本的旧存档**（最低支持版本策略）
- 🟥 **迁移在战斗中执行**（只在加载时执行）

---

## 9. 实现备注

### 9.1 迁移注册表

```rust
pub struct MigrationRegistry {
    migrations: Vec<(SemVer, SemVer, Box<dyn Fn(SaveData) -> Result<SaveData, MigrationError>>)>,
}

impl MigrationRegistry {
    pub fn new() -> Self {
        let mut registry = Self { migrations: Vec::new() };
        
        // 注册所有迁移函数
        registry.register(SemVer::new(1, 0, 0), SemVer::new(1, 1, 0), 
            Box::new(migrate_v1_0_to_v1_1));
        registry.register(SemVer::new(1, 1, 0), SemVer::new(2, 0, 0), 
            Box::new(migrate_v1_1_to_v2_0));
        
        registry
    }
    
    pub fn migrate(&self, data: SaveData, target: SemVer) -> Result<SaveData, MigrationError> {
        let mut current = data;
        while current.version < target {
            let migration = self.find_migration(&current.version)
                .ok_or(MigrationError::UnsupportedVersion(current.version))?;
            current = migration(current)?;
        }
        Ok(current)
    }
}
```

### 9.2 迁移测试模板

```rust
#[cfg(test)]
mod migration_tests {
    use super::*;
    
    #[test]
    fn test_v1_0_to_v1_1_preserves_data() {
        let original = create_test_save_v1_0();
        let migrated = migrate_v1_0_to_v1_1(original.clone()).unwrap();
        
        // 验证版本号更新
        assert_eq!(migrated.version, SemVer::new(1, 1, 0));
        
        // 验证核心数据保留
        assert_eq!(migrated.instances.units.len(), original.instances.units.len());
        
        // 验证新字段有默认值
        assert!(migrated.metadata.playtime >= 0);
    }
    
    #[test]
    fn test_migration_chain_v1_0_to_v2_0() {
        let original = create_test_save_v1_0();
        let migrated = migrate(original, SemVer::new(2, 0, 0)).unwrap();
        
        assert_eq!(migrated.version, SemVer::new(2, 0, 0));
        validate_save_data(&migrated).unwrap();
    }
}
```

---

## 10. 迁移链完整代码级示例

> **优化来源**: `docs/其他/65.md`

### V1 → V2 → V3 完整迁移链

```rust
// ═══════════════════════════════════════════════════
// V1.0.0 → V1.1.0：新增 playtime 字段
// ═══════════════════════════════════════════════════

// 输入结构（V1.0.0）
pub struct SaveDataV1_0 {
    pub version: SemVer,
    pub units: Vec<UnitInstanceV1>,
    pub skills: Vec<SkillInstance>,
    pub buffs: Vec<BuffInstance>,
    // 无 playtime 字段
}

// 输出结构（V1.1.0）
pub struct SaveDataV1_1 {
    pub version: SemVer,
    pub units: Vec<UnitInstanceV1>,
    pub skills: Vec<SkillInstance>,
    pub buffs: Vec<BuffInstance>,
    pub playtime: u64,  // 新增字段
}

pub fn migrate_v1_0_to_v1_1(data: SaveDataV1_0) -> Result<SaveDataV1_1, MigrationError> {
    // 纯函数：不依赖外部状态
    Ok(SaveDataV1_1 {
        version: SemVer::new(1, 1, 0),
        units: data.units,
        skills: data.skills,
        buffs: data.buffs,
        playtime: 0,  // 默认值
    })
}

// ═══════════════════════════════════════════════════
// V1.1.0 → V2.0.0：单位属性重构
// ═══════════════════════════════════════════════════

// 输入结构（V1.1.0）
pub struct UnitInstanceV1 {
    pub id: UnitId,
    pub hp: i32,
    pub mp: i32,
    pub attack: i32,
    pub defense: i32,
}

// 输出结构（V2.0.0）
pub struct UnitInstanceV2 {
    pub id: UnitId,
    pub attributes: Attributes,  // 重构为复合结构
    pub buffs: Vec<BuffInstance>,
}

pub fn migrate_v1_1_to_v2_0(data: SaveDataV1_1) -> Result<SaveDataV2_0, MigrationError> {
    let units = data.units.into_iter().map(|u| {
        UnitInstanceV2 {
            id: u.id,
            attributes: Attributes {
                hp: u.hp,
                max_hp: u.hp,
                mp: u.mp,
                max_mp: u.mp,
                attack: u.attack,
                defense: u.defense,
            },
            buffs: Vec::new(),
        }
    }).collect();

    Ok(SaveDataV2_0 {
        version: SemVer::new(2, 0, 0),
        units,
        skills: data.skills,
        playtime: data.playtime,
    })
}

// ═══════════════════════════════════════════════════
// V2.0.0 → V3.0.0：技能实例增加冷却状态
// ═══════════════════════════════════════════════════

pub fn migrate_v2_0_to_v3_0(data: SaveDataV2_0) -> Result<SaveDataV3_0, MigrationError> {
    let skills = data.skills.into_iter().map(|s| {
        SkillInstanceV3 {
            skill_id: s.skill_id,
            cooldown_remaining: 0,  // 新增冷却字段
            usage_count: 0,         // 新增使用次数
        }
    }).collect();

    Ok(SaveDataV3_0 {
        version: SemVer::new(3, 0, 0),
        units: data.units,
        skills,
        playtime: data.playtime,
    })
}
```

### 迁移注册表

```rust
pub fn register_all_migrations(registry: &mut MigrationRegistry) {
    registry.register(SemVer::new(1, 0, 0), SemVer::new(1, 1, 0),
        Box::new(migrate_v1_0_to_v1_1));
    registry.register(SemVer::new(1, 1, 0), SemVer::new(2, 0, 0),
        Box::new(migrate_v1_1_to_v2_0));
    registry.register(SemVer::new(2, 0, 0), SemVer::new(3, 0, 0),
        Box::new(migrate_v2_0_to_v3_0));
}
```

---

## 11. 纯函数验证测试规范

> **优化来源**: `docs/其他/65.md`

### 纯函数约束

迁移函数必须是**纯函数**——不依赖外部状态，输入决定输出。

🟥 **禁止的行为**：
- 访问 `Res<T>` 或 `ResMut<T>`
- 读取文件系统
- 访问网络
- 使用 `App` 或 `World`
- 依赖全局静态变量

### 验证测试模板

```rust
#[cfg(test)]
mod migration_purity_tests {
    use super::*;

    #[test]
    fn test_migrate_v1_0_to_v1_1_is_pure() {
        let input = create_test_save_v1_0();

        // 纯函数测试：多次调用结果一致
        let result1 = migrate_v1_0_to_v1_1(input.clone()).unwrap();
        let result2 = migrate_v1_0_to_v1_1(input).unwrap();

        assert_eq!(result1.version, result2.version);
        assert_eq!(result1.units.len(), result2.units.len());
    }

    #[test]
    fn test_migrate_v1_0_to_v1_1_preserves_data() {
        let original = create_test_save_v1_0();
        let migrated = migrate_v1_0_to_v1_1(original.clone()).unwrap();

        // 验证版本号更新
        assert_eq!(migrated.version, SemVer::new(1, 1, 0));

        // 验证核心数据保留
        assert_eq!(migrated.units.len(), original.units.len());
        assert_eq!(migrated.skills.len(), original.skills.len());

        // 验证新字段有默认值
        assert_eq!(migrated.playtime, 0);
    }

    #[test]
    fn test_migration_chain_v1_0_to_v3_0() {
        let original = create_test_save_v1_0();
        let migrated = migrate(original, SemVer::new(3, 0, 0)).unwrap();

        assert_eq!(migrated.version, SemVer::new(3, 0, 0));
        validate_save_data(&migrated).unwrap();
    }

    #[test]
    fn test_migrate_rejects_invalid_input() {
        let mut invalid = create_test_save_v1_0();
        invalid.units[0].hp = -100;  // 非法值

        let result = migrate_v1_0_to_v1_1(invalid);
        assert!(result.is_err());
    }
}
```

### 规则

- 🟥 **迁移函数必须通过纯函数测试** — 多次调用结果一致
- 🟥 **迁移函数必须验证输入合法性** — 拒绝非法数据
- 🟥 **迁移函数必须验证输出合法性** — 确保迁移后数据合法
- 🟩 **每个迁移函数必须有独立单元测试** — 覆盖正常/异常场景

---

## 12. Instance/Definition 分离保存策略

> **优化来源**: `docs/其他/65.md`

### 核心原则

🟥 **存档只保存 Instance 数据，禁止保存 Definition**。

```
存档文件（SaveData）
├── version: SemVer
├── timestamp: u64
├── instances: InstanceData
│   ├── units: Vec<UnitInstance>      ✅ 运行时状态
│   ├── skills: Vec<SkillInstance>    ✅ 运行时状态
│   ├── buffs: Vec<BuffInstance>      ✅ 运行时状态
│   └── ...
└── metadata: SaveMetadata

禁止保存的 Definition：
├── SkillDef       ❌ 从 SkillRegistry 恢复
├── BuffDef        ❌ 从 BuffRegistry 恢复
├── EquipmentDef   ❌ 从 EquipmentRegistry 恢复
└── ...
```

### Definition 恢复流程

```rust
pub fn load_game(path: &Path, registries: &Registries) -> Result<SaveData, LoadError> {
    // 1. 读取并反序列化
    let mut data: SaveData = read_save_file(path)?;

    // 2. 执行迁移
    if data.version < CURRENT_VERSION {
        data = migrate(data, CURRENT_VERSION)?;
    }

    // 3. 验证数据合法性
    validate_save_data(&data)?;

    // 4. 恢复 Definition 引用（从 Registry）
    data.restore_definitions(registries)?;

    Ok(data)
}

impl SaveData {
    pub fn restore_definitions(&mut self, registries: &Registries) -> Result<(), LoadError> {
        // 恢复技能定义
        for skill_instance in &mut self.instances.skills {
            let skill_def = registries.skills.get(&skill_instance.skill_id)
                .ok_or(LoadError::SkillNotFound(skill_instance.skill_id))?;
            skill_instance.restore_def(skill_def);
        }

        // 恢复 Buff 定义
        for buff_instance in &mut self.instances.buffs {
            let buff_def = registries.buffs.get(&buff_instance.buff_id)
                .ok_or(LoadError::BuffNotFound(buff_instance.buff_id))?;
            buff_instance.restore_def(buff_def);
        }

        // ... 其他类型

        Ok(())
    }
}
```

### 规则

- 🟥 **存档文件中禁止出现 Definition 类型** — 只保存 Instance
- 🟥 **Definition 必须从 Registry 恢复** — 加载时通过 ID 查询
- 🟩 **Definition 缺失时返回明确错误** — `LoadError::SkillNotFound(skill_id)`
- 🟩 **版本更新后 Definition 可能变化** — 存档仍有效（从新 Registry 恢复）

---

## 13. 备份与回滚策略

> **优化来源**: `docs/其他/65.md`

### 三步策略

```
备份 → 迁移 → 成功删备份 / 失败恢复备份
```

### 完整代码示例

```rust
pub fn migrate_with_backup(
    save_path: &Path,
    target: SemVer,
) -> Result<SaveData, MigrationError> {
    // 第一步：备份原始文件
    let backup_path = save_path.with_extension("bak");
    std::fs::copy(save_path, &backup_path)
        .map_err(|e| MigrationError::BackupFailed(e.to_string()))?;

    // 第二步：执行迁移
    let data = read_save_file(save_path)?;
    match migrate(data, target) {
        Ok(migrated) => {
            // 第三步（成功）：删除备份
            let _ = std::fs::remove_file(&backup_path);
            Ok(migrated)
        }
        Err(e) => {
            // 第三步（失败）：恢复备份
            let _ = std::fs::copy(&backup_path, save_path);
            let _ = std::fs::remove_file(&backup_path);

            error!(
                from_version = %read_save_file(save_path)?.version,
                to_version = %target,
                error = %e,
                "Migration failed, restored from backup"
            );

            Err(e)
        }
    }
}
```

### 备份文件命名规则

```
原始文件：saves/game_001.sav
备份文件：saves/game_001.bak
临时文件：saves/game_001.tmp
```

### 备份清理规则

- 🟩 **迁移成功后立即删除备份** — 避免备份文件堆积
- 🟩 **备份文件过期清理（7天）** — 定期扫描 `.bak` 文件
- 🟥 **备份失败必须中止迁移** — 不允许无备份迁移

### 规则

- 🟥 **迁移前必须创建备份** — 三步策略不可省略
- 🟥 **迁移成功后必须删除备份** — 避免磁盘浪费
- 🟥 **迁移失败必须恢复备份** — 保证原始文件完整
- 🟩 **备份文件使用 `.bak` 扩展名** — 统一命名规范

---

## 14. 验证驱动设计

> **优化来源**: `docs/其他/65.md`

### 加载时验证维度

存档加载后必须执行多维度验证：

| 验证维度 | 检查内容 | 失败处理 |
|---------|---------|---------|
| 版本号 | 在支持范围内（≥ MIN_SUPPORTED_VERSION） | 拒绝加载 |
| 时间戳 | 不早于游戏发布日期 | WARN 日志 |
| ID 引用 | 所有 SkillId/BuffId 在 Registry 中存在 | 恢复 Definition 失败 |
| 属性范围 | HP ≥ 0, MP ≥ 0, 攻击力 ≥ 0 | 修正为默认值 |
| Buff 数量 | 不超过系统上限 | 截断多余 Buff |
| 单位位置 | 在地图范围内 | 重置到出生点 |

### 验证代码示例

```rust
pub fn validate_save_data(data: &SaveData) -> Result<(), SaveValidationError> {
    // 1. 版本验证
    if data.version < MIN_SUPPORTED_VERSION {
        return Err(SaveValidationError::UnsupportedVersion {
            found: data.version,
            minimum: MIN_SUPPORTED_VERSION,
        });
    }

    // 2. 时间戳验证
    let game_release_timestamp = 1700000000; // 游戏发布日期
    if data.timestamp < game_release_timestamp {
        warn!(
            timestamp = data.timestamp,
            release = game_release_timestamp,
            "Save timestamp before game release"
        );
    }

    // 3. 属性范围验证
    for unit in &data.instances.units {
        if unit.attributes.hp < 0 {
            return Err(SaveValidationError::InvalidAttribute {
                unit_id: unit.id,
                attribute: "hp".to_string(),
                value: unit.attributes.hp,
            });
        }
    }

    // 4. Buff 数量验证
    for unit in &data.instances.units {
        if unit.buffs.len() > MAX_BUFFS_PER_UNIT {
            return Err(SaveValidationError::BuffOverflow {
                unit_id: unit.id,
                count: unit.buffs.len(),
                max: MAX_BUFFS_PER_UNIT,
            });
        }
    }

    Ok(())
}
```

### 规则

- 🟥 **加载时必须执行完整验证** — 不允许跳过验证
- 🟥 **验证失败必须返回明确错误** — 包含失败原因和上下文
- 🟩 **验证失败时提供修正建议** — 如"HP 为负数，已重置为 0"
- 🟩 **验证结果记录到诊断系统** — 用于监控存档质量

---

## 15. Bevy Diagnostics 集成

> **优化来源**: `docs/其他/65.md`

### 诊断指标

使用 Bevy 的 Diagnostics 系统监控迁移过程：

```rust
pub fn migrate_with_diagnostics(
    data: SaveData,
    target: SemVer,
    diagnostics: &mut Diagnostics,
) -> Result<SaveData, MigrationError> {
    let start = Instant::now();

    // 执行迁移
    match migrate(data, target) {
        Ok(migrated) => {
            // 记录成功指标
            let duration = start.elapsed().as_millis() as f64;
            diagnostics.add_measurement("save_migration_time", duration);
            diagnostics.add_measurement("save_migration_success", 1.0);
            diagnostics.add_measurement("save_migration_failure", 0.0);

            info!(
                from_version = %migrated.version,
                to_version = %target,
                duration_ms = duration,
                "Migration completed successfully"
            );

            Ok(migrated)
        }
        Err(e) => {
            // 记录失败指标
            diagnostics.add_measurement("save_migration_success", 0.0);
            diagnostics.add_measurement("save_migration_failure", 1.0);

            error!(
                error = %e,
                "Migration failed"
            );

            Err(e)
        }
    }
}
```

### 诊断报告

```
┌─────────────────────────────────────────────┐
│         Save Migration Diagnostics          │
├─────────────────────────────────────────────┤
│ Total migrations:     1250                  │
│ Successful:           1248 (99.84%)         │
│ Failed:               2 (0.16%)             │
│ Average time:         45ms                  │
│ Max time:             230ms                 │
│ Min time:             12ms                  │
│ Version distribution:                        │
│   V1.0 → V2.0:       1200 (96%)            │
│   V1.1 → V2.0:       50 (4%)               │
└─────────────────────────────────────────────┘
```

### 规则

- 🟩 **关键迁移步骤必须记录诊断指标** — 耗时、成功率
- 🟩 **诊断数据用于性能优化** — 识别慢迁移路径
- 🟩 **诊断报告定期生成** — 用于版本发布决策
- 🟥 **迁移耗时超过 5 秒必须告警** — 可能影响用户体验

---

## 16. 与其他文档的关系

| 文档 | 关系 |
|------|------|
| `migration-roadmap.md` | 该文档是项目架构迁移，本文档是存档数据迁移 |
| `infrastructure-design.md` | persistence 模块实现存档保存/加载 |
| `architecture.md` | 本文档是"Save"章节的详细补充 |
| `validation_rules.md` | 存档验证遵循全局校验规则 |
| `testing_architecture.md` | 迁移函数需要对应的单元测试 |
