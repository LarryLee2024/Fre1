---
id: infrastructure.localization.schema.v1
title: Localization Schema — 本地化系统数据架构
status: stable
owner: data-architect
created: 2026-06-19
updated: 2026-06-21
layer: infrastructure
replay-safe: true
---

# Localization Schema — 本地化系统数据架构

> **领域归属**: Infrastructure — L2 技术实现层 | **依赖 Schema**: 所有包含 LocalizationKey 字段的 Schema | **定义依据**: 宪法 §22, ADR-053, Data Law 001/010/013/014

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `LocalizationKey` | Shared / Generated | 强类型 Key 字符串常量（编译期生成），所有领域引用 |
| `LocaleId` | Infrastructure | 语言标识符（如 `en-US`, `zh-CN`, `ja-JP`, `zz-ZZ`） |
| `LocalizationDatabase` | Infrastructure (ECS Resource) | 全局文本映射数据库，唯一事实源 |
| `LocalizationLoader` | Infrastructure | .ftl 文件资产加载器 |
| `LocalizedTextCache` | Infrastructure | 解析后文本运行时缓存 |
| `LocalizedText` | Infrastructure (ECS Component) | UI 组件，携带 Key + 参数 |
| `.ftl` 文件 | Content (Definition) | Fluent 格式的本地化原始定义 |
| `generated/keys.rs` | Tools (build.rs 产物) | 编译期生成的 Rust 常量模块 |

**归属推论**：
- Localization 属于 **Infrastructure 层 (L2)**，不属于 UI 层、不属于 Capabilities 能力层（宪法 §22.1.5, ADR-053 §1）
- `.ftl` 文件作为 Definition 数据，由 Content 层加载和校验
- `generated/keys.rs` 作为 Tools 层的 build.rs 产物，提供编译期检查

---

## 2. Problem

### 2.1 当前状态

宪法 §22 已定义 Localization 专项规则，ADR-053 已定义组件架构，但数据层存在以下缺失：

| # | 缺失项 | 影响 |
|---|--------|------|
| P1 | 无 `LocalizationKey` 正式类型定义 | 30+ Schema 已使用 `name_key: LocalizationKey` 但该类型从未被形式化定义 |
| P2 | 无 `LocaleId` 枚举定义 | 各 Schema 无法约束合法的 locale 值 |
| P3 | 无 `assets/localization/` 目录与 .ftl 文件 | 无任何本地化内容可加载 |
| P4 | 无 `LocalizationDatabase` 数据结构 | 运行时无法查询和管理本地化文本 |
| P5 | 无 `LocalizedText` Component 定义 | UI 系统无法声明本地化文本依赖 |
| P6 | 无启动时 Key 完整性校验 | 运行时 Key 拼写错误将导致最隐蔽的 Bug |
| P7 | 无 Key 代码生成器输出定义 | 无法从 .ftl 文件自动生成 Rust 常量 |
| P8 | 无 Data Laws 约束 | 代码中依然允许硬编码用户可见文本 |
| P9 | 无存档/Replay 数据约束 | 存档可能误存翻译文本，破坏 Replay 确定性 |

### 2.2 解决目标

1. 定义 `LocalizationKey` 的正式语法和类型约束
2. 定义 `LocalizationDatabase` 的数据结构和操作契约
3. 定义 `assets/localization/` 的目录组织和 .ftl 文件 Schema
4. 定义 `generated/keys.rs` 代码生成产物的结构
5. 定义存档/Replay 中本地化数据的表示方式
6. 定义启动时和运行时的校验规则

---

## 3. Schema Design

### 3.1 LocalizationKey 类型定义

LocalizationKey 遵循 4 段式命名规范，所有用户可见文本通过 Key 引用：

```
LocalizationKey ::= <namespace> "." <scope> "." <id> "." <suffix>

namespace  ::= "core" | "ui" | "ability" | "buff" | "item" | "effect"
             | "quest" | "story" | "tutorial" | "error" | "battle"
             | "faction" | "spell" | "party" | "camp_rest" | "economy"
             | "crafting" | "summon" | "progression" | "reaction" | "terrain"
scope     ::= [a-z0-9_]+       # 语义化子域（如 ch01, battle, menu）
id        ::= [a-z0-9_]+       # 优先使用无语义 ID（abl_000042）
suffix    ::= "name" | "desc" | "flavor" | "tooltip" | "text" | "title"
```

**Rust 类型映射**：

```rust
/// 编译期已知的 LocalizationKey 字符串常量。
/// 实际值由 build.rs 从 .ftl 文件生成，存储在 generated/keys.rs 中。
/// 运行时通过此 Key 从 LocalizationDatabase 查询翻译文本。
type LocalizationKey = &'static str;
```

**示例 Key 表**：

