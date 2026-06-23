# UI 文档债务扫描报告

> 扫描日期: 2026-06-22
> 扫描范围: `docs/06-ui/` 全部 16 个文件 + `07-specs/`
> 标准基线: SSPEC（Screen Specification）标准 + ADR-066
> 发现总数: 21 项（P0: 4 / P1: 3 / P2: 8 / P3: 6）

---

## P0 — 关键

| 编号 | 问题 | 位置 | 建议 |
|------|------|------|------|
| DOC-004 | 所有 16 个文档均未引用 SSPEC 或 ADR-066 | 全部文件 | 每个文件添加 SSPEC 引用行 |
| DOC-005 | README.md 目录树中缺少 07-specs/ | README.md:68-91 | 添加 07-specs/ 索引项 |
| DOC-019 | `07-specs/references/` 4 个引用文件不存在 | references/ 目录（空） | 创建 widget-id-map.md、z-layer-spec.md、layout-intent-library.md、screen-metrics-baseline.md |
| DOC-020 | main_menu_screen.md DoD 全部未勾选 | main_menu_screen.md:719-738 | 审查并勾选 DoD 清单 |

## P1 — 高

| 编号 | 问题 | 位置 | 建议 |
|------|------|------|------|
| DOC-001/002 | screens.md 与 SSPEC 布局描述重复 | screens.md §2-§3 | 添加引用 + 精简运行时细节 |
| DOC-015 | 4 个文件状态字段不匹配 | README.md 状态表 | 同步 frontmatter 与状态表 |
| DOC-007 | README.md 上游表缺少 ADR-066 | README.md §3 | 添加 ADR-066 映射行 |

## P2 — 中

| 编号 | 问题 | 位置 |
|------|------|------|
| DOC-003 | camera-ui-interaction.md 不考虑 SSPEC | camera-ui-interaction.md |
| DOC-006 | 状态表缺少 camera/map-rendering | README.md 状态表 |
| DOC-008 | testing.md 无 SSPEC 验证测试 | testing.md |
| DOC-009 | 状态字段格式不一致（emoji vs 文字） | 多处 |
| DOC-012 | Dirty<T> 在两个文档重复定义 | projection-viewmodel.md + focus-binding.md |
| DOC-013 | Widget Contract 定义散落在 3 个文件 | screen-lifecycle.md + widget-atoms.md + widget-composites.md |
| DOC-021 | HeadingText/CaptionText 可能无 Contract | widget-atoms.md 验证 |

## P3 — 低

| 编号 | 问题 |
|------|------|
| DOC-010 | created/updated 字段不一致 |
| DOC-011 | 07-specs id 命名空间与 06-ui 不同（合理但需记录） |
| DOC-014 | Overlay 基础结构在两个文档重复 |
| DOC-016 | implementation-patterns.md 1184 行（AI 过载） |
| DOC-017 | widget-composites.md 1095 行（AI 过载） |
| DOC-018 | map-rendering.md 1010 行（AI 过载） |

---

## 建议处理优先级

1. 立即：修复 P0（DOC-004, 005, 019, 020）— 阻碍 AI 知道 SSPEC 存在
2. 下一批：修复 P1（DOC-001/002, 007, 015）— 架构文档同步
3. 后续：P2-P3 逐步清理，随 Screen Spec 推进自然解决

*完整发现列表见对话记录。*
