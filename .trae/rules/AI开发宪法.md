---
alwaysApply: false
description: AI生成任何代码前必须通读本宪法，生成后必须完成自检
---
# Bevy 0.19+ SRPG 项目总宪法 v5.0 紧凑执行版

## 效力说明
本宪法对AI生成代码具有**最高约束力**，优先级高于任何通用编程规范。条款强制等级：
- 🟥 **绝对禁止**：任何情况不允许，**不可豁免**
- 🟩 **必须遵守**：无例外强制执行，违反即不合格
- 🟨 **优先选择**：无明确技术理由必须采用
- ⚠️ **警觉阈值**：达到阈值AI必须主动提出重构

所有违反条款的代码必须标注`[宪法豁免]`并说明理由，每3个月重新评估。

---

## 🔴 AI最高优先级11条（超越所有其他条款）
1. 🟥 **Feature First**：按业务拆模块，绝不按技术拆模块
2. 🟥 **定义与实例强制分离**：配置与运行时状态完全隔离
3. 🟥 **规则与内容强制分离**：代码只实现规则，配置只定义内容
4. 🟥 **逻辑与表现强制分离**：业务逻辑与UI/特效完全隔离
5. 🟥 **四级通信机制**：Hook=生命周期、Trigger=事件链、Observer=局部响应、Message=跨域广播
6. 🟥 **双轴架构原则**：Capabilities管机制，Domains管业务，边界不可突破
7. 🟥 **数据驱动绝对优先**：新增内容绝不修改逻辑代码
8. 🟩 **小单元原则**：小函数、小模块、小依赖
9. 🟥 **测试优先**：Battle Replay+自动化测试优先于手工验证
10. 🟥 **组合绝对优先于继承**
11. 🟥 **Localization First**：所有用户可见文本必须使用 LocalizationKey，禁止硬编码自然语言文本；Event/存档只存 Key+参数，不存翻译文本

---

## 🟥 核心禁令速查
### 架构与ECS
- 🟥 禁止创建`components/systems/events/utils`巨文件
- 🟥 禁止把Entity当对象调用方法，禁止模拟`player.attack(enemy)`
- 🟥 禁止Component包含逻辑、System存储状态
- 🟥 禁止用bool代替Tag Component（用`Dead`不用`is_dead: bool`）
- 🟥 禁止手写状态标记，必须用`Added/Changed/Removed`
- 🟥 禁止把Resource当全局变量仓库
- 🟥 禁止同一模块内滥用事件模拟函数调用
- 🟩 组件依赖用`#[require(Component)]`，组件Hook用`#[component(on_add=...)]`

### 系统设计
- 🟥 禁止直接修改最终属性值，所有修改必须通过Modifier
- 🟥 禁止业务逻辑直接操作UI，禁止UI保存业务真相或修改业务状态
- 🟥 禁止Reflect参与核心运行时逻辑和高频计算
- 🟥 禁止凭直觉优化性能，所有优化必须基于Profile数据
- 🟥 禁止每帧打印Info/Debug级别日志，仅允许Error级别
- 🟩 派生属性优先实时计算，属性公式集中管理
- 🟩 跨模块写操作走Message/Trigger/Command，读操作走Query API

### 代码规范
- 🟥 禁止创建超过两层的继承树，禁止用继承实现角色差异化
- 🟥 禁止为单个实现创建Trait，禁止为了"优雅"创建无价值Trait
- 🟥 禁止为未明确的未来需求提前设计架构
- ⚠️ 文件>500行警觉，>1000行强制拆分
- ⚠️ 函数>100行警觉，>3层嵌套必须重构
- 🟩 代码重复3次以上再抽象，可读性优先于复用性

---

## 🤖 AI专属执行规范
### 反模式黑名单（生成前必须对照）
1. ❌ 把Entity当面向对象实例
2. ❌ 把Resource当全局变量仓库
3. ❌ 创建全局顶层`systems.rs/components.rs`巨文件（Domain 内部按需拆分，详见 `ai-constitution-complete.md` §3.4）
4. ❌ 滥用事件系统模拟函数调用
5. ❌ 业务逻辑直接操作UI
6. ❌ 直接修改最终属性值
7. ❌ 为单个实现创建Trait
8. ❌ 提前为未来需求过度设计
9. ❌ 手写状态变更检测
10. ❌ 每帧打印Info/Debug日志
11. ❌ 在Capabilities层硬编码业务规则
12. ❌ Domain之间直接use对方内部类型
13. ❌ 在 Definition 中硬编码用户可见文本（必须使用 LocalizationKey）

### AI自检（文档参考，不输出到代码）

> 此清单仅作为内部参考，不要求在代码中输出自检结果。
> 合规检查依赖CI门禁（cargo clippy / dependency_checker / 架构扫描）。

| 检查项 | 说明 |
|--------|------|
| Feature First | 按业务拆分模块 |
| Definition/Instance分离 | 配置与运行时分离 |
| Rule/Content分离 | 代码只实现规则 |
| Logic/Presentation分离 | 逻辑与表现分离 |
| Modifier管线 | 属性修改走Modifier |
| 双轴边界 | Capabilities无业务规则，Domain无重复机制 |
| 读路径无副作用 | 预览/仿真不修改状态 |
| 统一RNG | 不直接调用rand::random() |
| Localization First | 文本必须用Key，禁止硬编码，存档/Event只存Key+参数 |

---

**版本**：v5.0（Bevy 0.19+） | **发布日期**：2026-06-16
**执行要求**：AI生成任何代码前必须通读本宪法。
