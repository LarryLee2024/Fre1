# docs/11-refactor — 技术债扫描与重构

本目录存放技术债扫描报告和重构计划。已完成的扫描归档至 `done/`。

## 目录结构

```
11-refactor/
├── README.md                                   # 本文件
└── done/                                       # 已完成的扫描报告（共 28 份）
    ├── action-plan.md                          # Picking 架构重构行动计划
    ├── content-compatibility-report.md         # Content 架构与 Screen Spec 兼容性报告
    ├── overlap-review-report.md                # Shared/Infra vs Core 重复评审报告
    ├── refactor-event-flow.md                  # Picking 事件流重构方案
    ├── refactor-picking-layer.md               # Picking 分层迁移方案
    ├── refactor-remove-debug.md                # Picking 调试代码清理计划
    ├── refactor-selection.md                   # Selection 状态管理重构方案
    ├── schema-compatibility-report.md          # Data Schema 与 Screen Spec 兼容性报告
    ├── ui-doc-gaps.md                          # UI 文档债务扫描报告
    └── ...（19 份已完成扫描报告，详见下方列表）
```

## 技术债生命周期

1. **扫描** → refactor-guardian 执行扫描，输出报告至 `11-refactor/` 根目录
2. **修复** → feature-developer 按优先级修复
3. **归档** → 所有债务 Resolved 后，移动至 `done/`

## 当前状态

- **总扫描次数**: 28 次（全部已完成归档）
- **当前活跃扫描**: 无 — 全部扫描与重构计划已完成

### Picking 架构重构（2026-06-23 ✅ 全部完成）

| Phase | 名称 | 状态 |
|-------|------|------|
| 0 | 宪法确认 (ADR-068) | ✅ 已完成 |
| 1 | 目录迁移 (infra/picking/ → ui/picking/) | ✅ 代码验证通过 |
| 2 | 事件流管道 (Pointer → PickIntent → Domain) | ✅ 代码验证通过 |
| 3 | Selection 重构 (SelectionState + CameraRequest) | ✅ 代码验证通过 |
| 4 | 清理调试代码 (println! → tracing) | ✅ 代码验证通过 |
| 5 | Bevy 兼容性 (ViewVisibility + UI 穿透) | ✅ 已确认 |
| 6 | 构建验证 (cargo build + test) | ✅ 验证通过 |
| 7 | 文档归档 | ✅ 本次完成 |

> 相关文件：`action-plan.md`、`refactor-picking-layer.md`、`refactor-event-flow.md`、`refactor-selection.md`、`refactor-remove-debug.md` — 已归档至 `done/`

### 兼容性评估报告

| 报告 | 状态 | 说明 |
|------|------|------|
| `content-compatibility-report.md` | ✅ 已完成 | Content 架构与 Screen Spec 95% 兼容，无需架构变更 |
| `schema-compatibility-report.md` | ✅ 已完成 | Data Schema 兼容性评估，列出 13 项数据前置条件 |
| `overlap-review-report.md` | ✅ 已完成 | Shared/Infra/Core 三层代码重复扫描，0 处重复 |
| `ui-doc-gaps.md` | ✅ 已完成 | UI 文档债务扫描，P0 4 项待修复清单 |