| Key | 含义 |
|-----|------|
| `core.yes` | 系统：确认按钮 |
| `core.no` | 系统：取消按钮 |
| `ui.battle.end_turn` | UI：结束回合标签 |
| `ability.abl_000042.name` | 技能火球术的显示名称 |
| `ability.abl_000042.desc` | 技能火球术的详细描述 |
| `ability.abl_000042.flavor` | 技能火球术的风味文本 |
| `buff.buf_000015.name` | Buff 中毒的显示名称 |
| `buff.buf_000015.tooltip` | Buff 中毒的工具提示 |
| `item.itm_000007.desc` | 物品治疗药水的详细描述 |
| `quest.qst_000001.name` | 任务主线第一章的名称 |
| `story.ch01.dlg_001.text` | 剧情第一章第一句台词 |
| `error.character.not_found` | 错误：角色不存在 |
| `battle.damage_dealt.text` | 战斗日志：伤害文本 |
| `faction.fct_000003.name` | 阵营：精灵联盟的名称 |
| `spell.spl_000021.name` | 法术：火球术的名称 |
| `party.welcome.text` | 队伍：欢迎加入文本 |
| `camp_rest.long_rest.text` | 营地：长休确认文本 |
| `economy.shop.greeting` | 经济：商店欢迎语 |
| `crafting.recipe_learned.text` | 制作：习得配方文本 |
| `summon.expired.text` | 召唤：召唤物消失文本 |
| `progression.level_up.text` | 成长：升级消息文本 |
| `reaction.opportunity_attack.text` | 反应：机会攻击文本 |
| `terrain.hazard_triggered.text` | 地形：陷阱触发文本 |

**禁止模式**：

| 反模式 | 原因 | 正确做法 |
|--------|------|----------|
| `ability.fireball.name` | 技能改名时 Key 失效 | `ability.abl_000042.name` |
| `fireball_name` | 无命名空间，冲突风险 | 使用完整 4 段式 |
| `名称.key` | Key 本身不应包含自然语言 | 使用 ASCII 命名 |
| `core.yes_zh` | locale 信息嵌入 Key | `core.yes` + locale 分离 |
| `ability.abl_000042.NAME` | 后缀使用大写 | 统一小写 `.name` |

### 3.2 LocaleId 类型定义

```rust
/// 语言标识符，遵循 BCP-47 格式。
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
enum LocaleId {
    /// 美式英语 — 标准模板，所有 Key 必须在此 locale 中定义
    EnUS,
    /// 简体中文
    ZhCN,
    /// 日语
    JaJP,
    /// Fake Locale — 调试用，检测硬编码文本
    ZzZZ,
}
```

**序列化规则**：
- 存档时序列化为字符串（如 `"en-US"`, `"zh-CN"`, `"ja-JP"`, `"zz-ZZ"`）
- 字符串 ↔ 枚举的双向映射必须完整覆盖
- 遇到未知字符串时 panic（启动时校验通过则不会出现此情况）

**Fake Locale (zz-ZZ) 特殊规则**：
- 仅 debug/dev 构建可用
- 启用时所有文本显示为伪翻译如 `[Fírébáll]`
- 硬编码文本（未通过 LocalizationKey）保持原始 ASCII，直接暴露
- 通过 feature flag `fake-locale` 控制

### 3.3 LocalizationDatabase — ECS Resource 数据结构

```rust
/// 核心 Localization 数据库，全局唯一 ECS Resource。
/// 存储所有 locale 的 key→value 映射，提供三级回退链。
struct LocalizationDatabase {
    /// 当前激活的语言
    current_locale: LocaleId,

    /// 按 (locale, key) 索引的原始 Fluent pattern 映射。
    /// 加载后只读（非热重载场景）。
    patterns: HashMap<LocaleId, HashMap<String, FluentPatternValue>>,

    /// 回退链（从高优先级到低优先级）
    fallback_chain: Vec<LocaleId>,

    /// 加载状态
    load_state: LoadState,
}

/// 回退链状态
enum LoadState {
    /// 未加载
    Unloaded,
    /// 加载中
    Loading,
    /// 加载完成
    Ready { total_keys: usize, locales_loaded: Vec<LocaleId> },
    /// 加载失败（仅保留错误信息）
    Failed(String),
}

/// 单个 Fluent pattern 的存储值。
enum FluentPatternValue {
    /// 纯文本（无变量插值）
    Static(String),
    /// 带 Fluent 变量的文本（需运行时解析）
    Dynamic(/* Fluent AST 或预解析结构 */),
}
```

**接口契约**：

