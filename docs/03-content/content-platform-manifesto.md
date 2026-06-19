---
id: 03-content.content-platform-manifesto
title: Content Platform Manifesto — 内容平台核心理念与治理原则
status: draft
owner: content-architect
created: 2026-06-20
tags:
  - content
  - manifesto
  - governance
  - philosophy
  - validation
  - ownership
---

# Content Platform Manifesto — 内容平台核心理念与治理原则

> **职责**: @content-architect | **适用范围**: 所有内容架构决策、Def Schema 设计、Registry 实现、Validation 规则

本文档定义 Content Platform 的核心理念和长期治理原则。它不是操作手册，而是**每个内容架构决策的判断依据**。当 Schema 设计、Registry 实现、Validation 规则出现争议时，以本文档为最终裁决依据。

---

## 1. 第一原理: Content = 游戏数据库，不是配置文件集合

### 1.1 区分

| 方面 | 配置文件思维 | 游戏数据库思维 |
|------|------------|--------------|
| 每个文件 | "这个文件配置了 Fireball 技能" | "这是一条 AbilityDef 记录，主键是 fireball" |
| ID | 文件名作为唯一标识 | 显式的 `id: AbilityId("fireball")` 字段 |
| 引用 | 通过字符串常识关联 | 通过强类型 ID 建立可追踪的外键关系 |
| 校验 | 格式检查（是否合法的 RON） | 完整性检查（每个引用的 ID 存在、类型匹配、无循环） |
| 依赖 | 无（每个文件独立） | 有向无环图（DAG），支撑加载顺序和影响分析 |
| 修改 | "改这个文件就行" | "改这条记录，然后检查所有引用者的影响" |
| 扩展 | "加个新文件" | "INSERT INTO AbilityDef VALUES (...)" |

### 1.2 每条 Def 都是数据库记录

每个 RON 资产文件中的一个 `(...)` 块等价于关系数据库中的一行记录：

```ron
// 数据库表: ability_def
// 记录:
(
    id: "fireball",                    // PRIMARY KEY
    name_key: "ability_fireball_name", // FOREIGN KEY → localization
    desc_key: "ability_fireball_desc", // FOREIGN KEY → localization
    effect_id: Some("eff:fire_damage"),// FOREIGN KEY → effect_def
    targeting_id: Some("tgt:radius_2"),// FOREIGN KEY → targeting_def
    cooldown: 3,
    schema_version: 1,                 // Schema 版本号
)
```

### 1.3 数据库类比

| 数据库概念 | Content Platform 等价物 |
|-----------|-----------------------|
| 数据库引擎 | Content Pipeline（加载/校验/注册管线） |
| 表 Schema | Def Rust struct + Schema 文档 (04-data) |
| 行/记录 | 单个 Def（RON 中的一个条目） |
| 主键 | `id: XxxId` 字段 |
| 外键 | 强类型 ID 引用（`EffectId`, `TargetingId` 等） |
| 索引 | Registry 的查询方法（`by_id()`, `by_tag()`） |
| 参照完整性 | Validation 的引用存在性检查 |
| 事务 | Content Plugin 的原子加载（全部成功或全部回滚） |
| 迁移 | schema_version + Migration 函数 |
| View | Registry 的查询投影（只读视图） |

---

## 2. Content Layering 的哲学依据

### 2.1 为什么分层

游戏内容的抽象层级是客观存在的，不是人为划分的：

- **L0 词汇**：Fire 是一个客观存在的元素概念。它不依赖任何游戏逻辑存在。
- **L1 能力**：BurnEffect 需要 Fire Tag 来描述自己是什么"类型"的效果。不能反过来。
- **L2 实体**：FireMage 需要 BurnEffect 来描述自己的能力。不能反过来。
- **L3 玩法**：DragonSlayer 任务需要 FireMage 角色和 DragonScale 物品。不能反过来。
- **L4 世界**：DragonPeak 地图需要 DragonSlayer 任务和 FireMage 出现在地图上。不能反过来。

分层就是**承认这种客观的抽象层级**。违反分层不是"风格问题"，而是**语义错误**——L0 引 L2 意味着"Fire 这个概念需要 FireMage 才能存在"，这在语义上是错误的。

### 2.2 分层带来的具体好处

1. **加载管线确定性**：L0 先保证注册完毕，L4 在加载时所有引用都已就绪
2. **影响范围可控**：修改 L1 EffectDef 不需要重新加载 L4 的地图
3. **Mod 兼容性**：Mod 主要在 L4 工作，L0-L2 稳定意味着 Mod 很少因核心内容更新而损坏
4. **内容团队分工**：L0/L1 由系统设计师维护，L3/L4 由关卡/叙事设计师维护

