# 内容迁移 领域规则

Version: 1.0
Status: Draft
Applies To: 内容格式迁移管理

> **优化来源**: docs/architecture/content_migration_design.md

核心原则：
- 🟩 **12.1.3 兼容性优先**：配置的向后兼容性优先于任何其他考虑（宪法条款 12.1.3）
- 🟩 **12.1.2 配置稳定性**：配置结构的稳定性优先于配置格式的优雅性（宪法条款 12.1.2）
- 🟩 **12.5.1 三步删除原则**：核心配置字段禁止直接删除，必须遵循 Deprecated → Migration → Remove（宪法条款 12.5.1）

---

## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| 内容迁移（Content Migration） | RON 配置文件 Schema 在游戏版本间变化时，将旧格式自动转换为新格式 | 负责：配置文件格式转换；不负责：玩家存档转换、运行时状态迁移 |
| 迁移链（Migration Chain） | V1→V2→V3 的逐步迁移序列，每步处理单个版本跳跃 | 负责：按顺序执行单版本转换；不负责：跨版本跳跃 |
| SemVer 版本号 | 内容格式的语义化版本号（MAJOR.MINOR.PATCH） | 负责：标记格式变更等级；不负责：标记游戏功能版本 |
| 纯函数迁移（Pure Function Migration） | 迁移函数不依赖外部状态、不产生副作用的数据转换 | 负责：输入旧数据输出新数据；不负责：修改磁盘文件、访问 Registry |
| 内容来源（ContentSource） | 区分核心内容（Core）与 MOD 内容（Mod），决定迁移失败处理策略 | 负责：隔离失败影响范围；不负责：判断内容正确性 |
| ContentMigrationRegistry | 存储所有内容类型迁移链的注册表 | 负责：管理迁移函数映射；不负责：执行内容加载 |
| 迁移函数（Migration Function） | 处理单个版本跳跃的转换逻辑（如 V1→V2） | 负责：格式转换与数据验证；不负责：跨版本转换、运行时状态处理 |
| AssetLoader 集成 | 迁移在 Bevy AssetLoader::load() 内部执行，返回最新版本 Asset 类型 | 负责：在内容加载时自动触发迁移；不负责：独立预处理 pass |

---

## 2. 领域边界

### 2.1 内容迁移负责

- RON 配置文件的 Schema 版本检测与格式转换
- 迁移链的注册、编排与执行
- 各内容类型（SkillDef、BuffDef、ItemDef 等）的独立版本管理
- MAJOR 变更的迁移路径保证
- MINOR/PATCH 变更的向后兼容处理
- Core 内容迁移失败时阻止游戏启动
- MOD 内容迁移失败时降级跳过

### 2.2 内容迁移不负责

- 玩家存档格式迁移（由存档迁移领域负责，见 `save_migration_rules.md`）
- 项目架构迁移（由 migration-roadmap 负责）
- 内容运行时加载与注册到 Registry（由内容系统领域负责）
- RON 文件的创建与编辑（由策划/工具负责）
- MOD 发现与加载逻辑（由 Mod 系统领域负责）

### 2.3 与其他领域的关系

| 领域 | 关系 |
|------|------|
| 存档持久化领域 | 职责分离：内容迁移转配置格式，存档迁移转运行时状态 |
| 内容系统领域 | 内容迁移在 Registry 加载前执行，是内容加载管线的一环 |
| 资产生命周期领域 | AssetLoader 集成点在资产加载阶段 |
| Mod 系统领域 | MOD 内容迁移失败降级策略由本领域定义 |

---

## 3. 生命周期

### 3.1 内容迁移生命周期

```
游戏启动
  ↓
加载 ContentMigrationRegistry
  ↓
发现 content/ 目录下所有 RON 文件
  ↓
对每个 RON 文件：
  ├─ 读取文件内 version 字段
  ├─ 对比当前版本号（ContentVersions）
  ├─ 判断是否需要迁移（file_version < current_version）
  │   ├─ 不需要 → 直接注册到 Registry
  │   └─ 需要 → 执行迁移链
  │       ├─ 查找 file_version → next_version 的迁移函数
  │       ├─ 执行迁移（内存中强类型转换）
  │       ├─ 重复直到达到 current_version
  │       └─ 注册最新版本到 Registry
  └─ 处理失败
      ├─ Core 内容 → 阻止游戏启动
      └─ MOD 内容 → 警告 + 跳过该 MOD
```

