# ADR-030: ID 系统与 Registry 基础设施重构

## 状态
Accepted（2026-06-15）

## 背景

当前 ID 系统和 Registry 基础设施存在以下问题：

1. **无 `define_id!` 宏** — `docs/01-architecture/ids_design.md` 已定义宏规范但未实现，8+ 个 ID 类型各自手写 `new()`、`Display`、`From<&str>`、`From<String>`、`Hash`、`Eq`、`Serde` 等 50+ 行样板代码
2. **ID 重复定义** — `BuffId` 在 `shared/ids/buff_id.rs` 和 `core/buff/id.rs` 同时存在；`ItemId` 在 `shared/ids/item_id.rs` 和 `core/inventory/id.rs` 同时存在；`UnitId` 在 `shared/ids/unit_id.rs` 和 `core/character/components.rs` 是完全不同的类型
3. **22 个 Registry 实现不一致** — 15 个使用 `RegistryLoader` trait 但签名各异，7 个（Terrain/Level/Campaign/Execution/EffectHandler/ModifierRule/Trigger）使用手写加载逻辑
4. **所有 Registry 使用 `HashMap<String, T>`** — 不使用强类型 ID 作为 map key，运行时字符串比较无编译期类型安全
5. **无中心 Registry 管理系统** — 每个内容类型独立管理自己的 `load`/`get`/`list`/`validate`，无法统一执行引用完整性检查（Data Law 003：配置只引用 ID，不重复定义）

### 引用文档

- `docs/01-architecture/ids_design.md` — Strong ID newtype 设计、define_id! 宏规范
- `docs/01-architecture/03-data-config-asset/content-pipeline.md` — RON→Def→Data→Registry 数据流
- `docs/04-data/ll/data_relationship_overview.md` — ID 引用完整性矩阵
- `docs/04-data/ll/00_铃兰数据提取总览_ll.md` §五 — Data Laws 003

## 决策

### 1. 创建 `define_id!` 宏

在 `src/shared/ids/macro.rs` 中实现 `define_id!` 宏，一键生成完整 ID newtype：

```rust
// 使用示例
define_id!(SkillId, "skill");     // → Display: "skill(s_1001)"
define_id!(BuffId, "buff");       // → Display: "buff(b_001)"
define_id!(EffectId, "effect");   // → Display: "effect(e_001)"
define_id!(AbilityId, "ability"); // → Display: "ability(a_001)"
define_id!(UnitId, "unit");       // → Display: "unit(u_001)"
define_id!(ItemId, "item");       // → Display: "item(i_001)"
define_id!(ModifierId, "mod");    // → Display: "mod(m_001)"
define_id!(TagId, "tag");         // → Display: "tag(t_001)"
define_id!(TriggerId, "trigger"); // → Display: "trigger(tr_001)"
define_id!(TargetingId, "target");// → Display: "target(tg_001)"
define_id!(ExecutionId, "exec");  // → Display: "exec(ex_001)"
define_id!(StackingId, "stack");  // → Display: "stack(sk_001)"
define_id!(CueId, "cue");         // → Display: "cue(c_001)"
define_id!(TerrainId, "terrain"); // → Display: "terrain(te_001)"
define_id!(CampaignId, "campaign");
define_id!(StageId, "stage");
define_id!(FormulaId, "formula");
define_id!(RequirementId, "req");
define_id!(ConditionId, "cond");
define_id!(AiBehaviorId, "ai");
define_id!(TraitId, "trait");
```

宏生成的 Trait 实现：
- `new(id: impl Into<String>)` — 构造函数
- `as_str()` → `&str` — 内部字符串引用
- `Display` — `"prefix(id_string)"` 格式
- `FromStr` — 从 Display 格式解析
- `Serialize` / `Deserialize` — Serde 支持
- `Hash` + `Eq` + `Ord` — 集合操作
- `Clone` + `Debug` — 基础 trait
- `AsRef<str>` — 字符串转换

### 2. 统一 Registry trait

在 `src/shared/registry/` 中定义统一 Registry trait 族：

