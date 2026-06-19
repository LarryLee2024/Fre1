---
id: localization-implementation-plan
title: 国际化（i18n）基础设施实施计划
status: completed
owner: architect
created: 2026-06-19
based-on: docs/ai_ignore_this_dir/9国际化.md + 宪法 v5.0 + 项目现状审计
---

# 国际化（i18n）基础设施实施计划

## 术语说明

在本文档范围内，**国际化（i18n）**和**本地化（Localization）**互换使用，均指「使用户可见文本与代码分离、支持多语言」的完整系统。

---

## 1. 现状审计

### 1.1 已完成的前置设计

| 方面 | 状态 | 详情 |
|------|------|------|
| 数据架构 Key 规范 | ✅ 已有 | `docs/04-data/README.md` §3.2 定义了 `localization key` 格式：`<namespace>.<ID>.<suffix>` |
| Schema 中的 LocalizationKey | ✅ 已使用 | 30+ Schema 文件已统一使用 `name_key: LocalizationKey`、`desc_key: LocalizationKey` 等字段 |
| ID 策略 | ✅ 已定义 | `docs/04-data/foundation/id_strategy.md` 定义了无语义 ID 体系（`abl_000042` 而非 `ability.fireball`） |
| 数据四层架构 | ✅ 已定义 | Def/Spec/Instance/Persistence 四层为 LocalizationKey 的归属提供了清晰的定位 |

### 1.2 缺失项（本计划覆盖范围）

| 方面 | 缺失详情 |
|------|----------|
| **宪法规则** | 无任何条款禁止硬编码用户可见文本、要求使用 LocalizationKey |
| **架构 ADR** | 无 localization 作为基础设施层的架构决策记录 |
| **LocalizationKey 类型定义** | Schema 引用了 `LocalizationKey`，但该类型从未被正式定义 |
| **基础设施 Schema** | `infrastructure/` 下没有 `localization_schema.md` |
| **Data Laws** | 12 条 Data Laws 中无 localization 相关规则 |
| **领域文档** | Narrative/Quest/Ability/Effect 等领域文档未声明文本必须使用 Key |
| **技术设计** | 无 Fluent 集成方案、Key 代码生成设计、缓存策略等 |
| **实现代码** | `src/` 中无任何国际化相关代码 |
| **资产目录** | 无 `assets/localization/` |

---

## 2. 设计原则

### 2.1 核心定位

```
Localization 不属于 UI 层，也不属于 Capabilities（游戏机制层）
          ↓
Localization 属于 Infrastructure 层 (L2)
          ↓
Localization 是全局基础设施，所有用户可见文本的唯一下游
```

### 2.2 继承自参考文档的核心原则

| # | 原则 | 说明 |
|---|------|------|
| P1 | **代码中禁止出现用户可见文本** | 代码只出现 Key，不出现中文/英文/日文 |
| P2 | **国际化视为 Content** | Localization 属于 Content 领域，不是 UI 领域 |
| P3 | **Key 必须稳定** | 使用 ID 而非业务名称，`ability.abl_000042.name` 而非 `ability.fireball.name` |
| P4 | **Def 只存 Key** | 所有 Definition 中的文本字段使用 name_key/desc_key，不存直接文本 |
| P5 | **Fluent 优先** | 使用 Fluent (.ftl) 格式，利用其变量插值、复数、性别支持 |
| P6 | **描述模板化** | 数值描述通过 Fluent 变量实现，不硬编码数字 |
| P7 | **剧情与系统分离** | 系统文本和剧情文本分目录管理 |
| P8 | **Localization 作为一种可执行资源** | 影响编译、CI、运行时、UI、存档、Mod 的系统级组件 |
| P9 | **Localization Key 自动生成 Rust 常量** | 编译期检查，防止运行时 Key 拼写错误 |
| P10 | **Fake Locale 自动检测硬编码文本** | zz-ZZ 伪语言暴露未国际化文本 |
| P11 | **Replay/Event 只存 Key+参数** | 不存最终翻译文本 |
| P12 | **存档禁止保存翻译结果** | 存档只存 ID/Key，语言切换时实时翻译 |
| P13 | **设计时就支持热加载** | 修改 .ftl 文件无需重启 |
| P14 | **三级回退链** | `{locale}` → `en-US` → `raw_key` |
| P15 | **文本长度预算** | 各 UI 区域定义多语言最大字符限制 |
| P16 | **Mod 覆盖链** | Base → DLC → Mod，允许 Mod 覆盖 Key |
| P17 | **Localization 视为代码** | 与 Content/Config/Code 地位平等 |