### 3.2 单个文件迁移流程

```
RON 文件读取（一次 RON Parse）
  ↓
版本检测（version 字段）
  ↓
迁移链执行（内存强类型转换，禁止 String→String）
  ├─ V1 → V2（migrate_v1_to_v2）
  ├─ V2 → V3（migrate_v2_to_v3）
  └─ ...
  ↓
输出最新版本结构（SkillDefV3）
  ↓
注册到 Registry
```

---

## 4. 不变量（Invariants）

### 4.1 核心不变量

> **优化来源**: docs/architecture/content_migration_design.md §4.1

**INV-1: 迁移函数必须是纯函数。**
不依赖外部状态（Registry、配置文件、网络），不产生副作用（写文件、发事件）。输入旧数据，输出新数据，结果确定性。

> **优化来源**: docs/architecture/content_migration_design.md §4.2

**INV-2: 每个迁移函数只处理单版本跳跃。**
V1→V2 是一个函数，V2→V3 是另一个函数。禁止 V1→V3 的跨版本函数。

> **优化来源**: docs/architecture/content_migration_design.md §3.2

**INV-3: 目录必须保持扁平化。**
版本信息只存在于 RON 文件内部的 `version` 字段。禁止使用 `v1/`、`v2/` 等版本子目录。

> **优化来源**: docs/architecture/content_migration_design.md §5.1

**INV-4: 每个 RON 配置文件必须有 `version` 字段。**
从项目第一天起强制。模板自动注入版本号，禁止遗漏。

> **优化来源**: docs/architecture/content_migration_design.md §5.2

**INV-5: 迁移在内存中执行强类型转换。**
全程只有一次 RON Parse（入口处），迁移在内存中完成，禁止 String→String 的反复序列化/反序列化。

> **优化来源**: docs/architecture/content_migration_design.md §8.2

**INV-6: Core 内容迁移失败必须阻止游戏启动。**
MOD 内容迁移失败则降级为警告 + 跳过该 MOD。

> **优化来源**: docs/architecture/content_migration_design.md §6

**INV-7: 迁移函数是永久资产。**
即使旧版本不再支持，已注册的迁移函数也必须保留，不可删除。

### 4.2 版本不变量

**INV-8: MAJOR 变更必须提供迁移路径。**
字段删除、类型改变等破坏性变更必须有对应的迁移函数。

**INV-9: 每次格式变更必须递增版本号。**
版本号只增不减。

---

## 5. 业务规则

### 5.1 SemVer 变更策略

> **优化来源**: docs/architecture/content_migration_design.md §7.1

| 版本变更 | 含义 | 迁移策略 | 示例 |
|---------|------|---------|------|
| MAJOR | 破坏性变更（字段删除、类型改变） | 必须编写迁移脚本 | 1.0.0 → 2.0.0 |
| MINOR | 向后兼容的新增（新增可选字段） | 使用 `#[serde(default)]` 自动兼容，无需迁移脚本 | 1.0.0 → 1.1.0 |
| PATCH | Bug 修复，不影响格式 | 不影响格式，无需迁移 | 1.0.0 → 1.0.1 |

### 5.1b 向后兼容性优先

> 🟩 **宪法条款 12.1.3**：配置的向后兼容性优先于任何其他考虑

允许：
- 新增可选字段（使用 `#[serde(default)]` 提供默认值）
- 优化配置结构（保持向后兼容）

禁止：
- 🟥 删除已有字段而不提供迁移路径
- 🟥 修改已有字段的类型（破坏兼容性）
- 🟥 修改字段语义（破坏兼容性）

必须：
- 新字段必须有默认值保证向后兼容
- 破坏性变更必须提供迁移函数
- 旧版本配置必须能通过迁移链加载到最新版本

### 5.2 三步字段删除原则

> 🟩 **宪法条款 12.5.1**：核心配置字段禁止直接删除，必须遵循 Deprecated → Migration → Remove

允许：
- 标记字段为 deprecated（保留字段但不再使用）
- 新增迁移逻辑处理废弃字段
- 在下个大版本移除废弃字段

