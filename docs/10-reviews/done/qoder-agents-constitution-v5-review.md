# Qoder Agents vs 宪法 v5.0 深度对照审查报告

> 审查时间：2026-06-16
> 审查范围：`.qoder/agents/` 全部 7 个角色文件 vs `docs/00-governance/ai-constitution-complete.md` v5.0
> 审查重点：宪法中特别重要的事项是否在各角色中有遗漏或冲突
> 审查方法：逐条对照宪法 21 编核心条款，检查每个角色的覆盖情况

---

## 审查方法论

按宪法条款重要性，将审查维度分为三个层级：

| 层级 | 维度 | 说明 |
|------|------|------|
| **P0** | 架构边界/红线 | 宪法 §1.5、§3.5、§21 的核心铁则 |
| **P1** | 通信机制/ECS/测试 | 宪法 §6、§3.6、§10 的执行规范 |
| **P2** | 完整性引用 | 宪法 §14、§18、§19、§20 的辅助规范 |

---

## 一、architect.md（首席架构师）

### ❌ 冲突/遗漏

| # | 严重度 | 问题 | 宪法条款 | 说明 |
|---|--------|------|----------|------|
| A1 | 🔴 Critical | ADR 通信设计缺少 Query API | §3.6.2 | ADR 模板的 Communication Design 只列出 Hook/Trigger/Observer/Message，缺少 **Query API（读操作）**。宪法 §3.6.2 明确规定"双轨制：写操作走 Event，读操作走 Query API"，这是 Domain 间通信的核心原则 |
| A2 | 🔴 Critical | 验证清单引用旧架构 Plugin 注册顺序 | §3.1 | 验证清单第 2 项"符合 Plugin 注册顺序（Core → Data → Logic → Presentation）"是旧架构术语，v5.0 已升级为 Capabilities/Domains 双轴，应改为验证双轴边界合规性 |
| A3 | 🟡 High | 缺少 Capabilities/Domains 双轴架构原则 | §1.5, §3.1 | 铁律和工作原则中未明确将"Capabilities 管机制，Domains 管业务"作为核心设计原则，仅在工作原则中提到"规则与内容分离" |
| A4 | 🟡 High | 缺少 Domain integration.rs 模式 | §3.4, §3.6.1 | 宪法要求"每个 Domain 必须有且仅有一个集成层（`integration.rs`）作为与 Capabilities 的唯一交互入口"，但 ADR 模板和验证清单中未提及此约束 |
| A5 | 🟡 High | 缺少 17 条红线（§21）引用 | §21 | 宪法 §21 列出了 17 条绝对禁止事项，是架构设计的硬约束边界。验证清单中未引用 |
| A6 | 🟡 High | 缺少 26 条 AI 反模式（§20.1）引用 | §20.1 | 架构设计时应避免触发 AI 反模式，但角色中未提及 |
| A7 | 🔵 Medium | 缺少 Capabilities 内部三层（C1/C2/C3）引用 | §3.2 | 宪法定义了 Capabilities 内部的 Foundation/Mechanism/Runtime 三层，架构设计时需确保新能力领域遵循此结构 |
| A8 | 🔵 Medium | 缺少 15 个 Capabilities / 15 个 Domains 清单引用 | §3.3, §3.4 | 新增 Capability 需 ADR 审批（附则），但角色中未提及此约束 |
| A9 | 🔵 Medium | 缺少 Mod API Facade + Gateway 模式引用 | §3.7 | 宪法定义了完整的 Gateway 清单（15 个 Gateway），架构设计 Mod API 时应引用 |
| A10 | 🔵 Medium | 缺少性能例外机制引用 | §14 | 宪法定义了严格的性能例外申请流程（需 ADR + Profiling 数据），架构设计涉及性能权衡时需引用 |

---

## 二、domain-designer.md（领域建模专家）

### ❌ 冲突/遗漏

