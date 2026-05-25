//! 비헤이비어 트리 (Behavior Tree) 시스템 (Phase 36)
//!
//! # 핵심 타입
//! - [`BehaviorStatus`] — 노드 실행 결과 (`Running` / `Success` / `Failure`)
//! - [`BehaviorNode`] — 모든 노드가 구현해야 하는 트레잇
//! - [`Sequence`] — 자식 순서대로 실행, 첫 Failure에 즉시 중단
//! - [`Selector`] — 자식 순서대로 실행, 첫 Success에 즉시 중단
//! - [`Inverter`] — 자식 결과를 반전 (Success↔Failure)
//! - [`BehaviorTree`] — ECS 컴포넌트. 루트 노드를 감싼다.
//! - [`BehaviorSystem`] — 매 프레임 `BehaviorTree`를 가진 엔티티를 tick.
//!
//! # 사용 예
//! ```rust,no_run
//! use engine::behavior::{BehaviorNode, BehaviorStatus, BehaviorTree, Sequence, Selector};
//! use engine::ecs::World;
//! use engine::System;
//!
//! struct ChasePlayer;
//! impl BehaviorNode for ChasePlayer {
//!     fn tick(&mut self, _world: &mut World, _entity: engine::ecs::Entity, _dt: f32) -> BehaviorStatus {
//!         BehaviorStatus::Success
//!     }
//! }
//!
//! let mut world = engine::ecs::World::new();
//! let e = world.spawn();
//! world.add_component(e, BehaviorTree::new(Box::new(Sequence::new(vec![
//!     Box::new(ChasePlayer),
//! ]))));
//! ```

use crate::ecs::{Entity, World};
use crate::System;

// ─── BehaviorStatus ───────────────────────────────────────────────────────────

/// 비헤이비어 노드 실행 결과.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorStatus {
    /// 아직 실행 중 (다음 프레임에도 계속 tick 됨).
    Running,
    /// 성공적으로 완료.
    Success,
    /// 실패.
    Failure,
}

// ─── BehaviorNode 트레잇 ──────────────────────────────────────────────────────

/// 비헤이비어 트리의 단일 노드.
///
/// 커스텀 행동을 구현할 때 이 트레잇을 구현한다.
pub trait BehaviorNode: Send + Sync {
    /// 한 프레임 실행하고 상태를 반환한다.
    fn tick(&mut self, world: &mut World, entity: Entity, dt: f32) -> BehaviorStatus;

    /// 노드가 다시 시작될 때 내부 상태를 초기화한다 (선택 구현).
    fn reset(&mut self) {}
}

// ─── 내장 복합 노드 ───────────────────────────────────────────────────────────

/// 자식 노드를 순서대로 실행한다.
/// - 자식이 `Success` → 다음 자식으로 진행
/// - 자식이 `Running`  → 자신도 `Running` 반환 (다음 프레임에 같은 자식 재시작)
/// - 자식이 `Failure`  → 즉시 중단하고 `Failure` 반환
/// - 모든 자식 `Success` → `Success` 반환
pub struct Sequence {
    children: Vec<Box<dyn BehaviorNode>>,
    current: usize,
}

impl Sequence {
    pub fn new(children: Vec<Box<dyn BehaviorNode>>) -> Self {
        Self { children, current: 0 }
    }
}

impl BehaviorNode for Sequence {
    fn tick(&mut self, world: &mut World, entity: Entity, dt: f32) -> BehaviorStatus {
        while self.current < self.children.len() {
            match self.children[self.current].tick(world, entity, dt) {
                BehaviorStatus::Success => self.current += 1,
                BehaviorStatus::Running => return BehaviorStatus::Running,
                BehaviorStatus::Failure => {
                    self.current = 0;
                    return BehaviorStatus::Failure;
                }
            }
        }
        self.current = 0;
        BehaviorStatus::Success
    }

    fn reset(&mut self) {
        self.current = 0;
        for child in &mut self.children {
            child.reset();
        }
    }
}

/// 자식 노드를 순서대로 실행한다.
/// - 자식이 `Failure`  → 다음 자식으로 진행
/// - 자식이 `Running`  → 자신도 `Running` 반환
/// - 자식이 `Success`  → 즉시 중단하고 `Success` 반환
/// - 모든 자식 `Failure` → `Failure` 반환
pub struct Selector {
    children: Vec<Box<dyn BehaviorNode>>,
    current: usize,
}

