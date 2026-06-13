# 七层架构领域

Version: 1.0
Status: Proposed

七层架构领域管理项目源码的分层组织原则、依赖规则和归属判定标准。

核心原则：
- 规则不依赖技术实现
- 基础能力不依赖任何业务
- 内容与逻辑完全分离
- 每层有严格定义的职责边界

---

# 术语定义

## 层

代码组织的基本单元，具有明确的职责边界和依赖方向。

不是模块。不是命名空间。不是功能分组。

关键属性：
- 每层有唯一的职责定义
- 每层有严格的依赖方向
- 每层有明确的禁止事项

---

## 依赖方向

层与层之间允许的 `use` 引用关系。依赖方向决定代码耦合。

不是调用关系。不是数据流方向。

关键属性：
- 依赖方向是单向的
- 逆向依赖绝对禁止
- 同层内依赖通过 Message/Observer/Command 通信

---

## 归属判定

判断一个文件或模块属于哪一层的标准流程。

不是个人偏好。不是历史惯性。

关键属性：
- 使用三问判断法
- 判定结果唯一确定
- 判定结果决定依赖方向和禁止事项

---

## 三问判断法

判定代码归属的三个问题，按顺序逐一检验：

1. **Core 问题**：如果明天把 Bevy 删了，换成 Godot/Unity/UE/服务器模拟器，这个逻辑还存在吗？
2. **Infrastructure 问题**：如果游戏规则不变，能不能换一种实现方式？
3. **Shared 问题**：这个东西既不是游戏规则，也不是技术实现，而是所有模块都会用到的基础工具吗？

三个问题只有一个能回答"是"，答案即归属层。

---

## 垃圾桶目录

看似通用但实际混杂业务逻辑的目录。典型例子：`utils/`、`common/`、`helpers/`。

不是通用工具集。不是合理分类。

关键属性：
- 垃圾桶目录会无限膨胀
- 垃圾桶目录破坏依赖图
- 发现垃圾桶目录必须立即拆分

---

## 插件依赖契约（Plugin Dependency Contract）

插件间显式声明的依赖关系，通过 `add_plugins` 顺序表达。

不是隐式依赖。不是代码注释。不是注解标记。

关键属性：
- 每个 Plugin 必须在 build() 中显式声明依赖的其他 Plugin
- 禁止依赖其他 Plugin 注册的 Resource 但不声明依赖
- 依赖声明决定初始化顺序（后注册的 Plugin 依赖先注册的 Resource）
- 依赖关系构成 DAG（有向无环图），禁止循环依赖
- App 层统一注册时依赖关系一目了然

---

## 插件公共API（Plugin Public API）

插件对外暴露的唯一接口，由 Message + Resource + Component 组成。

不是私有系统。不是内部数据结构。不是 SystemSet。

关键属性：
- 公共 Message：其他 Plugin 可以监听（如 DamageApplied、CharacterDied）
- 公共 Resource：其他 Plugin 可以读取（如 BattleRecord）
- 公共 Component：Entity 上的数据
- System 默认私有，只在本 Plugin 内部执行
- 跨 Plugin 通信只能通过公共 API，禁止调用其他 Plugin 的私有系统

---

## 插件分层禁令（Plugin Layer Ban）

领域 Plugin 禁止依赖 UI Plugin 等跨层依赖规则。

不是配置。不是建议。不是最佳实践。

关键属性：
- Core Plugin 禁止依赖 UI Plugin、Debug Plugin
- Infrastructure Plugin 禁止反向依赖 Core Plugin
- Content Plugin 禁止依赖 UI Plugin
- 违反分层禁令是架构违规，必须立即停止
- 初始化顺序强制：Shared → Infrastructure → Core → Content → UI → Debug → Modding

---

# 领域边界

## 本领域负责

- 定义七层的职责边界
- 定义层间依赖方向
- 定义归属判定标准（三问判断法）
- 定义垃圾桶目录警示
- 定义跨层通信规范
- 定义新增模块的归属流程

## 本领域不负责

- 具体业务规则（由各业务领域负责）
- 具体技术实现（由 Infrastructure 领域负责）
- 具体数据格式（由 Content Pipeline 领域负责）
- 具体错误定义（由 Error System 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 层归属违规 | Architecture Violation | 所有领域 |
| 新增模块归属变更 | ADR | Architecture |
| 依赖方向违规 | Architecture Violation | 所有领域 |

---

# 生命周期

## 状态列表

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Proposed | 新增模块提出归属 | Accepted, Rejected |
| Accepted | 归属判定通过 | Deprecated |
| Deprecated | 归属判定过时 | Migrated |
| Migrated | 迁移完成 | Accepted |

