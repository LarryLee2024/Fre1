---
id: 04-data.ll.13_Registry
title: "Registry 统一参考（铃兰+代码映射）"
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
tags:
  - data-architect
  - registry
  - infrastructure
  - ll
---

# Data Architecture Proposal — Registry 统一参考

## Domain Ownership

归属领域：**Registry**（基础设施域）
管辖范围：所有内容注册表的统一 Schema、加载协议、校验契约
上游依赖：所有 Core Domain（Attribute、Tag、Modifier、Effect、Ability 等）
下游消费：Content Layer、Modding Layer

## Problem

项目中存在 12+ 个 Registry，分属两套加载系统（新 trait-based vs 旧 RegistryLoader），需要统一参考文档确保：
1. 所有 Registry 遵循一致的 Def → Data → Registry 三段式
2. 校验规则可复用
3. Modding 层可通过相同接口扩展

## Schema Design

### 统一 Registry 接口

所有 Registry 共享以下 trait 契约（`src/shared/registry/`）：

```rust
// 只读查询接口
pub trait Registry: Send + Sync + 'static {
    type Key: Display + Hash + Eq + 'static;
    type Data: 'static;
    fn len(&self) -> usize;
    fn get(&self, key: &Self::Key) -> Option<&Self::Data>;
    fn contains(&self, key: &Self::Key) -> bool;
    fn keys(&self) -> Vec<&Self::Key>;
    fn iter(&self) -> Box<dyn Iterator<Item = (&Self::Key, &Self::Data)> + '_>;
}

// RON 加载接口
pub trait LoadableRegistry: Registry + Default + Sized {
    type Def: DeserializeOwned + 'static;
    type Error: std::error::Error + From<LoadError>;
    fn register_def(&mut self, def: Self::Def) -> Result<(), Self::Error>;
    fn load_from_dir(path: &str) -> Result<Self, Self::Error>;
    fn load_from_dir_vec(path: &str) -> Result<Self, Self::Error>;
    fn load_from_file(path: &str) -> Result<Self, Self::Error>;
}

// 单文件加载接口（用于 attributes.ron、tags.ron 等单文件配置）
pub trait LoadableSingleRegistry: Registry + Default + Sized {
    type Def: DeserializeOwned + 'static;
    type Container: DeserializeOwned + 'static;
    type Error: std::error::Error + From<LoadError>;
    fn register_defs(&mut self, container: Self::Container) -> Result<(), Self::Error>;
    fn load_from_file(path: &str) -> Result<Self, Self::Error>;
}

// 校验接口
pub trait ValidatableRegistry: Registry {
    fn validate(&self) -> Vec<ValidationError>;
}

// 跨 Registry 校验接口
pub trait CrossRegistryValidator: Send + Sync + 'static {
    fn validate(&self, provider: &dyn RegistryProvider) -> Vec<ValidationError>;
}
```

### 12 个 Registry 完整清单

| # | Registry | Key 类型 | Def 类型 | 数据层 | 加载方式 | RON 路径 |
|---|----------|----------|----------|--------|----------|----------|
| 1 | `AttributeRegistry` | `AttributeId` | `AttributeDef` | Definition | LoadableSingleRegistry | `content/attributes/attributes.ron` |
| 2 | `TagRegistry` | `TagId` | `TagDef` | Definition | LoadableSingleRegistry | `content/tags/tags.ron` |
| 3 | `ModifierRuleRegistry` | `String` | `ModifierRuleDef` | Definition | RegistryLoader（旧） | `content/modifiers/*.ron` |
| 4 | `EffectRegistry` | `String` | `EffectDefEntry` | Definition | RegistryLoader（旧） | `content/effects/*.ron` |
| 5 | `SkillRegistry` | `String` | `SkillData` | Definition | RegistryLoader（旧） | `content/skills/*.ron` |
| 6 | `BuffRegistry` | `String` | `BuffData` | Definition | RegistryLoader（旧） | `content/buffs/*.ron` ⚠️已废弃 |
| 7 | `EquipmentRegistry` | `String` | `EquipmentDef` | Definition | RegistryLoader（旧） | `content/equipments/*.ron` |
| 8 | `UnitTemplateRegistry` | `String` | `UnitTemplate` | Definition | RegistryLoader（旧） | `content/characters/*.ron` |
| 9 | `TerrainRegistry` | `String` | `TerrainDef` | Definition | RegistryLoader（旧） | `content/terrains/*.ron` |
| 10 | `LevelRegistry` | `String` | `LevelConfig` | Definition | RegistryLoader（旧） | `content/stages/*.ron` |
| 11 | `AiBehaviorRegistry` | `String` | `AiBehaviorData` | Definition | RegistryLoader（旧） | `content/ai_behaviors/*.ron` |
| 12 | `ConversionRegistry` | `AttributeId` | `AttributeConversion` | Definition | 新 trait-based | `content/attributes/` |

### 7-Layer DAG 初始化顺序

Registry 按依赖关系分 7 层初始化（`src/shared/registry/init.rs`）：

```
Layer 1: AttributeRegistry, TagRegistry（无依赖）
    ↓
Layer 2: ConversionRegistry（依赖 Attribute）
    ↓
Layer 3: ModifierRuleRegistry（依赖 Tag）
    ↓
Layer 4: EffectRegistry（依赖 Modifier）
    ↓
Layer 5: SkillRegistry, BuffRegistry（依赖 Effect, Tag）
    ↓
Layer 6: EquipmentRegistry, UnitTemplateRegistry, TerrainRegistry（依赖 Skill, Buff, Tag）
    ↓
Layer 7: LevelRegistry, AiBehaviorRegistry（依赖 Unit, Terrain）
```