```rust
/// 所有内容注册表的统一接口
pub trait Registry {
    /// 内容类型唯一标识
    type Key: std::fmt::Display + Hash + Eq;
    /// 运行时数据类型
    type Data;

    /// 已注册项数量
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool { self.len() == 0 }

    /// 查询
    fn get(&self, id: &Self::Key) -> Option<&Self::Data>;
    fn contains(&self, id: &Self::Key) -> bool { self.get(id).is_some() }

    /// 遍历
    fn keys(&self) -> Vec<&Self::Key>;
    fn entries(&self) -> Vec<(&Self::Key, &Self::Data)>;
}

/// 支持从 RON 文件加载的注册表
pub trait LoadableRegistry: Registry {
    type Def: DeserializeOwned;  // RON 反序列化类型
    type LoadError: std::error::Error;

    /// 从单目录加载（每个文件一个记录）
    fn load_from_dir(path: &str) -> Result<Self, Self::LoadError>
    where Self: Sized;

    /// 从单个文件加载（数组格式）
    fn load_from_file(path: &str) -> Result<Self, Self::LoadError>
    where Self: Sized;

    /// 注册单个项目
    fn register(&mut self, id: Self::Key, data: Self::Data) -> Result<(), Self::LoadError>;
}

/// 支持引用完整性检查的注册表
pub trait ValidatableRegistry: Registry {
    type ValidationError;

    /// 检查所有跨注册表引用是否有效
    fn validate_references(&self, other_registries: &[&dyn AnyRegistry]) -> Vec<Self::ValidationError>;
}
```

### 3. 所有 Registry 变更为强类型 Key

| 当前 Registry | 当前 Key | 新 Key（Strong ID） |
|--------------|---------|-------------------|
| SkillRegistry | `HashMap<String, SkillData>` | `HashMap<SkillId, SkillData>` |
| BuffRegistry | `HashMap<String, BuffData>` | `HashMap<EffectId, EffectData>`（→ EffectRegistry） |
| UnitTemplateRegistry | `HashMap<String, UnitTemplate>` | `HashMap<UnitId, UnitTemplate>` |
| TraitRegistry | `HashMap<String, TraitData>` | `HashMap<TraitId, TraitData>` |
| EquipmentRegistry | `HashMap<String, EquipmentDef>` | `HashMap<EquipmentId, EquipmentDef>` |
| ItemRegistry | `HashMap<String, ItemDef>` | `HashMap<ItemId, ItemDef>` |
| AttributeRegistry | `HashMap<AttributeKind, AttributeDefinition>` | `HashMap<AttributeId, AttributeDefinition>` |
| TagRegistry | `HashMap<GameplayTag, TagDefinition>` | `HashMap<TagId, TagDefinition>` |
| TerrainRegistry | `HashMap<String, TerrainDef>` | `HashMap<TerrainId, TerrainDef>` |
| AiBehaviorRegistry | `HashMap<String, AiBehavior>` | `HashMap<AiBehaviorId, AiBehavior>` |
| CampaignRegistry | `HashMap<String, CampaignDef>` | `HashMap<CampaignId, CampaignDef>` |
| ExecutionRegistry | `HashMap<String, Box<dyn Execution>>` | `HashMap<ExecutionId, ExecutionDef>` + trait dispatch |
| EffectHandlerRegistry | `HashMap<String, Box<dyn EffectHandler>>` | 由 ExecutionRegistry 替代 |
| ModifierRuleRegistry | 自定义 | `HashMap<ModifierId, ModifierRule>` |

### 4. 删除重复 ID 定义

| 删除文件 | 保留替代 |
|----------|----------|
| `src/core/buff/id.rs` | `src/shared/ids/buff_id.rs`（但 BuffId 最终被 EffectId 替代） |
| `src/core/inventory/id.rs` | `src/shared/ids/item_id.rs` |
| `src/core/character/components.rs` 中 `struct UnitId` | 改为使用 `shared/ids::UnitId`（Component 改用 `name: UnitId` 字段） |

### 5. 新建 ID 类型

新增 Linglan 模型必需的 ID 类型：

| 新增 ID | 隶属 | 用途 |
|---------|------|------|
| `EffectId` | `shared/ids/` | 效果定义标识（替代旧 BuffId 的部分用途） |
| `AbilityId` | `shared/ids/` | 技能定义标识 |
| `TagId` | `shared/ids/` | 标签定义标识 |
| `ExecutionId` | `shared/ids/` | 执行算式标识 |
| `StackingId` | `shared/ids/` | 堆叠策略标识 |
| `TargetingId` | `shared/ids/` | 目标选择规则标识 |
| `CueId` | `shared/ids/` | 表现信号标识 |
| `TriggerId` | `shared/ids/` | 触发器标识 |
| `FormulaId` | `shared/ids/` | 公式标识 |
| `ModifierId` | `shared/ids/` | 修饰规则标识 |

### 6. Content Schema 标准化

所有 RON 配置文件必须包含以下通用字段：

