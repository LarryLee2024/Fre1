# 共享层领域

Version: 1.0
Status: Proposed

共享层管理所有模块复用的基础类型、标识、事件、标签、审计和测试工具，是依赖图的叶子节点。

核心原则：
- 零外部依赖：只依赖标准库和选定的第三方 crate，不依赖任何项目层
- 无业务逻辑：只提供类型、ID、事件定义和工具，不包含游戏规则
- 每模块单一职责：按功能拆分，禁止创建万能垃圾桶目录

---

# 术语定义

## Shared Layer（共享层）

依赖图的叶子节点，提供所有模块都能复用的基础工具。位于 `src/shared/` 目录。

不是业务逻辑层。不是工具文件夹。不是垃圾桶。

关键属性：
- 不依赖任何其他项目层（只依赖标准库和选定 crate）
- 其他所有层都可以依赖共享层
- 每个模块有明确单一的职责定义
- 新增模块必须通过三问准入检查

---

## Strong ID（强类型 ID）

用 newtype 模式包装的类型安全标识，为每个业务实体提供独立的 ID 类型。

不是裸 Entity。不是 String。不是原始整数。

关键属性：
- 使用 newtype 包装 String 内部存储
- 实现 Display、Debug、Hash、Eq、Clone
- 实现 From<str> 和 From<String> 方便创建
- 编译期防止不同实体类型的 ID 混用
- Display 格式为 `TypeName(inner_value)`

必须实现的 Trait 清单：

| Trait | 用途 | 强制等级 |
|-------|------|----------|
| Debug | 调试输出 | 必须 |
| Clone | 值传递 | 必须 |
| Copy | 仅当内部为 Copy 类型时 | 优先（String 不适用） |
| PartialEq | 相等比较 | 必须 |
| Eq | 哈希键 | 必须 |
| Hash | HashMap / HashSet 使用 | 必须 |
| Display | 人类可读输出，含类型前缀 | 必须 |
| Serialize | 序列化（审计、回放、存档） | 必须 |
| Deserialize | 反序列化 | 必须 |
| FromStr | 字符串解析 | 推荐 |
| Default | 仅当有明确空值语义时 | 按需 |

完整 ID 命名空间：

核心 ID 类型（8 种）：

| ID 类型 | 用途 |
|---------|------|
| UnitId | 战场上的战斗单位 |
| SkillId | 技能定义 |
| BuffId | Buff/Debuff 效果 |
| ItemId | 游戏物品 |
| EquipmentId | 装备定义 |
| QuestId | 任务定义 |
| StageId | 关卡/地图 |
| FactionId | 阵营 |

按需扩展 ID 类型（5 种，触发条件时创建）：

| ID 类型 | 用途 | 触发条件 |
|---------|------|----------|
| AiBehaviorId | AI 行为配置 | 新增 AI 策略模板时 |
| TerrainId | 地形类型 | 地形系统独立化时 |
| DialogueId | 对话配置 | 对话系统实现时 |
| ModifierRuleId | 修饰规则 | 规则引擎扩展时 |
| TraitId | Trait 定义 | Trait 配置化时 |

---

## ID分配策略（ID Allocation Strategy）

ID 值的选择策略，决定使用语义字符串还是单调递增编号还是 UUID。

不是 ID 本身。不是创建方法。不是 ID 格式。

关键属性：
- 语义字符串（推荐）：内容文件使用，人类可读、MOD 友好、可预测
- 单调递增 u64（替代）：运行时生成使用，分配快、天然唯一
- UUID（不推荐）：分布式场景使用，不可读、存储开销大
- 内容 ID（Skill/Buff/Item）使用语义字符串
- 运行时生成 ID（Unit）使用单调递增

---

## ID生命周期（ID Lifecycle）

ID 从创建到销毁的完整不可变周期。

不是 ID 格式。不是 ID 存储。不是 ID 分配方法。

