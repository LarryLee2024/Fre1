# 领域事件与审计领域

Version: 1.0
Status: Proposed

领域事件与审计领域管理跨模块通信的事实记录载体和结构化事件记录基础设施，为回放、调试和测试验证提供数据源。

核心原则：
- 事件只描述已发生的事实，不表达意图
- 审计系统不侵入业务逻辑路径
- 事件携带完整上下文，消费者无需反向查询
- 审计关闭时零运行时开销（编译时移除）

---

# 术语定义

## 领域事件（DomainEvent）

业务中已发生的事实记录，是标准化的跨模块通信载体。定义"发生了什么"，由生产者广播、消费者响应。

不是 Command。不是 Request。不是 Message 本身。

关键属性：
- 事件描述已经发生的事实（`SkillCasted` 表示"技能已释放"，不是"请释放技能"）
- 事件通过 Bevy Message 系统广播（参见 `ecs_communication_rules.md`）
- 事件只携带数据字段，不包含处理逻辑
- 事件类型在 `shared/events/domain_event.rs` 中定义
- 每个事件类型在 App 中只注册一次（`add_message::<T>()`）

---

## 事件目录（Event Catalog）

所有 DomainEvent 类型的注册表，定义在 `DomainEvent` 枚举中。不是代码。不是文档。

关键属性：
- 统一管理所有跨模块事件类型
- 每个事件变体包含完整的字段定义
- 新增事件类型必须在此枚举中注册
- 事件类型列表：SkillCasted、DamageDealt、HealApplied、BuffApplied、BuffRemoved、CharacterDied、CharacterRevived、TurnStarted、TurnEnded、UnitMoved、ItemEquipped、ItemUnequipped、ItemUsed、BattleInitialized、BattleEnded

---

## 审计记录（AuditRecord）

单个事件的完整快照，包含序号、帧号、事件数据、状态哈希和元数据。位于 `shared/audit/audit_record.rs`。

不是日志行。不是事件本身。

关键属性：
- 序号（sequence）严格单调递增，用于排序和回放
- 帧号（tick）记录逻辑帧号
- 事件（event）存储 DomainEvent 完整数据
- 状态哈希（state_hash）用于确定性验证
- 元数据（metadata）包含回合数、阶段、来源

---

## 审计轨迹（AuditTrail）

战斗中所有审计记录的时序集合，是 Bevy Resource。位于 `shared/audit/audit_trail.rs`。

不是日志文件。不是存档。

关键属性：
- 按 sequence 排序存储所有 AuditRecord
- 提供按 tick 范围查询的能力
- 支持状态哈希计算用于确定性验证
- 回放系统、调试面板、测试验证器从这里消费数据
- 审计轨迹不影响业务逻辑执行

---

## 事件白名单（EventWhitelist）

控制哪些事件类型被审计记录的集合。位于 `shared/audit/event_whitelist.rs`。

不是全量记录。不是黑名单。

关键属性：
- 使用 HashSet 管理已批准的事件类型名称
- 提供 register() 方法注册新事件类型
- 提供 is_approved() 方法校验事件是否被批准
- 新增事件类型必须先调用 register() 添加到白名单
- 默认包含所有 14 种核心事件类型

---

## 审计消费者（Audit Consumer）

消费审计数据的下游系统，包括回放系统、调试面板、测试验证器、AI 分析。不是事件发送者。

关键属性：
- 回放系统：通过 AuditTrail 读取事件序列，重新执行游戏逻辑
- 调试面板：通过 AuditTrail 展示战斗事件时间线
- 测试验证器：通过 AuditTrail 检查游戏不变量
- AI 分析：通过 AuditTrail 分析战斗模式（伤害分布、Buff 频率）
- 所有消费者只读取 AuditTrail，不修改审计数据

---

# 领域边界

## 本领域负责

- 领域事件类型定义（DomainEvent 枚举，14 种事件类型）
- 事件注册表管理（Event Catalog）
- 审计记录结构（AuditRecord、AuditMetadata）
- 审计轨迹管理（AuditTrail Resource）
- 事件白名单控制（EventWhitelist）
- 审计记录系统（监听事件 → 检查白名单 → 记录到 AuditTrail）
- 事件路由（通过 Bevy Message 系统广播）
- 事件发送方 → 接收方映射表维护

