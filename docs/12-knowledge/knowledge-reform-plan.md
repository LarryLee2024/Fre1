---
id: 12-knowledge.README
title: Knowledge Management System — v3 Final
status: proposal
owner: claude
created: 2026-06-22
updated: 2026-06-22
tags:
  - knowledge-management
  - repomix
  - memory-system
  - fitness-functions
  - final
---

# Knowledge Management System — v3（终版）

> **靶心**: Claude Code + Repomix + CodeGraph + Auto Memory + 50 万行 Bevy SRPG + 3 年代码生命周期
> **核心矛盾**: 知识无限增长 vs 注意力有限。解决方式不是"记更多"，而是"记对的 + 自动执行"。

---

## 全景图（八层知识治理架构）

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                      Fre Knowledge Governance System                              │
│  ┌──────────────────────────────────────────────────────────────────────────┐    │
│  │  L7  Architecture Recovery  ◀── 每季度运行一次                           │    │
│  │  CodeGraph 导出当前依赖图 vs ADR 预期依赖图对比 → 发现漂移 → 报警/修正    │    │
│  └──────────────────────────┬───────────────────────────────────────────────┘    │
│                             │ 触发修正                                            │
│                             ▼                                                    │
│  ┌──────────────────────────────────────────────────────────────────────────┐    │
│  │  L6  MEMORY  ── 跨会话自动加载（≤ 43 文件，超过需清理）                    │    │
│  │                                                                          │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐   │    │
│  │  │  user/   │ │feedback/ │ │ project/ │ │decision/ │ │ reference/   │   │    │
│  │  │  ≤5 文件 │ │ ≤10 文件 │ │  ≤3 文件 │ │ ≤20 文件 │ │  ≤5 文件     │   │    │
│  │  │  个人背景│ │ 偏好/反馈 │ │ 长期约束 │ │ 小型决策 │ │ 外部资源     │   │    │
│  │  └──────────┘ └──────────┘ └──────────┘ └─────┬────┘ └──────────────┘   │    │
│  │                                               │ 被引用>3次 或            │    │
│  │                                               │ 影响 3+ Domain           │    │
│  │                                               ▼                          │    │
│  │                                         ┌──────────┐                    │    │
│  │                                         │   ADR   │  ◀── 升迁           │    │
│  │                                         └──────────┘                    │    │
│  └──────────────────────────────────────────────────────────────────────────┘    │
│                                                                                  │
│  ┌──────────────────────────────────────────────────────────────────────────┐    │
│  │  L5  SKILLS  ── 按需加载，从 代码+ADR+规则 生成                          │    │
│  │                                                                          │    │
│  │  约束: 单 Skill ≤ 15k token，超过按任务场景拆分                            │    │
│  │  更新: cargo xtask update-skills / 过期检测: cargo xtask check-skills    │    │
│  │  工具: repomix generate_skill                                             │    │
│  │                                                                          │    │
│  │  ┌──────────────────┐ ┌──────────────┐ ┌──────────────┐ ┌────────────┐  │    │
│  │  │ fre-architecture │ │fre-domain-   │ │fre-capabil-  │ │  fre-ui    │  │    │
│  │  │                  │ │rules         │ │ities         │ │            │  │    │
│  │  │ src/core/ + ADR  │ │domains/rules │ │capabilities/ │ │ src/ui/ +  │  │    │
│  │  │ + 宪法 + 规则    │ │ + 02-domain/ │ │ + 对应 ADR   │ │ ADR-055    │  │    │
│  │  └──────────────────┘ └──────────────┘ └──────────────┘ └────────────┘  │    │
│  └──────────────────────────────────────────────────────────────────────────┘    │
│                                                                                  │
│  ┌──────────────────────────────────────────────────────────────────────────┐    │
│  │  L4  MENTAL MODELS  ── 系统连接图（≤ 10 张硬上限）                        │    │
│  │                                                                          │    │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌────────────────┐  │    │
│  │  │ battle-flow  │ │content-pipe- │ │ ui-navigation│ │ save-replay    │  │    │
│  │  │  战斗生命周期 │ │  line 配置   │ │  导航/浮层   │ │  -flow 存档/  │  │    │
│  │  │              │ │  管线        │ │              │ │  回放          │  │    │
│  │  └──────────────┘ └──────────────┘ └──────────────┘ └────────────────┘  │    │
│  │  ┌──────────────┐                              配额: 5/10 已用         │    │
│  │  │effect-modi-  │                              每篇 ≤ 3 页             │    │
│  │  │fier Effect   │                              本质: 跨 20+ 文件       │    │
│  │  │/Modifier     │                              的流程连接图            │    │
│  │  │管线          │                                                     │    │
│  │  └──────────────┘                                                     │    │
│  └──────────────────────────────────────────────────────────────────────────┘    │
│                                                                                  │
│  ┌──────────────────────────────────────────────────────────────────────────┐    │
│  │  L3  ADR  ── 决策日志，永不删除                                           │    │
│  │                                                                          │    │
│  │  ┌────────────────────────────┐  ┌────────────────────────────────┐      │    │
│  │  │  active/ 未实现/演进中      │  │  historical/ 已实现保留原因      │      │    │
│  │  │────────────────────────────│  │────────────────────────────────│      │    │
│  │  │ ADR-050 Game State Machine│  │ ADR-000~063  约 50 个           │      │    │
│  │  │ ADR-064 Camera System    │  │ 价值: 告诉未来 AI "为什么        │      │    │
│  │  │ ADR-065 Map Pipeline     │  │ 长这样，替代方案为什么失败"      │      │    │
│  │  │ ADR-066 SSPEC Standard   │  │                                │      │    │
│  │  └────────────────────────────┘  └────────────────────────────────┘      │    │
│  └──────────────────┬───────────────────────────────────────────────────────┘    │
│                     │ ADR 规则输入 Fitness Functions                             │
│                     ▼                                                            │
│  ┌──────────────────────────────────────────────────────────────────────────┐    │
│  │  L2  FITNESS FUNCTIONS  ── 自动化架构执行（CI 门禁）                       │    │
│  │                                                                          │    │
│  │  ┌─────────────────────────────────────────────────────────────────┐     │    │
│  │  │  通用规则（现成工具）                 │ 项目特定（cargo xtask）  │     │    │
│  │  ├─────────────────────────────────────────────────────────────────┤     │    │
│  │  │  cargo-deny    → 依赖许可证检查       │ check-forbidden-imports │     │    │
│  │  │  cargo-udeps   → 未使用依赖检测       │ check-layer-violation   │     │    │
│  │  │  cargo-geiger  → Unsafe 使用审计      │ check-effect-pipeline   │     │    │
│  │  │  clippy        → 代码质量检查         │ check-skills            │     │    │
│  │  └─────────────────────────────────────────────────────────────────┘     │    │
│  │                                                                          │    │
│  │  铁律: 失败 = CI ❌，除非附 ADR 豁免 ID                                   │    │
│  └──────────────────────────────────────────────────────────────────────────┘    │
│                                                                                  │
│  ┌──────────────────────────────────────────────────────────────────────────┐    │
│  │  L1  RULES  ── 精简参考文档（为什么，不是做了什么）                        │    │
│  │                                                                          │    │
│  │  00-governance/  │ 宪法 + 编码规则                                       │    │
│  │  02-domain/      │ 业务规则（不可代码化的领域约束）                        │    │
│  │  03-content/     │ Content 架构 + Def 设计规则                            │    │
│  │  04-data/        │ Schema 治理 + Data Laws                               │    │
│  │  05-testing/     │ 测试规范 + 策略                                       │    │
│  │  09-planning/    │ 仅活跃计划（已完成 → done/）                           │    │
│  └──────────────────────────────────────────────────────────────────────────┘    │
│                                                                                  │
│  ┌──────────────────────────────────────────────────────────────────────────┐    │
│  │  L0  CODE  ── 源码 = 最高知识密度                                         │    │
│  │                                                                          │    │
│  │  访问路径:                                                                │    │
│  │  codegraph_node(symbol=xxx)      → 符号定义                              │    │
│  │  codegraph_explore(query)        → 流程理解                              │    │
│  │  codegraph_callers(symbol=xxx)   → 调用链                                │    │
│  │  repomix pack + grep_repomix     → 全局搜索                              │    │
│  └──────────────────────────────────────────────────────────────────────────┘    │
│                                                                                  │
│  ┌──────────────────────────────────────────────────────────────────────────┐    │
│  │                        CI 执行闭环                                       │    │
│  │                                                                          │    │
│  │  每次 git push:     Fitness Functions → ❌ 或 ✅                          │    │
│  │  每季度:            Architecture Recovery → 漂移检测报告                   │    │
│  │  Skill 过期检测:    cargo xtask check-skills → CI ❌                      │    │
│  │  ADR 决策升迁:      升迁后更新对应的 Fitness Function                      │    │
│  │                                                                          │    │
│  └──────────────────────────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## 0. 设计哲学

