---
id: 01-architecture.content-migration-design
title: Content Migration Design
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
  - design
---

# Content Migration Design — 内容格式迁移设计

Version: 1.1
Status: Proposed
Source: `docs/其他/33遗漏2.md` B12

本文档定义 SRPG 项目的内容格式迁移架构。当 RON 配置文件的结构在版本间发生变化时，通过迁移链将旧格式转换为新格式。

### 宪法条款映射

| 本文档规则 | 宪法条款 | 强制等级 |
|-----------|---------|---------|
| §1.1 内容迁移定义 | 🟥 12.5.1 三步删除原则 | 必须遵循 |
| §3.2 扁平化目录 + 文件内版本 | 🟥 12.6.1 强制版本字段 | 必须遵循 |
| §4.1 迁移函数纯函数 | 🟥 1.1.4 逻辑与表现分离 | 必须遵循 |
| §4.2 单版本跳跃 | 🟥 12.5.1 三步删除原则 | 必须遵循 |
| §5.1 文件级版本 | 🟥 12.6.1 强制版本字段 | 必须遵循 |
| §5.2 AssetLoader 集成 | 🟥 12.6.2 向后兼容原则 | 必须遵循 |
| §8.2 MOD 降级策略 | 🟥 1.1.7 只解决当前复杂度 | 必须遵循 |

交叉引用：
- `docs/01-architecture/save_migration_rules.md` — 存档迁移（区别于本文档的内容迁移）
- `docs/01-architecture/00-overview/project-structure.md` — content/ 目录结构
- `docs/01-architecture/09-infrastructure-migration/migration-roadmap.md` — 项目架构迁移（区别于本文档的内容数据迁移）
- `docs/01-architecture/tools_architecture.md` — migration_tool 工具

---

## 1. 什么是内容迁移

### 1.1 定义

🟥 **内容迁移：当 RON 配置文件的结构（Schema）在游戏版本间发生变化时，将旧格式文件自动转换为新格式。**

示例：技能配置 V1 没有 `cooldown` 字段，V2 新增了必填的 `cooldown` 字段。内容迁移负责为旧技能配置补上默认的 `cooldown` 值。

### 1.2 触发场景

| 场景 | 示例 |
|------|------|
| 新增必填字段 | 技能配置新增 `cooldown` 字段 |
| 字段重命名 | `damage` 重命名为 `base_damage` |
| 类型变更 | `target_type: String` → `target_type: TargetType` |
| 字段删除 | 移除已废弃的 `legacy_effect` 字段 |
| 结构重组 | 嵌套结构拍平，或拍平结构嵌套化 |

### 1.3 提前建立的重要性

🟥 **技能格式一定会变。必须提前建立迁移系统。**

随着内容量增长，手动修改数百个 RON 文件不现实。迁移系统是内容持续迭代的基础设施。

---

## 2. 内容迁移 vs 存档迁移

| 维度 | 存档迁移 | 内容迁移 |
|------|---------|---------|
| 迁移对象 | 玩家存档文件（`SaveData`） | 内容 RON 配置文件（`SkillDef`、`BuffDef` 等） |
| 执行时机 | 游戏加载存档时 | 注册表加载之前（Registry 加载前） |
| 迁移方向 | V1 → V2 → V3 链式迁移 | V1 → V2 → V3 链式迁移（每种配置类型独立） |
| 失败后果 | 玩家丢失进度 | 游戏无法启动 |
| 执行频率 | 每次加载存档（低频） | 每次启动游戏（高频） |
| 数据来源 | 单一存档文件 | content/ 目录下所有 RON 文件 |
| 涉及模块 | `infrastructure/persistence/migration/` | `content/migration/` |

### 2.1 关键区别

```
存档迁移：玩家存档 → 加载时检测版本 → 执行迁移链 → 恢复到游戏状态
内容迁移：RON 文件 → 启动时检测版本 → 执行迁移链 → 注册到 Registry
```

---

## 3. 迁移链架构

### 3.1 迁移链

```
content/skills/v1/fireball.ron
    ↓ MigrationV1ToV2
content/skills/v2/fireball.ron
    ↓ MigrationV2ToV3
content/skills/v3/fireball.ron
```

### 3.2 版本目录结构

> **优化来源**：`docs/其他/46.md` — 致命陷阱「按版本分目录是工程灾难」

