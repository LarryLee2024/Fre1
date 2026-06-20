# 37条宝贵经验吸收 — 项目文档全面更新计划

> 来源：`docs/ai_ignore_this_dir/13宝贵经验.md`
> 日期：2026-06-20
> 状态：待执行
> 复核：2026-06-20（4智能体交叉验证，修正覆盖度判断 + 补充遗漏项）

---

## 一、覆盖度总览（复核后修正）

| 覆盖状态 | 修正前 | 修正后 | 修正说明 |
|---------|--------|--------|---------|
| **已充分覆盖** | 17 | 14 | #1,#2,#5,#10,#11,#12,#13,#22,#26,#29 降级为"部分覆盖"（有差距）；#19,#20 升级入此列 |
| **部分覆盖** | 15 | 22 | +10条从"已充分覆盖"降级 + #31,#33,#34,#37从"未覆盖"升级 + #17,#18,#27,#32从"未覆盖"升级 |
| **未覆盖** | 5 | 1 | 仅 #37架构预算仍为"未覆盖"（无自动检查机制） |

### 修正后的完整覆盖度表

| 编号 | 经验名称 | 修正后状态 | 修正理由 |
|------|---------|-----------|---------|
| 1 | 用数据消灭代码 | 部分覆盖 | 缺少"新增内容只需新增数据，0行Rust代码"的量化目标 |
| 2 | ECS Event化 | 部分覆盖 | 未明确禁止"系统互调"模式（仅约束Domain间，未约束Capability内/同Domain内） |
| 3 | Trait Object减少match | 部分覆盖 | Registry存在但缺 `dyn Executor` 模式 |
| 4 | Reflect消灭注册代码 | 部分覆盖 | Reflect已要求但缺自动注册宏 |
| 5 | Command模式 | 部分覆盖 | **Undo/Network场景完全未提及** |
| 6 | Query Facade | 部分覆盖 | 仅combat/tactical有integration层，其余13个Domain缺失 |
| 7 | Macro只做重复结构 | 部分覆盖 | 有AI可读性约束和BSN规范，缺声明式宏vs过程宏边界 |
| 8 | Bundle Factory | 部分覆盖 | UI层完整，Core层游戏实体缺标准化工厂 |
| 9 | 领域DSL | 部分覆盖 | RuleDef已实现声明式DSL，但与Data Law 002的矛盾需显式记录 |
| 10 | Layer化 | 部分覆盖 | 有依赖方向约束，缺"下层不认识上层"的类型不可见性表述 |
| 11 | Shared Kernel | 部分覆盖 | 缺 `formula/` 公式引擎（公式散落在各Domain的rules/中） |
| 12 | Content Pipeline | 部分覆盖 | 缺"10万行代码+50万行内容"的量化目标 |
| 13 | Capability而非类型层级 | 部分覆盖 | 缺 `has::<CanAttack>()` 式的显式运行时查询API |
| 14 | 领域规则集中化 | 已充分覆盖 | RuleDef { condition, effect } + Rule Engine 已完整 |
| 15 | Registry优于枚举 | 已充分覆盖 | DefRegistry + Mod扩展 + 热重载已完整 |
| 16 | 生命周期显式建模 | 部分覆盖 | Effect/Ability有enum，但部分领域仍用bool；缺enum vs bool选择标准 |
| 17 | 用Policy代替if | 部分覆盖 | **复核修正**：RestockPolicy已实现（economy域），缺DamagePolicy/TargetPolicy/LootPolicy |
| 18 | 读写模型分离CQRS | 部分覆盖 | **复核修正**：MovementCapabilityView + 8个Combat Facade + AggregateDirty事件已落地CQRS核心模式，但未用CQRS术语命名，缺显式WriteFacade/ReadFacade抽象 |
| 19 | 定义Context对象 | 已充分覆盖 | **复核修正**：GameplayContext + ExecutionContext + TargetContext + ConditionContext + PipelineContext + ActivationContext 已完整覆盖 |
| 20 | Specification模式 | 已充分覆盖 | **复核修正**：Condition系统已实现 `evaluate() -> ConditionResult(Passed/Failed{reason})`，完全对应 `spec.is_satisfied()` |
| 21 | 统一Resolver | 部分覆盖 | IdAllocator/RuntimeIdAllocator已落地，缺统一WorldResolver |
| 22 | 领域层隔离Bevy ECS | 部分覆盖 | rules/纯函数存在，但输入输出规范未文档化（未强制"只接受值类型"） |
| 23 | 组合关系数据化 | 已充分覆盖 | **复核补充**：Content Platform L1 AbilityDef组合模式（引用EffectDef+TargetingDef+ConditionDef+CueDef）已完整实现 |
| 24 | Feature Module按业务分 | 已充分覆盖 | 15 Capability + 15 Domain，无全局技术目录 |
| 25 | 设计删除机制 | 部分覆盖 | 文档级有三态，代码/配置级缺Experimental/Stable/Deprecated流转；id_strategy.md已有Active/Deprecated/Archived |
| 26 | 业务对象不长期持有其他业务对象 | 部分覆盖 | ID关系已实现，缺"对象图指数级 vs ID关系线性级"的对比指导 |
| 27 | 统一术语Vocabulary | 部分覆盖 | **复核修正**：L0 Vocabulary层（6种Def定义）已建立，缺显式ubiquitous_language.md项目级术语表 |
| 28 | 唯一真相源SSOT | 部分覆盖 | 有SSOT原则，但DamageFormula缺明确的唯一实现位置 |
| 29 | Query不允许跨Feature | 部分覆盖 | **复核修正**：仅combat和tactical有integration层，覆盖率仅13%（2/15） |
| 30 | Versioned Data + Migration | 部分覆盖 | save_version字段+SaveOperation::Migrate已定义，但无迁移逻辑；ContentMigration trait已存在于content-platform-manifesto.md |
| 31 | Architectural Fitness Function | 部分覆盖 | **复核修正**：check-identity-invariants.sh已实现（支持CI模式），审查规则有9维手动检查清单，缺自动化断言清单和CI集成 |
| 32 | Feature Flag管理演化 | 部分覆盖 | **复核修正**：RuleDef.enabled提供规则级开关；Semantic Tags已定义sem:stable/sem:deprecated/sem:experimental，缺通用Feature Flag框架 |
| 33 | Domain Event History | 部分覆盖 | **复核修正**：CommandHistory+Replay录制已落地；日志规则定位"日志=领域事件履历"；event_schema.md已预留Event History扩展点；event_domain.md已定义Archived状态；缺EventStore持久化和查询 |
| 34 | 复杂逻辑可解释 | 部分覆盖 | **复核修正**：CalcTrace完整覆盖战斗计算（formula_id+inputs+intermediate_values+output），PriceBreakdown覆盖经济交易；注释规则§9要求"复杂公式必须解释来源"；ContextChain溯源链支持Explain；缺统一explain()接口规范和UI展示管线 |
| 35 | 不追求通用系统 | 已充分覆盖 | "三次才抽象"原则存在，但缺"重复"的判定标准 |
| 36 | 反腐层ACL | 部分覆盖 | Combat integration层已标注为ACL；Mod API Facade+Gateway存在；缺外部系统接入ACL |
| 37 | 架构预算 | 未覆盖 | 无自动检查机制；现有500/1000行阈值均为软限制 |

