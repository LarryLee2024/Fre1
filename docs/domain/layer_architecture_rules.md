# 七层架构领域

Version: 1.2
Status: Proposed

> **优化来源**: docs/architecture/app-bootstrap.md（App 层哲学、AppState 状态机、Schedule 设计、SystemSet 排序、关闭序列、依赖规则）

七层架构领域管理项目源码的分层组织原则、依赖规则和归属判定标准。

核心原则（对标宪法条款）：
- 🟩 规则不依赖技术实现（宪法 1.3.1 三层架构：Domain > Application > Presentation）
- 🟥 基础能力不依赖任何业务（宪法 1.3.2 依赖方向铁则：绝对禁止反向依赖）
- 🟩 内容与逻辑完全分离（宪法 1.1.3 规则与内容强制分离）
- 🟩 每层有严格定义的职责边界（宪法 1.1.1 Feature First：按业务领域拆分）
- 🟩 复杂度优先于性能（宪法 1.5.1 复杂度预算优先于性能优化预算）
- 🟥 禁止为未落地需求预留复杂框架（宪法 1.5.2）
- 🟩 领域核心逻辑与引擎解耦（宪法 1.4.1 核心领域与引擎解耦）
- 🟩 领域无副作用（宪法 1.4.2 领域纯函数不得直接触发事件）

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

## App（装配器 / Assembler）

main.rs 仅注册 Plugin 的最外层装配入口，零业务逻辑。

不是业务层。不是系统集合。不是配置中心。

关键属性：
- main.rs 只保留一行 `App::new().add_plugins(AppPlugin).run()`
- 不创建 Entity、不修改业务状态、不硬编码数值
- 所有业务逻辑封装在对应层级的 Plugin 中
- 后续功能扩展只通过新增 Plugin 完成

---

## Plugin Group（插件组）

feature-gated 子 Plugin 的聚合体，通过 `PluginGroup` trait 统一管理条件编译的插件集合。

不是单个 Plugin。不是依赖声明。不是配置文件。

关键属性：
- 使用 Bevy 的 `PluginGroup` trait 将多个子 Plugin 聚合为一组
- 条件编译（`#[cfg(feature = "...")]`）在 PluginGroup 内部处理，对外暴露干净接口
- App 层无条件注册 PluginGroup，不在 App 层写 cfg 宏
- 支持按 feature 开关启用/禁用子 Plugin（如移动端禁用高级特效插件）

---

## LoadingState（加载中间态）

MainMenu→Loading→InGame 等重状态切换之间的过渡状态，异步 Asset 加载等待。

不是同步阻塞。不是跳过。不是可选中间态。

关键属性：
- 统一管理加载进度、加载动画，避免状态切换时画面冻结
- 资源加载、数据初始化逻辑放在 LoadingState 的 OnEnter 里
- 失败时兜底回退到主菜单，不直接崩溃
- 支持异步加载，配合 Bevy 的资产句柄做进度追踪
- 状态转换：MainMenu → Loading → InGame / LevelSelect → Loading → InGame

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

## 不变量1：依赖方向单向性（🟥 宪法 1.3.2 依赖方向铁则）

任意时刻：

🟥 依赖图必须是无环有向图（DAG）。绝对禁止反向依赖：领域层不得依赖应用层与表现层，应用层不得依赖表现层。
🟩 允许上层依赖下层：表现层 → 应用层 → 领域层。

违反表现：

编译循环依赖、模块间互相 `use`、领域层 use 了表现层或应用层的模块。

---

## 不变量2：Core 层零外部依赖（🟩 宪法 1.4.1 领域纯度）

任意时刻：

🟥 `core/` 模块的 `use` 语句只允许出现 `shared/` 和同层 `core/` 内部模块。绝对禁止依赖 Infrastructure、UI、Content、Modding 层。

违反表现：

`core/skill/` 中出现 `use crate::infrastructure::...`。

---

## 不变量3：Shared 层零所有依赖（🟩 宪法 3.0.7 通用代码规范）

任意时刻：

🟥 `shared/` 模块的 `use` 语句只允许同层 `shared/` 内部模块和标准库。绝对禁止存放任何业务逻辑。

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

## 不变量9：可替换性原则

任意时刻：

Infrastructure 模块必须可整体替换实现方式，而 Core 层业务逻辑不受任何影响。

