# 本地化领域

Version: 1.1

本地化领域管理多语言文本的存储、查找、渲染和切换，确保 Content 配置与展示文本彻底分离。

核心原则：
- Content 配置永远只存 Key，不存文本
- 语言切换自动刷新所有 UI，零遗漏
- 自封装 Fluent 适配层，禁止依赖第三方 Bevy 插件

## 宪法合规矩子

| 条款 | 级别 | 落地规则 |
|------|------|----------|
| 17.2.2 国际化预留 | 🟥 | 禁止在代码、配置中硬编码玩家可见文本，所有文本统一通过本地化资源管理 |
| 12.1.4 资源管理 | 🟩 | FTL 文件通过统一 Asset Pipeline 管理 |
| 22.4 数据驱动绝对优先 | 🟥 | 文本内容通过 FTL 文件配置，禁止硬编码 |

---

# 术语定义

## 本地化（Localization）

多语言文本管理系统，负责将 Content 中的 Key 映射为当前语言的展示文本。

不是翻译工具。不是内容配置。

关键属性：
- 管理 FTL 文件的加载、解析、缓存
- 通过 FluentBundle 实现 Key → 文本的查找
- 支持语言切换时批量刷新 UI
- 支持 MOD 翻译的合并

---

## Fluent 格式（Fluent .ftl）

Mozilla 设计的现代国际化文件格式，支持变量插值、复数、性别、条件文本。

不是 JSON。不是 RON。不是 CSV。

关键属性：
- 文件扩展名为 `.ftl`
- 支持变量：`{ $damage }`
- 支持术语复用（Term）：`-brand-name = 火焰`
- 支持属性（Attributes）：`.voice = vo_ch1_001`
- 天然支持 CLDR 复数规则

---

## 文本 Key（Text Key）

层级化的文本标识符，格式为 `domain.subdomain.identifier`（如 `battle.turn.start`）。

不是 text_001 编号。不是硬编码字符串。不是语义化名称（如 `skill.fireball.name`）。

关键属性：
- 点分层级命名，自带上下文语义
- 一旦创建不可修改（视为数据库主键 UUID）
- Content 配置中只存储 Key，不存储文本
- 按 domain 分文件存储（ui/battle/quest/skill/system）

> **优化来源**: `docs/architecture/i18n_design.md` § 3.3 — Key 永久 ID 策略

永久 ID 策略（Permanent ID Strategy）：

🟥 **禁止使用语义化名称作为 Key**（如 `skill.fireball.name`）。一旦策划将"火球术"改名为"烈焰爆"或重做技能 ID，Key 维护将陷入批量替换灾难。

工业级方案：使用「命名空间 + 永久唯一 ID」：
- ✅ 推荐：`skill.s_1001.name`（s_1001 是火球术的永久唯一 ID，即使改名也不变）
- 🟥 禁止：`skill.fireball.name`（"fireball" 与实际技能名耦合）

规则：
- Key 一旦创建，即使文案/技能名改了，Key 也绝对不允许修改
- 永久 ID 在 Content Pipeline 的 RON 配置中生成，随资产生命周期不变
- FTL 文件中的注释必须标注对应 ID 的业务含义（如 `# s_1001: 火球术（v2更名后仍用旧ID）`）

---

## LocalizedText 组件

Bevy 组件，持有 Key + 参数，语言切换时自动刷新文本。

不是 Text 组件。不是 String。

关键属性：
- 包含 key 字段（Text Key）和 args 字段（FluentArgs）
- 挂载到 UI Entity 上，系统自动解析并更新 Text 组件
- 变更时触发 Bevy Change Detection，自动重绘 UI
- 禁止在 UI 代码中直接使用 `Text::new("硬编码")`

---

## 语言回退链（Fallback Chain）

查找文本时的优先级顺序：请求 locale → 项目默认 → 英语（兜底）。

不是直接失败。不是错误。

关键属性：
- 查找顺序：MOD_zh-CN → 本体_zh-CN → 本体_en-US
- 通过组合多个 FluentBundle 实现
- 缺失 Key 时 Debug 模式显示 `[MISSING: key.name]` 红色占位
- 缺失 Key 时 Release 模式返回空字符串

---

## MOD 翻译