---

## 二、专项计划 A：宪法文件更新

### 目标文件
- `docs/00-governance/ai-constitution-complete.md`

### A1. 需要新增的内容

| 编号 | 经验 | 新增位置 | 新增内容 |
|------|------|---------|---------|
| A1-1 | #27 统一术语 | 新增编或 §2 补充 | 新增"统一术语宪法"：项目必须维护 `ubiquitous_language.md`（基于现有L0 Vocabulary层和31个领域文件的"统一术语"节汇总）；所有核心业务术语有唯一名称；代码类型名/函数名/配置Key必须与术语表一致；新增术语需经 domain-designer 审批 |
| A1-2 | #31 Fitness Function | §19 补充 | 扩展"架构守卫"条款：架构规则必须编码为可自动执行的 Fitness Function（基于现有check-identity-invariants.sh扩展），集成到 CI；每次 PR 必须通过 Fitness Function 检查；Fitness Function 是不变量测试的架构级泛化 |
| A1-3 | #34 Explain模式 | 新增编或 §8 补充 | 新增"可解释性宪法"：所有复杂计算必须支持 `explain()` 返回 `CalcBreakdown`；**CalcBreakdown 基于 CalcTrace 扩展**（非重新设计），增加 ContextChain 溯源数据；Explain 结果可序列化，支持 UI 展示和 QA 验证；与注释规则§9的关系：注释负责静态文档，Explain负责运行时分解 |
| A1-4 | #37 架构预算 | §16 补充 | 收紧架构预算条款：单函数 <= 50行（硬限制，从现有100行警觉阈值收紧），单文件 <= 500行（硬限制，从现有500行建议/1000行强制收紧），单Domain <= 15子模块（建议值）；架构预算纳入 Fitness Function 自动检查 |
| A1-5 | #1 量化目标 | §10 补充 | 增加"新增内容只需新增数据"的量化目标：新增技能/角色/Buff的目标是0行Rust代码+1个RON配置 |
| A1-6 | #5 Undo/Network | §8.7 补充 | 扩展Command模式条款：Command架构必须支持 Undo（基于Command反转栈）和 Network 同步（基于Command序列化） |
| A1-7 | #2 禁止系统互调 | §6.3 补充 | 明确禁止"系统互调"模式：不仅Domain间禁止直接调用，Capability内和同Domain内系统间也必须通过事件通信，禁止 `system_a()` 直接调用 `system_b()` |
| A1-8 | #10 类型不可见性 | §2.9 补充 | 增加"类型不可见性"条款：下层对上层的类型完全不可见，不仅是依赖方向约束，更是编译期类型隔离 |
| A1-9 | #13 Capability查询API | §8.1 补充 | 增加 `has::<CanAttack>()` 式的运行时Capability查询API规范 |
| A1-10 | #26 复杂度对比指导 | §6.1 补充 | 增加"对象图复杂度指数级 vs ID关系线性级"的对比指导，作为架构决策判断依据 |

