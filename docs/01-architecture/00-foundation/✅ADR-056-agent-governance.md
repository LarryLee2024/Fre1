---
id: 01-architecture.00-foundation.ADR-056
title: "ADR-056: 7→9 Agent 升级与三级治理体系"
status: accepted
owner: architect
created: 2026-06-20
tags:
  - architecture
  - governance
  - agent
  - workflow
  - collaboration
  - foundation
---

# ADR-056: 7→9 Agent 升级与三级治理体系

## 状态

**Accepted** — 已被架构委员会接受。

## 背景

项目基于 Bevy 0.19 构建 SRPG，代码规模已达 50 万行级别，预计持续开发 5 年以上。原有 7 个专用 Agent（architect, domain-designer, data-architect, feature-developer, code-reviewer, test-guardian, refactor-guardian）在协作过程中暴露出以下问题：

1. **Content 领域无专属治理** — 15 个业务域的 Def 配置（AbilityDef、EffectDef、BuffDef、ItemDef 等）需要一个专门的架构角色来设计 Def Schema、Registry、Validation 和 Dependency Graph。原有模式中这部分由 architect 兼任，导致 Def 设计深度不足。

2. **UI 层无专属治理** — ADR-055 定义了 UI 表现层架构（Projection/ViewModel/Screen/Widget），但缺少专门的架构角色来维护 UI 层的架构完整性。原有模式中 architect 同时设计 UI 架构，导致 UI 架构文档（docs/06-ui/）缺失。

3. **Architect 角色膨胀** — 单一 architect 同时负责数据流设计、配置架构、UI 架构、系统集成，成为设计瓶颈和单点故障。

4. **协作无分层** — 7 个 Agent 为扁平结构，没有明确的战略/治理/执行分层，决策升级路径不清晰。

5. **Review 维度单一** — 仅有代码审查和测试审查，缺少内容合规审查和 UI 合规审查，导致 Def 和 UI 问题在开发后期才被发现。

为应对这些挑战，项目升级为 9 Agent 三级治理体系。本 ADR 固化此次升级的设计决策和边界定义。

## 引用的领域规则与架构

- `AGENTS.md` — Agent 三级分治体系、协作流程、通用红线
- `CLAUDE.md` §Agent Delegation — Agent 代理指引（9 角色按 tier 组织）
- `.claude/agents/` (9 files) — Claude Code Agent 系统提示词
- `.qoder/agents/` (9 files) — OpenCode/Sisyphus Agent 定义
- `.mimocode/agents/` (9 files) — MiMo Code Agent 定义
- `docs/01-architecture/README.md` §附录 A — 上游输入追溯
- `docs/03-content/README.md` — 内容架构索引（Content Architect 输出目录）
- `docs/06-ui/README.md` — UI/表现层架构索引（Presentation Architect 输出目录）
- `docs/01-architecture/40-cross-cutting/ADR-055-ui-presentation-architecture.md` — UI 表现层架构（触发 Presentation Architect 角色的上游 ADR）
- `docs/00-governance/ai-constitution-complete.md` — 项目宪法（Agent 行为的最高约束）

## 决策

### DR-001: 三级治理体系

引入 Tier S / A / B 三级分治，解决扁平结构中的职权不清问题。

```
Tier S: 架构委员会（Architecture Board）      ── 战略层：定义规则与边界
Tier A: 工程委员会（Engineering Board）        ── 治理层：确保合规与质量
Tier B: 执行层（Implementation）              ── 战术层：按规则交付
```

**权力流**：Tier S 定规则 → Tier A 执行规则 → Tier B 遵守规则

**升级路径**：
- Tier B 遇到设计歧义 → 升级到 Tier A
- Tier A 无法裁决 → 升级到 Tier S
- Tier S 内部争议 → Architect 终审

**角色分配**：

| Tier | 角色 | 职责 |
|------|------|------|
| S | @domain-designer | 领域建模：业务术语、不变量、状态机、流程 |
| S | @data-architect | 数据架构：Schema、数据层划分、Replay/Save 兼容 |
| S | @content-architect | 内容架构：Def Schema、Registry、Validation、Localization |
| S | @presentation-architect | UI 架构：Projection、ViewModel、Screen、Widget、Navigation |
| S | @architect | 系统集成：模块边界、依赖方向、ADR、Plugin 组合 |
| A | @data-architect（双角色） | 治理审查：Schema 变更 / 迁移合规 |
| A | @code-reviewer | 代码审查：编码规范、性能、安全 |
| A | @test-guardian | 测试审查：领域不变量、边界条件 |
| B | @feature-developer | 功能实现：按架构编码，消费 01+02+03+04+06 |
| B | @refactor-guardian | 技术债扫描：六大维度，优先删除而非封装 |

