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
| 🟡 SSPEC 代码差距分析 | `screen-spec-code-gaps.md` — 审查 src/ui/ 与 Screen Specification 标准的差距，FAIL (4 P0) |