每个 MOD 自带 `localization/` 目录，运行时合并到主 Bundle。

不是项目翻译。不是覆盖。

关键属性：
- MOD 翻译文件在 `mod/` 目录下独立维护
- 加载时按 ID 深度合并到主 FluentBundle
- MOD Key 必须加前缀（如 `mod.super_pack.fireball.name`）
- 合并策略为 Merge，不为 Overwrite
- MOD 大概率只翻译部分语言，必须实现多层语言回退链

MOD 语言回退链（Fallback Chain）：

> **优化来源**: `docs/architecture/i18n_design.md` § 8.5 — MOD 覆盖链的实现细节

查找顺序：`MOD_zh-CN → 本体_zh-CN → 本体_en-US（兜底）`

实现方式：组合多个 FluentBundle，查询时逐级查找。每个语言对应一个 Fluent Bundle 列表，按优先级排列。避免日文/韩文玩家运行时显示空白。

---

## 语音集成（Voice Key Integration）

对话系统中语音资源与文本 Key 的关联机制，确保语音和文本通过同一标识符体系关联但各自独立加载。

不是硬编码文件名。不是音频资源直接引用。

> **优化来源**: `docs/architecture/i18n_design.md` § 7.3 — 语音集成预留

关键属性：
- `voice_key` 与 `text_key` 并列存储在对话配置中，各自独立
- `voice_key` 使用标识符体系（如 `voice.knight.intro.001`），不硬编码音频文件名
- 语音资源的加载由 Audio 领域负责，本地化领域只管理 Key 的定义和关联
- 语音 Key 遵循与文本 Key 相同的命名空间规则

---

## Fluent 术语（Term）

跨文件复用的全局术语定义，以 `-` 前缀声明（如 `-brand-short = xxx`）。

不是普通消息。不是变量。

关键属性：
- 全局唯一，所有 FTL 文件可引用
- 修改一处，所有引用处自动同步
- 用于统一术语（如火焰伤害、元素类型等）
- 通过 `{ -term-name }` 语法引用

---

## CJK 字体与排版适配（Font Fallback & Layout Adaptation）

中/日/韩文字体及多语言排版策略，确保切换语言后文本正确渲染、不溢出。

不是拉丁字体。不是图标字体。不是固定宽度布局。

> **优化来源**: `docs/architecture/i18n_design.md` § 6A — Cosmic Text 集成、Flexbox 弹性布局、RTL 语言支持

关键属性：
- 英文字体不包含 CJK 字形，切换中文后会显示豆腐块（□□□）
- 需在 TextFont 中配置多字体回退链（Font Fallback Chain），利用 Bevy Cosmic Text 集成
- 字体文件按语言分组：`noto-sans-cjk/`（中日韩）、`noto-sans-arabic/`（阿拉伯语）、`default.ttf`（西欧兜底）
- UI 布局必须使用 **Flexbox 弹性布局**（Bevy UI 默认支持），让按钮/容器随文本自动撑开
- 🟥 禁止使用固定像素宽度的 UI 容器承载本地化文本
- 容器宽度应设置 `min_width: 0, max_width: Some(parent_width)`，允许弹性伸缩

多语言排版陷阱（Text Overflow）：

🟥 **绝对禁止**：UI 按钮/文本框写死宽度。德语/俄语文本长度通常比中文/英文长 **30%~50%**，切换语言后文字会溢出。

RTL 语言支持：
- 阿拉伯语、希伯来语等 RTL（从右到左）语言需要额外 UI 布局适配
- 对话框/技能描述面板需检测文本方向，自动切换布局方向（`Direction::RightToLeft`）
- Bevy UI 的 `Node` 组件支持 `flex_direction` 动态设置
- RTL 判定规则：检测首字符 Unicode 范围（`\u{0600}-\u{06FF}` 为阿拉伯语）

---

# 领域边界

## 本领域负责

- FTL 文件的加载、解析、缓存
- FluentBundle 的构建和 Key 查找
- LocalizedText 组件的定义和刷新系统
- 语言切换事件的处理和 UI 批量更新
- 语言回退链的实现
- MOD 翻译的合并逻辑
- Fluent 术语的全局复用
- 缺失 Key 的处理策略

