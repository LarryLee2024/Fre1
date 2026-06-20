# 37条宝贵经验吸收 — 项目文档全面更新计划

> 来源：`docs/ai_ignore_this_dir/13宝贵经验.md`
> 日期：2026-06-20
> 状态：待执行

---

## 一、覆盖度总览

| 覆盖状态 | 数量 | 经验编号 |
|---------|------|---------|
| **已充分覆盖** | 17 | 1, 2, 5, 10, 11, 12, 13, 14, 15, 23, 24, 26, 28, 29, 35 + 9(设计意图), 22(设计意图) |
| **部分覆盖** | 15 | 3, 4, 6, 7, 8, 16, 17, 18, 19, 20, 21, 25, 30, 32, 36 |
| **未覆盖** | 5 | 27, 31, 33, 34, 37 |

---

## 二、专项计划 A：宪法文件更新

### 目标文件
- `docs/00-governance/ai-constitution-complete.md`

### A1. 需要新增的内容

| 编号 | 经验 | 新增位置 | 新增内容 |
|------|------|---------|---------|
| A1-1 | #27 统一术语 | 新增编或 §2 补充 | 新增"统一术语宪法"：项目必须维护 `ubiquitous_language.md`，所有核心业务术语有唯一名称；代码类型名/函数名/配置Key必须与术语表一致；新增术语需经 domain-designer 审批 |
| A1-2 | #31 Fitness Function | §19 补充 | 扩展"架构守卫"条款：架构规则必须编码为可自动执行的 Fitness Function，集成到 CI；每次 PR 必须通过 Fitness Function 检查；包含依赖方向、模块边界、文件大小、Data Law 合规等断言 |
| A1-3 | #34 Explain模式 | 新增编或 §8 补充 | 新增"可解释性宪法"：所有复杂计算（伤害结算、AI评分、掉落判定）必须支持 `explain()` 返回 `CalcBreakdown`；Explain 结果可序列化，支持 UI 展示和 QA 验证 |
| A1-4 | #37 架构预算 | §16 补充 | 收紧架构预算条款：单函数 <= 50行（硬限制），单文件 <= 500行（硬限制），单Domain <= 15子模块（建议值）；架构预算纳入 Fitness Function 自动检查 |

### A2. 需要修改的内容

| 编号 | 经验 | 修改位置 | 修改方案 |
|------|------|---------|---------|
| A2-1 | #17 Policy | §8 战斗宪法补充 | 在战斗结算条款中增加 Policy 模式要求：伤害/掉落/目标策略必须收敛为独立 Policy 对象，禁止散落在 System 中的 if 链 |
| A2-2 | #18 CQRS | §8.9 读写分离补充 | 将"CQRS Lite"升级为"CQRS"：Domain integration 层必须区分 WriteFacade（命令处理）和 ReadFacade（查询API），读模型使用扁平化 View 结构体 |
| A2-3 | #3 Trait Object | §8.1 角色系统补充 | 增加"Registry + Trait Object 替代 match"条款：Effect/Condition/Trigger 等能力域的执行分发必须使用 `dyn TraitExecutor` + Registry 查表，禁止 50+ 臂 match 表达式 |
| A2-4 | #4 Reflect | §3 ECS宪法补充 | 增加"Reflect 工程价值"条款：Reflect 不仅用于 Inspector，更是消灭手动 `app.register_type::<T>()` 的关键手段；必须通过 derive 宏或 build 脚本自动生成注册代码 |
| A2-5 | #7 Macro边界 | §16 AI可读性补充 | 区分"声明式宏（重复结构，允许）"和"过程宏生成逻辑（需 ADR 审批）"的边界 |
| A2-6 | #19 Context | §3.3 补充 | 增加"领域专用 Context"要求：每个复杂业务操作必须定义专用 Context 结构体，Context 是领域纯函数的输入参数，不依赖 ECS World 直接查询 |
| A2-7 | #32 Feature Flag | §16.4 补充 | 扩展 Feature 成熟度分级为运行时机制：Def 增加 `stability: Experimental | Stable | Deprecated` 字段，运行时根据 Flag 过滤可用内容 |

### A3. 需要删除的内容

无。宪法文件无需删除内容，仅补充和细化。

---

## 三、专项计划 B：规则文件更新

### B1. 架构规则（`.trae/rules/架构规则.md`）