核心测试：如果游戏规则不变，能不能换一种实现方式？
- 能 → Infrastructure（可替换的实现）
- 不能 → 不是 Infrastructure

违反表现：

替换存储格式（JSON→二进制）或日志框架（tracing→slog）时，Core 层代码需要修改。

> **优化来源**: docs/architecture/infrastructure-design.md

---

## 不变量10：三棵树物理分离

任意时刻：

项目必须分离为三棵独立的树：src/（源码）、content/（RON 配置）、assets/（二进制资源）。三棵树不仅是逻辑分离，更是物理分离——禁止交叉引用、禁止混放。

违反表现：

RON 配置文件放在 assets/ 中、美术资源放在 src/ 中、Rust 代码放在 content/ 中。

> **优化来源**: docs/architecture/project-structure.md

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

## 规则2：依赖方向严格限制（🟥 宪法 1.3.2 依赖方向铁则）

允许：
- Core 依赖 Shared
- Infra 依赖 Core 和 Shared
- Content 依赖 Core、Infra、Shared
- Modding 依赖 Core、Shared、Infra、Content
- UI 依赖 ViewModel only
- Debug 依赖 Core（只读）
- Tools 依赖 Core、Shared
- App 注册任意层（仅注册，不含逻辑）

禁止：
- 🟥 Core 依赖 Infra（宪法 1.3.2：领域层不得依赖应用层）
- 🟥 Core 依赖 Content
- 🟥 Core 依赖 UI（宪法 1.3.2：领域层不得依赖表现层）
- 🟥 Core 依赖 Modding
- 🟥 Core 依赖 Debug（领域逻辑不依赖调试工具）
- 🟥 Shared 依赖 Core（宪法 3.0.7：通用代码不得依赖业务逻辑）
- 🟥 Shared 依赖 Infra
- 🟥 Shared 依赖 UI
- 🟥 Infra 依赖 UI

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

## 规则5：垃圾桶目录零容忍（🟥 宪法 1.1.1 + 3.0.7）

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
- Tools Plugin 依赖 Core、Shared（仅开发期间，永不发布）

禁止：
- Core Plugin 依赖 UI Plugin（领域逻辑不依赖表现层）
- Core Plugin 依赖 Debug Plugin（领域逻辑不依赖调试工具）
- Core Plugin 依赖 Tools Plugin（领域逻辑不依赖开发工具）
- Infrastructure Plugin 反向依赖 Core Plugin（技术实现不依赖业务逻辑）
- Infrastructure Plugin 依赖 UI Plugin
- Content Plugin 依赖 UI Plugin

必须：
- 发现分层禁令违反时立即停止并输出 Architecture Violation

---

## 规则9：Plugin 初始化顺序

允许：
- 按 Shared → Infrastructure → Core → Content → UI → Debug → Modding → Tools 顺序注册

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

## 规则12：AppPlugin 代码类型限制

允许：
- AppPlugin 的 build() 中仅出现三类代码：`add_plugins`、`init_resource`（全局配置类）、`configure_sets/configure_schedules`
- AppPlugin 注册 PluginGroup

禁止：
- AppPlugin 中出现 `add_systems` 的业务系统
- AppPlugin 中包含任何业务逻辑（如伤害计算、状态判断）
- AppPlugin 中创建 Entity 或修改游戏状态

必须：
- 所有系统必须归属到对应层级的 Plugin 里，哪怕只有一行代码的系统
- App 层保持绝对"空心化"，只做组装不做执行

---

## 规则13：Plugin 显式依赖声明（require_plugins）

允许：
- 在 Plugin 的 build() 中通过 `app.require_plugins(OtherPlugin)` 声明依赖
- Bevy 在启动时检测依赖缺失并给出明确报错

禁止：
- 依赖其他 Plugin 注册的 Resource 但不声明依赖
- 依赖顺序错乱导致运行时 Panic

必须：
- 每个 Plugin 显式声明其依赖的所有其他 Plugin
- 发现隐式依赖时立即添加显式声明

---

## 规则14：.chain() 性能陷阱规避

允许：
- 使用 `after()` / `before()` 约束系统执行顺序，保留并行空间
- 对必须严格串行的逻辑（如效果管线的生成→修饰→执行），使用自定义 Schedule 在特定节点触发