关键属性：
- 创建：内容加载时从 RON 文件读取、运行时系统分配、MOD 加载器分配
- 使用：ECS Component 存储、Message 携带、Registry 为键、配置文件引用、存档序列化
- 不可变：创建后禁止修改，内部字段不可写
- 销毁不复用：实体删除时 ID 不释放，永不分配给新实体
- 已删除的 ID 可在日志/审计/存档中继续引用（作为历史记录）

---

## GameplayTag（游戏标签）

基于 u64 位掩码的标签系统，支持 O(1) 查询，用于分类和过滤游戏对象。

不是字符串标签。不是枚举匹配。不是文件路径。

关键属性：
- 内部存储为 u64 位掩码
- 每个标签占用一个 bit 位
- 支持 has/add/remove/has_any/has_all 操作
- 作为 Bevy Component 存储在实体上（GameplayTags）
- 运行时性能优先于可读性

---

## Shared Event（共享事件）

定义在共享层、被多个 Core 模块使用的事件类型。用于跨模块协调通信。

不是模块内部事件。不是 Bevy 命令。不是函数调用。

关键属性：
- 定义在 `shared/events/` 目录
- 被两个或以上 Core 模块引用
- 通过 Bevy Message 系统广播
- 只携带事件数据，不包含处理逻辑
- 使用 serde 支持序列化（用于审计和回放）

---

## DomainEvent（领域事件）

统一的跨模块通信载体，描述"已经发生的事实"。定义在 `shared/events/domain_event.rs` 中，是所有跨模块事件的类型目录。

不是命令或请求。不是模块内部事件。不是处理逻辑。

关键属性：
- 是已发生事实的记录，不是"请做什么"的请求
- 只描述事实，不含处理逻辑
- 携带接收方所需的完整上下文，禁止接收方反向查询发送方
- 事件类型通过 EventWhitelist 审计白名单控制记录
- 跨 Feature 广播使用领域事件，同模块内直接函数调用

完整事件目录（跨模块事件在此注册）：
- 战斗事件：SkillCasted、DamageDealt、HealApplied、BuffApplied、BuffRemoved、CharacterDied
- 回合事件：TurnStarted、TurnEnded、ActionPhaseStarted
- 移动事件：UnitMoved
- 装备事件：ItemEquipped、ItemUnequipped
- 物品事件：ItemUsed
- 阶段转换事件：BattleInitialized、BattleEnded

事件结构规范：每个事件必须包含 source/caster（发起者）、target/targets（目标）、event-specific payload（具体数据）。详细字段参见 `events_audit_design.md`。

---

## Shared Trait（共享特征）

定义在共享层、被多个模块实现的行为契约（Rust trait）。

不是具体实现。不是 Bevy Component。不是枚举。

关键属性：
- 定义在 `shared/traits/` 目录
- 只定义方法签名，不包含实现
- 被两个或以上模块实现
- 用于模块间的多态分发
- 不依赖任何业务类型

---

## Audit（审计）

结构化日志和测试基础设施，用于追踪游戏状态变化。包括审计轨迹收集和事件白名单管理。

不是游戏逻辑。不是调试 UI。不是用户日志。

关键属性：
- 审计轨迹（AuditTrail）是 Bevy Resource，收集所有审计事件
- 事件白名单（EventWhitelist）管理允许记录的事件类型
- 业务代码触发事件 → 审计收集 → 下游消费（回放/日志/成就）
- 审计不影响业务逻辑执行
- 新增事件必须先更新白名单

EventWhitelist 审计白名单详细规则：
- 白名单精确控制哪些事件类型被记录，避免不必要的性能开销
- 默认包含所有核心事件（SkillCasted、DamageDealt、HealApplied 等）
- 新增事件类型必须先调用 register() 添加到白名单
- 未在白名单注册的事件不被审计记录
- 审计系统通过 Bevy Message 监听事件，不侵入业务代码
- audit feature 禁用时审计代码完全不编译，零运行时开销
- 详细事件目录参见 `events_audit_design.md`