```ron
// 新版 RON Schema（所有内容类型统一）
(
    id: "ability.a_001",           // 命名空间格式：{type}.{prefix}{number}
    version: 1,                     // Schema 版本，必须 >= 1
    name_key: Some("skill.s_1001.name"),  // ADR-017 国际化 Key（可选）
    desc_key: Some("skill.s_1001.desc"),  // ADR-017 国际化 Key（可选）
    tags: ["tag.t_001", "tag.t_002"],     // 分类标签引用
    // ... 类型专用字段
)
```

## Module Design

```
src/shared/
├── ids/
│   ├── mod.rs          # 所有 ID 重导出 + define_id! 宏引用
│   ├── define_id.rs    # define_id! 宏实现
│   └── types/          # 生成的 ID 类型（单个文件过多时按领域拆分）
├── registry/
│   ├── mod.rs          # Registry trait + LoadableRegistry trait
│   ├── validatable.rs  # ValidatableRegistry trait + 引用完整性检查
│   └── loader.rs       # RON 文件加载器（统一的 load_from_dir/load_from_file 实现）
```

## Communication Design

```
Content（RON 文件）
  │  AssetLoader 读取
  ↓
RegistryLoader::load_from_dir() / load_from_file()
  │  反序列化为 Def
  ↓
Validator::validate_references()  ← 跨 Registry 检查
  │  校验通过
  ↓
Registry<Key=StrongId, Data=RuntimeData>
  │  运行时查询
  ↓
Core System（只读 Registry Resource）
```

- 所有 Registry 在 App 启动阶段（PreStartup）完成加载
- 加载失败 → 阻止游戏启动（Core 内容）/ 降级警告（Content 内容）
- 运行时 Registry 为只读（`&Registry<T>`），禁止 `&mut Registry<T>`

## 边界定义

| 依赖 | 允许 | 禁止 |
|------|------|------|
| 所有层 → `shared/ids/` | 引用 ID 类型 | 在 `shared/ids/` 中添加业务逻辑 |
| Registry → `shared/ids/` | 使用强类型 ID 作为 key | 使用 String 作为 key |
| Domain 模块 → Registry | 通过 `Res<XxxRegistry>` 查询 | 运行时修改 Registry 内容 |

## Forbidden（禁止事项）

- 🟥 **禁止** 在 Phase 1 完成后任何地方使用 `HashMap<String, T>` 作为 Registry 内部存储
- 🟥 **禁止** 手写 ID newtype — 必须使用 `define_id!` 宏
- 🟥 **禁止** 保留 `src/core/buff/id.rs`、`src/core/inventory/id.rs` 等重复 ID 定义文件
- 🟥 **禁止** 在 Registry 中使用 `Box<dyn Registry>` 绕过类型系统
- 🟥 **禁止** 运行时 `&mut Registry<T>` — 注册表加载后必须不可变
- 🟥 **禁止** 使用 `String` 类型在跨模块接口中传递 ID — 必须使用强类型 `*Id`

## Definition / Instance Design

| 类型 | Def（RON 反序列化） | Data（运行时） | Registry |
|------|---------------------|---------------|----------|
| Skill | `SkillDef` | `SkillData` | `SkillRegistry: HashMap<SkillId, SkillData>` |
| Ability | `AbilityDef` | `AbilityData` | `AbilityRegistry: HashMap<AbilityId, AbilityData>` |
| Effect | `EffectDef` | `EffectData` | `EffectRegistry: HashMap<EffectId, EffectData>` |

## 后果

### 正面
- 所有 ID 类型由宏统一生成，消除 400+ 行样板代码
- 所有 Registry 有统一的查询/加载/校验接口
- 编译期类型安全：传递 `WrongIdType` 到 `WrongRegistry` 在编译期捕获
- 引用完整性检查集中化，避免运行时悬空引用崩溃
- 删除 3 个重复 ID 文件，清理技术债务

### 负面
- 22 个 Registry 全部需要重写内部存储类型
- RON 文件中所有 ID 引用需要更新为强类型序列化格式
- `define_id!` 宏本身的实现和测试需要额外投入
- 现有代码中所有 `HashMap<String, Xxx>` 查询需要更新

## 替代方案（已拒绝）

| 方案 | 拒绝原因 |
|------|----------|
| 使用 `uuid` crate | String 格式的 UUID 难以调试，且与命名空间 ID 格式不兼容 |
| 使用整型 ID（u64） | 无法从 ID 值反推类型和来源，调试困难，存档兼容性差 |
| 只在 Shared 层定义类型，不提供宏 | 仍需要手写每个 ID 的 Serde/Hash/Eq 实现 |
| 保留现有 ID 类型不变 | 不解决重复定义问题，继续容忍编译期类型不安全 |
