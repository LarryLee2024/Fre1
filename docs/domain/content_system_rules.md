# 内容系统领域

Version: 1.0
Status: Proposed

内容系统领域管理游戏内容数据（RON 配置文件）与游戏规则代码（Rust）之间的分离、加载、校验和热重载机制。

核心原则：
- 🟩 **1.1.3 Rule/Content 强制分离**：代码是规则，配置是内容（宪法条款 1.1.3）
- 🟩 **1.1.2 Definition/Instance 强制分离**：配置不可变，运行时可变（宪法条款 1.1.2）
- 🟩 **1.1.5 数据驱动优先**：所有可配置内容必须通过数据驱动实现（宪法条款 1.1.5）
- 🟩 **12.1.1 职责划分**：配置定义内容，代码解释配置（宪法条款 12.1.1）
- 🟩 新增内容只改 RON 文件，禁止修改 Rust 代码

---

# 术语定义

## Rule（规则）

游戏逻辑的 Rust 代码实现，定义"怎么做"。

不是配置数据。不是美术资源。

关键属性：
- 规则代码解释配置数据
- 规则代码不包含具体数值
- 规则代码修改需要程序员

---

## Content（内容）

游戏数据化的 RON 配置文件，定义"是什么"。

不是 Rust 代码。不是美术资源。

关键属性：
- 内容由策划或数据驱动工具编辑
- 新增内容不需要修改 Rust 代码
- 内容文件存放在 `content/` 目录

---

## Definition（定义态）

从 RON 文件反序列化的不可变配置数据，使用字符串标签（TagName）。

不是运行时数据。不是可变状态。

关键属性：
- 加载后不可修改
- 使用字符串标签（便于阅读和编辑）
- 存在于 Registry 中作为全局唯一真相源

---

## Instance（实例态）

运行时创建的可变状态数据，使用位掩码标签（GameplayTag）。

不是配置数据。不是定义态。

关键属性：
- 每个实体独立拥有
- 可以在运行时被修改
- 引用 Definition ID 而不是复制 Definition

---

## Registry（注册表）

存储所有 Definition 的全局不可变 Resource。

不是可变缓存。不是临时数据结构。

关键属性：
- 游戏启动时加载完成
- 加载后不可修改（热重载除外）
- 通过 ID 快速查询

---

## 阶段式加载（Staged Loading）

Raw Asset → Baker System → Baked Asset 的两阶段加载架构。

不是单阶段加载。不是 AssetLoader 直接生成最终数据。

> **优化来源**: `docs/architecture/content_data_format.md` §阶段式加载架构 — 解决 Bevy Asset 系统的校验断层

关键属性：
- Stage 1：AssetLoader 只负责反序列化（RON → RawSkillDef），包含字符串 ID 和未校验的数据
- Stage 2：Baker System 监听 AssetEvent::Added，执行依赖图构建、交叉校验，生成 BakedSkillDef
- 游戏逻辑只读取 Baked Asset（包含 Handle<BuffDef> 等强类型引用）
- Raw Asset 包含未解析的字符串 ID 引用
- Baked Asset 包含解析后的强类型引用

---

## 命名空间强制 ID 格式（Namespace-forced ID）

所有配置 ID 必须遵循 `namespace:type:name` 格式，MOD 生态的基石。

不是可选格式。不是简单字符串。

> **优化来源**: `docs/architecture/content_data_format.md` §2 — 彻底杜绝 MOD 内容冲突

关键属性：
- 格式：`namespace:type:name`（如 `core:skill:fireball`、`fire_mod:buff:inferno`）
- `core:` — 基础内容（游戏本体）
- `mod_xxx:` — MOD 内容（MOD 名称作为前缀）
- 字符集：仅允许字母、数字、下划线、冒号
- 长度限制：≤ 64 字符
- 彻底杜绝"两个 MOD 作者都写了个叫 fireball 的技能"的冲突

---

## 字符串 ID 驻留（String ID Interning）

加载期将字符串 ID 转换为 u64，运行时使用整数进行查表和比对。

不是运行时字符串比较。不是可选优化。

> **优化来源**: `docs/architecture/content_data_format.md` §字符串 ID 驻留 — 运行时 HashMap 热路径性能提升 10 倍以上

关键属性：
- 加载期通过 IdInterner 将字符串 ID 转换为 u64
- Baked Asset 中只存储 u64 ID
- 运行时逻辑全部使用 u64 进行查表和比对
- 性能收益：运行时查表和比对从 O(n) 字符串比较变为 O(1) 整数比较

---

## MOD 冲突解决（MOD Conflict Resolution）

按 load_order 决定覆盖优先级，数组字段支持 Append/Replace 策略。

不是随机覆盖。不是禁止 MOD 冲突。

> **优化来源**: `docs/architecture/content_data_format.md` §MOD 冲突解决协议

关键属性：
- 覆盖优先级：基础内容 → 官方 DLC → 第三方 MOD（按 load_order 排序）
- 后加载的覆盖先加载的
- 数值/字符串字段：Replace（直接替换）
- 数组/列表字段：Replace（整体替换）或 Append（追加），需显式声明
- 嵌套结构：Deep Merge（深度合并）或 Replace（整体替换）
- 冲突日志：当发生 MOD 覆盖时，系统必须在 debug.log 中输出 Warning

---

## 字段级审计（Field-level Audit）

每个配置字段追踪 version_added，支持配置迁移和兼容性判断。

不是可选元数据。不是文件级版本号。

