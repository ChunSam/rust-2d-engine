use rhai::{Engine, EvalAltResult, Scope};

use crate::asset::{AssetServer, Handle, ScriptAsset};
use crate::components::Transform;
use crate::ecs::{Entity, System, World};

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

// ─── ScriptingSystem ──────────────────────────────────────────────────────────

/// ScriptRunner를 가진 모든 엔티티에 대해 매 프레임 스크립트를 실행하는 시스템.
///
/// 스코프 변수: `x`, `y`, `rot`, `sx`, `sy`  (Transform 읽기/쓰기)
///
/// 라이프사이클:
/// - `fn on_start()` — 처음 한 번만 호출 (없어도 무방)
/// - `fn on_update(dt)` — 매 프레임 호출
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

            // AssetServer에서 AST 읽기
            let ast = match world
                .resource::<AssetServer>()
                .and_then(|s| s.get_script_by_id(script_id))
                .map(|a| a.ast.clone())
            {
                Some(a) => a,
                None => continue,
            };

            // ScriptRunner scope 업데이트 후 스크립트 실행
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

            // Transform에 결과 적용
            if let Some(t) = world.get_mut::<Transform>(entity) {
                t.position.x = new_tx as f32;
                t.position.y = new_ty as f32;
                t.rotation = new_tr as f32;
                t.scale.x = new_tsx as f32;
                t.scale.y = new_tsy as f32;
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