| # | 严重度 | 问题 | 宪法条款 | 说明 |
|---|--------|------|----------|------|
| B1 | 🟡 High | 缺少 Capabilities/Domains 边界意识 | §3.5 | 领域设计师输出的领域模型应明确"哪些规则复用 Capabilities 已有机制，哪些是 Domain 独有的业务规则"。当前角色仅关注业务规则，未引导区分机制 vs 规则 |
| B2 | 🟡 High | 缺少 15 个 Capabilities 清单引用 | §3.3 | 领域模型应避免重复定义 Capabilities 已有的机制（如 Tag、Modifier、Effect、Stacking 等），但角色中未列出已有 Capabilities 供参考 |
| B3 | 🟡 High | 缺少双轨通信规范引用 | §3.6.2 | 领域模型中的"领域事件"定义应遵循双轨制（写→Event，读→Query API），但角色模板中只提到事件，未区分读写路径 |
| B4 | 🟡 High | 缺少 Domain integration.rs 模式 | §3.4 | 领域模型应明确"Domain 如何调用 Capabilities"，即通过 integration.rs 统一入口。当前模板未涉及 |
| B5 | 🔵 Medium | 缺少 17 条红线引用 | §21 | 领域模型的"禁止事项"应与宪法红线对齐，但角色中未引用 |
| B6 | 🔵 Medium | 缺少 Request-Response 反模式禁令 | §3.6.2 | 宪法明确禁止"通过事件传递查询请求"，领域模型定义事件时应避免此反模式 |
| B7 | 🔵 Medium | 缺少 15 个 Domains 清单引用 | §3.4 | 新增 Domain 需 Architect 审批（§19.3），领域设计师应了解已有 Domain 清单以避免冲突 |

---

## 三、feature-developer.md（功能开发专家）

### ❌ 冲突/遗漏

| # | 严重度 | 问题 | 宪法条款 | 说明 |
|---|--------|------|----------|------|
| F1 | 🟡 High | Step 4 通信机制描述不完整 | §3.6.2, §20.4 | Step 4 写"写操作走 Event/Trigger/Command"，但宪法 §20.4 第 5 条明确"四级通信机制：Hook=生命周期，Trigger=事件链，Observer=局部响应，Message=跨域广播"。应区分模块内通信（Trigger/Observer）和跨 Domain 通信（Message） |
| F2 | 🟡 High | 缺少 P0 铁则引用 | §1.5 | 宪法 §1.5 列出 6 条 P0 级顶层铁则（Feature First、Data Driven First、Replay First 等），但角色的"必须遵守的架构原则"中仅覆盖了部分 |
| F3 | 🟡 High | 缺少 17 条红线（§21）完整引用 | §21 | 角色的"绝对禁令"列表是子集，未完整覆盖宪法 §21 的 17 条红线（如"禁止非确定性随机源"、"禁止硬编码游戏数值"等） |
| F4 | 🟡 High | 缺少 26 条 AI 反模式（§20.1）引用 | §20.1 | 角色的自检清单是精简版，未引用完整的 26 条反模式黑名单 |
| F5 | 🔵 Medium | 缺少 CI 门禁引用 | §18.6 | 宪法定义了 6 项 CI 门禁标准，功能开发完成后应确保代码能通过门禁 |
| F6 | 🔵 Medium | 缺少新增模块检查清单引用 | §19.1 | 宪法 §19.1 定义了 8 项检查清单，功能开发新增模块时应逐项确认 |
| F7 | 🔵 Medium | 缺少 Replay First 具体要求 | §1.5, §10 | 宪法要求"所有核心战斗逻辑必须可确定性重放"，但角色仅在测试要求中提到"确定性 Seed=42"，未将 Replay First 作为设计约束 |
| F8 | 🔵 Medium | 缺少 Data Driven First 引用 | §1.5 | 宪法要求"新增内容优先通过配置数据实现"，但角色中未明确引用 |
| F9 | 🔵 Medium | 缺少性能例外机制引用 | §14 | 功能开发涉及性能优化时，应遵循宪法的性能例外申请流程 |

---

## 四、test-guardian.md（测试卫士）

### ❌ 冲突/遗漏

| # | 严重度 | 问题 | 宪法条款 | 说明 |
|---|--------|------|----------|------|
| T1 | 🟡 High | 缺少 AI 反模式中测试相关规则 | §20.1 #17 | 宪法 §20.1 第 17 条"新增核心业务系统未附带任何测试用例"是反模式，但角色未引用此条 |
| T2 | 🟡 High | 缺少读路径副作用禁令 | §20.1 #20 | 宪法 §20.1 第 20 条"预览/仿真等读路径带有副作用"是反模式，测试应验证读路径无副作用 |
| T3 | 🟡 High | 缺少 17 条红线中测试相关项 | §21 | 宪法红线第 4 条"禁止非确定性随机源破坏回放"、第 11 条"禁止核心业务领域使用 unwrap/expect/panic/todo"等应作为测试审查项 |
| T4 | 🔵 Medium | 缺少 CI 门禁引用 | §18.6 | 宪法要求 `cargo test` 全部通过是 CI 门禁标准之一，测试卫士应确保测试能通过门禁 |
| T5 | 🔵 Medium | 缺少 Architecture Governance 引用 | §19.1 | 新增模块的测试应符合架构检查清单（如"错误类型定义在对应领域内部"） |
| T6 | 🔵 Medium | 缺少 Domain integration.rs 测试要求 | §3.4, §3.6.1 | 集成测试应验证 Domain 通过 integration.rs 与 Capabilities 的交互是否正确 |