🟥 **禁止使用版本子目录（`v1/`、`v2/`、`v3/`）。目录必须保持扁平化，版本信息只存在于 RON 文件内部的 `version` 字段。**

版本子目录的灾难性后果（来自 46.md 深度点评）：
- **认知崩溃**：策划新建技能时该放哪个目录？同一技能的不同版本散落不同目录，Git 历史追踪极其困难
- **仓库膨胀**：v1 到 v10 的目录永远留在源码树中，包含大量废弃旧格式文件
- **Bevy Asset 监听噩梦**：Bevy 的 AssetServer 监听目录变化，同时监听 v1 到 v10，热重载时文件事件极其混乱

```
// 🟥 禁止：版本子目录
content/
├── skills/
│   ├── v1/fireball.ron
│   ├── v2/fireball.ron
│   └── v3/fireball.ron

// ✅ 正确：扁平化目录 + 文件内版本标记
content/
├── skills/
│   ├── fireball.ron    # version: "1.0.0" 或 "2.0.0" 或 "3.0.0"
│   ├── heal.ron
│   └── ...
├── buffs/
│   ├── ...
└── items/
    ├── ...
```

执行方式（二选一）：
- **方式 A（运行时内存迁移，推荐）**：AssetLoader 读取 `fireball.ron`，发现是 V1，在内存中执行 V1→V2→V3 转换，将 V3 结构注册到 Registry。磁盘文件保持原样。
- **方式 B（离线 CLI 工具升级）**：提供 `cargo run --bin upgrade_content` 工具，策划提交前运行，工具读取 V1 文件，内存中转为 V3，覆写原文件并更新 `version` 字段。

### 3.3 当前版本标记

每个内容类型有独立的当前版本号：

```rust
pub struct ContentVersions {
    pub skills: SemVer,    // 当前技能配置版本
    pub buffs: SemVer,     // 当前 Buff 配置版本
    pub items: SemVer,     // 当前物品配置版本
    pub maps: SemVer,      // 当前地图配置版本
    // ...
}
```

---

## 4. 迁移函数契约

### 4.1 纯函数

🟥 **每个迁移函数必须是纯函数——不依赖外部状态，不产生副作用。**

```rust
/// 将技能配置从 V1 格式转换为 V2 格式
pub fn migrate_skill_v1_to_v2(
    old: SkillConfigV1,
) -> Result<SkillConfigV2, ContentMigrationError> {
    // 1. 验证输入数据合法性
    validate_skill_v1(&old)?;
    
    // 2. 执行数据转换
    let new = SkillConfigV2 {
        id: old.id,
        name: old.name,
        description: old.description,
        damage: old.damage,
        // V2 新增字段：使用默认值
        cooldown: old.cooldown.unwrap_or(0),
        // V2 移除字段：忽略 old.legacy_effect
    };
    
    // 3. 验证输出数据合法性
    validate_skill_v2(&new)?;
    
    Ok(new)
}
```

### 4.2 单版本跳跃

🟥 **每个迁移函数只处理一个版本跳跃。**

```
V1 → V2：migrate_v1_to_v2
V2 → V3：migrate_v2_to_v3
```

不允许：
```
V1 → V3：migrate_v1_to_v3  🟥 禁止
```

> **优化来源**：`docs/其他/46.md` — 链式单版本迁移「禁止跨版本迁移，强制 V1→V2→V3 链式迁移」

理由：跳过中间版本太容易遗漏中间版本的字段变更。V1→V2 可能重命名了字段，V2→V3 可能删除了字段，直接 V1→V3 会同时踩两个坑。

### 4.3 Serde 默认值处理 MINOR 变更

> **优化来源**：`docs/其他/46.md` — MINOR 变更无需迁移脚本

🟥 **MINOR 版本变更（新增可选字段）使用 `#[serde(default)]` 自动兼容，不需要编写迁移脚本。**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillConfigV2 {
    pub id: SkillId,
    pub name: String,
    pub damage: i32,

    // V2 新增字段：serde(default) 自动为旧文件填充默认值
    #[serde(default)]
    pub cooldown: u32,

    #[serde(default)]
    pub description: String,
}
```

规则：
- MAJOR 变更（字段删除、类型改变）→ 必须写迁移脚本
- MINOR 变更（新增可选字段）→ `#[serde(default)]` 自动兼容
- PATCH 变更（Bug 修复）→ 不影响格式，无需迁移