禁止：
- 在普通 Update Schedule 中无脑 `.chain()` 多个系统（强制串行执行，破坏多线程并行优势）
- 将非严格依赖的系统用 `.chain()` 串行化

必须：
- Bevy 调度器在满足依赖关系的前提下自动并行化无冲突系统
- 效果管线的串行逻辑放在专属 Schedule 中，不污染 Update Schedule

---

## 规则15：FixedUpdate SystemSet 分解

允许：
- 在 FixedUpdate Schedule 中划分 `LogicFixedSet`、`PhysicsSet`、`AnimationFixedSet`
- 确定性战斗结算放在 FixedUpdate 里执行

禁止：
- 确定性战斗结算放在 Update 里（帧率波动会影响战斗数值）
- FixedUpdate 中无序执行系统

必须：
- 确定性战斗结算必须放在 FixedUpdate 中，保证固定步长执行
- Update 只处理输入和 UI 表现，不参与确定性结算
- FixedUpdate 中的 SystemSet 同样需要明确执行顺序

---

## 规则16：双层状态机架构

允许：
- `AppState` 作为宏观流程状态机（MainMenu、Loading、InGame、GameOver）
- `TurnPhase` 作为 `AppState::InGame` 的 SubState 控制回合内微观阶段
- 子状态仅在父状态激活时存在

禁止：
- 状态机超过两层嵌套（不在 ActionMenu 里再加子状态）
- OnEnter 中跨阶段跳转
- 用布尔值组合（is_in_game && is_selecting_unit）替代状态机

必须：
- 更细粒度的流程控制用组件标记或资源标志位实现，不嵌套 State
- 状态转换通过 `NextState` 驱动，保证可追溯性
- SubState 仅在对应父状态激活时存在

> **优化来源**: docs/architecture/app-bootstrap.md — AppState 状态机与 TurnPhase SubState 设计

### AppState 状态机详细职责

| 状态 | 职责 | 入场系统（OnEnter） | 退场系统（OnExit） |
|------|------|---------------------|---------------------|
| `MainMenu` | 主菜单：进入游戏的入口 | 初始化 UI、加载菜单资源 | 清理菜单 UI |
| `LevelSelect` | 关卡选择、队伍编成 | 显示关卡列表、队伍编辑器 | 清理选择 UI |
| `InGame` | 核心战斗循环 | 加载地图、生成单位、初始化回合 | 保存战斗结果、清理战场 |
| `GameOver` | 胜利/失败、存档、返回菜单 | 显示结算画面、触发存档 | 清理结算 UI |

### TurnPhase SubState 规则

- 🟥 **仅在 `AppState::InGame` 时激活**
- 🟥 **状态转换必须通过 `NextState<TurnPhase>` 驱动**
- 🟥 **禁止在 OnEnter 中执行跨阶段跳转**
- 🟩 **每个阶段系统必须轻量，重型逻辑拆分到独立系统**
- 🟥 **Bevy 0.15+ 要求显式声明 `#[derive(SubStates)]`**，TurnPhase 必须在 `InGame` 状态下注册

TurnPhase 阶段列表：

```
TurnPhase
├── SelectUnit         # 选择行动单位
├── MoveUnit           # 移动单位
├── ActionMenu         # 动作菜单（攻击/技能/道具/待机）
├── SelectTarget       # 选择目标
├── ExecuteAction      # 执行动作
├── WaitAction         # 等待动画/结算
└── TurnEnd            # 回合结束结算
```

---

## 规则17：LoadingState 过渡态管理

允许：
- 所有重状态切换经过 Loading 过渡态（MainMenu → Loading → InGame）
- 在 Loading 的 OnEnter 中执行资源加载和数据初始化
- 加载失败时回退到主菜单并提示错误

禁止：
- 状态切换时画面冻结（同步加载）
- 跳过 Loading 直接进入 InGame（重状态切换）
- 加载失败时直接 Panic

必须：
- LoadingState 统一管理加载进度和加载动画
- 资源加载失败有兜底回退机制
- 异步加载配合 Bevy 资产句柄做进度追踪

---

## 规则18：WaitAction 阶段逻辑与动画分离

