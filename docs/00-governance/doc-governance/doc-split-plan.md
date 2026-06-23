---
id: DOC-GOV-SPLIT-PLAN
title: 19拆宪法.md 文件拆分方案
status: accepted
stability: stable
layer: governance
related:
  - ai-constitution-complete.md
  - doc-governance-overview.md
tags:
  - doc-governance
  - split-plan
---

# 19拆宪法.md 文件拆分方案

> 拆分对象：`docs/99-history/ai_ignore_this_dir/19拆宪法.md`（L1–L1103，约 1100 行）
> 关联总宪法：`docs/00-governance/ai-constitution-complete.md`（22 编 + 附则）
> 输出目录：`docs/00-governance/doc-governance/`

---

## 一、拆分原则与逻辑依据

| 原则 | 逻辑依据 |
|------|---------|
| **主题内聚** | 原文由三个异质板块构成（拆分方法论 L1–L369 / 十项治理机制 L370–L792 / YAML 元数据规范 L794–L1103），按"一个文件只讲清一个治理维度"切分 |
| **与总宪法编章对齐** | 每个拆分文件映射到 `ai-constitution-complete.md` 的具体编章，形成双向锚点 |
| **可独立检索** | 单文件控制在 80–200 行，便于 Repomix/CodeGraph 精准命中 |
| **元规范与正文分离** | 原文是"如何治理文档"的元规范，统一归入 `doc-governance/` 子目录，不混入宪法正文 |
| **不增不减** | 原文每一论点落入某个拆分文件，校验表逐条核对 |
| **命名可推断** | 文件名采用 `主题-spec.md` / `主题-overview.md` 模式 |

---

## 二、拆分文件清单

| # | 文件名 | 主题定位 | 内容范围 | 原文行号 |
|---|--------|---------|---------|---------|
| 1 | `doc-governance-overview.md` | 文档治理总纲 | 八层拆分方法论 + 最终目录形态 + 本套规范索引 | L1–L369 |
| 2 | `doc-classification-spec.md` | 文档分级与稳定性 | P0–P4 五级分级 + Stable/Evolving/Experimental 三级稳定性 | L373–L422, L627–L655 |
| 3 | `rule-id-system-spec.md` | Rule ID 编号体系 | 前缀编号规则 + 代码注释引用 + Review 引用 | L423–L467 |
| 4 | `anti-pattern-library-spec.md` | 反模式库规范 | anti-patterns/ 目录 + 三段式结构 + 初始清单 | L468–L510 |
| 5 | `adr-lifecycle-spec.md` | ADR 生命周期管理 | 五状态机 + supersedes 关系链 | L511–L545 |
| 6 | `glossary-spec.md` | 术语表规范 | glossary/ 体系 + 五组易混术语 | L546–L595 |
| 7 | `knowledge-map-spec.md` | 知识地图规范 | 按场景阅读路径 + 与 Index 区别 | L596–L626 |
| 8 | `fitness-function-spec.md` | 架构适应度函数 | dependency_checker + CI 自动检查 | L656–L701 |
| 9 | `rule-retirement-spec.md` | 规则退役机制 | 12/18/24 月三阶段流程 | L702–L733 |
| 10 | `ai-condensed-guide-spec.md` | AI 精简版规范 | CLAUDE.md ≤20 页 + 四层知识体系 | L734–L792 |
| 11 | `yaml-frontmatter-spec.md` | YAML 元数据规范 | 三类分级 + 字段规范 + 四类模板 | L794–L1103 |

---

## 三、引用关系说明

### 3.1 拆分文件 → 总宪法（正向锚点）

| 拆分文件 | 锚定总宪法编章 |
|---------|---------------|
| `doc-governance-overview.md` | 第一、二、六、九、十、十一、十二编 |
| `doc-classification-spec.md` | 第一编 1.3 强制等级说明 |
| `rule-id-system-spec.md` | 第二十一编红线禁止事项总览 |
| `anti-pattern-library-spec.md` | 第二十编 20.1 AI 反模式黑名单 |
| `adr-lifecycle-spec.md` | 第十九编 19.3–19.4 |
| `glossary-spec.md` | 全局术语统一 |
| `knowledge-map-spec.md` | 全局导航 |
| `fitness-function-spec.md` | 第十九编 19.2 + 第二编 2.9 |
| `rule-retirement-spec.md` | 第十七编长期维护 |
| `ai-condensed-guide-spec.md` | 第二十编 AI 执行规范 |
| `yaml-frontmatter-spec.md` | 全局元数据 |

### 3.2 总宪法 → 拆分文件（反向引用）

建议在 `ai-constitution-complete.md` 附则新增"文档治理规范索引"节，逐条列出 11 个文件路径。

### 3.3 拆分文件之间