## 本领域不负责

- 事件的具体业务处理逻辑（由各业务模块的 Observer 负责）
- 事件的 UI 展示（由 UI 层的 combat_log、combat_vfx 负责）
- 回放系统的完整实现（由 replay_rules.md 定义的领域负责）
- 调试面板的实现（由 Debug 层负责）
- 具体游戏规则逻辑（由各业务领域负责）
- Bevy Message 系统的底层实现（由 Bevy 引擎负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 领域事件广播 | Message（DomainEvent 各变体） | 所有消费方模块 |
| 审计轨迹数据 | Resource（AuditTrail）只读 | 回放、调试、测试、AI |
| 事件白名单配置 | Resource（EventWhitelist） | 审计记录系统 |
| 事件发送方→接收方映射 | 文档（Event Catalog 表格） | 所有事件生产者和消费者 |

---

# 生命周期

## 事件生命周期

```
生产者 System → MessageWriter<T>.write(event) → EventBus 缓冲 → 按类型分发 → 消费者 System
```

## 审计记录生命周期

```
事件广播 → 白名单检查 → 构造 AuditMetadata → AuditTrail.record() → 消费者读取
```

## 状态列表

### 事件流转状态

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Produced | 生产者发送事件 | Buffered |
| Buffered | EventBus 缓冲中 | Dispatched |
| Dispatched | 已分发给消费者 | Consumed |
| Consumed | 消费者已处理 | 无（终态） |

### 审计记录状态

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Pending | 事件等待白名单检查 | Recorded / Skipped |
| Recorded | 已写入 AuditTrail | 无（终态） |
| Skipped | 未通过白名单检查 | 无（终态） |

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Produced | Buffered | MessageWriter 写入成功 |
| Buffered | Dispatched | 当前帧 Update 阶段末尾 |
| Dispatched | Consumed | 消费者 System 执行完成 |
| Pending | Recorded | EventWhitelist.is_approved() 返回 true |
| Pending | Skipped | EventWhitelist.is_approved() 返回 false |

---

# 不变量

## 不变量1：事件描述已发生的事实

任意时刻：

DomainEvent 的语义必须是"已经发生"的事实记录，不是命令、请求或意图。事件名称使用过去分词或过去时态（如 `SkillCasted`、`DamageDealt`）。

违反表现：

事件类型命名为 `CastSkill`（命令式）或 `RequestDamage`（请求式），语义混淆导致生产者/消费者职责不清。

---

## 不变量2：事件携带完整上下文

任意时刻：

每个 DomainEvent 变体必须包含接收方处理所需的全部信息。禁止接收方反向查询发送方获取缺失数据。

违反表现：

`DamageDealt` 事件不携带 `skill_id`，UI 消费者需要反向查询攻击者才能获取技能信息。

---

## 不变量3：审计系统不改变业务逻辑执行路径

任意时刻：

审计记录系统（`audit_recording_system`）是旁路观察者，不修改任何游戏状态，不阻塞任何业务流程。

违反表现：

审计记录系统中出现 `commands.entity(target).insert(Dead)` 等修改游戏状态的代码。

---

## 不变量4：每个事件类型在 App 中只注册一次

任意时刻：

每个 DomainEvent 变体通过 `app.add_message::<T>()` 注册一次。重复注册会导致消息被多次分发。

违反表现：

`DamageDealt` 被注册两次，UI 消费者收到两条重复的伤害事件，战斗日志显示重复记录。

---

## 不变量5：审计关闭时零运行时开销

任意时刻：

当 `audit` feature 禁用时，审计相关代码通过条件编译完全移除，不产生任何运行时开销。

违反表现：

禁用 `audit` feature 后，`audit_recording_system` 仍然被编译和执行，浪费 CPU 资源。

---

## 不变量6：AuditRecord 序号严格单调递增

任意时刻：

每条 AuditRecord 的 `sequence` 字段必须比前一条大 1。序号用于排序和回放，不可跳号、不可重复。

违反表现：

序号从 5 跳到 7（跳号），或两条记录的序号都是 5（重复），回放系统无法正确排序事件流。

---

# 业务规则

## 规则1：跨模块通信使用 DomainEvent Message

