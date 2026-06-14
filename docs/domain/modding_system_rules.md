# MOD 系统领域

Version: 2.1 [NEW since v2.0]
Status: Proposed

MOD 系统领域管理游戏中 MOD（修改/扩展包）的生命周期、安全边界、API 接口和兼容性检查。

核心原则：
- MOD 只能扩展内容，不能修改规则
- MOD 不能绕过 Effect Pipeline 和 Modifier Stack
- MOD API 是 MOD 作者唯一需要了解的稳定接口

## 宪法合规矩子

| 条款 | 级别 | 落地规则 |
|------|------|----------|
| 1.1.7 只解决当前复杂度 | 🟥 | 🟥 禁止为未来可能出现但未明确的需求提前设计完整 Mod 框架 |
| 1.5.1 复杂度优先于性能 | 🟥 | Mod 系统的架构复杂度预算优先级高于性能优化预算 |
| 1.5.2 禁止为未落地需求预留复杂框架 | 🟥 | 仅允许预留轻量扩展点，禁止提前实现完整的底层能力 |
| 3.0.1 接口最小化 | 🟥 | 每个 Feature 只暴露必要的公共接口，内部实现必须私有 |
| 17.2.1 Mod 支持预留 | 🟥 | 核心系统预留轻量扩展点，不提前实现完整 Mod 框架 |
| 22.10 组合绝对优先 | 🟥 | 所有差异化通过组件、Trait、Modifier 组合实现 |

---

# 术语定义

## MOD

用户或社区创建的游戏内容修改/扩展包。包含新的技能、Buff、装备、角色、关卡等游戏数据。

不是游戏规则引擎。不是 Rust 代码。

关键属性：
- 通过 `manifest.ron` 声明元数据
- 通过 `content/` 目录提供数据
- 通过 MOD API 注册到游戏 Registry
- 可以覆盖基础内容（需声明）

---

## MOD API

MOD 作者与游戏引擎之间的稳定接口层。定义了 MOD 可以做什么和不能做什么。

不是游戏内部 API。不是任意 ECS 操作接口。

关键属性：
- 只暴露安全的内容扩展操作
- 不暴露 ECS World 的直接访问
- 版本化，跨版本兼容
- 位于 `src/modding/api/`

---

## MOD Registry

管理所有已加载 MOD 的注册、依赖和冲突的中心组件。

不是游戏内容的 Registry（SkillRegistry 等）。

关键属性：
- 记录 MOD 的加载顺序
- 解析 MOD 间的依赖关系
- 检测 MOD 间的冲突
- 确定内容合并优先级

---

## MOD 沙箱

MOD 运行时的安全隔离环境，限制 MOD 的操作范围。

不是 Rust 沙箱（不执行任意代码）。

关键属性：
- MOD 只能通过 MOD API 操作
- MOD 不能直接访问 ECS World
- MOD 不能直接操作 Entity
- MOD 不能绕过 Effect Pipeline

---

## 版本化命名空间

MOD 资源的命名空间支持版本号后缀（如 mod_a_v2:fireball），用于 MOD 迭代时区分不同版本的内容。

不是纯字符串。不是 base:前缀。

关键属性：
- 格式：\<namespace\>\_v\<version\>:\<content_id\>
- 同一 MOD 不同版本可并存但不能同时激活
- 版本冲突检测在加载阶段完成
- base: 命名空间专用于基础游戏内容，MOD 禁止使用

---

## AssetId u64 实习化

资源标识符采用 u64 哈希值存储，命名空间+路径在加载时一次性计算为 u64，运行时全部使用 u64 比较。

不是 String。不是 HashMap。

关键属性：
- 启动时或加载时计算一次哈希，后续零拷贝比较
- 热路径（如每帧战斗查询）全部使用 u64 比较
- 保留全局 HashMap\<AssetId, String\> 用于调试和序列化
- 使用非加密快速哈希（如 FxHasher）

---

## 解析路径缓存

Resolved Path Cache，缓存已解析的最终资源路径，避免重复走 Resolution Chain。

不是永久缓存。不是无策略。

关键属性：
- 命中缓存：O(1) 直接返回
- 未命中：走 Resolution Chain，解析后写入缓存
- MOD 启用/禁用时：清空缓存，重建 chain
- 仅缓存已解析路径，不缓存解析过程

---

## MOD Patch/Override

MOD 覆盖基础内容的机制，通过 base/+overrides/ 目录结构和按 ID 深度合并实现。

不是完全覆盖。不是忽略 MOD。

关键属性：
- 目录结构：content/rules/base/（基础配置）+ content/rules/overrides/（覆盖配置）
- 加载顺序：先加载 base，再遍历 overrides 按优先级合并
- 合并策略：按 ID 深度合并，只覆盖声明的字段
- 其他未声明字段保留 base 值

---

## MOD Loaders（MOD 加载器）

负责将 MOD 的数据文件加载到游戏 Registry 的组件。包括基础内容加载和 MOD 内容合并。

不是 MOD API。不是 MOD Registry。不是校验器。

关键属性：
- 内容加载管线：先加载 base 基础内容，再按 priority 顺序加载 MOD 内容
- 合并策略：新增内容直接注册，覆盖内容替换基础内容（需 manifest 声明）
- 资源命名空间：MOD 资源使用 `mod_{mod_id}:` 前缀，基础资源使用 `base:` 前缀
- 卸载时遍历 Registry 中该 MOD 注册的所有资源 ID，从 AssetServer 移除

> **优化来源**: docs/architecture/modding-design.md

---

## 命名空间前缀格式

> **优化来源**: docs/architecture/asset_namespace_design.md

MOD 资源的完整命名空间格式为 `<namespace>:<category>/<name>`。