### A2. 需要修改的内容

| 编号 | 经验 | 修改位置 | 修改方案 |
|------|------|---------|---------|
| A2-1 | #17 Policy | §8 战斗宪法补充 | 在战斗结算条款中增加 Policy 模式要求：伤害/掉落/目标策略必须收敛为独立 Policy 对象（参考economy域RestockPolicy），禁止散落在 System 中的 if 链 |
| A2-2 | #18 CQRS | §8.9 读写分离补充 | 将"CQRS Lite"升级为"CQRS"：**基于现有MovementCapabilityView + Combat Facade + AggregateDirty模式显式化**，Domain integration 层必须区分 WriteFacade（命令处理）和 ReadFacade（查询API），读模型使用扁平化 View 结构体 |
| A2-3 | #3 Trait Object | §8.1 角色系统补充 | 增加"Registry + Trait Object 替代 match"条款：Effect/Condition/Trigger 等能力域的执行分发必须使用 `dyn TraitExecutor` + Registry 查表，禁止 50+ 臂 match 表达式 |
| A2-4 | #4 Reflect | §3 ECS宪法补充 | 增加"Reflect 工程价值"条款：Reflect 不仅用于 Inspector，更是消灭手动 `app.register_type::<T>()` 的关键手段；必须通过 derive 宏或 build 脚本自动生成注册代码 |
| A2-5 | #7 Macro边界 | §16 AI可读性补充 | 区分"声明式宏（重复结构，允许）"和"过程宏生成逻辑（需 ADR 审批）"的边界；**与ECS规则§3.7 BSN宏规范保持一致** |
| A2-6 | #32 Feature Flag | §16.4 补充 | 扩展 Feature 成熟度分级为运行时机制：**复用现有Semantic Tags（sem:stable/sem:deprecated/sem:experimental）**，不新增独立字段；运行时根据 Flag 过滤可用内容 |
| A2-7 | #22 Domain隔离ECS | §3.4 补充 | 强制 rules/ 纯函数的输入输出规范：纯函数只接受值类型参数（非ECS类型），输出为Domain结果类型；System层负责ECS→Domain值类型的转换 |
| A2-8 | #28 SSOT | §8.2 补充 | 明确 DamageFormula 在 combat/rules/ 中的唯一实现位置；UI伤害预览必须通过integration层调用同一DamageFormula |
| A2-9 | #11 Shared Kernel | §2.4 补充 | 考虑将通用公式计算基础框架下沉到 Shared 或 Core Capability 层（当前公式散落在各Domain的rules/中） |
| A2-10 | #35 抽象判定标准 | §16.2 补充 | 定义"重复"的判定标准：业务语义相同（非仅函数签名相同或逻辑相同）才算重复，第三次出现时才抽象 |

### A3. 需要删除的内容

无。宪法文件无需删除内容，仅补充和细化。

---

## 三、专项计划 B：规则文件更新

### B1. 架构规则（`.trae/rules/架构规则.md`）

