---
id: 01-architecture.collaboration-model
title: Collaboration Model
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
---

# Collaboration Model — 协作与规模化架构

Version: 1.0
Status: Proposed

本文档定义 SRPG 项目在以下场景的协作模型：
- AI 协作开发
- 多人团队协作
- 外包美术团队
- 长期维护与更新

> **优化来源**: `docs/其他/38.md`、`docs/其他/40.md`

---

## 一、AI 协作开发

### AI Agent 角色（现有）

项目已定义 6 个专用 AI Agent（见 `AGENTS.md`）：

| 角色 | 职责 | 输出 |
|------|------|------|
| @architect | 架构设计 | ADR |
| @domain-designer | 领域建模 | 领域文档 |
| @feature-developer | 功能实现 | Rust 代码 |
| @code-reviewer | 代码审查 | 审查报告 |
| @test-guardian | 测试守护 | 测试代码、回放测试 |
| @refactor-guardian | 技术债扫描 | 债务清单 |

### AI 协作流程

```
需求
    ↓
@domain-designer（领域模型）
    ↓ 输出：docs/02-domain/
@architect（ADR 架构设计）
    ↓ 输出：docs/08-decisions/
@feature-developer（代码实现）
    ↓ 输出：src/
@test-guardian（测试审查）
    ↓ 输出：tests/
@code-reviewer（代码审查）
    ↓ 输出：docs/reviews/
@refactor-guardian（技术债扫描）
    ↓ 输出：docs/refactor/
```

### AI 协作关键约束

🟥 **宪法约束（与 AGENTS.md 保持一致）**：
1. 🟥 AI Agent 不得违反 `architecture.md` 的任何禁止项
2. 🟥 AI Agent 不得越权跨环节作业（宪法 §21.3.1）
3. 🟩 所有 AI 输出必须符合自检清单（宪法 §21.2）
4. 🟩 AI 修改必须写日志，关键地方必须写日志（宪法 §13.1.2）

### 6-Agent 流水线边界守护

> **优化来源**: `docs/其他/40.md` — 严格边界防止"上下文污染"和"幻觉蔓延"，让 AI 像流水线工人一样在各自工位上干活。

🟥 **绝对边界**：
- @architect **永远不写代码**，只输出 ADR 和架构文档
- @feature-developer **永远不重新设计架构**，只按 ADR 实现
- @code-reviewer **只提意见不直接改代码**
- @test-guardian **只写测试不改业务逻辑**
- @refactor-guardian **只输出债务清单不直接重构**

**冲突仲裁**：
- AI Agent 与人类产生分歧时，人类拥有最终 Override（覆盖）权
- 人类 Override AI 决策时，必须在 `docs/08-decisions/` 中记录一条"破例 ADR"，说明为什么绕过 AI

**死循环保护**：
- 当 @feature-developer 和 @code-reviewer 陷入互相驳回的死循环时，人类在第 3 次驳回时强制介入

### AI Agent 交接协议（Handoff Protocol）

> **优化来源**: `docs/其他/38.md` — Agent 之间传递上下文的标准协议，防止漏读或误解导致代码偏离架构设计。

每个 Agent 完成任务后，必须更新标准化交接文件：

```text
docs/handoff/current_task.md
```

**交接文件模板**：

```markdown
# 当前任务交接

## 任务信息
- 任务名称: [具体任务]
- 执行 Agent: @[agent_name]
- 完成时间: [timestamp]
- 关联 ADR: [adr-xxx.md]

## 已完成内容
- [ ] [具体完成项 1]
- [ ] [具体完成项 2]

## 下游依赖
- @feature-developer 需要实现 [具体模块]
- @test-guardian 需要覆盖 [具体场景]

## 关键约束
- 必须遵循 [具体架构约束]
- 禁止 [具体禁止项]

## 决策记录
- 选择了方案 A 而非方案 B，原因：[具体原因]
```

**强制规则**：
- 下一个 Agent 启动时，必须首先读取 `docs/handoff/current_task.md`
- 如果是跨 Agent 协作，PR Description 模板强制要求填写 Handoff 信息

### 架构文档维护

| 文档 | 优先级 | 维护者 |
|------|--------|--------|
| `docs/01-architecture/README.md` | 最高 | @architect |
| `docs/其他/30.md` | 参考 | 人类 |
| `docs/01-architecture/*.md` | 高 | @architect |
| `docs/08-decisions/*.md` | 高 | @architect |
| `docs/02-domain/*.md` | 高 | @domain-designer |
| `docs/coding_rules.md` | 高 | @code-reviewer |
| `docs/testing/*.md` | 高 | @test-guardian |

---

## 二、多人团队协作

### 分支策略