## 状态转换图

```
Proposed → Accepted → Deprecated → Migrated → Accepted
              ↓
           Rejected
```

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Proposed | Accepted | 通过三问判断法且无依赖冲突 |
| Proposed | Rejected | 三问判断法无明确答案或依赖冲突 |
| Accepted | Deprecated | 项目演化导致职责模糊 |
| Deprecated | Migrated | 迁移到新归属完成 |

---

# 不变量

## 不变量1：依赖方向单向性

任意时刻：

依赖图必须是无环有向图（DAG）。

违反表现：

编译循环依赖、模块间互相 `use`。

---

## 不变量2：Core 层零外部依赖

任意时刻：

`core/` 模块的 `use` 语句只允许出现 `shared/` 和同层 `core/` 内部模块。

违反表现：

`core/skill/` 中出现 `use crate::infrastructure::...`。

---

## 不变量3：Shared 层零所有依赖

任意时刻：

`shared/` 模块的 `use` 语句只允许同层 `shared/` 内部模块和标准库。

违反表现：

`shared/` 中出现 `use crate::core::...`、`use crate::infrastructure::...`。

---

## 不变量4：每个文件归属唯一层

任意时刻：

每个 `.rs` 文件只属于七层中的一层。

违反表现：

一个文件同时被 `core/` 和 `infrastructure/` 引用且职责模糊。

---

## 不变量5：垃圾桶目录不出现

任意时刻：

项目中不存在 `utils.rs`、`common/`、`helpers.rs` 等通用垃圾桶。

违反表现：

某个目录下的文件职责描述为"各种工具函数"、"通用辅助"等模糊定位。

---

## 不变量6：每个 Plugin 可独立测试

任意时刻：

每个 Plugin 的声明依赖可 Mock 化，仅 Mock 其声明的依赖即可独立测试。

违反表现：

测试 BattlePlugin 时必须加载整个 UI 层、整个 Infrastructure 层。依赖不可 Mock。

---

## 不变量7：Plugin 间无循环依赖

任意时刻：

Plugin 依赖图必须是 DAG（有向无环图），不允许 A 依赖 B 且 B 依赖 A。

违反表现：

编译时循环依赖错误。两个 Plugin 的 build() 互相调用 add_plugins。

---

## 不变量8：Public API 稳定性

任意时刻：

Plugin 对外暴露的 Message + Resource + Component 新增字段需考虑向后兼容。禁止删除已有公共字段。

违反表现：

删除 Message 的已有字段导致所有消费方编译失败。修改 Resource 结构导致下游 Plugin 数据读取异常。

---

# 规则

## 规则1：三问判断法强制执行

允许：
- 使用三问判断法确定归属
- 在归属模糊时提交 ADR

禁止：
- 不经判断就将模块放入任何层
- 凭感觉将模块放入 `shared/` 或 `common/`

必须：
- 每个新增模块必须在 ADR 或文档中记录其归属判定

---

## 规则2：依赖方向严格限制

允许：
- Core 依赖 Shared
- Infra 依赖 Core 和 Shared
- Content 依赖 Core、Infra、Shared
- Modding 依赖 Core、Shared、Infra、Content
- UI 依赖 ViewModel only
- Debug 依赖 Core（只读）

禁止：
- Core 依赖 Infra
- Core 依赖 Content
- Core 依赖 UI
- Core 依赖 Modding
- Shared 依赖 Core
- Shared 依赖 Infra
- Shared 依赖 UI
- Infra 依赖 UI

必须：
- 发现依赖违规时立即停止并输出 Architecture Violation

---

## 规则3：同层内模块通信

允许：
- Core 内部模块通过 Message 广播通信
- Infrastructure 内部模块直接函数调用通信
- Shared 内部模块直接引用通信

禁止：
- Core 内部模块直接访问其他模块的内部组件
- 用 Observer 模拟函数调用（同模块内应直接调用）

必须：
- Core 内部跨模块通信只通过 Message

---

## 规则4：Content 层职责限制

允许：
- Content 层做三件事：加载、校验、注册
- Content 层调用 Core 的 Registry
- Content 层使用 Infrastructure 的 AssetServer

禁止：
- Content 层包含任何游戏规则逻辑
- Content 层直接创建 Entity
- Content 层直接修改运行时状态

必须：
- Content 层的每个模块只做"加载 RON → 校验 → 注册到 Registry"

---

## 规则5：垃圾桶目录零容忍

允许：
- 按功能拆分的通用工具目录（如 `shared/math/`、`shared/validation/`）
- 每个通用工具目录有明确单一职责

