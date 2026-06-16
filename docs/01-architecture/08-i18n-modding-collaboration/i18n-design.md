---
id: 01-architecture.i18n-design
title: I18n Design
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
  - design
---

# Internationalization System Architecture — 国际化系统架构

Version: 2.1
Status: Proposed

来源：`docs/其他/32.国际化.md`、`docs/其他/33..md`、`docs/其他/52.md`、`docs/01-architecture/09-infrastructure-migration/infrastructure-design.md`、`docs/01-architecture/03-data-config-asset/content-pipeline.md`、`docs/01-architecture/08-i18n-modding-collaboration/modding-design.md`

### 宪法条款映射

| 本文档规则 | 宪法条款 | 强制等级 |
|-----------|---------|---------|
| §4 Content Key 驱动 | 🟥 17.2.2 国际化：禁止硬编码玩家可见文本 | 必须遵循 |
| §6.3 反模式 | 🟥 17.2.2 所有文本通过本地化资源管理 | 必须遵循 |
| §6 LocalizedText 组件 | 🟥 10.0.2 状态单向流动 | 必须遵循 |
| §11 禁止事项 | 🟥 1.1.4 逻辑与表现分离 | 必须遵循 |
| §10.5 依赖关系 | 🟥 1.3.2 依赖方向铁则 | 必须遵循 |

交叉引用：`infrastructure-design.md` § 3.4 localization、`content-pipeline.md` § Rule/Content 分离、`modding-design.md` § MOD 内容目录结构

> **优化来源**: `docs/其他/33..md` — 自封装 fluent-rs 方案、Key 永久 ID 命名、字体回退机制、性能优化
> **优化来源**: `docs/其他/52.md` — 性能缓存策略、RTL 支持、翻译协作工具链、FTL 版本控制

---

## 1. 技术决策：Fluent (.ftl)

### 1.1 决策

采用 **Mozilla Fluent**（`fluent-rs` + `intl-memoizer`，**自封装** Bevy 适配层）作为国际化系统。

🟥 **绝对禁止**：依赖第三方 `bevy_fluent` 插件。Bevy 版本迭代极快（0.15~0.18 生态变动巨大），社区第三方插件极大概率年久失修。自封装代码量仅几百行（AssetLoader + Resource + System），完全掌控 ECS 生命周期，可控性远高于第三方 crate。

> **优化来源**: `docs/其他/33...md` — bevy_fluent 断更风险，推荐基于 fluent-rs 自研轻量适配层

### 1.2 为什么不选 JSON

| 缺陷 | 说明 |
|------|------|
| 无原生变量插值 | 需要自行实现 `replace("{attacker}")` 替换逻辑 |
| 无复数支持 | 英文 `1 item` vs `2 items` 需要手动判断 |
| 无性别/条件文本 | 不同语言的语法差异无法优雅表达 |
| 无参数顺序控制 | 不同语言的语序不同，JSON 无法调整变量顺序 |

### 1.3 为什么不选 RON

RON 不是为本地化设计的序列化格式。与 JSON 存在相同的无复数、无条件文本、无变量插值问题。

### 1.4 Fluent 的优势

Fluent 是 Mozilla 设计的现代国际化系统，天然支持：

```text
变量       → { $damage }
复数       → { $count -> [one] ... [other] ... }
性别       → { $gender -> [male] ... [female] ... }
条件文本   → { $has_buff -> [yes] ... [no] ... }
参数插值   → { $attacker } 对 { $target } 造成了 { $damage } 点伤害
```

### 1.5 Fluent 示例

中文：

```ftl
battle-damage =
    { $attacker } 对 { $target } 造成了 { $damage } 点伤害
```

英文：

```ftl
battle-damage =
    { $attacker } dealt { $damage } damage to { $target }
```

运行时传入 `attacker=Knight`、`target=Goblin`、`damage=50`，各语言自动输出正确文本。代码完全不需要修改。

---

## 2. 目录结构

### 2.1 资源目录