---

## Test Utility（测试工具）

辅助构造测试断言、运行确定性测试的工具函数。位于 `shared/testing/` 目录。

不是测试用例本身。不是生产代码。不是测试框架。

关键属性：
- 只提供辅助函数，不包含测试逻辑
- 支持确定性测试（固定随机种子）
- 提供测试环境构建工具
- 不包含任何业务断言
- 只在测试配置下编译

---

## TagName（标签名）

GameplayTag 的字符串表示，用于 RON 定义文件中的序列化/反序列化。

不是 GameplayTag 本身。不是运行时标识。不是字符串常量。

关键属性：
- 使用枚举类型，每个变体对应一个 GameplayTag
- 用于 RON 文件反序列化（Definition 阶段）
- 通过 to_tag() 方法转换为 GameplayTag
- 使用 serde rename_all = "SCREAMING_SNAKE_CASE"
- 运行时代码必须使用 GameplayTag，不使用 TagName

---

# 领域边界

## 本领域负责

- 强类型 ID 的定义和注册（UnitId、SkillId、BuffId、ItemId 等）
- GameplayTag 位掩码系统和 TagName 枚举
- 跨模块共享事件类型定义
- 跨模块共享 trait 定义
- 审计轨迹基础设施（AuditTrail、EventWhitelist）
- 测试辅助工具
- GameResult<T> 类型别名和错误转换 trait
- 确定性随机数
- 数学工具（距离计算等）
- 时间工具
- 通用集合类型
- 校验工具
- 全局常量
- 过程宏

## 本领域不负责

- 具体游戏规则逻辑（由各业务领域负责）
- 具体错误枚举定义（由各业务领域的 domain/ 子目录负责）
- 具体事件处理逻辑（由各业务模块的 Observer 负责）
- Bevy 资源加载（由 Infrastructure 的 AssetServer 负责）
- UI 渲染和输入处理（由 UI 层负责）
- 配置文件加载（由 Content 层负责）
- 存档保存和加载（由 Infrastructure 的 Persistence 负责）
- 审计事件的具体消费逻辑（由下游模块负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 共享事件定义 | Message 类型 | Core 层各模块 |
| 共享 trait 契约 | Rust trait | Core 层各模块 |
| 强类型 ID | 类型传递 | 所有层 |
| 审计事件收集 | Resource 读取 | Infrastructure 审计模块 |
| 测试工具 | 函数调用 | tests/ 目录 |

---

# 生命周期

本领域无状态机，为纯工具层。

共享层提供的是类型定义和工具函数，不参与游戏状态转换。具体的状态管理由使用这些工具的业务层负责。

---

# 不变量

## 不变量1：零外部层依赖

任意时刻：

`shared/` 模块的 `use` 语句只允许同层 `shared/` 内部模块和标准库。禁止使用 `crate::core::`、`crate::infrastructure::`、`crate::content::` 等。

违反表现：

`shared/` 中出现 `use crate::core::id::UnitId`、`use crate::infrastructure::audit::AuditTrail`。

---

## 不变量2：Strong ID 独立性

任意时刻：

每个 Strong ID 类型（UnitId、SkillId、BuffId、ItemId）是独立的 newtype，不同类型之间不能隐式转换。

违反表现：

`UnitId` 可以隐式转换为 `SkillId`。两个不同 ID 类型可以直接比较相等。

---

## 不变量3：GameplayTag 位唯一性

任意时刻：

每个 GameplayTag 常量占用唯一的 bit 位，不允许两个标签共享同一个 bit。

违反表现：

`GameplayTag::FIRE` 和 `GameplayTag::ICE` 使用相同的位值。`GameplayTag` 的两个不同常量进行 AND 运算结果非零。

---