不是纯字符串。不是文件路径。不是 AssetServer 路径。

关键属性：
- `namespace`：小写字母 + 下划线，最长 32 字符（如 `base`、`official_dlc`、`fire_expansion`）
- `category`：功能分类（如 `skills`、`buffs`、`items`）
- `name`：具体资源名（如 `fireball`、`heal`）
- 分隔符：冒号 `:` 分隔 namespace 与 category，斜杠 `/` 分隔 category 与 name
- 示例：`base:fireball`、`mod_a:skills/fire_storm`、`official_dlc:ice_lance`

---

## 命名空间分配

> **优化来源**: docs/architecture/asset_namespace_design.md

| 命名空间 | 用途 | 说明 |
|----------|------|------|
| `base` | 基础游戏内容 | 基础游戏的所有内容使用此前缀，内建保留名 |
| `official_dlc` | 官方 DLC 内容 | 官方扩展包使用此前缀 |
| `<mod_name>` | MOD 自定义内容 | 每个 MOD 使用其声明的命名空间 |

关键属性：
- `base` 和 `official_dlc` 是保留命名空间，MOD 禁止使用
- MOD 命名空间在 `manifest.ron` 中声明
- 命名空间命名规则：小写字母 + 下划线，禁止冒号、大写字母、数字开头
- 命名空间长度不超过 32 字符

---

## Registry 查找策略

> **优化来源**: docs/architecture/asset_namespace_design.md

Registry（SkillRegistry、BuffRegistry、ItemRegistry 等）的查找按优先级链执行。

不是无序查找。不是单一匹配。不是全局搜索。

关键属性：
1. **精确匹配**：先搜索 `namespace:content_id` 的完整键
2. **上下文回退**：如果未找到，在当前上下文的命名空间中搜索
3. **基础回退**：最后搜索 `base:content_id`
4. 没有冒号 `:` 的 ID 自动添加 `base:` 前缀（向后兼容）
5. 有冒号 `:` 的 ID 按完整路径解析

---

## 向后兼容：无前缀引用

> **优化来源**: docs/architecture/asset_namespace_design.md

当 RON 配置中没有命名空间前缀时，默认假设 `base:` 前缀。

不是可选行为。不是调试功能。不是 MOD 功能。

关键属性：
- 没有冒号 `:` 的 ID 自动添加 `base:` 前缀
- 有冒号 `:` 的 ID 按完整路径解析
- MOD 内容引用必须使用完整命名空间 ID
- 示例：`skill_id: "heal"` 等价于 `skill_id: "base:heal"`

---

## Resolution Chain（解析优先级链）

> **优化来源**: docs/architecture/asset_namespace_design.md

资源查找的完整优先级链定义。

不是简单的文件查找。不是注册表遍历。不是运行时动态解析。

关键属性：
- 查找顺序：User Override > Active Mods（按 priority 排序）> Base Game
- 当 MOD 资源缺失时，自动回退到 base 命名空间
- MOD 作者只需提供"差异化"资源，无需复制整个基础包
- 提供 Debug 诊断工具，输入 AssetId 显示完整 Resolution Chain 命中过程

---

## Manifest 驱动的元数据管理

> **优化来源**: docs/architecture/asset_namespace_design.md

MOD 元数据通过 `manifest.ron` 声明，不依赖文件系统扫描。

不是文件系统扫描。不是运行时发现。不是隐式加载。

关键属性：
- Manifest 声明 MOD 提供的资源列表（无需扫描 content/ 目录）
- Manifest 声明依赖关系（namespace + 版本约束）
- Manifest 声明覆盖权限（`allowed_override_patterns`）
- 加载阶段校验依赖完整性（依赖 MOD 是否已加载、版本约束、循环依赖）
- 优势：加载速度提升一个数量级，为创意工坊验证预留接口

---

## MOD Validators（MOD 校验器）

对 MOD 内容进行三级校验的组件。确保 MOD 内容符合 Schema、引用完整性和游戏规则。

不是加载器。不是 Registry。不是沙箱。

关键属性：
- Level 1 Schema 校验：必填字段检查、类型检查、数值范围检查、标签格式检查
- Level 2 引用完整性校验：effect_ids 必须存在、buff_ids 必须存在、标签必须存在
- Level 3 规则校验：MOD 内容不违反核心游戏规则、不覆盖受保护内容
- 校验失败的内容不注册到 Registry，记录 WARN 日志

> **优化来源**: docs/architecture/modding-design.md

---

## MOD Compatibility（MOD 兼容性）

管理 MOD 与游戏版本、MOD 与 MOD 之间兼容性的组件。

不是加载器。不是校验器。不是沙箱。

关键属性：
- 版本检查在加载前完成，不兼容时给出明确错误信息
- 兼容性矩阵声明：game_version、api_version、compatible_mods、incompatible_mods
- MOD API 版本必须匹配，不兼容的 MOD 导致游戏崩溃
- 静默跳过版本兼容性检查是禁止的

> **优化来源**: docs/architecture/modding-design.md

---

# 领域边界

## 本领域负责

- MOD 的生命周期管理（发现、加载、校验、合并、卸载）
- MOD API 的安全边界定义
- MOD 间的依赖解析和冲突检测
- MOD 内容与基础内容的合并策略
- MOD 兼容性版本检查
- MOD 的安全沙箱

## 本领域不负责

- 具体内容类型的定义（由各业务领域负责）
- 内容加载的具体实现（由 Content Pipeline 领域负责）
- 游戏规则的执行（由各 Core 模块负责）
- 美术资源的加载（由 Infrastructure 的 Assets 模块负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| MOD 注册内容 | MOD API 函数调用 | Content Pipeline, Core 各模块 |
| MOD 冲突 | Message | Debug, UI |
| MOD 加载完成 | Message | App, Debug |
| MOD 兼容性失败 | Message | UI, Debug |