```
main                    # 稳定发布分支
├── develop             # 开发主分支
│   ├── feature/xxx     # 功能分支
│   ├── bugfix/xxx      # 修复分支
│   └── refactor/xxx    # 重构分支
├── release/v0.x        # 发布分支
└── hotfix/xxx          # 紧急修复分支
```

### 目录所有权

> **注意**：以下目录结构对应七层架构迁移目标（参见 `docs/01-architecture/migration-roadmap.md`）。当前项目仍为 Feature First 顶层结构，迁移完成后此表生效。

| 目录 | 负责人 | 审批要求 |
|------|--------|---------|
| `src/core/` | 主程序员 | 所有 PR 需架构师审批 |
| `src/shared/` | 主程序员 | 所有 PR 需架构师审批 |
| `src/infrastructure/` | 基础设施工程师 | 基础设施 PR 自审 |
| `src/content/` | 策划程序员 | 内容 PR 策划自审 |
| `src/modding/` | MOD 工程师 | MOD PR 自审 |
| `src/ui/` | UI 工程师 | UI PR 自审 |
| `content/` | 策划团队 | 内容 PR 自审 |
| `assets/art/` | 美术团队 | 美术 PR 自审 |
| `assets/audio/` | 音频团队 | 音频 PR 自审 |

### 代码审查规范

**必须审查**：
- 所有 `src/core/` 的变更
- 所有 `src/shared/` 的变更
- 所有 `docs/01-architecture/README.md` 的变更
- 所有 `docs/08-decisions/` 的新增

**自审通过**：
- `content/` 的 RON 文件变更
- `assets/` 的美术资源变更
- `tests/` 的测试新增

---

## 三、外包美术团队协作

### 外包协作模型

```
┌────────────────┐       ┌────────────────┐
│  程序团队       │       │  美术外包团队    │
│                │       │                │
│  src/          │       │  assets/art/   │
│  content/      │       │  assets/audio/ │
│  docs/         │       │  assets/ui/    │
└────────────────┘       └────────────────┘
          │                       │
          │    共享仓库            │
          └───────┬───────────────┘
                  │
            ┌─────▼─────┐
            │   main     │
            └───────────┘
```

### 外包美术团队权限

```
可修改：
✅ assets/art/
✅ assets/audio/
✅ assets/ui/
✅ assets/particles/
✅ assets/shaders/

绝对禁止修改：
❌ src/
❌ content/
❌ assets/definitions/
❌ assets/rules/
❌ assets/maps/（地图数据，非地图图片）
❌ Cargo.toml
```

### 美术资源准入标准

1. **命名规范**：`snake_case`
2. **格式规范**：PNG（Sprite）、OGG（音频）、WAV（音效）
3. **尺寸规范**：2的幂次方，不超过 4096x4096
4. **透明度规范**：Sprite 必须使用 Alpha 通道
5. **色彩空间**：sRGB
6. **文件大小**：单文件不超过 5MB
7. **目录结构**：按角色/类型组织
8. **引用完整**：所有变体必须齐全（idle、attack、hurt、dead）

### 美术规格文档

每个角色需要提供美术规格文档：

```markdown
# 角色美术规格：骑士 (knight)

## Sprite 规格
- 尺寸：64x64 per frame
- 帧数：idle 4帧, move 6帧, attack 8帧, hurt 4帧, dead 6帧
- 格式：PNG with Alpha
- 色彩空间：sRGB

## 头像规格
- 尺寸：256x256
- 格式：PNG with Alpha
- 状态：normal, damaged

## 缩略图规格
- 尺寸：32x32
- 格式：PNG with Alpha
```

### 美术团队分支策略

> **优化来源**: `docs/其他/38.md` — 分支隔离支持多美术并行工作，且不干扰主分支代码。

```
main                        # 程序主分支
├── art/{artist_name}       # 各美术人员的分支
│   ├── art/alice/          # Alice 的分支
│   ├── art/bob/            # Bob 的分支
│   └── art/carol/          # Carol 的分支
```

**分支规则**：
- 每个美术人员在自己的 `art/{artist_name}` 分支上工作
- 分支合并周期：每周固定合并一次，或按需合并
- 多美术修改同一角色资源时，通过 Git 合并解决冲突
- 美术资源迭代使用 Git tag 管理版本（如 `art/knight/v1.2`）

### 美术资源审批流程

> **优化来源**: `docs/其他/38.md` — PR 提交→自动校验→审查→合并，形成完整管控链。

```
美术提交 PR（从 art/{artist_name} 分支）
    ↓
scripts/asset_pipeline/validate_assets.py  # 自动校验
    ↓
scripts/asset_pipeline/validate_references.py  # 引用完整性校验
    ↓
校验通过？
    ├── 否 → 通知美术修改（报错信息包含：文件路径 + 违规原因 + 修正建议）
    └── 是 → 代码审查
              ↓
         合并到 main
```