```
Code           → 做了什么（唯一事实源）
ADR            → 为什么这样（决策理由，永不过期）
Rule           → 不能做什么（架构约束，可自动检查）
Memory         → 上下文是什么（个人 + 项目 + 决策）
Mental Model   → 系统怎么连接（跨 20 个文件的流程，≤10 张）
Skill          → 此刻需要什么（按需打包的上下文）
Fitness Check  → 规则还在执行吗（自动化验证闭环）
```

### 0.1 三个闭环

```
知识记录闭环:
  Decision → ADR → Historical ADR → 永远可搜索

知识执行闭环:
  ADR Rule → Fitness Function → CI → 拒绝违规

知识访问闭环:
  Memory → Skill → Mental Model → CodeGraph → 找到答案
```

---

## 1. 六层知识架构

```
┌────────────────────────────────────────────────────────────────────────┐
│  L6: MEMORY —— 跨会话自动记忆，每次加载                                 │
│                                                                        │
│  记录不可从代码推导的稳定事实：                                           │
│  ┌───────────────────────────────────────────────────────────────┐     │
│  │  user/      → "独立游戏开发者"，"Rust 优先，避免过度设计"        │     │
│  │  feedback/  → "查代码先 CodeGraph"，"回复简洁不用 emoji"        │     │
│  │  project/   → ONLY 长期事实："DDD 三层+横切四层"，"不用 anyhow" │     │
│  │              ⚠️ 不记录 Sprint 状态（那是 09-planning/ 的职责）  │     │
│  │  decision/  → 小型设计决策，不够写 ADR 但丢了会重踩              │     │
│  │              → 被引用 >3 次 / 影响多域 → 升迁为 ADR            │     │
│  │  reference/ → 外部资源位置                                      │     │
│  └───────────────────────────────────────────────────────────────┘     │
│  写入: 会话中自动，无需手动操作                                          │
│  读取: 每次会话自动加载                                                  │
│  位置: .claude/projects/.../memory/                                     │
├────────────────────────────────────────────────────────────────────────┤
│  L5: SKILLS —— 工作技能包，按需加载                                      │
│                                                                        │
│  从 代码 + ADR + 宪法 + 设计规则 生成：                                 │
│                                                                        │
│  fre-architecture   │ src/core/ + active ADR + 宪法 + .trae/rules/     │
│  fre-domain-rules   │ src/core/domains/*/rules/ + 02-domain/           │
│  fre-capabilities   │ src/core/capabilities/ + 对应 ADR                │
│  fre-testing        │ docs/05-testing/（测试规范不可代码化）              │
│  fre-ui             │ src/ui/ + docs/06-ui/ + ADR-055                  │
│                                                                        │
│  位置: .claude/skills/<name>/  (repomix generate_skill 生成)            │
│  更新: cargo xtask update-skills（显式命令）                             │
│  过期检测: cargo xtask check-skills（CI 门禁）                           │
├────────────────────────────────────────────────────────────────────────┤
│  L4: MENTAL MODELS —— 系统连接图，≤10 张                                │
│                                                                        │
│  ⚠️ 硬上限 10 个，超过必须合并或删除。                                   │
│  ⚠️ 这不是"概览文档"，是"跨 20+ 个文件的流程连接图"                     │
│                                                                        │
│  docs/mental-models/（当前 5 个，配额剩余 5 个）：                      │
│  ├── battle-flow.md           │ 战斗生命周期：Input→Pipeline→Replay    │
│  ├── content-pipeline.md      │ 配置：RON→Deserialize→Register→Freeze  │
│  ├── ui-navigation.md         │ 导航：ScreenStack→Overlay→Focus        │
│  ├── save-replay-flow.md      │ 持久化：Command→Frame→Recorder/Player  │
│  └── effect-modifier.md       │ 管线：Ability→Effect→Aggregator→Stat   │
│                                                                        │
│  约束: 每篇 ≤ 3 页，只画流程不列 API，不解释"是什么"（那是 CodeGraph 的事）│
├────────────────────────────────────────────────────────────────────────┤
│  L3: ADR —— 决策日志，永不过期                                          │
│                                                                        │
│  ADR 不是"待办事项"，是"为什么长这样"。已实现的 ADR 价值更大——            │
│  它告诉一年后的 AI："以前试过方案 X，失败了，原因是 Y，不要再踩。"        │
│                                                                        │
│  docs/01-architecture/：                                                │
│  ├── active/       ← 未实现 / 仍在演进（当前 4 个）                     │
│  └── historical/   ← 已实现，保留决策原因（~50 个，永不删除）            │
│                                                                        │
│  decision/ → ADR 升迁规则：                                             │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │  触发升迁的条件（任一）：                                       │      │
│  │  1. 同一决策被引用 > 3 次                                     │      │
│  │  2. 影响 3+ Domain / Capability                               │      │
│  │  3. 有人提出替代方案需要正式评估                                │      │
│  │                                                              │      │
│  │  升迁流程：                                                    │      │
│  │  decision/xxx.md → docs/01-architecture/active/ADR-NNN-xxx.md │      │
│  │  → historical/ 中记录 superseded_by: ADR-NNN                 │      │
│  └──────────────────────────────────────────────────────────────┘      │
├────────────────────────────────────────────────────────────────────────┤
│  L2: FITNESS FUNCTIONS —— 自动架构合规检查                              │
│                                                                        │
│  ⚠️ 这是 v3 新增层。没有自动化执行的知识，只是空文。                      │
│                                                                        │
│  当前可实现的检查：                                                     │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │  cargo xtask check-forbidden-imports                         │      │
│  │    → Domain → Domain 直接引用         违反 P0 红线的第 6 条   │      │
│  │    → Core → Infra 反向依赖            违反架构原则             │      │
│  │                                                              │      │
│  │  cargo xtask check-layer-violation                           │      │
│  │    → Capability 引用另一个 Capability 的内部模块              │      │
│  │    → Domain 绕过 integration/ 直接访问 Capability 字段        │      │
│  │                                                              │      │
│  │  cargo xtask check-inline-tests                              │      │
│  │    → 检测 #[cfg(test)] mod tests                             │      │
│  │                                                              │      │
│  │  cargo xtask check-effect-pipeline                           │      │
│  │    → 检测是否有绕过 Effect 直接修改战斗数值的代码              │      │
│  │                                                              │      │
│  │  cargo xtask check-skills                                    │      │
│  │    → Skill 生成时间 vs src/core/ 最后修改时间                 │      │
│  │    → 过期则 CI 红                                            │      │
│  └──────────────────────────────────────────────────────────────┘      │
│                                                                        │
│  运作方式：                                                             │
│  ├── 每个 check 是独立的 Rust 二进制（xtask/ 下）                      │
│  ├── cargo xtask check-all → 全部运行                                  │
│  ├── CI 必跑 check-all，失败 → ❌                                      │
│  └── 允许 #[allow(fitness_violation)] 但必须附 ADR 豁免 ID              │
├────────────────────────────────────────────────────────────────────────┤
│  L1: REFERENCE —— 精简后的参考文档                                       │
│                                                                        │
│  只保留不可从代码推导的规范：                                             │
│  ├── 00-governance/  宪法 + 编码规则（行为规范）                        │
│  ├── 02-domain/      业务规则（不可代码化的领域约束）                    │
│  ├── 03-content/     Content 架构（Def 设计规则）                       │
│  ├── 04-data/        Schema 治理（Data Laws）                          │
│  └── 05-testing/     测试规范（测试策略）                               │
│                                                                        │
│  已删除: 08-knowledge/ 的纯代码重复内容                                  │
│  已归档: 09-planning/done/, 10-reviews/done/, 11-refactor/done/        │
├────────────────────────────────────────────────────────────────────────┤
│  L0: CODE —— 最高知识密度                                               │
│                                                                        │
│  CodeGraph 索引 + Repomix pack 覆盖所有代码查询场景：                    │
│  ├── 找符号定义 → codegraph_node(symbol=xxx)                           │
│  ├── 理解流程 → codegraph_explore(query="how X works")                 │
│  ├── 找调用者 → codegraph_callers(symbol=xxx)                          │
│  └── 全局搜索 → repomix pack + grep_repomix_output                     │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 2. 决策升迁路径（防止 Decision/ADR 分裂）

```
日常对话诞生决策
       │
       ▼
  [decision/ 记忆]
  记录：日期、决策、原因、替代方案被否决
       │
       │ 任一条件触发：
       │  • 被引用 > 3 次
       │  • 影响 3+ Domain
       │  • 有人提出替代方案
       │  • 时间 > 6 个月仍活跃
       ▼
  [ADR 升迁]
  decision/xxx.md → docs/01-architecture/active/ADR-NNN-xxx.md
       │
       ▼
  [historical/ 记录 superseded_by]
  原 decision 标记 ELEVATED，指向新 ADR
       │
       ▼
  [Fitness Function 更新]
  如果该决策可自动化检查 → 新增 check