| 编号 | 经验 | 操作 | 具体内容 |
|------|------|------|---------|
| B1-1 | #31 Fitness Function | 新增 | "架构适应度函数"章节：定义可自动执行的架构断言清单（依赖方向、Domain隔离、文件大小、模块数限制）；集成到 CI pipeline；参考 ArchUnit 设计理念 |
| B1-2 | #37 架构预算 | 新增 | "架构预算硬限制"章节：单函数50行、单文件500行、单Domain 15模块、单模块公开API 20个；纳入 Fitness Function |
| B1-3 | #3 Trait Object | 新增 | "Registry + Trait Object 替代 match"模式指导：在 Effect 执行、Condition 检查、Trigger 匹配场景中，优先使用 `dyn EffectExecutor` + Registry 查表 |
| B1-4 | #15 Registry vs 枚举 | 新增 | "Registry vs 全局枚举决策指南"：类型需运行时扩展时必须用 Registry；仅编译期封闭类型允许 enum；给出决策树 |
| B1-5 | #6 Query Facade | 新增 | "Query Facade 模式"：每个 Domain integration/ 层应提供只读查询 API，明确区分写操作 Facade 和读操作 Facade |
| B1-6 | #18 CQRS | 新增 | "CQRS 模型设计"：Domain integration 层区分 WriteFacade 和 ReadFacade，读模型使用扁平化 View 结构体 |
| B1-7 | #21 Resolver | 新增 | "统一 Resolver 设计"：定义 `WorldResolver` SystemParam，提供 `resolver.unit(id)` 等统一访问入口 |
| B1-8 | #36 ACL | 新增 | "反腐层 ACL 规范"：Infra 层所有外部接入必须通过 ACL 模块隔离；ACL 负责外部模型→内部模型转换 |
| B1-9 | #32 Feature Flag | 新增 | "运行时 Feature Flag"：定义 `FeatureFlag` Resource，支持运行时查询功能可用性；与 Content 层集成 |

### B2. ECS规则（`.trae/rules/ECS规则.md`）

| 编号 | 经验 | 操作 | 具体内容 |
|------|------|------|---------|
| B2-1 | #4 Reflect | 新增 | "Reflect 工程价值"章节：明确 Reflect 是消灭手动注册代码的关键手段；建议通过自定义 derive 宏自动生成注册代码 |
| B2-2 | #8 Bundle Factory | 新增 | "Core 层 Bundle Factory"：为游戏实体提供标准化 spawn 工厂函数（如 `spawn_unit(commands, UnitSpawnProps)`），与 UI Factory 保持一致 |
| B2-3 | #16 生命周期 | 新增 | "生命周期建模规范"：涉及状态流转的必须用 enum + 状态机，简单开关标记允许 bool；审查现有 bool 字段 |

### B3. SRPG专项规则（`.trae/rules/SRPG专项规则.md`）

| 编号 | 经验 | 操作 | 具体内容 |
|------|------|------|---------|
| B3-1 | #17 Policy | 新增 | "Policy 模式"章节：定义 Policy trait（如 `trait DamagePolicy`），将伤害计算中的暴击/免疫/减伤/属性克制抽离为独立 Policy 对象 |
| B3-2 | #19 Context | 新增 | "Context 对象规范"：每个复杂业务操作定义专用 Context 结构体（BattleContext/AbilityContext/QuestContext），Context 包含操作所需全部输入 |
| B3-3 | #34 Explain | 新增 | "Explain 模式"章节：所有复杂计算必须支持 `explain()` 返回 `CalcBreakdown`；Explain 结果可序列化，支持战斗履历 UI 展示 |
| B3-4 | #33 Event History | 新增 | "Domain Event History"章节：所有领域事件自动记录到 EventStore；支持按 Entity/Time/EventType 查询；与 Replay 系统集成 |

### B4. 代码风格（`.trae/rules/代码风格.md`）

| 编号 | 经验 | 操作 | 具体内容 |
|------|------|------|---------|
| B4-1 | #7 Macro边界 | 新增 | "Macro 使用边界"：允许用于重复结构声明（derive 宏、声明式 BSN），禁止用于生成业务逻辑；自定义过程宏必须经 ADR 审批 |

### B5. 测试规范（`.trae/rules/测试规范.md`）

| 编号 | 经验 | 操作 | 具体内容 |
|------|------|------|---------|
| B5-1 | #31 Fitness Function | 新增 | "架构适应度测试"：Fitness Function 本身需要测试覆盖，确保架构断言正确性 |
| B5-2 | #34 Explain | 新增 | "Explain 结果验证"：复杂计算的 Explain 输出应作为测试断言的一部分，验证中间步骤正确性 |

---

## 四、专项计划 C：领域文件更新

### C1. 新增文件