### 美术资源验收 Checklist

> **优化来源**: `docs/其他/38.md`

- [ ] 文件格式合规（PNG、OGG、WAV）
- [ ] 命名规范合规（snake_case、单数名词、无空格）
- [ ] 图片尺寸为 2 的幂次方
- [ ] 文件大小不超过阈值（5MB）
- [ ] Sprite 使用 Alpha 通道
- [ ] 色彩空间为 sRGB
- [ ] 目录结构正确（按角色/类型组织）
- [ ] 伴随配置文件齐全
- [ ] 无临时文件（PSD、AI 等）
- [ ] 通过 Data Validator

### 资源回滚机制

> **优化来源**: `docs/其他/38.md`

美术资源合入后发现问题的回滚流程：
1. 在 Git 中 revert 对应的 merge commit
2. 通知相关美术人员
3. 在 Hotfix 分支中修复并重新提交
4. 记录回滚原因到 docs/reviews/

---

## 四、长期维护与更新

### 内容更新流程

```
新增章节流程：
1. 策划编写内容规格
2. 策划编写 content/chapters/new_chapter.ron
3. 策划编写 content/stages/new_stages.ron
4. 美术制作 assets/art/maps/battle_maps/new_map/
5. 程序无需修改任何 Rust 代码
6. 测试验证
7. 发布
```

🟥 **如果需要修改 Rust 代码才能新增章节，说明架构有问题。**

### Hotfix 流程

🟥 **Bug 修复必须遵循宪法 §18.1.2 测试优先原则**：

```
发现 Bug：
1. @test-guardian 编写重现测试（🔴 宪法要求：先写重现测试，再修复）
2. @feature-developer 修复 Bug（重现测试必须先失败，修复后通过）
3. @code-reviewer 审查修复
4. @test-guardian 确认重现测试通过 + 回归测试无破坏
5. 发布 Hotfix
```

### 版本管理

```
游戏版本：遵循语义版本
MAJOR.MINOR.PATCH

MAJOR：破坏性更新（存档不兼容）
MINOR：新增内容（新章节、新角色）
PATCH：Bug 修复

MOD API 版本：独立版本号
MOD_API_VERSION = "1"
```

---

## 五、文档维护规范

### 文档与代码同步

| 文档 | 同步方式 | 更新频率 |
|------|---------|---------|
| `architecture.md` | 架构变更时更新 | 每次架构修改 |
| `architecture/*.md` | 架构变更时更新 | 每次架构修改 |
| `adr/*.md` | ADR 产生时新增 | 每次架构决策 |
| `domain/*.md` | 领域变更时更新 | 每次领域修改 |
| `coding_rules.md` | 规范变更时更新 | 每次规范修改 |
| `project-structure.md` | 目录变更时更新 | 每次目录调整 |

### 文档 Review

- 🟩 所有文档变更需要 PR Review
- 🟩 架构文档变更需要 @architect 审批
- 🟩 领域文档变更需要 @domain-designer 审批
- 🟩 测试文档变更需要 @test-guardian 审批

---

## 六、CI/CD 建议

> **优化来源**: `docs/其他/40.md`

### CI 分级策略

> **优化来源**: `docs/其他/40.md` — 随着项目推进，Bevy 编译时间和回放测试时间会大幅增加，必须分级避免开发者崩溃。

**PR CI（快速）— 控制在 5 分钟内**：
```yaml
on: [pull_request]
jobs:
  quick-check:
    - cargo clippy -- -D warnings    # Lint 检查
    - cargo fmt --check               # 格式检查
    - cargo test --lib                # 受影响的单元测试
    - python scripts/asset_pipeline/validate_assets.py  # 资源校验
```

**Merge CI（深度）— 合入 develop 时触发**：
```yaml
on:
  push:
    branches: [develop]
jobs:
  full-check:
    - cargo test                      # 全量测试（含 Replay 测试）
    - python scripts/asset_pipeline/validate_assets.py  # 全量资源校验
    - python scripts/validation/validate_content.py     # 配置校验
    - MOD 兼容性校验
```

**Nightly CI（每晚）— 性能与稳定性**：
```yaml
on:
  schedule:
    - cron: '0 2 * * *'  # 每天凌晨 2 点
jobs:
  nightly:
    - cargo bench                      # 全量性能基准测试
    - 内存泄漏检测
    - 多语言覆盖率检查
```

### 自动化检查

```yaml
# .github/workflows/ci.yml 建议
on: [push, pull_request]
jobs:
  check:
    - cargo build                    # 编译检查
    - cargo test                      # 测试检查
    - cargo clippy -- -D warnings    # Lint 检查
    - cargo fmt --check               # 格式检查
  
  asset_check:
    - python scripts/asset_pipeline/validate_assets.py  # 资源校验
  
  content_check:
    - python scripts/validation/validate_content.py     # 配置校验
```