## 本领域不负责

- Content 配置中的 Key 存储（由 Content Pipeline 领域负责）
- UI 布局和组件创建（由 UI 领域负责）
- 字体文件的加载和渲染（由 Asset 领域负责）
- 音频资源的本地化（由 Audio 领域负责）
- 存档中的文本存储（由 Save 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| LanguageChangedEvent | Message | UI 领域 |
| LocalizedText 查询 | 函数调用 | 所有领域 |
| FTL 文件加载请求 | Command | Asset 领域 |
| MOD 翻译合并 | Resource 访问 | MOD 领域 |

---

# 生命周期

本领域无状态机，为纯事件驱动。

状态列表：

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Initializing | FTL 文件加载中 | Ready |
| Ready | 本地化系统就绪，可响应查询 | Ready, Switching |
| Switching | 语言切换中，重建 Bundle | Ready |

状态转换图：

Initializing → Ready → Switching → Ready

转换条件：

| 从 | 到 | 条件 |
|----|-----|------|
| Initializing | Ready | 所有 FTL 文件加载完成 |
| Ready | Switching | LanguageChangedEvent 触发 |
| Switching | Ready | 所有 LocalizedText 刷新完成 |

---

# 不变量

## 不变量1：Content 配置永远只存 Key [宪法 17.2.2 🟥]

任意时刻：

🟥 Content 配置（RON 文件）中的文本字段必须存储 Text Key，禁止存储实际文本。违反此条款等于违反宪法最高优先级条款。

违反表现：

技能描述字段直接写 `"对目标造成100点伤害"` 而非 `"skill.fireball.desc"`。

---

## 不变量2：UI 永远使用 LocalizedText 组件 [宪法 17.2.2 🟥]

任意时刻：

🟥 所有展示给用户的文本必须通过 LocalizedText 组件渲染，禁止使用 `Text::new("硬编码")` 或 `Text::from("硬编码")`。

违反表现：

UI 代码中出现 `Text::new("开始战斗")`、`Text::from(format!("伤害: {}", damage))`。

---

## 不变量3：语言回退链永不以缺失 Key 导致崩溃

任意时刻：

任何语言下查找 Key 失败时，必须按回退链逐步查找，最终兜底到英语或空字符串，禁止 panic 或 unwrap。

违反表现：

切换到未完全翻译的语言时游戏崩溃，或未处理的 None 导致 panic。

---

## 不变量4：MOD 翻译合并不覆盖原版 Key

任意时刻：

MOD 翻译文件合并到主 FluentBundle 时，只添加新 Key，禁止覆盖已存在的原版 Key。

违反表现：

MOD 的 zh-CN.ftl 覆盖了原版的 `skill.fireball.name`，导致原版文本丢失。

---

## 不变量5：Fluent (.ftl) 是唯一技术决策

> **优化来源**: `docs/architecture/i18n_design.md` § 1 — 技术决策：Fluent (.ftl)

任意时刻：

国际化系统必须且只允许使用 Mozilla Fluent（`.ftl` 文件格式 + `fluent-rs` crate）。

违反表现：

使用 JSON、CSV、RON 等非 Fluent 格式存储本地化文本；或引入 `bevy_fluent` 等第三方 Bevy 插件。

理由：
- Fluent 天然支持变量插值、复数、性别、条件文本，JSON/RON 不具备这些能力
- Bevy 版本迭代极快（0.15~0.18 生态变动巨大），社区第三方插件极大概率年久失修
- 自封装代码量仅几百行（AssetLoader + Resource + System），完全掌控 ECS 生命周期

---

## 不变量6：性能静默期规则

> **优化来源**: `docs/architecture/i18n_design.md` § 10A — 性能优化

任意时刻：

FluentBundle 必须在启动时/加载时预解析完成并缓存。运行时仅在 `LanguageChangedEvent` 触发时重新格式化所有 `LocalizedText`，其余时间保持静默，禁止每帧解析 Fluent AST 或格式化字符串。

违反表现：

在 Update 阶段的 System 中每帧调用 `FluentBundle.get_message()` 或 `format()`；每帧遍历所有 `LocalizedText` 组件重新解析。