### DR-002: Content Architect 角色

**触发原因**：15 个业务域的 Def 配置（需 11+ 个 Def 类型，详见 docs/03-content/README.md §1.2）成为一个独立的工程领域，需要专职架构角色。

**职责**：
- Def Schema 设计（AbilityDef、EffectDef、BuffDef、ItemDef、QuestDef 等）
- Registry 架构（注册、查找、依赖解析、热重载）
- Validation Pipeline（字段校验、跨 Def 约束、循环依赖检测）
- Dependency Graph（Def 间引用关系、加载顺序）
- Localization Key 组织规范
- 资产目录结构（assets/config/ 下的 RON 文件组织）

**输入**：`docs/02-domain/`（领域规则）+ `docs/04-data/`（数据 Schema）

**输出**：`docs/03-content/`

**下游消费者**：@architect（系统集成）、@feature-developer（代码实现）

### DR-003: Presentation Architect 角色

**触发原因**：ADR-055 定义了完整的 UI 表现层架构（Projection 防火墙 / 单向数据流 / Widget Contract），需要一个专职架构角色来维护 UI 层完整性。

**职责**：
- UI 分层架构（Projection → ViewModel → Screen → Widget）
- 导航系统（Screen Stack、Push/Pop 导航）
- ViewModel 结构设计
- Widget Contract 定义（Props + Events + State）
- 主题令牌系统（语义化颜色、间距、字体）
- UI 状态分层（持久/会话/瞬态）

**输入**：`docs/02-domain/`（领域规则）

**输出**：`docs/06-ui/`

**下游消费者**：@architect（系统集成）、@feature-developer（UI 代码实现）

### DR-004: Architect 角色变更：全域设计师 → 系统集成者

**旧模式**：Architect 是"全能设计师"，直接设计数据 Schema、Def 配置、UI 架构、系统集成。单人瓶颈，非核心领域质量参差不齐。

**新模式**：Architect 是"系统集成者"，消费四个专业架构师的输出进行集成：

| 之前（自己做） | 现在（协调专业架构师） |
|---------------|---------------------|
| 设计数据 Schema | → 消费 @data-architect 的 Schema |
| 设计 Def/配置结构 | → 消费 @content-architect 的 Def 定义 |
| 设计 UI 架构 | → 消费 @presentation-architect 的 UI 设计 |
| 设计领域规则 | → 消费 @domain-designer 的领域规则 |
| **系统集成 + ADR** | ← **核心职责** |

**保留职责**：目录结构设计、Plugin 拆分、ECS 模式设计、事件流设计、状态机设计、系统集成方案、测试架构分层

**移交职责**：
- ~~数据流设计~~ → @data-architect（Schema层面），architect 只做集成层面的数据流决策
- ~~存档架构~~ → @data-architect（持久化策略），architect 只评估对模块边界的影响
- ~~配置架构~~ → @content-architect（Def Schema/Registry），architect 只检查 Plugin 集成
- ~~UI 架构~~ → @presentation-architect（UI分层），architect 只确保 Domain/UI 边界正确

### DR-005: 协作顺序

五个设计阶段必须是固定串行顺序，因为存在输入依赖。两个并行窗口位于集成之后。

```
需求
 │
 ├─→ ① @domain-designer（领域建模）
 │     输出：docs/02-domain/
 │
 ├─→ ② @data-architect（数据架构）
 │     输入：领域规则
 │     输出：docs/04-data/
 │
 ├─→ ③ @content-architect（内容架构）
 │     输入：领域规则 + 数据 Schema
 │     输出：docs/03-content/
 │
 ├─→ ④ @presentation-architect（UI 架构）
 │     输入：领域规则
 │     输出：docs/06-ui/
 │
 └─→ ⑤ @architect（系统集成）
       输入：① Domain + ② Data + ③ Content + ④ Presentation
       输出：docs/01-architecture/（ADR）
          │
          ▼
  @feature-developer（代码实现）
    消费：01 Architecture + 02 Domain + 03 Content + 04 Data + 06 UI
          │
   ┌──────┼──────────────┐
   ▼      ▼              ▼
 @test-guardian      @code-reviewer
 (测试验证)           (代码审查)
   │                   │
   │    @content-architect (内容审查)
   │    @presentation-architect (UI审查)
   │                   │
   └───────┼───────────┘
           ▼
  @refactor-guardian（技术债扫描）
```