| 编号 | 经验 | 操作 | 具体内容 |
|------|------|------|---------|
| B1-1 | #31 Fitness Function | 新增 | "架构适应度函数"章节：**基于现有check-identity-invariants.sh和审查规则9维手动检查清单扩展**；定义可自动执行的架构断言清单（依赖方向、Domain隔离、文件大小、模块数限制）；集成到 CI pipeline；Fitness Function 是不变量测试的架构级泛化 |
| B1-2 | #37 架构预算 | 新增 | "架构预算硬限制"章节：**从现有500行建议/1000行强制收紧为500行硬限制**；单函数50行（从100行警觉收紧）；单Domain 15模块；单模块公开API 20个；纳入 Fitness Function |
| B1-3 | #3 Trait Object | 新增 | "Registry + Trait Object 替代 match"模式指导 |
| B1-4 | #15 Registry vs 枚举 | 新增 | "Registry vs 全局枚举决策指南" |
| B1-5 | #6 Query Facade | 新增 | "Query Facade 模式"：**基于ADR-024已有facade.rs读/写分离模式**，为所有15个Domain逐步建立integration层 |
| B1-6 | #18 CQRS | 新增 | "CQRS 模型设计"：**基于现有MovementCapabilityView + Combat Facade + AggregateDirty模式显式化** |
| B1-7 | #21 Resolver | 新增 | "统一 Resolver 设计" |
| B1-8 | #36 ACL | 新增 | "反腐层 ACL 规范" |
| B1-9 | #32 Feature Flag | 新增 | "运行时 Feature Flag"：**复用Semantic Tags（sem:stable/sem:deprecated/sem:experimental）**，不新增独立字段 |

### B2. ECS规则（`.trae/rules/ECS规则.md`）

| 编号 | 经验 | 操作 | 具体内容 |
|------|------|------|---------|
| B2-1 | #4 Reflect | 新增 | "Reflect 工程价值"章节 |
| B2-2 | #8 Bundle Factory | 新增 | "Core 层 Bundle Factory"：**与现有§3.7 spawn_*()工厂函数规范保持一致**，增加标准化参数结构体 |
| B2-3 | #16 生命周期 | 新增 | "生命周期建模规范"：**与现有§1.3 Tag Component vs bool指导互补**，增加enum状态机 vs bool的选择标准 |

### B3. SRPG专项规则（`.trae/rules/SRPG专项规则.md`）

| 编号 | 经验 | 操作 | 具体内容 |
|------|------|------|---------|
| B3-1 | #17 Policy | 新增 | "Policy 模式"章节：**参考economy域RestockPolicy实现**，在Combat域引入DamagePolicy/TargetPolicy/LootPolicy |
| B3-2 | #34 Explain | 新增 | "Explain 模式"章节：**基于CalcTrace扩展为CalcBreakdown**，增加ContextChain溯源数据；与注释规则§9的关系说明 |
| B3-3 | #33 Event History | 新增 | "Domain Event History"章节：**基于event_domain.md的Archived状态和event_schema.md的Future Extension扩展**；与Replay系统的边界：Replay录制Command序列（输入确定性），Event History记录Domain Event（业务事实），两者互补不替代 |
| B3-4 | #9 领域DSL矛盾 | 新增 | "声明式DSL vs 命令式DSL"说明：RuleDef的condition+effect结构是声明式DSL（数据描述意图，代码解释执行），与命令式DSL（配置中写逻辑代码）有本质区别，因此不违反Data Law 002 |

### B4. 代码风格（`.trae/rules/代码风格.md`）

| 编号 | 经验 | 操作 | 具体内容 |
|------|------|------|---------|
| B4-1 | #7 Macro边界 | 新增 | "Macro 使用边界"：**与ECS规则§3.7 BSN宏规范保持一致** |

### B5. 测试规范（`.trae/rules/测试规范.md`）

| 编号 | 经验 | 操作 | 具体内容 |
|------|------|------|---------|
| B5-1 | #31 Fitness Function | 新增 | "架构适应度测试"：Fitness Function 是不变量测试的架构级泛化 |
| B5-2 | #34 Explain | 新增 | "Explain 结果验证"：复杂计算的Explain输出应作为测试断言的一部分 |

### B6. 复核新增：被遗漏的规则文件更新

以下8个规则文件在原计划中被完全忽略，需补充更新：