## 不变量4：TagName ↔ GameplayTag 双射

任意时刻：

每个 TagName 枚举变体必须有且仅有一个对应的 GameplayTag 常量。每个 GameplayTag 常量必须有且仅有一个对应的 TagName 枚举变体。

违反表现：

`TagName::Fire.to_tag()` 返回 `GameplayTag::FIRE` 以外的值。某个 GameplayTag 没有对应的 TagName 变体。

---

## 不变量5：共享事件不含业务逻辑

任意时刻：

`shared/events/` 中的事件类型只携带数据字段，不包含方法实现（除 Display/Debug 等标准 trait）。

违反表现：

`shared/events/` 中的事件类型包含 `fn execute(&self)` 等业务方法。

---

## 不变量6：共享 trait 不依赖业务类型

任意时刻：

`shared/traits/` 中的 trait 定义只使用标准库类型和共享层类型作为参数或返回值。禁止使用 Core 层的业务类型。

违反表现：

`shared/traits/` 中出现 `trait DamageSource { fn damage(&self, unit: &UnitId) -> i32; }`（UnitId 是共享层类型，此例合法）。但如果 trait 参数使用 `Unit`（Core 业务组件），则违反。

---

# 规则

## 规则1：共享层零依赖

允许：
- 使用标准库类型（String、Vec、HashMap、Result 等）
- 使用选定的第三方 crate（serde、thiserror、bevy 仅用于 Component/Resource derive）
- 引用同层 `shared/` 内部其他模块

禁止：
- 引用 `crate::core::` 下任何模块
- 引用 `crate::infrastructure::` 下任何模块
- 引用 `crate::content::` 下任何模块
- 引用 `crate::ui::` 下任何模块
- 引用 `crate::app::` 下任何模块

必须：
- 发现共享层引用其他层时立即停止并输出 ARCHITECTURE VIOLATION

---

## 规则2：Strong ID 创建规范

允许：
- 为每个业务实体类型创建独立的 ID newtype
- 使用 String 作为内部存储（便于日志和调试）
- 实现 Display 格式为 `TypeName(inner_value)`
- 实现 From<str> 和 From<String> 方便创建

禁止：
- 使用裸 String 作为业务标识
- 使用裸 Entity 作为业务标识跨模块传递
- 不同 ID 类型之间隐式转换
- 在 ID 中存储业务逻辑

必须：
- 每个 ID 实现 Debug、Clone、PartialEq、Eq、Hash、Display
- 每个 ID 有 `new(id: impl Into<String>) -> Self` 构造方法
- 每个 ID 的 Display 格式包含类型前缀（如 `Unit(warrior_001)`）

---

## 规则3：GameplayTag 位分配

允许：
- 按类别分配连续的 bit 位范围（元素 0-7、状态 8-15、武器 16-23 等）
- 使用 `1 << N` 常量定义标签
- 添加新的标签类别时扩展 bit 范围

禁止：
- 两个不同标签共享同一个 bit 位
- 使用运行时计算的 bit 位（必须是编译时常量）
- 在 RON 文件中使用位掩码值（RON 只使用 TagName 字符串）
- 运行时代码使用字符串查询标签

必须：
- 新增标签时在 GameplayTag 中定义常量
- 新增标签时在 TagName 枚举中添加变体
- 新增标签时在 TagName::to_tag() 中添加映射
- 新增标签时在 GameplayTag::label() 中添加中文名

---

## 规则4：共享事件只用于跨模块

允许：
- 定义被两个或以上 Core 模块引用的事件类型
- 事件只携带数据字段
- 事件通过 Bevy Message 系统广播

禁止：
- 在共享事件中包含处理逻辑
- 用共享事件替代模块内函数调用
- 定义只被单一模块使用的事件（应放在该模块内部）
- 在共享事件中引用 Core 层的业务类型（使用共享层 ID 替代）