```text
assets/
└── localization/
    ├── en-US/
    │   ├── ui.ftl
    │   ├── battle.ftl
    │   ├── quest.ftl
    │   ├── skill.ftl
    │   └── system.ftl
    ├── zh-CN/
    │   ├── ui.ftl
    │   ├── battle.ftl
    │   ├── quest.ftl
    │   ├── skill.ftl
    │   └── system.ftl
    ├── ja-JP/
    │   ├── ui.ftl
    │   ├── battle.ftl
    │   ├── quest.ftl
    │   ├── skill.ftl
    │   └── system.ftl
    └── ko-KR/
        ├── ui.ftl
        ├── battle.ftl
        ├── quest.ftl
        ├── skill.ftl
        └── system.ftl
```

### 2.2 文件拆分规则

按**业务域**拆分，禁止单一巨大文件。

| 文件 | 包含内容 |
|------|---------|
| `ui.ftl` | 界面文本（按钮、标题、菜单、提示信息） |
| `battle.ftl` | 战斗文本（伤害、治疗、回合开始/结束、状态消息） |
| `quest.ftl` | 任务文本（任务描述、目标、完成提示） |
| `skill.ftl` | 技能文本（技能名称、描述、效果说明） |
| `system.ftl` | 系统文本（设置、错误消息、存档提示） |

### 2.3 反模式

🟥 **绝对禁止**：单个巨大文件（如 `zh.json` 10 万行）。

拆分原则：每个 `.ftl` 文件不超过 500 条翻译条目。超过时按子域拆分（如 `skill.ftl` → `skill_magic.ftl` + `skill_physical.ftl`）。

---

## 3. 文本 Key 设计

### 3.1 命名规则

采用**层级命名**：`domain.subdomain.identifier`

### 3.2 Key 示例

```ftl
# battle.ftl
battle.turn.start = 回合开始
battle.turn.end = 回合结束
battle.damage = { $attacker } 对 { $target } 造成 { $damage } 点伤害
battle.heal = { $target } 恢复了 { $amount } 点生命值

# skill.ftl
skill.fireball.name = 火球术
skill.fireball.desc = 对目标造成 { $damage } 点火焰伤害
skill.heal.name = 治愈术

# quest.ftl
quest.chapter1.start = 第一章开始
quest.chapter1.objective = 击败所有敌人

# ui.ftl
ui.battle.start = 开始战斗
ui.menu.settings = 设置
ui.menu.save = 保存游戏

# system.ftl
system.error.skill_not_found = 技能未找到: { $skill_id }
system.save.success = 游戏保存成功
```

### 3.3 Key 命名的"永久 ID"策略

🟥 **禁止使用语义化名称作为 Key**（如 `skill.fireball.name`）。一旦策划将"火球术"改名为"烈焰爆"或重做技能 ID，Key 维护将陷入批量替换灾难。

**工业级方案**：使用「命名空间 + 永久唯一 ID」：

```ftl
# ✅ 推荐：永久 ID（s_1001 是火球术的永久唯一 ID，即使改名也不变）
skill.s_1001.name = 烈焰爆
skill.s_1001.desc = 对目标造成 { $damage } 点火焰伤害

# 🟥 禁止：语义化名称（"fireball" 与实际技能名耦合）
skill.fireball.name = 火球术
skill.fireball.desc = 对目标造成 { $damage } 点火焰伤害
```

**规则**：
- 🟥 Key 一旦创建，即使文案/技能名改了，Key 也绝对不允许修改（把 Key 当成数据库的主键 UUID）
- 🟩 永久 ID 在 Content Pipeline 的 RON 配置中生成，随资产生命周期不变
- 🟩 FTL 文件中的注释必须标注对应 ID 的业务含义（如 `# s_1001: 火球术（v2更名后仍用旧ID）`）

> **优化来源**: `docs/其他/33..md` — Key 设计的"语义化陷阱"，使用永久唯一 ID 避免批量替换灾难

### 3.4 反模式

🟥 **绝对禁止**：无意义的编号 Key（`text_001`、`msg_002`）。

原因：编号 Key 在数月后无人知道对应什么内容，维护成本极高。

🟥 **绝对禁止**：使用语义化名称作为 Key（如 `skill.fireball.name`）。

原因：当策划改名或重做 ID 时，Key 维护将陷入批量替换灾难。必须使用永久唯一 ID（如 `skill.s_1001.name`），参见 § 3.3。