---

# 生命周期

## 状态列表

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Discovered | 扫描到 MOD 清单 | Parsing |
| Parsing | 解析 manifest.ron | Validated, ParseFailed |
| Validated | 清单校验通过 | Resolving |
| ParseFailed | 清单校验失败 | Disabled |
| Resolving | 依赖和冲突解析 | Ordered, ConflictDetected |
| ConflictDetected | 检测到冲突 | Disabled |
| Ordered | 确定加载顺序 | Loading |
| Loading | 加载 MOD 内容 | Loaded, LoadFailed |
| Loaded | 加载成功 | Registering |
| LoadFailed | 加载失败 | Disabled |
| Registering | 注册到游戏 Registry | Active |
| Active | MOD 活跃运行中 | Unloading, HotReloading |
| Unloading | MOD 正在卸载 | Disabled |
| HotReloading | MOD 内容热重载中 | Active, HotReloadFailed |
| Disabled | MOD 被禁用 | Discovered（重新扫描） |

## 状态转换图

```
Discovered → Parsing → Validated → Resolving → Ordered → Loading → Loaded → Registering → Active
                ↓           ↓           ↓            ↓           ↓                          ↓
           ParseFailed  ParseFailed ConflictDetected LoadFailed                        HotReloading
                ↓           ↓           ↓            ↓                                    ↓
              Disabled   Disabled    Disabled     Disabled                          Active / HotReloadFailed
```

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Discovered | Parsing | 扫描到 manifest.ron 文件 |
| Parsing | Validated | manifest.ron 语法正确、必填字段完整 |
| Parsing | ParseFailed | manifest.ron 语法错误或字段缺失 |
| Validated | Resolving | 依赖 MOD 全部找到 |
| Resolving | ConflictDetected | 两个 MOD 声明冲突 |
| Resolving | Ordered | 依赖和优先级排序完成 |
| Ordered | Loading | 内容文件加载成功 |
| Loading | LoadFailed | 内容文件不存在或格式错误 |
| Loaded | Registering | 内容校验通过 |
| Registering | Active | 所有内容注册到 Registry 成功 |

---

# 不变量

## 不变量1：MOD 不能修改游戏规则

任意时刻：

MOD 只能通过 MOD API 注册新内容或覆盖现有内容的数值，不能修改 Rust 规则代码。

违反表现：

MOD 包含 `.rs` 文件或试图修改 Effect Pipeline 逻辑。

---

## 不变量2：MOD 不能绕过 Effect Pipeline

任意时刻：

MOD 添加的技能、Buff、装备必须通过统一的 Effect Pipeline 执行，不能直接扣血或修改属性。

违反表现：

MOD 的技能效果直接修改 HP Component。MOD 的 Buff 直接修改 Attributes。

---

## 不变量3：MOD API 向后兼容

任意时刻：

MOD API 版本升级后，旧版 MOD 仍能在新版游戏中运行（或给出明确的兼容性错误而不是崩溃）。

违反表现：

游戏版本升级后，之前能正常运行的 MOD 无声崩溃。

---

## 不变量4：MOD 内容不能泄漏到基础游戏命名空间

任意时刻：

MOD 内容必须使用 `mod_{mod_id}:` 命名空间前缀，禁止使用 `base:` 命名空间。MOD 内容不能泄漏到基础游戏命名空间。

违反表现：

MOD 使用 `base:skills/fire_storm` 注册技能，覆盖基础游戏内容。

> **优化来源**: docs/architecture/modding-design.md

---

## 不变量5：MOD 必须通过三级校验管线

任意时刻：

MOD 内容必须通过 Schema 校验（Level 1）、引用完整性校验（Level 2）、规则校验（Level 3）三级校验管线，才能注册到 Registry。

违反表现：

MOD 内容跳过校验直接注册，导致运行时引用不存在的 effect_id 或 buff_id。

> **优化来源**: docs/architecture/modding-design.md

---

## 不变量6：MOD 不能破坏核心游戏规则（沙箱原则）

任意时刻：

MOD 运行在沙箱环境中，只能通过 MOD API 操作游戏数据，不能直接访问 ECS World、操作 Entity、绕过 Effect Pipeline。

违反表现：

MOD 直接修改 HP Component、直接操作 Entity、执行任意 Rust 代码。

> **优化来源**: docs/architecture/modding-design.md

---

# 规则

## 规则0：Mod 系统复杂度预算 [宪法 1.1.7 / 1.5.1 / 17.2.1 🟥]

🟥 **Mod 系统必须严格控制复杂度，只解决当前明确的需求。**

必须：
- 🟥 只为当前已明确的扩展需求设计 API 接口
- 🟥 仅允许预留轻量扩展点，禁止提前实现完整的 Mod 框架
- 🟥 公共 API、配置格式、事件体系保持稳定，为未来 Mod 能力铺路
- 🟩 Mod 系统的架构复杂度预算优先级高于性能优化预算
- 🟩 每新增一个抽象层、一个新 Feature，必须证明其收益大于长期维护成本

禁止：
- 🟥 为未来可能出现但未明确的需求提前设计完整架构
- 🟥 过度抽象增加当前维护成本
- 🟥 为"可能的 Mod 需求"提前实现完整的脚本引擎、热更新框架等底层能力
- 🟥 创建无实际价值的 Trait（违反 6.0.3）

允许：
- 为明确规划的扩展点预留轻量接口
- 使用数据驱动方式扩展内容（RON 配置）
- 未来需求明确后再逐步实现完整能力