> **优化来源**: `docs/architecture/content_data_format.md` §字段级审计

关键属性：
- 每个字段标注 `version_added: "1.2.0"` 表示引入版本
- 配置迁移时知道哪些字段是新增的
- MOD 兼容性检查：MOD 使用的字段版本是否与当前游戏版本兼容
- 废弃字段追踪：标记 `deprecated_at: "1.3.0"` 的字段可在大版本更新时清理

---

## CurveTable（曲线表）

数据驱动的数值映射，所有随等级、阶段、条件变化的数值通过曲线表配置。

不是硬编码 match 分支。不是运行时计算。

> **优化来源**: `docs/architecture/content_data_format.md` §曲线表配置

关键属性：
- 所有随等级变化的数值必须通过 CurveTable 配置
- 插值方法：Step（阶梯）、Linear（线性）、Spline（样条）
- 关键点列表必须按 key 升序排列
- 通过 CurveId 被 Formula 系统引用
- 禁止硬编码 `match level { ... }` 数值映射

---

## 内容桥接层

加载 RON 配置数据、校验引用完整性、注册到 Registry 的代码层。

不是游戏规则层。不是表现层。

关键属性：
- 只做三件事：加载、校验、注册
- 位于 `src/content/` 目录
- 依赖 Core 和 Infrastructure

---

## AssetServer 异步加载（AssetServer Async Loading）

AssetServer::load() 返回 Handle<T> 而非 T，文件在后台 IO 线程中读取。

不是同步加载。不是立即返回数据。

> **优化来源**: `docs/architecture/content-pipeline.md` §致命异步时序错误修正

关键属性：
- AssetServer::load() 是非阻塞异步的，立即返回 Handle<T>
- 调用 load() 的同一帧内无法拿到数据并执行 .into()
- 必须使用 AssetEvent 响应式管线：监听 AssetEvent::Added 触发后续处理
- 禁止在 App::build 中同步加载资产（阻塞启动，WASM 不兼容）

---

## 加载进度屏障（Loading Barrier）

validate_all_references 必须在所有 Asset 完全加载后执行，否则会因引用缺失报满屏 Error。

不是状态切换时立即校验。不是加载完成后异步校验。

> **优化来源**: `docs/architecture/content-pipeline.md` §加载进度屏障

关键属性：
- 禁止在 OnEnter(AppState::InGame) 时立即执行 validate_all_references
- 使用 LoadingProgress Resource 跟踪加载进度
- 只有当 loaded >= total 时，才允许执行引用校验
- 校验通过后才允许状态机切换到 AppState::InGame

---

## Definition 即数据资产（Definition as Data Asset）

Definition 不是 ECS Component，而是可序列化的数据资产（Godot Resource / Unity ScriptableObject）。

不是 ECS Component。不是运行时状态。

> **优化来源**: `docs/architecture/content-pipeline.md` §Definition 即数据资产

关键属性：
- Godot Resource（.tres/.res）等价于 SkillDef/BuffDef/CharacterDef
- Unity ScriptableObject 等价于同上
- Bevy RON 配置 + AssetLoader 是数据资产的 Bevy 落地形态
- 不要什么都 spawn(Entity) 加 Component — 应该先定义 Definition，然后从 Definition 生成 Instance
- 定义态在内容管线中创建和加载，实例态在运行时由 System 从 Registry 中查询并生成

---

## Scene 模式（Scene Pattern）

大型 SRPG 项目应借鉴 Godot 的 Scene 组织方式，将相关 Entity 组织为独立的场景/Plugin。

不是什么都 spawn(Entity) 然后散落全局。不是单一 Plugin 管理所有 Entity。

> **优化来源**: `docs/architecture/content-pipeline.md` §Scene 模式

关键属性：
- BattleScene：独立 Plugin，管理战斗相关的所有 Entity 生命周期
- TownScene：独立 Plugin，管理城镇界面的 Entity 生命周期
- WorldMapScene：独立 Plugin，管理大地图的 Entity 生命周期
- 每个 Scene 是独立的 Bevy Plugin，拥有自己的 Entity 生命周期
- Scene 切换时清理该 Scene 的所有 Entity 和 Strong Handle
- 禁止跨 Scene 直接引用 Entity（必须通过 ID 查询）

---

## ScriptableObject 模式（Bevy 实现）

RON + AssetLoader + Registry 组合是 Unity ScriptableObject 的 Bevy 实现。

不是 const FIREBALL 硬编码。不是散落的配置数据。

> **优化来源**: `docs/architecture/content-pipeline.md` §ScriptableObject 模式的 Bevy 实现

关键属性：
- RON 配置文件：策划编辑的资产文件
- AssetLoader：运行时加载和反序列化
- Registry：全局注册表，查询入口
- 不要用 `const FIREBALL` 硬编码技能数据
- 所有游戏内容必须通过 Definition → Registry 管线

---

## 统一注册中心（Unified Registry Center）

所有内容类型的 Registry 共享相同的 ContentRegistry trait 模式。

不是各自为政的独立 Registry。不是统一的单一 Registry。

> **优化来源**: `docs/architecture/content-pipeline.md` §统一注册中心

关键属性：
- SkillRegistry：技能定义
- BuffRegistry：Buff 定义
- CharacterRegistry：角色模板
- QuestRegistry：任务定义
- FormulaRegistry：数值公式
- EquipmentRegistry：装备定义
- TerrainRegistry：地形定义
- 所有 Registry 共享相同的 insert/get/contains/all_ids/count 接口
- 启动时校验所有 Registry 交叉引用完整性
- 热重载时统一更新对应 Registry
- MOD 内容通过相同接口注册到 Registry