---

## 4. Content 层：Key 驱动架构

### 4.1 核心架构决策

**Content 数据只存储本地化 KEY，从不存储文本本身。**

### 4.2 反模式（当前）

```ron
// content/skills/fireball.ron
(
    name: "火球术",
    description: "对目标造成火焰伤害",
)
```

### 4.3 目标模式

> **宪法条款**: 🟥 §3.3 禁止使用语义化名称作为 Key，必须使用永久唯一 ID

```ron
// content/skills/fireball.ron
(
    // ✅ 正确：使用永久唯一 ID（s_1001 是火球术的永久 ID，即使改名也不变）
    name_key: "skill.s_1001.name",
    desc_key: "skill.s_1001.desc",
)
```

🟥 **Content 数据中的 Key 必须使用永久唯一 ID，禁止使用语义化名称（如 `skill.fireball.name`）。**

### 4.4 为什么这样设计

| 收益 | 说明 |
|------|------|
| 新增语言无需改内容 | 添加 `ja-JP/skill.ftl` 即可，Content 数据不变 |
| MOD 自带翻译 | MOD 内容 key 指向 MOD 自己的 `.ftl` 文件 |
| DLC 独立翻译 | DLC 内容 key 指向 DLC 自己的 `.ftl` 文件 |
| 平衡调整不需改描述 | 数值变化时描述中的变量自动更新 |

### 4.5 与 content-pipeline 的关系

参见 `docs/01-architecture/03-data-config-asset/content-pipeline.md` § Rule/Content 分离。

```
Content 层职责：
  content/skills/fireball.ron  → SkillDef（存储 name_key 而非 name）
  ↓ impl From<SkillDef> for SkillData
  SkillData（存储 name_key 用于运行时解析）
  ↓ LocalizationService.resolve(key, locale)
  显示文本
```

---

## 5. 技能描述：动态内容

### 5.1 问题

技能描述中包含动态数值（伤害值、冷却时间等），平衡调整后描述需要同步更新。

### 5.2 解决方案

使用 Fluent 变量将动态值注入描述：

```ftl
# skill.ftl
skill-fireball-desc =
    对目标造成 { $damage } 点火焰伤害

skill-fireball-desc-ja =
    対象に { $damage } ポイントの炎ダメージを与える
```

### 5.3 运行时注入

```rust
// 技能描述解析
let desc = localization.format("skill.fireball.desc", &fluent_args! {
    "damage" => skill_data.damage_value,
});
```

### 5.4 平衡修改场景

```text
修改前：damage = 150 → "对目标造成 150 点火焰伤害"
修改后：damage = 180 → "对目标造成 180 点火焰伤害"
```

描述自动更新，无需翻译人员参与。

### 5.5 Fluent 术语（Term）全局复用

> **优化来源**: `docs/其他/33..md` — Fluent Term 功能，全局术语统一

Fluent 原生支持「术语（Term）」，专门解决全局术语统一问题。对几千条文本的 SRPG 项目，术语统一是文案质量的核心保障。

```ftl
# 全局术语定义（所有 ftl 文件共享）
-fire-damage = 火焰伤害
-fire-element = 火属性
-heal-amount = 恢复量

# 引用术语
skill-fireball-desc =
    对目标造成 { $damage } 点{ -fire-damage }，附加 2 层{ -fire-element }灼烧

battle-heal-msg =
    { $target } 恢复了 { $amount } 点{ -heal-amount }
```

**规则**：
- 🟩 全局术语只改一处，所有引用的地方自动同步，不会出现「一半写火焰伤害、一半写灼烧伤害」的不一致
- 🟩 游戏核心属性名（伤害类型、元素属性、状态名称）必须定义为 Term
- 🟥 禁止在 FTL 文件中硬写重复的术语文本

---

## 6. LocalizedText 组件

### 6.1 组件定义

```rust
/// 标记一个 UI 元素需要本地化显示
/// 存储本地化 Key，由系统自动解析为当前语言文本
#[derive(Component)]
pub struct LocalizedText {
    /// 本地化 Key，如 "battle.turn.start"
    pub key: String,
    /// 可选的变量参数（用于插值文本）
    pub args: Option<FluentArgs>,
}
```