### 反仓库膨胀策略

> **优化来源**: `docs/其他/40.md` — 外包和程序在同一个 Git 仓库会导致 Git Bloat，拖慢所有人的 clone/pull 速度。

**方案 A（推荐）：美术使用独立仓库**
- 美术使用独立的 Git 仓库（强制开启 Git LFS）
- 主仓库通过 Git Submodule 或 CI/CD 脚本（如 rsync/S3 同步）自动拉取美术资源到 `assets/` 目录
- 优势：物理隔离，彻底解决仓库膨胀问题

**方案 B（妥协）：同仓库 + 强制 LFS**
- 强制全员开启 Git LFS
- 在 Git Hook 中限制单次 Push 的文件大小和类型
- 防止外包误传未压缩的 PSD/源文件

**校验脚本执行时机**：
- PR 提交时自动触发
- 每日定时校验（Nightly CI）
- 校验失败时推送详细报错信息到美术沟通群

### 发布检查

```yaml
# .github/workflows/release.yml 建议
on: [tag]
jobs:
  release:
    - cargo build --release
    - cargo test --release
    - 内容完整性校验
    - 资源完整性校验
    - MOD 兼容性校验
    - 多语言覆盖率检查
```

---

## 七、知识管理

### 知识库结构

```
docs/
├── architecture.md           # 最高架构规范（所有开发者的必读书）
├── architecture/              # 详细架构文档
├── adr/                       # 架构决策记录
├── domain/                    # 领域规则文档
├── coding_rules.md            # 编码规范
├── AI开发宪法.md              # AI 开发宪法
├── testing/                   # 测试规范
├── reviews/                   # 代码审查记录
├── refactor/                  # 技术债记录
├── planning/                  # 计划文档
└── 其他/                      # 参考材料
```

### 新人 Onboarding 流程

1. 阅读 `architecture.md`（最高优先级）
2. 阅读 `AI开发宪法.md`（编码规范）
3. 阅读 `architecture/project-structure.md`（目录结构）
4. 阅读 `architecture/layer-contracts.md`（层边界）
5. 理解七层架构（App/Core/Shared/Infra/Content/Modding/Tools）
6. 运行项目，通过测试
7. 尝试修改 `content/` 中的一个 RON 文件（验证数据驱动）

### AI Agent Onboarding

每次 AI Agent 开始工作时：
1. 读取 `AGENTS.md`（角色定义）
2. 读取 `architecture.md`（架构规范）
3. 读取 `coding_rules.md`（编码规范）
4. 读取相关 `docs/08-decisions/`（最近决策）
5. 读取 `docs/02-domain/`（领域规则）

### AI Agent 异常处理与降级

> **优化来源**: `docs/其他/40.md`

**AI Agent 失败重试协议**：
- AI Agent 输出不符合自检清单时，自动触发重生成（最多 3 次）
- 3 次失败后，标记为"需要人类介入"，暂停流水线

**上下文超限处理**：
- 当 AI 提示 Token 超限时，将大 Feature 拆分为 3 个独立的 PR
- 每个 PR 保持独立的 Handoff 上下文

**工具链故障**：
- 当 AI 无法调用编译器或测试工具时，必须输出"请求人类代理执行"的明确指令
- 🟥 禁止伪造测试结果

**PR 审批超时升级**：
- PR 提交后 24 小时未审批 → 自动通知审批人
- 48 小时未审批 → 升级到项目负责人
- 72 小时未审批 → 制作人介入

### 量化指标体系

> **优化来源**: `docs/其他/40.md`

**代码审查指标**：
| 指标 | 目标值 | 说明 |
|------|--------|------|
| `src/core/` PR 审查通过率 | ≥ 90% | 一次通过率 |
| 单次审查平均耗时 | ≤ 4 小时 | 核心代码审查 |
| 架构违规发现率 | 100% | 所有违规必须被捕获 |

**美术资源指标**：
| 指标 | 目标值 | 说明 |
|------|--------|------|
| 美术资源一次通过率 | ≥ 80% | 自动校验 + 人工审查 |
| 单文件大小超标率 | ≤ 5% | 超过 5MB 的文件占比 |

**AI Agent 指标**：
| 指标 | 目标值 | 说明 |
|------|--------|------|
| AI 自检通过率 | ≥ 95% | 输出符合自检清单 |
| 架构规范符合率 | 100% | AI 代码必须 100% 符合架构 |
| 死循环发生率 | ≤ 2% | 需要人类介入的比例 |

**技术债管理**：
- 债务清单需补充「优先级（高/中/低）」「修复时限」「责任人」
- 🟩 避免债务堆积，每月 review 一次
- 🟩 架构每 3 个月进行一次复盘和调整（宪法 §17.0.8），重点清理过度设计和无用代码