---

## RON格式契约（RON Format Contract）

配置文件的统一结构与字段要求，定义所有 RON 文件必须遵循的基础格式。

不是单个文件的格式。不是任意结构。

> **优化来源**: `docs/architecture/content_data_format.md` — RON 配置文件契约

关键属性：
- 每个 RON 文件必须包含：id、name、description、version、tags + 领域特定字段
- 引用机制：字符串 ID → 加载时 Strong ID 解析 → Registry 校验存在性
- 默认值规则：新增字段必须有默认值保证向后兼容
- MOD 内容格式：ID 前缀隔离、覆盖优先级规则
- 命名空间强制 ID 格式：`namespace:type:name`（如 `core:skill:fireball`）

---

# 领域边界

## 本领域负责

- RON 配置文件的格式规范
- Content 与 Rule 的分离规则
- Definition 与 Instance 的分离规则
- Registry 的加载和查询机制
- 内容校验管线（Schema、引用、规则三级校验）
- 热重载机制
- Content 目录与 Core 模块的对应关系
- 双类型模式（Def ↔ Data 的转换）
- 阶段式加载（Raw Asset → Baker System → Baked Asset）
- 命名空间强制 ID 格式（MOD 生态基石）
- 字符串 ID 驻留（加载期转换为 u64）
- MOD 冲突解决（load_order + Append/Replace 策略）
- 字段级审计（version_added / deprecated_at）
- CurveTable 内容类型（数值映射配置）
- AssetServer 异步加载（Handle<T> 响应式管线）
- 加载进度屏障（LoadingProgress + validate_all_references 时机）
- Definition 即数据资产（Godot Resource / Unity ScriptableObject 类比）
- Scene 模式（独立 Plugin 管理 Entity 生命周期）
- ScriptableObject 模式（RON + AssetLoader + Registry 组合）
- 统一注册中心（ContentRegistry trait 模式）

## 本领域不负责

- 具体游戏规则的实现（由 Battle、Skill、Buff 等领域负责）
- 美术资源的管理（由 Asset Organization 领域负责）
- MOD 的加载和冲突管理（由 Modding System 领域负责）
- 存档的序列化和反序列化（由 Infrastructure 的 Persistence 模块负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 内容加载完成 | Message | Battle, Skill, Buff 等 Core 模块 |
| 内容引用校验失败 | Message | Debug, Infrastructure |
| 内容热重载通知 | Observer | UI, Core 各模块 |
| Definition 注册请求 | 函数调用 | Registry |

---

# 生命周期

## 状态列表

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Unloaded | 配置文件尚未加载 | Loading |
| Loading | 正在读取和反序列化 | Loaded, LoadFailed |
| Loaded | 配置已加载为 Def | Validating |
| LoadFailed | 加载失败（RON 语法错误） | Unloaded（重试） |
| Validating | 正在校验引用完整性 | Valid, Invalid |
| Valid | 校验通过，可注册到 Registry | Registered |
| Invalid | 校验失败（引用缺失等） | Validating（修复后重试） |
| Registered | 已注册到 Registry | HotReloading |
| HotReloading | 热重载中 | Registered, HotReloadFailed |

## 状态转换图

```
Unloaded → Loading → Loaded → Validating → Valid → Registered
                                ↓              ↓
                            Invalid       LoadFailed
                                ↓
                          Validating（修复后）
                              
Registered → HotReloading → Registered
```

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Unloaded | Loading | 游戏启动或 Content Plugin 注册 |
| Loading | Loaded | RON 文件反序列化成功 |
| Loading | LoadFailed | RON 文件语法错误或字段缺失 |
| Loaded | Validating | Content Plugin 系统触发校验 |
| Validating | Valid | 所有引用 ID 存在、标签有效 |
| Validating | Invalid | 存在引用缺失或标签无效 |
| Valid | Registered | Registry.insert() 成功 |
| Registered | HotReloading | AssetServer 检测到文件变更 |

---

# 不变量

## 不变量1：Definition 不可变性

任意时刻：

Definition 对象从 RON 文件加载后，其任何字段不被修改。

违反表现：

`skill_def.damage = 100` 赋值语句出现。`buff_data.duration += 1` 运算出现。

---

## 不变量2：内容不触代码原则

任意时刻：

新增技能、Buff、装备、关卡等游戏内容时，不修改任何 Rust 源代码文件。

违反表现：

新增一个技能需要修改 `src/core/skill/` 下的任何代码。

---

## 不变量3：Registry 完整性

任意时刻：

Registry 中所有已注册的 Definition 的引用 ID 都指向存在且有效的目标。

违反表现：

`skill_def.effect_ids` 中的某个 ID 在 EffectRegistry 中不存在。

---

## 不变量4：Def/Data 双类型一致性

任意时刻：

`impl From<XxxDef> for XxxData` 的转换必须保持语义一致性——TagName 字符串正确转换为 GameplayTag 位掩码。

违反表现：

转换前后标签集合不一致。Tag 名拼写错误导致位掩码为 0。

---

## 不变量5：加载进度屏障

任意时刻：

validate_all_references 必须在所有 Asset 完全加载后执行。禁止在 OnEnter(AppState::InGame) 时立即执行校验。

> **优化来源**: `docs/architecture/content-pipeline.md` §加载进度屏障