### 6.2 系统行为

> **优化来源**: `docs/其他/33..md` — Change Detection 防御性编程，避免每帧重新格式化

```rust
/// 语言切换时重新解析所有 LocalizedText 组件
/// 仅在 LanguageChangedEvent 或组件刚添加时执行，平时静默
fn resolve_localized_texts(
    locale: Res<CurrentLocale>,
    localization: Res<LocalizationService>,
    mut query: Query<(&LocalizedText, &mut Text)>,
    mut lang_changed: EventReader<LanguageChangedEvent>,
) {
    // 性能优化：仅在语言切换或组件刚添加时才执行
    let lang_event_received = !lang_changed.is_empty().is_empty();
    if !lang_event_received && !query.is_changed() {
        return;
    }
    // 消费事件
    lang_changed.clear();

    for (localized, mut text) in query.iter_mut() {
        let new_string = localization.resolve(&localized.key, &locale.0, localized.args.as_ref());
        // 防御性赋值：仅在文本真正改变时更新，避免无效触发 Changed
        if text.as_str() != new_string {
            *text = Text::new(new_string); // 触发 Bevy 的 Change Detection，自动重绘 UI
        }
    }
}
```

### 6.3 反模式

🟥 **绝对禁止**：`Text::new("开始战斗")` 硬编码。

原因：
- 语言切换时无法自动刷新
- 无法支持多语言
- 违反 Logic/Presentation 分离原则

---

## 6A. 字体回退与 UI 布局适配

> **优化来源**: `docs/其他/33..md` — 德语/俄语文本长度溢出、CJK 字体缺失、Cosmic Text 集成

### 6A.1 多语言排版陷阱

🟥 **绝对禁止**：UI 按钮/文本框写死宽度。德语/俄语文本长度通常比中文/英文长 **30%~50%**，切换语言后文字会溢出。

**核心规则**：
- 🟩 UI 布局必须使用 **Flexbox 弹性布局**（Bevy UI 默认支持），让按钮/容器随文本自动撑开
- 🟥 禁止使用固定像素宽度的 UI 容器承载本地化文本
- 🟩 容器宽度应设置 `min_width: 0, max_width: Some(parent_width)`，允许弹性伸缩

### 6A.2 字体回退（Font Fallback）

英文字体不包含汉字/日文假名，切换语言后会出现"豆腐块（□□□）"。

**实现方案**：利用 Bevy 的 Cosmic Text 集成（如 `bevy_cosmic_edit`），在 `TextFont` 中配置多字体回退链：

```rust
// 字体回退链配置（Resource）
#[derive(Resource)]
pub struct FontFallbackChain {
    /// 按语言优先级排列的字体列表
    chains: HashMap<Locale, Vec<Handle<Font>>>,
}

impl FontFallbackChain {
    /// 为指定语言获取字体列表（含回退）
    pub fn get_fonts(&self, locale: &Locale) -> Vec<Handle<Font>> {
        self.chains
            .get(locale)
            .or_else(|| self.chains.get(&Locale::en_US))
            .cloned()
            .unwrap_or_default()
    }
}
```

**字体文件组织**：

```text
assets/fonts/
├── noto-sans-cjk/      # 中日韩统一字体
├── noto-sans-arabic/   # 阿拉伯语
├── noto-sans-deva/     # 印地语
└── default.ttf         # 西欧语言兜底
```

### 6A.3 RTL 语言支持

阿拉伯语、希伯来语等 RTL（从右到左）语言，Fluent 仅解决文本层面，**UI 布局需要额外适配**：

- 🟩 对话框/技能描述面板需检测文本方向，自动切换布局方向（`Direction::RightToLeft`）
- 🟩 Bevy UI 的 `Node` 组件支持 `flex_direction` 动态设置
- 🟩 RTL 判定规则：检测首字符 Unicode 范围（`\u{0600}-\u{06FF}` 为阿拉伯语）

> **优化来源**: `docs/其他/33..md` — Flexbox 弹性布局、Cosmic Text 多语言混排
> **优化来源**: `docs/其他/52.md` — RTL 语言 UI 布局适配规则

---

## 7. 对话系统

### 7.1 对话配置