**串行原因的依赖分析**：
- ③→②：Content Architect 需要 finalized Schema 设计 Def 结构
- ④→①：Presentation Architect 需要领域规则设计 ViewModel（不需要 Schema/Def）
- ⑤→③④：Architect 需要所有四个专业输出进行集成
- **并行窗口 1**：③ Content + ④ Presentation 可以并行（不同输入依赖）
- **并行窗口 2**：Review 阶段四个维度可以并行

### DR-006: 多维审查体系

**旧模型**：单一代码审查（@code-reviewer）+ 测试审查（@test-guardian）

**新模型**：四维并行审查

| 审查维度 | 负责 Agent | 审查内容 | 时机 |
|---------|-----------|---------|------|
| 代码质量 | @code-reviewer | 编码规范、性能、安全 | 功能完成后 |
| 测试覆盖 | @test-guardian | 领域不变量、边界条件 | 功能完成后 |
| 内容合规 | @content-architect（副角色） | Def 设计合理性、Localization 规范 | 功能完成后 |
| UI 合规 | @presentation-architect（副角色） | ViewModel 正确性、Projection 存在性 | 功能完成后 |

**理由**：内容架构和 UI 架构问题如果在代码审查阶段才发现，需要大量返工。提前审查可以在架构层面拦截问题。

### DR-007: Data Architect 双角色

Data Architect 同时存在于 Tier S（设计者）和 Tier A（审查者）：

- **Tier S 角色**：设计数据 Schema、数据层划分、Replay/Save 兼容
- **Tier A 角色**：审查 Schema 变更和数据迁移的向后兼容性

**理由**：Schema 设计需要深厚的领域知识，无法由独立审查者复制。Data Architect 是唯一有资格审查自己 Schema 的角色。保持同一个体避免上下文切换。

### DR-008: 8 条通用行为红线

所有 Agent（不分角色）必须遵守：

1. 禁止绕过 Effect/Modifier 管线直接修改战斗数值与属性
2. 禁止突破模块边界、违反 ECS 架构模式
3. 禁止修改定义态（Definition）配置数据
4. 禁止超出自身角色职责范围跨环节作业（@feature-developer 严禁写测试）
5. 禁止写过时、不符合最新 Bevy 0.19 版本的代码
6. 数据架构变更必须经过 @data-architect 审查（确保 Replay/Save 兼容）
7. 内容架构变更（Def Schema / Registry / Validation / Localization Key）必须经过 @content-architect 审查
8. UI 架构变更（Projection / ViewModel / Screen / Widget Contract）必须经过 @presentation-architect 审查

## Module Design

### Agent Module Map

| Agent | Tier | Input | Output | Key Doc |
|-------|------|-------|--------|---------|
| @domain-designer | S | 业务需求 | `docs/02-domain/` | 领域规则文档 |
| @data-architect | S+A | 领域规则 | `docs/04-data/` | Schema + Data Laws |
| @content-architect | S | Domain + Data | `docs/03-content/` | Def 定义 + Registry |
| @presentation-architect | S | 领域规则 | `docs/06-ui/` | UI 架构文档 |
| @architect | S | 4 specialists | `docs/01-architecture/` | ADR + 集成方案 |
| @code-reviewer | A | 代码 PR | Review comments | — |
| @test-guardian | A | 功能 | 测试用例 | `docs/05-testing/` |
| @feature-developer | B | 01+02+03+04+06 | `src/` | 执行计划 |
| @refactor-guardian | B | 代码库 | `docs/11-refactor/` | 债务清单 |

### Agent 定义文件

三目录同步维护，共 27 个文件：

| 目录 | 工具 | 文件数 | 格式特点 |
|------|------|--------|---------|
| `.claude/agents/` | Claude Code | 9 | `name` + `description` + `model` + `color` + body |
| `.qoder/agents/` | OpenCode/Sisyphus | 9 | `name` + `tools` frontmatter + body |
| `.mimocode/agents/` | MiMo Code | 9 | `mode: subagent` + `tools` frontmatter + body |