允许：
- WaitAction 阶段只等待动画事件，不执行逻辑
- 逻辑在 ExecuteAction 阶段全部完成
- 使用 ActionTimeline（行动时间轴/命令队列）作为逻辑层与动画层的桥梁

禁止：
- 在 WaitAction 阶段执行任何游戏逻辑
- 逻辑层直接控制动画播放
- 动画层修改游戏状态

必须：
- 逻辑层只产出 GameEvent（如 UnitMoved, UnitDamaged）到 ActionTimeline
- UI/动画层从 ActionTimeline 消费事件并播放动画
- 只有当 ActionTimeline 为空时，TurnPhase 才从 WaitAction 推进到 TurnEnd

> **优化来源**: docs/architecture/app-bootstrap.md — SRPG 核心痛点「逻辑与动画的时间轴撕裂」

SRPG 最怕"逻辑已经算完了（单位已死），但死亡动画还在播，玩家狂点鼠标触发了下一个回合"。WaitAction 阶段的设计正是解决这一痛点。

---

## 规则19：Schedule 设计规则

允许：
- `PreUpdate` 专用于输入处理（读取原始输入事件，转换为游戏输入）
- `Update` 用于游戏逻辑（核心业务系统运行，TurnPhase 驱动）
- `PostUpdate` 专用于 UI 更新与清理（ViewModel 刷新、临时资源清理）
- `FixedUpdate` 用于需要固定步长的逻辑（物理、动画帧）

禁止：
- 在 PreUpdate 中执行游戏逻辑
- 在 PostUpdate 中修改游戏状态
- 在 Update 中执行确定性战斗结算（应放在 FixedUpdate）

必须：
- 各 Schedule 严格遵守其职责边界
- 确定性战斗结算必须放在 FixedUpdate 中（固定 10Hz tick），避免帧率波动影响战斗数值
- Update 只处理输入和 UI 表现，不参与确定性结算

> **优化来源**: docs/architecture/app-bootstrap.md — Schedule 设计与职责划分

### TurnPhase 内的系统顺序

在 `Update` Schedule 中，系统按 TurnPhase 驱动：

```
TurnPhase::SelectUnit
  ├─ highlight_reachable_tiles    # 高亮可达格子
  ├─ highlight_enemies_in_range   # 高亮范围内敌人
  └─ select_unit_on_click         # 点击选择单位

TurnPhase::MoveUnit
  ├─ show_movement_range          # 显示移动范围
  ├─ move_unit_to_target          # 移动单位到目标格
  └─ path_find_system             # 寻路计算

TurnPhase::ActionMenu
  └─ show_action_menu             # 显示动作菜单

TurnPhase::SelectTarget
  ├─ highlight_targets            # 高亮可选目标
  └─ select_target_on_click       # 点击选择目标

TurnPhase::ExecuteAction
  ├─ combat_intent_system         # 生成战斗意图
  ├─ effect_generate              # 效果管线 - 生成
  ├─ effect_modify                # 效果管线 - 修饰
  ├─ effect_execute               # 效果管线 - 执行
  └─ buff_resolve_system          # Buff 结算

TurnPhase::WaitAction
  └─ wait_for_animation           # 等待动画/结算完成

TurnPhase::TurnEnd
  ├─ turn_end_cleanup             # 回合结束清理
  ├─ victory_defeat_check         # 胜负判定
  └─ next_turn_or_phase           # 下一回合/阶段
```

---

## 规则20：SystemSet 排序约束

允许：
- 使用 SystemSet 对系统进行分组和排序
- Set 之间通过 after()/before() 建立依赖关系
- Set 内部系统可并行执行

禁止：
- 绕过 Set 排序直接注册 System（所有 System 必须归属某个 Set）
- 在 Set 内部执行跨 Set 的逻辑（Set 之间只传递数据）
- Set 过度拆分（每个系统都建一个 Set）

必须：
- 顶层 Set 保持 6~8 个的量级
- Set 可以有子 Set（如 LogicSet 包含 TurnSet、CombatSet、MapSet）

### 标准 SystemSet 排序

```
InputSet → CommandSet → LogicSet → EffectSet → ViewModelSet → UISet
```