禁止：
- 跨模块通信使用函数直接调用（同模块内允许函数调用）
- 使用裸 Bevy Event 替代 DomainEvent
- 模块内部逻辑使用事件传递（应用函数调用）

必须：
- 跨 Feature 广播的通信使用 DomainEvent Message
- 事件生产者通过 `MessageWriter<T>.write(event)` 发送
- 事件消费者通过 `MessageReader<T>` 读取

允许：
- 同一模块内的逻辑直接函数调用
- 使用 DomainEvent 的子集进行模块内通信

---

## 规则2：事件字段使用 Strong ID

禁止：
- 事件字段使用裸 Entity（`Entity`）
- 事件字段使用裸 String 作为业务标识
- 事件字段使用原始整数作为 ID

必须：
- 事件字段使用 Strong ID 类型（UnitId、SkillId、BuffId、ItemId）
- Strong ID 在 `shared/ids/` 中定义（参见 `shared_layer_rules.md`）
- 事件字段类型与 ID 类型一致

---

## 规则3：审计系统通过 MessageReader 监听

禁止：
- 业务代码直接调用审计记录函数
- 审计系统通过 Query 访问业务组件
- 业务模块依赖审计模块的类型

必须：
- 审计记录系统通过 `MessageReader<T>` 监听所有领域事件
- 审计系统只读取 AuditTrail，不修改业务状态
- 业务代码只发送事件，不操作 AuditTrail

---

## 规则4：新增事件类型必须更新 EventWhitelist

禁止：
- 新增 DomainEvent 变体后不更新白名单
- 使用未注册的事件类型进行审计记录

必须：
- 新增事件类型时调用 `EventWhitelist.register()` 注册
- 白名单包含所有需要审计的事件类型
- 新增事件的 sender→receiver 映射同步更新到事件注册表

---

## 规则5：审计元数据完整性

禁止：
- AuditMetadata 中缺失 turn_number 或 phase
- AuditMetadata 的 source 字段为空字符串
- 审计记录缺少 state_hash

必须：
- 每条 AuditRecord 包含完整的 AuditMetadata
- AuditMetadata 包含 turn_number（当前回合数）
- AuditMetadata 包含 phase（当前阶段名称）
- AuditMetadata 包含 source（事件来源标识）

---

## 规则6：事件类型注册与映射

禁止：
- DomainEvent 变体在 App 中注册多次
- 新增事件类型后不更新 sender→receiver 映射表

必须：
- 每个事件类型在 App 中只注册一次
- 新增事件类型必须在事件注册表中添加 sender→receiver 映射
- 映射表包含发送方模块和所有接收方模块

---

# 流程管线

## 事件生产与分发管线

```
生产者 System → EventBus 缓冲 → 按类型分发 → 消费者 System
```

### 生产者 System

输入：业务逻辑执行结果
处理：构造 DomainEvent 变体，通过 MessageWriter 发送
输出：事件进入 EventBus 缓冲
禁止：在生产者中直接操作 AuditTrail

### EventBus 缓冲

输入：生产者发送的事件
处理：按事件类型缓冲，在当前帧 Update 阶段末尾统一分发
输出：事件分发给所有注册的消费者
禁止：跳过 EventBus 直接调用消费者

### 消费者 System

输入：从 MessageReader 读取的事件
处理：执行业务逻辑（UI 更新、状态变更等）
输出：业务状态变化
禁止：消费者修改事件数据本身

---

## 审计记录管线

```
事件广播 → 白名单检查 → 构造元数据 → 写入 AuditTrail → 消费者读取
```

### 事件广播

输入：领域事件（DomainEvent）
处理：审计记录系统通过 MessageReader 接收
输出：待审计的事件
禁止：业务代码直接推送事件到审计系统

### 白名单检查

输入：事件类型名称
处理：`EventWhitelist.is_approved()` 校验
输出：通过/拒绝
禁止：未通过白名单检查的事件被记录

### 构造元数据

输入：当前游戏状态（回合数、阶段、来源）
处理：构造 AuditMetadata 结构
输出：完整的元数据
禁止：元数据字段缺失（turn_number、phase、source）

### 写入 AuditTrail

