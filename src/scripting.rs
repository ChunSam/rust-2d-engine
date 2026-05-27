use std::cell::RefCell;
use std::collections::HashMap;
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

// ─── 내부 버퍼 타입 (모듈 레벨) ───────────────────────────────────────────────

/// 스크립트 실행 중 수집된 ECS 명령.
#[derive(Default)]
struct ScriptCommands {
    despawn: Vec<Entity>,
    spawn_count: u32,
    spawned_ids: Vec<i64>,
}

#[derive(Clone)]
enum BbEntry {
    Bool(String, bool),
    Float(String, f64),
    Int(String, i64),
}

#[derive(Clone)]
enum SteeringCmd {
    Seek {
        tx: f32,
        ty: f32,
        speed: f32,
    },
    Flee {
        tx: f32,
        ty: f32,
        speed: f32,
        radius: f32,
    },
    Stop,
}

// ─── Thread-local 실행 컨텍스트 ───────────────────────────────────────────────
//
// 엔티티마다 register_fn을 반복 호출하는 대신, with_limits()에서 1회만 등록하고
// 실행 컨텍스트(버퍼)를 thread_local로 전달한다.
// ECS 시스템은 단일 스레드이므로 RefCell이 안전하다.

struct ScriptCtx {
    cmd_buf: Arc<Mutex<ScriptCommands>>,
    bb_buf: Arc<Mutex<Vec<BbEntry>>>,
    steer_buf: Arc<Mutex<Option<SteeringCmd>>>,
    bb_snap: Arc<Mutex<HashMap<String, BbEntry>>>,
}

thread_local! {
    static SCRIPT_CTX: RefCell<Option<ScriptCtx>> = const { RefCell::new(None) };
}

/// thread_local 컨텍스트를 설정한다. 스크립트 실행 전 호출.
fn set_script_ctx(ctx: ScriptCtx) {
    SCRIPT_CTX.with(|c| *c.borrow_mut() = Some(ctx));
}