| 编号 | 规则文件 | 相关经验 | 更新内容 |
|------|---------|---------|---------|
| B6-1 | AI架构准则.md | #3, #7, #37 | 补充Trait Object执行器模式指导、宏使用边界、架构预算硬限制引用 |
| B6-2 | AI开发宪法.md | #3, #16, #37 | 补充Trait Object执行器作为正面模式、生命周期enum vs bool指导、架构预算从"警觉"升级为"硬限制" |
| B6-3 | AI协作规则.md | #17, #31, #34 | 反模式黑名单增加"散落if链代替Policy对象"和"复杂计算无Explain支持"；质量门禁增加Fitness Function通过 |
| B6-4 | 审查规则.md | #31 | 将9维手动检查清单与Fitness Function对齐，增加自动化断言条目 |
| B6-5 | Bug修复规则.md | #31 | Fitness Function集成到Bug修复的CI检查流程 |
| B6-6 | 日志规则.md | #33 | EventStore与日志系统的关系；领域事件持久化规范；**日志规则定位"日志=领域事件履历"是Event History的概念基础** |
| B6-7 | 注释规则.md | #34 | 补充注释级公式解释（§9）与运行时Explain的关系说明，避免重复 |
| B6-8 | 文档治理规则.md | #25, #27 | 增加术语一致性要求（#27）；代码/配置废弃机制与文档废弃机制的对齐（#25） |

---

## 四、专项计划 C：领域文件更新

### C1. 新增文件

| 编号 | 经验 | 文件路径 | 内容 |
|------|------|---------|------|
| C1-1 | #27 统一术语 | `docs/02-domain/ubiquitous_language.md` | 项目级统一术语表：**从31个领域文件的"统一术语"节和L0 Vocabulary层汇总**；所有核心业务术语的唯一名称和含义；术语与代码类型名/函数名/配置Key的一致性要求；新增术语审批流程；LocalizationKey命名空间与术语表对齐 |

### C2. 需要修改的文件

| 编号 | 经验 | 文件 | 修改方案 |
|------|------|------|---------|
| C2-1 | #17 Policy | `docs/02-domain/combat/` 相关规则文件 | 增加 Policy 模式在战斗域的应用：DamagePolicy/TargetPolicy/LootPolicy |
| C2-2 | #34 Explain | `docs/02-domain/combat/` 相关规则文件 | 增加"伤害结算可解释性"规则：**基于CalcTrace扩展为DamageBreakdown**，增加ContextChain溯源数据 |
| C2-3 | #33 Event History | `docs/02-domain/combat/` 相关规则文件 | 增加"战斗事件历史"规则：**基于event_domain.md的Archived状态扩展** |
| C2-4 | #9 领域DSL | `docs/02-domain/capabilities/condition_domain.md` 等 | 明确RuleDef的声明式DSL与Data Law 002的关系 |

### C3. 复核新增：删除原C2-5（#20 Specification）

原计划C2-5提议"Condition增加evaluate() -> ConditionResult返回结构化结果"，但复核发现**这已完整实现**。删除此条。

---

## 五、专项计划 D：架构文件更新

### D1. 需要新增的 ADR

| 编号 | 经验 | ADR 编号建议 | 标题 |
|------|------|-------------|------|
| D1-1 | #31 Fitness Function | ADR-0XX | 架构适应度函数（Architectural Fitness Function） |
| D1-2 | #27 统一术语 | ADR-0XX | 统一术语表（Ubiquitous Language） |
| D1-3 | #17 Policy | ADR-0XX | Policy 模式在战斗域的应用 |
| D1-4 | #34 Explain | ADR-0XX | 复杂逻辑可解释性（Explain 模式）— 基于CalcTrace扩展 |
| D1-5 | #33 Event History | ADR-0XX | Domain Event History 设计 — 基于event_schema.md Future Extension |
| D1-6 | #37 架构预算 | ADR-0XX | 架构预算硬限制 |

### D2. 需要修改的 ADR

