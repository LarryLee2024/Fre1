---
id: AI-REDLINE-CONSTITUTION
title: AI执行规范与红线禁止事项宪法
status: accepted
stability: stable
layer: governance
related:
  - ai-constitution-complete.md
tags:
  - ai
  - redline
  - anti-pattern
  - localization
---

> **原文来源**：`ai-constitution-complete.md` 第二十编（L1732-L1806）+ 第二十一编（L1809-L1839）+ 第二十二编（L1842-L1894）
> **锚定总宪法**：第二十编、第二十一编、第二十二编

## 第二十编 AI 专属执行规范
### 20.1 AI 反模式黑名单（生成前必须对照检查）
违反以下任意一条的代码必须立即重写：
1. 把 Entity 当面向对象实例，模拟 `player.attack(enemy)` 调用
2. 把 Resource 当全局变量仓库
3. 创建全局顶层的 `systems.rs/components.rs/events.rs` 巨文件
4. 滥用事件/Trigger 模拟普通函数调用
5. 业务逻辑直接操作 UI 组件、修改 UI 状态
6. 直接修改基础/派生属性的最终数值
7. 为单个实现创建无价值的 Trait
8. 为未明确的未来需求提前设计复杂架构
9. 手写 bool 脏标记检测组件生命周期变化
10. Release 版本每帧输出 INFO/DEBUG 级别日志
11. 输出技术流水账日志，而非领域事件日志
12. 业务代码直接手写 `info!` 输出核心业务事件
13. 使用全局 AppError、anyhow::Error 作为业务层统一错误类型
14. 核心业务领域代码中使用 unwrap/expect/panic
15. 混淆规则失败与程序错误，将正常规则不满足作为 Err 返回
16. 业务代码提交 todo!/unimplemented!() 占位
17. 新增核心业务系统未附带任何测试用例
18. 新增领域事件未纳入白名单文档直接使用
19. 核心领域逻辑直接依赖 Bevy 表现层类型，不做纯函数抽象
20. 预览/仿真等读路径带有副作用
21. 业务代码直接调用原始随机数 API，不使用统一 RNG 服务
22. 代码中硬编码数值平衡魔法数字
23. 跨模块直接修改其他 Feature 的内部状态
24. 未经授权创建新 Feature、修改公共 API
25. 在 Capabilities 层硬编码业务规则，破坏机制与业务的边界
26. Domain 之间直接 `use` 对方内部类型，绕过双轨通信（写操作走事件，读操作走 Query API）
27. 代码中硬编码用户可见文本字符串（中文/英文/日文等），未使用 LocalizationKey 引用本地化资源

### 20.2 AI 代码自检清单（文档参考，不输出到代码）

> **说明**：此清单仅作为 AI 生成代码时的内部参考，不要求在生成的代码中输出自检结果。
> 真正有效的合规检查依赖 CI 门禁（cargo clippy / dependency_checker / 架构扫描），而非 AI 自检。

AI 生成代码前应内部对照以下要点：

| 检查项 | 说明 |
|--------|------|
| 按业务拆分模块 | 无全局技术型巨文件（systems.rs / components.rs） |
| 配置与运行时分离 | Def（模板）与 Instance（运行时）不混写 |
| 逻辑与表现分离 | 业务规则不依赖渲染/音频/输入 |
| 组合优于继承 | 无子类派生式设计 |
| 不直接操作 UI | 业务逻辑不修改 UI 组件 |
| 属性走 Modifier 管线 | 不直接修改最终属性值 |
| 日志符合领域事件规范 | 不输出技术流水账 |
| 错误分领域定义 | 无全局 AppError 大枚举 |
| 核心业务层无 unwrap/panic | 仅测试/工具代码允许 |
| 双轴边界合规 | Capabilities 无业务规则，Domain 无重复机制 |
| Domain 间无直接依赖 | 写操作走事件，读操作走 Query API |
| 新增系统附带测试 | 测试跟领域走，含 invariant 层 |
| 读路径无副作用 | 预览/仿真不修改状态 |
| 使用统一 RNG 服务 | 不直接调用 rand::random() |
| 无硬编码魔法数字 | 数值配置归 content/ |

### 20.3 AI 权限边界
- 🟥 未经明确授权，禁止创建新的 Feature 模块与 Domain
- 🟥 未经明确授权，禁止修改各 Feature 的公共 API 定义
- 🟥 未经明确授权，禁止修改 Layers 层的核心机制实现