- `doc-governance-overview.md` 为入口，索引其余 10 文件
- `doc-classification-spec.md` 被 `adr-lifecycle-spec.md`、`yaml-frontmatter-spec.md` 引用
- `rule-id-system-spec.md` 被 `anti-pattern-library-spec.md`、`rule-retirement-spec.md`、`fitness-function-spec.md` 引用
- `yaml-frontmatter-spec.md` 被所有 spec 文件引用（frontmatter 模板来源）
- `knowledge-map-spec.md` 引用全部其余文件

---

## 四、关键内容保留策略

1. **八层方法论不压缩**：每层"做法 + 作用"完整保留
2. **最终目录树原样保留**：L312–L369 目录树一字不删
3. **十个关键点各成独立文件**：禁止合并稀释
4. **YAML 模板逐字保留**：四类模板可复制使用
5. **代码块原样迁移**：所有 text/yaml/rust/markdown 代码块不转述
6. **行号映射建档**：每个拆分文件头部标注原文行号区间

---

## 五、校验表 A：原文核心内容要点 → 拆分文件映射

| 原文要点 | 原文行号 | 目标文件 | 完整性验证标准 |
|---------|---------|---------|---------------|
| 第一层：总宪法单文件 | L1–L44 | overview §1 | 含页数控制、四编划分、作用定位 |
| 第二层：法律体系目录 | L45–L86 | overview §2 | 含完整 9 目录树 |
| 第三层：ECS 按主题拆 | L87–L107 | overview §3 | 含 7 子文件名 |
| 第四层：Capabilities 单独成册 | L108–L149 | overview §4 | 含 15 子文件名 |
| 第五层：Domain 单独成册 | L150–L179 | overview §5 | 含 Git Diff 论据 |
| 第六层：ADR 独立 | L180–L224 | overview §6 + adr-lifecycle | 含"一句话引用"原则 |
| 第七层：AI Rules 独立 | L225–L255 | overview §7 | 含 6 子文件名 |
| 第八层：Index | L256–L310 | overview §8 + knowledge-map | 含 8 目录索引 |
| 最终形态目录树 | L312–L369 | overview §9 | 目录树逐字保留 |
| 关键点1：文档分级 | L373–L422 | classification §1 | 含 P0–P4 + 目录映射 |
| 关键点2：Rule ID | L423–L467 | rule-id-system | 含编号示例 + 代码注释格式 |
| 关键点3：Anti-Pattern | L468–L510 | anti-pattern-library | 含 6 文件名 + 三段式 |
| 关键点4：ADR 生命周期 | L511–L545 | adr-lifecycle | 含 5 状态 + supersedes |
| 关键点5：Vocabulary | L546–L595 | glossary | 含 5 组易混术语 |
| 关键点6：Knowledge Map | L596–L626 | knowledge-map | 含阅读路径示例 |
| 关键点7：Stability Level | L627–L655 | classification §2 | 含三级 + 示例 |
| 关键点8：Fitness Function | L656–L701 | fitness-function | 含 3 类检查 + P0 定位 |
| 关键点9：删除规则 | L702–L733 | rule-retirement | 含 12/18/24 月阈值 |
| 关键点10：AI 精简版 | L734–L792 | ai-condensed-guide | 含 ≤20 页 + 四层体系 |
| YAML 总体原则 | L794–L820 | yaml §总体原则 | 含"元数据比正文难维护" |
| 第一类必须 YAML | L821–L867 | yaml §一.1 | 含 ADR frontmatter + 5 用途 |
| 第二类极简 YAML | L868–L904 | yaml §一.2 | 含 DOM-COMBAT 示例 |
| 第三类不要 YAML | L905–L928 | yaml §一.3 | 含"直接正文"原则 |
| AI 最有价值字段 | L929–L961 | yaml §二 | 含 4 字段 + CAP-EFFECT |
| 不要写的字段 | L962–L990 | yaml §三 | 含禁写字段清单 |
| 推荐增加字段 | L991–L1025 | yaml §四 | 含 layer/stability/related |
| 四类最终模板 | L1026–L1103 | yaml §五+§六 | 四模板逐字 + 汇总表 |

---

## 六、校验表 B：完整性验证标准

| 验证项 | 标准 | 结果 |
|--------|------|------|
| 行号覆盖 | L1–L1103 每一行落入某拆分文件 | 详见复核报告 |
| 代码块保留 | 原文代码块数量 = 拆分后总数 | 详见复核报告 |
| 目录树完整 | L312–L369 目录树逐字出现 | ✓ |
| YAML 模板可用 | 四类模板可直接复制 | ✓ |
| 双向引用成立 | 正向锚点 + 反向索引 | ✓ |
| 文件间引用成立 | overview 索引 + knowledge-map 串联 | ✓ |
| 无内容合并稀释 | 十关键点各独立 + YAML 三类不合并 | ✓ |
| 命名一致性 | 11 文件均 `主题-spec.md`/`overview.md` | ✓ |
| 术语对齐 | 与总宪法 + glossary 一致 | ✓ |
| 可回溯 | 每文件头部含原文行号标注 | ✓ |