impl Selector {
    pub fn new(children: Vec<Box<dyn BehaviorNode>>) -> Self {
        Self { children, current: 0 }
    }
}

impl BehaviorNode for Selector {
    fn tick(&mut self, world: &mut World, entity: Entity, dt: f32) -> BehaviorStatus {
        while self.current < self.children.len() {
            match self.children[self.current].tick(world, entity, dt) {
                BehaviorStatus::Failure => self.current += 1,
                BehaviorStatus::Running => return BehaviorStatus::Running,
                BehaviorStatus::Success => {
                    self.current = 0;
                    return BehaviorStatus::Success;
                }
            }
        }
        self.current = 0;
        BehaviorStatus::Failure
    }

    fn reset(&mut self) {
        self.current = 0;
        for child in &mut self.children {
            child.reset();
        }
    }
}

/// 자식 노드 결과를 반전한다 (Success → Failure, Failure → Success, Running 유지).
pub struct Inverter {
    child: Box<dyn BehaviorNode>,
}

impl Inverter {
    pub fn new(child: Box<dyn BehaviorNode>) -> Self {
        Self { child }
    }
}

impl BehaviorNode for Inverter {
    fn tick(&mut self, world: &mut World, entity: Entity, dt: f32) -> BehaviorStatus {
        match self.child.tick(world, entity, dt) {
            BehaviorStatus::Success => BehaviorStatus::Failure,
            BehaviorStatus::Failure => BehaviorStatus::Success,
            BehaviorStatus::Running => BehaviorStatus::Running,
        }
    }

    fn reset(&mut self) {
        self.child.reset();
    }
}

/// 항상 `Success`를 반환하는 데코레이터 노드.
pub struct AlwaysSucceed {
    child: Box<dyn BehaviorNode>,
}

impl AlwaysSucceed {
    pub fn new(child: Box<dyn BehaviorNode>) -> Self {
        Self { child }
    }
}

impl BehaviorNode for AlwaysSucceed {
    fn tick(&mut self, world: &mut World, entity: Entity, dt: f32) -> BehaviorStatus {
        self.child.tick(world, entity, dt);
        BehaviorStatus::Success
    }

    fn reset(&mut self) {
        self.child.reset();
    }
}

// ─── BehaviorTree 컴포넌트 ────────────────────────────────────────────────────

/// ECS 컴포넌트. 루트 `BehaviorNode`를 감싼다.
///
/// `BehaviorSystem`이 매 프레임 `tick()`을 호출한다.
/// `BehaviorNode: Send + Sync` 이므로 `BehaviorTree`도 스레드 안전하다.
pub struct BehaviorTree {
    root: Box<dyn BehaviorNode>,
}

// BehaviorNode: Send + Sync 이므로 BehaviorTree도 동일 보장
unsafe impl Send for BehaviorTree {}
unsafe impl Sync for BehaviorTree {}

impl BehaviorTree {
    pub fn new(root: Box<dyn BehaviorNode>) -> Self {
        Self { root }
    }

    pub fn tick(&mut self, world: &mut World, entity: Entity, dt: f32) -> BehaviorStatus {
        self.root.tick(world, entity, dt)
    }

    /// 루트부터 전체 트리 상태를 초기화한다.
    pub fn reset(&mut self) {
        self.root.reset();
    }
}

// ─── BehaviorSystem ───────────────────────────────────────────────────────────

/// `BehaviorTree` 컴포넌트를 가진 모든 엔티티를 매 프레임 tick하는 시스템.
///
/// # 등록
/// ```rust,no_run
/// # use engine::App;
/// # use engine::behavior::BehaviorSystem;
/// let mut app = App::new();
/// app.add_system(BehaviorSystem);
/// ```
pub struct BehaviorSystem;

