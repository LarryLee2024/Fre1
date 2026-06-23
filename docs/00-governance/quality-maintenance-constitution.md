---
id: QUALITY-MAINTENANCE-CONSTITUTION
title: 工程质量与长期维护宪法
status: accepted
stability: stable
layer: governance
related:
  - ai-constitution-complete.md
tags:
  - quality
  - maintenance
  - tech-debt
  - ci
---

> **原文来源**：`ai-constitution-complete.md` 第十七编（L1572-L1586）+ 第十八编（L1589-L1699）
> **锚定总宪法**：第十七编、第十八编

## 第十七编 长期维护与运营宪法
### 17.1 核心维护原则
- 🟩 代码首先是写给人看的，其次才是写给机器执行的
- 🟩 明确优于聪明，简单优于优雅，稳定优于炫技
- 🟩 删除无用代码通常比写新代码更有价值
- 🟩 社区维护的成本通常低于自维护成本
- 🟩 每引入一个自研系统，必须评估未来五年的维护成本
- 🟩 架构必须每 3 个月进行一次复盘和调整，重点清理过度设计和无用代码
- 🟩 工具链与内容生产能力最终决定项目成败

### 17.2 扩展预留规范
- 🟨 Mod 支持预留：核心系统预留轻量扩展点，不提前实现完整 Mod 框架
- 🟩 国际化强制：代码中绝对禁止出现用户可见的硬编码文本，所有用户可见文本必须通过 LocalizationKey + Fluent (.ftl) 文件管理；Def 只存 name_key/desc_key 等 Key 引用，不存直接文本；Replay/BattleLog/Event 只存 Key+参数，不保存最终翻译结果
- 🟨 遥测预留：核心领域事件设计时考虑数据埋点需求，不提前实现完整遥测系统

---

## 第十八编 工程质量与技术债治理
### 18.1 核心原则
- Warning Budget = 0：主分支不允许存在未处理的编译警告
- Bug Budget = 0：P0/P1 级缺陷不允许流入主分支
- Tech Debt Budget = 可控：技术债必须登记跟踪，设定偿还节点

### 18.2 问题分级标准
#### P0 致命级（必须立即修复，禁止提交/合并）
1. 核心业务代码出现 `unwrap()`、`expect()`、`panic!()`、`todo!()`、`unimplemented!()`
2. ECS 系统 Query Borrow 冲突
3. 事件/Observer/Trigger 触发无限循环
4. Entity 泄漏，长期运行资源持续增长
5. 存档兼容性损坏
6. 核心数据损坏、状态机非法跳转
7. 双轴边界严重突破：Capabilities 包含业务规则、Domain 间直接依赖

#### P1 高优先级（一个迭代内必须修复，CI 拦截）
1. Rust 必修编译警告（unused_imports、dead_code、deprecated 等）
2. 已废弃 API 调用
3. Clippy 必修项警告
4. 配置数据校验失败
5. 跨模块边界违规、架构约束被破坏
6. Domain 边界违规、Capabilities 越界实现业务逻辑

#### P2 中优先级（允许短期存在，登记跟踪）
1. 非热点代码存在可优化的性能问题
2. ECS Archetype 频繁抖动
3. 非核心路径存在过度日志、日志噪音
4. Archetype 数量膨胀风险

#### P3 低优先级（技术债，统一偿还）
1. 局部重复代码，未达到抽象阈值
2. 命名风格不统一，不影响功能与可读性
3. 单文件体积偏大但内聚性良好
4. 可补充的文档注释

### 18.3 Rust Warning 与 Clippy 规范
- 必修项警告必须 100% 修复，禁止无理由屏蔽
- 可暂缓项登记跟踪，按需处理
- 所有 `#[allow(...)]` 属性必须附带注释说明豁免理由与有效期

### 18.4 Bevy 专项检查
- 禁止每帧高频生成/销毁大量实体
- 禁止单帧触发大量无意义事件
- Observer/Trigger 递归必须设置深度限制
- 监控 Archetype 数量膨胀

### 18.5 编译治理专项
- 🟩 **依赖形状优化**：通过双轴架构实现 Domain 级平行编译，单个 Domain 修改不触发全量重编译
- 🟩 **Feature 开关**：支持按 Domain 开启/关闭编译，开发期可只编译当前工作域
- 🟩 **CI 增量编译**：配置 sccache 分布式编译缓存，大幅提升 CI 速度

### 18.6 CI 门禁强制标准
主分支准入必须全部满足，任意一项不满足禁止合并：
1. `cargo fmt` 0 格式问题
2. `cargo clippy` 0 必修项警告
3. `cargo test` 全部测试通过
4. 配置数据校验全部通过
5. 架构依赖检查无违规（含层间边界、Capabilities/Domain 边界）
6. Domain 间直接依赖检测无违规

### 18.7 技术债扫描六大维度

refactor-guardian 必须覆盖以下六个扫描维度（来源：50万行级项目实践评估）：

#### 18.7.1 Architecture Drift（架构漂移）
- ADR 定义的依赖方向 vs 实际代码依赖方向的偏差
- 检查：ADR 规定 A→B→C，实际是否出现 C→A 反向依赖
- 级别：方向违反 → Critical

#### 18.7.2 Abstraction Leakage（抽象泄漏）
- 跨域访问内部类型，绕过 integration/ 或 Facade 层
- 检查：`use xxx::mechanism` / `use xxx::internal` / `use xxx::model` 跨域出现
- 级别：跨域 internal 泄漏 → High

#### 18.7.3 AI Maintainability（AI 可维护性）
- 文件/函数/match 过大导致 AI 无法完整理解和修改
- 阈值：文件>1500行=High，>2500行=Critical；函数>100行=High；match>50 arm=High
- 级别：按阈值分级

#### 18.7.4 Test Debt（测试债务）
- 核心 Facade、Observer、Event 链缺乏测试覆盖
- 检查：`integration/facade.rs` 无对应 `tests/`；Observer 无集成测试
- 级别：核心 Facade 无测试 → High

#### 18.7.5 Content Debt（内容债务）
- 业务数值硬编码在代码中，应迁移到 `content/` 配置
- 检查：grep domains/ 中的 `damage=` / `range=` / `cooldown=` 等赋值
- 级别：硬编码业务数值 → Medium

#### 18.7.6 Debt Lifecycle（技术债生命周期）
- 所有 Debt 条目必须包含：状态（Open / Accepted / In Progress / Resolved / WontFix）、发现日期、负责人、关联 ADR
- ID 命名：`Debt-` / `Drift-ADR-` / `Leak-` / `Maintain-` / `TestDebt-` / `Content-`

### 第 X 条：Screen 复杂度治理（P1）

| 标记 | 规则 |
|------|------|
| 🟩 | Screen Metrics 基线追踪：每个 Screen 必须记录 `widget_count` / `container_count` / `interactive_count` / `max_depth` |
| 🟩 | Widget Budget：`max_widget_depth ≤ 6`，`max_children_per_container ≤ 20` |
| ⚠️ | 超过阈值时必须重构，不得累积复杂度债务 |

### 第 X 条：Figma 替代工具链治理（P0）

| 标记 | 规则 |
|------|------|
| 🟩 | 新增 Screen 的 UI 设计流程为：写 SSPEC → DoD 检查 → 实现代码 |
| 🟩 | SSPEC 是 UI 设计的唯一真相源（SSOT），不依赖任何 GUI 设计工具 |
| 🟥 | 禁止将 Figma / PSD / Sketch 文件作为 UI 需求附件 |
| 🟥 | 禁止在 SSPEC 中引用 GUI 设计工具的输出作为布局依据 |