理由：
- Fluent 的 AST 解析和 Message 格式化有 CPU 开销，每帧执行会导致帧率暴跌
- 正确策略：启动时预解析 → 运行时仅语言切换时重格式化 → 平时静默

---

## 不变量7：UI 布局必须弹性适配

> **优化来源**: `docs/architecture/i18n_design.md` § 6A — Text Overflow

任意时刻：

所有承载本地化文本的 UI 容器必须使用 Flexbox 弹性布局，禁止使用固定像素宽度。德语/俄语文本长度比中文/英文长 30%~50%，固定宽度会导致文本溢出或截断。

违反表现：

UI 按钮/文本框写死宽度（如 `width: Val::Px(200)`），切换语言后文字溢出或截断。

---

## 不变量8：Key 使用永久唯一 ID

> **优化来源**: `docs/architecture/i18n_design.md` § 3.3 — Key 永久 ID 策略

任意时刻：

技能、物品、Buff 等 Content 实体的本地化 Key 必须使用永久唯一 ID（如 `skill.s_1001.name`），禁止使用语义化名称（如 `skill.fireball.name`）。Key 一旦创建，即使文案/技能名改了，也绝对不允许修改。

违反表现：

使用 `skill.fireball.name` 作为 Key，策划改名为"烈焰爆"后 Key 变成 `skill.flame_burst.name`，导致所有引用方批量失效。

---

# 业务规则

## 规则1：自封装 Fluent 适配层

> **优化来源**: `docs/architecture/i18n_design.md` § 1.1、§ 10.1 — 自封装 fluent-rs + intl-memoizer

允许：
- 基于 `fluent-rs`（`fluent` + `fluent-bundle`）直接调用
- 使用 `intl-memoizer` 实现复数/性别规则
- 自定义 AssetLoader 解析 .ftl 文件
- 自定义 LocalizationResource 存入 Bevy Res
- 代码量控制在几百行核心逻辑（AssetLoader + Resource + System）

禁止：
- 依赖 bevy_fluent 等第三方 Bevy 插件
- 使用已知停止维护的社区 crate

必须：
- 自己实现 FluentBundle 的构建和缓存
- 自己实现语言切换时的 Bundle 重建
- 代码量控制在几百行核心逻辑

---

## 规则2：FTL 文件按 locale + domain 拆分

> **优化来源**: `docs/architecture/i18n_design.md` § 2.2、§ 2.3 — 文件拆分规则

允许：
- 按 locale + domain 双维度拆分：`{locale}/ui.ftl`、`{locale}/battle.ftl` 等
- 每个文件 ≤ 500 条 Key（超过时按子域拆分，如 `skill_magic.ftl` + `skill_physical.ftl`）
- 按需加载（战斗场景只加载 battle.ftl）
- 文件编码统一为 UTF-8

禁止：
- 单个巨大 FTL 文件（如十万行）
- FTL 文件中包含非本地化内容（如配置数据）

必须：
- 文件命名与 domain 对齐
- 每个 FTL 文件顶部添加注释说明用途
- 变量名在文件内保持一致

---

## 规则3：Fluent 变量支持动态描述

允许：
- 使用 `{ $variable }` 语法插入动态值
- 使用 Fluent Attributes 绑定语音、时长等元数据
- 使用 `<em>` 等标记实现富文本效果

禁止：
- 在 FTL 文件中硬编码数值
- 使用 BBCode/HTML 标签作为富文本（使用 Fluent 原生标记）

必须：
- 变量名语义化（`$damage` 而非 `$v1`）
- 富文本标记在代码层解析生成 Bevy TextSection

---

## 规则4：MOD 翻译合并策略与回退链

> **优化来源**: `docs/architecture/i18n_design.md` § 8.5 — MOD 覆盖链（Fallback Chain）

允许：
- MOD 翻译文件在 `mod/` 目录下独立维护
- 按 ID 深度合并到主 FluentBundle
- MOD Key 加前缀（`mod.{mod_id}.`）避免冲突
- MOD 本地化遵循回退链：MOD locale → 本体 locale → 本体默认语言（兜底）

禁止：
- MOD 翻译覆盖原版 Key
- MOD 之间互相覆盖 Key
- 合并时忽略语言回退链