| SystemSet | 职责 | 包含的典型系统 |
|-----------|------|----------------|
| `InputSet` | 输入处理 | `keyboard_input`, `mouse_input`, `touch_input` |
| `CommandSet` | 命令分发 | `command_handler`, `ui_command_dispatch` |
| `LogicSet` | 业务逻辑 | `turn_system`, `movement_system`, `combat_system` |
| `EffectSet` | 效果管线 | `generate_effects`, `modify_effects`, `execute_effects` |
| `ViewModelSet` | 视图模型更新 | `update_battle_ui`, `update_buff_panel`, `update_turn_queue` |
| `UISet` | UI 渲染 | `render_ui`, `refresh_ui_panels` |

> **优化来源**: docs/architecture/app-bootstrap.md — SystemSet 设计与排序约束

---

## 规则21：关闭序列规则

允许：
- 使用 BattleEntity 标记组件统一清理战斗 Entity
- 使用 OnExit 自动清理机制
- 关闭逻辑必须幂等（多次调用不会出错）

禁止：
- 在关闭时修改业务状态（关闭是清理，不是操作）
- 在 OnExit 中手动逐个 despawn Entity
- 在 OnExit 里手动清理特定业务 Resource（让 Bevy 的 Res 自然覆盖或在 OnEnter 时重置）

必须：
- 使用 `#[derive(Component)] struct BattleEntity` 标记所有在 InGame 状态生成的 Entity
- 使用 `despawn_recursive()` 统一清理 BattleEntity，包含所有子 Entity
- OnExit 只做一件事：`commands.entity(e).despawn_recursive()` with `BattleEntity` marker
- 优先使用 Bevy 的 OnExit 自动清理（而非手动清理）

### 关闭序列流程

```
游戏关闭
  │
  ├─ OnExit(AppState::InGame)
  │   ├─ save_game_state()           # 保存游戏状态（可选）
  │   └─ cleanup_battle_resources()  # 清理战斗资源
  │
  ├─ OnExit(AppState::GameOver)
  │   └─ cleanup_game_over_ui()      # 清理结算 UI
  │
  └─ App 关闭
      ├─ 各 Plugin::cleanup()        # 各 Plugin 自行清理
      └─ 释放全局资源
```

> **优化来源**: docs/architecture/app-bootstrap.md — BattleEntity 标记组件清理与 OnExit 机制

---

## 规则22：Plugin 注册顺序强制执行

允许：
- 按 Shared → Infrastructure → Core → Content → UI → Debug → Modding → Tools 顺序注册
- 使用 require_plugins 机制在编译时检测依赖顺序

禁止：
- 任何违反依赖方向的注册顺序
- 在 App 层手动维护大量 `#[cfg]` 条件编译注册

必须：
- 所有 Plugin 按依赖层次从底层到顶层注册
- App 层的 add_plugins 调用顺序反映依赖关系
- 条件编译在 PluginGroup 内部处理，App 层保持干净

---

## 规则23：Plugin 粒度控制

允许：
- 按业务领域拆分 Plugin（BattlePlugin、TurnPlugin、SkillPlugin）
- 每个业务模块一个 Plugin
- Shared/Content/UI/Debug/Modding 各一个统一 Plugin

禁止：
- 为单个实现创建 Plugin（过度拆分）
- 超过 3 个不同业务领域的 Plugin 不拆分
- 超过 50 个 System 的 Plugin 不考虑拆分

必须：
- Plugin 职责过大时必须拆分
- 拆分按业务领域，不按代码数量

> **优化来源**: docs/architecture/plugin-design.md

---

## 规则24：Plugin build() 声明式约束

允许：
- build() 中仅做声明式注册：add_plugins、add_message、init_resource、add_systems
- build() 中声明依赖关系

禁止：
- build() 中执行业务逻辑（如伤害计算、状态判断）
- build() 中硬编码数值
- build() 中直接修改 Resource 值（build 时 Resource 可能未就绪）
- build() 中触发 Message（build 时无 System 消费）
- build() 中执行文件 I/O（阻塞注册流程）

必须：
- 业务逻辑放在 System 中，不在 build() 中
- 初始化后逻辑放在 initialize() 钩子中
- 清理逻辑放在 shutdown() 钩子中

> **优化来源**: docs/architecture/plugin-design.md

---

## 规则25：Plugin 通信方式选择

