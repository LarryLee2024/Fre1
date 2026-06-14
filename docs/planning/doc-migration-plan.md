---
id: planning.doc-migration-plan
title: Doc Migration Plan
status: draft
owner: feature-developer
created: 2026-06-14
updated: 2026-06-14
tags:
  - planning
---

# Document Migration Plan

依据 `.trae/rules/文档治理规则.md` v3.0，对 `docs/` 进行完整的结构重组。

## Status

```
status: draft
```

## Current State

### 现存文档分布

| 目录 | 状态 | 文件数 | 说明 |
|------|------|--------|------|
| `docs/` (根) | 🟡 待迁移 | 7 | architecture.md(2503行), domain.md, AI开发宪法完整版.md, coding_rules.md, test_spec.md, test_spec_json.md, 目录.md |
| `docs/01-architecture/` | 🟡 待迁移 | 36 | 活跃架构设计文档 |
| `docs/02-domain/` | 🟡 待迁移 | 39 | 活跃领域规则文档 |
| `docs/08-decisions/` | 🟡 待迁移 | 9 | ADR 决策记录 |
| `docs/reviews/` | 🩶 归档态 | 18+ | 审查记录 (ai_ignore_this_dir) |
| `docs/refactor/` | 🩶 归档态 | 2 | 重构记录 (ai_ignore_this_dir) |
| `docs/02-domain/ai_ignore_this_dir/` | 🩶 归档态 | 22+ | 旧版本草稿 (_v1.md, _v2.md) |
| `docs/planning/` `testing/` `docs/` `其他/` | ⚪ 空 | 0 | 无内容 |
| `.trae/rules/` | 🔴 治理规则 | 14 | AI 行为规则，含 文档治理规则.md |
| `AGENTS.md` | 🟡 待迁移 | 1 | 项目根 |

### 新建编号目录（全部为空）

```
00-governance/ 01-architecture/ 02-domain/ 03-technical/ 04-data/
05-testing/ 06-ai/ 07-operations/ 08-decisions/ 09-history/ 10-roadmap/
```

---

## Gap Analysis

### 已违反规则摘要

| # | 规则 | 违反描述 | 严重度 |
|---|------|---------|--------|
| 1 | §1.1 SSOT | 17 对同主题文件同时存在于 `architecture/` 和 `domain/` | 🔴 |
| 2 | §1.2 Feature First | 文档分布在旧结构(architecture/domain/adr)而非 Feature 编号目录 | 🟡 |
| 3 | §1.3 落盘 | 部分治理规则仅存在于 AI 工具链 (`.trae/rules/`) 而未在 `docs/` 中有 SSOT | 🟡 |
| 4 | §1.4 禁止 Final/V2 | `domain/ai_ignore_this_dir/` 含 22 个 `_v1.md` `_v2.md` 文件 | 🟡 |
| 5 | §3 目录结构 | 新编号目录全空，旧目录无迁移 | 🔴 |
| 6 | §5 领域子目录 | 39 个 domain 文件平铺在同一目录，未按子领域分组 | 🟡 |
| 7 | §6 README | 所有目录缺少 README.md | 🔴 |
| 8 | §7 命名规范 | 存在 `snake_case` (asset_lifecycle_rules.md)、中文名(目录.md) | 🟡 |
| 9 | §8 Frontmatter | 95%+ 文档缺少 frontmatter | 🔴 |
| 10 | §13 复杂度预算 | `architecture.md` 2503 行超预算 2.5x | 🟡 |
| 11 | §14 自检清单 | 从未执行过文档自检 | 🟡 |

### 🔴 关键问题 1：17 对 SSOT 重复

以下主题在 `architecture/`（设计）和 `domain/`（规则）中都有文档：

| 主题 | architecture/ | domain/ |
|------|--------------|---------|
| 层次架构 | layer-contracts.md | layer_architecture_rules.md |
| 错误处理 | error-architecture.md | error_system_rules.md |
| 日志 | logging_design.md | logging_rules.md |
| 命令总线 | command_bus_design.md | command_bus_rules.md |
| 配置系统 | config_system_design.md | config_system_rules.md |
| 内容迁移 | content_migration_design.md | content_migration_rules.md |
| 资源生命周期 | asset_lifecycle_rules.md | asset_lifecycle_rules.md |
| 资源命名空间 | asset_namespace_design.md | asset_organization_rules.md |
| 校验 | validation_rules.md | validation_rules.md |
| 性能预算 | performance_budget.md | performance_budget_rules.md |
| 测试架构 | testing_architecture.md | testing_rules.md |
| Feature Flag | feature_flag_design.md | feature_flag_rules.md |
| UI 边界 | ui_domain_boundary_rules.md | ui_architecture_rules.md |
| 确定性 | determinism_rules.md | determinism_rules.md |
| ECS 通信 | — | ecs_communication_rules.md (仅 domain) |
| Hot Reload | — | hot_reload_rules.md (仅 domain) |
| 回放 | — | replay_rules.md (仅 domain) |