必须：
- 合并策略为 Merge（追加新 Key）
- 未翻译的 Key 自动回退到原版
- 合并后验证 Key 完整性

---

## 规则5：CI 集成 fluent-lint

允许：
- CI 自动校验多语言 Key 对齐
- 校验变量名一致性
- 校验语法错误

禁止：
- 缺失 Key 的翻译文件通过 CI
- 变量名拼写不一致的 FTL 文件通过 CI

必须：
- 开发模式下缺失 Key 显示 `[MISSING: key.name]` 红色占位
- CI 流程中运行 fluent-lint 检查

---

## 规则6：性能优化策略

> **优化来源**: `docs/architecture/i18n_design.md` § 10A — FluentBundle 预解析、文本解析缓存、异步加载

允许：
- 启动时/加载时将 .ftl 解析为 FluentBundle 并缓存
- 使用 intl-memoizer 缓存复数/性别计算结果
- 仅在 LanguageChangedEvent 触发时重新格式化所有 LocalizedText
- 仅在 FluentArgs 数值改变时（如伤害值变化）重新格式化对应组件
- 使用 `LocalizedTextCache` 按 Key + 参数哈希缓存结果（LRU 淘汰，上限 2048 条）
- 大体积 FTL 文件（>100KB）使用 Bevy AssetServer 异步加载
- 启动时仅加载当前语言的 FTL 文件，其他语言按需异步加载

禁止：
- 每帧解析 Fluent AST
- 每帧调用 resolve 查询字典
- 在 Update 阶段的 System 中频繁格式化字符串

必须：
- FluentBundle 缓存在 Res<Localization> 中
- 静默期不执行文本刷新逻辑

---

## 规则7：语音 Key 集成规范

> **优化来源**: `docs/architecture/i18n_design.md` § 7.1、§ 7.3 — 语音集成预留

允许：
- 对话配置中 `voice_key` 与 `text_key` 并列存储
- `voice_key` 使用标识符体系（如 `voice.knight.intro.001`），不硬编码音频文件名
- 语音资源的加载由 Audio 领域负责，本地化领域只管理 Key 的定义和关联

禁止：
- 硬编码音频文件路径（如 `"audio/voices/ch1_knight_001.ogg"`）
- `voice_key` 依赖特定语言的文件结构
- 本地化领域直接加载音频资源

必须：
- `voice_key` 遵循与文本 Key 相同的命名空间规则
- 对话配置中 `voice_key` 为 `Option` 类型，允许无语音的纯文本对话

---

## 规则8：翻译协作与版本控制

> **优化来源**: `docs/architecture/i18n_design.md` § 11A — FTL 版本控制与翻译协作

允许：
- FTL 文件头部包含版本注释（如 `# version: 1.2.0`）
- 废弃的 Key 加 `# DEPRECATED: use xxx instead` 注释保留 3 个版本后移除
- 新增 Key 同步更新所有语言的 FTL 文件（缺失语言用英语占位）
- 使用 `fluent-lint` 工具在 CI 中自动校验多语言 Key 对齐、变量名一致性
- FTL 文件内包含上下文注释（翻译场景备注、变量说明）
- 变量插值（`{ $damage }`）在注释中说明含义和取值范围

禁止：
- 随意废弃或重命名已有 Key
- 缺失 Key 的翻译文件通过 CI
- 变量名拼写不一致的 FTL 文件通过 CI
- FTL 文件中不添加上下文注释（翻译人员因缺乏语境导致误翻）

必须：
- 配套 Fluent 专用编辑器（如 Pontoon、Fluent Editor），降低翻译人员使用成本
- CI 流程中运行 fluent-lint 检查多语言 Key 对齐
- 新增 Key 时所有语言文件同步更新

---

# 管线

## 文本解析管线

```
FTL文件加载 → FluentBundle构建 → intl-memoizer缓存 → Runtime Key查找
```

### Step1：FTL 文件加载

