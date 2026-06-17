//! Tools 层：开发工具（Layer 7）
//!
//! Layer 7 职责：提供开发期间的工具链，永不进入发布构建。
//!
//! Tools 是**独立 Cargo 二进制**，位于项目根 `tools/` 目录下，不作为此 crate 的一部分。
//! 本模块仅作为架构文档映射，记录 Tools 层的设计约定。
//!
//! ## 工具清单
//!
//! | 工具 | 说明 | 实现阶段 |
//! |------|------|---------|
//! | `data_validator` | 数据验证器：检查 RON 配置的引用完整性与格式正确性 | Phase 1 |
//! | `content_linter` | 内容 Lint：规范检查、缺失字段、未使用资源 | Phase 2 |
//! | `balance_checker` | 数值平衡分析：批量战斗模拟、伤害分布统计 | Phase 3 |
//! | `replay_inspector` | 回放查看器：逐帧回放战斗过程 | Phase 4 |
//! | `save_inspector` | 存档查看器：数据完整性检查与浏览 | Phase 5 |
//! | `schedule_dumper` | 调度图导出：SystemSet 依赖可视化 | Phase 6 |
//! | `config_diff_analyzer` | 配置差异分析：内容版本对比 | Phase 7 |
//! | `content_editor` | 内容编辑器（GUI） | Future |
//! | `map_editor` | 地图编辑器（GUI） | Future |
//!
//! ## 设计原则
//!
//! - 🟥 永不进入 Release 构建（feature gate: `#[cfg(feature = "tools")]`）
//! - 🟥 禁止修改游戏数据（只读分析）
//! - 🟥 禁止包含业务逻辑
//! - 🟩 支持 headless 模式用于 CI
//! - 🟩 独立 Cargo binary，按需编译
//!
//! ## 依赖规则
//!
//! Tools → Shared  ✅ 允许
//! Tools → Content ✅ 允许（读取 RON 文件）
//! Tools → Core    ❌ 禁止（通过读取 Registry 数据替代）
//! Tools → Infra   ❌ 禁止
//!
//! 详细设计见 `docs/architecture/tools_architecture.md`。
//! 参见 `docs/architecture/layer-contracts.md` §第七层。