违反表现：

加载未完成时执行引用校验，因大量引用找不到而报出满屏 Error，甚至导致游戏判定加载失败。

---

## 不变量6：AssetServer 异步时序

任意时刻：

AssetServer::load() 返回 Handle<T>，文件在后台 IO 线程中读取。禁止假设 load() 同一帧内能拿到数据。

> **优化来源**: `docs/architecture/content-pipeline.md` §致命异步时序错误修正

违反表现：

在 load() 后立即 unwrap Handle 获取数据，导致 panic（文件尚未加载完成）。

---

# 规则

## 规则1：Rule/Content 强制分离

允许：
- 新增 RON 配置文件添加游戏内容
- 通过 Data 驱动的机制扩展游戏行为
- Config 热重载修改数值平衡

禁止：
- 新增内容时修改 Rust 规则代码
- 在 RON 文件中硬编码游戏逻辑
- 绕过 Registry 直接创建运行时数据

必须：
- 新增技能只创建 RON 文件
- 新增职业只创建 RON 文件
- 新增关卡只创建 RON 文件

---

## 规则1b：平衡参数零硬编码

> 🟩 **宪法条款 12.4.1**：所有平衡参数（暴击倍率、地形加成、系数常量）必须放入配置文件

允许：
- 所有数值平衡参数存储在 RON 配置文件中
- 通过 CurveTable 配置随等级变化的数值

禁止：
- 🟥 在 Rust 代码中硬编码魔法数字（如 `const CRIT_MULTIPLIER: f32 = 1.5`）
- 🟥 硬编码数值映射（如 `match level { 1 => 100, 2 => 250, ... }`）

必须：
- 所有影响游戏平衡的数值通过 RON 配置加载
- 数值调整无需修改代码，策划可独立调优

---

## 规则1c：所有配置必须有 Schema

> 🟩 **宪法条款 12.2.1**：SkillConfig、BuffConfig、CharacterConfig 等所有配置结构体，必须对应明确的 Schema 定义

允许：
- 每个 RON 配置文件有明确的字段类型和结构定义
- Schema 校验在加载时自动执行

禁止：
- 🟥 没有 Schema 定义的配置文件
- 🟥 配置字段漂移（字段名或类型不一致）

必须：
- 所有 RON 配置文件必须有对应的 Schema 定义
- Schema 校验在 Level 1（Schema 校验）阶段执行

---

## 规则1d：配置唯一事实源

> 🟩 **宪法条款 12.3.1**：每份配置数据必须有唯一的归属 Feature 与维护入口

允许：
- 每种内容类型（Skill、Buff、Character）有独立的 Registry
- 每个 Registry 有唯一的归属 Feature

禁止：
- 🟥 多个 Feature 同时维护同一份核心配置数据
- 🟥 配置数据分散在多个位置

必须：
- CharacterDef 归属 character 模块，SkillDef 归属 skill 模块
- 配置数据通过 Registry 统一管理

---

## 规则2：Definition/Instance 分离

允许：
- Definition 加载后作为全局不可变参考
- Instance 独立创建并引用 Definition ID
- 运行时修改 Instance 状态

禁止：
- 运行时修改 Definition 的任何字段
- 在 Instance 中硬编码配置数据（应引用 Definition）
- Instance 把 Definition 复制为自有数据

必须：
- Instance 引用 Definition ID（不是复制内容）
- 加载时从 Registry 恢复 Definition
- 穿戴装备后重建 TraitCollection

---

## 规则3：三级校验管线

允许：
- Level 1（Schema）校验 RON 语法和字段类型
- Level 2（引用）校验 ID 引用的完整性
- Level 3（规则）校验游戏逻辑一致性

禁止：
- 跳过 Level 1 直接加载
- 加载后不校验引用完整性
- 忽略校验错误继续运行

必须：
- Level 1 在 Content Plugin 加载时执行
- Level 2 在所有 Content 加载完成后执行
- Level 3 在开发工具中执行（非发布构建）

---

## 规则4：Content 目录与 Core 模块对应

允许：
- 每个 Core 模块有对应的 Content 目录
- Content 目录名与其 Core 模块名一致

禁止：
- Content 目录名与 Core 模块名不一致
- 一个 Content 目录对应多个 Core 模块
- Content 文件中包含游戏逻辑代码

必须：
- `content/skills/` 对应 `src/core/skill/`
- `content/buffs/` 对应 `src/core/buff/`
- `content/equipments/` 对应 `src/core/equipment/`

---

## 规则5：热重载安全机制

允许：
- 开发模式下热重载 Definition 数据
- 热重载更新 Registry 中的配置

禁止：
- 热重载修改运行中的 Instance 数据
- 热重载影响进行中的战斗回合
- 生产构建中启用热重载

必须：
- 热重载只更新 Registry（不可变配置）
- 热重载不更新已有 Instance（运行时状态）
- 热重载后广播 SkillDataReloaded / BuffDataReloaded 等事件

---

## 规则6：RON 格式契约

允许：
- 每个 RON 文件包含 id、name、description、version、tags + 领域特定字段
- 新增字段提供默认值（保证向后兼容）
- MOD 配置使用 ID 前缀隔离（如 "fire_mod.inferno"）

禁止：
- 硬编码配置值在 Rust 代码（必须从 RON 加载）
- 配置循环引用（加载顺序不确定）
- MOD 配置绕过冲突检测（必须通过 MOD 加载器）