| 编号 | 经验 | ADR | 修改方案 |
|------|------|-----|---------|
| D2-1 | #3 Trait Object | ADR-013 Registry | 补充"Registry + Trait Object 替代 match"的模式指导 |
| D2-2 | #4 Reflect | ADR-054 Bevy 0.19 迁移 | 补充 Reflect 自动注册机制的设计 |
| D2-3 | #18 CQRS | ADR-024 Combat integration | 补充 WriteFacade/ReadFacade 的区分设计；**ADR-024已有facade.rs读/写分离雏形（build_effect_view读 / request_effect_apply写），应基于此显式化** |
| D2-4 | #6 Query Facade | ADR-046 统一模块接口 | 补充只读查询 API 的独立地位 |
| D2-5 | #33 Event History | ADR-041 Replay | 明确Event History与Replay系统的边界：Replay录制Command序列（输入），Event History记录Domain Event（输出），两者互补 |
| D2-6 | #33 Event History | ADR-049 共享事件 | ADR-049已定义的共享事件（TurnEnded/BattleStarted等）是Event History的种子数据 |
| D2-7 | #34 Explain | ADR-051 Error/Failure | RuleFailure.code()与Explain的CalcBreakdown存在协同关系，可复用code体系进行错误溯源 |

---

## 六、专项计划 E：数据架构文件更新

### E1. 需要修改的文件

| 编号 | 经验 | 文件 | 修改方案 |
|------|------|------|---------|
| E1-1 | #30 Migration | `docs/04-data/foundation/migration_policy.md` | 完成 pending 状态的迁移策略文档；**复用ContentMigration trait（已存在于content-platform-manifesto.md §8.3）**，统一Content Schema + Save Schema + Replay Schema的迁移框架 |
| E1-2 | #32 Feature Flag | `docs/04-data/` 相关文件 | 增加 Def 稳定性设计：**复用Semantic Tags（sem:stable/sem:deprecated/sem:experimental），不新增独立stability字段**；废弃Def的自动检测和版本迁移方案 |
| E1-3 | #25 删除机制 | `docs/04-data/` 相关文件 | 补充Experimental/Stable/Deprecated三态流转规范：**基于id_strategy.md已有的Active/Deprecated/Archived三态扩展**，增加Experimental态；代码中 `#[deprecated]` 的使用规范 |
| E1-4 | #33 Event History | `docs/04-data/capabilities/event_schema.md` | 增加 Domain Event History 的存储设计：**基于event_schema.md §10 Future Extension（已预留"事件历史缓冲区"和"事件追踪"）扩展**；EventStore Schema、查询接口 |
| E1-5 | #34 Explain | `docs/04-data/capabilities/execution_schema.md` | 补充CalcTrace的扩展设计：**CalcTrace已存在（formula_id+inputs+intermediate_values+output），CalcBreakdown应基于此扩展**，增加ContextChain溯源数据和RuleFailure.code()关联 |

---

## 七、专项计划 F：UI设计文件更新

### F1. 需要修改的文件

| 编号 | 经验 | 文件 | 修改方案 |
|------|------|------|---------|
| F1-1 | #34 Explain | `docs/06-ui/` 相关文件 | 增加"战斗履历 UI"设计：**基于overlays.md已有的DamageText Overlay扩展**，而非全新设计；Explain结果通过Cue传递给UI的管线设计 |
| F1-2 | #18 CQRS | `docs/06-ui/` 相关文件 | 明确 **Projection 防火墙 = ReadFacade 的 UI 层实现**（不是"需要强化"，而是已有实现需显式化CQRS语义）；ViewModel 是 Domain Read Model 的 UI 投影 |
| F1-3 | #6 Query Facade | `docs/06-ui/` 相关文件 | 补充 UI 层通过 ReadFacade 获取数据的规范，禁止 UI 直接构造 ECS Query |

---

## 八、执行优先级排序（复核后修正）

### P0 — 立即执行（影响项目可维护性根基）

| 序号 | 经验 | 计划编号 | 核心行动 | 复核修正 |
|------|------|---------|---------|---------|
| 1 | #27 统一术语 | C1-1 | 创建 `ubiquitous_language.md`（从31个领域文件术语节+L0 Vocabulary汇总） | 修正：从"未覆盖"→"部分覆盖"，已有L0 Vocabulary基础 |
| 2 | #31 Fitness Function | B1-1, B6-1~6, D1-1 | 扩展架构适应度函数（基于check-identity-invariants.sh），集成到CI | 修正：从"未覆盖"→"部分覆盖"，已有1个Fitness Function工具 |
| 3 | #34 Explain模式 | A1-3, B3-2, C2-2, E1-5, F1-1 | 基于CalcTrace扩展为CalcBreakdown | **重大修正**：从"未覆盖"→"部分覆盖"，CalcTrace+PriceBreakdown已实现 |
| 4 | #37 架构预算 | A1-4, B1-2, D1-6 | 收紧架构预算（从软限制→硬限制），纳入Fitness Function | 仍为"未覆盖"（无自动检查机制） |
| 5 | #29 Integration层覆盖 | B1-5 | 为其余13个Domain建立integration层 | **新增**：原计划遗漏，覆盖率仅13%（2/15） |

