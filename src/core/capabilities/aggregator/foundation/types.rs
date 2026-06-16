//! Aggregator 基础类型定义

/// 计算阶段枚举。
///
/// 严格按此顺序执行: Add → Multiply → Override → Clamp
/// 枚举判别值编码了执行顺序:
/// - Add = 0, Multiply = 1, Override = 2, Clamp = 3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CalcStage {
    /// 加法阶段: Sum(所有 Add 类型 Modifier 的值)
    Add,
    /// 乘法阶段: Product(所有 Multiply 类型 Modifier 的值)
    /// 注意：乘法叠加是连乘而非加法
    Multiply,
    /// 覆盖阶段: 取优先级最高的 Override Modifier 的值
    Override,
    /// 钳制阶段: 限制在 [MinValue, MaxValue] 范围内
    Clamp,
}

/// 聚合管线中使用的修改器运算类型。
///
/// 与 `modifier::foundation::ModifierOp` 语义一致，
/// 此处独立定义以保持 aggregate 管道纯函数边界。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierOp {
    Add,
    Multiply,
    Override,
}

/// 管线错误类型。
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineError {
    /// 无效的 Clamp 边界（min > max）
    InvalidClampBounds { min: f32, max: f32 },
    /// 循环检测触发
    CycleDetected { cycle_chain: Vec<String> },
    /// 无有效 Modifier 但启用了 Override 阶段
    OverrideStageEmpty,
    /// 乘法阶段遇到零值（可能导致结果归零但非错误，仅警告级）
    MultiplyByZero,
}
