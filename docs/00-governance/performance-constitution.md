---
id: PERFORMANCE-CONSTITUTION
title: 性能宪法
status: accepted
stability: stable
layer: governance
related:
  - ai-constitution-complete.md
tags:
  - performance
  - profiling
  - optimization
---

> **原文来源**：`ai-constitution-complete.md` 第十四编（L1303-L1330）
> **锚定总宪法**：第十四编

## 第十四编 性能宪法
- 🟩 正确性优先：必须先保证代码正确，再考虑性能优化
- 🟩 测量优先：所有性能问题必须通过 Profile 测量确认，🟥 绝对禁止凭直觉优化
- 🟩 优化原则：优先优化热点代码，🟥 绝对禁止为了性能进行全局重构
- 🟩 可读性优先：禁止为了性能牺牲代码可读性，除非有明确的 Profile 数据证明
- 🟩 优先使用 `Changed` 过滤器减少不必要的计算
- 🟩 🟥 绝对禁止在高频计算中使用 Reflect
- 🟩 缓存通用规范：所有缓存必须明确定义失效条件与重建方式，缓存永远不是事实源
- 🟩 不需要的 Feature 必须裁剪，而非无脑开启
- 🟩 大多数独立游戏死于复杂度，而非性能

### 性能例外机制
严格禁止凭感觉优化突破架构边界，申请例外必须同时满足：
1. 有 Profiling 实证数据证明跨层通信产生了可观测的性能瓶颈
2. 仅限核心战斗路径（伤害计算、属性结算、寻路等高频调用）
3. 提交 ADR 架构决策记录，说明原因、影响范围、有效期
4. 明确后续重构计划与到期时间

#### 例外标记规范
```rust
// ARCH_EXCEPTION: 战斗伤害计算性能优化
// ADR-042
// Expires: 2027-01-01
// Approver: Architect
// 说明：跳过事件广播，直接函数调用，降低结算开销
```

---
