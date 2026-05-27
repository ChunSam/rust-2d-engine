use crate::ecs::{Entity, World};

/// 지연 실행할 World 수정 함수의 타입 별칭.
type DeferredFn = Box<dyn FnOnce(&mut World) + Send>;

/// ECS 시스템 실행 중 엔티티/컴포넌트 변경을 안전하게 지연 예약하는 버퍼.
///
/// 시스템이 `world`를 이미 빌리고 있는 도중에는 `world.spawn()` 등을 직접 호출할 수
/// 없다. `Commands`는 수정 명령을 클로저로 큐에 쌓아두었다가 시스템 실행이 끝난 뒤
/// `world.apply_commands(cmds)` 로 일괄 적용한다.
///
/// # 예시
///
/// ```rust,ignore
/// use engine::{Commands, System, World};
///
/// struct SpawnSystem;
/// impl System for SpawnSystem {
///     fn run(&mut self, world: &mut World, _dt: f32) {
///         let mut cmds = Commands::new();
///
///         // 새 엔티티 스폰
///         cmds.spawn(|world, e| {
///             world.add_component(e, MyTag);
///         });
///
///         // 기존 엔티티에 컴포넌트 추가
///         let entities: Vec<_> = world.query::<MyComp>().map(|(e, _)| e).collect();
///         for entity in entities {
///             cmds.insert(entity, NewComp { value: 42 });
///         }
///
///         world.apply_commands(cmds);
///     }
/// }
/// ```
pub struct Commands {
    deferred: Vec<DeferredFn>,
}

impl Commands {
    /// 빈 Commands 버퍼를 생성한다.
    pub fn new() -> Self {
        Self {
            deferred: Vec::new(),
        }
    }

    /// 새 엔티티를 스폰하는 명령을 예약한다.
    ///
    /// 클로저는 `apply` 시 실제 생성된 `Entity`와 `&mut World`를 인수로 받으므로,
    /// 클로저 안에서 컴포넌트를 자유롭게 추가할 수 있다.
    ///
    /// ```rust,ignore
    /// cmds.spawn(|world, e| {
    ///     world.add_component(e, Transform::default());
    ///     world.add_component(e, Sprite::colored(1.0, 0.0, 0.0));
    /// });
    /// ```
    pub fn spawn(&mut self, f: impl FnOnce(&mut World, Entity) + Send + 'static) {
        self.deferred.push(Box::new(move |world: &mut World| {
            let e = world.spawn();
            f(world, e);
        }));
    }

    /// 기존 엔티티를 삭제하는 명령을 예약한다.
    ///
    /// `apply` 시점에 엔티티가 이미 삭제되어 있으면 조용히 무시된다 (멱등성 보장).
    pub fn despawn(&mut self, entity: Entity) {
        self.deferred.push(Box::new(move |world: &mut World| {
            world.despawn(entity);
        }));
    }

    /// 기존 엔티티에 컴포넌트를 추가하는 명령을 예약한다.
    ///
    /// 이미 동일 타입의 컴포넌트가 있으면 교체된다.
    /// `apply` 시점에 엔티티가 존재하지 않으면 조용히 무시된다.
    pub fn insert<T: Send + Sync + 'static>(&mut self, entity: Entity, comp: T) {
        self.deferred.push(Box::new(move |world: &mut World| {
            world.add_component(entity, comp);
        }));
    }

    /// 기존 엔티티에서 컴포넌트를 제거하는 명령을 예약한다.
    ///
    /// 컴포넌트가 없거나 엔티티가 존재하지 않으면 조용히 무시된다 (멱등성 보장).
    pub fn remove<T: Send + Sync + 'static>(&mut self, entity: Entity) {
        self.deferred.push(Box::new(move |world: &mut World| {
            world.remove_component::<T>(entity);
        }));
    }

    /// 버퍼에 쌓인 모든 명령을 순서대로 World에 적용한다.
    ///
    /// 일반적으로는 `world.apply_commands(cmds)` 를 사용한다.
    pub fn apply(self, world: &mut World) {
        for f in self.deferred {
            f(world);
        }
    }
}

impl Default for Commands {
    fn default() -> Self {
        Self::new()
    }
}