### 2.3 禁止的架构模式

- **Layer Leakage**：低层偷偷引用高层（如 ConditionDef 引用 CharacterDef）
- **Layer Skipping**：跳过中间层直接引用底层（如 QuestDef 直接硬编码 Tag 值而非引用 TagDef）
- **Layer Confusion**：将本应属于高层的 Def 放在低层目录（如 RecipeDef 放在 capabilities/ 目录下）
- **Layer Inversion**：高层逻辑进入低层（如 MapDef 级别的数据烘焙进 CharacterDef）

---

## 3. Content Pipeline 管线架构

### 3.1 管线五阶段

```
                  ┌─────────────────────────┐
                  │   1. Load               │
                  │   AssetServer::load()    │
                  │   按层序并行加载 RON      │
                  └──────────┬──────────────┘
                             │
                  ┌──────────▼──────────────┐
                  │   2. Deserialize         │
                  │   ron::from_str()        │
                  │   严格模式：未知字段报错   │
                  └──────────┬──────────────┘
                             │
                  ┌──────────▼──────────────┐
                  │   3. Validate            │
                  │   ContentValidator       │
                  │   8 项强制校验（见 3.2）  │
                  └──────────┬──────────────┘
                             │
                  ┌──────────▼──────────────┐
                  │   4. Register            │
                  │   DefRegistry<T>::insert  │
                  │   写入后的 Registry 可查询 │
                  └──────────┬──────────────┘
                             │
                  ┌──────────▼──────────────┐
                  │   5. Freeze              │
                  │   DefRegistry<T>::freeze  │
                  │   冻结后只读，禁止插入/修改 │
                  └─────────────────────────┘
```

### 3.2 8 项强制校验（详细定义）

#### V1: ID 唯一性

```
输入: 当前层所有 Def 的 ID 集合
校验: 同类型、跨类型均不可重复 ID
      TagDef::id("fire") 和 EffectDef::id("fire") 冲突（全局 ID 唯一）
错误: "Duplicate ID 'fire': registered as TagDef, also found in EffectDef"
```

#### V2: 引用存在性

```
输入: Def 中所有强类型 ID 引用字段
校验: 每个被引用的 ID 已在 Registry 中注册
错误: "EffectDef 'fireball_damage' references unknown ConditionId 'cond:is_burning'"
```

#### V3: 循环检测

```
输入: Content Dependency Graph（所有 Def 节点 + 跨 Def 引用边）
校验: DFS 检测是否存在反向边（back edge）
      同层引用和跨层引用一起检测
错误: "Cycle detected: AbilityDef 'fireball' → EffectDef 'fire_damage' → ConditionDef 'has_fire_tag' → AbilityDef 'fireball'"
```

#### V4: 枚举有效性

```
输入: Def 中所有 enum 类型字段
校验: 值匹配 enum 的已定义变体
错误: "EffectDef 'fire_damage' has invalid targeting_mode: 'SelfOnly' (expected one of: Single, AoE, Cone, Line, Global)"
```

#### V5: Tag 有效性

```
输入: Def 中所有引用的 TagId
校验: 所有 TagId 在 TagDef Registry 中注册
错误: "CharacterDef 'fire_mage' references unknown tag 'tag:ice_magic'"
```

#### V6: 资产存在性

```
输入: Def 中所有 Icon/VFX/SFX/Model 等资产路径
校验: 资产路径在 assets/ 目录下存在
错误: "AbilityDef 'fireball' references missing icon: 'assets/icons/spells/fireball.png'"
```

#### V7: Schema 兼容性

```
输入: Def 的 schema_version 字段
校验: 匹配当前代码中预期的 schema_version 范围
      不匹配时尝试 Migration（如已定义）
错误: "AbilityDef 'fireball' schema_version=2, expected 3 (migration available: add cooldown field)"
```

#### V8: Localization 完整性

```
输入: Def 中所有 name_key, desc_key 等 LocalizationKey 字段
校验: 每个 Key 在 Localization 数据库中有关联条目
      同一条目在所有已加载语言中均存在
错误: "AbilityDef 'fireball' name_key 'ability_fireball_name' missing in zh-CN locale"
```

### 3.3 Pipeline 错误处理

```
单条 Def 校验失败 → 标记该 Def 为 Failed，继续处理同层其他 Def
同层任意 Def 失败 → 整层加载失败，不进入下一层
全部层加载成功 → 冻结 Registry → 进入游戏就绪状态
```

