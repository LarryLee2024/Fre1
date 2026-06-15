//! ContentPlugin：内容加载协调器
//!
//! Layer 5 职责：只做"加载→校验→注册"，不包含游戏规则。
//! 当前内容加载由各 core Plugin 在 build() 中直接调用 load_from_dir。
//! ContentPlugin 作为统一入口，记录加载顺序合约，未来可逐步收拢加载逻辑。
//!
//! ## 加载顺序合约
//!
//! 内容类型存在隐式依赖关系，加载时应遵循以下顺序：
//!
//! 1. definitions/     — 属性、标签定义（无依赖）
//! 2. terrains/       — 地形类型（无依赖）
//! 3. modifiers/      — 修饰规则（无依赖）
//! 4. skills/         — 技能定义（无依赖）
//! 5. buffs/          — Buff 定义（无依赖）
//! 6. ai_behaviors/   — AI 行为模板（无依赖）
//! 7. characters/     — 单位模板（引用 skills）
//! 8. classes/        — 职业与特质（引用 skills/buffs）
//! 9. equipments/     — 装备定义（引用 modifiers）
//! 10. items/         — 物品定义（引用 skills/buffs/modifiers）
//! 11. stages/        — 关卡配置（依赖 terrains）
//! 12. campaigns/     — 战役定义（依赖 stages）

use bevy::prelude::*;

/// 内容加载协调插件
///
/// 确保所有游戏内容在正确时机完成加载与注册。
/// 当前行为：各核心模块在各自 Plugin 中独立加载。
/// 未来规划：将 `load_from_dir` 调用从 core Plugin 迁移至此。
pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, _app: &mut App) {
        // Phase 1.4: 初步收拢加载逻辑
        // 当前内容加载由各 core Plugin 自行处理。
        // ContentPlugin 作为契约声明，未来可在此集中管理：
        //
        // 1. 注册所有 Content 子模块的加载系统
        // 2. 确保跨内容类型的校验（如 Stage 引用的 terrain_id 必须存在）
        // 3. 集中化错误处理与恢复策略
        //
        // 参见 ADR-004 §4.3 迁移计划。
        //
        // 注意：当前保持原有加载逻辑不变，避免破坏现有功能。
        // 后续 Phase 将逐步迁移加载逻辑至此。
    }
}