禁止：
- 🟥 直接删除配置字段（跳过 Deprecated 阶段）
- 🟥 删除字段时不提供迁移函数
- 🟥 在标记 deprecated 的同一版本中删除字段

必须：
- 步骤1：标记 deprecated（保留字段，输出 WARN 日志）
- 步骤2：新增迁移逻辑（处理旧数据中的废弃字段）
- 步骤3：下个大版本移除字段

### 5.3 各内容类型独立版本

> **优化来源**: docs/architecture/content_migration_design.md §4.3

| 操作 | 处理方式 |
|------|---------|
| 新增字段 | 使用合理默认值填充 |
| 删除字段 | 忽略旧字段 |
| 字段重命名 | 映射到新字段名 |
| 类型变更 | 执行类型转换 |
| 必填字段缺失 | 返回错误 |

### 5.3 各内容类型独立版本

> **优化来源**: docs/architecture/content_migration_design.md §3.3

每种内容类型有独立的当前版本号：
- 技能配置（SkillDef）：独立版本
- Buff 配置（BuffDef）：独立版本
- 物品配置（ItemDef）：独立版本
- 地图配置：独立版本
- 其他内容类型：各自独立版本

各类型的迁移链独立运行，互不影响。

### 5.4 错误处理策略

> **优化来源**: docs/architecture/content_migration_design.md §8.2

| 内容来源 | 失败后果 | 理由 |
|---------|---------|------|
| Core（content/） | 阻止游戏启动 | 保证官方数据的绝对正确 |
| MOD（mods/） | 警告日志 + 跳过该 MOD | 不能因为第三方 MOD 导致整个游戏无法启动 |

错误类型分类：
- **MissingMigration**: 缺少从 from 到 to 的迁移函数
- **MigrationFailed**: 迁移函数执行失败（输入数据不合法）
- **ValidationFailed**: 迁移后数据验证失败
- **UnsupportedVersion**: 不支持的版本号

### 5.5 迁移执行方式

> **优化来源**: docs/architecture/content_migration_design.md §3.2

两种执行方式（二选一）：
- **运行时内存迁移（推荐）**：AssetLoader 读取 RON 文件，发现旧版本，在内存中执行迁移链，将最新版本结构注册到 Registry。磁盘文件保持原样。
- **离线 CLI 工具升级**：提供升级工具，策划提交前运行，工具读取旧文件，内存中转为最新版本，覆写原文件并更新 `version` 字段。

---

## 6. 流程管线

### 6.1 内容迁移在加载管线中的位置

```
RON 文件读取（AssetLoader::load()）
  ↓
反序列化为 Raw Enum（包含所有历史版本的 Untagged Enum）
  ↓ ← 内容迁移在此执行
版本检测 + 迁移链执行（内存强类型转换）
  ↓
输出最新版本 Asset 类型（SkillDef / BuffDef / ItemDef）
  ↓
注册到 Registry
```

> **优化来源**: docs/architecture/content_migration_design.md §5.2

关键约束：
- 迁移在 AssetLoader::load() 内部执行，而非独立的预处理 pass
- AssetLoader 返回值永远是最新版本的 Asset 类型
- 全程只有一次 RON Parse（入口处）

### 6.2 迁移链编排

```
ContentMigrationRegistry
  ├─ Skill: V1→V2→V3
  ├─ Buff: V1→V2
  ├─ Item: V1→V2→V3
  └─ Map: V1→V2

每种内容类型的迁移链独立编排，按版本号顺序执行。
```

### 6.3 Raw Enum 反序列化策略

> **优化来源**: docs/architecture/content_migration_design.md §5.2

使用 Serde Untagged Enum 自动匹配版本：
- 最新版本优先匹配（V3 → V2 → V1 顺序）
- 反序列化时自动识别文件版本
- 匹配后进入对应的迁移链入口

---

## 7. 数据结构

### 7.1 版本标记

每个 RON 配置文件携带 SemVer 版本标记：
- `version: "1.0.0"` — 必填字段
- 格式：`MAJOR.MINOR.PATCH`

### 7.2 目录结构

> **优化来源**: docs/architecture/content_migration_design.md §3.2