- 校验失败必须提供清晰的错误信息（Def ID + 校验项 + 期望/实际值）
- 热重载时校验失败的 Def 保留旧版本不替换（不回退已加载的旧版）
- 开发模式（`feature = "dev"`）允许加载含警告的 Def，但不允许加载含错误的 Def

---

## 4. Content Dependency Graph

### 4.1 定义

Content Dependency Graph (CDG) 是一个**有向图**（DAG -- Directed Acyclic Graph），其中：

- **节点** = 每个注册的 Def（全局唯一 ID 标识）
- **有向边** = 从一个 Def 指向它引用的另一个 Def
- **层标记** = 每个节点标注其 Content Layer（L0-L4）

### 4.2 支撑的能力

| 能力 | 图算法 | 用途 |
|------|--------|------|
| 加载顺序确定 | 拓扑排序 | 按层+L0→L1→L2→L3→L4 层内拓扑序 |
| 循环引用检测 | DFS 反向边检测 | 阻止循环依赖进入 Registry |
| 引用完整性 | 遍历所有出边检查目标存在 | 无悬空引用 |
| 影响分析 | 反向边遍历（"谁引用了我"） | 修改前查影响范围 |
| 死内容检测 | 零入度孤立节点 | 未被引用的 Def 可标识为 Dead |
| 级联删除 | 反向边 BFS | 删除 Def 时标记所有受影响者 |

### 4.3 影响分析示例

```
修改 "eff:burn"（燃烧效果）
  ↓ 反向查询结果：
  ├── AbilityDef "fireball"          (L1)
  ├── AbilityDef "fire_wall"         (L1)
  ├── AbilityDef "flame_trap"        (L1)
  ├── CharacterDef "fire_mage"       (L2)  ← 间接：通过 fireball 引用
  ├── MonsterDef "fire_elemental"    (L2)  ← 直接：fire_elemental 有常驻 burn 效果
  ├── EncounterDef "volcano_ambush"  (L3)  ← 间接：通过 fire_elemental 引用
  └── MapDef "volcano_peak"          (L4)  ← 间接：通过 encounter 引用
```

---

## 5. Content Ownership

### 5.1 定义

每个 Def 有一个 `owner` 字段，标识该 Def 的**所属者**：

```ron
(
    id: "fireball",
    owner: ContentOwner("base_game"),     // 基础游戏内容
    // 或 DLC 包
    // owner: ContentOwner("dlc:echoes_of_war"),
    // 或 Mod
    // owner: ContentOwner("mod:epic_magic_pack"),
    // ...
)
```

### 5.2 所有权规则

- 每个 Def 有且仅有一个 Owner
- Owner 可以是 `base_game`、特定 DLC 包、特定 Mod
- 内容删除/卸载时必须按 Owner 级联处理

### 5.3 级联删除

```
删除 DLC "echoes_of_war"
  ↓
1. 查找所有 owner = "dlc:echoes_of_war" 的 Def
2. 对每个找到的 Def，反向查询 CDG 找出仅被它引用的 Def
3. 级联删除：Def → 仅被该 Def 引用的低层 Def
4. 跳过被其他 Owner 引用的 Def（保留，但可能产生"灰色引用"）

处理灰色引用：
  - 被删除的 Def 引用了仍在的 Def → 正常（遗留的引用可工作）
  - 仍在的 Def 引用了被删除的 Def → 标记为 MISSING_REFERENCE
  - MISSING_REFERENCE 在运行时变成空操作（安全降级）
```

---

## 6. Semantic Tags（语义标签）

### 6.1 定义

Semantic Tags 是**内容管理标签**，区别于 L0 定义的**游戏玩法标签**。

| 特性 | 玩法标签 (Gameplay Tag) | 语义标签 (Semantic Tag) |
|------|------------------------|------------------------|
| 用途 | 游戏逻辑判断（"这个单位免疫火焰"） | 内容管理（"这是一个新手技能"） |
| 定义位置 | L0 TagDef | Content Platform 内部 |
| 运行时可见 | 是，被 Effect/Modifier 引用 | 否，仅工具和编辑器可见 |
| 影响游戏逻辑 | 是 | 否 |
| 示例 | `tag:fire`, `tag:humanoid`, `tag:healing` | `sem:starter_skill`, `sem:elite_monster` |

### 6.2 语义标签分类