必须：
- 共享事件只定义在 `shared/events/` 目录
- 新增共享事件前确认至少有两个模块会使用

---

## 规则5：垃圾桶目录零容忍

允许：
- 按功能拆分的专用模块（如 `shared/ids/`、`shared/error/`、`shared/testing/`）
- 每个模块有明确单一的职责描述

禁止：
- 创建 `utils.rs`、`helpers.rs`、`common/` 目录
- 在 `shared/` 中放置业务相关工具（如 `skill_utils`、`battle_utils`）
- 任何职责描述为"各种工具"的模块

必须：
- 新增 `shared/` 模块时回答三个准入问题：
  1. 所有模块都有用吗？
  2. 不包含任何业务逻辑吗？
  3. 不依赖任何业务类型吗？
  只有三个"是"才能加入

---

## 规则6：ID 分配规则

允许：
- 内容文件（RON）使用语义字符串 ID（如 `"fireball"`、`"iron_sword"`）
- 运行时生成使用单调递增 u64（如 `"unit_000001"`）
- MOD 提供的 ID 加 MOD 前缀隔离

禁止：
- 运行时生成 ID 使用语义字符串（语义字符串用于内容，不用于运行时）
- 内容文件使用单调递增编号（内容 ID 必须可读可预测）
- 使用 UUID（本项目不推荐，不可读、存储开销大）

必须：
- 内容 ID（Skill/Buff/Item 等）使用语义字符串，从 RON 文件的 `id` 字段读取
- 运行时 ID（UnitId）使用单调递增，由 IdAllocator 分配
- 内容加载时校验所有 ID 引用的完整性（存在/缺失/必需缺失三级处理）

---

## 规则7：ID 不可变规则

允许：
- 创建 ID 时通过 new()、From<str>、From<String> 构造
- 在日志、审计、存档中引用已删除实体的 ID（作为历史记录）

禁止：
- 修改已创建的 ID 值（内部字段不可写）
- 将已删除实体的 ID 分配给新实体（销毁不复用）
- 通过任何方式修改 ID 的内部字符串

必须：
- ID 创建后保持不可变直到程序结束
- 实体删除时 ID 不释放

---

## 规则8：ID 到 Entity 映射规则

允许：
- 通过 UnitEntity 组件（包含 UnitId）建立 Strong ID 与 Entity 的关联
- 通过 Query<(Entity, &UnitEntity)> 查询查找对应 Entity
- Message 中携带 Strong ID 而非裸 Entity

禁止：
- 跨模块传递裸 Entity 作为业务标识
- 在 Shared 层之外定义 Strong ID
- 在 `shared/` 之外重复定义同一 ID 类型

必须：
- Strong ID 与 Entity 的映射由 UnitEntity 等组件维护
- 跨模块传递只使用 Strong ID，不使用裸 Entity
- ID 类型定义在 `shared/ids/` 目录，统一管理

---

# 管线

## 管线1：新增共享模块准入管线

```
提出需求 → 回答三问 → 实现 → 审查
```

### Step1：提出需求

输入：新增共享模块的功能描述
处理：明确模块的职责和必要性
输出：候选模块描述
禁止：不经判断直接在 shared/ 中创建文件

### Step2：回答三问

输入：候选模块描述
处理：逐一回答三问准入问题
输出：准入判定（通过/拒绝）
禁止：跳过任何问题、对包含业务逻辑的模块回答"是"

### Step3：实现

输入：准入通过的模块描述
处理：按规范实现模块，确保零外部依赖
输出：可编译的共享模块代码
禁止：在实现过程中引入对 Core/Infrastructure 等层的依赖

### Step4：审查

输入：实现完成的模块代码
处理：检查依赖规则、职责单一性、命名规范
输出：审查通过/拒绝
禁止：不经过审查直接合并

---

## 管线2：Strong ID 创建管线

```
识别实体类型 → 创建 newtype → 实现 trait → 注册到 ID 模块
```