### P1 — 近期执行（影响开发效率和代码质量）

| 序号 | 经验 | 计划编号 | 核心行动 | 复核修正 |
|------|------|---------|---------|---------|
| 6 | #17 Policy模式 | A2-1, B3-1, C2-1, D1-3 | 在Combat域引入Policy模式（参考RestockPolicy） | 修正：从"未覆盖"→"部分覆盖" |
| 7 | #18 CQRS显式化 | A2-2, B1-6, D2-3, F1-2 | 将现有View/Facade/AggregateDirty模式显式化为CQRS术语 | **重大修正**：从"未覆盖"→"部分覆盖"，CQRS核心模式已落地 |
| 8 | #33 Event History | B3-3, C2-3, E1-4, D1-5, D2-5~6 | 基于event_schema.md扩展点设计EventStore | 修正：从"未覆盖"→"部分覆盖" |
| 9 | #5 Command Undo/Network | A1-6 | 扩展Command模式支持Undo和Network | **新增**：原计划遗漏 |
| 10 | #3 Trait Object | A2-3, B1-3, D2-1 | 统一Trait Object执行器模式 |
| 11 | #4 Reflect | A2-4, B2-1, D2-2 | 设计自动注册宏 |

### P2 — 中期执行（完善设计模式覆盖）

| 序号 | 经验 | 计划编号 | 核心行动 | 复核修正 |
|------|------|---------|---------|---------|
| 12 | #7 Macro边界 | A2-5, B4-1, B6-1~2 | 明确宏使用边界 |
| 13 | #8 Bundle Factory | B2-2 | Core层实体工厂标准化 |
| 14 | #15 Registry vs 枚举 | B1-4 | 增加决策指南 |
| 15 | #16 生命周期 | B2-3, B6-2 | 规范enum vs bool选择标准 |
| 16 | #21 Resolver | B1-7 | 设计统一Resolver |
| 17 | #25 删除机制 | E1-3 | 基于id_strategy.md已有三态扩展 |
| 18 | #30 Migration | E1-1 | 复用ContentMigration trait完成迁移策略 |
| 19 | #32 Feature Flag | A2-6, B1-9, E1-2 | 复用Semantic Tags设计Feature Flag |
| 20 | #22 Domain隔离ECS | A2-7 | 强制rules/纯函数输入输出规范 |
| 21 | #28 SSOT | A2-8 | 明确DamageFormula唯一实现位置 |
| 22 | #1 量化目标 | A1-5 | 增加"0行Rust代码"目标 |
| 23 | #2 禁止系统互调 | A1-7 | 明确禁止Capability内/同Domain内系统互调 |
| 24 | #10 类型不可见性 | A1-8 | 增加编译期类型隔离条款 |
| 25 | #13 Capability查询API | A1-9 | 增加has::<>()式查询API |
| 26 | #26 复杂度对比指导 | A1-10 | 增加对象图vs ID关系复杂度对比 |
| 27 | #11 Shared Kernel | A2-9 | 公式引擎下沉 |
| 28 | #35 抽象判定标准 | A2-10 | 定义"重复"的判定标准 |

### P3 — 远期执行（当前规模影响有限）

| 序号 | 经验 | 计划编号 | 核心行动 |
|------|------|---------|---------|
| 29 | #36 ACL | B1-8 | 外部系统接入的防腐层（单机暂不需要） |
| 30 | #9 领域DSL | B3-4 | 声明式DSL vs 命令式DSL的显式记录 |
| 31 | #12 量化目标 | A1-5 | "10万行代码+50万行内容"比例目标（战略愿景） |

---

## 九、关键发现总结（复核后修正）

### 项目优势（已超越经验要求）

1. **四级通信机制**（Hook>Trigger>Observer>Message）比简单的"Event化"更精细
2. **五层能力架构**（Type→Tag→Query→Rule→Content）比简单的"Capability而非继承"更系统
3. **Content Platform 5层内容体系**比简单的"配置>代码"更完整
4. **Command 模式**已完整实现，支持 Replay/AI/玩家统一入口
5. **CalcTrace + PriceBreakdown**已构成完整的计算解释体系（#34 Explain已部分覆盖）
6. **Condition evaluate() -> ConditionResult**已完整实现Specification模式（#20已充分覆盖）
7. **Context体系**已完整覆盖6种业务Context（#19已充分覆盖）
8. **CQRS核心模式**已通过View+Facade+AggregateDirty落地（#18已部分覆盖，非"未覆盖"）