---

## 规则1：MOD 内容只扩展不修改规则

允许：
- MOD 添加新技能（通过 `register_skill`）
- MOD 添加新 Buff（通过 `register_buff`）
- MOD 添加新装备（通过 `register_equipment`）
- MOD 覆盖现有内容数值（通过 `override_skill` 等，需声明）

禁止：
- MOD 修改 Rust 代码
- MOD 绕过 Effect Pipeline 直接扣血
- MOD 绕过 Modifier Stack 直接修改属性
- MOD 直接操作 ECS Entity
- MOD 访问其他 MOD 的内部数据

必须：
- MOD 通过稳定 API 注册内容
- MOD 覆盖内容需在 manifest 中声明
- MOD 内容通过同样的校验管线

---

## 规则2：MOD 安全边界

允许：
- MOD 通过 `ModApi` 接口操作
- MOD 的内容通过 Registry 注册
- MOD 查询已注册的内容

禁止：
- MOD 直接访问 `World`
- MOD 直接操作 `Commands`
- MOD 直接操作 `Entity`
- MOD 绕过 `ModApi` 接口
- MOD 执行任意 Rust 代码

必须：
- MOD 所有写操作通过 `ModApi`
- MOD 所有读操作通过 `ModQuery`
- MOD 写操作必须通过校验管线

---

## 规则3：MOD 依赖和冲突

允许：
- MOD 声明依赖其他 MOD
- MOD 声明与其他 MOD 冲突
- MOD 声明优先级数值

禁止：
- 循环依赖
- 未声明的覆盖（覆盖必须在 manifest 中声明）

必须：
- 依赖解析使用拓扑排序
- 冲突检测在加载前完成
- 覆盖内容明确声明

---

## 规则4：MOD 兼容性检查

允许：
- MOD 声明游戏版本兼容范围
- MOD 声明 MOD API 版本兼容性
- 不兼容的 MOD 给出明确错误信息

禁止：
- 静默跳过版本兼容性检查
- 不兼容的 MOD 导致游戏崩溃

必须：
- 版本检查在加载前完成
- 不兼容时给出明确错误信息
- MOD API 版本必须匹配

---

## 规则5：MOD Override 权限白名单 [NEW since v2.0]

必须：
- 每个内容类别定义 allowed_override_patterns 白名单
- 视觉类资源（图标/特效/UI 样式）允许覆盖
- 逻辑类资源（战斗公式/属性数值）锁定不可覆盖

