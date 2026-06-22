# CLAUDE.md

This file provides guidance to Claude Code when working in this repository. It's optimized for Claude's workflow — what to do, what to read, what to avoid.

## Build & Test

```bash
cargo build                    # 构建
cargo run                      # 运行
cargo nextest run              # 所有测试（必须用 nextest，不要用 cargo test）
cargo nextest run <test_name>  # 单测
cargo clippy -- -D warnings    # lint 检查
cargo build --features dev     # 开发模式（热重载 + dev tools）
```

## Architecture (3-Second Summary)

```
shared/ ← core/ ← infra/     # 依赖方向，禁止反向
app/ | content/ | tools/ | modding/  # 横切层，可依赖纵向层
```

- **shared/** — 零业务语义的原子工具：强类型 ID、数学、RNG、错误工具、验证
- **core/capabilities/** — 15 个通用机制：ability, effect, modifier, condition, stacking, trigger, cue, targeting, tag, attribute, aggregator, execution, spec, event, gameplay_context
- **core/domains/** — 15 个业务域：combat, spell, tactical, terrain, faction, reaction, inventory, progression, party, camp_rest, narrative, quest, economy, crafting, summon
- **infra/** — 技术实现：registry, pipeline, replay, save, input, logging
- **app/** — 启动装配（唯一知道所有层的地方，不含业务逻辑）
- **content/** — 配置加载 → 校验 → 注册

### Module Patterns

Capability 结构：
```
ability/
├── foundation/   # types, values, ids, errors（纯类型定义）
├── mechanism/    # 核心逻辑（可选 systems/ 子目录）
├── tests/{unit,invariant,integration,fixtures}/
└── plugin.rs
```

Domain 结构：
```
combat/
├── rules/        # 纯函数业务规则（零 ECS 依赖）
├── systems/      # ECS System
├── integration/  # facade.rs（跨 Capability 集成）+ tests/
├── tests/{unit,invariant,integration,fixtures}/
└── plugin.rs
```

## P0 Red Lines（绝对禁止）

1. 禁止绕过 Effect/Modifier 管线直接修改战斗数值和属性
2. 禁止修改 Definition（配置）数据，Definition 加载后不可变、全局唯一
3. 禁止 `#[cfg(test)] mod tests` 内联测试，测试必须放在 `tests/` 子目录
4. 禁止全局 `AppError` 枚举，每个模块自有 Error 枚举（thiserror）
5. 禁止 Core 层依赖 Infra 层（Infra → Core → Shared 单向）
6. 禁止 Domain 间直接引用——只通过 Event 通信

## P0 Mandatory Rules（必须遵守）

1. **Feature First** — 按业务领域组织代码，不要按技术类型拆全局目录
2. **Replay First** — 所有核心战斗逻辑必须可确定性重放，禁止不可控随机源
3. **Localization First** — 用户可见文本必须用 `LocalizationKey`，禁止硬编码；Def 只存 `name_key`/`desc_key`；Replay/Event/BattleLog 只存 Key + 参数
4. **Rule/Content 分离** — 玩法规则下沉 `domain/rules/`，数值配置归 `content/`
5. **Definition/Instance 分离** — 配置全局不可变，运行时状态独立不写回
6. **分领域错误枚举** — 每个模块自有 Error，用 `thiserror` 派生，禁止全局 AppError
7. **四级通信** — 通信优先级：Hook > Trigger > Observer > Message（按耦合度从低到高）

## What To Read For What Task

| 任务 | 先读什么 |
|------|---------|
| 架构决策、模块边界划分 | `docs/01-architecture/README.md` + 相关 ADR |
| 领域建模、业务规则 | `docs/02-domain/README.md` + 对应领域规则文件 |
| Def Schema、配置定义 | `docs/03-content/README.md` + 对应 Def 定义 |
| 数据 Schema、Save/Replay 兼容 | `docs/04-data/README.md` + 对应 Schema 文件 |
| UI/表现层架构 | `docs/06-ui/README.md` + ADR-055 |
| 写测试 | `docs/05-testing/test-spec.md`（四层测试规则） |
| Bug 修复 | 先写失败测试 → 修复 → 验证 |
| 不确定架构约束 | `.trae/rules/架构规则.md` + `.trae/rules/AI协作规则.md` |

大文档太长不用通读——按需查找具体规则。如有疑问但找不到对应条款，问用户。

## Tool Priority

```
CodeGraph → Repomix → Context7 → Git → Filesystem
```
除非内置工具更好，否则尽可能用下面工具
- 查已索引的信息用 CodeGraph/Repomix）
- 查外部文档用 Context7
- 用 git 看 git 历史
- 用 Filesystem 直接读文件

## Agent Delegation

这个项目有 9 个专用 Agent（3 级分治，详见 AGENTS.md），遇到下面情况优先派 agent 而非自己干：

**Tier S — 架构委员会（战略层）**
- **架构设计/系统集成** → `@architect`（输出 ADR，协调各架构师输入）
- **领域建模** → `@domain-designer`（输出领域规则文档，不讨论实现）
- **数据 Schema** → `@data-architect`（确保 Replay/Save 兼容）
- **内容架构（Def/Registry/Validation）** → `@content-architect`（确保配置可加载可校验）
- **UI/表现层架构** → `@presentation-architect`（设计 Projection/ViewModel/Widget）

**Tier A — 工程委员会（治理层）**
- **代码审查** → `@code-reviewer`（只审查不修改）
- **测试编写/审查** → `@test-guardian`（领域规则优先）

**Tier B — 执行层**
- **功能实现** → `@feature-developer`（按架构编码，不写测试）
- **技术债扫描** → `@refactor-guardian`（优先删代码而非加封装）

简单任务自己干，复杂或跨领域的任务派 agent。

## Testing Rules

- 用 `cargo nextest run`，不用 `cargo test`
- 测试目录结构：`tests/{unit,invariant,integration,fixtures}/`
- 禁止 `#[cfg(test)] mod tests` 内联测试
- 根 `tests/` 只放跨域/E2E 测试
- Bug 修复流程：先写失败测试 → 修复 → 验证

## Memory (Engramory — curated file-based long-term memory)

记忆目录：`.claude/memory/`（索引：`MEMORY.md`），随代码版本控制。

- **任务开始**，读 `MEMORY.md`（一行一条），只打开 hook 相关的详情文件。把回忆到的内容当作可能过时的背景——执行前先验证文件/标志/版本。
- **学到持久性内容**值得未来会话记住时：确认不在仓库/git/`CLAUDE.md` 里（别重复已有信息），也不是密钥值；搜索索引，**更新已有笔记**而非重复；否则写一个原子 markdown 文件（一个事实）带 frontmatter：`name` / `description`（精炼的 one-line hook）/ `type`（`user | feedback | project | reference`）/ `created` + `updated`（`YYYY-MM-DD`）。`feedback` 或 `project` 笔记必须带 **`Why:`** 和 **`How to apply:`** 行。在 `MEMORY.md` 加一行指针。**发现错的就删除**。
- **绝不**把凭据/密钥/令牌/cookie/恢复码写进记忆——只记录 *在哪里* 存着。
- 保持 `MEMORY.md` 小（宿主编译只加载前 200 行 / 25 KB）。超了就要压缩：指针化过长的行、合并重复、归档冷笔记。

完整协议 & 理由：`.claude/skills/engramory/SKILL.md`。

## Key Documents Quick-Reference

| File | Why |
|------|-----|
| `docs/00-governance/ai-constitution-complete.md` | 总宪法 21 编，最高约束力 |
| `docs/01-architecture/README.md` | 架构总纲 + ADR 索引 |
| `docs/02-domain/README.md` | Capabilities 15 + Domains 15 领域规则索引 |
| `docs/03-content/README.md` | 内容架构索引（Def Schema / Registry / Validation） |
| `docs/04-data/README.md` | 数据架构规范 |
| `docs/05-testing/test-spec.md` | 测试宪法 v4.0 |
| `docs/06-ui/README.md` | UI/表现层架构索引（Projection / ViewModel / Widget） |
| `AGENTS.md` | 9 Agent 角色定义 + 三级分治协作流程 |
| `.trae/rules/` | 15 个编码规则（架构/ECS/错误/日志/审查等） |

Note: `docs/` 下有 `ai_ignore_this_dir/` 目录，除非用户主动提起，否则视为不存在、不读。

## Document Lifecycle Management

After processing content related to docs in `docs/09-planning/`, `docs/10-reviews/`, or `docs/11-refactor/`, I must automatically:

1. **Update status markers** in the relevant doc file:
   - `✅` = completed
   - `❌` = not done / missing
   - `🟡` = in progress / wip
   - Update frontmatter `status:` field if present

2. **Archive when complete**: When ALL items in a doc are `✅`, move the file to its `done/` subdirectory:
   - `docs/09-planning/done/`
   - `docs/10-reviews/done/`
   - `docs/11-refactor/done/`

3. **Update README.md** in the parent directory to reflect the change (list active docs, remove archived ones).

This must be done automatically in the same session — no reminders or prompts needed.