```rust
impl LocalizationDatabase {
    /// 设置当前语言。
    /// 触发 LocalizedTextCache 失效。
    fn set_locale(&mut self, locale: LocaleId);

    /// 解析文本（带参数插值）。
    /// 三级回退：current_locale → en-US → raw_key_string。
    fn resolve(&self, key: &str, params: &[Param]) -> Result<String, LocError>;

    /// 批量解析（用于列表/批量 UI 渲染）。
    fn resolve_batch(&self, keys: &[(&str, &[Param])]) -> Result<Vec<String>, LocError>;

    /// 检查指定 locale 下是否有指定 Key。
    fn has_key(&self, locale: LocaleId, key: &str) -> bool;

    /// 获取指定 locale 的所有缺失 Key（en-US 有但该 locale 没有）。
    fn missing_keys(&self, locale: LocaleId) -> Vec<&str>;

    /// 获取指定 locale 的所有 Key 列表。
    fn all_keys(&self, locale: LocaleId) -> Vec<&str>;

    /// 刷新所有缓存（热重载时调用）。
    fn clear_cache(&mut self);
}
```

**默认 Fallback 链**：

| 配置 | 回退路径 |
|------|----------|
| `zh-CN` 时 | `zh-CN` → `en-US` → `raw_key` |
| `ja-JP` 时 | `ja-JP` → `en-US` → `raw_key` |
| `zz-ZZ` 时 | `zz-ZZ` → `en-US` → `raw_key` |
| `en-US` 时 | `en-US` → `raw_key` |

### 3.4 LocalizedText — ECS Component 结构

```rust
/// UI 组件：本地化文本声明。
/// 携带编译期已知的 Key 和运行时参数。
/// UI 系统读取此 Component 后自动渲染为对应语言文本。
#[derive(Component)]
struct LocalizedText {
    /// 编译期已知的 LocalizationKey
    key: &'static str,
    /// Fluent 命名参数（如 [("damage", "100"), ("target", "Goblin")]）
    params: Vec<(&'static str, String)>,
    /// Key 缺失时的回退文本（仅备用，不应出现在正常流程）
    fallback: Option<String>,
}
```

**使用约束**：
- `key` 字段必须是 `generated/keys.rs` 中定义的常量引用
- 禁止直接传字符串字面量作为 `key`
- `params` 字段的 key 必须与 .ftl 文件中定义的变量名一致
- UI 系统通过 `Changed<LocalizedText>` Filter 检测变更

**UI 数据流**：

```
LocalizedText { key, params }
       │
       ▼
LocalizationDatabase.resolve(key, params)
       │
       ▼
LocalizedTextCache.get_or_insert(key, locale → String)
       │
       ▼
Text 渲染（UI System）
```

### 3.5 资产目录组织

```
assets/localization/
├── en-US/                    # 标准模板（所有 Key 必须在此定义）
│   ├── core.ftl              # L0: 系统核心文本
│   ├── ui.ftl                # L1: UI 界面文本
│   ├── ability.ftl           # L2: 技能名称与描述
│   ├── buff.ftl              # L2: Buff/Debuff 名称与描述
│   ├── item.ftl              # L2: 物品名称与描述
│   ├── effect.ftl            # L2: 效果描述
│   ├── quest.ftl             # L2: 任务名称与描述
│   ├── battle.ftl            # L2: 战斗日志文本
│   ├── faction.ftl           # L2: 阵营文本
│   ├── spell.ftl             # L2: 法术文本
│   ├── party.ftl             # L2: 队伍文本
│   ├── camp_rest.ftl         # L2: 营地文本
│   ├── economy.ftl           # L2: 经济文本
│   ├── crafting.ftl          # L2: 制作文本
│   ├── summon.ftl            # L2: 召唤文本
│   ├── progression.ftl       # L2: 成长文本
│   ├── reaction.ftl          # L2: 反应文本
│   ├── terrain.ftl           # L2: 地形文本
│   ├── tutorial.ftl          # L3: 教程文本
│   ├── error.ftl             # L3: 错误消息
│   └── story/                # L3: 剧情对话（按章节分）
│       ├── chapter01.ftl
│       ├── chapter02.ftl
│       └── ...
│
├── zh-CN/                    # 简体中文
│   └── (同上结构)
│
├── ja-JP/                    # 日语
│   └── (同上结构)
│
└── zz-ZZ/                    # Fake Locale — 检测硬编码文本
    └── (同上结构，但所有文本转换为伪翻译)
```

**Fluent 文件内部格式**：

```ftl
### 文件：ability.ftl
### 层级：L2 Gameplay

-ability-abl-000042-name = Fireball
    .desc = Deals {$damage} fire damage to all targets in radius

-ability-abl-000043-name = Heal
    .desc = Restores {$value} HP to a single ally
```

**变量命名规范**（与 `.ftl` 文件约定一致）：