### 20.4 AI 最高优先级执行条款（10条）
超越所有其他条款的最高优先级：
1. 🟥 Feature First：按业务领域拆模块，不按技术类型拆全局目录
2. 🟥 Definition / Instance 强制分离：配置定义与运行时状态完全隔离
3. 🟥 Rule / Content 强制分离：代码只实现通用规则，配置只定义内容
4. 🟥 Logic / Presentation 强制分离：业务逻辑与表现层完全隔离
5. 🟥 四级通信机制：Hook=生命周期，Trigger=事件链，Observer=局部响应，Message=跨域广播
6. 🟥 属性管线统一：基础/派生属性修改必须走 Modifier 体系
7. 🟥 数据驱动绝对优先：成熟扩展点纯配置扩展，新机制允许修改逻辑
8. 🟩 Capabilities/Domains 双轴架构原则：Capabilities 管机制，Domains 管业务，边界不可突破
9. 🟥 测试与确定性优先：Battle Replay + 自动化测试，核心战斗必须可重现
10. 🟥 组合绝对优先：所有差异化通过组件、Trait、Modifier 组合实现
11. 🟥 Localization First：所有用户可见文本必须通过 LocalizationKey 引用，禁止任何用户可见文本硬编码在 Rust 代码中；Def 只存 name_key/desc_key 不存直接文本；存档/Replay/Event 只存 Key+参数，不存翻译结果

---

## 第二十一编 红线禁止事项总览
1. 🟥 禁止创建 `utils.rs`、`helpers.rs`、`common.rs` 垃圾桶文件
2. 🟥 禁止用 `bool` 标志位替代实体级 Tag 系统
3. 🟥 禁止面向对象式实体调用（如 `player.attack(enemy)`）
4. 🟥 禁止非确定性随机源破坏回放
5. 🟥 禁止 UI 层持有业务真相，UI 只能只读展示
6. 🟥 禁止直接修改最终属性值，必须通过 Modifier 管线
7. 🟥 禁止 Core 层引入渲染、音频、资源、输入等引擎表现能力
8. 🟥 禁止 Shared 层引入任何业务逻辑或业务类型
9. 🟥 禁止反向依赖与循环依赖
10. 🟥 禁止硬编码游戏数值与业务内容
11. 🟥 禁止核心业务领域使用 unwrap/expect/panic/todo
12. 🟥 禁止全局统一错误大枚举与 anyhow 滥用
13. 🟥 禁止为临时副作用随意新增领域事件
14. 🟥 禁止凭感觉优化突破架构边界
15. 🟥 禁止 Capabilities 层包含具体业务规则，突破机制与业务的边界
16. 🟥 禁止 Domain 之间直接依赖、直接调用内部实现
17. 🟥 禁止 Domain 重复实现 Capabilities 已有的通用机制
18. 🟥 禁止在 Rust 代码中硬编码任何用户可见文本（技能名称、描述、对话、UI 标签、错误提示等），所有用户可见文本必须通过 LocalizationKey 从外部本地化文件引用
19. 🟥 禁止用 Tag 替代参与规则计算的 Type — `if target.has_tag("boss")` 决定伤害公式是错误的，应使用强类型（DamageType::Fire）。Tag 只做语义描述，不参与规则计算
20. 🟥 禁止用 Tag 表达动态状态 — `Character.Dead`、`Character.Stunned` 应使用 ECS Component（struct Dead;），不用 Tag System（TagSet）。Tag 描述长期不变语义（Enemy.Boss, Character.Human）
21. 🟥 禁止 Tag 命名空间随意新增 — 顶级命名空间控制在 12 个以内，新增需架构评审。否则几年后 Skill.Fire 和 Ability.Fire 同时存在，直接灾难
22. 🟥 禁止 Tag 承载数据 — Tag 只回答"是不是"，不能回答"多少"。`Damage.100`、`Level.30`、`Cooldown.3` 均为非法 Tag
23. 🟥 禁止直接修改 Camera Entity 的 Transform/Projection/GlobalTransform，所有镜头操作必须通过 `commands.trigger(CameraRequest::...)` 事件驱动——违反 Event 驱动原则，导致 Camera 状态机被绕过，不可 Replay（ADR-064）
24. 🟥 Camera 模块禁止依赖 `core::domains::*` 的任何类型——Camera 是 Infra 层表现模块，不应感知 Combat/Dialogue/Unit 等业务领域（ADR-064）
25. 🟥 禁止 Tile 承载 Gameplay 数值（move_cost/defense_bonus 等），Tile 只存 terrain_id，地形数值归 TerrainDef Config Registry（ADR-065）
26. 🟥 禁止运行时加载/解析 TMX 文件，TMX 是编辑格式仅由 Importer 处理，游戏二进制不包含 TMX 解析逻辑（ADR-065）
27. 🟥 禁止 Map Object 直接实例化为 ECS Entity，Object 是定义（不可变），实例化策略属 Domain 职责（ADR-065）
28. 🟥 禁止引入 bevy_ecs_tilemap，地图渲染自研（ADR-065）
29. 🟥 禁止运行时修改 MapAsset 数据，MapAsset 是 Definition（不可变），加载后不得回写（ADR-065）

---

## 第二十二编 Localization（国际化）专项规则

### 22.1 核心原则（P0 级）

