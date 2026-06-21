//! Camera Systems — ECS System 实现
//!
//! 按调度顺序排列：
//! - PreUpdate:  input_handler::handle_camera_input
//! - Update:     state_machine::process_camera_requests (Observer)
//! - Update:     state_machine::idle_timeout
//! - Update:     state_machine::update_focus
//! - PostUpdate: movement::interpolate_pose
//! - PostUpdate: bounds::clamp_position
//! - PostUpdate: shake::apply_shake
//! - PostUpdate: movement::write_to_transform

pub mod bounds;
pub mod input_handler;
pub mod movement;
pub mod shake;
pub mod state_machine;