允许：
- 跨 Plugin 广播使用 Message（如 BattlePlugin → UiPlugin：DamageApplied）
- 同 Plugin 内局部响应使用 Observer（如 Dead 标签添加时清理）
- 跨 Plugin 只读查询使用 Resource（如 DebugPlugin 读取 BattleRecord）
- UI → 业务命令使用 UiCommand
- 组件固有行为使用 Hook

禁止：
- 跨 Plugin 通信不使用 Message（直接调用其他 Plugin 的 System）
- 同 Plugin 内简单逻辑使用 Observer（应直接函数调用）
- Resource 用于广播场景（应使用 Message）

必须：
- 是否跨 Plugin 边界是选择通信方式的首要判断
- 跨 Plugin 通信必须使用 Message
- 同 Plugin 内局部响应优先使用 Observer

> **优化来源**: docs/architecture/plugin-design.md

---

## 规则26：架构例外审批

允许的例外场景：
- 性能优化：Core 层可直接读取 Infrastructure 数据（只读），需 Architect 审批 + ADR 记录
- 紧急修复：跨层直接调用（跳过事件机制），需 Tech Lead 审批 + 24h 内修复
- 平台集成：Infrastructure 直接依赖平台 API，需 Architect 审批 + feature gate

禁止：
- 未经审批的跨层直接调用
- 例外超过有效期未修复
- 例外未在 ADR 中记录

必须：
- 例外申请必须在 ADR 中记录原因、放宽的具体约束、影响范围、预计修复时间
- 代码中标记 TODO 和有效期
- 下次迭代中优先修复

> **优化来源**: docs/architecture/layer-contracts.md

---

## 规则27：核心路径性能简化

允许的简化：
- 核心战斗路径（每帧多次调用的伤害计算、属性修饰计算）可简化为直接函数调用
- 简化需 ADR 批准 + 文档说明 + 有效期

禁止：
- 非核心路径的跨层通信简化（必须严格遵守分层规则）
- 未经 ADR 批准的简化
- 凭直觉优化（必须先 Profile 确认瓶颈）

必须：
- 核心路径识别标准：每帧多次调用的计算（伤害、属性、寻路）
- 简化审批流程：发现问题 → Profile 确认 → 编写 ADR → Architect 审批 → 实施 → 有效期评估

> **优化来源**: docs/architecture/layer-contracts.md

---

## 规则28：跨平台适配

允许：
- 统一使用 std::path::Path / PathBuf 处理路径
- 使用 dirs crate 获取标准目录（config_dir()、data_dir()）
- 平台特定 API 封装为统一 trait，通过 feature flag 条件编译

禁止：
- 直接调用 Windows 注册表 API（应封装为 PlatformConfig trait）
- 依赖 macOS 独有文件系统特性（如 .app bundle 路径）
- 硬编码 / 或 \ 路径分隔符（使用 Path::join）
- 假设文件系统大小写敏感性（macOS 默认不敏感）

必须：
- 涉及文件系统、平台 API 的模块使用跨平台抽象
- 适用模块：steam、config、cloud_save、crash_report、persistence、assets

> **优化来源**: docs/architecture/infrastructure-design.md

---

## 规则29：错误枚举独立归属

允许：
- 每个 Infrastructure 模块定义自己的错误枚举（SaveError、LoadError、AssetError 等）
- 跨模块错误通过转换处理

禁止：
- 创建全局统一 InfrastructureError 大枚举
- 使用 anyhow::Error、Box<dyn Error> 作为业务层返回类型
- 基础设施错误包含领域语义（SkillId、UnitId 等）
- 领域错误放在 infrastructure/ 或 shared/

必须：
- 领域错误放在 core/xxx/domain/xxx_error.rs
- 基础设施错误放在 infrastructure/xxx/xxx_error.rs
- 共享错误工具放在 shared/error/

> **优化来源**: docs/architecture/infrastructure-design.md

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

## 层间依赖自动化检测管线

```
源码扫描 → 违规检测 → CI 阻断 → 修复验证
```

### Step1：源码扫描

输入：src/ 目录
处理：扫描所有 .rs 文件的 use 语句，检测跨层引用
输出：依赖引用列表
工具：dependcheck.rs 脚本 + 自定义 Clippy lint

### Step2：违规检测

输入：依赖引用列表
处理：对照层间依赖规则检测违规
输出：违规报告（文件、行号、违规类型）
禁止：忽略任何违规

### Step3：CI 阻断

