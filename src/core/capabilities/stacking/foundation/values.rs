//! Stacking 值对象
//!
//! 定义堆叠运行时状态与标识值。
//!
//! 详见 docs/04-data/capabilities/stacking_schema.md §3.4。

/// 堆叠运行时状态（Instance 层）。
///
/// 维护当前堆叠计数、上限和成员追踪。
/// 不变量 3.1: stack_count ≤ max_stacks。
#[derive(Debug, Clone, PartialEq)]
pub struct StackingState {
    /// 当前堆叠层数
    pub stack_count: u32,
    /// 最大上限
    pub max_stacks: u32,
    /// 活跃堆叠成员实例 ID 列表
    pub stack_members: Vec<String>,
}

impl StackingState {
    /// 创建新的堆叠状态。
    ///
    /// 初始层数为 1，表示效果第一次被施加。
    ///
    /// # Errors
    /// - V1: max_stacks ≥ 1
    pub fn new(max_stacks: u32) -> Result<Self, super::types::StackingError> {
        if max_stacks < 1 {
            return Err(super::types::StackingError::InvalidConfig(
                "max_stacks must be ≥ 1".into(),
            ));
        }
        Ok(Self {
            stack_count: 1,
            max_stacks,
            stack_members: Vec::new(),
        })
    }

    /// 增加堆叠层数。
    ///
    /// 不变量 3.1: 堆叠层数不得超过 max_stacks。
    /// 返回实际增加的层数（受上限限制）。
    pub fn add_layers(&mut self, layers: u32) -> u32 {
        let new_count = self.stack_count.saturating_add(layers);
        let actual = if new_count > self.max_stacks {
            let added = self.max_stacks - self.stack_count;
            self.stack_count = self.max_stacks;
            added
        } else {
            self.stack_count = new_count;
            layers
        };

        // 不变量 3.1: 堆叠层数不得超过 max_stacks
        debug_assert!(
            self.stack_count <= self.max_stacks,
            "stack_count ({}) exceeded max_stacks ({})",
            self.stack_count,
            self.max_stacks
        );

        actual
    }

    /// 减少堆叠层数。
    ///
    /// 减少到 0 视为 1（至少 1 层效果仍然存在）。
    pub fn remove_layers(&mut self, layers: u32) {
        if layers >= self.stack_count {
            self.stack_count = 1;
        } else {
            self.stack_count -= layers;
        }
    }

    /// 检查是否已达到上限。
    pub fn is_at_max(&self) -> bool {
        self.stack_count >= self.max_stacks
    }

    /// 添加一个成员实例 ID 到追踪列表。
    pub fn add_member(&mut self, instance_id: impl Into<String>) {
        self.stack_members.push(instance_id.into());
    }

    /// 从追踪列表移除一个成员实例 ID。
    pub fn remove_member(&mut self, instance_id: &str) {
        self.stack_members.retain(|id| id != instance_id);
    }

    /// 获取当前层数。
    pub fn current_layers(&self) -> u32 {
        self.stack_count
    }

    /// 获取剩余可用层数（还可叠加多少层）。
    pub fn remaining_capacity(&self) -> u32 {
        self.max_stacks.saturating_sub(self.stack_count)
    }
}