| 编号 | 经验 | 文件路径 | 内容 |
|------|------|---------|------|
| C1-1 | #27 统一术语 | `docs/02-domain/ubiquitous_language.md` | 项目级统一术语表：所有核心业务术语的唯一名称和含义；术语与代码类型名/函数名/配置Key的一致性要求；新增术语审批流程；LocalizationKey 命名空间与术语表对齐 |

### C2. 需要修改的文件

| 编号 | 经验 | 文件 | 修改方案 |
|------|------|------|---------|
| C2-1 | #17 Policy | `docs/02-domain/combat/` 相关规则文件 | 增加 Policy 模式在战斗域的应用：DamagePolicy（暴击/免疫/减伤/属性克制）、TargetPolicy（目标优先级/选择规则）、LootPolicy（掉落规则） |
| C2-2 | #19 Context | `docs/02-domain/combat/` 相关规则文件 | 增加 BattleContext/AbilityContext 的领域规则定义 |
| C2-3 | #34 Explain | `docs/02-domain/combat/` 相关规则文件 | 增加"伤害结算可解释性"规则：DamageBreakdown 必须包含基础伤害+修正+暴击+抗性的完整分解 |
| C2-4 | #33 Event History | `docs/02-domain/combat/` 相关规则文件 | 增加"战斗事件历史"规则：所有战斗事件必须持久化，支持回放分析 |
| C2-5 | #20 Specification | 各 Domain 规则文件 | 明确 Condition 系统即 Specification 模式的实现，增加 `evaluate() -> ConditionResult` 返回结构化结果（Pass/Fail + 原因 + LocalizationKey）的规范 |

---

## 五、专项计划 D：架构文件更新

### D1. 需要新增的 ADR

| 编号 | 经验 | ADR 编号建议 | 标题 |
|------|------|-------------|------|
| D1-1 | #31 Fitness Function | ADR-0XX | 架构适应度函数（Architectural Fitness Function） |
| D1-2 | #27 统一术语 | ADR-0XX | 统一术语表（Ubiquitous Language） |
| D1-3 | #17 Policy | ADR-0XX | Policy 模式在战斗域的应用 |
| D1-4 | #34 Explain | ADR-0XX | 复杂逻辑可解释性（Explain 模式） |
| D1-5 | #33 Event History | ADR-0XX | Domain Event History 设计 |
| D1-6 | #37 架构预算 | ADR-0XX | 架构预算硬限制 |

### D2. 需要修改的 ADR

| 编号 | 经验 | ADR | 修改方案 |
|------|------|-----|---------|
| D2-1 | #3 Trait Object | ADR-013 Registry | 补充"Registry + Trait Object 替代 match"的模式指导 |
| D2-2 | #4 Reflect | ADR-054 Bevy 0.19 迁移 | 补充 Reflect 自动注册机制的设计 |
| D2-3 | #18 CQRS | ADR-024 Combat integration | 补充 WriteFacade/ReadFacade 的区分设计 |
| D2-4 | #6 Query Facade | ADR-046 统一模块接口 | 补充只读查询 API 的独立地位 |

---

## 六、专项计划 E：数据架构文件更新

### E1. 需要修改的文件

| 编号 | 经验 | 文件 | 修改方案 |
|------|------|------|---------|
| E1-1 | #30 Migration | `docs/04-data/foundation/migration_policy.md` | 完成 pending 状态的迁移策略文档；设计统一版本迁移框架覆盖 Content Schema + Save Schema + Replay Schema |
| E1-2 | #32 Feature Flag | `docs/04-data/` 相关文件 | 增加 Def 稳定性字段设计：`stability: Experimental | Stable | Deprecated`；废弃 Def 的自动检测和版本迁移方案 |
| E1-3 | #25 删除机制 | `docs/04-data/` 相关文件 | 补充完整的 Experimental/Stable/Deprecated 三态流转规范；代码中 `#[deprecated]` 的使用规范 |
| E1-4 | #33 Event History | `docs/04-data/` 相关文件 | 增加 Domain Event History 的存储设计：EventStore Schema、查询接口、与 Replay 的关系 |

---

## 七、专项计划 F：UI设计文件更新

### F1. 需要修改的文件

| 编号 | 经验 | 文件 | 修改方案 |
|------|------|------|---------|
| F1-1 | #34 Explain | `docs/06-ui/` 相关文件 | 增加"战斗履历 UI"设计：DamageBreakdown 的 UI 展示方案；Explain 结果通过 Cue 传递给 UI 的管线设计 |
| F1-2 | #18 CQRS | `docs/06-ui/` 相关文件 | 强化 Projection 防火墙的 CQRS 语义：明确 ViewModel 是 Domain Read Model 的 UI 投影，不是简单的数据拷贝 |
| F1-3 | #6 Query Facade | `docs/06-ui/` 相关文件 | 补充 UI 层通过 ReadFacade 获取数据的规范，禁止 UI 直接构造 ECS Query |