| # | 规则 | 等级 | 说明 |
|---|------|------|------|
| 22.1.1 | **代码中绝对禁止出现用户可见文本** | 🟥 | 代码中只允许出现 LocalizationKey，不允许出现任何中文/英文/日文等用户可见自然语言文本 |
| 22.1.2 | **Def 只存 LocalizationKey** | 🟥 | AbilityDef、EffectDef、ItemDef、QuestDef 等所有 Definition 类型的文本字段必须使用 name_key/desc_key/text_key，禁止直接存储用户可见字符串 |
| 22.1.3 | **Replay/Event 只存 Key+参数** | 🟥 | BattleLog、领域事件、回放帧中禁止保存最终翻译文本，必须使用 Key + 结构化参数，确保语言切换时正确渲染 |
| 22.1.4 | **存档禁止保存翻译文本** | 🟥 | 存档中只能存储 ID/Key，禁止保存任何翻译结果，确保切语言、更新翻译、Mod 覆盖全部安全 |
| 22.1.5 | **Localization 属于 Infrastructure 层** | 🟩 | Localization 是全局基础设施，不属于 UI 层，不属于 Capabilities 能力层。所有用户可见文本的唯一下游 |

### 22.2 LocalizationKey 规范

| # | 规则 | 等级 | 说明 |
|---|------|------|------|
| 22.2.1 | **Key 格式** | 🟩 | `LocalizationKey ::= <namespace> "." <scope> "." <id> "." <suffix>` |
| 22.2.2 | **Key 使用无语义 ID** | 🟩 | 优先使用 `ability.abl_000042.name` 而非 `ability.fireball.name`，避免业务重命名导致 Key 失效 |
| 22.2.3 | **Key 必须稳定** | 🟩 | Key 一旦分配永久有效，删除时标记 deprecated，不重新分配 |
| 22.2.4 | **命名空间分层** | 🟩 | L0 Core（系统文本）→ L1 UI（界面文本）→ L2 Gameplay（玩法文本）→ L3 Story（剧情文本），生命周期从稳定到高频变化 |
| 22.2.5 | **必须使用 Fluent (.ftl) 格式** | 🟨 | 优先使用 Fluent (.ftl) 作为本地化文件格式，利用其变量插值、复数规则、性别支持能力 |
| 22.2.6 | **禁止手写复数逻辑** | 🟥 | 复数规则必须交给 Fluent 内置复数系统处理，禁止在代码中手写 if-en/other 等复数判断 |

### 22.3 基础设施与工具

| # | 规则 | 等级 | 说明 |
|---|------|------|------|
| 22.3.1 | **LocalizationPlugin** | 🟩 | 必须建立 `LocalizationPlugin` 统一管理本地化生命周期，注册在 Content Plugin 之后、UI Plugin 之前 |
| 22.3.2 | **LocalizationKey 自动生成 Rust 常量** | 🟩 | 必须通过 build.rs 从 .ftl 文件自动生成 Rust 常量模块（如 `loc::ability::abl_000042::NAME`），提供编译期 Key 检查 |
| 22.3.3 | **启动时完整性校验** | 🟩 | 启动时必须对所有已注册的 LocalizationKey 进行完整性检查，缺失 Key 直接阻止启动 |
| 22.3.4 | **Fake Locale (zz-ZZ)** | 🟨 | 必须建立 zz-ZZ 伪语言 locale 用于检测硬编码文本，通过 feature flag 启用 |
| 22.3.5 | **三级回退链** | 🟩 | `{locale}` → `en-US` → `raw_key` 三级回退，禁止直接显示 [Missing Localization] |
| 22.3.6 | **热加载支持** | 🟨 | 修改 .ftl 文件必须热加载生效，无需重启游戏 |
| 22.3.7 | **LocalizedTextCache** | 🟨 | 运行时必须使用缓存避免每帧查询 LocalizationDatabase，语言切换时清空重建 |
| 22.3.8 | **Mod 覆盖链** | 🟩 | 支持 Base Game → DLC → Mod 三级本地化覆盖链 |
| 22.3.9 | **文本长度预算** | 🟨 | UI 设计必须为多语言预留扩展空间（建议 30%~50%），CI 自动检查超长文本 |

### 22.4 CI 与审计

| # | 规则 | 等级 | 说明 |
|---|------|------|------|
| 22.4.1 | **CI Localization 检查** | 🟩 | CI 必须包含缺失 Key、重复 Key、未引用 Key、参数不匹配、文本长度超限等本地化检查 |
| 22.4.2 | **翻译覆盖率报告** | 🟩 | 必须定期生成按分类（UI/Gameplay/Quest/Story/Tutorial）的翻译覆盖率报告 |
| 22.4.3 | **废弃 Key 管理** | 🟩 | 支持 deprecated Key 标记，审计输出废弃 Key 列表供清理 |
| 22.4.4 | **术语库（Glossary）** | 🟨 | 必须建立项目术语库，确保术语翻译全项目一致 |

### 22.5 语音预留

| # | 规则 | 等级 | 说明 |
|---|------|------|------|
| 22.5.1 | **文本设计预留语音** | 🟨 | 对话数据设计时预留 voice_key/subtitle 字段，即使当前不做配音 |
| 22.5.2 | **Key 体系支持语音扩展** | 🟩 | `story.ch01.dlg_001` 天然支持 text/voice/subtitle 三层扩展 |