### Step1：识别实体类型

输入：需要标识的业务实体描述
处理：确认实体类型名称和用途
输出：实体类型名称（如 Unit、Skill、Buff）
禁止：为非实体概念创建 ID（如事件类型、状态枚举）

### Step2：创建 newtype

输入：实体类型名称
处理：创建 `TypeName(pub String)` 结构体
输出：新的 ID 类型定义
禁止：使用非 String 的内部存储、使用 pub 字段以外的可见性

### Step3：实现 trait

输入：新的 ID 类型
处理：实现 Debug、Clone、PartialEq、Eq、Hash、Display、From
输出：完整的 ID 类型实现
禁止：省略任何必须实现的 trait、Display 格式不包含类型前缀

### Step4：注册到 ID 模块

输入：完成的 ID 类型
处理：在 `shared/ids/mod.rs` 中添加 pub use 导出
输出：全局可用的 ID 类型
禁止：不注册就使用、在多个模块中重复定义同一 ID 类型

---

## 管线3：GameplayTag 注册管线

```
定义 TagName 变体 → 分配 bit 位 → 添加 GameplayTag 常量 → 添加映射
```

### Step1：定义 TagName 变体

输入：新标签的名称和用途
处理：在 TagName 枚举中添加新变体
输出：新的 TagName 变体
禁止：添加与现有变体语义重复的变体

### Step2：分配 bit 位

输入：新标签名称
处理：在对应类别的 bit 范围内分配未使用的位
输出：bit 位编号
禁止：分配已被占用的 bit 位、使用运行时计算的位

### Step3：添加 GameplayTag 常量

输入：bit 位编号
处理：在 GameplayTag 中添加 `pub const NAME: Self = Self(1 << N)`
输出：新的 GameplayTag 常量
禁止：使用与现有常量相同的位值

### Step4：添加映射

输入：TagName 变体和 GameplayTag 常量
处理：在 TagName::to_tag() 和 GameplayTag::label() 中添加映射
输出：完整的标签定义
禁止：只添加一端映射而遗漏另一端

---

## 管线4：ID 解析管线

```
字符串引用 → Registry 查询 → 存在? 返回ID / 缺失? warn+跳过 / 必需缺失? 加载失败
```

### Step1：字符串引用

输入：RON 文件中的字符串 ID 引用（如 `buff_effects: ["burning", "slow"]`）
处理：Content 层加载 RON 文件，提取字符串引用
输出：待解析的字符串 ID 列表
禁止：直接使用未解析的字符串 ID 作为运行时标识

### Step2：Registry 查询

输入：字符串 ID 列表
处理：在对应 Registry 中查找每个字符串 ID 是否存在
输出：查询结果（存在/不存在）
禁止：跳过 Registry 查询直接使用字符串 ID

### Step3：引用校验

输入：Registry 查询结果
处理：
- 存在 → 返回 Strong ID（解析成功）
- 缺失（可选引用）→ `warn!` 日志 + 跳过该引用或使用默认值
- 必需缺失（必填引用）→ 加载失败，报告错误
输出：解析后的 Strong ID 或错误
禁止：必需引用缺失时继续加载（必须失败）

---

# 数据结构

## UnitId（单位标识）

职责：战场上每个战斗单位的唯一标识

结构：
- 内部值：String — 单位的可读标识（如 "warrior_001"）

要求：
- Display 格式为 `Unit(value)`
- 实现 Hash 和 Eq 以支持 HashSet/HashMap
- 实现 From<str> 和 From<String> 方便创建

---

## SkillId（技能标识）

职责：技能的唯一标识

结构：
- 内部值：String — 技能的可读标识（如 "fireball"）

要求：
- Display 格式为 `Skill(value)`
- 与 UnitId 类型安全隔离

---

## BuffId（Buff 标识）

职责：Buff/Debuff 效果的唯一标识

