pub mod animation;
pub mod app;
pub mod audio;
pub mod camera;
pub mod collision;
pub mod components;
pub mod ecs;
pub mod input;
pub mod particle;
pub mod physics;
pub mod renderer;
pub mod resources;
pub mod save;
pub mod tilemap;
pub mod ui;

// ── 편의 재수출 ────────────────────────────────────────────────────────────────

pub use animation::{AnimationClip, AnimationPlayer, AnimationSystem, UvRect};
pub use app::App;
pub use audio::AudioManager;
pub use camera::Camera;
pub use collision::{
    Collider, CollisionDebugSystem, CollisionGridSystem, CollisionLayer, DebugConfig, SpatialGrid,
};
pub use components::{Sprite, Transform};
pub use ecs::{Entity, Events, System, World};
pub use input::{InputMap, InputState};
pub use particle::{Particle, ParticleEmitter, ParticleSystem};
pub use physics::{PhysicsBody, PhysicsSystem, PhysicsWorld};
pub use renderer::{DrawRect, DrawText, TextQueue, TextRenderer, UiQueue};
pub use resources::{DebugDrawQueue, DebugRect, FontData, GameState, PendingResize, ShouldQuit, ViewportSize, WindowConfig};
pub use tilemap::{Tilemap, TilemapAtlas, TilemapSystem};
pub use ui::{Anchor, Button, ButtonState, Label, UiEvent, UiNode, UiSystem};
