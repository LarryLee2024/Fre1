# MOD 系统领域

Version: 1.0
Status: Proposed

MOD 系统领域管理游戏中 MOD（修改/扩展包）的生命周期、安全边界、API 接口和兼容性检查。

核心原则：
- MOD 只能扩展内容，不能修改规则
- MOD 不能绕过 Effect Pipeline 和 Modifier Stack
- MOD API 是 MOD 作者唯一需要了解的稳定接口

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

# 规则

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