输入：AuditRecord（序号、帧号、事件、状态哈希、元数据）
处理：`AuditTrail.record()` 追加记录
输出：AuditRecord 序号递增
禁止：写入后修改已有记录

### 消费者读取

输入：AuditTrail（只读）
处理：回放系统、调试面板、测试验证器、AI 分析
输出：各消费方的处理结果
禁止：消费者修改 AuditTrail 数据

---

# 数据结构

## DomainEvent（领域事件枚举）

职责：统一的领域事件类型目录，所有跨模块事件在此注册

结构：
- SkillCasted — 技能已释放（caster、skill_id、targets）
- DamageDealt — 伤害已施加（source、target、amount、is_critical、skill_id）
- HealApplied — 治疗已施加（source、target、amount、skill_id）
- BuffApplied — Buff 已施加（source、target、buff_id、stacks）
- BuffRemoved — Buff 已移除（target、buff_id、reason）
- CharacterDied — 角色死亡（unit、killer、final_hp）
- CharacterRevived — 角色复活（unit、reviver、revived_hp）
- TurnStarted — 回合开始（turn_number、active_unit）
- TurnEnded — 回合结束（turn_number）
- UnitMoved — 单位移动（unit、from、to、path_length）
- ItemEquipped — 装备穿戴（unit、item_id、slot）
- ItemUnequipped — 装备脱下（unit、item_id、slot）
- ItemUsed — 物品使用（user、item_id、targets）
- BattleInitialized — 战斗初始化完成（stage_id、units）
- BattleEnded — 战斗结束（winner、total_turns）

要求：
- 每个变体只携带数据字段，不包含方法
- 所有 ID 字段使用 Strong ID 类型
- 使用 serde 支持序列化/反序列化
- 语义为"已发生"的事实记录

---

## AuditRecord（审计记录）

职责：单个事件的完整快照，用于回放、调试和测试验证

结构：
- sequence — u64 — 事件序号（单调递增）
- tick — u32 — 游戏 tick 编号（逻辑帧号）
- event — DomainEvent — 领域事件
- state_hash — u64 — 全局状态哈希（确定性验证）
- metadata — AuditMetadata — 审计元数据

要求：
- sequence 严格单调递增，不可跳号、不可重复
- 使用 serde 支持序列化/反序列化
- state_hash 用于回放时验证确定性

---

## AuditMetadata（审计元数据）

职责：审计事件的上下文信息

结构：
- turn_number — u32 — 事件发生的回合数
- phase — String — 事件发生的阶段
- source — String — 事件来源标识

要求：
- 三个字段均不可为空或缺失
- 提供事件在游戏流程中的定位信息

---

## AuditTrail（审计轨迹）

职责：按时间顺序收集所有被审计的领域事件

结构：
- records — Vec — 事件序列（按 sequence 排序）
- next_sequence — u64 — 下一个序号
- current_tick — u32 — 当前 tick 编号

要求：
- Bevy Resource，可被审计记录系统和消费者访问
- 只提供追加写入和只读查询接口
- 支持按 tick 范围查询记录
- 禁止修改已写入的记录

---

## EventWhitelist（事件白名单）

职责：管理允许记录到审计轨迹的事件类型集合

结构：
- approved — HashSet — 已批准的事件类型名称集合

要求：
- 新增事件必须先调用 register() 添加到白名单
- 提供 is_approved() 方法校验事件是否被批准
- 提供 entries() 方法返回完整清单
- 默认包含所有 14 种核心事件类型

---

# 禁止事项

禁止：业务代码直接写入 AuditTrail

原因：审计系统是旁路观察者，业务代码直接写入会破坏审计与业务的解耦，导致审计逻辑侵入业务路径。

违反后果：审计与业务耦合，无法独立测试，审计系统的修改可能引入业务 Bug。

---

禁止：未在白名单注册就记录事件

原因：白名单是审计系统的门控机制，绕过白名单会导致不可控的审计数据量和未预期的性能开销。

违反后果：审计数据量膨胀，内存占用不可预测，回放系统处理超时。

---

禁止：在审计事件中包含随机数

原因：审计事件是确定性回放的数据源，包含随机数会破坏"相同初始条件 + 相同事件流 → 相同结果"的确定性保证。

