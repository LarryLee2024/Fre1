---
id: 10-reviews.domain-vs-governance
title: Review — Domain Documents vs Governance Compliance
status: completed
owner: code-reviewer
created: 2026-06-16
updated: 2026-06-16
tags:
  - review
  - domain
  - governance
  - compliance
---

# Code Review Report: Domain Rules (`docs/02-domain/`) vs Governance

**Reviewer**: @code-reviewer
**Scope**: `docs/02-domain/` (30 domain rule files + README)
**Standards**: `docs/00-governance/ai-constitution-complete.md` (v5.0) + `docs/00-governance/Fre项目架构设计.md`
**Date**: 2026-06-16

---

## ✅ Checks Passed

| Check | Status |
|-------|--------|
| 30 domain files cover all 15 Capabilities + 15 Business Domains per constitution §3.3 and §3.4 | ✅ Pass |
| Domain files follow an 8-section structure (术语表/状态机/不变量/禁止事项/流程定义/领域事件/对齐校验/自检清单) | ✅ Pass |
| Capabilities domains are clearly separated from Business Domains | ✅ Pass |
| Terms are consistently defined with 职责边界 (responsibility boundary) columns | ✅ Pass |
| Invariants section present with condition + rule + consequence format | ✅ Pass |
| Forbidden actions sections list explicit prohibitions | ✅ Pass |
| Process flows (输入 → 处理 → 输出 → 失败处理) are well-defined | ✅ Pass |
| Domain events are clearly listed with subscriber relationships | ✅ Pass |
| Domain documents reference upstream/downstream domains | ✅ Pass |
| README provides dependency graphs and domain-to-schema mappings | ✅ Pass |
| README correctly states the 8-section standard for all domain docs | ✅ Pass |
| All domain docs align with the 15/15 split from constitution | ✅ Pass |

---

## ❌ Issues Found

### [HIGH] 1. Missing Standard Metadata Header

**Location**: All 30 domain `.md` files in `docs/02-domain/`

**Constitutional Rule**: 宪法 §第三编要求所有架构/领域文档包含标准元数据头。`Fre项目架构设计.md` 中的各节文档示例使用 `---` 分隔的 YAML front matter 元数据头。

**Violation**: Every domain file starts with its own format:
```markdown
# Tag（标签）领域规则 v1.0

Version: 1.0
Status: Draft
Applies To: Capabilities — 核心基石层
```

But the standard metadata format used by architecture and data docs is:
```yaml
---
id: <domain>.<type>.<version>
title: <Title>
status: draft | stable | deprecated
owner: <domain-designer>
created: <YYYY-MM-DD>
updated: <YYYY-MM-DD>
tags:
  - domain
  - ...
---
```

The domain files are missing: `id`, `owner`, `created`, `updated`, `tags`. This makes cross-referencing, automated validation, and status tracking harder.

**Why This Is High**: While the domain content is correct, the missing metadata means:
- No traceable ownership (who owns each domain?)
- No creation/update timestamps for governance audits
- Cannot be automatically validated against the schema mapping in README
- AI tools cannot reliably parse standard metadata from these files

**Recommendation**: Add standard YAML front matter to all 30 domain files, matching the format used by `docs/04-data/` and `docs/01-architecture/`. Migrate the inline `Version:`, `Status:`, `Applies To:` into the YAML header.

**Severity**: HIGH — P1 (跨文件格式一致性)

---

### [HIGH] 2. Status Value Inconsistency: README says "stable", files say "Draft"

**Location**: `docs/02-domain/README.md` §5 (File Status table) vs individual domain files

**Constitutional Rule**: 宪法 §19.3 — 架构版本管理要求状态一致性。

**Violation**: The README §5 status table marks ALL 30 domain files as "✅ stable". However, individual domain files consistently say `Status: Draft` in their inline header (e.g., `tag_domain.md` line 4: `Status: Draft`). This is a direct contradiction:

| Source | Says | Actually verified |
|--------|------|-------------------|
| README §5 | `✅ stable` | Not verified — implies approved |
| Individual files | `Status: Draft` | Still in draft — implies not finalized |

**Why This Is High**: Downstream consumers (architects, developers) cannot determine whether these domain rules are finalized and authoritative. If they are "Draft", they should not be treated as binding; if they are "stable", the files should say so. The self-contradiction is confusing and undermines trust in the documentation.

**Recommendation**: 
- Either: Update all 30 domain files to `Status: Stable` (if they are indeed approved)
- Or: Update the README status table to `⬜ draft` (if the are still drafts)
- Add dates when the status was last reviewed
- Consider adding a `reviewed_by` metadata field

**Severity**: HIGH — P1

---

### [MEDIUM] 3. Applies To Field Inconsistency

**Location**: Domain file headers — the `Applies To:` field uses inconsistent categorization

**Constitutional Rule**: 宪法 §3.4 — Domains 业务域 vs Capabilities 能力领域 should use consistent terminology.

**Violation**: The `Applies To:` field uses different naming conventions:
- `tag_domain.md`: "Capabilities — 核心基石层"
- `attribute_domain.md`: "Capabilities — 核心基石层"
- `condition_domain.md`: "Capabilities — 配置/条件层"
- `ability_domain.md`: "Capabilities — 行为表现层"
- `combat_domain.md`: "Domains — 战斗核心层"
- `tactical_domain.md`: "Domains — 战术空间层"
- `economy_domain.md`: "Domains — 经济系统层"

Looking closer, the domain README §1.1 and §1.2 provide a classification system:
- Capabilities: 核心基石 / 配置/条件 / 行为表现
- Business Domains: Foundation / Core / Narrative / Economy

