---
id: ARCH-GOVERNANCE
title: 架构治理与演进宪法
status: accepted
stability: stable
layer: governance
related:
  - ai-constitution-complete.md
tags:
  - governance
  - adr
  - evolution
  - ci-gate
---

> **原文来源**：`ai-constitution-complete.md` 第十九编（L1702-L1729）
> **锚定总宪法**：第十九编

## 第十九编 架构治理与演进
### 19.1 新增模块检查清单
每次新增模块/文件必须逐项确认：
- [ ] 明确归属层级与领域/Domain
- [ ] 未依赖禁止的层级与模块
- [ ] 未包含不属于当前层的逻辑
- [ ] Capabilities 模块未包含业务规则，Domain 未重复实现通用机制
- [ ] 未突破 Domain 边界直接依赖其他 Domain
- [ ] 错误类型定义在对应领域内部
- [ ] 强类型 ID 放在 `shared/ids/`（五类 ID 分类详见 `docs/04-data/foundation/id-taxonomy.md`）
- [ ] Domain 层无裸 Entity/u64 作为业务标识（Entity 映射在 Infrastructure 层）
- [ ] 未创建 utils/helpers 垃圾桶文件

### 19.2 自动化门禁
- 开发期：自定义 Clippy Lint 实时提示跨层依赖、Domain 边界违规
- CI 阶段：运行依赖检查脚本全量扫描，违规直接阻断 PR
- 架构违规零容忍，不允许「先通过后修复」

### 19.3 架构版本管理
采用语义化版本，分级审批：
- **MAJOR**：结构性重构（合并/拆分层级、双轴结构调整）→ 全员评审 + 负责人批准
- **MINOR**：新增领域、新增接口、新增 Domain → Architect 审批
- **PATCH**：规则修正、文档补全 → 自主修改 + 同步周知

### 19.4 演进流程
需求评审 → ADR 记录 → 文档更新 → 全团队同步 → 代码迁移 → 验收确认
迁移期间新旧代码共存，通过 `#[deprecated]` 标记，确保零遗留。