结构：
- 内部值：String — Buff 的可读标识（如 "poison"）

要求：
- Display 格式为 `Buff(value)`
- 与 UnitId、SkillId 类型安全隔离

---

## ItemId（物品标识）

职责：游戏物品的唯一标识

结构：
- 内部值：String — 物品的可读标识（如 "iron_sword"）

要求：
- Display 格式为 `Item(value)`
- 与 UnitId、SkillId、BuffId 类型安全隔离

---

## GameplayTag（游戏标签）

职责：基于位掩码的标签分类系统

结构：
- 位掩码值：u64 — 每个 bit 代表一个标签

要求：
- 每个常量使用 `1 << N` 格式
- bit 位按类别分组（元素 0-7、状态 8-15 等）
- 提供 label() 方法返回中文名

---

## GameplayTags（标签集合组件）

职责：存储实体上的多个标签，作为 Bevy Component 使用

结构：
- 位掩码值：u64 — 多个标签的组合位掩码

要求：
- 提供 has/add/remove/has_any/has_all 操作
- 支持从标签列表构建（from_tags）
- 支持返回已激活的标签列表（active_tags）

---

## TagName（标签名枚举）

职责：RON 文件中的标签序列化/反序列化

结构：
- 枚举变体：每个变体对应一个 GameplayTag 常量

要求：
- 使用 serde rename_all = "SCREAMING_SNAKE_CASE"
- 每个变体必须有对应的 to_tag() 映射
- 运行时代码禁止使用 TagName，只在 Definition 阶段使用

---

## AuditEvent（审计事件）

职责：记录单个领域事件的完整快照

结构：
- timestamp：u64 — 事件发生时间
- event_type：String — 事件类型名称（必须在白名单中）
- entity：Option<Entity> — 关联实体（可选）
- data：serde_json::Value — 事件的序列化数据
- metadata：AuditMetadata — 事件元数据（回合数、阶段、来源）

要求：
- 使用 serde 支持序列化/反序列化
- event_type 必须在 EventWhitelist 中注册

---

## AuditMetadata（审计元数据）

职责：审计事件的上下文信息

结构：
- turn_number：u32 — 事件发生的回合数
- phase：String — 事件发生的阶段
- source：String — 事件来源标识

要求：
- 提供事件在游戏流程中的定位信息

---

## EventWhitelist（事件白名单）

职责：管理允许记录到审计轨迹的事件类型集合

结构：
- approved：HashSet — 已批准的事件类型名称集合

要求：
- 新增事件必须先调用 register() 添加到白名单
- 提供 check() 方法校验事件是否被批准
- 提供 entries() 方法返回完整清单

---

# 禁止事项

禁止：Shared 层 use 任何其他层的模块

原因：Shared 是依赖图的叶子节点，依赖任何层都会破坏依赖方向，导致循环依赖倾向。

违反后果：编译循环依赖、shared 变成垃圾桶、项目架构崩塌。

---

禁止：在 shared/ 中创建 utils/common/helpers 目录

原因：垃圾桶目录会无限膨胀，混杂无关功能，最终变成所有不知道放哪里的东西的归宿。

违反后果：shared 目录职责模糊、依赖图混乱、新模块归属判定失效。

---

禁止：使用裸 String 或裸 Entity 作为业务标识跨模块传递

原因：裸类型无法在编译期防止传参错误，UnitId 传成 SkillId 不会报错。

违反后果：运行时 ID 混淆导致数据错乱、难以定位的 Bug。

---

禁止：在 GameplayTag 中使用运行时计算的 bit 位

原因：运行时计算的 bit 位无法保证唯一性，可能导致两个标签共享同一个 bit。

违反后果：标签查询错误、位运算结果不符合预期。

---

禁止：运行时代码使用 TagName 而不是 GameplayTag

原因：TagName 是字符串枚举，运行时使用会导致性能下降和不必要的字符串比较。