### 系统性缺失（需优先补齐）

1. **Integration层覆盖率仅13%**（#29）— 15个Domain仅2个有integration层，这是最严重的架构债务
2. **统一术语表**（#27）— L0 Vocabulary已有基础，缺项目级ubiquitous_language.md
3. **Fitness Function**（#31）— 已有1个工具，需扩展覆盖面并集成CI
4. **架构预算**（#37）— 无自动检查机制，现有阈值均为软限制

### 原计划最大遗漏（复核发现）

1. **8个规则文件完全被忽略**（AI架构准则、AI开发宪法、AI协作规则、审查规则、Bug修复规则、日志规则、注释规则、文档治理规则），其中多个包含与37条经验直接相关的内容
2. **已有基础设施的复用不足** — CalcTrace、Semantic Tags、ContentMigration trait、id_strategy.md三态、event_schema.md扩展点、ADR-024 facade雏形等已有设计未被计划识别
3. **概念间的协同关系未建立** — ContextChain与Explain、RuleFailure.code()与Explain、不变量测试与Fitness Function、Replay与Event History、Projection与ReadFacade等关系未在计划中体现
4. **#29 Integration层覆盖率严重失实** — 原计划标为"已充分覆盖"，实际仅2/15 Domain有integration层
5. **#34 Explain被严重低估** — 原计划标为"未覆盖"，实际CalcTrace+PriceBreakdown已构成完整计算解释体系
6. **#5 Command模式Undo/Network遗漏** — 原计划标为"已充分覆盖"，但Undo和Network场景完全未提及
7. **#9 领域DSL与Data Law 002的矛盾未显式记录** — RuleDef的声明式DSL不违反Data Law 002，但需显式说明

### 覆盖度判断修正汇总

| 经验 | 原计划 | 修正后 | 修正原因 |
|------|--------|--------|---------|
| #1 数据驱动 | 已充分覆盖 | 部分覆盖 | 缺量化目标 |
| #2 事件化 | 已充分覆盖 | 部分覆盖 | 未禁止系统互调 |
| #5 Command | 已充分覆盖 | 部分覆盖 | 缺Undo/Network |
| #10 Layer化 | 已充分覆盖 | 部分覆盖 | 缺类型不可见性 |
| #11 Shared Kernel | 已充分覆盖 | 部分覆盖 | 缺formula/ |
| #12 Content Pipeline | 已充分覆盖 | 部分覆盖 | 缺量化比例目标 |
| #13 Capability | 已充分覆盖 | 部分覆盖 | 缺has::<>()API |
| #17 Policy | 未覆盖 | 部分覆盖 | RestockPolicy已实现 |
| #18 CQRS | 未覆盖 | 部分覆盖 | View+Facade+AggregateDirty已落地 |
| #19 Context | 部分覆盖 | 已充分覆盖 | 6种Context已完整 |
| #20 Specification | 部分覆盖 | 已充分覆盖 | evaluate()->ConditionResult已实现 |
| #22 Domain隔离ECS | 已充分覆盖 | 部分覆盖 | rules/输入输出规范未文档化 |
| #26 ID关系 | 已充分覆盖 | 部分覆盖 | 缺复杂度对比指导 |
| #27 统一术语 | 未覆盖 | 部分覆盖 | L0 Vocabulary已建立 |
| #28 SSOT | 已充分覆盖 | 部分覆盖 | DamageFormula缺唯一实现位置 |
| #29 Query不跨Feature | 已充分覆盖 | 部分覆盖 | 覆盖率仅13%（2/15） |
| #31 Fitness Function | 未覆盖 | 部分覆盖 | check-identity-invariants.sh已实现 |
| #32 Feature Flag | 未覆盖 | 部分覆盖 | RuleDef.enabled+Semantic Tags已存在 |
| #33 Event History | 未覆盖 | 部分覆盖 | CommandHistory+Replay+日志规则+event_schema扩展点 |
| #34 Explain | 未覆盖 | 部分覆盖 | CalcTrace+PriceBreakdown+注释规则§9+ContextChain |
| #37 架构预算 | 未覆盖 | 未覆盖 | 无自动检查机制（维持原判断） |
