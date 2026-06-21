# battle_rules.md → battle_rules_v1.md 更新差异

Version: 1.0 → 1.1

---

## 变更总览

| 变更类型 | 数量 |
|----------|------|
| 新增 | 14 |
| 修改 | 18 |
| 删除 | 0 |

---

## 逐项变更

### 1. 核心原则（修改）

**位置**：核心原则

**变更类型**：修改

**原内容**：
```
- Logic / Presentation 分离
- ECS 是数据流，不是调用链
- Message 负责跨 Feature 广播
```

**新内容**：
```
- 🟥 Logic / Presentation 分离（宪法 1.1.4）
- 🟥 ECS 是数据流，不是调用链（宪法 2.3.1）
- 🟥 Message 负责跨 Feature 广播（宪法 5.0）
- 🟥 Rule / Content 分离（宪法 1.1.3）
- 🟩 组合优于继承（宪法 1.1.6）
```

**原因**：宪法要求所有规则标注强制等级；补充宪法条款引用；新增 Rule/Content 分离和组合优于继承原则。

---

### 2. 术语定义新增 EffectHandler（新增）

**位置**：术语定义

**变更类型**：新增

**新内容**：
```
## EffectHandler

效果处理器 Trait，定义效果的生成和执行逻辑。

不是 enum+match 分发。EffectHandler 通过注册表分发，新增效果类型只需实现 Trait 并注册。

关键属性：
- generate()：生成效果
- execute()：执行效果
```

**原因**：宪法 6.0.2 要求 Trait 用于扩展点，EffectHandler 是效果分发的核心 trait，应在术语中定义。

---

### 3. 不变量1-6 添加强制等级（修改）

**位置**：不变量1-6

**变更类型**：修改

**原内容**：无强制等级标记

**新内容**：
```
不变量1：EffectQueue 执行后清空 🟥
不变量2：伤害下限 🟥
不变量3：治疗上限 🟩
不变量4：死亡判定一致性 🟥
不变量5：管线严格顺序 🟥
不变量6：CombatIntent 消费后清除 🟩
```

**原因**：宪法要求所有规则标注强制等级。EffectQueue清空、伤害下限、死亡判定、管线顺序为🟥绝对禁止级别。

---

### 4. 不变量4 新增架构违规检测（修改）

**位置**：不变量4

**变更类型**：修改

**新增内容**：
```
架构违规检测：
发现 HP ≤ 0 的单位缺少 Dead Tag 时，必须停止。必须输出：
ARCHITECTURE VIOLATION: HP ≤ 0 的单位缺少 Dead Tag，违反死亡判定一致性不变量。
```

**原因**：架构文档定义了标准违规检测模式，关键不变量应纳入。

---

### 5. 不变量5 新增架构违规检测（修改）

**位置**：不变量5

**变更类型**：修改

**新增内容**：
```
架构违规检测：
发现效果处理跳步时，必须停止。必须输出：
ARCHITECTURE VIOLATION: 效果管线跳步 [从 XXX 直接到 XXX]，违反 Generate → Modify → Execute 严格顺序。
```

**原因**：管线跳步是核心架构违规，应有明确检测模式。

---

### 6. 不变量7：属性修改必须通过 Modifier（新增）

**位置**：不变量

**变更类型**：新增

**新内容**：
```
## 不变量7：属性修改必须通过 Modifier 🟥

宪法依据：2.2.1（禁止直接修改最终属性值）

任意时刻：
所有属性修改必须通过 Modifier 管线，禁止直接修改 HP 等最终属性值（Execute 阶段扣血除外，因为 Execute 是管线的终点执行）。

违反表现：
在 Generate 或 Modify 阶段直接修改 HP。
```

**原因**：宪法 2.2.1 明确禁止直接修改最终属性值，这是核心不变量，原文档缺失。

---

### 7. 规则1-6 添加强制等级和宪法引用（修改）

**位置**：业务规则1-6

**变更类型**：修改

**原内容**：无强制等级标记，无宪法引用

**新内容**：
```
规则1：效果管线 🟥
宪法依据：1.1.4（Logic/Presentation 分离）、2.3.1（ECS 是数据流）

规则2：CombatIntent 🟥
宪法依据：7.0.5（CombatIntent 是唯一攻击意图通道）

规则3：死亡处理 🟥
宪法依据：2.1.4（禁止用 bool 代替 Tag Component）

规则4：Trait 触发 🟩
宪法依据：6.0.2（Trait 用于扩展点）

规则5：行动路由 🟩
规则6：战斗记录 🟩
```

