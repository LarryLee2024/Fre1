
# Bug与技术债专项规则（AI提示词精简版）
> 与《Bevy SRPG AI开发宪法v1.5》配套，优先级等同
> 核心原则：Warning预算=0，Bug预算=0，技术债可控可追溯

---

## 核心分级标准
### P0 致命级（发现即修，禁止提交）
- 业务代码出现unwrap/expect/panic/todo/unimplemented
- ECS Query Borrow冲突、事件/Observer无限循环
- Entity泄漏、存档兼容性损坏、核心数据损坏
- 任何情况下不得进入主分支

### P1 高优先级（一个迭代内修复，CI拦截）
- Rust必修警告：unused_imports、dead_code、deprecated、unused_variables等
- Clippy必修项：clone_on_copy、redundant_clone、needless_collect等
- 已废弃API调用、配置资源引用失效
- 主分支不允许存在未处理的必修项警告

### P2 中优先级（登记跟踪，阈值触发处理）
- 非热点代码性能问题、ECS Archetype抖动
- 非核心路径日志噪音
- 记录入技术债台账，达到阈值统一优化

### P3 低优先级（技术债，集中偿还）
- 局部重复代码、命名不统一、文件偏大
- 架构复盘时集中清理，不影响当前迭代

---

## 强制要求
1. 业务领域代码绝对禁止unwrap/expect/panic/todo，仅测试/工具/原型可豁免
2. 新增核心业务系统必须至少附带1个基础测试用例
3. 新增领域事件必须同步纳入事件白名单文档
4. 所有警告豁免必须标注理由与有效期，禁止无理由屏蔽检查
5. Bevy专项必须检查：Commands滥用、事件洪泛、Observer递归、Query过大

---

## 绝对禁令
❌ 业务代码提交todo!/unimplemented!占位
❌ 无理由屏蔽Rust警告与Clippy检查
❌ 配置引用失效仍合并入主分支
❌ 为了规避规范机械改写代码，破坏业务语义
❌ 技术债只记录不跟踪，无限期积压

---

## 生成后自检
✅ 业务代码无panic/unwrap/expect/todo
✅ 无必修级Rust与Clippy警告
✅ 新增核心系统附带对应测试
✅ 新增领域事件已纳入白名单
✅ 问题分级准确，技术债已登记
✅ 主分支准入条件全部满足