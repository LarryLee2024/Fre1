---
id: domains.element.schema.v1
title: Element Schema — 元素属性数据架构
status: stable
owner: data-architect
created: 2026-06-20
updated: 2026-06-20
layer: definition
replay-safe: true
---

# Element Schema — 元素属性数据架构

> **领域归属**: Domains — 战斗/魔法层 | **依赖 Schema**: 无（L0 自包含） | **定义依据**: `docs/03-content/definitions/vocabulary/element-def.md`

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `ElementDef` | Definition | 元素属性的静态定义（ID、显示元数据） |
| `ElementInteractionMatrix` | L3 Gameplay（延迟定义） | 元素间克制/削弱关系矩阵 |

---

## 2. Problem

Element 是游戏中与 DamageType 正交的元素属性维度，用于表达元素之间的克制关系、亲和度加成、以及技能的元素归属。Schema 必须解决：

- 元素属性的唯一标识和展示元数据
- 元素克制关系的层级归属（L3 Gameplay 层，不嵌入 L0）
- 克制关系的数据结构（L3 引用本 Schema）

---

## 3. Schema Design

### 3.1 ElementDef（Definition 层）

```rust
/// 元素属性定义——游戏世界中元素属性的唯一标识。
///
/// ElementDef 只定义元素的身份标识和展示元数据。
/// 元素的克制关系、亲和度加成由 L3 Gameplay 层定义，
/// 因为元素-元素克制关系涉及 ElementId-to-ElementId 引用
/// （L0-to-L0 引用），违反 L0 同层引用禁止规则。
///
/// 来源：`docs/03-content/definitions/vocabulary/element-def.md` §2
struct ElementDef {
    /// 元素唯一标识（前缀: `elem:`）
    id: ElementId,

    /// 元素名称本地化 Key
    name_key: LocalizationKey,

    /// 元素描述本地化 Key
    desc_key: LocalizationKey,

    /// Schema 版本号
    schema_version: u32,

    /// 图标 Key（用于 UI 中的元素标识）
    icon_key: Option<String>,

    /// 元素颜色（十六进制 RGB，用于 UI 着色）
    color_hex: Option<String>,
}
```

### 3.2 ElementInteractionMatrix（L3 Gameplay 层 — 延迟定义）

```rust
/// 元素交互矩阵——定义在 L3 Gameplay 层。
///
/// ElementDef 之间的克制/削弱关系涉及 ElementId-to-ElementId 引用，
/// 违反 L0 同层引用禁止规则。因此元素交互矩阵推迟到 L3 Gameplay 层定义。
///
/// 本 Schema 只给出数据结构蓝图，实际注册在 L3 层。
/// 一个只定义了 ElementDef 但不定义 ElementInteractionMatrix 的游戏
/// 仍然可以正常运行——所有元素的交互倍率为 1.0（无加成/削弱）。
///
/// 来源：`docs/03-content/definitions/vocabulary/element-def.md` §6
/// ── 标记为 L3（延迟到 Gameplay 层） ──
struct ElementInteractionMatrix {
    /// 每对元素之间的克制倍率
    /// elem:fire 对 elem:ice 造成 1.5x 伤害
    strengths: Vec<(ElementId, ElementId, f32)>,

    /// 每对元素之间的削弱倍率
    /// elem:fire 对 elem:fire 造成 0.5x 伤害（同类抗性）
    weaknesses: Vec<(ElementId, ElementId, f32)>,

    /// 元素-伤害类型映射
    /// elem:fire 增强 dmg:fire 和 dmg:burning
    damage_type_affinities: Vec<(ElementId, DamageTypeId, f32)>,
}
```

### 3.3 ElementResistance（Instance 层）

```rust
/// 实体对特定元素的抗性值。
/// 取值范围 [-100, +100]：
///   +100 = 完全免疫
///   0    = 无抗性
///   -100 = 极度脆弱（承受 2x 伤害）
struct ElementResistance {
    /// 目标元素
    element_id: ElementId,

    /// 抗性值
    value: i32,

    /// 抗性来源类型
    source: ResistanceSource,
}

enum ResistanceSource {
    /// 种族/生物类型基础抗性
    Racial,
    /// Buff/Effect 提供的临时抗性
    Buff,
    /// 装备提供的抗性
    Equipment,
    /// 剧情/天赋提供的永久抗性
    Permanent,
}
```