### 4.3 字段处理规则

| 操作 | 处理方式 |
|------|---------|
| 新增字段 | 使用合理默认值填充 |
| 删除字段 | 忽略旧字段 |
| 字段重命名 | 映射到新字段名 |
| 类型变更 | 执行类型转换 |
| 必填字段缺失 | 返回错误 |

### 4.4 单元测试

🟥 **每个迁移函数必须有对应的单元测试，验证数据完整性。**

```rust
#[cfg(test)]
mod migration_tests {
    use super::*;
    
    #[test]
    fn test_skill_v1_to_v2_preserves_data() {
        let original = create_test_skill_v1();
        let migrated = migrate_skill_v1_to_v2(original.clone()).unwrap();
        
        // 验证核心数据保留
        assert_eq!(migrated.id, original.id);
        assert_eq!(migrated.name, original.name);
        assert_eq!(migrated.damage, original.damage);
        
        // 验证新字段有默认值
        assert_eq!(migrated.cooldown, 0);
    }
    
    #[test]
    fn test_skill_migration_chain_v1_to_v3() {
        let original = create_test_skill_v1();
        let migrated = migrate_skill(original, SemVer::new(3, 0, 0)).unwrap();
        
        assert_eq!(migrated.version, SemVer::new(3, 0, 0));
        validate_skill_v3(&migrated).unwrap();
    }
}
```

---

## 5. 版本标记

### 5.1 文件级版本

每个 RON 配置文件携带版本标记：

```rust
// content/skills/fireball.ron
(
    version: "1.0.0",
    id: "fireball",
    name: "火球术",
    damage: 50,
)
```

> **优化来源**：`docs/其他/46.md` — Version field mandatory「每个 RON 文件必须有 `version: 1.0.0` 从第一天起」

🟥 **每个 RON 配置文件必须有 `version` 字段，从项目第一天起就强制。模板自动注入版本号，禁止遗漏。**

理由：
- 前期加一行 `version: 1` 的事，后期改数据格式时不用面对一堆无版本旧文件手足无措
- 版本号是"格式变更的唯一语言"，没有版本号就无法判断是否需要迁移

### 5.2 RegistryLoader 检测

RegistryLoader 在加载内容时检测版本：

> **优化来源**：`docs/其他/46.md` — AssetLoader 集成断层与 RawAsset 模式

🟥 **迁移在 AssetLoader::load() 中执行，而非独立的预处理 pass。Bevy AssetLoader 期望返回特定 Asset 类型，迁移必须在返回前完成。**

```rust
// ✅ 推荐：迁移在 AssetLoader::load() 内部执行
impl AssetLoader for SkillAssetLoader {
    type Asset = SkillDef;
    type Error = SkillLoadError;

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext<'_>,
    ) -> BoxFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            // 1. 反序列化为 RawSkill（包含所有历史版本的 Untagged Enum）
            let raw: RawSkill = ron::de::from_bytes(bytes)?;

            // 2. 根据版本执行迁移链，最终只产出最新版本
            let final_v3 = match raw {
                RawSkill::V3(v3) => v3,
                RawSkill::V2(v2) => migrate_v2_to_v3(v2)?,
                RawSkill::V1(v1) => migrate_v2_to_v3(migrate_v1_to_v2(v1)?)?,
            };

            Ok(final_v3) // 只产出最新版本的 Asset
        })
    }
}

/// 包含所有历史版本的原始枚举 — 用于反序列化
#[derive(Deserialize)]
#[serde(untagged)] // 关键：让 serde 自动尝试匹配版本
enum RawSkill {
    V3(SkillDefV3),  // 最新版本优先匹配
    V2(SkillDefV2),
    V1(SkillDefV1),
}
```

关键约束：
- 🟥 迁移链在内存中执行强类型转换（`SkillConfigV1 → SkillConfigV2`），禁止 String→String 的反复序列化/反序列化
- 🟥 全程只有一次 RON Parse（入口处），迁移在内存中完成
- 🟩 AssetLoader 返回值永远是最新版本的 Asset 类型

---

## 6. 迁移注册表

### 6.1 注册表结构

> **宪法条款**: 🟥 §5.2 禁止 String→String 的反复序列化/反序列化，迁移必须在内存中执行强类型转换

🟥 **注册表必须使用强类型迁移函数，禁止 String→String 的类型擦除。**

