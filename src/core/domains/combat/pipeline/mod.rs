//! Combat Pipeline — 回合流程管线
//!
//! 使用 `infra::pipeline` 驱动战斗回合循环，替代原有的 TurnSubState 状态机。
//!
//! 驾驶员模式 (Pipeline Driver)：由于 UnitAction 阶段需要异步等待玩家/AI 输入，
//! 不能直接使用同步的 `execute_pipeline`。改为 CombatPipelineDriver Resource
//! 在 Update 中逐步骤推进，在 UnitAction 处暂停，等待 UnitActionComplete 事件后恢复。
//!
//! # 管线流程
//!
//! ```text
//! TurnStart → PhaseCheck → UnitAction(暂停) → TurnSettlement → TurnEnd
//!                                ↑                    │
//!                                └── UnitActionComplete┘
//! ```
//!
//! 详见 docs/01-architecture/20-tactical-combat/ADR-021-turn-state-machine.md

pub(crate) mod definition;
pub(crate) mod driver;
pub(crate) mod steps;