输入：违规报告
处理：CI 中运行依赖检查，有违规则失败
输出：构建结果
必须：CI 必须运行层间依赖检查，每次 PR 必须通过

### Step4：修复验证

输入：修复后的代码
处理：重新运行依赖检查，确认违规已修复
输出：合规报告
必须：发现违规必须立即修复，不允许"先通过后续修"

> **优化来源**: docs/architecture/layer-contracts.md

---

## 架构演进流程

```
需求评审 → 规则修订 → 全团队同步 → 代码迁移
```

### Step1：需求评审

输入：业务变化或性能需求
处理：评估是否需要调整分层规则或放宽约束
输出：演进需求文档
禁止：未经评审直接修改架构规则

### Step2：规则修订

输入：演进需求文档
处理：Architect 编写 ADR，更新 layer-contracts.md 和相关文档
输出：更新后的架构文档
必须：MAJOR 变更需全员 Review + CEO 批准，MINOR 变更需 Architect 审批

### Step3：全团队同步

输入：更新后的架构文档
处理：架构评审会议、更新 Code Review 检查清单、更新 CI 检测脚本
输出：团队同步确认
必须：每次架构变更必须配套迁移指南

### Step4：代码迁移

输入：迁移指南
处理：按迁移检查清单逐项迁移，新旧代码共存期间使用 #[deprecated] 标记
输出：迁移完成确认
必须：迁移期间每个阶段保持项目可编译、可运行、测试通过

> **优化来源**: docs/architecture/layer-contracts.md

---

## 模块迁移流程

```
创建 Git Tag → 逐模块迁移 → 编译验证 → 测试验证 → 技术债务清理 → 合入主分支
```

### Step1：创建 Git Tag

输入：迁移计划
处理：创建 phase-N-start 和 phase-N-complete Tag
输出：回滚点
必须：每个 Phase 必须建立明确的 Git Tag 回滚点

### Step2：逐模块迁移

输入：待迁移模块
处理：逐模块移动文件、更新 use 引用和 mod 声明
输出：迁移后的代码
必须：每迁一个模块就验证一次编译

### Step3：编译验证

输入：迁移后的代码
处理：运行 cargo build + cargo clippy
输出：编译结果
必须：每个阶段必须通过所有现有测试

### Step4：测试验证

输入：编译通过的代码
处理：运行 cargo test + 依赖图检查
输出：测试结果
禁止：测试不通过就合入

### Step5：技术债务清理

输入：通过测试的代码
处理：清理 #[deprecated] 标记、死代码、旧路径重导出
输出：清理后的代码
必须：每个 Phase 完成后预留 1 天作为技术债务清理窗口期

> **优化来源**: docs/architecture/migration-roadmap.md

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

🟥 禁止：Core 模块 use 任何 Infrastructure、UI、Content、Modding 层的模块（宪法 1.3.2 依赖方向铁则）

原因：Core 是游戏规则的纯领域逻辑层，依赖技术实现或表现层会破坏可移植性和可测试性

违反后果：循环依赖、代码耦合爆炸、无法独立测试

---

🟥 禁止：Shared 模块 use 任何其他层的模块（宪法 3.0.7 通用代码规范）

原因：Shared 是依赖图的叶子节点，依赖任何层都会创建循环依赖倾向

违反后果：编译循环依赖、shared 变成垃圾桶

---

🟥 禁止：在 shared/ 中放置业务相关工具模块（如 skill_utils、battle_utils）（宪法 3.0.7：绝对禁止在 common/ 中放入任何业务逻辑）

原因：业务工具应该放在业务领域内部，shared 只放与业务无关的基础工具

违反后果：shared 变成万能垃圾桶、依赖图混乱

---

🟥 禁止：Content 层包含任何游戏规则逻辑（宪法 1.1.3 规则与内容强制分离）

原因：Content 只做"加载→校验→注册"三件事，包含规则逻辑违反 Content/Rule 分离

违反后果：内容扩展时需要修改 Rust 代码

---

🟥 禁止：App 层包含任何业务逻辑（宪法 1.1.1 Feature First + 1.3.1 三层架构）

原因：App 只组装游戏，不执行游戏规则，包含逻辑违反职责单一

违反后果：App 层变成上帝对象、无法测试

---