// ─── 단위 테스트 ──────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Health(u32);

    #[derive(Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    /// 1. spawn — Commands::spawn 후 apply 하면 새 엔티티가 월드에 생성됨
    #[test]
    fn spawn_creates_entity() {
        let mut world = World::new();
        let mut cmds = Commands::new();

        cmds.spawn(|world, e| {
            world.add_component(e, Health(100));
        });

        assert_eq!(world.entity_count(), 0);
        world.apply_commands(cmds);
        assert_eq!(world.entity_count(), 1);

        let count = world.query::<Health>().count();
        assert_eq!(count, 1);

        let health = world.query::<Health>().next().map(|(_, h)| h.0).unwrap();
        assert_eq!(health, 100);
    }

    /// 2. despawn — Commands::despawn 후 apply 하면 엔티티 제거됨
    #[test]
    fn despawn_removes_entity() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, Health(50));

        let mut cmds = Commands::new();
        cmds.despawn(e);

        assert_eq!(world.entity_count(), 1);
        world.apply_commands(cmds);
        assert_eq!(world.entity_count(), 0);
        assert!(!world.is_alive(e));
    }

    /// 3. insert — 기존 엔티티에 컴포넌트 추가됨
    #[test]
    fn insert_adds_component() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, Health(10));

        let mut cmds = Commands::new();
        cmds.insert(e, Position { x: 1.0, y: 2.0 });

        assert!(world.get::<Position>(e).is_none());
        world.apply_commands(cmds);
        let pos = world.get::<Position>(e).unwrap();
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
    }

    /// 4. remove — 기존 엔티티에서 컴포넌트 제거됨
    #[test]
    fn remove_removes_component() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, Health(99));
        world.add_component(e, Position { x: 5.0, y: 5.0 });

        let mut cmds = Commands::new();
        cmds.remove::<Position>(e);

        assert!(world.get::<Position>(e).is_some());
        world.apply_commands(cmds);
        assert!(world.get::<Position>(e).is_none());
        // Health는 그대로 유지
        assert_eq!(world.get::<Health>(e).unwrap().0, 99);
    }

    /// 5. 순서 보장 — spawn → insert 순서로 apply될 때 정상 동작
    #[test]
    fn spawn_then_insert_ordering() {
        let mut world = World::new();
        let mut cmds = Commands::new();

        // spawn과 동시에 컴포넌트 추가 (클로저 내에서)
        cmds.spawn(|world, e| {
            world.add_component(e, Health(42));
            world.add_component(e, Position { x: 10.0, y: 20.0 });
        });

        world.apply_commands(cmds);

        let results: Vec<_> = world.query2::<Health, Position>().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1 .0, 42);
        assert_eq!(results[0].2.x, 10.0);
        assert_eq!(results[0].2.y, 20.0);
    }

    /// 6. 복수 spawn — 여러 엔티티를 한 번에 예약해도 모두 생성됨
    #[test]
    fn multiple_spawns() {
        let mut world = World::new();
        let mut cmds = Commands::new();

        for i in 0..5u32 {
            cmds.spawn(move |world, e| {
                world.add_component(e, Health(i * 10));
            });
        }

        world.apply_commands(cmds);
        assert_eq!(world.entity_count(), 5);
        assert_eq!(world.query::<Health>().count(), 5);
    }

    /// 7. despawn이 존재하지 않는 엔티티에도 panic 하지 않음 (멱등성)
    #[test]
    fn despawn_nonexistent_is_noop() {
        let mut world = World::new();
        let e = world.spawn();
        world.despawn(e); // 직접 제거

        let mut cmds = Commands::new();
        cmds.despawn(e); // 이미 없는 엔티티
        world.apply_commands(cmds); // panic 없음
    }

    /// 8. insert가 존재하지 않는 엔티티에도 panic 하지 않음
    #[test]
    fn insert_nonexistent_entity_is_noop() {
        let mut world = World::new();
        let e = world.spawn();
        world.despawn(e);

        let mut cmds = Commands::new();
        cmds.insert(e, Health(1));
        world.apply_commands(cmds); // panic 없음
    }
}