### 3.4 ElementAffinity（Instance 层 — L3 Gameplay 层）

```rust
/// 实体对特定元素的亲和度——影响元素技能的伤害加成。
/// L3 Gameplay 层定义，非 L0 Schema。
///
/// ── 标记为 L3（延迟到 Gameplay 层） ──
struct ElementAffinity {
    /// 目标元素
    element_id: ElementId,

    /// 亲和度倍率（1.0 = 无加成）
    affinity_multiplier: f32,

    /// 剩余持续回合数（None = 永久）
    remaining_turns: Option<u32>,
}
```

---

## 4. Layer Analysis

| 数据结构 | Layer | 持久化 | 可热重载 | 备注 |
|----------|-------|--------|----------|------|
| `ElementDef` | Definition | 是（配置文件） | 是 | 元素定义 |
| `ElementInteractionMatrix` | L3 Gameplay | 是（L3 配置） | 是 | L3 延迟定义 |
| `ElementResistance` | Instance | 是（通过 Save） | 否 | 实体元素抗性 |
| `ElementAffinity` | L3 Gameplay | 是（通过 Buff） | 否 | 元素亲和度 |

---

## 5. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 被依赖 | ← ExecutionSchema | 元素缩放公式引用 ElementId |
| 被依赖 | ← EffectSchema | 技能元素归属引用 ElementId |
| 被依赖 | ← CombatSchema | 元素克制计算引用 ElementId |

---

## 6. Validation Rules

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V1 | ID 格式合法 | Def 加载 | 必须匹配 `^elem:[a-z][a-z0-9_]+$` |
| V2 | `color_hex` 格式合法（若设置） | Def 加载 | 必须匹配 `^#[0-9A-Fa-f]{6}([0-9A-Fa-f]{2})?$` |
| V3 | 无跨 Def 引用 | Def 加载 | ElementDef 是 L0 Def，禁止引用其他 Def |
| V4 | 交互矩阵 ElementId 存在（L3 加载时） | L3 加载 | ElementInteractionMatrix 中所有 ElementId 在 DefRegistry<ElementDef> 中存在 |

---

## 7. Replay Compatibility

| 场景 | 兼容性 | 说明 |
|------|--------|------|
| ElementDef 加载 | 🟩 完全确定 | 配置文件静态确定 |
| 元素克制计算 | 🟩 完全确定 | 元素交互矩阵确定 |
| 抗性修正 | 🟩 完全确定 | ElementResistance 值确定 |

---

## 8. Save Compatibility

| 场景 | 兼容性 | 版本策略 |
|------|--------|----------|
| ElementDef | 🟩 | 纯配置文件，不参与存档 |
| ElementResistance | 🟩 | Save v1: 实体持久化包含抗性数据 |
| ElementAffinity | 🟩 | 通过 Buff 系统间接持久化 |

---

## 9. Migration Strategy

| 版本 | 变更 | 迁移策略 |
|------|------|----------|
| v1 | 初始版本 | — |
| v2（未来） | 新增元素复合（如 Fire+Ice = Steam） | ElementInteractionMatrix 新增组合类型 |

---

## 10. Future Extension

- **元素复合**: 两个元素组合产生新的交互效果（如 Fire + Ice = Steam 造成额外伤害）
- **动态元素**: 运行时根据环境/装备改变元素的属性和交互矩阵
- **元素觉醒**: 特定条件下元素发生质变（如 Fire → Hellfire）

---

## 11. Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| 交互矩阵膨胀 | 元素数量增加导致交互矩阵组合爆炸 | 克制关系稀疏存储，默认倍率 1.0 |
| 元素与伤害类型混淆 | 开发人员混淆 ElementId 和 DamageTypeId | 类型安全的 ElementId 和 DamageTypeId 区分 |
| L3 加载顺序依赖 | ElementDef 未注册时 L3 交互矩阵加载失败 | L3 校验阶段检查 ElementId 存在性 |

---

## 12. Constitution Check

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| L0 自包含 | ✅ | ElementDef 不引用任何其他 Def 类型 |
| 克制关系上移 L3 | ✅ | ElementInteractionMatrix 在 L3 定义 |
| 类型安全 | ✅ | 使用 ElementId 而非 TagId 确保编译期安全 |
| Def-Instance 分离 | ✅ | ElementDef 为纯 Definition，ElementResistance 为 Instance |
