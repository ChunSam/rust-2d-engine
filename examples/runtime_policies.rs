use engine::{
    App, ScheduleErrorPolicy, System, SystemConfig, SystemPanicPolicy, WindowConfig, World,
};

#[derive(Default)]
struct Counter(u32);

struct HealthySystem;

impl System for HealthySystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        if let Some(counter) = world.resource_mut::<Counter>() {
            counter.0 += 1;
        }
    }

    fn name(&self) -> &'static str {
        "healthy"
    }
}

struct PanicsOnceSystem;

impl System for PanicsOnceSystem {
    fn run(&mut self, _world: &mut World, _dt: f32) {
        panic!("intentional demo panic");
    }

    fn name(&self) -> &'static str {
        "panics_once"
    }
}

fn configure_continue_after_panic() -> App {
    let mut app = App::new();
    app.world.insert_resource(WindowConfig {
        title: "runtime policy demo".to_string(),
        width: 640,
        height: 360,
        ..Default::default()
    });
    app.world.insert_resource(Counter::default());

    // Existing-compatible default, made explicit for discoverability:
    // a panicked system is logged, disabled, and later systems keep running.
    app.set_system_panic_policy(SystemPanicPolicy::DisableSystemAndContinue);
    app.add_system(PanicsOnceSystem);
    app.add_system(HealthySystem);
    app
}

fn configure_strict_schedule_policy() -> App {
    let mut app = App::new();

    // In test/CI-style builds, this makes dependency cycles fail fast instead of
    // falling back to insertion order. The systems below intentionally do not form
    // a cycle; this example shows the policy and labeled registration shape.
    app.set_schedule_error_policy(ScheduleErrorPolicy::PanicOnCycle);
    app.add_system_labeled(
        HealthySystem,
        SystemConfig::new().label("gameplay").after("input"),
    );
    app.add_system_labeled(HealthySystem, SystemConfig::new().label("input"));
    app
}

fn configure_abort_after_log() -> App {
    let mut app = App::new();

    // Use this in development when a system panic should stop immediately after
    // the engine writes its crash log. This demo does not call app.run(), so it
    // remains safe to execute as a short command-line example.
    app.set_system_panic_policy(SystemPanicPolicy::AbortAfterLog);
    app.add_system(PanicsOnceSystem);
    app
}

fn main() {
    let _continue_app = configure_continue_after_panic();
    let _strict_schedule_app = configure_strict_schedule_policy();
    let _abort_app = configure_abort_after_log();

    println!("runtime policy demo configured three App instances:");
    println!("  - DisableSystemAndContinue for recoverable development runs");
    println!("  - PanicOnCycle for strict schedule validation");
    println!("  - AbortAfterLog for fail-fast system panic handling");
    println!("Call app.run() in a real game to apply these policies in the frame loop.");
}