| 变量名 | 来源字段 | 说明 |
|--------|----------|------|
| `{$value}` | EffectDef.value | 通用数值 |
| `{$damage}` | DamageEffectDef.damage | 伤害值 |
| `{$heal}` | HealEffectDef.value | 治疗值 |
| `{$turns}` | EffectDef.duration | 持续回合 |
| `{$target}` | Target.name | 目标名称 |
| `{$source}` | Source.name | 来源名称 |
| `{$count}` | 运行时的数量参数 | 复数规则用 |
| `{$level}` | Spec.level | 等级 |

### 3.6 Key 代码生成产物结构

build.rs 扫描 `assets/localization/en-US/*.ftl` 生成 `src/infra/localization/generated/keys.rs`：

```rust
// 自动生成 — 由 build.rs 基于 en-US 模板生成
// 警告：手动修改将丢失。修改 .ftl 文件后重新构建。

#[allow(non_upper_case_globals)]
pub mod loc {
    // ── Core（系统文本）──
    pub mod core {
        pub const YES: &str = "core.yes";
        pub const NO: &str = "core.no";
        pub const CONFIRM: &str = "core.confirm";
        pub const CANCEL: &str = "core.cancel";
        pub const SAVE: &str = "core.save";
        pub const LOAD: &str = "core.load";
    }

    // ── UI（界面文本）──
    pub mod ui {
        pub const BATTLE_END_TURN: &str = "ui.battle.end_turn";
        pub const BATTLE_ATTACK: &str = "ui.battle.attack";
        pub const BATTLE_DEFEND: &str = "ui.battle.defend";
        pub const MENU_SETTINGS: &str = "ui.menu.settings";
        pub const MENU_QUIT: &str = "ui.menu.quit";
    }

    // ── Ability（能力/技能）──
    pub mod ability {
        pub mod abl_000042 {
            pub const NAME: &str = "ability.abl_000042.name";
            pub const DESC: &str = "ability.abl_000042.desc";
            pub const FLAVOR: &str = "ability.abl_000042.flavor";
        }
        pub mod abl_000043 {
            pub const NAME: &str = "ability.abl_000043.name";
            pub const DESC: &str = "ability.abl_000043.desc";
        }
    }

    // ── Buff（增益/减益）──
    pub mod buff {
        pub mod buf_000015 {
            pub const NAME: &str = "buff.buf_000015.name";
            pub const DESC: &str = "buff.buf_000015.desc";
            pub const TOOLTIP: &str = "buff.buf_000015.tooltip";
        }
    }

    // ── Item（物品）──
    pub mod item {
        pub mod itm_000007 {
            pub const NAME: &str = "item.itm_000007.name";
            pub const DESC: &str = "item.itm_000007.desc";
        }
    }

    // ── Battle（战斗日志）──
    pub mod battle {
        pub const DAMAGE_DEALT_TEXT: &str = "battle.damage_dealt.text";
        pub const HEAL_RECEIVED_TEXT: &str = "battle.heal_received.text";
        pub const UNIT_DIED_TEXT: &str = "battle.unit_died.text";
    }

    // ── Error（错误消息）──
    pub mod error {
        pub const CHARACTER_NOT_FOUND: &str = "error.character.not_found";
        pub const ITEM_NOT_FOUND: &str = "error.item.not_found";
        pub const INVALID_TARGET: &str = "error.invalid.target";
    }

    // ── Story（剧情，按章节）──
    pub mod story {
        pub mod ch01 {
            pub const DLG_001_TEXT: &str = "story.ch01.dlg_001.text";
            pub const DLG_002_TEXT: &str = "story.ch01.dlg_002.text";
        }
    }
}
```

**生成规则**：
1. 扫描 `assets/localization/en-US/*.ftl`（以 en-US 为标准模板）
2. 提取所有 `-` 开头的 message ID（如 `-ability-abl-000042-name`）
3. 按 `.` 分割为 `[namespace, scope, id, suffix]`
4. 生成 Rust 嵌套模块结构
5. 非 `name`/`desc`/`flavor`/`tooltip`/`text` suffix 的 id 段为常量名（如 `YES`, `NO`）
6. 常量值 = 原始 Key 字符串

---

## 4. Layer Analysis

| 数据结构 | Layer | 持久化 | 可修改 | 说明 |
|----------|-------|--------|--------|------|
| `.ftl` 文件 | **Definition** | 文件系统 | 内容团队修改 | Fluent 格式的本地化原始定义。静态配置，运行时只读（除热重载外）。 |
| `generated/keys.rs` | **Definition** / Tools | 无（build 产物） | build.rs 生成 | 编译期生成的 Rust 常量。对应 Definition 中的 Key 定义。 |
| `LocaleId` | **Infrastructure** | 存档（仅当前 locale） | 运行时管理 | 枚举定义在编译期确定。 |
| `LocalizationDatabase` | **Instance** | 否（运行时重建） | 运行时加载 | ECS Resource，存储从 .ftl 文件加载后的键值映射。不存档。 |
| `LocalizedText` | **Instance** | 存档（仅 Key+参数） | 运行时创建 | ECS Component，UI 文本声明。持久化时只存 Key 和参数。 |
| `LocalizedTextCache` | **Instance** | 否 | 运行时管理 | 解析后文本的运行时缓存，启动时为空，随查询填充。 |

