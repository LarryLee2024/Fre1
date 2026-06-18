# 文档冲突评审报告

> 评审人: @architect + @feature-developer + @test-guardian + @data-architect（多角色联合评审）
> 日期: 2026-06-17
> 范围: 全部 `.md` 文档交叉审查
> 输入: `.mimocode/plans/1781693055697-mighty-squid.md`（审计计划）
> 最后更新: 2026-06-17（全部修复完成）

---

## 评审摘要

| 严重程度 | 计划提出 | 确认存在 | 不存在 | 已处理 | 待处理 |
|----------|----------|----------|--------|--------|--------|
| CRITICAL | 4 | 4 | 0 | ✅ 4 | 0 |
| HIGH | 4 | 4 | 0 | ✅ 4 | 0 |
| MEDIUM | 5 | 5 | 0 | ✅ 5 | 0 |
| LOW | 3 | 2 | 1 | ✅ 2 | 0 |
| **合计** | **16** | **15** | **1** | **✅ 15** | **0** |

---

## CRITICAL — 4 处全部确认

### C1: `integration.rs` vs `integration/` — 架构总纲自相矛盾

**状态**: ✅ 已修复

**证据**:
- `docs/01-architecture/README.md:191` 写 `integration.rs`（单文件）
- `docs/01-architecture/README.md:453` 写 `integration/`（目录）
- 同一文件内矛盾。宪法/agent prompts/代码均已是 `integration/`

**处理人**: @architect
**修复**: L191 改为 `integration/`，与 L453 统一

---

### C2: 数据层模型不一致 — data-architect 用了错误的四层

**状态**: ✅ 已修复

**证据**:
- `docs/04-data/README.md:58-62` 定义: Definition / **Spec** / Instance / Persistence
- `.qoder/agents/data-architect.md:41-44` 用: Definition / Instance / **Runtime** / Persistence
- `data-architect.md:224-227` 和 `249` 也用 Runtime
- data-architect 不知道 Spec 层，会设计出错误的数据架构

**处理人**: @data-architect
**修复**: data-architect.md 中所有 `Runtime` → `Spec`，更新四层描述和检查清单

---

### C3: data-architect 域范围缺失 6 个域

**状态**: ✅ 已修复

**证据**:
- `data-architect.md:199-218` 列出 13 个域（Core 10 + Infra 3）
- 实际 `docs/04-data/` 有 15 个 capabilities schema + 4 个 infrastructure schema = 19 个
- 缺失: GameplayContext, Spec, Condition, Event, Input

**处理人**: @data-architect
**修复**: 更新 Domain Ownership 列表，补充缺失的 5 个域

---

### C4: 优先级层级冲突 — domain-designer 与 test-guardian 优先级相反

**状态**: ✅ 已修复

**证据**:
- `.qoder/agents/domain-designer.md:26`: `架构 > 领域规则`
- `.qoder/agents/test-guardian.md:18-22`: `领域规则 > 架构`
- `docs/05-testing/test-spec.md:61`: `领域规则 > 架构`
- 宪法明确领域规则为最高准则
- 两个 agent 在冲突时会做出相反决策

**处理人**: @architect（需统一优先级标准）
**修复**: domain-designer.md 中 `架构 > 领域规则` → `领域规则 > 架构`，与宪法和 test-guardian 一致

---

## HIGH — 4 处全部确认

### H1: 幽灵文件引用 — 多处引用不存在的文件

**状态**: ✅ 已修复

**证据**:
- `docs/01-architecture/README.md:19,46,649` 引用 `docs/00-governance/Fre项目架构设计.md`（不存在）
- `docs/00-governance/coding-rules.md:26,402` 引用 `architecture.md`（不存在）
- `docs/05-testing/test-spec.md:54,55,61,69,82,345,346,382` 引用 `architecture.md` 和 `domain_rules.md`（不存在）

**处理人**: @feature-developer
**修复**:
- `Fre项目架构设计.md` → 删除引用或替换为 `docs/01-architecture/README.md`
- `architecture.md` → `docs/01-architecture/README.md`
- `domain_rules.md` → `docs/02-domain/README.md`

---

### H2: AI Self Check 输出要求矛盾

**状态**: ✅ 已修复

**证据**:
- `docs/00-governance/coding-rules.md:474-491` 要求在代码中输出自检结果块
- 宪法 L1353-1356: "不要求在生成的代码中输出自检结果"
- `.trae/rules/AI协作规则.md:63-66`: 同上
- `.trae/rules/编码规则.md:47-50`: 同上
- `.trae/rules/AI开发宪法.md:77-80`: 同上
- coding-rules.md 是唯一要求代码嵌入的文件，与所有更新版本矛盾