禁止：
- 创建 `utils.rs`、`helpers.rs`、`common/` 目录
- 在 `shared/` 中放置业务相关工具（如 `skill_utils`、`battle_utils`）
- 任何职责描述为"各种××"的目录

必须：
- 新增 `shared/` 模块时回答三个准入问题：
  1. 所有模块都有用吗？
  2. 不包含任何业务逻辑吗？
  3. 不依赖任何业务类型吗？
  只有三个"是"才能加入。

---

## 规则6：Plugin 显式依赖声明

允许：
- 在 Plugin 的 build() 中通过 add_plugins 声明依赖
- 在 App 层统一注册所有 Plugin（推荐，依赖关系一目了然）

禁止：
- 依赖其他 Plugin 注册的 Resource 但不声明依赖（隐式依赖）
- 不声明依赖就使用其他 Plugin 的公共 API

必须：
- 每个 Plugin 显式声明其依赖的所有其他 Plugin
- 发现隐式依赖时立即添加显式声明

---

## 规则7：Plugin Public API 边界

允许：
- Plugin 对外暴露公共 Message、公共 Resource、公共 Component
- Plugin 内部 System 保持私有

禁止：
- 调用其他 Plugin 的私有系统
- 通过 pub use 暴露内部 System 供其他 Plugin 直接执行
- Plugin 内部数据结构对外暴露

必须：
- 跨 Plugin 通信只通过 Message/Observer/Resource
- 新增公共 API 字段考虑向后兼容

---

## 规则8：Plugin 分层禁令

允许：
- Core Plugin 依赖 Shared Plugin、Infrastructure Plugin
- Infrastructure Plugin 依赖 Shared Plugin
- Content Plugin 依赖 Core、Shared、Infrastructure
- UI Plugin 只读 Core 的 ViewModel
- Debug Plugin 只读 Core 的业务数据

禁止：
- Core Plugin 依赖 UI Plugin（领域逻辑不依赖表现层）
- Core Plugin 依赖 Debug Plugin（领域逻辑不依赖调试工具）
- Infrastructure Plugin 反向依赖 Core Plugin（技术实现不依赖业务逻辑）
- Infrastructure Plugin 依赖 UI Plugin
- Content Plugin 依赖 UI Plugin

必须：
- 发现分层禁令违反时立即停止并输出 Architecture Violation

---

## 规则9：Plugin 初始化顺序

允许：
- 按 Shared → Infrastructure → Core → Content → UI → Debug → Modding 顺序注册

禁止：
- UI Plugin 在 Core Plugin 之前注册
- Debug Plugin 在 Core Plugin 之前注册
- 任何违反依赖方向的注册顺序

必须：
- 所有 Plugin 按依赖层次从底层到顶层注册
- App 层的 add_plugins 调用顺序反映依赖关系

---

## 规则10：Plugin 间通信方式

允许：
- 通过 Message（Bevy Message）跨 Plugin 广播事件
- 通过 Resource（Bevy Resource）共享全局状态
- 通过 Observer 响应 Component 变更

禁止：
- 直接调用其他 Plugin 的 System 函数
- 绕过 Plugin 边界在 App 层直接注册其他 Plugin 的 Resource
- Plugin 间直接共享内部状态

必须：
- Plugin 间通信只通过 Message/Observer/Resource
- 发现直接系统调用时重构为 Message 通信

---

## 规则11：循环依赖检测与修复

允许：
- 发现循环依赖时提取公共依赖到 Shared Plugin
- 通过 Message 通信解耦互相依赖的 Plugin

禁止：
- 忽视循环依赖继续开发
- 通过中间层绕过循环依赖检测

必须：
- Plugin 依赖图必须是 DAG
- 发现循环依赖时立即修复：提取公共部分到 Shared，或改为 Message 通信

---

# 管线

## 新增模块归属管线

```
提出归属 → 三问判断 → 依赖检查 → 禁止事项检查 → 归属确认 → 记录 ADR
```

### Step1：提出归属

输入：新增模块的功能描述
处理：提出初步归属层
输出：候选层
禁止：不经过判断直接放入任何层

### Step2：三问判断

输入：候选层
处理：按顺序回答 Core → Infrastructure → Shared 三个问题
输出：确定归属层
禁止：跳过任何问题、对多个问题回答"是"

### Step3：依赖检查

输入：确定归属层
处理：检查新模块的 `use` 语句是否符合层间依赖规则
输出：依赖合规报告
禁止：发现违规依赖时继续推进

### Step4：归属确认