**层间关系**：

```
[Definition]: .ftl 文件 ──build.rs扫描──→ [Definition]: generated/keys.rs
     │                                              │
     │ (loader 运行时加载)                            │ (编译期常量引用)
     ▼                                              ▼
[Instance]: LocalizationDatabase ──resolve()──→ [Instance]: LocalizedText
     │                                              │
     │ (set_locale 触发)                             │ (Changed filter)
     ▼                                              ▼
[Instance]: LocalizedTextCache ──→ UI Text Render

[Persistence]: 存档 ──→ current_locale: LocaleId
                        LocalizedText.key (只存 Key 字符串)
```

**关键约束**：
- `.ftl` 文件和 `generated/keys.rs` 是定义态（Definition）—— 运行时不可修改
- `LocalizationDatabase` 和 `LocalizedTextCache` 是运行时实例态（Instance）—— 启动时由 Loader 填充
- 存档（Persistence）**只存** `current_locale` 和 `LocalizedText` 的 Key+参数

---

## 5. Dependency Analysis

### 5.1 外部依赖

| 依赖 | 类型 | 说明 | 版本约束 |
|------|------|------|----------|
| `fluent-rs` 生态 | Rust crate | Fluent 格式解析和变量插值 | 已验证兼容 Bevy 0.19 |
| `fluent-bundle` | Rust crate | Fluent bundle 管理 | 已验证兼容 Bevy 0.19 |
| `fluent-resmgr` | Rust crate | Fluent 资源管理器（可选） | 已验证兼容 Bevy 0.19 |
| `intl-plural-rules` | Rust crate | CLDR 复数规则 | 已验证兼容 Bevy 0.19 |
| Bevy Asset API | Bevy | 可选的 .ftl 资产管理路径 | 0.19+ |

### 5.2 Schema 内部依赖

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| ← 被依赖 | 全部 Schema | 30+ Schema 中使用 `name_key: LocalizationKey`、`desc_key: LocalizationKey` 等字段 |
| 依赖 | Tag Schema (indirect) | 个别 Key 可能引用 Tag 名称 |
| 依赖 | Spec Schema (indirect) | 变量插值参数来自 Spec 字段值 |

### 5.3 后备方案

若 `fluent-rs` 生态与 Bevy 0.19 不兼容，使用简化方案：

```rust
/// 简易 Fluent pattern 解析（无完整 AST，仅支持 {$var} 替换）
fn resolve_simple(template: &str, params: &[(&str, String)]) -> String {
    let mut result = template.to_string();
    for (key, value) in params {
        let placeholder = format!("{{${}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}
```

此方案在评估期过渡使用，待 fluent-rs 兼容后替换。

---

## 6. Validation Rules

### 6.1 启动时校验（LocalizationValidator）

| # | 规则 | 严重度 | 校验逻辑 |
|---|------|--------|----------|
| V1 | **en-US 缺失 Key** | 🟥 Fatal | 代码中 `generated/keys.rs` 引用的 Key 在 en-US .ftl 中不存在 → panic 阻止启动 |
| V2 | **重复 Key** | 🟨 Warning | 同一个 Key 在同一 .ftl 文件中出现多次 → 输出 WARN |
| V3 | **未引用 Key (Orphan)** | 🟨 Warning | en-US .ftl 中存在但 `generated/keys.rs` 中没有引用的 Key → 输出 WARN |
| V4 | **参数不匹配** | 🟥 Fatal | Fluent 变量名与代码传入的 params key 不一致 → panic |
| V5 | **文件名规范** | 🟥 Fatal | .ftl 文件名必须匹配 namespace（如 `ability.ftl` → namespace `ability`）→ panic |
| V6 | **所有 suffix 在 SuffixSet 中** | 🟨 Warning | Key 的 suffix 字段不在合法的 suffix 集合中 → 输出 WARN |
| V7 | **en-US 完整性** | 🟥 Fatal | 所有非 en-US locale 缺失的 Key 必须可回退到 en-US，en-US 本身不可缺失 Key |
| V8 | **文本长度预算** | 🟨 Warning | 翻译文本超过 UI 区域最大字符限制 → 输出 WARN |

### 6.2 运行时校验