```

---

## 3. 各层容量上限

```
Memory/
  ├── user/       ≤ 5 文件（越多越记不住你是谁）
  ├── feedback/   ≤ 10 文件（越多越自相矛盾）
  ├── project/    ≤ 3 文件（超过说明你把 Sprint 塞进来了）
  ├── decision/   ≤ 20 文件（超过说明该升迁了）
  └── reference/  ≤ 5 文件（超过说明你在堆链接）

Mental Models/  ≤ 10 文件（硬上限）

ADR/
  ├── active/    ≤ 10 文件（超过说明架构还没稳定）
  └── historical/ 无上限（越多越好）

Fitness Checks/  ≤ 20 个 check（超过说明规则太琐碎）
```

---

## 4. 分阶段实施计划

### Phase 1: Skill 生成（~30min）

```bash
# 从代码 + ADR + 规则 生成 5 个 Skill
repomix generate_skill --directory src/core/ \
  --include "docs/01-architecture/active/**,docs/00-governance/ai-constitution-complete.md,.trae/rules/*.md" \
  --skillName "fre-architecture"

repomix generate_skill --directory src/core/domains/ --skillName "fre-domain-rules"

repomix generate_skill --directory src/core/capabilities/ \
  --include "docs/01-architecture/historical/ADR-01*-**,docs/02-domain/capabilities/**" \
  --skillName "fre-capabilities"