---

## 五、code-reviewer.md（代码审查员）

### ❌ 冲突/遗漏

| # | 严重度 | 问题 | 宪法条款 | 说明 |
|---|--------|------|----------|------|
| C1 | 🔴 Critical | 审查清单缺少双轨通信审查 | §3.6.2 | 审查清单中"跨模块通信"仅提到"Hook/Trigger/Observer/Message"，缺少 **Query API（读操作）** 的审查维度。应审查 Domain 间读操作是否走 Query API 而非事件 |
| C2 | 🟡 High | 缺少 Capabilities/Domains 边界审查 | §3.5 | 审查清单未明确将"Capabilities 是否包含业务规则"和"Domain 是否重复实现通用机制"作为独立审查项 |
| C3 | 🟡 High | 缺少 Domain integration.rs 审查 | §3.4, §3.6.1 | 审查清单未检查"Domain 是否有且仅有一个 integration.rs 作为与 Capabilities 的唯一交互入口" |
| C4 | 🟡 High | 缺少 17 条红线（§21）引用 | §21 | 审查清单是精选子集，未完整覆盖宪法 §21 的 17 条红线 |
| C5 | 🟡 High | 缺少 26 条 AI 反模式（§20.1）引用 | §20.1 | 审查清单未引用完整的 AI 反模式黑名单作为审查标准 |
| C6 | 🔵 Medium | 缺少 Performance Constitution 审查 | §14 | 审查清单未检查"性能优化是否基于 Profiling 数据"、"是否使用了 ARCH_EXCEPTION 标记" |
| C7 | 🔵 Medium | 缺少 CI 门禁验证 | §18.6 | 审查报告应包含 CI 门禁通过情况 |
| C8 | 🔵 Medium | 缺少 Request-Response 反模式审查 | §3.6.2 | 宪法禁止"通过事件传递查询请求"，审查应检测此反模式 |
| C9 | 🔵 Medium | 缺少 Domain 间文件数阈值审查 | §3.5.2 | 宪法规定"单个 Domain 内部文件超过 80 个时必须评估是否拆分"，审查应包含此维度 |

---

## 六、refactor-guardian.md（技术债扫描专家）

### ❌ 冲突/遗漏

| # | 严重度 | 问题 | 宪法条款 | 说明 |
|---|--------|------|----------|------|
| R1 | 🟡 High | 缺少 Capabilities/Domains 边界违规扫描 | §3.5 | 扫描目标中未将"Capabilities 包含业务规则"和"Domain 重复实现通用机制"作为独立扫描类别 |
| R2 | 🟡 High | 缺少 Domain integration.rs 模式违规扫描 | §3.4, §3.6.1 | 扫描目标未检查"Domain 是否绕过 integration.rs 直接调用 Capabilities 内部" |
| R3 | 🟡 High | 缺少 17 条红线（§21）作为扫描标准 | §21 | 扫描目标未完整引用宪法红线（如"禁止非确定性随机源"、"禁止全局 AppError"等） |
| R4 | 🟡 High | 缺少双轨通信违规扫描 | §3.6.2 | 扫描目标未检查"Domain 间是否通过事件传递查询请求（Request-Response 反模式）" |
| R5 | 🔵 Medium | 缺少 Performance Constitution 扫描 | §14 | 扫描目标未检查"是否在高频计算中使用 Reflect"、"缓存是否明确定义失效条件" |
| R6 | 🔵 Medium | 缺少 Architecture Governance 引用 | §19.1 | 扫描应检查新增模块是否符合架构检查清单 |
| R7 | 🔵 Medium | 缺少 CI 门禁合规扫描 | §18.6 | 扫描应验证代码是否满足 CI 门禁的 6 项标准 |

---

## 七、data-architect.md（数据架构师）

### ❌ 冲突/遗漏