**策略**：一对一重复 → 比较内容合并为一。仅一方存在 → 按主题归属目录迁移。

### 🔴 关键问题 2：`.trae/rules/` 与 `docs/00-governance/` 的关系

`.trae/rules/` 是 OpenCode 工具链的 AI 行为规则加载机制，`docs/00-governance/` 是项目级 SSOT。

**策略**：
- `.trae/rules/` 保留为 AI-facing 副本（工具机制），在 frontmatter 标注 `source: docs/00-governance/`
- `docs/00-governance/` 作为人类可读的 SSOT
- 14 个 `.trae/rules/` 文件各对应一个 `docs/00-governance/` 英文 kebab-case 版本
- 两套可同步但不应独立演化

---

## Migration Phases

### Phase 0 — 文档审计与映射表

**任务**：
- [ ] 建立完整文件清单（已完成）
- [ ] 标记 17 对 SSOT 重复的处理策略（merge/keep-both/redirect）
- [ ] 分类所有文件到 11 个目标目录
- [ ] 决定每个 `.trae/rules/` → `docs/00-governance/` 的对应关系

**产出**：`docs/planning/doc-migration-matrix.csv`（源路径 → 目标路径映射）

**耗时估计**：1 次 AI Session

---

### Phase 1 — 目录重组（硬迁移）

将现有文件移动到对应的编号目录。按依赖顺序：