```ron
// content/dialogues/chapter1/knight.ron
(
    dialogue_id: "chapter1_knight_intro",
    lines: [
        (
            speaker: "knight",
            text_key: "dialogue.chapter1.knight.001",
            voice_key: "voice.knight.intro.001",  // 可选，预留语音集成
        ),
        (
            speaker: "knight",
            text_key: "dialogue.chapter1.knight.002",
            voice_key: "voice.knight.intro.002",
        ),
    ],
)
```

### 7.2 对话文本

```ftl
# quest.ftl
dialogue-chapter1-knight-001 =
    我们必须守住这座城。

dialogue-chapter1-knight-002 =
    敌人的数量比我们预想的要多。
```

### 7.3 语音集成预留

`voice_key` 字段用于未来语音配音。`text_key` 和 `voice_key` 通过同一标识符体系关联，但各自独立加载。

---

## 8. MOD 本地化

### 8.1 MOD 目录结构

```text
mods/
└── super_skill_pack/
    ├── manifest.ron
    ├── skills/
    │   └── thunder_strike.ron
    ├── localization/
    │   ├── en-US/
    │   │   └── mod.ftl
    │   ├── zh-CN/
    │   │   └── mod.ftl
    │   └── ja-JP/
    │       └── mod.ftl
    └── assets/
```

### 8.2 MOD Key 命名空间

MOD 内容的 Key 必须使用 MOD ID 作为前缀：

```ftl
# mods/super_skill_pack/localization/zh-CN/mod.ftl
mod.super_skill_pack.skill.thunder_strike.name = 雷霆一击
mod.super_skill_pack.skill.thunder_strike.desc = 对目标造成 { $damage } 点雷电伤害
```

### 8.3 MOD 内容中的 Key 引用

```ron
// mods/super_skill_pack/skills/thunder_strike.ron
(
    id: "thunder_strike",
    name_key: "mod.super_skill_pack.skill.thunder_strike.name",
    desc_key: "mod.super_skill_pack.skill.thunder_strike.desc",
    damage: 200,
)
```

### 8.4 主工程无感知

MOD 自带 `localization/` 目录，主工程代码不需要任何修改。MOD 加载器在扫描 MOD 内容时，同时加载 MOD 的本地化文件并合并到全局翻译表。

### 8.5 MOD 语言回退链

MOD 大概率只翻译部分语言（如仅中英文），必须实现多层语言回退，避免日文/韩文玩家运行时显示空白。

**查找顺序**：

```text
MOD_zh-CN → 本体_zh-CN → 本体_en-US（兜底）
```

**实现方式**：组合多个 FluentBundle，查询时逐级查找。每个语言对应一个 Fluent Bundle 列表，按优先级排列。

> **优化来源**: `docs/其他/33..md` — MOD 覆盖链（Fallback Chain）的实现细节

---

## 9. 语言切换

### 9.1 触发源

| 触发场景 | 说明 |
|---------|------|
| 用户设置 | 玩家在设置界面切换语言 |
| 平台检测 | 首次启动时检测系统语言 |
| 开发调试 | 调试面板切换语言 |

### 9.2 切换流程

```
用户切换语言
    ↓
1. 更新 CurrentLocale Resource
    ↓
2. 热加载目标语言的 FTL 文件（如果未加载）
    ↓
3. 遍历所有 LocalizedText 组件，重新解析文本
    ↓
4. 触发 LanguageChanged Event
    ↓
5. UI 系统响应事件刷新
```

### 9.3 Fallback 链

```text
请求的语言 (requested locale)
    ↓ 找不到 Key
默认语言 (zh-CN)
    ↓ 找不到 Key
英语 (en-US)
    ↓ 找不到 Key
Key 标识符本身（调试模式）/ 空字符串（发布模式）
```

### 9.4 缺失 Key 处理

| 构建模式 | 缺失 Key 行为 |
|---------|--------------|
| Debug | 显示 Key 标识符（如 `[MISSING: battle.start]`），便于发现遗漏 |
| Release | 显示空字符串，避免暴露内部 Key |

### 9.5 热重载

FTL 文件修改后，通过 `infrastructure/hot_reload/` 机制触发重新加载。热重载只影响 Definition（翻译表），不影响 Instance（已解析的文本缓存）。

