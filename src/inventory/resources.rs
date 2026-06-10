// 资源统一化：ResourceStack / Resources
// 金币/银币/木材/铁矿/声望/贡献点统一管理

use bevy::prelude::*;

/// 资源堆叠（金币/银币/木材/铁矿/声望/贡献点）
#[derive(Clone, Debug)]
pub struct ResourceStack {
    pub resource_id: String,
    pub amount: u32,
}

/// 资源容器（挂在角色 Entity 上）
#[derive(Component, Default, Debug, Clone)]
pub struct Resources {
    pub stacks: Vec<ResourceStack>,
}

impl Resources {
    pub fn add(&mut self, resource_id: &str, amount: u32) {
        if let Some(stack) = self
            .stacks
            .iter_mut()
            .find(|s| s.resource_id == resource_id)
        {
            stack.amount += amount;
        } else {
            self.stacks.push(ResourceStack {
                resource_id: resource_id.into(),
                amount,
            });
        }
    }

    pub fn spend(&mut self, resource_id: &str, amount: u32) -> bool {
        if let Some(stack) = self
            .stacks
            .iter_mut()
            .find(|s| s.resource_id == resource_id)
        {
            if stack.amount >= amount {
                stack.amount -= amount;
                return true;
            }
        }
        false
    }

    pub fn get(&self, resource_id: &str) -> u32 {
        self.stacks
            .iter()
            .find(|s| s.resource_id == resource_id)
            .map(|s| s.amount)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 资源_添加() {
        let mut res = Resources::default();
        res.add("gold", 100);
        assert_eq!(res.get("gold"), 100);
    }

    #[test]
    fn 资源_累加() {
        let mut res = Resources::default();
        res.add("gold", 50);
        res.add("gold", 30);
        assert_eq!(res.get("gold"), 80);
    }

    #[test]
    fn 资源_消费成功() {
        let mut res = Resources::default();
        res.add("gold", 100);
        assert!(res.spend("gold", 50));
        assert_eq!(res.get("gold"), 50);
    }

    #[test]
    fn 资源_消费不足() {
        let mut res = Resources::default();
        res.add("gold", 30);
        assert!(!res.spend("gold", 50));
        assert_eq!(res.get("gold"), 30);
    }

    #[test]
    fn 资源_查询不存在的资源() {
        let res = Resources::default();
        assert_eq!(res.get("silver"), 0);
    }

    #[test]
    fn 资源_多种资源() {
        let mut res = Resources::default();
        res.add("gold", 100);
        res.add("silver", 50);
        res.add("fame", 10);
        assert_eq!(res.get("gold"), 100);
        assert_eq!(res.get("silver"), 50);
        assert_eq!(res.get("fame"), 10);
    }
}