必须：
- 每个 RON 文件包含 id（唯一标识符）和 version（格式版本号）
- 新增字段必须有默认值（Option→None、Vec→[]、bool→false、u32→0、String→""）
- 引用的 ID 在对应 Registry 中存在
- MOD 内容的 ID 必须加 MOD 前缀避免冲突

---

## 规则7：引用校验管线

允许：
- 字符串 ID → 加载时 Strong ID 解析 → Registry 校验存在性
- 必填引用缺失时加载失败并报告错误
- 可选引用缺失时跳过该引用使用默认值

禁止：
- 引用不存在的 ID（必须校验）
- 配置文件之间循环引用（必须检测并拒绝）
- 加载后不校验引用完整性

必须：
- 引用校验在所有 Content 加载完成后执行
- 缺失引用记录 WARN 日志并跳过（可选引用）或返回错误（必填引用）
- 引用类型不匹配时加载失败

---

## 规则8：阶段式加载

允许：
- AssetLoader 只负责反序列化（Raw Asset）
- Baker System 执行依赖图构建和交叉校验
- 游戏逻辑只读取 Baked Asset

禁止：
- AssetLoader 直接生成最终的 Baked Asset
- 游戏逻辑读取 Raw Asset（包含未解析的字符串 ID）
- 跳过 Baker System 直接注册到 Registry

必须：
- Stage 1（Raw Asset）包含字符串 ID 和未校验的数据
- Stage 2（Baked Asset）包含解析后的强类型引用
- Baker System 监听 AssetEvent::Added 触发转换

---

## 规则9：命名空间强制 ID 格式

允许：
- 基础内容使用 `core:type:name` 格式（如 `core:skill:fireball`）
- MOD 内容使用 `mod_xxx:type:name` 格式
- 字符集：字母、数字、下划线、冒号

禁止：
- 使用不带命名空间的 ID（如 `fireball`）
- 命名空间包含特殊字符
- ID 长度超过 64 字符

必须：
- 所有配置 ID 遵循 `namespace:type:name` 格式
- MOD 内容的命名空间使用 MOD 名称作为前缀
- 加载时自动解析命名空间到对应的 Registry

---

## 规则10：字符串 ID 驻留

允许：
- 加载期将字符串 ID 转换为 u64
- Baked Asset 中只存储 u64 ID
- 运行时使用 u64 进行查表和比对

禁止：
- 运行时使用字符串 ID 进行高频查表（性能问题）
- Baked Asset 中存储未驻留的字符串 ID
- 驻留 ID 映射在运行时被修改

必须：
- 加载期通过 IdInterner 执行字符串到 u64 的转换
- 驻留后的 u64 ID 在整个运行时生命周期内保持稳定
- 热重载时重新执行驻留（字符串 ID 可能变化）

---

## 规则11：MOD 冲突解决

允许：
- 按 load_order 决定覆盖优先级（后加载覆盖先加载）
- 数值字段使用 Replace（直接替换）
- 数组字段使用 Append（追加）或 Replace（整体替换）
- 嵌套结构使用 Deep Merge（深度合并）

禁止：
- MOD 之间无优先级地覆盖同一字段
- 数组字段默认使用 Append（必须显式声明）
- 覆盖时不记录冲突日志

必须：
- 覆盖优先级：基础内容 → 官方 DLC → 第三方 MOD
- 冲突发生时在 debug.log 中输出 Warning
- 合并后执行引用完整性校验

---

## 规则12：字段级审计

允许：
- 每个字段标注 `version_added` 表示引入版本
- 废弃字段标注 `deprecated_at` 表示废弃版本
- 配置迁移时根据版本号判断字段兼容性

禁止：
- 新增字段不标注 version_added
- 删除字段不标注 deprecated_at
- 修改字段类型而不递增版本号

必须：
- 新增字段提供默认值并标注 version_added
- 破坏性变更提供迁移脚本
- MOD 兼容性检查时校验字段版本

---

## 规则13：曲线表内容类型

允许：
- 所有随等级、阶段、条件变化的数值通过 CurveTable 配置
- 使用 Step（阶梯）、Linear（线性）、Spline（样条）三种插值方法
- 曲线表通过 CurveId 被 Formula 系统引用

禁止：
- 硬编码 match level 分支（如 `match level { 1 => 100, 2 => 250, ... }`）
- 不按 key 升序排列关键点
- 运行时修改曲线表数据

必须：
- 曲线表关键点列表按 key 升序排列
- 插值方法在配置时明确指定
- 曲线表遵循 RON 格式契约（id、name、version、tags）

---

## 规则14：AssetServer 异步加载

允许：
- AssetServer::load() 返回 Handle<T>，文件在后台 IO 线程中读取
- 使用 AssetEvent 响应式管线处理加载完成事件
- 使用 load_folder 批量加载目录下的所有资产

禁止：
- 在 App::build 中同步加载资产（阻塞启动，WASM 不兼容）
- 假设 load() 同一帧内能拿到数据
- 在 AssetLoader 中执行游戏逻辑

必须：
- 监听 AssetEvent::Added 触发后续处理（Baker System / Registry 填充）
- 使用 AssetServer::is_loaded_with_dependencies() 检查加载状态
- 热重载时监听 AssetEvent::Modified 更新 Registry

---

## 规则15：加载进度屏障

允许：
- 使用 LoadingProgress Resource 跟踪加载进度
- 只有当 loaded >= total 时才允许执行引用校验
- 校验通过后才允许状态机切换到 AppState::InGame