### 两套加载系统对比

| 特性 | 新系统（trait-based） | 旧系统（RegistryLoader） |
|------|----------------------|-------------------------|
| trait | `LoadableRegistry` / `LoadableSingleRegistry` | `RegistryLoader` |
| 校验 | `ValidatableRegistry` + `CrossRegistryValidator` | 无内置校验 |
| 错误 | `LoadError`（Io + Ron） | 自定义 Error |
| DAG | 7-layer 自动排序 | 手动顺序 |
| 适用 | Attribute, Tag, Conversion | 其余 9 个 Registry |

### Registry 校验规则

每个 Registry 加载后必须执行校验：

| 校验类型 | 规则 | 严重级别 |
|----------|------|----------|
| ID 唯一性 | 同一 Registry 内 ID 不重复 | Error |
| 引用完整性 | 所有引用的 ID 在目标 Registry 中存在 | Error |
| 字段范围 | 数值在 min/max 范围内 | Error |
| 版本兼容 | RON version 字段与代码版本匹配 | Warning |
| i18n Key | name_key/desc_key 格式符合 `namespace.permanent_id.field` | Warning |
| Tag 引用 | 所有 Tag 引用在 TagRegistry 中已注册 | Error |

### 跨 Registry 校验矩阵

| 源 Registry | 引用目标 | 校验内容 |
|-------------|----------|----------|
| SkillRegistry | TagRegistry | tags 字段中的 Tag ID 存在 |
| SkillRegistry | EffectRegistry | effects 字段中的 Effect ID 存在 |
| BuffRegistry | TagRegistry | tags 字段中的 Tag ID 存在 |
| UnitTemplateRegistry | SkillRegistry | skill_ids 字段中的 Skill ID 存在 |
| UnitTemplateRegistry | TagRegistry | trait_ids 字段中的 Trait ID 存在 |
| UnitTemplateRegistry | AiBehaviorRegistry | ai_behavior 字段存在 |
| LevelRegistry | UnitTemplateRegistry | template 字段中的 Template ID 存在 |
| LevelRegistry | TerrainRegistry | terrain_grid 中的 Terrain char_code 存在 |
| EquipmentRegistry | TagRegistry | tags 字段中的 Tag ID 存在 |
| ModifierRuleRegistry | TagRegistry | source_tag / target_tag 存在 |

## Dependency Analysis

```
AttributeRegistry ──┐
                    ├──→ ConversionRegistry
TagRegistry ────────┤
                    ├──→ ModifierRuleRegistry
                    │       │
                    │       └──→ EffectRegistry
                    │               │
                    │               ├──→ SkillRegistry
                    │               ├──→ BuffRegistry（废弃）
                    │               └──→ EquipmentRegistry
                    │
                    └──→ UnitTemplateRegistry
                            │
                            └──→ LevelRegistry
```

## Validation Rules

| 规则 | 等级 | 说明 |
|------|------|------|
| VR-REG-001 | 🟥 绝对禁止 | Registry 不得包含业务逻辑（计算、状态变更） |
| VR-REG-002 | 🟥 绝对禁止 | Registry 加载失败时必须阻止启动（非降级） |
| VR-REG-003 | 🟩 必须遵守 | 所有 Registry 必须实现 ValidatableRegistry |
| VR-REG-004 | 🟩 必须遵守 | 跨 Registry 引用必须在启动时校验完整性 |
| VR-REG-005 | 🟩 必须遵守 | 新增 Registry 必须注册到 7-Layer DAG |
| VR-REG-006 | 🟨 优先选择 | 优先使用新 trait-based 系统，旧系统逐步迁移 |

## Replay Compatibility

- ✅ Registry 是只读数据，不参与运行时状态变更
- ✅ Registry 内容通过 RON 文件确定性加载
- ✅ Replay 只需记录 Seed + Command Stream，Registry 内容从 RON 重建

## Save Compatibility

- ✅ Registry 不需要持久化（Definition 层，从 RON 加载）
- ✅ Save 只保存 Instance 层数据，引用 Registry ID

## Migration Strategy

**旧系统迁移计划（ADR-030）**：

| Phase | 目标 Registry | 迁移内容 |
|-------|---------------|----------|
| Phase 1 | SkillRegistry, BuffRegistry | 迁移到 trait-based + LoadableRegistry |
| Phase 2 | EquipmentRegistry, UnitTemplateRegistry | 同上 |
| Phase 3 | ModifierRuleRegistry, EffectRegistry | 同上 |
| Phase 4 | TerrainRegistry, LevelRegistry, AiBehaviorRegistry | 同上 |

## Future Extension

- `RegistryProvider` trait 支持多源 Registry 查询（MOD 覆盖）
- `CrossRegistryValidator` 支持自定义校验规则注册
- Registry 支持热重载（Definition 层，战斗中禁止）

## Constitution Check

| 宪法条款 | 检查结果 |
|----------|----------|
| Data Law 001（Definition/Instance 分离） | ✅ Registry 存储 Definition，不存储 Instance |
| Data Law 003（配置只能引用 ID） | ✅ Registry 间通过 ID 引用，不内联定义 |
| Data Law 010（Replay 优先于便利） | ✅ Registry 只读，不影响确定性 |
