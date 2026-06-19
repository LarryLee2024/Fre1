# Agent 前置约束审查报告

> 审查时间：2026-06-16
> 审查范围：`.qoder/agents/` 全部 7 个角色
> 审查维度：每个 Agent 在开始工作前，是否明确要求检查/引用上游设计产出

---

## 协作链路（来自 AGENTS.md）

```
@domain-designer（领域规则）→ @data-architect（数据Schema）→ @architect（架构ADR）
    → @feature-developer（编码）→ @test-guardian（测试）→ @code-reviewer（审查）
                                                              ↓
                                                      @refactor-guardian（技术债）
```

每个 Agent 应该明确知道：**我需要哪些上游产出才能开始工作？**

---

## 逐角色审查

### 1. @architect（首席架构师）— ✅ 前置约束完善

**已有约束：**
- Step 1（强制）：先检查 `docs/02-domain/` 目录
- Step 2：检查 `docs/01-architecture/`、`docs/08-decisions/`、`AGENTS.md`、宪法
- 如果涉及新领域 → 建议先调用 @domain-designer

**评价：** 前置约束完整，是所有 Agent 中做得最好的。

---

### 2. @domain-designer（领域建模专家）— ✅ 前置约束完善

**已有约束：**
- 第一步（强制）：先检查 `docs/02-domain/` 目录
- 优先级声明：`architecture > 已有 domain rules > 新领域模型`
- 架构约束段：明确列出双轴架构、15 个 Capabilities、integration.rs、双轨通信

**评价：** 前置约束完整。

---

### 3. @data-architect（数据架构师）— ⚠️ 缺少前置约束

**已有约束：**
- 架构上下文段：列出双轴架构、数据流、双轨通信、红线

**缺失：**
- ❌ 工作流程 Step 1 是"识别所属领域"，但没有要求先检查 `docs/02-domain/` 相关领域规则
- ❌ 没有要求先检查 `docs/04-data/` 已有 Schema（避免重复设计）
- ❌ 没有明确要求先获取 @domain-designer 的领域模型作为输入
- ❌ 没有明确要求先获取 @architect 的架构决策作为约束

**影响：** 可能在不了解领域规则的情况下设计 Schema，导致数据结构与业务规则不一致。

---

### 4. @feature-developer（功能开发专家）— ⚠️ 前置约束不足

**已有约束：**
- 铁律1：发现与 architecture.md 冲突就停止
- 启动条件：理想输入 = ADR + 领域模型 + 测试规范
- 单人开发模式：要求实现前阅读 architecture、domain rules、现有代码

**缺失：**
- ❌ "最低要求"仅为"有明确的功能需求描述"——没有要求 ADR 必须存在
- ❌ "单人开发模式"说"可以从需求描述直接开始"——绕过了架构设计
- ❌ 没有明确要求先获取 @data-architect 的 Schema 设计
- ❌ 没有明确要求先获取 @architect 的 ADR

**影响：** 可能在没有架构决策的情况下直接编码，导致实现与架构不一致。

---

### 5. @test-guardian（测试卫士）— ✅ 前置约束完善

**已有约束：**
- 优先级声明：`domain_rules > architecture > test_spec > existing_code`
- AI Decision Rules：测试失败时强制按 Step 1-4 流程判断
- 引用了 domain rules、architecture、test_spec 作为权威来源

**评价：** 前置约束完整，优先级链清晰。

---

### 6. @code-reviewer（代码审查员）— ⚠️ 前置约束不足

**已有约束：**
- 审查清单引用了 `docs/01-architecture/` 和 `docs/02-domain/`
- 工作流程 Step 2：阅读相关领域规则

**缺失：**
- ❌ 没有要求先了解 ADR（架构决策）——审查时不知道设计意图
- ❌ 没有明确要求先了解 @data-architect 的 Schema 设计——无法审查数据结构合规性
- ❌ 审查清单没有明确要求检查"代码是否符合 ADR 中的 Module Design"

**影响：** 可能在不了解架构意图的情况下审查代码，漏掉设计层面的问题。

---

### 7. @refactor-guardian（技术债扫描专家）— ❌ 完全缺少前置约束

**已有约束：** 无

**缺失：**
- ❌ 没有要求先检查 `docs/01-architecture/` 了解架构边界
- ❌ 没有要求先检查 `docs/02-domain/` 了解领域规则
- ❌ 工作流程直接从"确定扫描范围"开始，没有前置知识要求
- ❌ 没有明确要求了解双轴架构边界——无法判断"Capabilities 是否包含业务规则"

**影响：** 可能在不了解架构边界的情况下扫描，误判或漏判技术债。

---

## 汇总

| 角色 | 前置约束 | 评价 | 需补充 |
|------|----------|------|--------|
| @architect | ✅ 完善 | 强制检查 domain + architecture | 无 |
| @domain-designer | ✅ 完善 | 强制检查 domain + architecture 约束 | 无 |
| @data-architect | ⚠️ 不足 | 有架构上下文但工作流无前置检查 | 补充：先检查已有 Schema + 领域规则 |
| @feature-developer | ⚠️ 不足 | 允许绕过 ADR 直接编码 | 补充：强制要求 ADR 存在 |
| @test-guardian | ✅ 完善 | 优先级链清晰 | 无 |
| @code-reviewer | ⚠️ 不足 | 有审查清单但缺 ADR 引用 | 补充：先了解 ADR 设计意图 |
| @refactor-guardian | ❌ 缺失 | 完全没有前置约束 | 补充：架构/领域规则前置检查 |

---

## 修复建议

### P0（必须修复）

**@feature-developer.md** — 收紧启动条件：
```markdown
## 启动条件

🟥 **强制前置**：开始编码前必须确认以下文档存在且已阅读：
1. `docs/01-architecture/` 相关 ADR（架构决策）
2. `docs/02-domain/` 相关领域规则
3. `docs/04-data/` 相关 Schema 设计（如涉及数据结构）

**最低要求**：有 ADR + 领域规则。
**理想输入**：ADR + 领域模型 + Schema 设计 + 测试规范。

如果 ADR 或领域规则缺失，立即停止并建议调用 @architect 或 @domain-designer。
禁止在没有架构决策的情况下直接编码。
```

### P1（应该修复）

**@data-architect.md** — 工作流程增加前置检查步骤：
```markdown
## 工作流程

### Step 0: 前置检查（强制）
- 检查 `docs/02-domain/` 下相关领域规则
- 检查 `docs/04-data/` 下已有 Schema（避免重复设计）
- 检查 `docs/01-architecture/` 了解架构约束
- 如有 @domain-designer 的领域模型，作为输入参考

### Step 1: 识别所属领域
...
```

**@code-reviewer.md** — 工作流程增加 ADR 了解步骤：
```markdown
当被调用时：

1. **识别审查范围**：确定要审查的文件或变更
2. **阅读 ADR**：检查 `docs/01-architecture/` 下相关 ADR，了解设计意图
3. **阅读相关领域规则**：检查 `docs/02-domain/` 下相关领域的规则文档
4. **阅读相关 Schema**：检查 `docs/04-data/` 下相关数据结构设计
5. **按优先级逐项检查**：...
```

### P2（建议修复）

**@refactor-guardian.md** — 增加前置约束段：
```markdown
## 前置约束（扫描前必须了解）

扫描前必须阅读：
- `docs/01-architecture/` — 了解架构边界和双轴规则
- `docs/02-domain/` — 了解领域规则和不变量
- `docs/00-governance/ai-constitution-complete.md` §21 — 红线清单

不了解架构边界就无法判断"双轴边界突破"等技术债。
```
