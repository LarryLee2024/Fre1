---
alwaysApply: false
description: 
---
# AI BugFix Constitution v2.0
> 适用于：Bevy 0.19+ / DDD / SRPG
> 优先级仅次于《AI开发宪法》
> 核心原则：
> Bug Budget = 0
> Warning Budget = 0
> Architecture Debt Budget = 可追踪、可偿还

---

# 一、问题分级

## P0 - 致命级（立即修复，禁止提交）

### 程序正确性
* panic!
* unwrap()
* expect()
* todo!()
* unimplemented!()
（测试代码、工具代码除外）

### ECS正确性
* Query Borrow冲突
* Observer/Event无限递归
* Commands导致实体泄漏
* 生命周期错误
* 非预期状态机循环

### 数据正确性
* 存档损坏
* Replay失效
* Registry引用失效
* Content配置损坏
* 数据迁移失败

### 架构正确性
* 违反DDD边界
* Core依赖Infrastructure
* Shared变成业务模块
* 业务逻辑进入UI层
* 业务逻辑进入Plugin装配层

---

## P1 - 高优先级（CI拦截）
### Rust
* unused_imports
* dead_code
* deprecated
* unused_variables
* unused_must_use
* unreachable_code

### Clippy
* clone_on_copy
* redundant_clone
* needless_collect
* large_enum_variant
* await_holding_lock
* map_clone

### Bevy专项
* Event洪泛
* Query过大
* Commands滥用
* Archetype频繁抖动

### Content
* 缺失Skill引用
* 缺失Buff引用
* 缺失Localization Key
* 缺失Asset引用

---

## P2 - 中优先级（登记技术债）
### 性能
* 非热点N²循环
* 重复查询
* 非关键路径内存分配

### 工程
* 文件过大
* 模块职责偏重
* 轻度重复代码

---

## P3 - 低优先级（集中治理）
### 风格
* 命名不统一
* 注释风格不统一
* 文档缺失

---

# 二、AI强制检查项
每次提交前必须检查：

## 1. 边界检查
确认没有出现：
* Core -> Infrastructure依赖
* Domain -> UI依赖
* Domain -> Bevy表现层依赖

---

## 2. Event检查
新增事件时：
* 是否进入事件白名单
* 是否可能形成循环触发
* 是否需要Replay记录

---

## 3. Content检查
新增内容时：
* 是否拥有唯一ID
* 是否进入Registry
* 是否具备Localization Key
* 是否可被Validator检查

---

## 4. Replay检查
新增逻辑时：
确认：
* 不依赖thread_rng()
* 不依赖系统时间
* 不依赖不确定执行顺序

必须保证Replay可重放。

---

## 5. 测试检查
新增核心逻辑：
至少包含：
* 1个正常路径测试
满足以下任意情况时额外增加异常路径测试：
* 状态变化
* Buff处理
* Skill执行
* Turn推进
* Victory判定

---

# 三、绝对禁令

禁止：

* 临时修复绕过架构
* 为通过编译删除测试
* 为通过CI关闭检查
* 复制粘贴已有逻辑而不抽象
* 在业务代码中硬编码配置数据
* 使用字符串代替Id类型
* 使用Entity作为长期业务标识
* 使用thread_rng()作为游戏随机源

---

# 四、修复优先级原则

修复顺序：

P0 正确性
→ P1 稳定性
→ P2 性能
→ P3 风格

禁止：

为了修复P2/P3问题而引入新的P0/P1问题。

---

# 五、生成后自检

提交前必须满足：

[ ] 无 panic/unwrap/expect/todo
[ ] 无 Rust 必修警告
[ ] 无 Clippy 必修警告
[ ] 无失效引用
[ ] 无 DDD 边界破坏
[ ] 无 Replay破坏
[ ] 新增内容已注册Registry
[ ] 新增事件已进入白名单
[ ] 新增核心逻辑包含测试
[ ] 技术债已登记
[ ] 主分支准入条件全部满足

---

# 六、Fitness Function 门禁

Bug 修复提交前必须执行以下步骤：

### 运行架构预算检查
- 执行 `tools/check-architecture-budget.sh --ci` 进行全量检查
- 修复不得引入新的架构预算违规（函数超 50 行、文件超 500 行、导入方向违规等）
- 若修复过程必须突破预算，需在提交信息中注明 `[架构预算豁免]` 并附理由

### 回归保证
- 架构预算检查作为 CI 前置步骤，失败即阻塞合并
- 禁止为绕过检查而修改 Fitness Function 脚本本身