输入：依赖合规报告
处理：确认最终归属
输出：归属记录
禁止：不记录归属就合并代码

---

# 数据结构

## LayerAssign（层归属记录）

职责：记录模块与层的归属关系

结构：
- 模块路径：String — 模块在源码树中的位置
- 归属层：Layer — 七层之一
- 判定依据：String — 三问判断法的回答
- 依赖列表：Vec<Dependency> — 该模块依赖的其他模块

要求：
- 每个模块路径只出现一次
- 判定依据必须引用三问判断法的具体回答
- 依赖列表必须符合层间依赖规则

---

## Dependency（依赖关系）

职责：记录模块间的依赖

结构：
- 源模块：String — 依赖发起方
- 目标模块：String — 依赖目标
- 依赖类型：DependencyType — use/Message/Observer/Command

要求：
- 依赖方向必须符合层间规则
- 同层内 Message 依赖不需要记录
- 跨层依赖必须记录并定期审查

---

# 禁止事项

禁止：Core 模块 use 任何 Infrastructure、UI、Content、Modding 层的模块

原因：Core 是游戏规则的纯领域逻辑层，依赖技术实现或表现层会破坏可移植性和可测试性

违反后果：循环依赖、代码耦合爆炸、无法独立测试

---

禁止：Shared 模块 use 任何其他层的模块

原因：Shared 是依赖图的叶子节点，依赖任何层都会创建循环依赖倾向

违反后果：编译循环依赖、shared 变成垃圾桶

---

禁止：在 shared/ 中放置业务相关工具模块（如 skill_utils、battle_utils）

原因：业务工具应该放在业务领域内部，shared 只放与业务无关的基础工具

违反后果：shared 变成万能垃圾桶、依赖图混乱

---

禁止：Content 层包含任何游戏规则逻辑

原因：Content 只做"加载→校验→注册"三件事，包含规则逻辑违反 Content/Rule 分离

违反后果：内容扩展时需要修改 Rust 代码

---

禁止：App 层包含任何业务逻辑

原因：App 只组装游戏，不执行游戏规则，包含逻辑违反职责单一

违反后果：App 层变成上帝对象、无法测试

---

禁止：无视三问判断法凭感觉放置模块

原因：三问判断法是唯一客观的归属判定标准，凭感觉会导致归属混乱

违反后果：模块归属不一致、依赖方向违规

---

禁止：UI 直接访问 Core 的 ECS 组件

原因：UI 只能通过 ViewModel 读取状态、通过 UiCommand 发送命令，直接访问会破坏 Logic/Presentation 分离

违反后果：UI 和业务逻辑耦合、无法独立替换 UI

---

禁止：Plugin 间隐式依赖

原因：隐式依赖导致 Resource 未就绪时运行 panic，且无法被编译器检测

违反后果：运行时 panic、依赖关系不明确、重构困难

---

禁止：绕过 Plugin 边界注册资源

原因：在 App 层直接注册属于其他 Plugin 的 Resource 破坏 Plugin 封装性

违反后果：Plugin 的 build() 职责模糊、Resource 所有权不明确

---

禁止：Plugin 循环依赖

原因：循环依赖导致编译失败、架构无法维护

违反后果：编译错误、Plugin 无法独立初始化

---

# AI 修改规则

## 如果新增模块

允许：
- 使用三问判断法确定归属层
- 在 ADR 中记录归属判定
- 创建符合层职责的模块

禁止：
- 不经判断直接创建模块
- 创建垃圾桶目录（utils、common、helpers）

优先检查：
- 三问判断法的回答是否明确
- 模块的 `use` 语句是否符合层间规则
- 模块职责是否单一且清晰

---

## 如果迁移模块

允许：
- 按照迁移路线图逐步迁移
- 每步迁移后验证编译通过

禁止：
- 跳过迁移步骤
- 迁移后不更新所有引用

优先检查：
- 目标层的依赖规则是否允许
- 所有引用路径是否更新
- 测试是否通过

---

## 如果发现依赖违规

允许：
- 立即停止并报告 Architecture Violation
- 提出修复建议

禁止：
- 忽视违规继续开发
- 用 `#[allow]` 压制违规警告

优先检查：
- 违规的模块属于哪一层
- 违规的方向是什么（如 Core → Infra）
- 是否有替代方案（如通过 Shared 间接依赖）

---

## 如果测试失败（架构相关）

排查顺序：
1. 检查是否有循环依赖（`cargo check` 依赖图）
2. 检查模块归属是否正确（三问判断法）
3. 检查层间通信是否使用正确方式（Message/Observer/Command）
4. 检查是否有垃圾桶模块需要拆分