禁止：
- 在 OnEnter(AppState::InGame) 时立即执行 validate_all_references
- 加载未完成时执行引用校验（会因引用缺失报满屏 Error）
- 跳过加载进度检查直接切换状态

必须：
- 在 AppState::Loading 阶段触发所有 load_folder
- 使用 AssetServer::is_loaded_with_dependencies() 轮询进度
- 所有 Asset 加载完成后才允许执行引用校验

---

## 规则16：Definition 即数据资产

允许：
- Definition 作为可序列化的数据资产（Godot Resource / Unity ScriptableObject）
- 策划编写 RON → XxxDef（数据资产） → Registry → 从 Definition 生成 Instance
- 使用 Bevy RON 配置 + AssetLoader 作为数据资产的落地形态

禁止：
- 什么都 spawn(Entity) 加 Component（应先定义 Definition）
- 用 `const FIREBALL` 硬编码技能数据
- 直接从 RON 文件生成运行时 Instance（应经过 Registry）

必须：
- 所有游戏内容通过 Definition → Registry 管线
- 定义态在内容管线中创建和加载
- 实例态在运行时由 System 从 Registry 中查询并生成

---

## 规则17：Scene 模式

允许：
- 每个 Scene 是独立的 Bevy Plugin，拥有自己的 Entity 生命周期
- Scene 切换时清理该 Scene 的所有 Entity 和 Strong Handle
- 新内容通过 Plugin 合并注册，不散落在全局

禁止：
- 什么都 spawn(Entity) 然后散落全局
- 跨 Scene 直接引用 Entity（必须通过 ID 查询）
- 单一 Plugin 管理所有 Scene 的 Entity

必须：
- BattleScene / TownScene / WorldMapScene 各自独立
- Scene 切换时执行完整的资源清理
- Scene 内部通过 Entity 管理生命周期

---

## 规则18：ScriptableObject 模式

允许：
- 使用 RON + AssetLoader + Registry 组合实现 ScriptableObject 模式
- 策划编辑 RON 配置文件
- AssetLoader 负责运行时加载和反序列化
- Registry 作为全局查询入口

禁止：
- 用 `const FIREBALL` 硬编码技能数据
- 散落的配置数据（应通过 Registry 统一管理）
- 跳过 AssetLoader 直接解析 RON 文件

必须：
- 所有游戏内容通过 Definition → Registry 管线
- RON 配置文件遵循格式契约
- AssetLoader 实现自定义的 AssetLoader trait

---

## 规则19：统一注册中心

允许：
- 所有 Registry 共享相同的 ContentRegistry trait 模式
- 启动时校验所有 Registry 交叉引用完整性
- 热重载时统一更新对应 Registry

禁止：
- 各自为政的独立 Registry（应遵循统一模式）
- Registry 之间不校验交叉引用
- MOD 内容绕过统一注册接口

必须：
- 所有 Registry 实现 insert/get/contains/all_ids/count 接口
- 启动时遍历所有 Registry 交叉校验引用
- MOD 内容通过相同接口注册到 Registry

---

# 管线

## 内容加载管线

```
RON 文件 → AssetServer::load() 返回 Handle<T> → AssetLoader 反序列化（Raw Asset） → Baker System 校验+转换（Baked Asset） → Registry.insert()
```

> **优化来源**: `docs/architecture/content-pipeline.md` §致命异步时序错误修正 + §阶段式加载架构

### Step1：AssetServer 异步加载

输入：`content/skills/fireball.ron`
处理：AssetServer::load() 返回 Handle<T>（非阻塞异步）
输出：Handle<SkillDef>（文件在后台 IO 线程中读取）
禁止：假设 load() 同一帧内能拿到数据

### Step2：AssetLoader 反序列化（Stage 1: Raw Asset）

输入：Handle<SkillDef> 对应的文件内容
处理：AssetLoader 反序列化为 RawSkillDef（包含字符串 ID 和未校验的数据）
输出：RawSkillDef 对象
禁止：在 AssetLoader 中执行游戏逻辑

### Step3：Baker System 校验+转换（Stage 2: Baked Asset）

输入：RawSkillDef 对象 + AssetEvent::Added 事件
处理：依赖图构建、交叉校验、引用解析，生成 BakedSkillDef
输出：BakedSkillDef 对象（包含 Handle<BuffDef> 等强类型引用）
禁止：游戏逻辑读取 Raw Asset

### Step4：引用完整性校验

输入：所有 BakedSkillDef 对象
处理：检查所有 ID 引用的目标是否存在
输出：校验通过或缺失引用报告
禁止：加载过程中跳过引用校验

### Step5：Registry 注册

输入：校验通过的 XxxData 对象
处理：`registry.insert(data.id.clone(), data)`
输出：全局可查询的 Registry
禁止：注册后修改 Registry 内容（热重载除外）

---

## 热重载管线

```
文件变更 → AssetServer 检测 → 重新加载 Def → 转换为 Data → 更新 Registry → 广播事件
```

### Step1：文件变更检测

输入：文件系统变更事件
处理：检测 content/ 目录下的 .ron 文件变更
输出：变更的文件路径列表
禁止：检测 src/ 目录下的代码变更

### Step2：重新加载

输入：变更的文件路径
处理：重新执行加载管线 Step1-Step4
输出：更新后的 XxxData 对象
禁止：修改已有 Instance 数据

### Step3：Registry 更新与事件广播

