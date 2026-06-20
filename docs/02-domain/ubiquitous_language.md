# 项目统一术语表（Ubiquitous Language）

> 基于 31 个领域文件的"统一术语"节和 L0 Vocabulary 层汇总。
> 所有核心业务术语有唯一名称，代码类型名/函数名/配置 Key 必须与此表一致。

## 核心术语

| 术语 | 英文标识 | 定义 | 所在模块 |
|------|---------|------|---------|
| 实体 | Entity | ECS 实体，纯 ID | bevy_ecs |
| 单位 | Unit | 战斗中的角色实例 | combat |
| 角色 | Character | 配置模板/定义 | content |
| 技能 | Ability | 主动能力 | ability |
| 效果 | Effect | 技能产生的效果 | effect |
| 修改器 | Modifier | 数值修正器 | modifier |
| 条件 | Condition | 条件判断 | condition |
| 标签 | Tag | 语义分类标签 | tag |
| 触发器 | Trigger | 条件触发机制 | trigger |
| 法术位 | SpellSlot | 法术消耗资源 | spell |
| 专注 | Concentration | 施法专注状态 | spell |
| 反应 | Reaction | 回合外响应 | reaction |
| Buff | Buff | 临时状态效果 | effect |
| 堆叠 | Stacking | 效果叠加规则 | stacking |
| 属性 | Attribute | 角色数值属性 | attribute |
| 聚合 | Aggregator | 属性聚合计算 | aggregator |
| 执行 | Execution | 效果执行计算 | execution |
| 定位 | Targeting | 目标选择 | targeting |
| 提示 | Cue | 表现层信号 | cue |

> 新增术语必须经 domain-designer 审批，并更新此表。