---

## 八、执行优先级排序

### P0 — 立即执行（影响项目可维护性根基）

| 序号 | 经验 | 计划编号 | 核心行动 |
|------|------|---------|---------|
| 1 | #27 统一术语 | C1-1 | 创建 `docs/02-domain/ubiquitous_language.md` |
| 2 | #31 Fitness Function | B1-1, D1-1 | 设计架构适应度函数，集成到 CI |
| 3 | #34 Explain模式 | B3-3, C2-3, F1-1 | 设计 CalcBreakdown/Explain 系统 |
| 4 | #37 架构预算 | A1-4, B1-2, D1-6 | 收紧架构预算，纳入 Fitness Function |

### P1 — 近期执行（影响开发效率和代码质量）

| 序号 | 经验 | 计划编号 | 核心行动 |
|------|------|---------|---------|
| 5 | #17 Policy模式 | A2-1, B3-1, C2-1, D1-3 | 在 Combat 域引入 Policy 模式 |
| 6 | #3 Trait Object | A2-3, B1-3, D2-1 | 统一 Trait Object 执行器模式 |
| 7 | #18 CQRS | A2-2, B1-6, D2-3 | Domain 层引入 Aggregate + ReadFacade |
| 8 | #33 Event History | B3-4, C2-4, E1-4, D1-5 | 设计 Domain Event History |
| 9 | #19 Context | A2-6, B3-2, C2-2 | 定义领域专用 Context 对象 |
| 10 | #4 Reflect | A2-4, B2-1, D2-2 | 设计自动注册宏 |

### P2 — 中期执行（完善设计模式覆盖）

| 序号 | 经验 | 计划编号 | 核心行动 |
|------|------|---------|---------|
| 11 | #6 Query Facade | B1-5, D2-4, F1-3 | 为所有 Domain 建立 ReadFacade |
| 12 | #7 Macro边界 | A2-5, B4-1 | 明确宏使用边界 |
| 13 | #8 Bundle Factory | B2-2 | Core 层实体工厂标准化 |
| 14 | #15 Registry vs 枚举 | B1-4 | 增加决策指南 |
| 15 | #16 生命周期 | B2-3 | 规范 enum vs bool 选择标准 |
| 16 | #20 Specification | C2-5 | Condition 返回结构化结果 |
| 17 | #21 Resolver | B1-7 | 设计统一 Resolver |
| 18 | #25 删除机制 | E1-3 | 完善三态生命周期 |
| 19 | #30 Migration | E1-1 | 完成迁移策略文档 |
| 20 | #32 Feature Flag | A2-7, B1-9, E1-2 | 设计运行时 Feature Flag |

### P3 — 远期执行（当前规模影响有限）

| 序号 | 经验 | 计划编号 | 核心行动 |
|------|------|---------|---------|
| 21 | #36 ACL | B1-8 | 外部系统接入的防腐层（单机暂不需要） |
| 22 | #9 领域DSL | — | Trigger+Effect 组合配置 DSL 化（当前 Rule/Content 分离是有意选择） |

---

## 九、关键发现总结

### 项目优势（已超越经验要求）

1. **四级通信机制**（Hook>Trigger>Observer>Message）比简单的"Event化"更精细
2. **五层能力架构**（Type→Tag→Query→Rule→Content）比简单的"Capability而非继承"更系统
3. **Content Platform 5层内容体系**比简单的"配置>代码"更完整
4. **Command 模式**已完整实现，支持 Replay/AI/玩家统一入口
5. **CalcTrace**已提供计算追踪能力，是 Explain 模式的基础

### 系统性缺失（需优先补齐）

1. **统一术语表**（#27）— 50万行项目术语漂移是隐性杀手
2. **Fitness Function**（#31）— 架构规则不自动检查等于没有规则
3. **Explain 模式**（#34）— 战棋游戏伤害明细是核心体验
4. **架构预算**（#37）— 阻止复杂度无限增长的硬限制

### 设计意图存在但未显式化（需文档化）

1. **Policy 模式**（#17）— Condition 部分实现，但缺少显式 Policy 对象
2. **CQRS**（#18）— 读写分离原则存在，但缺少 Aggregate/View 模型
3. **Trait Object 执行器**（#3）— Registry 存在，但缺少 `dyn Executor` 模式
4. **统一 Resolver**（#21）— 各 Registry 独立访问，缺少统一入口