---

## 3. 架构定位

### 3.1 在 DDD 三层 + 横切四层中的位置

```
L2: Infrastructure（技术实现层）
  └── localization/
      ├── LocalizationPlugin       # Bevy Plugin
      ├── LocalizationDatabase     # 全局文本存储 (ECS Resource)
      ├── LocalizationLoader       # .ftl 文件加载器
      ├── LocalizedTextCache       # 运行时缓存
      ├── LocalizationValidator    # 启动时校验
      └── LocalizationAudit        # 运行时审计

横切2: Content（内容桥接层）
  └── 负责 localization/ 资产注册到 AssetServer

横切3: Tools（开发工具层）
  └── localization-report CLI     # 覆盖率报告
  └── build.rs Key 生成器         # .ftl → Rust 常量
```

### 3.2 Plugin 注册顺序

```
Phase 7 (Infrastructure):
  .add_plugins(infra::registry::RegistryPlugin)
  .add_plugins(infra::pipeline::PipelinePlugin)
  .add_plugins(infra::replay::ReplayPlugin)
  .add_plugins(infra::save::SavePlugin)
  .add_plugins(infra::input::InputPlugin)
  .add_plugins(infra::localization::LocalizationPlugin)  // ← 新增
```

LocalizationPlugin 必须注册在 UI Plugin 之前、Content Plugin 之后，确保文本数据在 UI 渲染前就绪。

### 3.3 数据流

```
.ftl 文件 (assets/localization/{locale}/*.ftl)
    │
    ▼
LocalizationLoader ──→ LocalizationDatabase (ECS Resource)
    │                       │
    │                       ├── LocalizedTextCache ──→ UI Components
    │                       │
    │                       ├── LocalizationAudit ──→ 覆盖率报告
    │                       │
    │                       └── Fallback Chain ──→ {locale → en-US → raw_key}
    │
    ▼
LocalizationValidator (启动时)
    ├── 缺失 Key 检查
    ├── 重复 Key 检查
    ├── 未引用 Key 检查 (orphan keys)
    ├── 参数不匹配检查
    └── 文本长度预算检查
```

---

## 4. 资产目录设计

```
assets/localization/
├── en-US/
│   ├── core.ftl         # L0: 系统核心文本（确认/取消/保存/加载/是/否）
│   ├── ui.ftl           # L1: UI 界面文本（按钮/菜单/提示）
│   ├── gameplay.ftl     # L2: 玩法文本（战斗日志/状态/提示）
│   ├── ability.ftl      # L2: 技能/能力名称与描述
│   ├── buff.ftl         # L2: Buff/Debuff 名称与描述
│   ├── item.ftl         # L2: 物品名称与描述
│   ├── quest.ftl        # L2: 任务名称与描述
│   ├── tutorial.ftl     # L3: 教程文本
│   └── story/           # L3: 剧情对话（分章节）
│       ├── chapter01.ftl
│       ├── chapter02.ftl
│       └── ...
│
├── zh-CN/
│   └── ... (同上结构)
│
├── ja-JP/
│   └── ... (同上结构)
│
└── zz-ZZ/               # Fake Locale — 用于检测硬编码文本
    └── core.ftl
    └── ...
```

### 4.1 层次说明

| 层级 | 内容 | 稳定性 | Key 示例 |
|------|------|--------|---------|
| L0 Core | 系统级文本（是/否/确认/取消） | 几乎不变 | `core.yes`, `core.no` |
| L1 UI | 界面文本（菜单、按钮、标签） | 稳定 | `ui.confirm`, `ui.cancel` |
| L2 Gameplay | 玩法文本（技能、物品、战斗） | 随内容变 | `ability.abl_000001.name` |
| L3 Story | 剧情文本（对话、叙事） | 高频变化 | `story.ch01.dlg_001` |

---

## 5. LocalizationKey 语法规范

### 5.1 正式定义

```
LocalizationKey ::= <namespace> "." <scope> "." <id> "." <suffix>

namespace  ::= "core" | "ui" | "ability" | "buff" | "item" | "effect"
             | "quest" | "story" | "tutorial" | "error" | "battle"
             | "faction" | "spell" | "party" | "camp_rest" | "economy"
             | "crafting" | "summon" | "progression" | "reaction" | "terrain"
scope     ::= [a-z0-9_]+
id        ::= [a-z0-9_]+  (优先使用无语义 ID: abl_000042)
suffix    ::= "name" | "desc" | "flavor" | "tooltip" | "text" | "title"
```