repomix generate_skill --directory docs/05-testing/ --skillName "fre-testing"

repomix generate_skill --directory src/ui/ \
  --include "docs/06-ui/**,docs/01-architecture/historical/ADR-055-*" \
  --skillName "fre-ui"
```

### Phase 2: ADR 重构（~30min）

```
当前: docs/01-architecture/00-foundation/ADR-xxx.md 等
改为:
docs/01-architecture/
├── active/        ← 未实现（ADR-050, 064, 065, 066）
└── historical/    ← 已实现（50+ ADR，保留决策原因）
```

### Phase 3: Mental Model 创建（~2h）

在 `docs/mental-models/` 创建 5 个核心流程模型，每篇 ≤ 3 页。
硬上限 10，当前用 5，剩余 5 个配额用于未来。

### Phase 4: 文档清理（~1h）

- 删除 `08-knowledge/` 中纯代码重复的概览文档
- 归档旧版 bevy 文档（0.1~0.18）
- 删除 `00-governance/` 中的 bevy 示例目录（官方文档更优）
- 确认 `09-planning/` 等已完成文件在 `done/` 中

### Phase 5: Memory 初始化（~30min）

```markdown
memory/
├── MEMORY.md                   索引
├── user-role.md                "独立游戏开发者，Fre SRPG"
├── feedback-codegraph-first.md "查代码优先 CodeGraph"
├── feedback-terse-response.md  "回复简洁，不用 emoji"
├── project-architecture.md     "DDD 三层+横切四层，能力系统三层"
├── project-constraints.md      "不用 anyhow，不用 Service 模式"
├── decision-capability-not-service.md  "→ 待观察是否升迁 ADR"
├── decision-no-anyhow.md              "→ 已升迁 ADR-051"
└── reference-github-issues.md  "Bug/Issue 在 GitHub"
```

### Phase 6: Fitness Functions 搭建（~3h）

创建 `cargo xtask check-*` 命令：

| 命令 | 实现方式 | 优先级 |
|------|---------|--------|
| `check-forbidden-imports` | 解析 AST 检测跨域 import | P0 — 架构破坏最高风险 |
| `check-layer-violation` | 检测模块路径违反依赖方向 | P0 |
| `check-inline-tests` | grep `#[cfg(test)] mod tests` | P0 — 宪法禁止 |
| `check-effect-pipeline` | 检测 HP/MP 直接修改 | P1 — 需领域知识 |
| `check-skills` | 比较 Skill 生成时间 vs 源码修改时间 | P1 |