禁止：无视三问判断法凭感觉放置模块

原因：三问判断法是唯一客观的归属判定标准，凭感觉会导致归属混乱

违反后果：模块归属不一致、依赖方向违规

---

🟥 禁止：UI 直接访问 Core 的 ECS 组件（宪法 1.1.4 逻辑与表现强制分离）

原因：UI 只能通过 ViewModel 读取状态、通过 UiCommand 发送命令，直接访问会破坏 Logic/Presentation 分离

违反后果：UI 和业务逻辑耦合、无法独立替换 UI

---

🟥 禁止：Plugin 间隐式依赖（宪法 3.0.1 接口最小化 + 3.0.2 Plugin 是唯一对外入口）

原因：隐式依赖导致 Resource 未就绪时运行 panic，且无法被编译器检测

违反后果：运行时 panic、依赖关系不明确、重构困难

---

禁止：绕过 Plugin 边界注册资源

原因：在 App 层直接注册属于其他 Plugin 的 Resource 破坏 Plugin 封装性

违反后果：Plugin 的 build() 职责模糊、Resource 所有权不明确

---

🟥 禁止：Plugin 循环依赖（宪法 1.3.2 依赖方向铁则的延伸）

原因：循环依赖导致编译失败、架构无法维护

违反后果：编译错误、Plugin 无法独立初始化

---

🟥 禁止：AppPlugin 功能膨胀（从注册器退化为大泥球）（宪法 1.1.7 只解决当前复杂度）

原因：一开始 AppPlugin 只注册插件，后来慢慢加全局系统、资源初始化、状态切换逻辑，最终变成新的上帝对象

违反后果：App 层不可测试、业务逻辑与装配逻辑耦合、所有修改都要经过 App 层

---

禁止：SystemSet 过度拆分（每个系统都建一个 Set）

原因：导致排序规则爆炸、维护成本飙升，顶层 Set 应保持 6~8 个的量级

违反后果：执行流水线难以理解、新增系统时 Set 排序决策困难

---

🟥 禁止：状态机无限嵌套（超过两层子状态）（宪法 1.1.7 只解决当前复杂度）

原因：两层是复杂度的黄金平衡点，再深会出现状态转换地狱

违反后果：状态转换逻辑指数级增长、调试和维护成本失控

---

禁止：Infrastructure 模块包含领域语义错误（SkillError、BattleError、BuffError 等）

原因：Infrastructure 是技术实现层，包含领域语义会导致技术实现与业务规则耦合

违反后果：替换 Infrastructure 实现时需要修改领域逻辑，破坏可替换性原则

> **优化来源**: docs/architecture/infrastructure-design.md

---

🟥 禁止：Plugin build() 中执行业务逻辑（宪法 3.0.2 Plugin 是唯一对外入口）

原因：build() 只在注册时调用一次，不是业务逻辑执行点，业务逻辑应放在 System 中

违反后果：业务逻辑只执行一次、无法被调度器管理、破坏声明式注册原则

> **优化来源**: docs/architecture/plugin-design.md

---

禁止：三棵树之间交叉引用或混放

原因：src/（源码）、content/（RON 配置）、assets/（二进制资源）必须物理分离

违反后果：内容路径变化影响编译、资产与配置混淆、MOD 扩展困难

具体禁止：
- src/ 中引入 content/ 路径硬编码（应通过 AssetServer 动态加载）
- content/ 中放入 Rust 代码（混淆数据与逻辑）
- assets/ 中放入 RON 配置（混淆资产与配置）
- mods/ 中直接修改 src/（MOD 应通过 modding/api/ 扩展）

> **优化来源**: docs/architecture/project-structure.md

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

---

## 如果执行架构迁移

允许：
- 按 Phase 0→7 顺序逐步迁移
- 每个 Phase 创建 Git Tag 回滚点
- 使用 pub use 在旧位置创建重新导出

禁止：
- 跳过 Phase 顺序
- 一个 Phase 中同时迁移多个不相关模块
- 迁移后不清理技术债务（#[deprecated] 标记、死代码）

优先检查：
- 每个 Phase 是否保持编译通过和测试通过
- 迁移后是否更新所有 use 引用
- 是否预留技术债务清理窗口期
- 功能验证清单是否逐项通过

> **优化来源**: docs/architecture/migration-roadmap.md