输入：assets/localization/{lang}/*.ftl 文件路径
处理：自定义 AssetLoader 读取 FTL 文件内容
输出：FluentResource 列表
禁止：在运行时动态修改 FTL 文件内容

### Step2：FluentBundle 构建

输入：FluentResource 列表 + 语言标识
处理：创建 FluentBundle，注册所有资源和术语
输出：完整的 FluentBundle 实例
禁止：跳过术语注册直接构建 Bundle

### Step3：intl-memoizer 缓存

输入：FluentBundle + 复数/性别规则
处理：使用 intl-memoizer 缓存 CLDR 规则计算结果
输出：带缓存的 Bundle 实例
禁止：每次查询重新计算复数规则

### Step4：Runtime Key 查找

输入：Text Key + FluentArgs
处理：通过 FluentBundle.get_message() + format() 查找并格式化
输出：格式化后的文本字符串
禁止：跳过回退链直接返回 None

---

## 语言切换管线

```
LanguageChangedEvent → 重建Bundle → 遍历LocalizedText → 重新解析所有Key
```

### Step1：LanguageChangedEvent 触发

输入：用户切换语言操作
处理：发送 LanguageChangedEvent 事件
输出：事件广播到所有监听系统
禁止：在战斗结算阶段处理语言切换

### Step2：重建 Bundle

输入：新语言标识
处理：加载新语言的 FTL 文件，构建新的 FluentBundle
输出：新的 Bundle 实例替换旧实例
禁止：同时保留多个语言的 Bundle（内存浪费）

### Step3：遍历 LocalizedText

输入：所有挂载 LocalizedText 组件的 Entity
处理：遍历 Query<(LocalizedText, &mut Text)>
输出：需要刷新的 Entity 列表
禁止：跳过任何 LocalizedText Entity

### Step4：重新解析所有 Key

输入：Entity 的 LocalizedText + 新 Bundle
处理：调用 localization.get() 获取新文本，更新 Text 组件
输出：所有 UI 文本更新为新语言
禁止：缓存旧的格式化结果而不更新

---

# 数据结构

## Localization（本地化资源）

职责：管理所有语言的 FluentBundle，提供 Key 查找接口

结构：
- current_lang：当前语言标识
- bundles：语言到 FluentBundle 的映射
- default_lang：项目默认语言

要求：
- 通过 Res<Localization> 访问
- get() 方法实现回退链查找
- 语言切换时重建 bundles

---

## LocalizedText（本地化文本组件）

职责：标记需要本地化的 UI 文本，持有 Key 和参数

结构：
- key：Text Key 字符串
- args：可选的 FluentArgs 参数

要求：
- 挂载到需要本地化的 Text Entity 上
- 系统自动解析并更新 Text 组件
- 禁止手动调用 localization.get() 更新

---

## LanguageChangedEvent（语言切换事件）

职责：通知所有系统语言已切换

结构：
- new_lang：新语言标识
- old_lang：旧语言标识

要求：
- 语言切换时必须发送此事件
- 所有监听此事件的系统必须刷新相关数据
- 事件在帧结束时统一处理

---

## FTL 文件结构

职责：存储特定语言的文本 Key 和翻译

结构：
- 按 domain 拆分：ui.ftl、battle.ftl、quest.ftl、skill.ftl、system.ftl
- 每个文件 ≤ 500 条 Key
- 支持 Term 定义（`-term-name = value`）
- 支持 Attributes（`.voice = xxx`）

要求：
- 文件编码为 UTF-8
- Key 命名遵循 `domain.subdomain.identifier` 格式
- 变量使用 `{ $name }` 语法

---

# 禁止事项

禁止：Content 配置中硬编码文本 [宪法 17.2.2 🟥]

原因：🟥 破坏 Content 与文本的分离，导致新增语言时必须修改配置文件。违反宪法最高优先级条款。

违反后果：每次新增语言或 MOD 翻译都需要修改 Content 配置，无法实现零侵入扩展。

---

禁止：UI 中硬编码字符串 [宪法 17.2.2 🟥]

原因：🟥 绕过 LocalizedText 组件，语言切换时该文本不会更新。违反宪法最高优先级条款。

违反后果：切换语言后部分 UI 仍显示旧语言文本，用户体验崩溃。

---

禁止：text_001 编号 Key

原因：编号 Key 无语义，维护时无法从 Key 推断内容，批量替换困难

违反后果：几年后维护时无法识别 `text_001` 对应什么内容，成为技术债

---

禁止：单个巨大 FTL 文件

原因：多人协作时频繁冲突，按需加载困难，内存占用高

违反后果：Git 合并冲突频发，战斗场景加载全部文本浪费内存

---

禁止：依赖第三方 Bevy 本地化插件

原因：Bevy 版本迭代快，第三方插件极大概率年久失修，无法兼容最新版本

违反后果：Bevy 升级后本地化系统瘫痪，被迫重写或卡在旧版本

---

禁止：每帧解析 Fluent AST

原因：Fluent 的 AST 解析和 Message 格式化有 CPU 开销，每帧执行会导致帧率暴跌

违反后果：战斗场景帧率下降，UI 文本刷新卡顿

---

禁止：MOD 翻译覆盖原版 Key

原因：破坏原版文本的完整性，导致其他玩家的翻译丢失

违反后果：MOD 安装后原版技能名称、描述等被覆盖，无法恢复

---

禁止：FTL 文件中硬编码数值

原因：数值应通过 Fluent 变量注入，硬编码导致平衡调整时必须修改翻译文件

违反后果：修改伤害数值时必须同步修改所有语言的 FTL 文件，增加维护成本

---

禁止：使用语义化名称作为 Key（如 `skill.fireball.name`）

原因：一旦策划改名或重做 ID，所有引用方批量失效，维护陷入替换灾难

违反后果：技能改名后需全局搜索替换所有 FTL 文件和 Content 配置，极易遗漏

---

禁止：语音资源硬编码文件名

原因：硬编码文件名与文本 Key 体系脱节，无法统一管理和维护

违反后果：语音文件重命名或移动路径后，所有引用方需要手动更新文件路径

---

禁止：UI 容器固定像素宽度承载本地化文本

原因：德语/俄语文本比中文/英文长 30%~50%，固定宽度会导致文本溢出或截断

违反后果：切换语言后按钮文字溢出、文本框截断、布局崩坏

---

# AI 修改规则

## 如果新增文本 Key

允许：
- 在对应 domain 的 FTL 文件中添加新 Key
- 遵循 `domain.subdomain.identifier` 命名格式
- Content 配置中使用新 Key

禁止：
- 使用编号 Key（如 text_123）
- 在非对应 domain 的 FTL 文件中添加
- Content 配置中直接写文本

优先检查：
- Key 命名是否符合层级格式
- 对应语言的 FTL 文件是否都添加了此 Key
- 变量名是否与代码中一致

---

## 如果修改语言切换逻辑

允许：
- 修改 LanguageChangedEvent 的触发条件
- 调整 Bundle 重建的时机
- 优化刷新系统的性能

禁止：
- 移除回退链逻辑
- 在战斗结算阶段处理语言切换
- 跳过任何 LocalizedText Entity

优先检查：
- 回退链是否完整（请求 → 默认 → 英语）
- 所有 LocalizedText 是否都被刷新
- 事件是否在帧结束时统一处理

---

## 如果新增 MOD 翻译支持

允许：
- 添加 MOD 翻译文件的加载逻辑
- 实现合并到主 Bundle 的逻辑
- MOD Key 加前缀避免冲突

禁止：
- MOD 翻译覆盖原版 Key
- 合并时忽略语言回退链
- 不验证 MOD 翻译的完整性

优先检查：
- MOD Key 是否加了前缀
- 合并策略是否为 Merge
- 未翻译的 Key 是否正确回退

---

## 如果修改 FTL 文件结构

允许：
- 调整文件拆分粒度
- 添加新的 domain 文件
- 优化文件大小（≤500 条/文件）

禁止：
- 合并多个 domain 到单个文件
- 在 FTL 文件中包含非本地化内容
- 修改已有 Key 的命名

优先检查：
- 文件大小是否 ≤ 500 条
- Key 命名是否与 domain 对齐
- CI 的 fluent-lint 是否通过

---

## 如果测试失败

排查顺序：
1. 检查 FTL 文件是否正确加载（FluentBundle 构建是否成功）
2. 检查 Key 是否存在于当前语言的 FTL 文件中
3. 检查回退链是否正确执行（请求 → 默认 → 英语）
4. 检查 LocalizedText 组件是否挂载到正确的 Entity
5. 检查变量名是否与代码中一致（FluentArgs 是否匹配）
