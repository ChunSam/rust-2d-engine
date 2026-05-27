use crate::ecs::{Entity, World};
use std::collections::VecDeque;

/// ECS 엔티티 재사용 풀.
///
/// 총알·파티클처럼 자주 생성/소멸되는 엔티티를 재활용해
/// archetype 재할당 비용을 줄인다.
///
/// # 사용 패턴
///
/// ```rust,ignore
/// // 리소스로 등록
/// world.insert_resource(Pool::new(32));
///
/// // 획득 (없으면 새 엔티티 스폰)
/// let bullet = pool.acquire(world, |w, e| {
///     w.add_component(e, Bullet::default());
///     w.add_component(e, Transform::default());
/// });
///
/// // 반납 (엔티티 유지, 비활성 마커 추가)
/// pool.release(bullet, world);
/// ```
pub struct Pool {
    available: VecDeque<Entity>,
    capacity: usize,
}

impl Pool {
    /// 최대 `capacity` 개 엔티티를 저장하는 풀을 생성한다.
    pub fn new(capacity: usize) -> Self {
        Self {
            available: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// 풀에서 엔티티를 가져온다.
    ///
    /// 사용 가능한 엔티티가 없으면 `world.spawn()`으로 새로 생성한다.
    /// `setup` 클로저에서 컴포넌트를 초기화한다.
    pub fn acquire(&mut self, world: &mut World, setup: impl FnOnce(&mut World, Entity)) -> Entity {
        // Try to reuse an existing entity
        while let Some(entity) = self.available.pop_front() {
            if world.is_alive(entity) {
                // Remove the Pooled marker to "activate" it
                world.remove_component::<Pooled>(entity);
                setup(world, entity);
                return entity;
            }
            // Entity was despawned externally — skip it
        }
        // No available entity — spawn a new one
        let entity = world.spawn();
        setup(world, entity);
        entity
    }

    /// 엔티티를 풀에 반납한다.
    ///
    /// 풀이 가득 차면(`capacity` 초과) 엔티티를 despawn한다.
    /// `Pooled` 마커 컴포넌트를 추가해 비활성 상태를 표시한다.
    pub fn release(&mut self, entity: Entity, world: &mut World) {
        if self.available.len() >= self.capacity {
            world.despawn(entity);
            return;
        }
        world.add_component(entity, Pooled);
        self.available.push_back(entity);
    }

    /// 현재 풀에서 사용 가능한 엔티티 수.
    pub fn available_count(&self) -> usize {
        self.available.len()
    }

    /// 풀의 최대 용량.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// 풀에 있는 모든 엔티티를 despawn하고 풀을 비운다.
    pub fn clear(&mut self, world: &mut World) {
        for entity in self.available.drain(..) {
            if world.is_alive(entity) {
                world.despawn(entity);
            }
        }
    }
}

/// 오브젝트 풀에 반납된 엔티티를 표시하는 마커 컴포넌트.
///
/// 이 컴포넌트를 가진 엔티티는 "비활성" 상태다.
/// 렌더링/시스템에서 `query_without::<Pooled>()` 로 제외할 수 있다.
#[derive(Debug, Clone, Copy, Default)]
pub struct Pooled;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::World;

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Bullet {
        speed: f32,
    }

    #[test]
    fn acquire_spawns_when_empty() {
        let mut world = World::new();
        let mut pool = Pool::new(4);
        let e = pool.acquire(&mut world, |w, e| {
            w.add_component(e, Bullet { speed: 10.0 });
        });
        assert!(world.is_alive(e));
        assert!(world.get::<Bullet>(e).is_some());
        assert_eq!(pool.available_count(), 0);
    }

    #[test]
    fn release_and_reacquire() {
        let mut world = World::new();
        let mut pool = Pool::new(4);
        let e1 = pool.acquire(&mut world, |w, e| {
            w.add_component(e, Bullet { speed: 5.0 });
        });
        pool.release(e1, &mut world);
        assert_eq!(pool.available_count(), 1);
        // Pooled marker should be added
        assert!(world.get::<Pooled>(e1).is_some());

        // Reacquire — should return same entity
        let e2 = pool.acquire(&mut world, |w, e| {
            w.add_component(e, Bullet { speed: 20.0 });
        });
        assert_eq!(e1, e2);
        assert_eq!(pool.available_count(), 0);
        // Pooled marker removed on reacquire
        assert!(world.get::<Pooled>(e2).is_none());
        assert_eq!(world.get::<Bullet>(e2).unwrap().speed, 20.0);
    }

    #[test]
    fn overflow_despawns_entity() {
        let mut world = World::new();
        let mut pool = Pool::new(1); // capacity = 1
        let e1 = pool.acquire(&mut world, |_, _| {});
        let e2 = pool.acquire(&mut world, |_, _| {});
        pool.release(e1, &mut world); // fills pool
        pool.release(e2, &mut world); // overflow → despawn e2
        assert!(world.is_alive(e1));
        assert!(!world.is_alive(e2));
    }

    #[test]
    fn clear_despawns_all() {
        let mut world = World::new();
        let mut pool = Pool::new(4);
        let e1 = pool.acquire(&mut world, |_, _| {});
        let e2 = pool.acquire(&mut world, |_, _| {});
        pool.release(e1, &mut world);
        pool.release(e2, &mut world);
        assert_eq!(pool.available_count(), 2);
        pool.clear(&mut world);
        assert_eq!(pool.available_count(), 0);
        assert!(!world.is_alive(e1));
        assert!(!world.is_alive(e2));
    }

    #[test]
    fn skips_externally_despawned_entity() {
        let mut world = World::new();
        let mut pool = Pool::new(4);
        let e = pool.acquire(&mut world, |_, _| {});
        pool.release(e, &mut world);
        // Externally despawn the pooled entity
        world.despawn(e);
        // acquire should gracefully skip dead entity and return a live entity
        let e2 = pool.acquire(&mut world, |_, _| {});
        assert!(world.is_alive(e2));
        assert_eq!(pool.available_count(), 0);
    }
}