impl System for BehaviorSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        // borrow checker 우회: 먼저 엔티티 목록을 수집
        let entities: Vec<Entity> = world
            .query::<BehaviorTree>()
            .map(|(e, _)| e)
            .collect();

        for entity in entities {
            // BehaviorTree를 임시로 꺼내 tick한 뒤 다시 넣는다.
            // take_component → tick(world) → add_component 로 이중 borrow 없이 처리.
            if let Some(mut tree) = world.take_component::<BehaviorTree>(entity) {
                tree.tick(world, entity, dt);
                world.add_component(entity, tree);
            }
        }
    }

    fn name(&self) -> &'static str {
        "BehaviorSystem"
    }
}

// ─── 테스트 ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysOk;
    impl BehaviorNode for AlwaysOk {
        fn tick(&mut self, _: &mut World, _: Entity, _: f32) -> BehaviorStatus {
            BehaviorStatus::Success
        }
    }

    struct AlwaysFail;
    impl BehaviorNode for AlwaysFail {
        fn tick(&mut self, _: &mut World, _: Entity, _: f32) -> BehaviorStatus {
            BehaviorStatus::Failure
        }
    }

    struct AlwaysRun;
    impl BehaviorNode for AlwaysRun {
        fn tick(&mut self, _: &mut World, _: Entity, _: f32) -> BehaviorStatus {
            BehaviorStatus::Running
        }
    }

    fn dummy() -> (World, Entity) {
        let mut w = World::new();
        let e = w.spawn();
        (w, e)
    }

    #[test]
    fn sequence_all_success() {
        let (mut w, e) = dummy();
        let mut seq = Sequence::new(vec![Box::new(AlwaysOk), Box::new(AlwaysOk)]);
        assert_eq!(seq.tick(&mut w, e, 0.016), BehaviorStatus::Success);
    }

    #[test]
    fn sequence_fails_on_failure() {
        let (mut w, e) = dummy();
        let mut seq = Sequence::new(vec![Box::new(AlwaysOk), Box::new(AlwaysFail), Box::new(AlwaysOk)]);
        assert_eq!(seq.tick(&mut w, e, 0.016), BehaviorStatus::Failure);
    }

    #[test]
    fn sequence_running_pauses() {
        let (mut w, e) = dummy();
        let mut seq = Sequence::new(vec![Box::new(AlwaysOk), Box::new(AlwaysRun), Box::new(AlwaysOk)]);
        assert_eq!(seq.tick(&mut w, e, 0.016), BehaviorStatus::Running);
    }

    #[test]
    fn selector_succeeds_on_first_success() {
        let (mut w, e) = dummy();
        let mut sel = Selector::new(vec![Box::new(AlwaysFail), Box::new(AlwaysOk), Box::new(AlwaysFail)]);
        assert_eq!(sel.tick(&mut w, e, 0.016), BehaviorStatus::Success);
    }

    #[test]
    fn selector_fails_when_all_fail() {
        let (mut w, e) = dummy();
        let mut sel = Selector::new(vec![Box::new(AlwaysFail), Box::new(AlwaysFail)]);
        assert_eq!(sel.tick(&mut w, e, 0.016), BehaviorStatus::Failure);
    }

    #[test]
    fn inverter_flips_success() {
        let (mut w, e) = dummy();
        let mut inv = Inverter::new(Box::new(AlwaysOk));
        assert_eq!(inv.tick(&mut w, e, 0.016), BehaviorStatus::Failure);
    }

    #[test]
    fn inverter_flips_failure() {
        let (mut w, e) = dummy();
        let mut inv = Inverter::new(Box::new(AlwaysFail));
        assert_eq!(inv.tick(&mut w, e, 0.016), BehaviorStatus::Success);
    }

    #[test]
    fn behavior_system_ticks_entity() {
        use std::sync::{Arc, Mutex};

        let ticked = Arc::new(Mutex::new(false));
        let ticked_clone = Arc::clone(&ticked);

        struct TickRecorder(Arc<Mutex<bool>>);
        impl BehaviorNode for TickRecorder {
            fn tick(&mut self, _: &mut World, _: Entity, _: f32) -> BehaviorStatus {
                *self.0.lock().unwrap() = true;
                BehaviorStatus::Success
            }
        }

        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, BehaviorTree::new(Box::new(TickRecorder(ticked_clone))));

        let mut sys = BehaviorSystem;
        sys.run(&mut world, 0.016);

        assert!(*ticked.lock().unwrap());
    }
}