### 5.2 示例

| Key | 含义 |
|-----|------|
| `core.yes` | 系统：确认按钮 |
| `core.no` | 系统：取消按钮 |
| `ui.battle.end_turn` | UI：结束回合 |
| `ability.abl_000042.name` | 技能：火球术的名称 |
| `ability.abl_000042.desc` | 技能：火球术的描述 |
| `buff.buf_000015.name` | Buff：中毒的名称 |
| `item.itm_000007.desc` | 物品：治疗药水的描述 |
| `quest.qst_000001.name` | 任务：主线第一章的名称 |
| `story.ch01.dlg_001.text` | 剧情：第一章第一句台词 |
| `error.character.not_found` | 错误：角色不存在 |

### 5.3 — 禁止模式

| 反模式 | 原因 |
|--------|------|
| `ability.fireball.name` | 技能改名时 Key 失效 |
| `fireball_name` | 无命名空间，冲突风险 |
| `名称.key` | Key 本身不应包含自然语言 |

---

## 6. Fluent 文件约定

### 6.1 基本格式

```ftl
### 文件：ability.ftl
### 层级：L2 Gameplay

-ability-abl-000042-name = Fireball
    .desc = Deals {$damage} fire damage to all targets in radius

-ability-abl-000043-name = Heal
    .desc = Restores {$value} HP to a single ally
```

```ftl
### 文件：zh-CN/ability.ftl

-ability-abl-000042-name = 火球术
    .desc = 对半径内所有目标造成 {$damage} 点火系伤害
```

### 6.2 变量命名约定

| 变量名 | 来源 | 说明 |
|--------|------|------|
| `{$value}` | EffectDef.value | 通用数值 |
| `{$damage}` | DamageEffectDef.damage | 伤害值 |
| `{$heal}` | HealEffectDef.value | 治疗值 |
| `{$turns}` | EffectDef.duration | 持续回合 |
| `{$target}` | Target.name | 目标名称 |
| `{$source}` | Source.name | 来源名称 |

### 6.3 复数支持

```ftl
-damage-dealt =
    { $count ->
        [one] Dealt {$count} damage
       *[other] Dealt {$count} damage
    }
```

Fluent 内置 CLDR 复数规则，禁止手写复数逻辑。

---

## 7. Key 代码生成（build.rs）

### 7.1 设计目标

从 `.ftl` 文件提取所有 Key 名称，生成 Rust 常量模块，提供编译期检查：

```rust
// 自动生成: src/infra/localization/generated/keys.rs

pub mod ability {
    pub mod abl_000042 {
        pub const NAME: &str = "ability.abl_000042.name";
        pub const DESC: &str = "ability.abl_000042.desc";
    }
}

pub mod core {
    pub const YES: &str = "core.yes";
    pub const NO: &str = "core.no";
}
```

使用方式：
```rust
// 编译期安全
let text = db.resolve(loc::ability::abl_000042::NAME);
// 错误拼写 → 编译错误
let text = db.resolve(loc::ability::abl_000043::NAME);  // 不存在 → 编译错误
```

### 7.2 扫描策略

- `build.rs` 扫描 `assets/localization/en-US/*.ftl`（以 en-US 为标准模板）
- 提取所有 `-` 开头的 message ID
- 生成 `keys.rs` 到 `src/infra/localization/generated/`

---

## 8. Fake Locale（zz-ZZ）设计

### 8.1 原理

Fake Locale 将所有文本替换为 ASCII 扩展字符，使硬编码文本在整个游戏中以原始 ASCII 形式暴露：

```ftl
### zz-ZZ/core.ftl
-core-yes = [Ýéś]
-core-no = [Ñó]
-ability-abl-000042-name = [Fírébáll]
```

### 8.2 检测目标

| 场景 | 表现 | 问题 |
|------|------|------|
| 正确使用 LocKey | `[Fírébáll]` 显示 | ✅ 正常 |
| 硬编码文本 | 普通 ASCII 文本直接显示 | 🚨 发现未国际化文本 |
| 代码中直接写入字符串 | 不受 Fake Locale 影响 | 🚨 紧急修复 |

### 8.3 启用方式

通过 feature flag 或命令行参数启用：
```
cargo run --features fake-locale
```

---

## 9. 实现阶段划分

### 阶段 0：宪法更新 — 预计 1 天 ✅ 已完成