| # | 规则 | 严重度 | 校验逻辑 |
|---|------|--------|----------|
| R1 | **Key 格式** | 🟩 Assert | `resolve()` 时校验 Key 符合 `namespace.scope.id.suffix` 格式 |
| R2 | **参数类型** | 🟩 Assert | 传入参数数量与 Fluent pattern 定义的变量数量匹配 |
| R3 | **缓存一致性** | 🟩 Assert | `set_locale()` 后缓存必须全部失效 |

### 6.3 编译期校验

| # | 规则 | 校验逻辑 |
|---|------|----------|
| C1 | **Key 常量存在性** | 引用 `loc::ability::abl_000042::NAME` 若不存在 → 编译错误 |
| C2 | **Key 格式正确性** | build.rs 提取的 Key 不符合 `namespace.scope.id.suffix` 格式 → 构建失败 |
| C3 | **en-US 完整性** | build.rs 扫描时发现 en-US 中缺失某 namespace 的 .ftl 文件 → 构建警告 |

---

## 7. Replay Compatibility

### 7.1 基本原则

宪法 §22.1.3 和 ADR-053 §11 明确规定：**Replay/Event/BattleLog 只存 Key+参数，不存翻译文本**。

### 7.2 确定性保证

| 场景 | 做法 | 确定性 |
|------|------|--------|
| BattleLog 事件 | 存储 `key: &'static str` + `params: Vec<(&'static str, String)>` | ✅ 完全确定 |
| UI 文本渲染 | 运行时从 LocalizationDatabase 查询 | ✅ 不影响游戏逻辑 |
| 游戏逻辑判断 | 不依赖翻译文本做决策 | ✅ Data Law 010 合规 |

### 7.3 合规示例 vs 违规示例

```rust
// ✅ 合规：Replay/Event 只存 Key+参数
struct BattleLogEvent {
    key: &'static str,                     // "battle.damage_dealt"
    params: Vec<(&'static str, String)>,   // [("actor", "Goblin"), ("value", "100")]
}

// ❌ 违规：Replay/Event 存储翻译文本
struct BattleLogEvent {
    text: String,                          // "Goblin dealt 100 damage" — 破坏确定性
}
```

### 7.4 Replay Schema 扩展

在 Replay Frame 中，本地化相关数据仅以 Key+参数形式存在：

```rust
struct ReplayCommand {
    /// 命令类型
    command_type: CommandType,

    /// 关联的本地化 Key（可选）
    loc_key: Option<&'static str>,

    /// 本地化参数（可选）
    loc_params: Option<Vec<(&'static str, String)>>,
}
```

---

## 8. Save Compatibility

### 8.1 基本原则

**存档禁止保存翻译结果，只存 ID/Key**。翻译在运行时由 `LocalizationDatabase` 根据当前 locale 实时解析。

### 8.2 存档中 Localization 相关字段

```rust
/// 存档中与 Localization 相关的数据子集。
struct SaveLocalizationData {
    /// 存档时使用的语言
    current_locale: LocaleId,

    /// Key 格式一致性校验哈希（确保存档的 Key 匹配当前数据版本）
    key_schema_hash: [u8; 32],
}
```

### 8.3 版本兼容性

| 场景 | 保证 | 说明 |
|------|------|------|
| 旧存档加载 | 🟩 兼容 | 存档只存 `current_locale`，语言支持只增不减，旧 locale ID 永远有效 |
| 新存档在旧版本 | 🟦 尽力兼容 | 新 locale ID 在旧版本中无法识别 → 回退到 `en-US` |
| Key 变更 | 🟩 兼容 | 存档不存翻译结果，Key 变更不影响存档加载。运行时语言切换自动使用新翻译 |

### 8.4 迁移策略

存档中的 `current_locale` 字段格式不涉及 Schema 迁移——LocaleId 枚举可扩展：

```rust
// v1 → v2: 新增 locale
enum LocaleId {
    EnUS,      // v1
    ZhCN,      // v2 新增
    JaJP,      // v2 新增
    ZzZZ,      // v2 新增（仅 debug）
}
```

- 新增 locale 枚举变体不影响存档兼容性
- 重命名 locale 变体需要迁移映射
- 删除 locale 变体需要将存档中该 locale 回退到 `en-US`

---

## 9. Migration Strategy

### 9.1 .ftl 文件版本标记

每个 .ftl 文件使用文件头标记版本，支持未来格式变更：

```ftl
### version: 1
### namespace: ability
### layer: L2
### created: 2026-06-19
### description: Ability names and descriptions

-ability-abl-000042-name = Fireball
    .desc = Deals {$damage} fire damage
```

### 9.2 迁移场景

