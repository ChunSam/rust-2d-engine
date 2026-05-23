pub mod animation;
pub mod app;
pub mod audio;
pub mod camera;
pub mod collision;
pub mod components;
pub mod ecs;
pub mod input;
pub mod physics;
pub mod renderer;
pub mod save;

// 편의를 위한 최상위 재수출
pub use animation::AnimationSystem;
pub use app::App;
pub use audio::AudioManager;
pub use camera::Camera;
pub use collision::{Collider, CollisionGridSystem, CollisionLayer, SpatialGrid};
pub use components::{
    AnimationClip, AnimationPlayer, FontData, GameState, PendingResize, ShouldQuit, Sprite,
    Transform, UvRect, ViewportSize, WindowConfig,
};
pub use ecs::{Entity, System, World};
pub use input::InputState;
pub use physics::{PhysicsBody, PhysicsSystem, PhysicsWorld};
pub use renderer::{DrawRect, DrawText, TextQueue, TextRenderer, UiQueue};