| 任务 | 输出 | 状态 |
|------|------|------|
| P0 顶层铁则新增第7条 | `ai-constitution-complete.md` §1.5 | ✅ 完成 |
| 红线禁止新增第18条 | `ai-constitution-complete.md` §21 | ✅ 完成 |
| AI 反模式黑名单新增第27条 | `ai-constitution-complete.md` §20.1 | ✅ 完成 |
| 新增第二十二编：Localization 专项规则 | `ai-constitution-complete.md` §22 | ✅ 完成 |
| 同步更新 `.trae/rules/` 相关文件 | 架构规则、AI开发宪法、编码规则 | ✅ 完成 |

### 阶段 1：架构设计 — 预计 2 天 ✅ 已完成

| 任务 | 输出 | 状态 |
|------|------|------|
| 新增 ADR-053：Localization Infrastructure Architecture | `docs/01-architecture/40-cross-cutting/ADR-053-localization-architecture.md` | ✅ 完成 |
| 更新架构总纲 L2 Infra 表 | `docs/01-architecture/README.md` §3.4 | ✅ 完成 |
| 更新 Plugin 注册顺序 | `docs/01-architecture/README.md` §6.1 | ✅ 完成 |
| 更新附录 A 依赖表 | `docs/01-architecture/README.md` §附录A | ✅ 完成 |

### 阶段 2：数据架构 — 预计 2 天 ✅ 已完成

| 任务 | 输出 | 状态 |
|------|------|------|
| 新增 `localization_schema.md` | `docs/04-data/infrastructure/localization_schema.md` | ✅ 完成 |
| 扩展 README §3.2 Key 格式 | `docs/04-data/README.md` §3.2 | ✅ 完成 |
| 新增 Data Law 013/014 | `docs/04-data/README.md` §5 | ✅ 完成 |
| 更新 Schema 评审 Checklist | `docs/04-data/README.md` §4.3 | ✅ 完成 |
| 更新附录 B 文件状态 | `docs/04-data/README.md` §附录B | ✅ 完成 |

### 阶段 3：领域文档调整 — 预计 1 天 ✅ 已完成

| 任务 | 输出 | 状态 |
|------|------|------|
| Domain README 新增 localization 约束说明 | `docs/02-domain/README.md` | ✅ 完成 |
| 更新 narrative_domain.md | 增加对话文本必须使用 LocalizationKey 条款 | ✅ 完成 |
| 更新 quest_domain.md | 增加任务文本必须使用 LocalizationKey 条款 | ✅ 完成 |
| 更新 ability_domain.md | 增加技能文本必须使用 LocalizationKey 条款 | ✅ 完成 |
| 更新 effect_domain.md | 增加效果文本必须使用 LocalizationKey 条款 | ✅ 完成 |
| 更新其他涉及用户可见文本的领域文档 | 按需 | ✅ 完成 |

### 阶段 4：技术设计 — 预计 1 天 ✅ 已完成

| 任务 | 输出 | 状态 |
|------|------|------|
| 新增 localization 技术设计文档 | `docs/03-technical/localization-design.md` | ✅ 完成 |

### 阶段 5：代码实现 — 预计 5-7 天 ✅ 已完成

| 子阶段 | 任务 | 状态 |
|--------|------|------|
| 5a | Localization 基础设施核心（Plugin/Database/Loader/Cache/Components） | ✅ 完成 |
| 5b | build.rs Key 代码生成 | ✅ 完成（含编译错误修复） |
| 5c | 资产目录创建 + 示例 .ftl 文件 | ✅ 完成（en-US/zh-CN/zz-ZZ 骨架） |
| 5d | 启动校验 + Fake Locale | ✅ 完成 |
| 5e | Localization 集成测试 | ✅ 完成（1537 tests pass） |
| 5f | 覆盖率报告工具 | ✅ 完成 |
| 5g | 合并入主 Plugin 注册链 | ✅ 完成 |

---

## 10. 文件变更清单（完整）

### 新增文件

```
docs/
├── 01-architecture/
│   └── 40-cross-cutting/
│       └── ADR-053-localization-architecture.md   # 新 ADR
├── 03-technical/
│   └── localization-design.md                     # 技术设计
├── 04-data/
│   └── infrastructure/
│       └── localization_schema.md                 # 数据 Schema
└── 09-planning/
    └── localization-implementation-plan.md        # 本文件

src/
└── infra/
    └── localization/
        ├── mod.rs                                 # 模块导出
        ├── plugin.rs                              # LocalizationPlugin
        ├── database.rs                            # LocalizationDatabase
        ├── loader.rs                              # LocalizationLoader
        ├── cache.rs                               # LocalizedTextCache
        ├── components.rs                          # LocalizedText Component
        ├── validator.rs                           # LocalizationValidator
        ├── audit.rs                               # LocalizationAudit
        ├── generated/
        │   └── keys.rs (自动生成)                  # 编译期生成
        └── test.rs                                # 单元测试

assets/localization/
├── en-US/
│   ├── core.ftl
│   ├── ui.ftl
│   ├── gameplay.ftl
│   ├── ability.ftl
│   ├── buff.ftl
│   ├── item.ftl
│   ├── quest.ftl
│   └── tutorial.ftl
├── zh-CN/
│   └── ... (同上结构)
└── zz-ZZ/
    └── core.ftl
```