**原因**：宪法要求所有规则标注强制等级和宪法条款引用。

---

### 8. 规则1 禁止事项添加强制等级（修改）

**位置**：规则1

**变更类型**：修改

**原内容**：
```
禁止：
- 跳过管线直接执行效果
- 跳过 Modify 阶段
- 在 Generate 阶段修改 HP
- 在 Modify 阶段发送 BattleMessage
- 在 Execute 阶段创建新的 CombatIntent
```

**新内容**：
```
禁止：
- 🟥 跳过管线直接执行效果
- 🟥 跳过 Modify 阶段
- 🟥 在 Generate 阶段修改 HP
- 🟥 在 Modify 阶段发送 BattleMessage
- 🟥 在 Execute 阶段创建新的 CombatIntent
```

**原因**：宪法要求所有禁止事项标注强制等级。

---

### 9. 规则2 新增架构违规检测（修改）

**位置**：规则2

**变更类型**：修改

**新增内容**：
```
架构违规检测：
发现绕过 CombatIntent 直接发起攻击时，必须停止。必须输出：
ARCHITECTURE VIOLATION: 绕过 CombatIntent 直接发起攻击，违反"CombatIntent 是唯一攻击意图通道"原则。
```

**原因**：架构 AI 模块边界明确规定 CombatIntent 是唯一攻击意图通道，应有违规检测。

---

### 10. 规则3 新增禁止 is_dead: bool（修改）

**位置**：规则3

**变更类型**：修改

**新增内容**：
```
禁止：
- 🟥 用 is_dead: bool 代替 Dead Tag
```

**原因**：宪法 2.1.4 明确禁止用 bool 代替 Tag Component。

---

### 11. 规则7：EffectHandler trait 分发（新增）

**位置**：业务规则

**变更类型**：新增

**新内容**：
```
## 规则7：EffectHandler trait 分发 🟩

宪法依据：6.0.2（Trait 用于扩展点）

禁止：
- 🟥 match 分发效果类型
- 🟥 新增效果类型时修改管线流程

必须：
- 通过 EffectHandlerRegistry 查找 trait 对象
- 新增效果类型只需实现 EffectHandler trait 并注册
```

**原因**：宪法 6.0.2 要求 Trait 用于扩展点，效果分发应通过 trait 而非 match，原文档缺失此规则。

---

### 12. 流程管线步骤添加强制等级（修改）

**位置**：流程管线 → Step1-4

**变更类型**：修改

**原内容**：
```
禁止：修改 HP、发送 Message、跳过前置检查
禁止：修改 HP、发送 Message、创建新效果
禁止：创建新 CombatIntent、跳过死亡判定
禁止：在 Message 处理中修改战斗状态
```

**新内容**：
```
🟥 禁止：修改 HP、发送 Message、跳过前置检查
🟥 禁止：修改 HP、发送 Message、创建新效果
🟥 禁止：创建新 CombatIntent、跳过死亡判定
🟥 禁止：在 Message 处理中修改战斗状态
```

**原因**：宪法要求所有禁止事项标注强制等级。

---

### 13. 数据结构添加强制等级（修改）

**位置**：数据结构 → CombatIntent / EffectQueue / PendingEffect / DamageBreakdown / BattleRecord / EntityBattleStats

**变更类型**：修改

**新增内容**：
```
CombatIntent: 🟥 玩家和 AI 共用（宪法 7.0.5）、🟥 Execute 后必须清除、🟩 不可当全局变量仓库（宪法 2.1.5）
EffectQueue: 🟥 Generate 推入，Modify 修改，Execute drain 清空、🟥 Execute 后 pending 必须为空
PendingEffect: 🟥 必须经过 Modify → Execute
BattleRecord: 🟩 不可当全局变量仓库（宪法 2.1.5）
```

**原因**：宪法要求所有规则标注强制等级和宪法条款引用。

---

### 14. 禁止事项添加强制等级和新增3条（修改+新增）

**位置**：禁止事项

**变更类型**：修改 + 新增