**处理人**: @feature-developer
**修复**: coding-rules.md §19 删除代码嵌入要求，改为"内部参考，不要求输出"

---

### H3: 版本号混乱

**状态**: ✅ 已修复

**证据**:
- `docs/00-governance/ai-constitution-complete.md:1` 自称 **v5.0**
- `docs/00-governance/README.md:20` 说 **v1.6**（指同一文件）
- `.trae/rules/ECS规则.md:5` 说 **v1.6**
- `.trae/rules/架构规则.md:5` 说 **v5.0**
- `docs/05-testing/test-spec.md` 说 **v4.0**
- `docs/00-governance/coding-rules.md:14` 引用"测试宪法**v3.1**"但实际是 v4.0

**处理人**: @feature-developer
**修复**:
- `docs/00-governance/README.md:20` v1.6 → v5.0
- `.trae/rules/ECS规则.md:5` v1.6 → v5.0
- `docs/00-governance/coding-rules.md:14` v3.1 → v4.0

---

### H4: AGENTS.md 文件数量不准确

**状态**: ✅ 已修复

**证据** (对比 AGENTS.md L111-121 与实际):

| 目录 | AGENTS.md 声称 | 实际数量 | 状态 |
|------|---------------|---------|------|
| `00-governance/` | 8 | 7 | ❌ |
| `04-data/` | 1 | 38 | ❌ |
| `05-testing/` | 4 | 3 | ❌ |
| `10-reviews/` | 7 | 11+ | ❌ |

**处理人**: @feature-developer
**修复**: 更新文件数量表

---

## MEDIUM — 5 处全部确认

### M1: Tools 层依赖方向矛盾

**状态**: ✅ 已修复

**证据**:
- 宪法 L96: Tools 依赖 **Core + Shared**
- 架构 README L95: Tools 依赖 **所有层（仅 dev 构建）**
- 架构 README L117: `Tools ──→ 所有层（仅 dev）`

**处理人**: @architect
**修复**: 架构 README L95,117 改为与宪法一致（Core + Shared）

---

### M2: ECS 规则版本号 vs 宪法版本号

**状态**: ✅ 已修复

**证据**:
- `.trae/rules/ECS规则.md:5` 写 "v1.6"
- `.trae/rules/架构规则.md:5` 写 "v5.0"
- 同一宪法的不同章节版本号不同

**处理人**: @feature-developer
**修复**: ECS规则.md v1.6 → v5.0（同 H3）

---

### M3: Component 逻辑禁令措辞不一致

**状态**: ✅ 已修复

**证据**:
- `coding-rules.md:141`: 禁止"**复杂业务逻辑**"（impl 块中）
- 宪法 L679: "**绝对禁止包含任何逻辑**"
- 措辞强度不同：coding-rules 暗示简单逻辑可接受，宪法零容忍

**处理人**: @feature-developer
**修复**: coding-rules.md L141 "复杂业务逻辑" → "任何逻辑"

---

### M4: test-spec.md 状态为 draft 但被标记为 🟩 必须遵守

**状态**: ✅ 已修复

**证据**:
- `docs/05-testing/test-spec.md:4` frontmatter 写 `status: draft`
- `AGENTS.md:83` 标记为 🟩 必须遵守
- draft 文档不应标记为必须遵守

**处理人**: @test-guardian
**修复**: test-spec.md frontmatter `status: draft` → `status: stable`

---

### M5: "运行时三层" 幽灵引用

**状态**: ✅ 已修复

**证据**:
- `docs/01-architecture/README.md:32`: "运行时三层（Domain / Application / Presentation）"
- `docs/01-architecture/README.md:123`: "宪法（`.trae/rules/架构规则.md`）定义的 Domain / Application / Presentation 三层"
- `docs/01-architecture/README.md:633`: "符合三层运行时分离（Domain / Application / Presentation）"
- `.trae/rules/架构规则.md` **未定义**此三层模型（它定义的是 DDD 三层: Shared/Core/Infrastructure）
- "运行时三层" 模型仅存在于架构 README 自身，不在其引用的规则文件中

**处理人**: @architect
**修复**: 修正引用，明确"运行时三层"是架构 README 自定义的视角模型，不是引用自宪法

---

## LOW — 3 处，1 处不存在

### L1: coding-rules.md 与宪法的 "三重分离" 表述差异