`AGENTS.md` 是三目录的 Single Source of Truth。

### 输入依赖图

```
Feature Developer 消费的全部输入：

┌─────────────────────────────────────────────────────────┐
│                     Feature Developer                     │
├─────────────────────────────────────────────────────────┤
│  01 Architecture   02 Domain   03 Content   04 Data   06 UI  │
│  (模块边界)       (业务规则)   (Def 定义)    (Schema)   (UI 架构)│
└─────────────────────────────────────────────────────────┘
```

## 边界定义

### Agent 权限表

| Agent | 拥有 | 不拥有 |
|-------|------|--------|
| @domain-designer | `docs/02-domain/`、业务术语、不变量、状态机 | 代码实现、Schema 设计、UI 线框 |
| @data-architect | `docs/04-data/`、Schema、Data Laws、Save/Replay | 领域逻辑、Def 配置值 |
| @content-architect | `docs/03-content/`、Def 类型、Registry、Validation | 游戏平衡数值、运行时状态 |
| @presentation-architect | `docs/06-ui/`、Projection、ViewModel、Widget Contract | 具体 Widget 实现、像素级布局 |
| @architect | `docs/01-architecture/`、ADR、模块边界、Plugin 组合 | 具体 Schema、具体 Def 类型、UI 像素 |
| @code-reviewer | PR 代码质量审查 | 写代码、设计决策 |
| @test-guardian | 测试覆盖、领域不变量、回放测试 | Feature 实现、生产代码 |
| @feature-developer | `src/` 实现、执行计划 | 架构决策、测试代码 |
| @refactor-guardian | `docs/11-refactor/` 债务清单 | 架构重构、Feature 实现 |

### Agent 输出目录归属

每个 Agent 只能写入自己的目录（除非经过上游协调）：

| Agent | 写入目录 |
|-------|---------|
| @domain-designer | `docs/02-domain/` |
| @data-architect | `docs/04-data/` |
| @content-architect | `docs/03-content/` |
| @presentation-architect | `docs/06-ui/` |
| @architect | `docs/01-architecture/` |
| @test-guardian | `tests/`（测试代码）、`docs/05-testing/`（测试文档） |
| @feature-developer | `src/`（生产代码） |
| @refactor-guardian | `docs/11-refactor/`（债务清单） |
| @code-reviewer | 无写入权限（只审查不修改） |

## Forbidden

| 行为 | 理由 |
|------|------|
| @feature-developer 写测试代码 | 违反角色分离，测试由 @test-guardian 负责 |
| @feature-developer 改架构 | 绕过 Tier S 权限，架构变更必须有 ADR |
| @code-reviewer 修改代码 | Review 只审查不修改 |
| @domain-designer 写 Rust 代码 | 领域建模是纯设计工作 |
| @content-architect 设置游戏数值 | Content architect 设计 Schema，不设具体值 |
| @presentation-architect 写 UI Widget 代码 | 架构师设计，开发者实现 |
| @architect 直接设计 Schema/Def/UI | Architect 集成不替代，必须消费专业架构师输出 |
| Tier B 直接升级到 Tier S | 必须经 Tier A，除非紧急架构违规 |
| Agent 写入自身输出目录之外 | 每个 Agent 有明确定义的输出目录 |
| 非重要功能跳过四维审查 | 新功能必须经过全部 4 个审查维度 |

## Definition / Instance Design

### Definition（静态 Agent 配置，会话间不变）

- `.claude/agents/*.md` — Claude Code Agent 系统提示词
- `.qoder/agents/*.md` — OpenCode/Sisyphus Agent 定义
- `.mimocode/agents/*.md` — MiMo Code Agent 定义
- `AGENTS.md` — 协作规则、工作流、红线的 Single Source of Truth
- `CLAUDE.md` §Agent Delegation — 代理指引

### Instance（运行时 Agent 调用，每次不同）

- Agent 通过 `actor()`（MiMo Code）或 `task()`（OpenCode/Sisyphus）在开发过程中实例化
- 每次调用 = 一个运行时实例，携带具体的任务 prompt
- Agent 行为由 Definition（系统提示词）+ 任务 prompt 共同决定
- 平行实例运行：MiMo Code 支持并发 `spawn`

### 同步契约