```rust
/// 强类型迁移注册表 — 每种内容类型独立注册
pub struct ContentMigrationRegistry<S> {
    /// 每种内容类型的迁移链（S = 该内容类型的强类型结构体）
    migrations: Vec<MigrationEntry<S>>,
}

pub struct MigrationEntry<S> {
    pub from_version: SemVer,
    pub to_version: SemVer,
    /// 强类型迁移函数：S → S，不做 String 序列化/反序列化
    pub migrate_fn: Box<dyn Fn(S) -> Result<S, ContentMigrationError>>,
}
```

### 6.2 注册示例

```rust
impl ContentMigrationRegistry<SkillDef> {
    pub fn new() -> Self {
        let mut registry = Self {
            migrations: Vec::new(),
        };
        
        // 技能迁移 — 强类型函数，不做 String 转换
        registry.register(
            SemVer::new(1, 0, 0),
            SemVer::new(2, 0, 0),
            Box::new(migrate_skill_v1_to_v2),  // SkillDefV1 → SkillDefV2
        );
        registry.register(
            SemVer::new(2, 0, 0),
            SemVer::new(3, 0, 0),
            Box::new(migrate_skill_v2_to_v3),  // SkillDefV2 → SkillDefV3
        );
        
        registry
    }
    
    pub fn register(
        &mut self,
        from: SemVer,
        to: SemVer,
        migrate_fn: Box<dyn Fn(S) -> Result<S, ContentMigrationError>>,
    ) {
        self.migrations.push(MigrationEntry {
            from_version: from,
            to_version: to,
            migrate_fn,
        });
    }
}
```

### 6.3 迁移执行

```rust
impl<S> ContentMigrationRegistry<S> {
    /// 执行强类型迁移链：S → S，不做 String 序列化
    pub fn migrate(
        &self,
        data: S,
        from: SemVer,
        to: SemVer,
    ) -> Result<S, ContentMigrationError> {
        let mut current_version = from;
        let mut data = data;
        
        while current_version < to {
            let entry = self.find_migration(&current_version)
                .ok_or(ContentMigrationError::MissingMigration {
                    from: current_version,
                    to: to,
                })?;
            
            // 强类型迁移：S → S，无 String 中间转换
            data = (entry.migrate_fn)(data)?;
            current_version = entry.to_version;
        }
        
        Ok(data)
    }
}
```

---

## 7. 内容格式版本管理

### 7.1 SemVer 版本号

内容格式使用 SemVer 版本号：`MAJOR.MINOR.PATCH`

| 版本变更 | 含义 | 示例 |
|---------|------|------|
| MAJOR | 破坏性变更（字段删除、类型改变） | 1.0.0 → 2.0.0 |
| MINOR | 向后兼容的新增（新增可选字段） | 1.0.0 → 1.1.0 |
| PATCH | Bug 修复，不影响格式 | 1.0.0 → 1.0.1 |

### 7.2 版本号规则

- 🟥 每次格式变更必须递增版本号
- 🟥 MAJOR 变更必须提供迁移路径
- 🟩 MINOR 变更可以不提供迁移路径（向后兼容）
- 🟩 PATCH 变更不影响内容格式

### 7.3 兼容性判断

```rust
impl SemVer {
    /// 判断是否需要迁移
    pub fn needs_migration(&self, target: &SemVer) -> bool {
        self < target
    }
    
    /// 判断是否为破坏性变更
    pub fn is_breaking(&self, other: &SemVer) -> bool {
        self.major != other.major
    }
}
```

---

## 8. 错误处理

### 8.1 错误类型

```rust
pub enum ContentMigrationError {
    /// 缺少从 from 到 to 的迁移函数
    MissingMigration {
        from: SemVer,
        to: SemVer,
    },
    
    /// 迁移函数执行失败
    MigrationFailed {
        from_version: SemVer,
        to_version: SemVer,
        reason: String,
    },
    
    /// 迁移后数据验证失败
    ValidationFailed {
        version: SemVer,
        reason: String,
    },
    
    /// 不支持的版本
    UnsupportedVersion(SemVer),
}
```

### 8.2 失败处理

🟥 **核心内容（Core）迁移失败必须阻止游戏启动。MOD 内容迁移失败则降级跳过。**

> **优化来源**：`docs/其他/46.md` — 缺失对"第三方 MOD"的迁移降级策略