CI 集成：
```yaml
# .github/workflows/architecture.yml
check-architecture:
  steps:
    - run: cargo xtask check-all
```

### Phase 7: CLAUDE.md 精简（~20min）

保留约 60 行核心：Build/Test 命令、P0 红线和规则、六层架构一句话索引。

---

## 5. 维护规则（全文最短但最重要的章节）

```
1. decision/ 升迁条件：被引用 > 3 次 OR 影响 3+ Domain → 必须升 ADR
2. Mental Models 上限 10，超过强制合并/删除
3. Skill 不自动更新，但 CI 会检查过期
4. Fitness Function 失败 = CI 红，除非附 ADR 豁免
5. project/ 只记长期事实，不记 Sprint 状态
6. ADR 永不删除，永不 done/
```

---

## 6. 与 v2 的差异

| 维度 | v2 | v3 |
|------|----|----|
| 层数 | 5 | **6**（+Fitness Functions） |
| decision ↔ ADR | 无升迁机制 | **3 条件升迁 + superseded_by 追踪** |
| Mental Model 上限 | 无 | **≤ 10 硬上限** |
| Skill 更新 | pre-push hook | **cargo xtask + CI 门禁** |
| project/ 范围 | sprint + 长期混用 | **仅长期稳定事实** |
| 执行闭环 | 无 | **Fitness Function → CI → ❌** |
| 各层容量上限 | 无 | **每层明确上限** |