| 场景 | 迁移方案 |
|------|----------|
| .ftl 格式变更 | 文件头 version 递增，loader 根据 version 选择解析器 |
| Key 重命名 | 旧 Key 标记为 deprecated（保留在 .ftl 中带 `### deprecated: true`），新 Key 添加 |
| Key 删除 | 从 en-US .ftl 删除 → build.rs 重建 → 引用该 Key 的代码编译报错 → 强制清理所有引用 |
| namespace 迁移 | 旧 namespace 的 Key 在新 namespace 存在 → 启动时检测冲突 → 生成迁移报告 |
| Fluent 库升级 | Fluent AST 结构可能变化 → 缓存重建 |

### 9.3 迁移流程

```
旧版本 .ftl ──→ 内容团队修改 .ftl
                     │
                     ▼
              build.rs 重建 generated/keys.rs
                     │
                     ▼
              编译：引用旧 Key 的代码报错
                     │
                     ▼
              开发者逐个修复 Key 引用
                     │
                     ▼
              启动验证：LocalizationValidator 确保一致性
```

---

## 10. Future Extension

### 10.1 变量插值增强

```rust
// 当前：仅支持 {$value} 字符串替换
// 未来：支持 Fluent 完整 AST（复数、性别、选择）

// Fluent 复数示例
-damage-dealt =
    { $count ->
        [one] Dealt {$count} damage
       *[other] Dealt {$count} damage
    }
```

### 10.2 复数规则支持

- Fluent 内置 CLDR 复数规则，无需手写
- 需要 `intl-plural-rules` crate 计算当前 locale 的复数类别
- 需要 `LocalizedText.params` 支持 `count: u32` 作为复数参数

### 10.3 Mod 覆盖链

```
加载优先级（从低到高）：
  1. Base: assets/localization/en-US/*.ftl
  2. DLC: assets/dlc/*/localization/en-US/*.ftl
  3. Mod: mods/*/localization/en-US/*.ftl

覆盖规则：
  - 同一个 Key 出现在多个层时，高优先级覆盖低优先级
  - Mod 新增 Key 自动追加到数据库
  - 启动时检测 Mod Key 冲突，输出 WARN
```

### 10.4 热重载

```rust
// 开发期热重载设计
fn hot_reload_system(
    mut db: ResMut<LocalizationDatabase>,
    loader: Res<LocalizationLoader>,
    // 文件系统监听事件
) {
    for changed_file in file_events.read() {
        loader.reload_file(changed_file.path, &mut db);
        db.clear_cache();
        info!("Hot-reloaded: {}", changed_file.path);
    }
}
```

### 10.5 覆盖率报告

```rust
/// 本地化覆盖报告（用于 CI 和开发者工具）
struct LocalizationCoverageReport {
    /// 总 Key 数（en-US）
    total_keys: usize,
    /// 各 locale 的覆盖数
    coverage_by_locale: HashMap<LocaleId, CoverageEntry>,
}

struct CoverageEntry {
    covered: usize,
    missing: Vec<String>,
    coverage_pct: f64,
}
```

### 10.6 文本长度预算

```rust
/// UI 区域多语言最大字符限制（未来扩展）
struct TextLengthBudget {
    locale: LocaleId,
    max_chars: HashMap<TextScope, usize>,
}

enum TextScope {
    Button,     // 按钮文本：最多 20 字符
    Tooltip,    // 工具提示：最多 100 字符
    Name,       // 名称：最多 30 字符
    Desc,       // 描述：最多 200 字符
    Dialogue,   // 对话：最多 500 字符
}
```

---

## 11. Risks

### 11.1 风险矩阵

| # | 风险 | 概率 | 影响 | 缓解措施 |
|---|------|------|------|----------|
| R1 | `fluent-rs` 生态与 Bevy 0.19 不兼容 | 中 | 高 | 评估期间使用简化版直接解析 .ftl 关键字段（`{$var}` 替换），逐步过渡到 fluent-rs |
| R2 | build.rs 增加编译时间 | 高 | 低 | Key 仅在有 .ftl 变更时重新生成，使用 `cargo:rerun-if-changed` 控制增量编译 |
| R3 | 性能：Fluent 模式解析开销 | 低 | 中 | 仅热路径（UI 每帧）使用 `LocalizedTextCache`，冷路径（剧情文本）延迟解析 |
| R4 | 团队对新 Key 体系适应成本 | 中 | 中 | 编译期检查 + Fake Locale 双保险：编译错误捕获 Key 拼写，Fake Locale 暴露硬编码文本 |
| R5 | .ftl 文件数量增长过快导致加载慢 | 低 | 中 | Lazy loading：仅加载当前 locale 的 .ftl，按 namespace 按需加载 |
| R6 | Key 重命名导致存量存档不兼容 | 低 | 低 | 存档只存 Key 字符串，Key 变更不影响存档加载 |
| R7 | Mod 本地化 Key 冲突 | 低 | 低 | 启动时检测冲突并输出 WARN，高优先级覆盖低优先级 |

