# docs/10-reviews — 审查报告

本目录存放代码审查、架构审查、一致性审查等报告。

## 目录结构

```
10-reviews/
├── README.md                          # 本文件（当前无活跃审查）
└── done/                              # 已归档的审查报告（共 34 份）
```

## 审查文档生命周期

1. **创建** → 文件置于 `10-reviews/` 根目录（当状态为 active）
2. **执行中** → 按审查清单逐项检查
3. **完成** → 移动至 `done/`，更新本 README

## 当前状态

| 状态 | 说明 |
|------|------|
| ✅ 全部已归档 | `done/` 目录共 34 份审查报告 |
| 🟡 `screen-spec-code-gaps.md` | partial — 3/4 P0 已修复，1/4 部分修复；P1/P2 项目待跟进 |
| 🟡 `screen-spec-test-gaps.md` | draft — 测试差距分析（7 种测试类型，P0/P1/P2 分批建议） |
| 🟡 `ui-design-code-drift-review.md` | draft — UI 设计与代码偏移审查（2 P0: 工厂绕过/Dirty 消费断裂） |
