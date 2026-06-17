//! 统一可观察事件目录
//!
//! 所有可观察的领域事件在此统一管理。按领域分文件，不膨胀。
//! 一个目录即可看到"项目有哪些可观察事件"——自动满足宪法 13.10.2 事件白名单需求。
//!
//! 🟩 这里只有 struct 定义，零业务逻辑
//! 🟩 所有事件使用 Strong ID 替代 Entity
//! 🟥 禁止在这里定义任何方法、逻辑、System

pub mod battle;
pub mod buff;
pub mod campaign;
pub mod character;
pub mod equipment;
pub mod infra;
pub mod inventory;
pub mod skill;
pub mod turn;