---

## 10. 基础设施集成

### 10.1 模块位置

> **优化来源**: `docs/其他/33..md` — 自封装 fluent-rs + intl-memoizer，不依赖第三方 bevy_fluent

```text
src/infrastructure/localization/
├── mod.rs                  # 模块入口
├── plugin.rs               # LocalizationPlugin（自封装，非 bevy_fluent）
├── localization_error.rs   # LocalizationError 错误类型
├── locale.rs               # Locale 类型定义
├── service.rs              # LocalizationService 核心服务（封装 fluent-rs + intl-memoizer）
├── ftl_loader.rs           # 自定义 AssetLoader 解析 .ftl 文件
├── component.rs            # LocalizedText 组件
├── systems.rs              # 文本解析系统（仅在语言切换时执行）
├── cache.rs                # LocalizedTextCache 解析缓存
└── font_fallback.rs        # FontFallbackChain 字体回退链
```

### 10.2 核心接口

```rust
// infrastructure/localization/service.rs

/// 本地化核心服务
/// 负责 FTL 翻译表管理、Key 解析、变量插值
pub struct LocalizationService {
    /// 当前已加载的翻译表（locale → FTL bundle）
    bundles: HashMap<Locale, FluentBundle>,
    /// Fallback 链配置
    fallback_chain: Vec<Locale>,
}

impl LocalizationService {
    /// 解析本地化 Key，返回目标语言文本
    pub fn resolve(&self, key: &str, locale: &Locale, args: Option<&FluentArgs>) -> String;

    /// 加载指定 locale 的 FTL 文件
    pub fn load_bundle(&mut self, locale: &Locale, ftl_content: &str) -> Result<(), LocalizationError>;

    /// 切换当前语言
    pub fn set_locale(&mut self, locale: Locale);
}
```

### 10.3 Plugin 注册

```rust
// infrastructure/localization/plugin.rs

pub struct LocalizationPlugin;

impl Plugin for LocalizationPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(CurrentLocale::default())
            .add_systems(Startup, initialize_localization)
            .add_systems(Update, resolve_localized_texts);
    }
}
```

### 10.4 与 ContentPlugin 的集成

```rust
// content/content_plugin.rs
impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(LocalizationPlugin)  // 在内容加载前初始化本地化
            .add_plugins(SkillContentPlugin)
            .add_plugins(BuffContentPlugin)
            // ...
    }
}
```

### 10.5 依赖关系

```
infrastructure/localization/
    → infrastructure/assets/     （加载 FTL 文件）
    → shared/error/              （错误工具）
    ← content/                   （Content 层调用 resolve）
    ← ui/                        （UI 层使用 LocalizedText 组件）
    ← modding/                   （MOD 加载本地化文件）
```

---

## 10A. 性能优化

> **优化来源**: `docs/其他/33..md` — 启动时预解析 FluentBundle、仅在语言切换时重新格式化
> **优化来源**: `docs/其他/52.md` — 文本解析缓存、异步加载大 FTL 文件

### 10A.1 FluentBundle 预解析

🟥 **禁止在 Update 阶段的 System 里每帧解析 Fluent AST 和格式化字符串**。Fluent 的 AST 解析和 Message 格式化有 CPU 开销，每帧执行会导致帧率暴跌。

**正确策略**：

```
启动时/加载时：
  1. 将 .ftl 文件解析为 FluentBundle 并缓存在 Res<Localization> 中
  2. 此时已完成 AST 解析，后续查询仅需查字典 + 变量插值

运行时：
  1. 仅在 LanguageChangedEvent 触发时重新格式化所有 LocalizedText
  2. 仅在 FluentArgs 数值改变时（如伤害值从 150 变为 180）重新格式化对应组件
  3. 平时保持静默，不做任何解析
```

### 10A.2 文本解析缓存

高频场景（如战斗刷屏的伤害文本）反复调用 `resolve` 可能有性能损耗。

**缓存策略**：

```rust
#[derive(Resource)]
pub struct LocalizedTextCache {
    /// key: (ftl_key, locale, args_hash) → formatted_string
    cache: HashMap<(String, Locale, u64), String>,
}
```