输入：更新后的 XxxData 对象
处理：更新 Registry 中对应条目，广播 XxxDataReloaded 事件
输出：UI 和其他模块收到更新通知
禁止：直接通知战斗系统重算当前回合（只广播事件，不触发逻辑）

---

## 引用校验管线

```
解析 RON → 提取 ID 引用 → 查询 Registry → 存在→OK / 缺失→warn+跳过
```

### Step1：解析 RON

输入：RON 文件内容
处理：反序列化为 XxxDef 结构
输出：XxxDef 对象（包含字符串 ID 引用）
禁止：跳过 Schema 校验

### Step2：提取 ID 引用

输入：XxxDef 对象
处理：遍历所有引用字段，提取字符串 ID 列表
输出：ID 引用列表（按类型分组）
禁止：忽略可选引用字段

### Step3：查询 Registry

输入：ID 引用列表
处理：逐个查询对应 Registry 是否存在该 ID
输出：存在/缺失状态
禁止：跳过查询直接标记为 OK

### Step4：校验结果处理

输入：查询结果
处理：存在→OK；必填缺失→ERROR + 加载失败；可选缺失→WARN + 跳过使用默认值
输出：校验通过/失败报告
禁止：忽略必填引用缺失继续加载

---

# 数据结构

## XxxDef（定义态）

职责：RON 反序列化用的中间类型，使用 TagName 字符串

结构：
- id：String — 唯一标识符
- name：String — 显示名称
- tags：Vec<String> — 标签名称列表
- 业务字段：各领域特有

要求：
- 必须实现 `Deserialize`
- 使用字符串标签（TagName），不使用位掩码
- 不包含任何运行时计算字段
- 从 RON 文件反序列化得到

---

## XxxData（运行时态）

职责：运行时使用的不可变配置类型，使用 GameplayTag 位掩码

结构：
- id：XxxId — 强类型 ID（UnitId, SkillId 等）
- name：String — 显示名称
- tags：GameplayTag — 位掩码标签
- 业务字段：各领域特有

要求：
- 必须实现 `From<XxxDef>`
- 使用位掩码标签，不使用字符串
- 不包含任何可变状态
- 注册到 Registry 后不可修改

---

## XxxRegistry（注册表）

职责：存储所有同类型 XxxData 的全局不可变 Resource

结构：
- entries：HashMap<XxxId, XxxData> — ID 到数据的映射

要求：
- 游戏启动时加载完成
- 加载后不修改（热重载更新除外）
- 通过 ID 快速 O(1) 查询
- 不存储运行时状态

---

# 禁止事项

禁止：新增内容时修改 Rust 代码

原因：Rule/Content 分离的核心原则。新增火球术只需创建 `content/skills/fireball.ron`，不应修改任何 Rust 代码。

违反后果：每次新增内容都需要程序员介入，内容生产效率归零。

---

禁止：运行时修改 Definition 字段

原因：Definition/Instance 分离的核心原则。Definition 是不可变配置，修改会导致其他引用该 Definition 的实例行为不一致。

违反后果：存档损坏、多实体引用同一 Definition 时行为不一致。

---

禁止：跳过引用校验直接注册到 Registry

原因：缺失引用会导致运行时 panic 或查询返回 None，破坏游戏完整性。

违反后果：技能引用不存在的 Buff、装备引用不存在的物品，运行时崩溃。

---

禁止：在 Content 层包含游戏规则逻辑

原因：Content 只做加载、校验、注册三件事。包含逻辑违反 Content/Rule 分离。

违反后果：内容扩展需要修改 Rust 代码，数据驱动形同虚设。

---

禁止：热重载修改运行中 Instance 数据

原因：热重载只更新 Definition（Registry），不更新已有 Instance（运行时状态）。当前战斗中的 Buff 实例不应因热重载而改变。

违反后果：正在进行的战斗回合数据不一致，可能导致崩溃或错误结果。

---

禁止：config/ 目录出现 xxx_utils 模块

原因：通用工具放 shared/，业务工具放各自领域。Content 层不需要通用工具。

违反后果：Content 层变成垃圾桶，加载校验逻辑混杂其中。

---

禁止：硬编码配置值在 Rust 代码

原因：违反数据驱动原则，新增内容必须修改代码。

违反后果：每次新增内容都需要程序员介入，数据驱动形同虚设。

---

禁止：配置循环引用

原因：加载顺序不确定，可能导致加载失败或无限循环。

违反后果：游戏启动时卡死或崩溃，无法加载配置。

---

禁止：代码中硬编码魔法数字

原因：违反宪法条款 12.4.1，所有平衡参数必须放入配置文件。

违反后果：每次数值调整都需要程序员介入，策划无法独立调优。

---

禁止：配置文件没有 Schema 定义

原因：违反宪法条款 12.2.1，AI 生成配置时无明确结构依据，字段漂移与格式错误。

违反后果：配置文件格式不一致，加载时解析失败。

---

禁止：多个 Feature 维护同一份配置数据

原因：违反宪法条款 12.3.1，多源修改导致数据不一致。

违反后果：配置数据冲突，加载时产生不可预测行为。

---

禁止：MOD 配置绕过冲突检测

原因：MOD 内容必须通过 MOD 加载器，跳过检测会导致 ID 冲突。

违反后果：MOD 覆盖基础内容时无日志记录，调试困难。

---

禁止：使用不带命名空间的 ID

原因：MOD 生态冲突，两个 MOD 可能定义同名配置。

违反后果：MOD 内容 ID 冲突，加载时覆盖错误的内容。