**1a: 08-decisions/**（无依赖，纯移动）
```
docs/08-decisions/ADR-001-migration-plan.md
  → docs/08-decisions/ADR-001-migration-plan.md
... (9 ADRs)
```

**1b: 01-architecture/**（架构目录）
```
docs/01-architecture/*.md (36 文件) → docs/01-architecture/
docs/02-domain/layer_architecture_rules.md → docs/01-architecture/layer-architecture-rules.md
docs/01-architecture/README.md → 拆分为 docs/01-architecture/README.md + 子文档
```

**1c: 02-domain/**（领域目录，需子目录化）
```
docs/02-domain/*.md (39 文件) → 按领域分组到子目录：
  02-domain/battle/battle.md + effect-pipeline.md + damage-formula.md
  02-domain/character/character.md + faction.md + unit-snapshot.md
  02-domain/skill/skill.md + targeting.md + cost.md + cooldown.md + requirement.md
  02-domain/buff/buff.md + stack-policy.md + duration.md
  02-domain/equipment/equipment.md
  02-domain/inventory/inventory.md
  02-domain/turn/turn.md
  02-domain/campaign/campaign.md
  02-domain/map/map.md + terrain.md + pathfinding.md
  02-domain/ai/ai.md
  02-domain/trigger/trigger.md
  02-domain/condition/condition.md
  02-domain/formula/formula.md
  02-domain/selector/selector.md
  02-domain/attribute-modifier/attribute-modifier.md
  02-domain/input/input.md
docs/02-domain/README.md → docs/02-domain/README.md
```

**1d: 03-technical/**
```
从 architecture/ 和 domain/ 筛选技术实现类文档：
  - 组件/system 设计规则
  - Plugin 设计
  - Schedule 设计
  - ECS 通信
  - 确定性
  - 回放
  - Hot Reload
  - UI 架构/边界
  - 输入处理
  - Feature Flag
  - 本地化
  - 性能预算
  - 错误处理（实现面）
  - 日志（实现面）
  - 命令总线（实现面）
```

**1e: 04-data/**
```
内容/数据/配置类文档：
  - Content Pipeline, Data Format, Migration Design
  - Content System Rules, Content Migration Rules
  - Config System Design + Rules
  - Asset Organization + Lifecycle + Namespace
  - IDs Design
  - Save Migration Rules
```

**1f: 05-testing/**
```
测试类文档：
  - docs/test_spec.md + test_spec_json.md
  - docs/01-architecture/testing_architecture.md
  - docs/05-testing/testing-rules.md
```

**1g: 06-ai/**
```
AI 协作类文档：
  - AGENTS.md（从项目根移入）
  - docs/01-architecture/collaboration-model.md
  - .trae/rules/AI协作规则.md → 06-ai/ 英文版
  - docs/reviews/ai_ignore_this_dir/ → 审查记录
```

**1h: 00-governance/**
```
治理规则类文档：
  - docs/AI开发宪法完整版.md → 00-governance/ai-constitution.md
  - docs/coding_rules.md → 00-governance/ （合并 .trae/rules/编码规则.md）
  对应 .trae/rules/ 各文件创建 00-governance/ 英文版
```

**1i: 09-history/**
```
历史归档类文档：
  - docs/01-architecture/migration-roadmap.md
  - domain/ai_ignore_this_dir/ 的 _v1 _v2 文件 → deprecate 标记 + 移入 09-history/archive/
  - review 记录
```

**1j: 10-roadmap/**
```
未来规划类文档：
  - 从 migration-roadmap.md 提取 roadmap 部分
```

---

### Phase 2 — 命名规范化（批量重命名）

将 `snake_case` 文件名改为 `kebab-case`：

| 当前 | 目标 |
|------|------|
| `asset_lifecycle_rules.md` | `asset-lifecycle-rules.md` |
| `component_design_rules.md` | `component-design-rules.md` |
| `system_design_rules.md` | `system-design-rules.md` |
| `plugin_contract_rules.md` | `plugin-contract-rules.md` |
| `schema_design.md` | `schema-design.md` |
| ... 所有 `_` 命名 | 全部统一 |

规则：
- 英文、小写、kebab-case
- 禁止中文文件名（`目录.md` → 移除或改名）
- 禁止大驼峰（`SkillSystem.md` → 不存在但需拦截）

---

### Phase 3 — Frontmatter 添加（批量）

所有 80+ 文件添加标准 frontmatter：

```yaml
---
id: <层级>.<主题>
title: <英文标题>
status: draft | stable | deprecated | archived
owner: architect | domain-designer | feature-developer
created: 2026-06-14
updated: 2026-06-14
related:
  - ../<相关路径>/<文件>.md
tags:
  - <标签>
---
```

批量策略：
- 按目录分类批量处理
- `architecture.md`: id = `architecture.index`
- `domain/*.md`: id = `domain.<subdomain>`
- `adr/*.md`: 保留编号，id = `adr.XXX`

---

### Phase 4 — README 导航创建

每个目录创建 `README.md`：

| 目录 | README 内容 |
|------|------------|
| `00-governance/` | 治理规则索引，阅读顺序 |
| `01-architecture/` | 架构设计目录，从 `docs/01-architecture/README.md` 提炼 |
| `02-domain/` | 领域规则索引，从 `docs/02-domain/README.md` 提炼 |
| `03-technical/` | 技术实现文档列表 |
| `04-data/` | 数据/内容文档列表 |
| `05-testing/` | 测试文档索引 |
| `06-ai/` | AI Agent 与协作流程索引 |
| `07-operations/` | 运维文档（初期为空） |
| `08-decisions/` | ADR 列表 |
| `09-history/` | 历史记录列表 |
| `10-roadmap/` | 未来规划列表 |

---

### Phase 5 — 复杂度治理

**需要拆分的文档：**

| 文件 | 行数 | 策略 |
|------|------|------|
| `docs/01-architecture/README.md` | 2503 | 拆为 `01-architecture/README.md`(索引) + 各子文档 |
| `docs/02-domain/README.md` | 313 | 转化为 `02-domain/README.md` |
| `docs/01-architecture/infrastructure-design.md` | 预估大 | 按 20 个模块拆分子文档 |

拆分原则（§13）：
- ≤1000 行：保持
- >1000 行：警觉
- >1500 行：评估按概念边界拆分
- 禁止机械按行数拆分

---

### Phase 6 — SSOT 合并

17 对重复文档的合并方案：

| 对 | architecture/ | domain/ | 合并策略 | 目标目录 |
|----|--------------|---------|---------|---------|
| 层次架构 | layer-contracts | layer_architecture_rules | 合并 → 01-architecture/ | 01-architecture/ |
| 错误 | error-architecture | error_system_rules | 合并 → 03-technical/ | 03-technical/ |
| 日志 | logging_design | logging_rules | 合并 → 03-technical/ | 03-technical/ |
| 命令总线 | command_bus_design | command_bus_rules | 合并 → 03-technical/ | 03-technical/ |
| 配置 | config_system_design | config_system_rules | 合并 → 04-data/ | 04-data/ |
| 内容迁移 | content_migration_design | content_migration_rules | 合并 → 04-data/ | 04-data/ |
| 资源生命周期 | asset_lifecycle_rules | asset_lifecycle_rules | 去重保留其一 | 04-data/ |
| 资源组织 | asset_namespace_design | asset_organization_rules | 合并 → 04-data/ | 04-data/ |
| 校验 | validation_rules | validation_rules | 去重保留其一 | 04-data/ |
| 性能预算 | performance_budget | performance_budget_rules | 合并 → 03-technical/ | 03-technical/ |
| 测试 | testing_architecture | testing_rules | 合并 → 05-testing/ | 05-testing/ |
| Feature Flag | feature_flag_design | feature_flag_rules | 合并 → 03-technical/ | 03-technical/ |
| UI | ui_domain_boundary_rules | ui_architecture_rules | 合并 → 03-technical/ | 03-technical/ |
| 确定性 | determinism_rules | determinism_rules | 去重保留其一 | 03-technical/ |
| ECS 通信 | — | ecs_communication_rules | 仅 domain → 03-technical/ | 03-technical/ |
| Hot Reload | — | hot_reload_rules | 仅 domain → 03-technical/ | 03-technical/ |
| 回放 | — | replay_rules | 仅 domain → 03-technical/ | 03-technical/ |

**去重策略**：
- 内容完全相同 → 删除重复，保留 SSOT
- 内容互补（design + rules） → 合并为一个文档
- 内容重叠 → 提炼差异，合并去重

---

### Phase 7 — 归档处理

`ai_ignore_this_dir/` 中的旧版本文件处理：

```yaml
# 每条记录
path: docs/02-domain/ai_ignore_this_dir/skill_rules_v1.md
action: moved to 09-history/archive/skill-rules-v1.md
frontmatter:
  status: archived
  superseded_by: ../02-domain/skill/skill.md
```

**21 个旧版本文件**分类：
- 被现有 stable 文档取代的 _v1 _v2 文件 → `09-history/archive/`
- review 记录 → `09-history/reviews/` 或 `06-ai/reviews/`
- 完全过时且无引用价值的 → 标记 `deprecated` 后移入 `09-history/archive/`

---

### Phase 8 — 交叉引用修复

迁移后更新所有跨文档链接：
- `../architecture/xxx.md` → `../01-architecture/xxx.md`
- `../domain/xxx.md` → `../02-domain/xxx.md`
- `../adr/ADR-XXX.md` → `../08-decisions/ADR-XXX.md`
- AGENTS.md 中引用的路径
- README.md 中的 docs/ 相关路径
- `.trae/rules/` 中引用的路径

---

### Phase 9 — 验证（AI 自检清单）

执行 §14 自检清单：

- [ ] 无重复事实源（无 SSOT 违规）
- [ ] 文件归属正确（每个文件在正确目录）
- [ ] 文件命名正确（kebab-case, 英文, 小写）
- [ ] Frontmatter 完整（所有文件）
- [ ] 存在唯一主文档（每个领域）
- [ ] README 已更新（所有目录）
- [ ] Related 链接有效（无 404）
- [ ] 状态合法（draft/stable/deprecated/archived）
- [ ] 未创建 Final/V2 文件
- [ ] 未破坏 SSOT

---

## 执行顺序

```
Phase 0 (审计) → Phase 1 (目录重组)
                → Phase 2 (命名) + Phase 6 (SSOT合并) [并行]
                → Phase 3 (Frontmatter) + Phase 4 (README) [并行]
                → Phase 5 (复杂度拆分) + Phase 7 (归档) [并行]
                → Phase 8 (引用修复)
                → Phase 9 (验证)
```

**依赖关系**：
- Phase 1 必须最先（否则路径全错）
- Phase 2 和 Phase 6 可在 Phase 1 后并行
- Phase 3/4 可在 Phase 2 后并行
- Phase 5/7 无严格先后
- Phase 8 必须等 1-7 全部完成
- Phase 9 最后

---

## 工作量估计

| Phase | 操作 | 文件数 | AI Session |
|-------|------|--------|------------|
| 0 | 审计 | 1 产出 | 1 |
| 1 | 目录重组 | 80+ 移动 | 2-3 |
| 2 | 命名标准化 | 40+ 重命名 | 1 |
| 3 | Frontmatter | 80+ 添加 | 2-3 |
| 4 | README | 11 目录 | 1 |
| 5 | 复杂度拆分 | 2-3 文件 | 1-2 |
| 6 | SSOT 合并 | 17 对 | 3-4 |
| 7 | 归档 | 20+ 文件 | 1 |
| 8 | 引用修复 | 全局 | 1-2 |
| 9 | 验证 | 全局 | 1 |
| | **总计** | | **14-19** |

---

## 风险与注意事项

1. **SSOT 合并风险**：17 对文档合并时可能丢失信息。必须逐对人工（AI）审查内容，不可机械合并。
2. **引用断裂**：Phase 1 移动后至 Phase 8 修复前，跨文档链接会断裂。宜批量完成 Phase 1→8 减少中间态时间。
3. **`.trae/rules/` 同步**：迁移后 `.trae/rules/` 仍保留旧路径引用。需决定是否更新 `.trae/rules/` 中的引用路径。
4. **`ai_ignore_this_dir` 约定**：当前作为"AI 忽略目录"的 `ai_ignore_this_dir` 在目标结构中如何对应？建议在 `09-history/` 中保留此机制。
5. **非文档文件**：`.DS_Store`、`目录.md`（已过时的旧目录结构参考）等需要清理。
6. **README.md 的 `docs/` 引用**：项目 `README.md` 中列出了旧 `docs/` 结构，需同步更新。
