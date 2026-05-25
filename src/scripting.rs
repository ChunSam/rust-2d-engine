use std::sync::{Arc, Mutex};

use rhai::{Engine, EvalAltResult, Scope};

use crate::asset::{AssetServer, Handle, ScriptAsset};
use crate::behavior::Blackboard;
use crate::components::Transform;
use crate::ecs::{Entity, System, World};
use crate::steering::{Flee, Seek, SteeringVelocity};

// ─── ScriptRunner ─────────────────────────────────────────────────────────────

/// 엔티티에 붙이는 스크립트 실행기 컴포넌트.
///
/// ```rust,no_run
/// # use engine::{ScriptRunner, ScriptingSystem};
/// # let mut app = engine::App::new();
/// let handle = app.load_script("assets/enemy_ai.rhai");
/// // world.add_component(entity, ScriptRunner::new(handle));
/// // app.add_system(Box::new(ScriptingSystem::new()));
/// ```
pub struct ScriptRunner {
    pub script: Handle<ScriptAsset>,
    pub(crate) scope: Scope<'static>,
    pub(crate) started: bool,
}

impl ScriptRunner {
    pub fn new(script: Handle<ScriptAsset>) -> Self {
        let mut scope = Scope::new();
        scope.push("x", 0.0_f64);
        scope.push("y", 0.0_f64);
        scope.push("rot", 0.0_f64);
        scope.push("sx", 1.0_f64);
        scope.push("sy", 1.0_f64);
        Self {
            script,
            scope,
            started: false,
        }
    }

    /// 다음 프레임에 on_start()가 다시 호출되도록 리셋한다 (핫 리로드 후 유용).
    pub fn reset(&mut self) {
        self.started = false;
    }
}

// ─── ScriptCommand ────────────────────────────────────────────────────────────

/// 스크립트 실행 중 수집된 ECS 명령.
#[derive(Default)]
struct ScriptCommands {
    despawn: Vec<Entity>,
    spawn_count: u32,
    spawned_ids: Vec<i64>,
}

// ─── ScriptingSystem ──────────────────────────────────────────────────────────

/// ScriptRunner를 가진 모든 엔티티에 대해 매 프레임 스크립트를 실행하는 시스템.
///
/// 스코프 변수: `x`, `y`, `rot`, `sx`, `sy`  (Transform 읽기/쓰기)
///
/// 라이프사이클:
/// - `fn on_start()` — 처음 한 번만 호출 (없어도 무방)
/// - `fn on_update(dt)` — 매 프레임 호출
///
/// ## 추가 스크립트 API (Phase 38d)
///
/// ### Commands
/// ```rhai
/// let id = spawn_entity();   // 새 엔티티 생성 → ID(i64) 반환
/// despawn_entity(id);        // 엔티티 삭제 예약
/// ```
///
/// ### Blackboard
/// ```rhai
/// bb_set_bool("is_chasing", true);
/// bb_set_float("speed", 150.0);
/// bb_set_int("hp", 100);
/// let chasing = bb_get_bool("is_chasing");  // 없으면 false
/// let speed   = bb_get_float("speed");       // 없으면 0.0
/// let hp      = bb_get_int("hp");            // 없으면 0
/// ```
///
/// ### Steering
/// ```rhai
/// seek_target(player_x, player_y, 120.0);        // Seek 컴포넌트 설정
/// flee_from(enemy_x, enemy_y, 200.0, 80.0);      // Flee 컴포넌트 설정
/// stop_steering();                                // SteeringVelocity 속도 리셋
/// ```
pub struct ScriptingSystem {
    engine: Engine,
}

impl ScriptingSystem {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        engine.register_fn("log", |msg: &str| println!("[Script] {msg}"));
        engine.set_max_operations(1_000_000);
        Self { engine }
    }
}

