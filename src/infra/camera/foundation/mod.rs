//! Camera Foundation — 纯类型定义模块（零 ECS 依赖，可独立测试）

pub mod command;
pub mod pose;
pub mod request;
pub mod state;
pub mod target;

pub use command::CameraCommand;
pub use pose::{
    CAMERA_MOVE_SPEED, CameraPose, CurrentPose, FREE_MOVE_TIMEOUT, LERP_SPEED, MAX_ZOOM, MIN_ZOOM,
    SHAKE_FREQUENCY, TargetPose, Z_CAMERA, ZOOM_STEP_FACTOR,
};
pub use request::CameraRequest;
pub use state::CameraState;
pub use target::CameraTarget;