禁止：
- MOD 覆盖未在白名单中的内容类别
- MOD 覆盖 base:formulas/* 等逻辑层资源

允许：
- MOD 覆盖 base:skills/* 的图标/特效（视觉替换）
- MOD 追加对话/剧情文本（禁止覆盖本体 Key）

---

## 规则6：Cross-MOD 依赖声明 [NEW since v2.0]

必须：
- manifest.ron 中显式声明 dependencies 字段
- 加载阶段校验所有依赖 MOD 是否已加载
- 依赖 MOD 未加载时，依赖方 MOD 给出明确错误并拒绝加载

禁止：
- 未声明依赖就引用其他 MOD 的命名空间
- 隐式依赖（运行时才发现缺失）

允许：
- 只读查询其他 MOD 已注册的内容（通过 ModQuery）
- 可选依赖（dependencies 中标注 optional: true）

---

## 规则7：命名空间别名 [NEW since v2.0]

必须：
- 别名在 manifest.ron 中声明
- 别名校验唯一性（不同 MOD 不能注册相同别名）
- 别名仅用于配置文件的简写

禁止：
- 两个 MOD 使用相同别名
- 运行时逻辑中使用别名（运行时必须用完整命名空间）

允许：
- fire_expansion → fe: 等简短别名
- 别名通过 ModContext 自动注入

---

## 规则8：MOD 内容迁移降级 [NEW since v2.0]

必须：
- MOD 内容迁移失败时记录 WARN 日志
- 迁移失败跳过该内容，允许游戏继续启动
- 迁移失败的 MOD 内容不注册到 Registry

禁止：
- MOD 内容迁移失败导致游戏崩溃
- 忽略迁移失败（必须记录 WARN）

允许：
- 迁移失败的 MOD 仍可正常启动（跳过失败内容）
- 迁移失败后 MOD 降级为部分功能状态

---

## 规则9：MOD API 稳定性契约

必须：
- MOD API 是 MOD 作者唯一需要了解的稳定接口
- 所有核心规则的扩展点都通过 MOD API 暴露
- MOD API 版本升级后，旧版 MOD 仍能运行（或给出明确兼容性错误）

禁止：
- 暴露 ECS World 的直接访问
- 提供 Entity 操作接口
- 提供直接属性修改接口
- MOD API 破坏向后兼容性

允许：
- 新增安全的内容注册接口（register_skill、register_buff 等）
- 新增只读查询接口（query_skill、query_buff 等）
- 新增 manifest.ron 字段（必须有默认值）

> **优化来源**: docs/architecture/modding-design.md

---

## 规则10：MOD Registry 依赖解析

必须：
- 依赖解析使用拓扑排序确定加载顺序
- 冲突检测在加载前完成
- 循环依赖时所有相关 MOD 无法加载

禁止：
- 存在循环依赖时继续加载
- 忽略 MOD 间的显式冲突声明
- 未声明的覆盖（覆盖必须在 manifest 中声明）

允许：
- MOD 声明依赖其他 MOD（dependencies 字段）
- MOD 声明与其他 MOD 冲突（conflicts 字段）
- MOD 声明优先级数值（priority 字段，数字越大越后加载）

> **优化来源**: docs/architecture/modding-design.md

---

## 规则11：MOD 内容加载管线

必须：
- 先加载基础内容 (content/)，再按 priority 顺序加载 MOD 内容
- 合并策略：新增内容直接注册，覆盖内容替换基础内容（需 manifest 声明）
- 覆盖优先级：高 priority 覆盖低 priority，相同 priority 后加载覆盖先加载

禁止：
- MOD 内容跳过校验管线
- MOD 覆盖未在 manifest 中声明的内容
- 两个 MOD 使用相同命名空间

允许：
- 基础内容 + MOD 内容按优先级合并
- 多 MOD 覆盖同一内容时按 priority 决定
- 覆盖冲突记录审计日志（含 MOD ID、覆盖内容 ID、优先级）

> **优化来源**: docs/architecture/modding-design.md

---

## 规则12：MOD 校验器三级校验

必须：
- Level 1 Schema 校验：必填字段检查、类型检查、数值范围检查、标签格式检查
- Level 2 引用完整性校验：effect_ids 必须存在、buff_ids 必须存在、标签必须存在
- Level 3 规则校验：MOD 内容不违反核心游戏规则、不覆盖受保护内容

禁止：
- 注册校验失败的内容
- 跳过任何一级校验
- 核心内容被覆盖（基础职业定义、元素交互规则、回合状态机定义等）

允许：
- 校验失败的内容记录 WARN 日志
- 校验失败的内容不注册到 Registry
- 校验失败的 MOD 仍可正常启动（跳过失败内容）

> **优化来源**: docs/architecture/modding-design.md

---

## 规则13：MOD 内容隔离

必须：
- MOD 内容使用 `mod_{mod_id}:` 命名空间前缀
- MOD 内容不能泄漏到基础游戏命名空间
- MOD 卸载时遍历 Registry 中该 MOD 注册的所有资源 ID，从 AssetServer 移除

禁止：
- MOD 使用 `base:` 命名空间注册内容
- MOD 内容泄漏到基础游戏命名空间
- MOD 卸载后资源残留

允许：
- MOD 内容在 `mod_{mod_id}:` 命名空间内自由扩展
- MOD 内容通过命名空间前缀防止资源冲突
- MOD 卸载时触发 AssetServer 的资源回收

> **优化来源**: docs/architecture/modding-design.md

---

## 规则14：MOD 性能预算

必须：
- 单 MOD 加载时间 ≤ 50ms（包含解析、校验、注册全流程）
- 总 MOD 加载时间 ≤ 200ms（所有已启用 MOD 的加载总时间）
- MOD 内存占用 ≤ 128MB（所有 MOD 内容的内存总预算）

禁止：
- MOD 加载超过性能预算导致游戏启动缓慢
- MOD 内存占用超过预算导致游戏崩溃

允许：
- 缓存已解析的 manifest，避免重复解析
- 仅校验变更 MOD，未变更的跳过校验
- 美术/音频资源异步加载，不阻塞主线程

> **优化来源**: docs/architecture/modding-design.md

---

## 规则15：MOD 分级权限策略

必须：
- 不同来源的 MOD 拥有不同的权限级别
- 官方 MOD（Level 0）：可扩展战斗逻辑
- 社区 MOD（Level 1）：仅扩展内容数据
- 测试 MOD（Level 2）：开发环境全权限

禁止：
- 社区 MOD 访问调试接口
- 测试 MOD 在生产环境使用
- 未授权的 MOD 执行敏感操作

允许：
- 官方 MOD 通过 `register_custom_rule` 注册自定义战斗规则
- 社区 MOD 通过 `register_skill` / `register_buff` 等基础接口扩展内容
- 测试 MOD 访问 `ModDebugApi` 读取内部状态

> **优化来源**: docs/architecture/modding-design.md

---

## 规则16：MOD 核心内容保护

必须：
- 基础职业定义（如 Warrior、Mage）为受保护内容，任何 MOD 不得覆盖
- 元素交互规则为受保护内容，任何 MOD 不得覆盖
- 回合状态机定义为受保护内容，任何 MOD 不得覆盖
- 胜负条件检查逻辑为受保护内容，任何 MOD 不得覆盖
- 属性计算 Modifier 规则为受保护内容，任何 MOD 不得覆盖

禁止：
- MOD 覆盖受保护内容
- 校验器拦截核心内容被覆盖时返回 `ModError::ProtectedContent` 错误

允许：
- MOD 覆盖视觉类资源（图标/特效/UI 样式）
- MOD 追加对话/剧情文本（禁止覆盖本体 Key）
- 覆盖冲突记录审计日志便于排查

> **优化来源**: docs/architecture/modding-design.md

---

## 规则17：命名空间前缀格式合规 [NEW since v2.2]

> **优化来源**: docs/architecture/asset_namespace_design.md

必须：
- 所有资源引用使用完整的 `namespace:category/name` 格式
- namespace 仅使用小写字母 + 下划线，最长 32 字符
- 内容 ID 不能包含冒号 `:`（与命名空间分隔符冲突）

禁止：
- 命名空间包含冒号 `:`、大写字母、以数字开头
- 命名空间长度超过 32 字符
- MOD 使用 `base` 或 `official_dlc` 作为命名空间
- 内容 ID 包含冒号 `:`

允许：
- MOD 命名空间使用小写字母 + 下划线 + 数字（非首位）：`fire_expansion`、`mod_v2`

---

## 规则18：Registry 查找优先级链 [NEW since v2.2]

> **优化来源**: docs/architecture/asset_namespace_design.md

必须：
- Registry 查找按精确匹配 → 上下文回退 → 基础回退的优先级链执行
- 无冒号的 ID 自动添加 `base:` 前缀（向后兼容）
- 有冒号的 ID 按完整路径解析

禁止：
- 跳过精确匹配直接回退
- MOD 内容引用不使用完整命名空间 ID
- 在 Registry 中绕过查找策略直接访问内部数据

允许：
- 无前缀引用自动解析为 `base:` 前缀（向后兼容）
- MOD 通过 Registry API 只读查询其他 MOD 的内容

---

## 规则19：Resolution Chain 优先级 [NEW since v2.2]

> **优化来源**: docs/architecture/asset_namespace_design.md

必须：
- 资源查找优先级：User Override > Active Mods（按 priority）> Base Game
- MOD 资源缺失时自动回退到 base 命名空间
- MOD 作者只需提供"差异化"资源，无需复制整个基础包

禁止：
- MOD 覆盖 base 命名空间的内容（除非在 `allowed_override_patterns` 中声明）
- 跳过优先级链直接加载资源

允许：
- 提供 Debug 诊断工具显示 AssetId 的完整 Resolution Chain 命中过程
- MOD 启用/禁用时清空解析路径缓存并重建 chain

---

## 规则20：SRPG 特殊覆盖规则 [NEW since v2.2]

> **优化来源**: docs/architecture/asset_namespace_design.md

必须：
- 视觉/表现层（图标/特效/UI 样式）允许 MOD 覆盖
- 逻辑/规则层（战斗公式/属性数值）禁止 MOD 覆盖，仅允许新增
- 对话/剧情文本允许追加，禁止覆盖本体 Key
- 音频/语音允许覆盖，但限制文件大小

禁止：
- MOD 覆盖 `base:formulas/*` 等逻辑层资源
- MOD 覆盖本体对话/剧情 Key
- MOD 塞入巨大音频文件导致内存爆炸

允许：
- MOD 覆盖 `base:skills/*` 的图标/特效（视觉替换）
- MOD 覆盖 `base:ui/icons/*`（UI 图标）
- MOD 追加对话/剧情文本（禁止覆盖本体 Key）
- 覆盖权限通过 Manifest 的 `allowed_override_patterns` 字段强制执行

---

## 规则21：Manifest 驱动元数据管理 [NEW since v2.2]

> **优化来源**: docs/architecture/asset_namespace_design.md

必须：
- MOD 元数据通过 `manifest.ron` 声明，不依赖文件系统扫描
- Manifest 声明资源列表、依赖关系、覆盖权限
- 加载阶段校验依赖完整性（依赖 MOD 是否已加载、版本约束、循环依赖）

禁止：
- 依赖的 MOD 未加载时继续加载依赖方 MOD
- 未声明依赖就引用其他 MOD 的命名空间
- Manifest 声明的资源列表与实际文件不一致

允许：
- Manifest 声明可选依赖（`optional: true`）
- 加载阶段拒绝不完整的依赖链并记录错误

---

# 管线

## MOD 加载管线

```
扫描 mods/ → 解析 manifest → 依赖解析 → 冲突检测 → 排序 → 加载基础内容 → 加载 MOD 内容 → 校验 → 注册 → 游戏
```

### Step1：扫描发现

输入：mods/ 目录
处理：扫描所有子目录，查找 manifest.ron
输出：MOD 清单列表
禁止：加载没有 manifest.ron 的目录

### Step2：解析清单

输入：MOD 清单列表
处理：解析每个 manifest.ron，提取元数据
输出：ModManifest 对象列表
禁止：忽略解析错误继续加载

### Step3：依赖解析

输入：ModManifest 对象列表
处理：拓扑排序，确定加载顺序
输出：有序的 MOD 加载列表
禁止：存在循环依赖时继续加载

### Step4：冲突检测

输入：有序的 MOD 加载列表
处理：检查 MOD 间的冲突声明
输出：冲突报告或通过
禁止：忽略 MOD 间的显式冲突声明

### Step5：内容加载

输入：基础内容 + MOD 内容（按优先级顺序）
处理：AssetServer 加载基础内容，然后按优先级加载 MOD 内容
输出：合并后的 Registry
禁止：MOD 内容跳过校验管线

### Step6：校验注册

输入：合并后的内容
处理：运行三级校验管线（Schema → 引用 → 规则）
输出：校验通过的最终 Registry
禁止：注册校验失败的内容

---

## 三级校验管线

```
Level 1 Schema 校验 → Level 2 引用完整性校验 → Level 3 规则校验
```

### Step1：Level 1 Schema 校验

输入：MOD 内容数据
处理：必填字段检查、类型检查、数值范围检查、标签格式检查
输出：通过 Schema 校验的内容
禁止：跳过 Schema 校验直接注册

### Step2：Level 2 引用完整性校验

输入：通过 Schema 校验的内容
处理：验证 effect_ids 必须存在、buff_ids 必须存在、标签必须存在
输出：通过引用完整性校验的内容
禁止：引用不存在的 effect_id 或 buff_id

### Step3：Level 3 规则校验

输入：通过引用完整性校验的内容
处理：验证 MOD 内容不违反核心游戏规则、不覆盖受保护内容
输出：通过规则校验的最终内容
禁止：MOD 内容违反核心游戏规则

> **优化来源**: docs/architecture/modding-design.md

---

## 内容合并管线

```
基础内容 (base/) → MOD 1 内容 → MOD 2 内容 → ... → 最终 Registry
```

### Step1：加载基础内容

输入：content/rules/base/ 目录
处理：加载基础游戏内容到 Registry
输出：基础内容 Registry
禁止：基础内容加载失败时继续加载 MOD 内容

### Step2：按优先级加载 MOD 内容

输入：基础内容 Registry + MOD 内容（按 priority 排序）
处理：按 priority 顺序加载 MOD 内容，高 priority 覆盖低 priority
输出：合并后的 Registry
禁止：MOD 内容跳过校验管线

### Step3：处理覆盖冲突

输入：合并后的 Registry
处理：两个 MOD 覆盖同一内容时按 priority 决定，记录审计日志
输出：无冲突的最终 Registry
禁止：忽略覆盖冲突不记录审计日志

> **优化来源**: docs/architecture/modding-design.md

---

# 数据结构

## ModManifest（MOD 清单）

职责：描述 MOD 的元数据、依赖和冲突

结构：
- id：String — MOD 唯一标识
- name：String — MOD 显示名称
- version：String — MOD 版本号
- author：String — MOD 作者
- game_version：String — 兼容的游戏版本范围
- dependencies：Vec<String> — 依赖的 MOD ID 列表
- conflicts：Vec<String> — 冲突的 MOD ID 列表
- priority：i32 — 加载优先级（越大越后加载）
- provides：Vec<String> — 提供的内容 ID 列表
- overrides：Vec<String> — 覆盖的内容 ID 列表

要求：
- id 全局唯一
- version 遵循语义版本
- dependencies 和 conflicts 列表完整声明
- 覆盖内容必须在 overrides 中显式声明

---

## ModApi（MOD API 接口）

职责：MOD 作者与游戏引擎之间的稳定接口

结构：
- register_skill：注册新技能
- override_skill：覆盖现有技能
- register_buff：注册新 Buff
- override_buff：覆盖现有 Buff
- register_equipment：注册新装备
- register_character：注册新角色模板
- register_stage：注册新关卡
- query_skill：查询已注册技能
- query_buff：查询已注册 Buff

要求：
- 所有操作通过安全接口，不暴露 ECS World
- 写操作必须通过校验管线
- 读操作只返回不可变引用

---

## ModContext（MOD 上下文）

职责：MOD 操作的上下文环境，限制 MOD 的操作范围

结构：
- mod_id：String — 当前 MOD 的 ID
- registry：RegistryAccessor — 受限的 Registry 访问
- validator：ContentValidator — 内容校验器

要求：
- ModContext 绑定到特定 MOD
- 只允许操作声明的内容范围
- 写操作自动附带 MOD 来源标记

---

# 禁止事项

禁止：MOD 直接访问 ECS World

原因：MOD 通过安全 API 操作，不暴露 World 的内部结构，防止 MOD 破坏游戏数据一致性。

违反后果：MOD 可以绕过所有安全检查，直接修改任何组件，破坏游戏规则。

---

禁止：MOD 绕过 Effect Pipeline 直接扣血

原因：所有战斗效果必须通过 Effect Pipeline 执行，这是项目的铁律。MOD 也不例外。

违反后果：MOD 可以无视战斗规则，直接修改 HP，破坏游戏平衡。

---

禁止：MOD 绕过 Modifier Stack 直接修改属性

原因：所有属性修改必须通过 Modifier 管线，MOD 也不例外。

违反后果：MOD 可以绕过属性修饰规则，直接设置属性值，破坏数值平衡。

---

禁止：MOD 包含 Rust 代码

原因：MOD 只能通过数据驱动扩展游戏内容，不允许执行任意代码。

违反后果：安全风险、崩溃风险、跨平台兼容性问题。

---

禁止：循环依赖

原因：依赖图的拓扑排序要求无环，循环依赖导致无法确定加载顺序。

违反后果：所有相关 MOD 无法加载。

---

禁止：MOD 使用 base: 命名空间注册 [NEW since v2.0]

原因：base: 命名空间专用于基础游戏内容，MOD 使用会导致基础内容被意外覆盖。

违反后果：基础游戏内容被 MOD 覆盖，破坏游戏核心体验。

---

禁止：两个 MOD 使用相同命名空间 [NEW since v2.0]

原因：相同命名空间导致资源路径冲突，无法确定哪个 MOD 的资源被加载。

违反后果：资源加载不确定，运行时表现不可预测。

---

禁止：MOD 间直接引用 [NEW since v2.0]

原因：MOD 间直接引用破坏隔离性，一个 MOD 的变更直接影响另一个 MOD。

违反后果：MOD 间强耦合，任何 MOD 变更都可能破坏其他 MOD。

---

禁止：MOD 内容泄漏到基础游戏命名空间

原因：MOD 内容使用 `base:` 命名空间会导致基础游戏内容被意外覆盖，破坏游戏核心体验。

违反后果：基础游戏内容被 MOD 覆盖，破坏游戏核心体验。

> **优化来源**: docs/architecture/modding-design.md

---

禁止：MOD 覆盖核心受保护内容

原因：基础职业定义、元素交互规则、回合状态机定义、胜负条件检查逻辑、属性计算 Modifier 规则等为核心内容，MOD 覆盖会破坏游戏核心规则。

违反后果：游戏核心规则被破坏，导致游戏崩溃或数值失衡。

> **优化来源**: docs/architecture/modding-design.md

---

禁止：MOD 跳过校验管线

原因：MOD 内容必须通过三级校验（Schema → 引用 → 规则），跳过校验会导致运行时引用不存在的资源或违反游戏规则。

违反后果：MOD 内容引用不存在的 effect_id 或 buff_id，导致运行时崩溃。

> **优化来源**: docs/architecture/modding-design.md

---

禁止：MOD 未声明覆盖

原因：覆盖必须在 manifest.ron 中显式声明，未声明的覆盖会导致其他 MOD 无法检测到冲突。

违反后果：MOD 间冲突无法检测，覆盖行为不可预测。

> **优化来源**: docs/architecture/modding-design.md

---

禁止：MOD 加载超过性能预算

原因：MOD 加载必须满足性能约束（单 MOD ≤ 50ms，总 MOD ≤ 200ms），超过预算会导致游戏启动缓慢。

违反后果：游戏启动时间过长，用户体验下降。

> **优化来源**: docs/architecture/modding-design.md

---

# AI 修改规则

## 如果新增 MOD API 接口

允许：
- 在 `modding/api/` 中新增安全的内容注册接口
- 在 manifest.ron 中新增字段（必须有默认值）

禁止：
- 暴露 ECS World 的直接访问
- 提供 Entity 操作接口
- 提供直接属性修改接口

优先检查：
- 新接口是否只操作内容数据
- 新接口是否通过校验管线
- 新接口的向后兼容性

---

## 如果新增 MOD 内容类型

允许：
- 先在 Core 层实现新规则引擎
- 在 Content 层实现新内容加载器
- 再在 MOD API 中暴露新注册接口

禁止：
- 只在 MOD API 中添加接口而没有规则引擎支持
- 让 MOD 可以注册没有规则引擎支持的内容类型

优先检查：
- Core 层是否有对应的规则引擎
- Content 层是否有对应的内容加载器
- MOD API 的接口参数是否安全

---

## 如果 MOD 加载失败

排查顺序：
1. manifest.ron 语法是否正确
2. 依赖 MOD 是否全部存在
3. 冲突检测是否通过
4. 内容文件是否完整（Schema 校验）
5. 引用完整性是否通过（Level 2 校验）

---

## 如果 MOD 与基础内容冲突

排查顺序：
1. MOD 的 overrides 声明是否完整
2. MOD 的优先级数值是否合理
3. 冲突 MOD 是否在 conflicts 列表中声明
4. 合并策略是否正确（后加载覆盖先加载）

---

## 如果新增 MOD Override 白名单类别 [NEW since v2.0]

允许：
- 在 Manifest Schema 中新增 allowed_override_patterns
- 视觉类资源允许覆盖
- 逻辑类资源锁定不可覆盖

禁止：
- 未在白名单中的类别允许覆盖
- 逻辑层资源被 MOD 覆盖

优先检查：
- 新类别是否在白名单中声明
- 白名单中是否明确标注允许/锁定
- 是否与现有白名单类别冲突

---

## 如果 MOD 内容迁移失败 [NEW since v2.0]

排查顺序：
1. 检查 WARN 日志中的迁移失败原因
2. 确认失败内容是否已跳过（不注册到 Registry）
3. 确认游戏是否正常启动（迁移失败不影响启动）
4. 检查 MOD 是否降级为部分功能状态
5. 修复迁移问题后重新加载

---

## 如果修改 MOD 加载管线 [NEW since v2.1]

允许：
- 调整 MOD 加载管线的步骤顺序
- 新增加载管线步骤（如性能监控、增量校验）
- 优化加载性能（缓存、异步加载）

禁止：
- 删除校验管线步骤（必须保持三级校验）
- 跳过依赖解析或冲突检测
- 修改合并策略导致覆盖行为不可预测

优先检查：
- 加载管线是否保持：扫描 → 解析 → 依赖解析 → 冲突检测 → 排序 → 加载 → 校验 → 注册
- 三级校验是否完整（Level 1 Schema → Level 2 引用 → Level 3 规则）
- 加载性能是否满足预算（单 MOD ≤ 50ms，总 MOD ≤ 200ms）

> **优化来源**: docs/architecture/modding-design.md

---

## 如果新增 MOD 内容类型 [NEW since v2.1]

允许：
- 先在 Core 层实现新规则引擎
- 在 Content 层实现新内容加载器
- 再在 MOD API 中暴露新注册接口

禁止：
- 只在 MOD API 中添加接口而没有规则引擎支持
- 让 MOD 可以注册没有规则引擎支持的内容类型
- 新内容类型跳过校验管线

优先检查：
- Core 层是否有对应的规则引擎
- Content 层是否有对应的内容加载器
- MOD API 的接口参数是否安全
- 新内容类型是否通过三级校验管线

> **优化来源**: docs/architecture/modding-design.md

---

## 如果修改校验规则 [NEW since v2.1]

允许：
- 新增 Level 1 Schema 校验规则（必填字段、类型、数值范围）
- 新增 Level 2 引用完整性校验规则（effect_ids、buff_ids、标签）
- 新增 Level 3 规则校验规则（核心内容保护、受保护内容白名单）

禁止：
- 删除现有校验规则（导致 MOD 内容不安全）
- 放宽校验规则（导致 MOD 内容质量下降）
- 跳过任何一级校验

优先检查：
- 新增的校验规则是否符合三级校验架构
- 校验规则是否覆盖所有已知的 MOD 内容类型
- 校验规则是否与核心游戏规则一致

> **优化来源**: docs/architecture/modding-design.md

---

## 如果修改 MOD 沙箱规则 [NEW since v2.1]

允许：
- 新增沙箱约束（禁止直接访问 ECS World、操作 Entity、绕过 Effect Pipeline）
- 收紧沙箱权限（减少 MOD 可操作的接口）
- 新增 MOD 调试机制（状态查询 API、冲突检测预览工具）

禁止：
- 放宽沙箱权限（允许 MOD 直接访问 ECS World）
- 移除沙箱约束（允许 MOD 执行任意 Rust 代码）
- 让 MOD 绕过 Effect Pipeline 或 Modifier Stack

优先检查：
- 沙箱规则是否覆盖所有安全边界
- MOD API 是否只暴露安全的内容扩展操作
- MOD 调试机制是否便于 MOD 作者排查问题

> **优化来源**: docs/architecture/modding-design.md