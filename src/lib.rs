pub mod animation;
pub mod app;
pub mod asset;
pub mod atlas;
#[cfg(not(target_arch = "wasm32"))]
pub mod audio;
pub mod camera;
pub mod collision;
pub mod components;
pub mod debug_ui;
pub mod ecs;
pub mod hierarchy;
pub mod input;
pub mod particle;
#[cfg(not(target_arch = "wasm32"))]
pub mod physics;
pub mod prefab;
pub mod material;
pub mod network;
pub mod reflect;
pub mod renderer;
pub mod resources;
pub mod save;
pub use save::{delete, exists, load, load_or_default, save, save_path, SaveError};
pub mod scene;
pub mod scripting;
pub mod tilemap;
pub mod timer;
pub mod tween;
pub mod ui;

// ── 편의 재수출 ────────────────────────────────────────────────────────────────

pub use animation::{
    AnimParam, AnimState, AnimTransition, AnimationClip, AnimationPlayer, AnimationStateMachine,
    AnimationSystem, BlendEntry, BlendTree1D, BlendTreeSystem, BlendWeight, StateMachineSystem,
    TransitionCond, UvRect,
};
pub use app::App;
pub use asset::{AssetServer, Handle, ImageAsset, ImageEntry, ScriptAsset};
pub use atlas::{AtlasSprite, TextureAtlas};
#[cfg(not(target_arch = "wasm32"))]
pub use audio::AudioManager;
pub use camera::Camera;
pub use collision::{
    Collider, CollisionDebugSystem, CollisionGridSystem, CollisionLayer, DebugConfig, SpatialGrid,
};
pub use components::{Sprite, Transform};
pub use debug_ui::DebugUi;
pub use ecs::{Entity, Events, System, World};
pub use hierarchy::{attach, detach, Children, GlobalTransform, HierarchySystem, Parent};
pub use input::{GamepadAxis, GamepadButton, GamepadState, InputMap, InputState};
pub use particle::{Particle, ParticleEmitter, ParticleSystem};
#[cfg(not(target_arch = "wasm32"))]
pub use physics::{
    CharacterController, CollisionEvent, PhysicsBody, PhysicsSystem, PhysicsWorld, RaycastHit,
};
pub use prefab::{
    spawn_entity_def, spawn_scene_def, topological_sort_entities, EntityDef, Prefab, SceneDef,
    Tag,
};
pub use material::ShaderMaterial;
pub use network::{NetworkClient, NetworkEvent, NetworkSystem};
// par_query_for_each / par_query_map / par_query2_for_each / par_query2_map 은
// World 메서드이므로 World re-export를 통해 자동 접근 가능 (별도 re-export 불필요)
pub use reflect::{Reflect, ReflectValue};
pub use renderer::{DrawRect, DrawText, PostProcessConfig, TextQueue, TextRenderer, UiQueue};
pub use resources::{
    CullConfig, DebugDrawQueue, DebugRect, DisplayScaleFactor, FontData, GameState, PendingResize,
    ProfilerData, RenderStats, SelectedEntity, ShouldQuit, SystemProfile, ViewportSize,
    WindowConfig,
};
pub use scene::{Scene, SceneChange, SceneCmd};
pub use scripting::{ScriptRunner, ScriptingSystem};
pub use tilemap::{Tilemap, TilemapAtlas, TilemapSystem};
pub use timer::Timer;
pub use tween::{Easing, Tween};
pub use ui::{
    Anchor, Button, ButtonState, CheckBox, Label, LayoutDir, LayoutSystem, Panel, ScrollView,
    Slider, TextInput, UiEvent, UiNode, UiSystem,
};

// ── WASM 패닉 훅 ─────────────────────────────────────────────────────────────
#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_init() {
    console_error_panic_hook::set_once();
}