- 三个 Agent 目录（`.claude/` + `.qoder/` + `.mimocode/`）必须保持内容同步
- `AGENTS.md` 是角色描述和协作流程的唯一真实来源
- 角色定义不能相互矛盾（三个目录对同一角色的描述必须语义一致）

## 后果

### 正面

1. **专业化深度** — Content/Presentation Architect 在各自领域比原有 generalist Architect 更深入
2. **瓶颈消除** — Architect 不再为每个 Schema/UI/Def 决策所必需
3. **设计并行** — Content Architect 和 Presentation Architect 可并行工作
4. **清晰升级路径** — Tier S/A/B 三级为冲突解决提供了明确的权力框架
5. **提前拦截问题** — 内容/UI 审查在代码审查前发现问题，降低返工成本
6. **扩展性** — 9 Agent 比 7 Agent 分配更灵活，工作量分布更均匀

### 负面

1. **协调开销增加** — 5 个设计角色需对齐输出后开发者才能开始
2. **Agent 定义维护量** — 3 目录 × 9 文件 = 27 个同步文件
3. **开发者输入增加** — Feature developer 必须消费 5 个输入目录（01+02+03+04+06）而非 3 个
4. **过度专业化风险** — Agent 可能过于狭窄，忽略跨领域问题
5. **学习曲线** — 新成员需理解 9 Agent 的交互模式

### 缓解

1. **固定协作顺序** 防止协调混乱 — 顺序不可协商
2. **并行路径**（Content + Presentation 可同时运行）可减少实际交付时间
3. **AGENTS.md 作为唯一真实来源** 降低 Agent 定义文件同步负担
4. **Architect 作为集成点** 确保跨专业问题在开发前被捕获
5. **升级路径** 防止死锁 — Architect 有最终裁决权

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| **维持 7 Agent（不新增）** | Content 和 UI 复杂度超出单一 architect 治理能力。ADR-055 本身就需要 UI 架构专职所有权 |
| **14 Agent（每个 Domain 一个）** | 过度设计。大多数 Domain 共享通用模式（15 个 capabilities 的实例），Domain 特有 agent 会重复相同架构推理 |
| **4 级体系（加 Operations Tier）** | 运维（部署、监控）尚未在项目范围中。保持扁平直到需要 |
| **Flat 9 Agent（无层级）** | 无权力结构导致决策瘫痪。没有层级则 @feature-developer 不知道听 @code-reviewer 还是 @architect |
| **Data Architect 拆为 S/A 两个独立角色** | 不必要的重复。Data Architect 是唯一有资格审查自己 Schema 的角色。双角色减少上下文切换 |
| **Presentation Architect 放在 Architect 之后** | 会错过 UI 约束早期暴露。Architect 在集成前就需要知道 UI 边界 |
| **Content Architect 放在 Data Architect 之前** | Content Architect 需要 finalized Schema 设计 Def 类型。Content 先于 Data 会导致 Schema-Def 不匹配 |
| **Architect 继续做全域设计师（加 2 个助手）** | 会变成"委员会设计"——Architect 审查 4 个专家输出但做相同工作。清晰拆分职责域更好 |

## 架构合规性自检

- [ ] 符合 Feature First 原则（Agent 按角色组织，非技术类型）
- [ ] 符合三层运行时分离（S=战略，A=治理，B=执行，不可越级）
- [ ] 定义了明确的 Forbidden 事项（10 条禁止项）
- [ ] 引用了上游 Agent 定义和协作规则（AGENTS.md 是所有 Agent 的宪法）
- [ ] 协作顺序符合依赖方向（Domain → Data → Content → Presentation → Architect）
- [ ] 定义了完整的 9 Agent Scope 矩阵（输入/输出/关键文档）
- [ ] 确定了并行窗口（Content + Presentation 可并行；4 维审查可并行）
- [ ] 确定了升级路径（B → A → S → Architect 终审）
- [ ] Agent 定义文件三目录同步（.claude/agents/ = .qoder/agents/ = .mimocode/agents/）
- [ ] 符合数据架构规范（Data Schema 由 @data-architect 设计，非 @architect）
- [ ] 符合内容架构规范（Def Schema 由 @content-architect 设计，非 @feature-developer）
- [ ] 符合 UI 架构规范（Projection/ViewModel/Widget 由 @presentation-architect 设计，非 @architect）