| 类别 | 标签 | 含义 |
|------|------|------|
| Content Status | `sem:stable` | 内容稳定，可供发布 |
| | `sem:deprecated` | 标记弃用，将在未来版本移除 |
| | `sem:experimental` | 实验性内容，可能不稳定 |
| | `sem:wip` | 未完成内容，不加载到发行版 |
| Content Phase | `sem:early_access` | 仅 Early Access 构建中包含 |
| | `sem:release` | 正式发布内容 |
| | `sem:post_launch` | 发售后通过更新添加 |
| Content Priority | `sem:critical_path` | 主线任务必须内容 |
| | `sem:side_content` | 支线内容 |
| | `sem:flavor` | 装饰性/彩蛋内容 |
| Content Rarity | `sem:common` | 常见/大量出现 |
| | `sem:rare` | 稀有/有限出现 |
| | `sem:unique` | 唯一/不可重复 |
| | `sem:quest_critical` | 任务关键/不可替代 |

### 6.3 语义标签的使用

```ron
// 在 Def 中定义
(
    id: "fireball",
    name_key: "...",
    desc_key: "...",
    tags: ["tag:fire", "tag:damage"],        // 玩法标签
    semantic_tags: ["sem:stable", "sem:core_skill"],  // 语义标签
)
```

语义标签用于：
1. **内容过滤器**：发行版构建排除 `sem:wip` 的 Def
2. **编辑器分类**：按 `sem:starter_skill` 过滤显示新手技能
3. **质量门禁**：`sem:deprecated` 的 Def 在加载时产生警告
4. **内容审计**：统计 `sem:quest_critical` 的 Def 确保关键路径完整

---

## 7. AI-Friendly Content

### 7.1 原则

Content Platform 的设计必须考虑 AI 作为内容生成工具：

- **规则清晰**：每个字段有明确的取值范围和语义，AI 不需要猜测
- **结构统一**：同类 Def 的结构完全一致，没有"特殊版本的 Fireball"
- **无魔法值**：特殊含义的值必须用显式枚举替代（`damage: -1` 表示秒杀 → `damage_mode: InstantKill`）
- **无隐藏规则**：所有影响游戏行为的参数都在 Def 字段中显式定义
- **自文档化**：字段名表达语义，必要时使用 Rust doc 注释

### 7.2 反模式

```
🟥 反模式 1: 魔法值
damage: -1          // -1 表示秒杀——需要阅读代码才能理解
改为:
damage: 999,
damage_mode: InstantKill,   // 显式声明

🟥 反模式 2: 隐含规则
effect_type: "periodic",
period_seconds: 6,           // 但隐含了 "periodic 效果每 tick 造成 1 次 damage"
改为:
effect_type: "periodic",
period_seconds: 6,
ticks_per_period: 1,
damage_per_tick: Some(15),   // 所有参数显式定义

🟥 反模式 3: 字符串条件
condition: "health < 50%"    // 需要解析字符串
改为:
condition: AttributeCompare {
    attribute_id: "attr:current_hp",
    operator: LessThan,
    value: Ratio(0.5),
}

🟥 反模式 4: 特殊 Case 枚举
damage_type: "fire"           // 需要检查 damage_type 是否支持 "fire"
改为:
damage_type: DamageTypeId("fire"), // 通过 Registry 校验存在性
```

### 7.3 AI 生成的最佳 Schema 特征

```
✅ 枚举优先于布尔：ActivationMode { Manual, Auto, Reaction } 优于 is_auto: bool + is_reaction: bool
✅ 结构体嵌套：将条件拆分为 ConditionBlock { conditions: Vec<ConditionClause>, logic: LogicOp } 优于扁平的 conditions: Vec<String>
✅ 显式可选域：Option<T> 比魔法值哨兵更清晰
✅ 版本字段：schema_version: u32 让 AI 知道这是可演进的 Schema
✅ 固定枚举值列表：AI 不需要猜测有哪些值
```

---

## 8. Content Stability Contract

### 8.1 契约内容

每个 Def 类型的 Schema 版本化遵循以下契约：

```
游戏代码和 Mod 之间承诺：
1. schema_version 相同的 Def 结构相同，游戏代码保证可反序列化
2. schema_version 升级必须提供 Migration 脚本（旧版 → 新版转换）
3. 游戏代码必须同时支持当前版本和前一个版本的 schema_version
4. 存档中引用的 Def 通过 id + schema_version 标识版本
```

### 8.2 schema_version 字段

```ron
// 每个 Def 结构体的第一个字段
(
    id: "fireball",
    schema_version: 3,        // ← 数据 Schema 版本号
    // ...
)
```

### 8.3 Migration 函数

```rust
// 在 Validation 阶段，如果 schema_version 不匹配，执行 Migration
// Migration 支持可串联（v1 → v2 → v3）

pub trait ContentMigration<T: ContentDef> {
    fn from_version(&self) -> u32;
    fn to_version(&self) -> u32;
    fn migrate(&self, def: T) -> Result<T, MigrationError>;
}
```