### 修改文件

```
docs/
├── 00-governance/
│   └── ai-constitution-complete.md                # 多条新增
├── 01-architecture/
│   └── README.md                                  # 3处更新
├── 02-domain/
│   ├── README.md                                  # 新增说明
│   ├── capabilities/ability_domain.md             # LocalizationKey 约束
│   ├── capabilities/effect_domain.md              # LocalizationKey 约束
│   ├── domains/narrative_domain.md                # LocalizationKey 约束
│   └── domains/quest_domain.md                    # LocalizationKey 约束
├── 04-data/
│   ├── README.md                                  # 3处更新
│   └── foundation/id_strategy.md                  # 补充 LocalizationKey ID 说明
├── 09-planning/
│   └── README.md                                  # 新增本文件索引
└── 09-planning/README.md                          # 新增本文件索引

.trae/rules/
├── 架构规则.md                                    # 同步宪法更新
├── AI开发宪法.md                                   # 同步宪法更新
└── 编码规则.md                                    # 新增 i18n 编码规范

src/
└── app/plugin.rs                                  # 注册 LocalizationPlugin

build.rs                                           # 新增 Key 代码生成
```

---

## 11. 依赖关系与并行策略

```
宪法更新 (0) ──→ 架构 ADR (1a) ──→ 架构 README (1b)
                      │
                      ├──→ 数据 Schema (2a) ──→ 数据 README (2b, 2c)
                      │
                      ├──→ 领域文档 (3) ──→ 全部并行
                      │
                      └──→ 技术设计 (4)
                              │
                              ↓
                         代码实现 (5a → 5b → 5c → 5d → 5e → 5f → 5g)
```

阶段 0 必须最先完成（宪法为所有后续提供依据）。
阶段 1a（ADR）完成后，阶段 2（数据）、3（领域）、4（技术设计）可并行。
阶段 5（代码）依赖阶段 0-4 全部完成。

---

## 12. 风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| Fluent Rust 库的 Bevy 兼容性 | 可能需要自研解析器 | 评估 `fluent-rs` vs `i18n-embed`，预留直接解析 .ftl 方案 |
| build.rs 增加编译时间 | 开发者体验下降 | Key 仅在有 .ftl 变更时重新生成，增量编译优化 |
| Def 中 name→name_key 迁移破坏现有代码 | Schema 与代码不一致 | 所有 Schema 已使用 name_key，代码层尚未实现，迁移成本低 |
| 团队对新约定适应成本 | 初期可能漏用 | 编译期检查（build.rs 生成）+ Fake Locale 双保险 |
| 性能：每帧大量 LocalizedText 查询 | UI 卡顿 | LocalizedTextCache 兜底，仅在语言切换时失效 |

---

## 13. 验收标准

### 文档验收 ✅ 全部完成

- [x] 宪法 v5.1 新增 Localization 专项编，P0 铁则第7条生效
- [x] ADR-053 被 @architect 审核通过
- [x] `localization_schema.md` 通过 @data-architect 审核
- [x] 所有涉及用户可见文本的领域文档已更新
- [x] 技术设计文档包含 Fluent / Key生成 / Fake Locale / CI 方案

### 代码验收 ✅ 全部通过

- [x] `LocalizationPlugin` 在 Plugin 链中正确注册
- [x] `.ftl` 文件可正确加载到 `LocalizationDatabase`
- [x] `LocalizedText { key, params }` Component 可在 UI 中使用
- [x] `build.rs` 成功生成 `keys.rs`，误拼 Key 导致编译错误
- [x] `zz-ZZ` Fake Locale 可检测硬编码文本
- [x] 语言切换时 UI 立即刷新
- [x] 启动时缺失 Key 校验阻止启动
- [x] 三级回退链正常工作
- [x] `cargo nextest run` 全部通过（1537 tests pass）
- [x] `cargo clippy -- -D warnings` 全部通过