扁平化目录，禁止版本子目录：
```
content/
├── skills/
│   ├── fireball.ron    # version: "1.0.0" 或 "2.0.0" 或 "3.0.0"
│   ├── heal.ron
│   └── ...
├── buffs/
│   └── ...
└── items/
    └── ...
```

### 7.3 ContentVersions 结构

每种内容类型有独立的当前版本号，存储在全局版本配置中。

### 7.4 ContentMigrationRegistry 结构

- 按 ContentType 分组的迁移链集合
- 每个迁移条目包含：from_version、to_version、迁移函数
- 支持按版本查找下一个迁移步骤

### 7.5 内容来源类型

区分 Core（content/ 目录）与 Mod（mods/ 目录），决定迁移失败处理策略。

---

## 8. 禁止事项

> **优化来源**: docs/architecture/content_migration_design.md §10

| 编号 | 禁止事项 | 原因 |
|------|---------|------|
| FORBID-1 | 禁止内容格式变更不提供迁移路径 | MAJOR 版本变更必须有对应的迁移函数 |
| FORBID-2 | 禁止删除旧迁移函数 | 迁移函数是永久资产，即使旧版本不再支持也必须保留 |
| FORBID-3 | 禁止迁移失败静默继续 | Core 内容迁移失败必须阻止游戏启动（MOD 除外） |
| FORBID-4 | 禁止迁移函数中引入随机或外部依赖 | 迁移函数必须是纯函数，保证确定性 |
| FORBID-5 | 禁止跳过版本执行迁移 | 必须通过链式迁移，不允许跳过中间版本 |
| FORBID-6 | 禁止内容迁移在运行时执行 | 内容迁移只在启动时执行，不在战斗中执行 |
| FORBID-7 | 禁止内容格式无版本号 | 每个 RON 配置文件必须有 SemVer 版本号（从 Day 1 强制） |
| FORBID-8 | 禁止使用版本子目录（v1/、v2/） | 目录保持扁平，版本信息只在文件内 `version` 字段 |
| FORBID-9 | 禁止 String→String 的迁移链 | 迁移应在内存中执行强类型 Struct 转换，避免反复序列化 |
| FORBID-10 | 禁止 MOD 迁移失败阻止游戏启动 | MOD 失败应降级为警告 + 跳过该 MOD |
| FORBID-11 | 禁止直接删除配置字段而不提供迁移路径 | 违反宪法条款 12.5.1，必须遵循三步删除原则 |
| FORBID-12 | 禁止配置变更破坏向后兼容性 | 违反宪法条款 12.1.3，兼容性优先于任何其他考虑 |
| FORBID-13 | 禁止配置文件没有版本号 | 违反宪法条款 12.1.3，版本号是兼容性管理的基础 |

---

## 9. AI 修改规则

### 9.1 修改前检查

修改内容迁移相关代码前，必须确认：

1. **不违反纯函数约束** — 迁移函数不依赖外部状态
2. **不破坏单版本跳跃** — 每个函数只处理一个版本跳跃
3. **不删除旧迁移函数** — 迁移函数是永久资产
4. **不引入 String→String 迁移** — 保持强类型内存转换
5. **不修改定义态配置** — 迁移函数不修改 RON 模板

### 9.2 新增迁移函数要求

新增内容类型的迁移函数时，必须满足：

1. 迁移函数是纯函数（不依赖 Registry、配置文件、网络）
2. 有对应的单元测试（验证数据完整性）
3. 处理所有字段操作（新增、删除、重命名、类型变更）
4. 有输入验证（验证旧数据合法性）
5. 有输出验证（验证新数据合法性）
6. 注册到 ContentMigrationRegistry

### 9.3 版本号修改规则

修改内容格式版本号时，必须确认：

1. MAJOR 变更必须提供迁移路径（编写迁移函数）
2. MINOR 变更使用 `#[serde(default)]` 向后兼容
3. 版本号只增不减
4. 每次格式变更必须递增版本号

### 9.4 禁止的操作

- 禁止在迁移函数中访问 Bevy World 或 ECS 状态
- 禁止在迁移函数中进行文件 I/O 操作
- 禁止在迁移函数中使用随机数或时间戳
- 禁止删除已注册的迁移函数（即使对应的旧版本不再支持）
- 禁止将内容迁移与存档迁移的实现混合