// ── WASM 데모 진입점 ──────────────────────────────────────────────────────────
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn run_demo() {
    use crate::{
        app::App,
        components::{Sprite, Transform},
        ecs::{Entity, System, World},
        renderer::{DrawText, TextQueue},
        resources::{ViewportSize, WindowConfig},
    };
    use glam::Vec2;
    use std::f32::consts::TAU;

    struct BounceVel {
        vx: f32,
        vy: f32,
    }

    struct DemoSystem;
    impl System for DemoSystem {
        fn run(&mut self, world: &mut World, dt: f32) {
            // 좌표계: (0,0) = 좌상단, (width, height) = 우하단
            let (w, h) = world
                .resource::<ViewportSize>()
                .map(|v| (v.width as f32, v.height as f32))
                .unwrap_or((1280.0, 720.0));
            let margin = 32.0;

            let data: Vec<(Entity, f32, f32, f32, f32)> = world
                .query2::<Transform, BounceVel>()
                .map(|(e, t, b)| (e, t.position.x, t.position.y, b.vx, b.vy))
                .collect();

            for (e, x, y, vx, vy) in data {
                let nx = x + vx * dt;
                let ny = y + vy * dt;
                let nvx = if nx < margin || nx > w - margin {
                    -vx
                } else {
                    vx
                };
                let nvy = if ny < margin || ny > h - margin {
                    -vy
                } else {
                    vy
                };
                if let Some(t) = world.get_mut::<Transform>(e) {
                    t.position.x = nx;
                    t.position.y = ny;
                    t.rotation += 1.5 * dt;
                }
                if let Some(b) = world.get_mut::<BounceVel>(e) {
                    b.vx = nvx;
                    b.vy = nvy;
                }
            }

            if let Some(tq) = world.resource_mut::<TextQueue>() {
                tq.push(DrawText {
                    text: "rust-2d-engine  —  WASM demo  (wgpu + WebGL2)".to_string(),
                    position: Vec2::new(20.0, 20.0),
                    size: 22.0,
                    color: [255, 255, 255, 220],
                });
                tq.push(DrawText {
                    text: "ECS  ·  Rendering  ·  Animation  ·  Scripting  ·  Reflect  ·  UI"
                        .to_string(),
                    position: Vec2::new(20.0, 52.0),
                    size: 16.0,
                    color: [160, 210, 255, 200],
                });
            }
        }
    }

    let mut app = App::new();
    app.world.insert_resource(WindowConfig {
        title: "rust-2d-engine WASM demo".to_string(),
        width: 1280,
        height: 720,
        clear_color: [0.05, 0.05, 0.10, 1.0],
    });

    let colors: &[[f32; 3]] = &[
        [1.0, 0.4, 0.4],
        [0.4, 1.0, 0.4],
        [0.4, 0.5, 1.0],
        [1.0, 1.0, 0.3],
        [1.0, 0.4, 1.0],
        [0.3, 1.0, 0.9],
        [1.0, 0.65, 0.3],
        [0.6, 0.3, 1.0],
        [0.3, 0.9, 0.5],
        [0.9, 0.4, 0.2],
    ];
    let velocities: &[(f32, f32)] = &[
        (150.0, 110.0),
        (-130.0, 170.0),
        (190.0, -100.0),
        (-160.0, -145.0),
        (120.0, 185.0),
        (-200.0, 130.0),
        (145.0, -175.0),
        (-115.0, -160.0),
        (175.0, 140.0),
        (-155.0, 125.0),
    ];

    // 화면 중앙(640, 360) 기준으로 원형 배치
    let cx = 640.0_f32;
    let cy = 360.0_f32;
    for (i, (&[r, g, b], &(vx, vy))) in colors.iter().zip(velocities.iter()).enumerate() {
        let angle = i as f32 * TAU / colors.len() as f32;
        let e = app.world.spawn();
        app.world.add_component(
            e,
            Transform {
                position: Vec2::new(cx + angle.cos() * 220.0, cy + angle.sin() * 160.0),
                scale: Vec2::splat(64.0),
                rotation: angle,
                z: 0.0,
            },
        );
        app.world.add_component(e, Sprite::colored(r, g, b));
        app.world.add_component(e, BounceVel { vx, vy });
    }

    app.add_system(DemoSystem);
    app.run();
}