---

## 7. Evolution Path（5 年演进路线）

以下内容不在当前 Phase 1-7 实施范围内，是项目增长到 50 万行 → 100 万行 / 3 年 → 5 年 时才会触发的进化方向。

### 7.1 Skill 分层治理（触发条件：单 Skill > 15k token）

当 Skill 文件超过 15k token，按**任务场景**而非**模块目录**拆分：

```
当前（按模块分）:
  fre-capabilities → 15 个能力领域全部打包

未来（按任务分，示例）:
  fre-combat-system       → combat/spell/reaction + 相关 ADR
  fre-progression-system  → progression/inventory/party + 相关 ADR
  fre-content-authoring   → content 层配置 + 校验规则
  fre-ui-implementation   → UI 骨架 + Screen 实现模式
```

拆分规则：
- 一个 Skill = 一个**常见开发任务**的完整上下文
- 不以目录边界为 Skill 边界
- 每个 Skill 的 `includePatterns` 显式声明包含范围
- 每次拆分后清理旧 Skill，避免 N 个版本同时存在

### 7.2 Fitness Function 工具化（触发条件：xtask check-* 超过 5 个自定义检查）

```
当前（全部自研）:
  5 个 cargo xtask check-*

未来（现成工具 + xtask 补充）:
  cargo-deny         → 依赖许可 + 禁止特定 crate
  cargo-udeps        → 未使用依赖检测
  cargo-geiger       → Unsafe 使用审计
  clippy             → 代码质量（已有）

  cargo xtask check-forbidden-imports   ← 现成工具无法覆盖
  cargo xtask check-effect-pipeline     ← 项目特定
  cargo xtask check-skills              ← Repomix 特定
```