| # | 严重度 | 问题 | 宪法条款 | 说明 |
|---|--------|------|----------|------|
| D1 | 🟡 High | 缺少 Capabilities/Domains 架构上下文 | §3.1, §3.3 | 数据架构设计应了解 Core 层的双轴结构（15 Capabilities + 15 Domains），确保 Schema 设计与架构对齐。角色中 Data Laws 未提及此上下文 |
| D2 | 🟡 High | 缺少 Domain integration.rs 数据流引用 | §3.4 | 数据流应遵循"Domain 通过 integration.rs 调用 Capabilities"的模式，但角色中未涉及 |
| D3 | 🟡 High | 缺少 17 条红线引用 | §21 | 数据架构设计应避免触发宪法红线（如"禁止硬编码游戏数值"、"禁止非确定性随机源"） |
| D4 | 🔵 Medium | 缺少 CI 门禁引用 | §18.6 | 宪法要求"配置数据校验全部通过"是 CI 门禁标准，数据架构设计应考虑校验机制 |
| D5 | 🔵 Medium | 缺少 Architecture Governance 引用 | §19.3 | 数据架构变更属于 MINOR 或 MAJOR 级别，需遵循架构版本管理流程 |
| D6 | 🔵 Medium | 缺少双轨通信对数据设计的影响 | §3.6.2 | 数据 Schema 设计应考虑 Domain 间双轨通信模式（写操作走 Event，读操作走 Query API）对数据流的影响 |

---

## 八、汇总统计

### 按严重度统计

| 严重度 | 数量 | 说明 |
|--------|------|------|
| 🔴 Critical | 3 | 架构边界/核心机制冲突 |
| 🟡 High | 27 | 重要规则遗漏 |
| 🔵 Medium | 21 | 完整性引用缺失 |
| **合计** | **51** | |

### 按角色统计

| 角色 | Critical | High | Medium | 合计 |
|------|----------|------|--------|------|
| architect | 2 | 4 | 4 | 10 |
| domain-designer | 0 | 4 | 3 | 7 |
| feature-developer | 0 | 4 | 5 | 9 |
| test-guardian | 0 | 3 | 3 | 6 |
| code-reviewer | 1 | 4 | 4 | 9 |
| refactor-guardian | 0 | 4 | 3 | 7 |
| data-architect | 0 | 3 | 3 | 6 |

### 覆盖率分析

| 宪法维度 | architect | domain-designer | feature-developer | test-guardian | code-reviewer | refactor-guardian | data-architect |
|----------|-----------|-----------------|-------------------|---------------|---------------|-------------------|----------------|
| §1.5 P0 铁则 | ❌ 缺失 | ⚠️ 部分 | ⚠️ 部分 | ⚠️ 部分 | ⚠️ 部分 | ❌ 缺失 | ⚠️ 部分 |
| §3.1 双轴架构 | ❌ 缺失 | ⚠️ 部分 | ⚠️ 部分 | ⚠️ 部分 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |
| §3.4 Domain 结构 | ❌ 缺失 | ⚠️ 部分 | ⚠️ 部分 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |
| §3.5 边界铁则 | ❌ 缺失 | ❌ 缺失 | ⚠️ 部分 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |
| §3.6 交互规范 | ❌ 部分 | ❌ 缺失 | ⚠️ 部分 | ❌ 缺失 | ❌ 部分 | ❌ 缺失 | ❌ 缺失 |
| §6 ECS 宪法 | ⚠️ 部分 | ❌ 缺失 | ⚠️ 部分 | ❌ 缺失 | ⚠️ 部分 | ⚠️ 部分 | ❌ 缺失 |
| §14 性能宪法 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |
| §18 工程质量 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |
| §19 架构治理 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |
| §20 AI 规范 | ⚠️ 部分 | ❌ 缺失 | ⚠️ 部分 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |
| §21 红线 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |

> ✅ = 完整覆盖，⚠️ = 部分覆盖，❌ = 缺失

---

## 九、根因分析

### 系统性遗漏（影响所有角色）

1. **§21 红线禁止事项**：17 条红线是宪法的硬约束底线，但所有 7 个角色均未引用。这导致各角色在执行时可能无意触碰红线而无感知。

2. **§14 性能宪法**：性能例外机制（需 ADR + Profiling 数据）是架构权衡的重要约束，但所有角色均未引用。

3. **§19 架构治理**：新增模块检查清单、架构版本管理、演进流程等是架构演进的规范，但所有角色均未引用。