违反后果：回放结果不可复现，测试验证器无法通过状态哈希比对确认一致性。

---

禁止：审计记录修改游戏状态

原因：审计系统是只读观察者，修改游戏状态会引入非确定性行为，破坏回放系统的正确性。

违反后果：回放时游戏状态与原始执行不一致，调试面板展示的信息与实际不符。

---

禁止：在 audit feature 禁用时仍有审计代码执行

原因：审计系统的零成本设计依赖条件编译移除。禁用时执行审计代码会引入不必要的性能开销。

违反后果：Release 构建的性能被审计系统拖累，零成本设计被破坏。

---

禁止：共享事件中包含处理逻辑

原因：事件只携带数据，处理逻辑由消费者负责。事件包含处理逻辑会导致事件与处理逻辑耦合。

违反后果：事件无法独立测试，修改处理逻辑需要修改事件定义，职责边界模糊。

---

禁止：审计事件使用裸 Entity 而非 Strong ID

原因：裸 Entity 不可序列化，回放系统需要序列化审计事件来持久化存储。Strong ID 支持 serde 序列化。

违反后果：回放系统无法序列化审计事件，审计数据无法持久化，跨会话回放失效。

---

禁止：新增事件类型不更新 EventWhitelist

原因：白名单是审计系统的完整视图，新增事件不注册会导致该事件类型永远不被审计记录。

违反后果：新增的事件类型无法被回放系统追踪，调试面板无法展示该事件，测试验证器无法检查该事件。

---

禁止：AuditRecord 序号跳号或重复

原因：序号是排序和回放的唯一依据，跳号或重复会导致事件流顺序混乱。

违反后果：回放系统无法正确排序事件，状态哈希验证失败，调试面板时间线错乱。

---

禁止：消费者通过反向查询获取事件缺失数据

原因：事件必须携带完整上下文，反向查询会引入生产者与消费者的隐式耦合。

违反后果：生产者重构时消费者可能因反向查询路径变化而失效，事件语义不完整。

---

# AI 修改规则

## 如果新增 DomainEvent 事件类型

允许：
- 在 `DomainEvent` 枚举中添加新变体
- 为新事件添加字段（携带完整上下文）
- 在 `EventWhitelist` 中注册新事件类型

禁止：
- 在事件变体中添加业务方法
- 事件字段使用裸 Entity 或 String（应使用 Strong ID）
- 未在白名单注册就使用新事件
- 事件字段缺失接收方所需的上下文

优先检查：
- 事件是否真的需要跨模块广播（同模块内应直接函数调用）
- 事件字段是否携带接收方所需的完整上下文
- 是否与现有事件类型语义重复
- sender→receiver 映射是否同步更新

---

## 如果修改审计系统

允许：
- 新增审计消费方
- 优化审计记录性能
- 改进状态哈希算法

禁止：
- 审计记录影响业务逻辑路径
- 审计系统修改游戏状态
- 在 audit feature 禁用时引入审计代码
- 修改 AuditRecord 的序号生成逻辑

优先检查：
- 修改是否保持零成本当禁用
- 新增消费方是否正确使用 AuditTrail 只读接口
- 状态哈希算法是否覆盖所有影响确定性的状态

---

## 如果修改事件白名单

允许：
- 新增事件类型到白名单
- 移除不再需要审计的事件类型
- 调整白名单的默认配置

禁止：
- 清空白名单（所有事件都应被审计）
- 将未在 DomainEvent 中定义的类型加入白名单
- 修改白名单的检查逻辑

优先检查：
- 新增的事件类型是否在 DomainEvent 枚举中定义
- 移除的事件类型是否仍有消费者依赖
- 白名单修改是否影响现有审计数据的完整性

---

## 如果测试失败

排查顺序：
1. 检查事件是否携带完整上下文（违反不变量2）
2. 检查审计系统是否修改了游戏状态（违反不变量3）
3. 检查事件类型是否只注册一次（违反不变量4）
4. 检查 audit feature 禁用时是否有审计代码执行（违反不变量5）
5. 检查 AuditRecord 序号是否严格单调递增（违反不变量6）
6. 检查事件字段是否使用 Strong ID（禁止事项7）
7. 检查 EventWhitelist 是否包含所有使用的事件类型
