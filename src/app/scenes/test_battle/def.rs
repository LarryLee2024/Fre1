//! Test Battle Definition — RON 配置反序列化结构体
//!
//! 定义测试战斗场景的配置结构，与 `assets/configs/scenarios/test_battle.ron` 对应。
//! 这些结构体仅是配置数据的容器，不含任何业务逻辑。
//!
//! 详见 ADR-052: 数据驱动的测试场景

use serde::Deserialize;

/// 测试战斗场景完整配置
#[derive(Debug, Clone, Deserialize)]
pub struct TestBattleDef {
    /// 单位列表
    pub units: Vec<UnitEntry>,
    /// 网格配置
    pub grid: GridConfig,
}

/// 单个单位配置条目
#[derive(Debug, Clone, Deserialize)]
pub struct UnitEntry {
    /// 单位标识符
    pub id: String,
    /// 本地化名称 Key
    pub name_key: String,
    /// 所属队伍（从 RON 反序列化为 TeamId string）
    pub team: String,
    /// 网格坐标 (x, y)
    pub coord: (i32, i32),
    /// 当前生命值
    pub hp: u32,
    /// 最大生命值
    pub max_hp: u32,
    /// 行动点数
    pub ap: u32,
}

/// 战场网格配置
#[derive(Debug, Clone, Deserialize)]
pub struct GridConfig {
    /// 网格宽度（格数）
    pub width: u32,
    /// 网格高度（格数）
    pub height: u32,
    /// 每格像素大小
    pub cell_size: f32,
}