**修改内容**：原有6条禁止事项全部添加🟥标记和宪法引用

**新增内容**：
```
🟥 禁止：用 is_dead: bool 代替 Dead Tag
原因：宪法 2.1.4 明确禁止用 bool 代替 Tag Component

🟥 禁止：match 分发效果类型
原因：应通过 EffectHandler trait 分发（宪法 6.0.2）

🟥 禁止：绕过 CombatIntent 直接发起攻击
原因：CombatIntent 是唯一攻击意图通道（宪法 7.0.5）
架构违规检测：ARCHITECTURE VIOLATION: 绕过 CombatIntent 直接发起攻击...
```

**原因**：宪法 2.1.4、6.0.2、7.0.5 明确要求；架构 AI 模块边界定义了违规检测模式。

---

### 15. AI 修改规则添加强制等级和测试要求（修改）

**位置**：AI 修改规则

**变更类型**：修改

**新增内容**：
```
禁止：
- 🟥 match 分发效果类型（新增效果类型场景）
- 🟥 在触发器中处理非 ApplyBuff 效果（Trait 触发场景）
- 🟥 在 Execute 中直接调用 UI（新增消息场景）

测试要求（宪法 13.0.1-13.0.3）：
- 🟩 单元测试：验证管线各阶段输入输出
- 🟩 集成测试：验证完整战斗流程
- 🟩 Bug 修复必须先编写重现测试（宪法 13.0.2）
- 🟩 Battle Replay 优先于手工验证（宪法 1.1.9）
```

**原因**：宪法要求所有禁止事项标注强制等级；宪法 13.0.1-13.0.3 要求测试优先；宪法 1.1.9 要求 Battle Replay 优先。

---

### 16. 宪法条款映射（新增）

**位置**：文档末尾

**变更类型**：新增

**新内容**：
```
| 宪法条款 | 本领域对应 |
|----------|-----------|
| 1.1.3 Rule/Content 分离 | EffectHandler trait 是规则，RON 配置是内容 |
| 1.1.4 Logic/Presentation 分离 | 战斗逻辑不包含 UI/动画逻辑 |
| 1.1.9 测试优先 | Battle Replay + 自动化测试优先 |
| 2.1.1 Entity 只是 ID | source_entity / target 仅作 ID |
| 2.1.4 禁止 bool 代替 Tag | Dead Tag 代替 is_dead: bool |
| 2.1.5 Resource 不是全局仓库 | CombatIntent / EffectQueue 有明确职责 |
| 2.1.6 禁止手写状态标记 | 使用 Added/Changed/Removed 检测 |
| 2.2.1 禁止直接修改最终属性 | HP 修改只在 Execute 阶段 |
| 2.3.1 ECS 是数据流 | 管线是数据流，不是调用链 |
| 5.0 通信三原则 | Message 负责跨 Feature 广播 |
| 6.0.2 Trait 用于扩展点 | EffectHandler trait 分发 |
| 7.0.5 战斗事件链 | CombatIntent 是唯一攻击意图通道 |
```

**原因**：便于 AI 快速定位宪法条款与领域规则的对应关系。

---

### 17. 架构违规检测（新增）

**位置**：文档末尾

**变更类型**：新增

**新内容**：
```
| 违规行为 | 检测方式 | 输出 |
|----------|----------|------|
| 效果管线跳步 | 代码审查 | ARCHITECTURE VIOLATION: 效果管线跳步... |
| HP ≤ 0 缺少 Dead Tag | 代码审查 | ARCHITECTURE VIOLATION: HP ≤ 0 的单位缺少 Dead Tag... |
| 绕过 CombatIntent 发起攻击 | 代码审查 | ARCHITECTURE VIOLATION: 绕过 CombatIntent 直接发起攻击... |
| Generate/Modify 阶段修改 HP | 代码审查 | ARCHITECTURE VIOLATION: 在 [Generate/Modify] 阶段直接修改 HP... |
```

**原因**：架构文档定义了标准违规检测模式，领域文档应纳入。

---

## 变更统计

| 强制等级 | 原文档数量 | 新文档数量 |
|----------|-----------|-----------|
| 🟥 绝对禁止 | 0 | 18 |
| 🟩 必须遵守 | 0 | 10 |
| 宪法条款引用 | 0 | 12 |
| 架构违规检测 | 0 | 4 |