/// thread_local 컨텍스트를 제거한다. 스크립트 실행 후 호출.
fn clear_script_ctx() {
    SCRIPT_CTX.with(|c| *c.borrow_mut() = None);
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScriptingLimits {
    pub max_operations: u64,
}

impl Default for ScriptingLimits {
    fn default() -> Self {
        Self {
            max_operations: 1_000_000,
        }
    }
}

impl ScriptingSystem {
    /// Creates a scripting system for trusted local script assets.
    ///
    /// Rhai operation limits reduce accidental runaway scripts, but engine script
    /// assets are still treated as trusted local game code rather than hostile
    /// sandboxed input.
    pub fn new() -> Self {
        Self::with_limits(ScriptingLimits::default())
    }

    pub fn with_limits(limits: ScriptingLimits) -> Self {
        let mut engine = Engine::new();
        engine.set_max_operations(limits.max_operations);

        // ── 모든 함수를 1회만 등록 (thread_local로 실행 컨텍스트 전달) ──────

        engine.register_fn("log", |msg: &str| println!("[Script] {msg}"));

        engine.register_fn("spawn_entity", || -> i64 {
            SCRIPT_CTX.with(|c| {
                let borrow = c.borrow();
                let ctx = borrow
                    .as_ref()
                    .expect("SCRIPT_CTX must be set during script execution");
                let mut cmds = ctx.cmd_buf.lock().unwrap();
                cmds.spawn_count += 1;
                let handle = -(cmds.spawn_count as i64);
                cmds.spawned_ids.push(handle);
                handle
            })
        });

        engine.register_fn("despawn_entity", |id: i64| {
            SCRIPT_CTX.with(|c| {
                let borrow = c.borrow();
                let ctx = borrow
                    .as_ref()
                    .expect("SCRIPT_CTX must be set during script execution");
                if id >= 0 {
                    ctx.cmd_buf.lock().unwrap().despawn.push(Entity(id as u32));
                }
            });
        });

        engine.register_fn("bb_set_bool", |key: &str, val: bool| {
            SCRIPT_CTX.with(|c| {
                let borrow = c.borrow();
                let ctx = borrow
                    .as_ref()
                    .expect("SCRIPT_CTX must be set during script execution");
                ctx.bb_buf
                    .lock()
                    .unwrap()
                    .push(BbEntry::Bool(key.to_string(), val));
            });
        });

        engine.register_fn("bb_set_float", |key: &str, val: f64| {
            SCRIPT_CTX.with(|c| {
                let borrow = c.borrow();
                let ctx = borrow
                    .as_ref()
                    .expect("SCRIPT_CTX must be set during script execution");
                ctx.bb_buf
                    .lock()
                    .unwrap()
                    .push(BbEntry::Float(key.to_string(), val));
            });
        });

        engine.register_fn("bb_set_int", |key: &str, val: i64| {
            SCRIPT_CTX.with(|c| {
                let borrow = c.borrow();
                let ctx = borrow
                    .as_ref()
                    .expect("SCRIPT_CTX must be set during script execution");
                ctx.bb_buf
                    .lock()
                    .unwrap()
                    .push(BbEntry::Int(key.to_string(), val));
            });
        });

        engine.register_fn("bb_get_bool", |key: &str| -> bool {
            SCRIPT_CTX.with(|c| {
                let borrow = c.borrow();
                let ctx = borrow
                    .as_ref()
                    .expect("SCRIPT_CTX must be set during script execution");
                let snap = ctx.bb_snap.lock().unwrap();
                match snap.get(key) {
                    Some(BbEntry::Bool(_, v)) => *v,
                    _ => false,
                }
            })
        });

        engine.register_fn("bb_get_float", |key: &str| -> f64 {
            SCRIPT_CTX.with(|c| {
                let borrow = c.borrow();
                let ctx = borrow
                    .as_ref()
                    .expect("SCRIPT_CTX must be set during script execution");
                let snap = ctx.bb_snap.lock().unwrap();
                match snap.get(key) {
                    Some(BbEntry::Float(_, v)) => *v,
                    _ => 0.0,
                }
            })
        });

        engine.register_fn("bb_get_int", |key: &str| -> i64 {
            SCRIPT_CTX.with(|c| {
                let borrow = c.borrow();
                let ctx = borrow
                    .as_ref()
                    .expect("SCRIPT_CTX must be set during script execution");
                let snap = ctx.bb_snap.lock().unwrap();
                match snap.get(key) {
                    Some(BbEntry::Int(_, v)) => *v,
                    _ => 0,
                }
            })
        });

        engine.register_fn("seek_target", |tx: f64, ty: f64, speed: f64| {
            SCRIPT_CTX.with(|c| {
                let borrow = c.borrow();
                let ctx = borrow
                    .as_ref()
                    .expect("SCRIPT_CTX must be set during script execution");
                *ctx.steer_buf.lock().unwrap() = Some(SteeringCmd::Seek {
                    tx: tx as f32,
                    ty: ty as f32,
                    speed: speed as f32,
                });
            });
        });

        engine.register_fn("flee_from", |tx: f64, ty: f64, speed: f64, radius: f64| {
            SCRIPT_CTX.with(|c| {
                let borrow = c.borrow();
                let ctx = borrow
                    .as_ref()
                    .expect("SCRIPT_CTX must be set during script execution");
                *ctx.steer_buf.lock().unwrap() = Some(SteeringCmd::Flee {
                    tx: tx as f32,
                    ty: ty as f32,
                    speed: speed as f32,
                    radius: radius as f32,
                });
            });
        });

        engine.register_fn("stop_steering", || {
            SCRIPT_CTX.with(|c| {
                let borrow = c.borrow();
                let ctx = borrow
                    .as_ref()
                    .expect("SCRIPT_CTX must be set during script execution");
                *ctx.steer_buf.lock().unwrap() = Some(SteeringCmd::Stop);
            });
        });

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

            // AST 읽기
            let ast = match world
                .resource::<AssetServer>()
                .and_then(|s| s.get_script_by_id(script_id))
                .map(|a| a.ast.clone())
            {
                Some(a) => a,
                None => continue,
            };

            // ── 엔티티별 버퍼 생성 ─────────────────────────────────────────────
            let cmd_buf: Arc<Mutex<ScriptCommands>> =
                Arc::new(Mutex::new(ScriptCommands::default()));
            let bb_buf: Arc<Mutex<Vec<BbEntry>>> = Arc::new(Mutex::new(Vec::new()));
            let steer_buf: Arc<Mutex<Option<SteeringCmd>>> = Arc::new(Mutex::new(None));
            let bb_snap: Arc<Mutex<HashMap<String, BbEntry>>> =
                Arc::new(Mutex::new(HashMap::new()));

            // Blackboard 스냅샷 수집
            {
                use crate::behavior::BlackboardValue;
                if let Some(bb) = world.get::<Blackboard>(entity) {
                    let mut snap = bb_snap.lock().unwrap();
                    for (key, val) in bb.entries() {
                        let entry = match val {
                            BlackboardValue::Bool(v) => BbEntry::Bool(key.to_string(), *v),
                            BlackboardValue::Float(v) => BbEntry::Float(key.to_string(), *v as f64),
                            BlackboardValue::Int(v) => BbEntry::Int(key.to_string(), *v as i64),
                            _ => continue,
                        };
                        snap.insert(key.to_string(), entry);
                    }
                }
            }

            // ── thread_local 컨텍스트 설정 → 스크립트 실행 → 컨텍스트 제거 ───
            set_script_ctx(ScriptCtx {
                cmd_buf: Arc::clone(&cmd_buf),
                bb_buf: Arc::clone(&bb_buf),
                steer_buf: Arc::clone(&steer_buf),
                bb_snap: Arc::clone(&bb_snap),
            });

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

            clear_script_ctx();

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
            for _ in 0..spawn_count {
                world.spawn();
            }
            for e in despawn_list {
                world.despawn(e);
            }

            // ── Blackboard 변경 적용 ──────────────────────────────────────────
            let bb_changes = { bb_buf.lock().unwrap().clone() };
            if !bb_changes.is_empty() {
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

// ─── 테스트 ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine() -> ScriptingSystem {
        ScriptingSystem::new()
    }

    fn eval_with_ctx(sys: &ScriptingSystem, ctx: ScriptCtx, script: &str) {
        let ast = sys.engine.compile(script).unwrap();
        let mut scope = Scope::new();
        set_script_ctx(ctx);
        let _ = sys
            .engine
            .eval_ast_with_scope::<rhai::Dynamic>(&mut scope, &ast);
        clear_script_ctx();
    }

    #[test]
    fn scripting_spawn_entity_works() {
        let sys = make_engine();
        let cmd_buf = Arc::new(Mutex::new(ScriptCommands::default()));
        let ctx = ScriptCtx {
            cmd_buf: Arc::clone(&cmd_buf),
            bb_buf: Arc::new(Mutex::new(Vec::new())),
            steer_buf: Arc::new(Mutex::new(None)),
            bb_snap: Arc::new(Mutex::new(HashMap::new())),
        };
        eval_with_ctx(&sys, ctx, "let id = spawn_entity(); spawn_entity();");
        assert_eq!(cmd_buf.lock().unwrap().spawn_count, 2);
    }

    #[test]
    fn scripting_bb_roundtrip() {
        let sys = make_engine();
        let bb_buf = Arc::new(Mutex::new(Vec::new()));
        let bb_snap = Arc::new(Mutex::new(HashMap::new()));
        // 사전에 스냅샷에 값 넣기 (get 테스트용)
        bb_snap.lock().unwrap().insert(
            "score".to_string(),
            BbEntry::Float("score".to_string(), 42.0),
        );

        let ctx = ScriptCtx {
            cmd_buf: Arc::new(Mutex::new(ScriptCommands::default())),
            bb_buf: Arc::clone(&bb_buf),
            steer_buf: Arc::new(Mutex::new(None)),
            bb_snap: Arc::clone(&bb_snap),
        };
        eval_with_ctx(
            &sys,
            ctx,
            r#"
            bb_set_bool("active", true);
            bb_set_int("hp", 99);
            let s = bb_get_float("score");
        "#,
        );

        let changes = bb_buf.lock().unwrap().clone();
        assert!(changes
            .iter()
            .any(|e| matches!(e, BbEntry::Bool(k, true) if k == "active")));
        assert!(changes
            .iter()
            .any(|e| matches!(e, BbEntry::Int(k, 99) if k == "hp")));
    }

    #[test]
    fn scripting_two_entities_no_buffer_cross_contamination() {
        let sys = make_engine();

        // 엔티티 A 실행
        let bb_buf_a = Arc::new(Mutex::new(Vec::new()));
        let ctx_a = ScriptCtx {
            cmd_buf: Arc::new(Mutex::new(ScriptCommands::default())),
            bb_buf: Arc::clone(&bb_buf_a),
            steer_buf: Arc::new(Mutex::new(None)),
            bb_snap: Arc::new(Mutex::new(HashMap::new())),
        };
        eval_with_ctx(&sys, ctx_a, r#"bb_set_bool("flag_a", true);"#);

        // 엔티티 B 실행
        let bb_buf_b = Arc::new(Mutex::new(Vec::new()));
        let ctx_b = ScriptCtx {
            cmd_buf: Arc::new(Mutex::new(ScriptCommands::default())),
            bb_buf: Arc::clone(&bb_buf_b),
            steer_buf: Arc::new(Mutex::new(None)),
            bb_snap: Arc::new(Mutex::new(HashMap::new())),
        };
        eval_with_ctx(&sys, ctx_b, r#"bb_set_bool("flag_b", true);"#);

        // A 버퍼에는 flag_a만, B 버퍼에는 flag_b만 있어야 한다
        let a = bb_buf_a.lock().unwrap();
        let b = bb_buf_b.lock().unwrap();
        assert!(a
            .iter()
            .any(|e| matches!(e, BbEntry::Bool(k, _) if k == "flag_a")));
        assert!(!a
            .iter()
            .any(|e| matches!(e, BbEntry::Bool(k, _) if k == "flag_b")));
        assert!(b
            .iter()
            .any(|e| matches!(e, BbEntry::Bool(k, _) if k == "flag_b")));
        assert!(!b
            .iter()
            .any(|e| matches!(e, BbEntry::Bool(k, _) if k == "flag_a")));
    }
}