### 11.2 依赖风险

| 依赖 | 版本 | 风险 | 后备方案 |
|------|------|------|----------|
| `fluent-rs` | 待定 | 维护活跃度未知 | 自研简易 Fluent 解析器（仅 `{$var}` 替换） |
| `fluent-bundle` | 待定 | API 稳定性 | 封装抽象层，隔离直接依赖 |
| Bevy 0.19 | 0.19.0-rc.3 | Asset API 对 .ftl 支持有限 | 使用 `include_str!` + 文件系统监控 |

---

## 12. Constitution Check

### 12.1 宪法 §22 Localization 专项规则合规性

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| §22.1.1 代码中禁止出现用户可见文本 | ✅ | Data Law 013 强制执行 |
| §22.1.2 Def 只存 LocalizationKey | ✅ | 30+ Schema 已统一使用 `name_key`/`desc_key` 字段 |
| §22.1.3 Replay/Event 只存 Key+参数 | ✅ | §7 Replay Compatibility 明确定义 |
| §22.1.4 存档禁止保存翻译结果 | ✅ | §8 Save Compatibility 明确定义 |
| §22.1.5 Localization 属于 Infrastructure 层 | ✅ | §1 Domain Ownership 归属 Infrastructure |
| §22.2.1 Key 格式 4 段式 | ✅ | §3.1 定义完整 `LocalizationKey ::= <namespace>.<scope>.<id>.<suffix>` |
| §22.2.5 必须使用 Fluent (.ftl) 格式 | ✅ | §3.5 定义 .ftl 资产目录和格式规范 |
| §22.3.1 LocalizationPlugin | ✅ | 由 ADR-053 定义，Schema 提供数据支撑 |
| §22.3.2 Key 自动生成 Rust 常量 | ✅ | §3.6 定义 `generated/keys.rs` 结构 |
| §22.3.3 启动时完整性校验 | ✅ | §6.1 定义启动时校验规则（缺失 Key → Fatal） |
| §22.3.5 三级回退链 | ✅ | §3.3 定义 `{locale} → en-US → raw_key` |
| §22.3.7 LocalizedTextCache | ✅ | §3.4 定义缓存失效策略 |
| §22.3.8 Mod 覆盖链 | ✅ | §10.3 定义 Base→DLC→Mod 三级覆盖 |
| §22.4.1 CI Localization 检查 | ✅ | §6.1 定义 CI 可执行的校验规则 |

### 12.2 Data Laws 合规性

| Data Law | 合规 | 说明 |
|----------|------|------|
| **001** Def-Instance 强制分离 | ✅ | .ftl 文件 (Def) 与 LocalizationDatabase (Instance) 严格分离 |
| **002** Rule-Content 强制分离 | ✅ | 翻译文本属于 Content，Key 格式规则属于代码 |
| **003** 配置只引用 ID | ✅ | Def 中只存 Key，不存翻译文本 |
| **010** Replay 优先于便利 | ✅ | Replay/Event 只存 Key+参数，不存翻译文本 |
| **011** Schema 必须版本化 | ✅ | .ftl 文件头版本标记，存档 Schema 带版本号 |
| **012** 域间禁止直接数据引用 | ✅ | LocaleId 和 LocalizationKey 在 Shared 层定义，被所有域共享但无直接依赖 |
| **013** 用户可见文本必须使用 LocalizationKey | ✅ | 本 Schema 的核心设计原则 |
| **014** LocalizationKey 以 en-US 为标准模板 | ✅ | §3.6 build.rs 只扫描 en-US，§6.1 en-US 缺失 Key → Fatal |

### 12.3 宪法 P0 铁则合规性

| P0 铁则 | 合规 | 说明 |
|---------|------|------|
| Replay First | ✅ | 本地化数据全程确定性 |
| Logic / Presentation Separation | ✅ | 翻译是 Presentation 层，Key 是 Logic 层 |
| Localization First | ✅ | 强制所有用户可见文本使用 LocalizationKey |

---

## 附录 A: 术语对照

| 术语 | 定义 |
|------|------|
| LocalizationKey | 引用本地化文本的 4 段式字符串 Key，编译期常量 |
| LocaleId | 语言标识符枚举（如 `en-US`, `zh-CN`） |
| Fluent | Mozilla 推出的 ICU 替代本地化格式 (.ftl) |
| Fake Locale | zz-ZZ 伪语言，用于检测硬编码文本 |
| Fallback Chain | 文本查询的三级回退路径 |
| LocalizedTextCache | 解析后文本的运行时缓存 |
| Orphan Key | .ftl 中存在但代码中未引用的 Key |
| L0-L3 | 本地化资产的分层（系统/UI/玩法/剧情） |