- 🟩 按 Key + 参数哈希缓存结果，避免重复格式化
- 🟩 仅在 LanguageChangedEvent 时清空缓存
- 🟩 缓存上限 2048 条，LRU 淘汰策略

### 10A.3 异步加载 FTL 文件

🟥 **大体积 FTL 文件（>100KB）禁止同步加载**，会阻塞主线程。

- 🟩 使用 Bevy AssetServer 的异步加载机制
- 🟩 启动时仅加载当前语言的 FTL 文件
- 🟩 其他语言按需加载（语言切换时触发异步加载）
- 🟩 加载期间使用缓存的默认语言文本作为占位

---

## 11. 禁止事项

| 禁止 | 强制等级 | 说明 |
|------|---------|------|
| Content 配置中硬编码文本 | 🟥 绝对禁止 | 必须使用 `*_key` 字段存储本地化 Key |
| UI 中硬编码显示文本 | 🟥 绝对禁止 | 必须使用 `LocalizedText` 组件 |
| 单个巨大 FTL 文件 | 🟥 绝对禁止 | 按业务域拆分，每个文件不超过 500 条 |
| 使用无意义的 Key 编号 | 🟥 绝对禁止 | 必须使用层级命名 `domain.subdomain.identifier` |
| `Text::new("硬编码文本")` | 🟥 绝对禁止 | 使用 `LocalizedText` 替代 |
| Core 层直接调用 LocalizationService | 🟥 绝对禁止 | Core 层只存储 Key，解析在 Content/UI 层 |
| 运行时修改 FTL 翻译表 | 🟥 绝对禁止 | 翻译表是 Definition，不可变 |
| MOD 内容 Key 无前缀 | 🟥 绝对禁止 | MOD Key 必须使用 `mod.{mod_id}.` 前缀 |
| 使用 `bevy_fluent` 第三方插件 | 🟥 绝对禁止 | 基于 `fluent-rs` 自封装，避免版本断更风险 |
| 使用语义化名称作为 Key | 🟥 绝对禁止 | 使用永久唯一 ID（如 `skill.s_1001.name`） |

---

## 11A. FTL 版本控制与翻译协作

> **优化来源**: `docs/其他/52.md` — FTL 文件版本控制策略、翻译人员协作工具链

### 11A.1 FTL 文件版本管理

🟥 **禁止随意废弃或重命名已有 Key**。大型项目中翻译迭代易引发兼容问题。

**规则**：
- 🟩 FTL 文件头部必须包含版本注释（如 `# version: 1.2.0`）
- 🟩 废弃的 Key 必须加 `# DEPRECATED: use xxx instead` 注释保留 3 个版本后移除
- 🟩 新增 Key 必须同步更新所有语言的 FTL 文件（缺失语言用英语占位）
- 🟩 使用 `fluent-lint` 工具在 CI 中自动校验多语言 Key 对齐、变量名一致性

### 11A.2 翻译人员协作规范

- 🟩 FTL 文件内必须包含上下文注释（翻译场景备注、变量说明），避免翻译人员因缺乏语境导致误翻
- 🟩 变量插值（`{ $damage }`）必须在注释中说明含义和取值范围
- 🟩 建议配套 Fluent 专用编辑器（如 Pontoon、Fluent Editor），降低翻译人员使用成本

> **优化来源**: `docs/其他/52.md` — 翻译协作流程、FTL 注释规范、CI 校验集成

---

## 12. 与其他架构文档的关系

| 文档 | 关联点 |
|------|--------|
| `architecture.md` | 七层架构总纲；Localization 归属 Infrastructure 层（Layer 4） |
| `infrastructure-design.md` § 3.4 | Localization 模块在 Infrastructure 层的定位与接口定义 |
| `content-pipeline.md` | Content 数据使用 Key 而非文本，遵循 Rule/Content 分离 |
| `modding-design.md` | MOD 内容自带 localization/ 目录，与主工程本地化隔离 |
| `project-structure.md` | `assets/localization/` 在资产树中的位置 |
| `ui_domain_boundary_rules.md` | UI 层通过 `LocalizedText` 组件读取文本，不直接访问 LocalizationService |
