/// 调试面板查看器模块：所有 egui 调试面板的渲染逻辑集中在此

/// AI 决策状态查看器
pub mod ai_viewer;
/// 战斗状态快照查看器
pub mod battle_debugger;
/// Buff 状态查看器
pub mod buff_viewer;
/// 伤害分解与属性修饰查看器
pub mod damage_attribute_viewer;
/// 装备状态查看器
pub mod equipment_viewer;
/// 地形网格查看器
pub mod grid_viewer;
/// 游戏设置查看器
pub mod settings_viewer;
/// 回合队列查看器
pub mod turn_queue_viewer;

pub use grid_viewer::GridViewerState;