### 8.4 版本兼容矩阵

| Def 类型 | 当前版本 | 支持旧版 | 最后升级日期 | 迁移可用 |
|---------|---------|---------|-------------|---------|
| TagDef | 1 | 无 | — | 否 |
| EffectDef | 3 | v2 | 2026-06-01 | 是 |
| AbilityDef | 2 | v1 | 2026-05-15 | 是 |
| CharacterDef | 2 | v1 | 2026-05-20 | 是 |
| QuestDef | 1 | 无 | — | 否 |

---

## 9. Content Package

### 9.1 定义

Content Package 是一组相关 Def 的**逻辑单元**，跨 L0-L4 层打包：

```
FireMagePackage/
├── L1: AbilityDef "fireball", "fire_shield"
├── L1: EffectDef "burn", "fire_resistance"
├── L1: CueDef "vfx_fire_explosion", "sfx_fire_cast"
├── L1: TargetingDef "cone_fire"
├── L2: CharacterDef "fire_mage"
├── L2: EquipmentDef "flame_staff", "robe_of_embers"
├── L3: RecipeDef "flame_staff_recipe"
├── L4: CompanionDef "fire_mage_companion"
```

### 9.2 Package 的作用

- **内容完整性检查**：确保一个 "FireMage Package" 中所有必需的 Def 都存在
- **Mod 分发单位**：Mod 发布的最小单元
- **DLC 内容包**：DLC 的内容组织单位

### 9.3 Package Manifiest

```ron
// package_manifest.ron
(
    id: "package:fire_mage",
    name_key: "pkg_fire_mage_name",
    version: "1.0.0",
    content_ids: [
        "ability:fireball",
        "ability:fire_shield",
        "eff:burn",
        // ...
    ],
    dependencies: ["package:core_combat", "package:magic_foundation"],
)
```

---

## 10. Content Architect 治理原则

### 10.1 职责

| 职责 | 内容 |
|------|------|
| **Schema 仲裁** | 所有新增/修改的 Def 类型需经过 Content Architect 审查 |
| **分层合规** | 确保 Def 类型和实例落在正确的 Content Layer |
| **验证规则** | 定义和维护 8 项强制校验规则 |
| **平台演进** | 确保 Content Platform 架构支持 10,000+ 资产规模 |
| **Mod 兼容性** | 确保 Schema 变更不会破坏已有 Mod |

### 10.2 红线（禁止行为）

- **绕过 Registry**：任何直接解析 RON 文件或硬编码 Def 数据的代码
- **字符串引用**：使用字符串字面量引用另一个 Def（必须用强类型 ID）
- **Def 可变**：加载并冻结后的 Def 被运行时修改
- **Schema 不兼容变更**：升级 schema_version 时不提供 Migration
- **层违规**：Def 引用违反层间依赖方向
- **运行时缺失校验**：Def 中引用的资源在加载时未校验

### 10.3 审查清单

每个涉及 Content 的 PR/MR 必须检查：

```
[ ] 新增 Def 类型是否属于正确的 Content Layer？
[ ] 所有引用使用强类型 ID 而非字符串？
[ ] schema_version 字段存在且正确？
[ ] 所有 LocalizationKey 指向已注册的 Key？
[ ] 新增校验规则是否已加入 Validator？
[ ] Content Dependency Graph 是否正确反映了新增的引用？
[ ] Mod 能否通过同一管线注册新 Def 类型？
[ ] 该变更是否需要新增 ADR？存在已有的 ADR 覆盖吗？
[ ] AI 生成该 Def 类型的配置时是否会遇到歧义？
[ ] 这个设计在 10,000+ Def 规模下是否仍然可维护？
```

---

## 11. 平台宣言

我们不是在做"配置文件加载器"。我们在构建**游戏数据库引擎**。

```
Content Platform 承诺：

1. 每个 Def 都有身份（ID）、分类（Layer）、归属（Owner）
2. 每条引用都经过校验（Reference Integrity）
3. 每个错误都在加载时暴露（Fail Fast, Fail Early）
4. 每次 Schema 变更都版本化（Versioned Evolution）
5. 每段用户可见文本都本地化（Localization First）
6. 每个 Mod 都与基础游戏共享同一管线（First-Class Modding）
7. 每项规则都明确无隐藏（AI-Friendly Transparency）
8. 每种扩展都不以修改核心代码为代价（Open for Extension）
```

---

*本文档由 @content-architect 维护，是 Content Platform 的最高治理文件。任何违反本文档原则的架构决策必须经过 Content Architect 审查并记录例外理由。*