---

禁止：运行时使用字符串 ID 进行高频查表

原因：字符串比较性能差，运行时 HashMap 热路径性能下降 10 倍以上。

违反后果：战斗结算卡顿，帧率下降。

---

禁止：硬编码数值映射（match level 分支）

原因：修改数值映射必须重新编译代码，策划无法独立调优。

违反后果：每次数值调整都需要程序员介入，策划无法使用曲线表调优。

---

禁止：新增字段不标注 version_added

原因：配置迁移时无法判断字段兼容性，MOD 兼容性检查失败。

违反后果：配置迁移脚本无法正确处理新旧版本差异。

---

禁止：假设 AssetServer::load() 同步返回数据

原因：AssetServer::load() 是非阻塞异步的，同一帧内无法拿到数据。

违反后果：编译错误或运行时 panic（ unwrap None）。

---

禁止：在 OnEnter(AppState::InGame) 时立即执行 validate_all_references

原因：AssetServer 可能还在后台加载文件，此时执行校验会因引用缺失报满屏 Error。

违反后果：游戏判定加载失败，无法进入战斗。

---

禁止：什么都 spawn(Entity) 加 Component

原因：应先定义 Definition，然后从 Definition 生成 Instance。直接 spawn Entity 违反 Definition/Instance 分离。

违反后果：配置数据散落在 Entity 中，无法统一管理和热重载。

---

禁止：跨 Scene 直接引用 Entity

原因：Scene 切换时 Entity 会被清理，直接引用会导致悬垂引用。

违反后果：Scene 切换后访问已清理的 Entity 导致 panic。

---

禁止：各自为政的独立 Registry

原因：不遵循统一模式会导致加载、校验、热重载逻辑重复，MOD 支持困难。

违反后果：新增内容类型需要重复实现加载和校验逻辑。

---

# AI 修改规则

## 如果新增游戏内容（技能、Buff、装备等）

允许：
- 创建新的 RON 配置文件
- 确保所有引用 ID 存在
- 运行游戏验证内容可用

禁止：
- 修改任何 Rust 源代码文件
- 在 RON 中硬编码游戏逻辑表达式
- 创建没有规则引擎支持的全新内容类型

优先检查：
- 新内容的所有引用是否完整
- 新内容是否符合 Schema 校验
- 新内容是否可以通过现有规则引擎解释

---

## 如果新增内容类型（需要新规则引擎）

允许：
- 先在 Core 层实现新规则引擎
- 再创建对应的 Content 加载器
- 最后创建 RON Schema

禁止：
- 先创建 RON 文件却没有规则引擎支持
- 在 Content 层实现规则逻辑

优先检查：
- Core 层是否有对应的规则引擎
- Content 层是否有对应的加载器
- Schema 校验是否覆盖了新类型的所有字段

---

## 如果内容校验失败

排查顺序：
1. RON 语法是否正确（Level 1 Schema 校验）
2. 所有引用 ID 是否存在（Level 2 引用校验）
3. 数值是否在合理范围内（Level 3 规则校验）
4. Content 目录结构是否与 Core 模块对应

---

## 如果热重载后游戏异常

排查顺序：
1. 热重载是否只更新了 Registry 而非 Instance
2. 广播事件是否正确触发
3. 新旧 Definition 的字段结构是否一致（版本不兼容）
4. 是否有硬编码的 Definition 数据在 Instance 中

---

## 如果新增 RON 配置文件

允许：
- 在 content/ 对应目录创建新的 .ron 文件
- 遵循通用配置结构（id、name、version、tags）
- 在 RON 中添加注释说明配置含义

禁止：
- 不包含 id 和 version 字段
- 在 RON 中包含逻辑代码
- 引用不存在的 ID

优先检查：
- 配置结构是否与 Core 层的 Def 类型一致
- 所有引用的 ID 是否在对应 Registry 中存在
- 版本号是否正确递增

---

## 如果修改 RON 配置格式

允许：
- 新增可选字段（提供默认值）
- 优化配置结构（保持向后兼容）

禁止：
- 删除已有字段（破坏兼容性）
- 修改字段类型（破坏兼容性）
- 修改字段语义（破坏兼容性）

优先检查：
- 所有使用该配置格式的模块是否同步更新
- 版本号是否递增
- 是否需要提供迁移脚本
- 新字段是否有默认值（保证旧数据兼容）

---

## 如果新增 Scene

允许：
- 创建独立的 Bevy Plugin 管理 Scene 的 Entity 生命周期
- Scene 切换时清理所有 Entity 和 Strong Handle
- 新内容通过 Plugin 合并注册

禁止：
- 跨 Scene 直接引用 Entity
- 单一 Plugin 管理所有 Scene
- Scene 切换时不清理 Entity

优先检查：
- Scene 是否作为独立 Plugin 实现
- Entity 生命周期是否正确管理
- 是否有跨 Scene 引用

---

## 如果修改 AssetServer 加载逻辑

允许：
- 使用 AssetEvent 响应式管线处理加载完成事件
- 使用 load_folder 批量加载目录下的所有资产
- 使用 is_loaded_with_dependencies() 检查加载状态

禁止：
- 假设 load() 同一帧内能拿到数据
- 在 App::build 中同步加载资产
- 在 AssetLoader 中执行游戏逻辑

优先检查：
- 是否正确使用 Handle<T> 响应式管线
- 是否有同步加载的代码
- AssetLoader 是否只负责反序列化