But the individual files use their own descriptions. The `combat_domain.md` says "战斗核心层" while README says "Core Layer (战斗核心)". These are mostly consistent, but the README's canonical classification should be the single source of truth.

**Recommendation**: Standardize the `Applies To:` format across all 30 domain files to match the README's official classification, or remove `Applies To:` from individual files and rely solely on the README index.

**Severity**: MEDIUM — P2

---

### [MEDIUM] 4. No Explicit Constitution Compliance Section

**Location**: All domain documents — end of each file

**Constitutional Rule**: 宪法 §1.4 — 所有文档必须可追溯至宪法条款。`Fre项目架构设计.md` 的 ADR 模板要求 Constitution Check。

**Violation**: While domain documents have an §7 "对齐校验" (Alignment Check) section and §8 "自检清单" (Self-Check List), neither explicitly references the constitution (`ai-constitution-complete.md`) or the architecture design doc (`Fre项目架构设计.md`). The alignment check references only upstream domain docs and the 7-layer architecture.

Given that the constitution is the highest authority (§1.1 - 效力说明), all domain documents should have a dedicated constitution compliance section that explicitly maps domain rules to constitution clauses.

**Recommendation**: Add a "Constitution Check" subsection to §7 (对齐校验) in all domain files, cross-referencing relevant constitution clauses. For example:
```markdown
### 宪法合规检查
- [ ] 符合 Feature First 原则（宪法 §1.5 P0）
- [ ] Effect Pipeline 未被绕过（宪法 §8.5）
- [ ] Modifier Pipeline 未被绕过（宪法 §8.2）
- [ ] Def/Instance 已分离（宪法 §1.4）
```

**Severity**: MEDIUM — P2

---

### [MEDIUM] 5. Comma Placement in README Domain Count

**Location**: `docs/02-domain/README.md` line 19

**Violation**: The README states "30 个领域规则文件的索引和快速参考" — but then lists 15 Capabilities + 15 Business Domains = 30, plus 0 infrastructure domains. This is correct. However, the architecture README lists Layer 7 infrastructure features (registry, pipeline, replay, save, input) as having no domain documents, which means the 30 domain files do not cover Infrastructure features.

**Why This Is Medium**: This creates a blind spot — cross-cutting features like replay, save, and input have no domain rules defined anywhere, yet they are first-class features in the architecture. Their behavior is not governed by any domain document.

**Recommendation**: Add a note to the domain README explicitly noting that Infrastructure (Layer 7) features do not yet have domain documents. Consider creating domain documents for at least `replay_domain.md`, `save_domain.md`, and `input_domain.md` since they have significant architectural implications.

**Severity**: MEDIUM — P2

---

### [LOW] 6. README Section Structure Describes Format Not Fully Followed

**Location**: `docs/02-domain/README.md` §3 (各文档结构标准) vs actual domain files

**Constitutional Rule**: 宪法 §16.1 — 文档标准一致性。

**Violation**: README §3 defines the 8-section structure as:
```
1. 统一术语
2. 状态机
3. 不变量
4. 禁止事项
5. 流程定义
6. 领域事件
7. 对齐校验
8. 自检清单
```

Actual files follow this structure but with some deviations:
- `tag_domain.md` has sections: 统一术语 → 标签层级状态机 → 标签不变量 → 禁止事项 → 标签操作流程 → 领域事件 → 对齐校验 → 自检清单
- `combat_domain.md` has sections: 统一术语 → 战斗核心规则 (not "状态机") → 战斗不变量 → 禁止事项 → 战斗流程 → 领域事件 → 对齐校验 → 自检清单

The content is present and well-organized, but section numbering and titles vary slightly from the advertised standard. For example, §2 is "状态机" in the standard but "标签层级状态机" in tag_domain and "战斗核心规则" in combat_domain.

**Recommendation**: Either relax the README description to acknowledge title variations, or enforce strict section title compliance across all 30 files.

**Severity**: LOW — P3

---

### [LOW] 7. Pre-versioned Files Without Clear Versioning Scheme

**Location**: All domain files — `Version: 1.0` in inline header

**Violation**: All domain files declare `Version: 1.0` but:
- It's unclear when version should increment (major/minor/patch)
- No changelog or version history in any file
- No cross-reference to ADR that established v1.0
- The constitution §19.3 defines semantic versioning for architecture, but domain docs have no explicit versioning policy

**Recommendation**: Define a versioning policy for domain documents. At minimum, add a `## 修订历史` section to each file or a central changelog.

**Severity**: LOW — P3

---

## 📋 Summary

| Severity | Count | Issues |
|----------|-------|--------|
| **CRITICAL** | 0 | — |
| **HIGH** | 2 | Missing standard metadata headers, README status vs file status conflict |
| **MEDIUM** | 3 | Applies To inconsistency, no explicit constitution compliance section, missing Infrastructure domains |
| **LOW** | 2 | Section title deviations from advertised standard, unclear versioning scheme |

---

## 🎯 Conclusion

### PASS with High-severity issues

The domain documents are content-complete and structurally sound. No CRITICAL issues found. However, two HIGH issues need attention before these documents can be considered fully production-ready.

### Required Actions

- **P1**: Add standard YAML front matter (id/title/status/owner/created/updated) to all 30 domain files
- **P1**: Resolve the `stable` vs `Draft` status conflict — either promote all files to `stable` or demote the README status table
- **P2**: Standardize the `Applies To:` categorization across all files
- **P2**: Add constitution compliance checking to the §7 对齐校验 section
- **P2**: Create domain documents for Infrastructure (Layer 7) features or note their absence
- **P3**: Relax or enforce strict section title compliance per the README standard

### Next Steps

After fixing HIGH issues → call **@code-reviewer** for re-review.
Consider also calling **@data-architect** to align domain metadata format with schema metadata format.