**状态**: ✅ 已修复

**证据**:
- AI协作规则: 三重合一表述
- AI开发宪法: 三重分开 + 额外"小单元原则"
- 宪法: 不同的 Top-10 列表
- 核心含义一致，只是粒度和排序不同

**处理人**: @feature-developer
**修复**: 对齐 Top-10 列表，确保核心规则一致

---

### L2: tactical_domain.md L283 仍写 `integration.rs`

**状态**: ✅ 已修复

**证据**:
- `docs/02-domain/domains/tactical_domain.md:283`: "禁止将所有能力域塞入单个 `integration.rs` 文件"
- 同文件 L238,249,282 已用 `integration/`
- 仅这一行遗留旧名

**处理人**: @feature-developer
**修复**: L283 `integration.rs` → `integration/`

---

### L3: governance README 文件数量过时

**状态**: ❌ **不存在（误报）**

**证据**:
- `docs/00-governance/README.md` 列出 6 个文件条目
- 实际目录有 7 个文件（含 README.md 自身）
- README 未声明"共 X 个文件"，只是列出了条目
- 这不是冲突，只是条目不完整

**处理人**: 无需处理
**修复**: 无需修复。如需完善可在 README 中补充遗漏的 `README.md` 条目

---

## 执行计划

### Phase 1: CRITICAL 修复（必须，4 项）

| # | 修复内容 | 处理人 | 涉及文件 |
|---|----------|--------|----------|
| C1 | `integration.rs` → `integration/` | @architect | `docs/01-architecture/README.md:191` |
| C2 | 数据层 Runtime → Spec | @data-architect | `.qoder/agents/data-architect.md:41-44,224-227,249` |
| C3 | 补充 data-architect 域列表 | @data-architect | `.qoder/agents/data-architect.md:199-218` |
| C4 | 优先级统一 | @architect | `.qoder/agents/domain-designer.md:26` |

### Phase 2: HIGH 修复（应该，4 项）

| # | 修复内容 | 处理人 | 涉及文件 |
|---|----------|--------|----------|
| H1 | 幽灵文件引用 | @feature-developer | `docs/01-architecture/README.md`, `docs/00-governance/coding-rules.md`, `docs/05-testing/test-spec.md` |
| H2 | AI Self Check | @feature-developer | `docs/00-governance/coding-rules.md:474-491` |
| H3 | 版本号统一 | @feature-developer | `docs/00-governance/README.md:20`, `.trae/rules/ECS规则.md:5`, `docs/00-governance/coding-rules.md:14` |
| H4 | 文件数量 | @feature-developer | `AGENTS.md:111-121` |

### Phase 3: MEDIUM 修复（可选，5 项）

| # | 修复内容 | 处理人 | 涉及文件 |
|---|----------|--------|----------|
| M1 | Tools 依赖方向 | @architect | `docs/01-architecture/README.md:95,117` |
| M2 | ECS 版本号 | @feature-developer | `.trae/rules/ECS规则.md:5`（同 H3） |
| M3 | Component 措辞 | @feature-developer | `docs/00-governance/coding-rules.md:141` |
| M4 | test-spec 状态 | @test-guardian | `docs/05-testing/test-spec.md:4` |
| M5 | 运行时三层引用 | @architect | `docs/01-architecture/README.md:32,123,633` |

### Phase 4: LOW 修复（可选，2 项）

| # | 修复内容 | 处理人 | 涉及文件 |
|---|----------|--------|----------|
| L1 | Top-10 对齐 | @feature-developer | `.trae/rules/AI协作规则.md`, `.trae/rules/AI开发宪法.md` |
| L2 | tactical_domain 旧名 | @feature-developer | `docs/02-domain/domains/tactical_domain.md:283` |

---

## 验证

```bash
# 1. 确认无 integration.rs 引用残留（除 L283 外）
grep -rn "integration\.rs" docs/ .trae/rules/ .qoder/agents/ AGENTS.md

# 2. 确认无幽灵文件引用
grep -rn "Fre项目架构设计" docs/
grep -rn "architecture\.md" docs/ .trae/rules/ | grep -v "README"
grep -rn "domain_rules\.md" docs/ .trae/rules/

# 3. 确认 data-architect 使用 Spec 而非 Runtime
grep -n "Runtime" .qoder/agents/data-architect.md

# 4. 确认版本号统一
grep -rn "v1\.6\|v3\.1" docs/ .trae/rules/

# 5. 测试通过
cargo test --lib
```
