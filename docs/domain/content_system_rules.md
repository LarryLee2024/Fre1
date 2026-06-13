# 内容系统领域

Version: 1.0
Status: Proposed

内容系统领域管理游戏内容数据（RON 配置文件）与游戏规则代码（Rust）之间的分离、加载、校验和热重载机制。

核心原则：
- Rule/Content 强制分离：代码是规则，配置是内容
- Definition/Instance 强制分离：配置不可变，运行时可变
- 新增内容只改 RON 文件，禁止修改 Rust 代码

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

## 内容桥接层

加载 RON 配置数据、校验引用完整性、注册到 Registry 的代码层。

不是游戏规则层。不是表现层。

关键属性：
- 只做三件事：加载、校验、注册
- 位于 `src/content/` 目录
- 依赖 Core 和 Infrastructure

---

## RON格式契约（RON Format Contract）

配置文件的统一结构与字段要求，定义所有 RON 文件必须遵循的基础格式。

不是单个文件的格式。不是任意结构。

关键属性：
- 每个 RON 文件必须包含：id、name、description、version、tags + 领域特定字段
- 引用机制：字符串 ID → 加载时 Strong ID 解析 → Registry 校验存在性
- 默认值规则：新增字段必须有默认值保证向后兼容
- MOD 内容格式：ID 前缀隔离、覆盖优先级规则
- 来源：`docs/architecture/content_data_format.md`

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

# 管线

## 内容加载管线

```
RON 文件 → AssetServer 加载 → XxxDef（反序列化） → 校验 → From<XxxDef> for XxxData → Registry.insert()
```

### Step1：AssetServer 加载

输入：`content/skills/fireball.ron`
处理：Bevy AssetServer 读取文件并反序列化为 XxxDef
输出：XxxDef 对象
禁止：在加载阶段执行任何游戏逻辑

### Step2：Schema 校验

输入：XxxDef 对象
处理：检查必填字段、类型正确性
输出：校验通过或失败报告
禁止：在 Schema 校验中检查引用完整性（属于 Level 2）

### Step3：Def → Data 转换

输入：XxxDef 对象
处理：`impl From<XxxDef> for XxxData`，将 TagName 字符串转换为 GameplayTag 位掩码
输出：XxxData 对象
禁止：在转换过程中修改任何游戏逻辑

### Step4：引用完整性校验

输入：所有 XxxData 对象
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

禁止：MOD 配置绕过冲突检测

原因：MOD 内容必须通过 MOD 加载器，跳过检测会导致 ID 冲突。

违反后果：MOD 覆盖基础内容时无日志记录，调试困难。

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