4. **§18 CI 门禁**：6 项 CI 门禁标准是代码质量的底线，但所有角色均未引用。

### 架构层遗漏（影响 architect、feature-developer、code-reviewer）

5. **双轨通信机制不完整**：宪法 §3.6.2 定义的"写操作→Event，读操作→Query API"双轨制，仅在 feature-developer 的 Step 4 中部分提及，architect 的 ADR 模板和 code-reviewer 的审查清单中均不完整。

6. **Domain integration.rs 模式**：宪法 §3.4 和 §3.6.1 要求每个 Domain 必须有且仅有一个 integration.rs，但仅 feature-developer 提及了此模式。

7. **Capabilities/Domains 边界**：宪法 §3.5 定义的边界铁则，仅 feature-developer 部分提及，其他角色均未引用。

---

## 十、修复建议优先级

### P0（必须修复）

1. **architect.md**：ADR 模板 Communication Design 补充 Query API；验证清单更新为双轴架构术语
2. **code-reviewer.md**：审查清单补充双轨通信审查、Capabilities/Domains 边界审查、Domain integration.rs 审查
3. **所有角色**：补充 §21 红线引用（至少作为参考文档链接）

### P1（应该修复）

4. **architect.md**：补充双轴架构原则、integration.rs 模式、C1/C2/C3 结构引用
5. **domain-designer.md**：补充 15 个 Capabilities 清单引用、双轨通信规范、integration.rs 模式
6. **feature-developer.md**：补充 P0 铁则完整引用、26 条反模式完整引用
7. **refactor-guardian.md**：补充 Capabilities/Domains 边界违规扫描、双轨通信违规扫描

### P2（建议修复）

8. **所有角色**：补充 §14 性能宪法、§18 CI 门禁、§19 架构治理引用
9. **test-guardian.md**：补充 CI 门禁引用、integration.rs 测试要求
10. **data-architect.md**：补充双轴架构上下文、integration.rs 数据流引用

---

## 附录：宪法条款 vs 角色覆盖矩阵

| 宪法条款 | 角色覆盖情况 |
|----------|--------------|
| §1.5 P0 铁则（6条） | architect ❌ / domain-designer ⚠️ / feature-developer ⚠️ / test-guardian ⚠️ / code-reviewer ⚠️ / refactor-guardian ❌ / data-architect ⚠️ |
| §3.1 双轴架构 | architect ❌ / domain-designer ⚠️ / feature-developer ⚠️ / test-guardian ⚠️ / code-reviewer ⚠️ / refactor-guardian ❌ / data-architect ❌ |
| §3.4 Domain integration.rs | architect ❌ / domain-designer ⚠️ / feature-developer ⚠️ / test-guardian ❌ / code-reviewer ❌ / refactor-guardian ❌ / data-architect ❌ |
| §3.5 边界铁则 | architect ❌ / domain-designer ❌ / feature-developer ⚠️ / test-guardian ❌ / code-reviewer ❌ / refactor-guardian ❌ / data-architect ❌ |
| §3.6.2 双轨通信 | architect ⚠️ / domain-designer ❌ / feature-developer ⚠️ / test-guardian ❌ / code-reviewer ⚠️ / refactor-guardian ❌ / data-architect ❌ |
| §6 ECS 宪法 | architect ⚠️ / domain-designer ❌ / feature-developer ⚠️ / test-guardian ❌ / code-reviewer ⚠️ / refactor-guardian ⚠️ / data-architect ❌ |
| §14 性能宪法 | architect ❌ / domain-designer ❌ / feature-developer ❌ / test-guardian ❌ / code-reviewer ❌ / refactor-guardian ❌ / data-architect ❌ |
| §18 CI 门禁 | architect ❌ / domain-designer ❌ / feature-developer ❌ / test-guardian ❌ / code-reviewer ❌ / refactor-guardian ❌ / data-architect ❌ |
| §19 架构治理 | architect ❌ / domain-designer ❌ / feature-developer ❌ / test-guardian ❌ / code-reviewer ❌ / refactor-guardian ❌ / data-architect ❌ |
| §20 AI 规范 | architect ⚠️ / domain-designer ❌ / feature-developer ⚠️ / test-guardian ❌ / code-reviewer ❌ / refactor-guardian ❌ / data-architect ❌ |
| §21 红线 | architect ❌ / domain-designer ❌ / feature-developer ❌ / test-guardian ❌ / code-reviewer ❌ / refactor-guardian ❌ / data-architect ❌ |