原则：**通用检查用现成工具，项目特定检查用 xtask。** 不重复造轮子。

### 7.3 Architecture Recovery（触发条件：项目 > 2 年 或 代码 > 30 万行）

#### 问题
所有知识系统都假设"知识是对的"，但代码会随时间漂移。ADR 说 A→B，实际代码已经是 A→C→B。没有 Recovery，整个系统逐渐变成 fiction。

#### 方案

```
CodeGraph → 导出当前真实依赖图
         ↓
   对比
         ↓
ADR / 配置文件定义的预期依赖图
         ↓
   发现差异
    ├── 代码漂了     → 修复代码
    └── ADR 过期了  → 更新 ADR
         ↓
   输出 Current Architecture Report（Repomix 生成）
   供人工 Review
```

#### 实现

```bash
cargo xtask recovery-check
  --extract-graph     # CodeGraph 导出当前依赖图
  --compare-rules     # 对比 .architecture/expected-deps.toml
  --report-drift      # 输出差异报告
  --auto-suggest      # （可选）建议更新方向
```

预期依赖规则定义在 `.architecture/expected-deps.toml`：

```toml
[layer.core]
allowed_imports = ["shared"]
forbidden_imports = ["infra", "app"]

[layer.domain]
allowed_imports = ["core", "shared"]
forbidden_imports = ["infra", "app"]

[communication]
domain_to_domain = "event_only"
capability_to_capability = "facade_only"
```

#### 节奏

| 频率 | 动作 |
|------|------|
| 每次 CI | Fitness Functions（检查未来违规） |
| 每季度 | Architecture Recovery（检测已经发生的漂移） |
| 每年 | 全量 ADR Review + Mental Model 更新 |

---

## 8. 八层全景（最终演化目标）

```
L7  Architecture Recovery  ← 季度漂移检测
L6  Memory                 ← 个人 + 项目长期事实 + 决策
L5  Skills                 ← 按任务场景拆分，≤15k token/个
L4  Mental Models          ← ≤10 张系统连接图
L3  ADR                    ← active + historical，永不删除
L2  Fitness Functions      ← 现成工具 + xtask 补充
L1  Rules                  ← 不可代码化的规范
L0  Code                   ← 源码（CodeGraph + Repomix 索引）
```

当前实施（Phase 1-7）覆盖 L0-L6，L7 Architecture Recovery 作为 2 年后的进化路径预留。