违反后果：标签查询从 O(1) 退化为 O(N) 字符串匹配。

---

禁止：共享事件包含业务处理逻辑

原因：共享事件只负责传递数据，处理逻辑由各业务模块的 Observer 负责。

违反后果：事件处理逻辑与数据定义耦合、无法独立测试。

---

禁止：共享 trait 使用 Core 层的业务类型作为参数

原因：共享 trait 不依赖任何业务层，使用业务类型会导致共享层依赖 Core 层。

违反后果：依赖方向违反、编译循环依赖。

---

禁止：新增共享模块不经过三问准入检查

原因：三问准入是防止 shared 变成垃圾桶的唯一有效机制。

违反后果：不需要共享的功能混入 shared、职责边界模糊。

---

禁止：在 shared/ 中定义具体错误枚举

原因：具体错误枚举属于各业务领域，shared 只提供错误处理工具（GameResult<T>、错误转换 trait）。

违反后果：shared 成为新的万能错误垃圾桶。

---

禁止：为单个模块创建专用的共享 ID 类型

原因：只被单一模块使用的 ID 类型应该放在该模块内部，不需要共享。

违反后果：shared 中充斥只被一次使用的类型、目录膨胀。

---

# AI 修改规则

## 如果新增 Strong ID 类型

允许：
- 在 `shared/ids/` 中创建新的 newtype 包装
- 为新类型实现所有必须的 trait
- 在 `shared/ids/mod.rs` 中导出新类型

禁止：
- 使用裸 String 或 Entity 替代 newtype
- 省略 Display、Hash、Eq 等 trait 实现
- 在 newtype 中添加业务逻辑方法

优先检查：
- 新 ID 类型是否真的需要在 shared（而不是在业务模块内部）
- Display 格式是否包含类型前缀
- 是否与现有 ID 类型完全隔离

---

## 如果新增 GameplayTag

允许：
- 在 GameplayTag 中添加新的常量
- 在 TagName 枚举中添加新的变体
- 在 TagName::to_tag() 和 GameplayTag::label() 中添加映射

禁止：
- 使用已被占用的 bit 位
- 只添加 GameplayTag 常量而不添加 TagName 变体
- 在 RON 文件中直接使用位掩码值

优先检查：
- bit 位是否唯一（不与现有标签冲突）
- TagName ↔ GameplayTag 映射是否完整
- 新标签是否属于现有类别或需要新类别

---

## 如果新增共享事件

允许：
- 在 `shared/events/` 中定义新的事件结构体
- 使用 serde 支持序列化
- 确认至少有两个模块会使用该事件

禁止：
- 在事件中添加处理逻辑方法
- 定义只被单一模块使用的事件
- 在事件中引用 Core 层的业务组件类型

优先检查：
- 事件是否真的被两个以上模块使用
- 事件数据字段是否使用共享层类型而非业务类型
- 事件命名是否与现有事件一致

---

## 如果新增共享模块

允许：
- 通过三问准入检查后在 shared/ 中创建新模块
- 按功能命名（如 `shared/random/`、`shared/math/`）
- 每个模块有明确单一的职责

禁止：
- 创建 utils/common/helpers 垃圾桶目录
- 不经过三问准入就创建模块
- 模块职责描述为"各种工具"

优先检查：
- 三问准入的三个回答是否都是"是"
- 模块是否零外部依赖（只用标准库和选定 crate）
- 模块命名是否清晰表达职责

---

## 如果测试失败

排查顺序：
1. 检查是否引入了对 Core/Infrastructure 等层的依赖（违反不变量1）
2. 检查 GameplayTag 的 bit 位是否唯一（违反不变量3）
3. 检查 TagName ↔ GameplayTag 映射是否完整（违反不变量4）
4. 检查共享事件是否包含业务逻辑（违反不变量5）
5. 检查共享 trait 是否引用了业务类型（违反不变量6）
