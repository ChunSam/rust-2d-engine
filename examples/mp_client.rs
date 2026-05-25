//! Phase 27 — 멀티플레이어 클라이언트 데모
//!
//! 먼저 서버를 실행한 뒤 이 클라이언트를 여러 창으로 실행한다.
//!
//! ```
//! # 터미널 1
//! cargo run --example mp_server
//!
//! # 터미널 2, 3, ...
//! cargo run --example mp_client
//! ```
//!
//! # 조작
//! - WASD / 방향키: 플레이어 이동
//! - 흰색 사각형: 자신
//! - 색상 사각형: 다른 접속자 (ID별 고유 색상)

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use engine::ecs::{System, World};
    use engine::scene::Scene;
    use engine::{
        App, DrawText, Events, NetworkClient, NetworkEvent, NetworkSystem, Sprite, TextQueue,
        Transform, WindowConfig,
    };
    use glam::Vec2;
    use serde::{Deserialize, Serialize};

    const MAX_JSON_MESSAGE_BYTES: usize = 4096;

    #[derive(Debug, Deserialize)]
    #[serde(tag = "type")]
    enum ServerMessage {
        #[serde(rename = "hello")]
        Hello { id: usize },
        #[serde(rename = "pos")]
        Position { id: usize, x: f32, y: f32 },
        #[serde(rename = "bye")]
        Bye { id: usize },
    }

    #[derive(Debug, Serialize)]
    struct ClientPosition {
        x: f32,
        y: f32,
    }

    // ── 씬 ──────────────────────────────────────────────────────────────────────

    struct MultiScene;

    impl Scene for MultiScene {
        fn on_enter(&mut self, world: &mut World, systems: &mut Vec<Box<dyn System>>) {
            let client = NetworkClient::connect("ws://127.0.0.1:9001");
            world.insert_resource(client);
            systems.push(Box::new(NetworkSystem));
            systems.push(Box::new(MultiplayerSystem::new()));
        }
    }

    // ── 게임 시스템 ────────────────────────────────────────────────────────────

    struct MultiplayerSystem {
        local_entity: Option<engine::Entity>,
        local_id: Option<usize>,
        remote_players: std::collections::HashMap<usize, engine::Entity>,
        send_timer: f32,
        status: String,
    }

    impl MultiplayerSystem {
        fn new() -> Self {
            Self {
                local_entity: None,
                local_id: None,
                remote_players: std::collections::HashMap::new(),
                send_timer: 0.0,
                status: "Connecting to ws://127.0.0.1:9001 ...".into(),
            }
        }
    }

    impl System for MultiplayerSystem {
        fn run(&mut self, world: &mut World, dt: f32) {
            // 1. 네트워크 이벤트 처리
            let events: Vec<NetworkEvent> = world
                .resource::<Events<NetworkEvent>>()
                .map(|bus| bus.read().to_vec())
                .unwrap_or_default();

            for ev in events {
                match ev {
                    NetworkEvent::Connected => {
                        self.status = "Connected — waiting for player ID...".into();
                    }
                    NetworkEvent::TextMessage(ref text) => {
                        self.handle_message(world, text);
                    }
                    NetworkEvent::MessageTooLarge { len, limit } => {
                        self.status = format!("Dropped oversized message: {len} > {limit} bytes");
                    }
                    NetworkEvent::Disconnected { reason } => {
                        self.status = format!("Disconnected: {reason}");
                    }
                    NetworkEvent::JsonParseError { message } => {
                        self.status = format!("Protocol error: {message}");
                    }
                    NetworkEvent::Error(e) => {
                        self.status = format!("Error: {e}");
                    }
                    _ => {}
                }
            }

            // 2. 로컬 플레이어 엔티티 보장
            if self.local_entity.is_none() {
                let e = world.spawn();
                world.add_component(
                    e,
                    Transform {
                        position: Vec2::new(400.0, 300.0),
                        scale: Vec2::splat(32.0),
                        rotation: 0.0,
                        z: 0.0,
                    },
                );
                world.add_component(e, Sprite::colored(1.0, 1.0, 1.0));
                self.local_entity = Some(e);
            }

            // 3. 입력 → 이동
            let (dx, dy) = {
                use winit::keyboard::KeyCode;
                if let Some(input) = world.resource::<engine::InputState>() {
                    let right = (input.is_pressed(KeyCode::KeyD)
                        || input.is_pressed(KeyCode::ArrowRight))
                        as i32;
                    let left = (input.is_pressed(KeyCode::KeyA)
                        || input.is_pressed(KeyCode::ArrowLeft))
                        as i32;
                    let down = (input.is_pressed(KeyCode::KeyS)
                        || input.is_pressed(KeyCode::ArrowDown))
                        as i32;
                    let up = (input.is_pressed(KeyCode::KeyW) || input.is_pressed(KeyCode::ArrowUp))
                        as i32;
                    ((right - left) as f32, (down - up) as f32)
                } else {
                    (0.0, 0.0)
                }
            };

            let speed = 200.0_f32;
            if let Some(e) = self.local_entity {
                if let Some(tr) = world.get_mut::<Transform>(e) {
                    tr.position.x += dx * speed * dt;
                    tr.position.y += dy * speed * dt;
                }
            }

            // 4. 위치 송신 (20 Hz, ID 할당 이후)
            self.send_timer -= dt;
            if self.send_timer <= 0.0 && self.local_id.is_some() {
                self.send_timer = 0.05;
                let pos = self
                    .local_entity
                    .and_then(|e| world.get::<Transform>(e).map(|t| t.position));
                if let Some(pos) = pos {
                    if let Some(client) = world.resource::<NetworkClient>() {
                        if let Ok(text) = serde_json::to_string(&ClientPosition {
                            x: (pos.x * 100.0).round() / 100.0,
                            y: (pos.y * 100.0).round() / 100.0,
                        }) {
                            client.send_text(text);
                        }
                    }
                }
            }

            // 5. 상태 HUD
            let id_label = self
                .local_id
                .map(|id| format!("Player #{id}"))
                .unwrap_or_else(|| "...".into());
            let peers = self.remote_players.len();
            let hud = format!("{id_label}  |  peers: {peers}  |  {}", self.status);

            if let Some(tq) = world.resource_mut::<TextQueue>() {
                tq.push(DrawText::new(
                    hud,
                    Vec2::new(10.0, 10.0),
                    18.0,
                    [255, 255, 255, 210],
                ));
                tq.push(DrawText::new(
                    "WASD / Arrow keys to move",
                    Vec2::new(10.0, 36.0),
                    14.0,
                    [160, 160, 160, 180],
                ));
            }
        }
    }

    impl MultiplayerSystem {
        fn handle_message(&mut self, world: &mut World, text: &str) {
            if text.len() > MAX_JSON_MESSAGE_BYTES {
                self.status = format!(
                    "Dropped oversized protocol message: {} > {} bytes",
                    text.len(),
                    MAX_JSON_MESSAGE_BYTES
                );
                return;
            }

            let message = match serde_json::from_str::<ServerMessage>(text) {
                Ok(message) => message,
                Err(err) => {
                    self.status = format!("Protocol error: {err}");
                    return;
                }
            };

            match message {
                ServerMessage::Hello { id } => {
                    self.local_id = Some(id);
                    self.status = format!("Connected as Player #{id}");
                }
                ServerMessage::Position { id, x, y } => {
                    if let Some(&entity) = self.remote_players.get(&id) {
                        if let Some(tr) = world.get_mut::<Transform>(entity) {
                            tr.position = Vec2::new(x, y);
                        }
                    } else {
                        // 새 원격 플레이어 스폰
                        let e = world.spawn();
                        let [r, g, b] = remote_color(id);
                        world.add_component(
                            e,
                            Transform {
                                position: Vec2::new(x, y),
                                scale: Vec2::splat(32.0),
                                rotation: 0.0,
                                z: 0.0,
                            },
                        );
                        world.add_component(e, Sprite::colored(r, g, b));
                        self.remote_players.insert(id, e);
                    }
                }
                ServerMessage::Bye { id } => {
                    if let Some(entity) = self.remote_players.remove(&id) {
                        world.despawn(entity);
                    }
                }
            }
        }
    }

    // ── 진입점 ──────────────────────────────────────────────────────────────────

    let mut app = App::new();
    app.world.insert_resource(WindowConfig {
        title: "Multiplayer Demo — Phase 27".to_string(),
        width: 800,
        height: 600,
        clear_color: [0.05, 0.07, 0.12, 1.0],
    });
    app.register_event::<NetworkEvent>();
    app.set_scene(Box::new(MultiScene));
    app.run();
}

#[cfg(target_arch = "wasm32")]
fn main() {}

/// ID를 6색 팔레트로 매핑한다.
fn remote_color(id: usize) -> [f32; 3] {
    const PALETTE: &[[f32; 3]] = &[
        [1.0, 0.35, 0.35], // red
        [0.35, 1.0, 0.45], // green
        [0.35, 0.55, 1.0], // blue
        [1.0, 0.95, 0.3],  // yellow
        [1.0, 0.4, 1.0],   // magenta
        [0.3, 1.0, 0.95],  // cyan
    ];
    PALETTE[id % PALETTE.len()]
}