```rust
/// 内容来源类型 — 决定迁移失败的处理策略
pub enum ContentSource {
    /// 核心内容（content/ 目录）— 迁移失败 → Panic
    Core,
    /// MOD 内容（mods/ 目录）— 迁移失败 → 警告 + 跳过
    Mod { mod_id: String },
}

pub fn load_all_content(registry: &ContentMigrationRegistry) -> Result<(), ContentLoadError> {
    // 1. 加载核心技能 — 失败即 Panic
    load_skills(registry, ContentSource::Core).map_err(|e| {
        error!("Core skill content migration failed: {}", e);
        ContentLoadError::MigrationFailed("skills".to_string(), e)
    })?;

    // 2. 加载 MOD 技能 — 失败则警告 + 跳过该 MOD
    for mod_entry in discover_mods() {
        if let Err(e) = load_skills(registry, ContentSource::Mod { mod_id: mod_entry.id.clone() }) {
            warn!(
                "MOD '{}' skill migration failed, skipping: {}",
                mod_entry.id, e
            );
            // 不阻止游戏启动，只在主界面弹出 MOD 兼容性警告
        }
    }

    Ok(())
}
```

失败策略对比：

| 内容来源 | 失败后果 | 理由 |
|---------|---------|------|
| Core（content/） | 🟥 Panic / 阻止启动 | 保证官方数据的绝对正确 |
| MOD（mods/） | 🟩 Error 日志 + 跳过该 MOD | 不能因为一个第三方 MOD 导致整个游戏无法启动 |

---

## 9. 与存档迁移的关系

### 9.1 职责分离

```
内容迁移：RON 配置文件格式转换 → 在 Registry 加载前执行
存档迁移：玩家存档数据转换 → 在存档加载时执行
```

### 9.2 独立模块

```rust
// 内容迁移 — 在 content/ 模块
src/content/migration/
├── mod.rs
├── content_migration_registry.rs
├── content_migration_error.rs
├── skill_migration.rs
├── buff_migration.rs
└── item_migration.rs

// 存档迁移 — 在 infrastructure/ 模块
src/infrastructure/persistence/migration/
├── mod.rs
├── migration_error.rs
└── ...
```

### 9.3 不共享实现

内容迁移和存档迁移不共享迁移函数实现。即使两者都处理技能数据，迁移逻辑也不同：
- 内容迁移：转换 RON 配置格式（SkillConfigV1 → SkillConfigV2）
- 存档迁移：转换运行时状态（SaveDataV1 → SaveDataV2）

---

## 10. 禁止事项

- 🟥 **禁止内容格式变更不提供迁移路径** — MAJOR 版本变更必须有对应的迁移函数
- 🟥 **禁止删除旧迁移函数** — 迁移函数是永久资产，即使旧版本不再支持也必须保留
- 🟥 **禁止迁移失败静默继续** — 迁移失败必须报错并阻止游戏启动（MOD 除外，见 8.2）
- 🟥 **禁止迁移函数中引入随机或外部依赖** — 迁移函数必须是纯函数
- 🟥 **禁止跳过版本执行迁移** — 必须通过链式迁移，不允许跳过中间版本
- 🟥 **禁止内容迁移在运行时执行** — 内容迁移只在启动时执行，不在战斗中执行
- 🟥 **禁止内容格式无版本号** — 每个 RON 配置文件必须有 SemVer 版本号（从 Day 1 强制）
- 🟥 **禁止使用版本子目录（v1/、v2/）** — 目录保持扁平，版本信息只在文件内 `version` 字段
- 🟥 **禁止 String→String 的迁移链** — 迁移应在内存中执行强类型 Struct 转换，避免反复序列化
- 🟥 **禁止 MOD 迁移失败阻止游戏启动** — MOD 失败应降级为警告 + 跳过该 MOD

---

## 11. 与其他文档的关系

| 文档 | 关系 |
|------|------|
| `save_migration_rules.md` | 存档迁移，与本文档的内容迁移职责分离 |
| `project-structure.md` | content/ 目录结构参考 |
| `layer-contracts.md` | 第五层 Content 桥接层定义 |
| `migration-roadmap.md` | 项目架构迁移（不同于此文档的内容数据迁移） |
| `tools_architecture.md` | migration_tool 工具定义 |
| `content-pipeline.md` | 内容加载管线，内容迁移是其中一步 |
