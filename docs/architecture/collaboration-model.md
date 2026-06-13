# Collaboration Model — 协作与规模化架构

Version: 1.0
Status: Proposed

本文档定义 SRPG 项目在以下场景的协作模型：
- AI 协作开发
- 多人团队协作
- 外包美术团队
- 长期维护与更新

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
    ↓ 输出：docs/domain/
@architect（ADR 架构设计）
    ↓ 输出：docs/adr/
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

1. 🟥 AI Agent 不得违反 `architecture.md` 的任何禁止项
2. 🟥 AI Agent 不得越权跨环节作业
3. 🟩 所有 AI 输出必须符合自检清单
4. 🟩 AI 修改必须写日志，关键地方必须写日志

### 架构文档维护

| 文档 | 优先级 | 维护者 |
|------|--------|--------|
| `docs/architecture.md` | 最高 | @architect |
| `docs/其他/30.md` | 参考 | 人类 |
| `docs/architecture/*.md` | 高 | @architect |
| `docs/adr/*.md` | 高 | @architect |
| `docs/domain/*.md` | 高 | @domain-designer |
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
- 所有 `docs/architecture.md` 的变更
- 所有 `docs/adr/` 的新增

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

```
发现 Bug：
1. @test-guardian 编写重现测试
2. @feature-developer 修复 Bug
3. @code-reviewer 审查修复
4. 发布 Hotfix
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
4. 读取相关 `docs/adr/`（最近决策）
5. 读取 `docs/domain/`（领域规则）