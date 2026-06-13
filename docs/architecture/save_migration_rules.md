# Save Migration Rules — 存档迁移与版本兼容

Version: 1.0
Status: Proposed

来源：`docs/其他/31遗漏.md` 第三节 — 存档迁移与版本兼容

本文档定义存档文件的版本管理、向前兼容策略和数据迁移规范。

交叉引用：
- `docs/architecture/migration-roadmap.md` — 项目架构迁移（不同于此文档的存档数据迁移）
- `docs/architecture/infrastructure-design.md` — persistence 模块设计
- `docs/architecture.md` — Save 章节（只保存 Instance，不保存 Definition）

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

## 10. 与其他文档的关系

| 文档 | 关系 |
|------|------|
| `migration-roadmap.md` | 该文档是项目架构迁移，本文档是存档数据迁移 |
| `infrastructure-design.md` | persistence 模块实现存档保存/加载 |
| `architecture.md` | 本文档是"Save"章节的详细补充 |
| `validation_rules.md` | 存档验证遵循全局校验规则 |
| `testing_architecture.md` | 迁移函数需要对应的单元测试 |