impl Default for ScriptingSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for ScriptingSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        let entities: Vec<Entity> = world.query::<ScriptRunner>().map(|(e, _)| e).collect();

        for entity in entities {
            // 스크립트 핸들 id + started 플래그 읽기
            let (script_id, is_started) = match world.get::<ScriptRunner>(entity) {
                Some(r) => (r.script.id(), r.started),
                None => continue,
            };

            // Transform 스냅샷 읽기
            let (tx, ty, tr, tsx, tsy) = world
                .get::<Transform>(entity)
                .map(|t| {
                    (
                        t.position.x as f64,
                        t.position.y as f64,
                        t.rotation as f64,
                        t.scale.x as f64,
                        t.scale.y as f64,
                    )
                })
                .unwrap_or((0.0, 0.0, 0.0, 1.0, 1.0));

            // Blackboard 스냅샷 읽기 (bool/float/int)
            // — 스크립트 실행 전에 현재 엔티티의 Blackboard 값을 꺼낸다.
            // — 스크립트가 bb_set_* 를 호출하면 Arc<Mutex<...>> 버퍼에 변경을 기록하고,
            //   실행 후 World에 반영한다.

            // AssetServer에서 AST 읽기
            let ast = match world
                .resource::<AssetServer>()
                .and_then(|s| s.get_script_by_id(script_id))
                .map(|a| a.ast.clone())
            {
                Some(a) => a,
                None => continue,
            };

            // ── Commands 버퍼 ──────────────────────────────────────────────────
            // 스크립트가 spawn_entity / despawn_entity 를 호출할 때 사용.
            // World를 클로저로 직접 캡처할 수 없으므로 Arc<Mutex<...>>로 수집 후 후처리.
            let cmd_buf: Arc<Mutex<ScriptCommands>> =
                Arc::new(Mutex::new(ScriptCommands::default()));

            // ── Blackboard 변경 버퍼 ──────────────────────────────────────────
            // (key, value) 쌍 목록. 스크립트 실행 후 Blackboard 컴포넌트에 반영.
            #[derive(Clone)]
            enum BbEntry {
                Bool(String, bool),
                Float(String, f64),
                Int(String, i64),
            }
            let bb_buf: Arc<Mutex<Vec<BbEntry>>> = Arc::new(Mutex::new(Vec::new()));

            // ── Steering 변경 버퍼 ────────────────────────────────────────────
            #[derive(Clone)]
            enum SteeringCmd {
                Seek { tx: f32, ty: f32, speed: f32 },
                Flee { tx: f32, ty: f32, speed: f32, radius: f32 },
                Stop,
            }
            let steer_buf: Arc<Mutex<Option<SteeringCmd>>> = Arc::new(Mutex::new(None));

            // ── Rhai 함수 등록 ────────────────────────────────────────────────

            // --- spawn_entity ---
            {
                let cb = Arc::clone(&cmd_buf);
                self.engine.register_fn("spawn_entity", move || -> i64 {
                    let mut cmds = cb.lock().unwrap();
                    cmds.spawn_count += 1;
                    // 실제 Entity ID는 스크립트 실행 후 World에서 할당된다.
                    // 여기서는 임시 음수 핸들(-1, -2, …)을 반환한다.
                    let handle = -(cmds.spawn_count as i64);
                    cmds.spawned_ids.push(handle);
                    handle
                });
            }

            // --- despawn_entity ---
            {
                let cb = Arc::clone(&cmd_buf);
                self.engine.register_fn("despawn_entity", move |id: i64| {
                    if id >= 0 {
                        cb.lock().unwrap().despawn.push(Entity(id as u32));
                    }
                });
            }

            // --- bb_set_bool ---
            {
                let buf = Arc::clone(&bb_buf);
                self.engine
                    .register_fn("bb_set_bool", move |key: &str, val: bool| {
                        buf.lock()
                            .unwrap()
                            .push(BbEntry::Bool(key.to_string(), val));
                    });
            }

            // --- bb_set_float ---
            {
                let buf = Arc::clone(&bb_buf);
                self.engine
                    .register_fn("bb_set_float", move |key: &str, val: f64| {
                        buf.lock()
                            .unwrap()
                            .push(BbEntry::Float(key.to_string(), val));
                    });
            }

            // --- bb_set_int ---
            {
                let buf = Arc::clone(&bb_buf);
                self.engine
                    .register_fn("bb_set_int", move |key: &str, val: i64| {
                        buf.lock()
                            .unwrap()
                            .push(BbEntry::Int(key.to_string(), val));
                    });
            }

            // --- bb_get_bool (현재 엔티티의 Blackboard에서 읽기) ---
            // World를 클로저로 캡처할 수 없으므로, 실행 전 스냅샷을 찍어 scope에 전달하는
            // 방식 대신, Rhai `Dynamic` 맵을 scope에 주입하는 단순 패턴을 사용한다:
            // 스크립트 실행 전에 값을 추출해 scope 변수로 제공하고,
            // bb_get_* 함수는 그 scope 변수를 읽는다.
            //
            // 단, 함수로 제공하려면 스냅샷 Arc를 공유해야 한다.
            // 여기서는 스크립트 실행 전에 현재 값을 Arc<Mutex<HashMap>> 에 복사한다.
            let bb_snap: Arc<Mutex<std::collections::HashMap<String, BbEntry>>> =
                Arc::new(Mutex::new(std::collections::HashMap::new()));
            {
                use crate::behavior::BlackboardValue;
                if let Some(bb) = world.get::<Blackboard>(entity) {
                    let mut snap = bb_snap.lock().unwrap();
                    for (key, val) in bb.entries() {
                        let entry = match val {
                            BlackboardValue::Bool(v) => BbEntry::Bool(key.to_string(), *v),
                            BlackboardValue::Float(v) => {
                                BbEntry::Float(key.to_string(), *v as f64)
                            }
                            BlackboardValue::Int(v) => BbEntry::Int(key.to_string(), *v as i64),
                            // Vec2 / String은 bb_get_* API에서 지원하지 않으므로 건너뜀
                            _ => continue,
                        };
                        snap.insert(key.to_string(), entry);
                    }
                }
            }

            // bb_get_bool
            {
                let snap = Arc::clone(&bb_snap);
                self.engine
                    .register_fn("bb_get_bool", move |key: &str| -> bool {
                        let s = snap.lock().unwrap();
                        match s.get(key) {
                            Some(BbEntry::Bool(_, v)) => *v,
                            _ => false,
                        }
                    });
            }

            // bb_get_float
            {
                let snap = Arc::clone(&bb_snap);
                self.engine
                    .register_fn("bb_get_float", move |key: &str| -> f64 {
                        let s = snap.lock().unwrap();
                        match s.get(key) {
                            Some(BbEntry::Float(_, v)) => *v,
                            _ => 0.0,
                        }
                    });
            }

            // bb_get_int
            {
                let snap = Arc::clone(&bb_snap);
                self.engine
                    .register_fn("bb_get_int", move |key: &str| -> i64 {
                        let s = snap.lock().unwrap();
                        match s.get(key) {
                            Some(BbEntry::Int(_, v)) => *v,
                            _ => 0,
                        }
                    });
            }

            // --- seek_target ---
            {
                let buf = Arc::clone(&steer_buf);
                self.engine.register_fn(
                    "seek_target",
                    move |tx: f64, ty: f64, speed: f64| {
                        *buf.lock().unwrap() = Some(SteeringCmd::Seek {
                            tx: tx as f32,
                            ty: ty as f32,
                            speed: speed as f32,
                        });
                    },
                );
            }

            // --- flee_from ---
            {
                let buf = Arc::clone(&steer_buf);
                self.engine.register_fn(
                    "flee_from",
                    move |tx: f64, ty: f64, speed: f64, radius: f64| {
                        *buf.lock().unwrap() = Some(SteeringCmd::Flee {
                            tx: tx as f32,
                            ty: ty as f32,
                            speed: speed as f32,
                            radius: radius as f32,
                        });
                    },
                );
            }

            // --- stop_steering ---
            {
                let buf = Arc::clone(&steer_buf);
                self.engine.register_fn("stop_steering", move || {
                    *buf.lock().unwrap() = Some(SteeringCmd::Stop);
                });
            }

            // ── 스크립트 실행 ─────────────────────────────────────────────────
            let (new_tx, new_ty, new_tr, new_tsx, new_tsy) = {
                let runner = world.get_mut::<ScriptRunner>(entity).unwrap();
                runner.scope.set_value("x", tx);
                runner.scope.set_value("y", ty);
                runner.scope.set_value("rot", tr);
                runner.scope.set_value("sx", tsx);
                runner.scope.set_value("sy", tsy);

                if !is_started {
                    call_fn_optional(&self.engine, &mut runner.scope, &ast, "on_start", ());
                    runner.started = true;
                }
                call_fn_optional(
                    &self.engine,
                    &mut runner.scope,
                    &ast,
                    "on_update",
                    (dt as f64,),
                );

                let nx = runner.scope.get_value::<f64>("x").unwrap_or(tx);
                let ny = runner.scope.get_value::<f64>("y").unwrap_or(ty);
                let nr = runner.scope.get_value::<f64>("rot").unwrap_or(tr);
                let nsx = runner.scope.get_value::<f64>("sx").unwrap_or(tsx);
                let nsy = runner.scope.get_value::<f64>("sy").unwrap_or(tsy);
                (nx, ny, nr, nsx, nsy)
            };

            // ── Transform 결과 적용 ──────────────────────────────────────────
            if let Some(t) = world.get_mut::<Transform>(entity) {
                t.position.x = new_tx as f32;
                t.position.y = new_ty as f32;
                t.rotation = new_tr as f32;
                t.scale.x = new_tsx as f32;
                t.scale.y = new_tsy as f32;
            }

            // ── Commands 적용 ────────────────────────────────────────────────
            let (spawn_count, despawn_list) = {
                let guard = cmd_buf.lock().unwrap();
                (guard.spawn_count, guard.despawn.clone())
            };
            // spawn (임시 핸들이므로 Entity ID는 순서대로 할당된다)
            for _ in 0..spawn_count {
                world.spawn();
            }
            // despawn
            for e in despawn_list {
                world.despawn(e);
            }

            // ── Blackboard 변경 적용 ──────────────────────────────────────────
            let bb_changes = {
                let guard = bb_buf.lock().unwrap();
                guard.clone()
            };
            if !bb_changes.is_empty() {
                // Blackboard 컴포넌트가 없으면 새로 추가
                if world.get::<Blackboard>(entity).is_none() {
                    world.add_component(entity, Blackboard::new());
                }
                if let Some(bb) = world.get_mut::<Blackboard>(entity) {
                    for entry in bb_changes {
                        match entry {
                            BbEntry::Bool(k, v) => bb.set_bool(&k, v),
                            BbEntry::Float(k, v) => bb.set_float(&k, v as f32),
                            BbEntry::Int(k, v) => bb.set_int(&k, v as i32),
                        }
                    }
                }
            }

            // ── Steering 변경 적용 ────────────────────────────────────────────
            let steer_cmd = steer_buf.lock().unwrap().take();
            if let Some(cmd) = steer_cmd {
                match cmd {
                    SteeringCmd::Seek { tx, ty, speed } => {
                        use glam::Vec2;
                        if world.get::<SteeringVelocity>(entity).is_none() {
                            world.add_component(entity, SteeringVelocity::default());
                        }
                        world.add_component(
                            entity,
                            Seek {
                                target: Vec2::new(tx, ty),
                                max_speed: speed,
                            },
                        );
                    }
                    SteeringCmd::Flee {
                        tx,
                        ty,
                        speed,
                        radius,
                    } => {
                        use glam::Vec2;
                        if world.get::<SteeringVelocity>(entity).is_none() {
                            world.add_component(entity, SteeringVelocity::default());
                        }
                        world.add_component(
                            entity,
                            Flee {
                                target: Vec2::new(tx, ty),
                                max_speed: speed,
                                flee_radius: radius,
                            },
                        );
                    }
                    SteeringCmd::Stop => {
                        if let Some(sv) = world.get_mut::<SteeringVelocity>(entity) {
                            sv.velocity = glam::Vec2::ZERO;
                        }
                    }
                }
            }
        }
    }
}

// ─── 내부 헬퍼 ────────────────────────────────────────────────────────────────

fn call_fn_optional<A: rhai::FuncArgs>(
    engine: &Engine,
    scope: &mut Scope,
    ast: &rhai::AST,
    fn_name: &str,
    args: A,
) {
    if let Err(e) = engine.call_fn::<()>(scope, ast, fn_name, args) {
        match *e {
            EvalAltResult::ErrorFunctionNotFound(_, _) => {}
            ref other => log::warn!("Script '{fn_name}' 오류: {other}"),
        }
    }
}
