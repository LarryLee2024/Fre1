---
id: 10-reviews.screen-spec-test-gaps
title: Screen Spec Test Gaps Analysis
status: draft
owner: test-guardian
created: 2026-06-22
tags:
  - testing
  - ui
  - screen-spec
  - test-gap
---

# Screen Spec Test Gaps

## 现有测试策略摘要

总测试规范 `test-spec.md` 将 UI 视觉和动画测试列为 Non-goal（Section 1.1），但 UI 专项测试文档 `docs/06-ui/05-testing/testing.md` 定义了三层 UI 测试架构：Widget 单元测试（Contract 合规 / 渲染正确性 / 交互响应）、Screen 集成测试（Widget 组合完整性 / ViewModel 绑定 / 导航流程）、UI 快照测试（Entity 层级 / Component 结构一致性），并配套 Mock Projection / Test Fixtures 基础设施。当前代码库已实现 4 个测试套件（Dirty 生命周期 9 个、ScreenStack 导航 14 个、FocusManager 焦点 15 个、BattleProjection 投影 7 个），共 46 个测试用例，均属纯数据结构/函数级测试。Screen 集成测试和快照测试当前为 0 覆盖率。

## 新增测试类型

### Type 1: Screen Spec Compliance Test（Screen 规格合规测试）

验证 Screen 的实际 UI 树实现是否与 SSPEC 文件定义的 Widget Tree、region_id、widget_count、container_count 一致。测试读取 SSPEC 的 P0 字段（region_id 列表、Widget Tree 结构、Flexbox 尺寸），在最小 Bevy App 中 spawn Screen 后通过 ECS query 验证所有预期 region_id 存在、无多余匿名节点、max_depth ≤ 6、widget_id 与 SSPEC 交叉引用无遗漏。这不是视觉测试，而是结构合约测试——确保实现不偏离规范。

### Type 2: Per-Region State Rendering Test（逐区域状态渲染测试）

SSPEC Section 8 要求每个 region 独立定义 Loading / Empty / Normal / Error 四种状态。新测试类型为每个 region 的每种状态提供独立的 Mock ViewModel 数据，验证对应 UI 元素正确呈现：Loading 状态显示骨架屏或 Spinner（不显示数据或错误），Empty 状态显示空状态提示文本（不含 Loading 指示器或数据内容），Normal 状态显示正常数据，Error 状态显示错误提示和重试按钮。每种状态切换由 ViewModel 中的 `RegionState` 枚举字段控制，测试验证同一个 region Entity 上仅出现对应状态的 Component，不出现其他状态的残留 Component。

### Type 3: State Transition Test（状态转换测试）

验证 region 状态间的合法转换正确执行，非法转换被禁止。加载完成应触发 Loading → Normal（清除 Spinner 组件，渲染数据组件），加载失败应触发 Loading → Error（显示错误 UI），重试应触发 Error → Loading（切换回加载指示器），数据清空应触发 Normal → Empty（数据组件消失，空状态提示出现）。转换通过 Projection 修改 ViewModel 的 `RegionState` 字段并标记 Dirty<T> 触发，测试验证转换后的 UI Entity 组件结构正确替换。

### Type 4: Multi-Region Composite State Test（多区域复合状态测试）

Screen 的不同 region 可以处于不同状态。测试覆盖一个 region 为 Loading 而另一个为 Normal、一个为 Error 另一个为 Empty、全部为 Loading、全部为 Normal 等组合场景。例如：BattleScreen 的 `battle_area` 区域正在 Loading（地图数据未就绪）时，`top_bar` 区域已经处于 Normal（回合信息已就绪）。测试验证各 region 的状态独立且互不干扰——一个 region 的 Spinner 不应影响相邻 region 的数据渲染。

### Type 5: Screen Factory Function Test（Screen 工厂函数测试）

验证 `spawn_{screen_name}()` 工厂函数的行为合约：首次调用生成正确的 UI 树（所有 region 存在、Screen marker 组件正确设置），重复调用不产生重复根节点（no-op 或 despawn 后重建），`despawn_{screen_name}()` 后所有标记了 Screen marker 的实体被完全清理（无实体残留）。工厂函数的输出应与 SSPEC Widget Tree 结构一一对应——测试不是验证 Entity 数量，而是验证 region_id 的集合与 SSPEC 定义的集合一致。

### Type 6: Screen Snapshots via SSPEC（基于 SSPEC 的 UI 快照测试）

扩展现有快照测试（testing.md Section 4），将快照内容与 SSPEC 的 Widget Tree 绑定。快照文件命名包含 Screen 名称和状态场景（如 `battle_screen_top_bar_loading.snap`），快照结构（entity_count、层级、region_id）与 SSPEC 的 Widget Tree 和 Screen Metrics 基线交叉校验——CI 阶段自动对比快照与 SSPEC 的 `max_depth` 和 `widget_count`，超过阈值时告警。每次结构性变更（新增/删除 region）必须先更新 SSPEC 再更新快照，确保文档-代码双同步。

### Type 7: SSPEC Cross-Reference Invariant Test（SSPEC 交叉引用不变量测试）

自动校验 SSPEC 文件内部的引用一致性：所有在 Flexbox Layout / State Mapping / Interaction Zones / Region Responsibility 中引用的 region_id，都必须在 Widget Tree 中存在（无孤立引用）；所有 Widget Tree 中的 leaf widget 都有对应的 Widget Contract（无未定义组件）；所有 referenced widget_id 没有拼写错误。这是纯文件级测试（不启动 Bevy App），在内容加载管线中对 SSPEC YAML/JSON Schema 做静态分析。

## 优先级建议

**批次 1（P0，与 SSPEC 实现同步）**：
- Type 5 (Screen Factory Function Test) — 最基础，blocking。不验证工厂函数正确性，后续所有测试无意义。
- Type 1 (Screen Spec Compliance Test) — 确保实现与 SSPEC 对齐，是 SSOT（Single Source of Truth）的自动化保障。
- Type 7 (SSPEC Cross-Reference Invariant Test) — 纯文件级，成本最低但收益最大，防止 SSPEC 自身缺陷导致的 AI 代码偏差。

**批次 2（P0，在首批 Screen Spec 升级到 active 后）**：
- Type 2 (Per-Region State Rendering Test) — 每个 region 独立测试，适合并行编写。
- Type 3 (State Transition Test) — 依赖 Type 2 的 Mock ViewModel 基础设施，在 ViewModel 加入 `RegionState` 枚举后实现。

**批次 3（P1，在首批 Screen 的 ViewModel 集成完成后）**：
- Type 4 (Multi-Region Composite State Test) — 验证 region 间状态隔离，场景较多，安排在 region 级测试稳定后。
- Type 6 (Screen Snapshots via SSPEC) — 依赖快照基础设施（`capture_ui_tree` 辅助函数当前为 `todo!()`），且需要 SSPEC Screen Metrics 基线稳定，安排在所有 Screen 的 SSPEC 进入 active 后。
