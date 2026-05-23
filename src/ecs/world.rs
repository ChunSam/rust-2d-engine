use std::any::{Any, TypeId};
use std::collections::{HashMap, VecDeque};

/// 게임 오브젝트를 식별하는 고유 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub u32);

/// ECS의 중심 저장소
///
/// - 엔티티(Entity): 단순한 u32 ID
/// - 컴포넌트: TypeId 기준으로 구분되는 Vec<Option<Box<dyn Any>>>
/// - 리소스: 전역 싱글턴 데이터 (입력 상태, 물리 세계 등)
pub struct World {
    next_id: u32,
    entities: Vec<Entity>,
    // 서바이버처럼 수천 개 생성/소멸이 반복될 때 ID 누수를 막기 위한 재사용 큐
    free_ids: VecDeque<u32>,
    // TypeId -> Vec indexed by entity id (없으면 None)
    components: HashMap<TypeId, Vec<Option<Box<dyn Any>>>>,
    resources: HashMap<TypeId, Box<dyn Any>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            entities: Vec::new(),
            free_ids: VecDeque::new(),
            components: HashMap::new(),
            resources: HashMap::new(),
        }
    }

    /// 빈 엔티티를 생성하고 반환한다.
    /// free_ids 에 반납된 ID가 있으면 재사용, 없으면 next_id 증가.
    pub fn spawn(&mut self) -> Entity {
        let id = if let Some(reused) = self.free_ids.pop_front() {
            reused // despawn 때 None 처리가 보장되므로 슬롯은 이미 깨끗하다
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        };
        let entity = Entity(id);
        self.entities.push(entity);
        // 기존 컴포넌트 저장소를 새 엔티티 크기만큼 확장
        for vec in self.components.values_mut() {
            while vec.len() <= entity.0 as usize {
                vec.push(None);
            }
        }
        entity
    }

    /// 엔티티를 제거하고 모든 컴포넌트를 해제한다.
    /// 같은 엔티티를 두 번 호출해도 panic 하지 않는다 (멱등성).
    pub fn despawn(&mut self, entity: Entity) {
        let idx = entity.0 as usize;

        // entities 에서 swap_remove: O(1), &[Entity] 시그니처를 유지할 수 있다
        if let Some(pos) = self.entities.iter().position(|&e| e == entity) {
            self.entities.swap_remove(pos);

            // 모든 컴포넌트 슬롯을 None 으로 비운다
            for vec in self.components.values_mut() {
                if let Some(slot) = vec.get_mut(idx) {
                    *slot = None;
                }
            }

            // ID를 재사용 큐에 반납
            self.free_ids.push_back(entity.0);
        }
        // entities 에 없으면 이미 despawn 된 엔티티 → 아무것도 하지 않는다
    }

    /// 엔티티에 컴포넌트를 붙인다.
    pub fn add_component<T: 'static>(&mut self, entity: Entity, component: T) {
        let vec = self
            .components
            .entry(TypeId::of::<T>())
            .or_insert_with(Vec::new);
        while vec.len() <= entity.0 as usize {
            vec.push(None);
        }
        vec[entity.0 as usize] = Some(Box::new(component));
    }

    /// 엔티티의 컴포넌트를 불변 참조로 가져온다.
    pub fn get<T: 'static>(&self, entity: Entity) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())?
            .get(entity.0 as usize)?
            .as_ref()?
            .downcast_ref::<T>()
    }

    /// 엔티티의 컴포넌트를 가변 참조로 가져온다.
    pub fn get_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        self.components
            .get_mut(&TypeId::of::<T>())?
            .get_mut(entity.0 as usize)?
            .as_mut()?
            .downcast_mut::<T>()
    }

    /// 특정 컴포넌트 T를 가진 모든 (Entity, &T) 쌍을 순회한다.
    pub fn query<T: 'static>(&self) -> impl Iterator<Item = (Entity, &T)> {
        let type_id = TypeId::of::<T>();
        let entities = &self.entities;
        let components = self.components.get(&type_id);
        entities.iter().filter_map(move |&entity| {
            let comp = components?
                .get(entity.0 as usize)?
                .as_ref()?
                .downcast_ref::<T>()?;
            Some((entity, comp))
        })
    }

    /// 컴포넌트 A, B 를 모두 가진 엔티티를 순회한다.
    /// 둘 중 하나라도 없는 엔티티는 건너뛴다.
    pub fn query2<A: 'static, B: 'static>(&self) -> impl Iterator<Item = (Entity, &A, &B)> {
        // 두 타입의 슬라이스를 미리 꺼내 클로저 안에서 재빌림 없이 사용한다
        // (클로저가 &self 전체를 캡처하면 lifetime 추론이 복잡해지므로 분리)
        let ca = self.components.get(&TypeId::of::<A>());
        let cb = self.components.get(&TypeId::of::<B>());
        self.entities.iter().filter_map(move |&entity| {
            let idx = entity.0 as usize;
            let a = ca?.get(idx)?.as_ref()?.downcast_ref::<A>()?;
            let b = cb?.get(idx)?.as_ref()?.downcast_ref::<B>()?;
            Some((entity, a, b))
        })
    }

    /// 컴포넌트 A, B, C 를 모두 가진 엔티티를 순회한다.
    pub fn query3<A: 'static, B: 'static, C: 'static>(
        &self,
    ) -> impl Iterator<Item = (Entity, &A, &B, &C)> {
        let ca = self.components.get(&TypeId::of::<A>());
        let cb = self.components.get(&TypeId::of::<B>());
        let cc = self.components.get(&TypeId::of::<C>());
        self.entities.iter().filter_map(move |&entity| {
            let idx = entity.0 as usize;
            let a = ca?.get(idx)?.as_ref()?.downcast_ref::<A>()?;
            let b = cb?.get(idx)?.as_ref()?.downcast_ref::<B>()?;
            let c = cc?.get(idx)?.as_ref()?.downcast_ref::<C>()?;
            Some((entity, a, b, c))
        })
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    // ── 리소스 (전역 싱글턴) ────────────────────────────────────────────────

    pub fn insert_resource<T: 'static>(&mut self, resource: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(resource));
    }

    pub fn resource<T: 'static>(&self) -> Option<&T> {
        self.resources.get(&TypeId::of::<T>())?.downcast_ref::<T>()
    }

    pub fn resource_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.resources
            .get_mut(&TypeId::of::<T>())?
            .downcast_mut::<T>()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

// ─── 단위 테스트 ──────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    struct Position {
        x: f32,
        y: f32,
    }
    #[allow(dead_code)]
    struct Health(u32);
    #[allow(dead_code)]
    struct Velocity {
        vx: f32,
        vy: f32,
    }

    #[test]
    fn spawn_and_query() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, Position { x: 1.0, y: 2.0 });
        world.add_component(e, Health(100));

        let pos = world.get::<Position>(e).unwrap();
        assert_eq!(pos.x, 1.0);

        let count = world.query::<Position>().count();
        assert_eq!(count, 1);
    }

    #[test]
    fn resource() {
        let mut world = World::new();
        world.insert_resource(42u32);
        assert_eq!(*world.resource::<u32>().unwrap(), 42);
    }

    /// despawn 한 엔티티는 query 결과에서 사라져야 한다.
    #[test]
    fn despawn_removes_entity_from_query() {
        let mut world = World::new();
        let e0 = world.spawn();
        let e1 = world.spawn();
        let e2 = world.spawn();
        world.add_component(e0, Position { x: 0.0, y: 0.0 });
        world.add_component(e1, Position { x: 1.0, y: 0.0 });
        world.add_component(e2, Position { x: 2.0, y: 0.0 });

        world.despawn(e1);

        let positions: Vec<_> = world.query::<Position>().collect();
        assert_eq!(positions.len(), 2);
        // e1 은 결과에 없어야 한다
        assert!(positions.iter().all(|(e, _)| *e != e1));
    }

    /// 같은 엔티티를 두 번 despawn 해도 panic 하지 않아야 한다.
    #[test]
    fn despawn_is_idempotent() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, Health(50));
        world.despawn(e);
        world.despawn(e); // 두 번째 호출 — panic 없이 조용히 무시
    }

    /// despawn 후 spawn 하면 같은 ID 가 재사용되고, 이전 컴포넌트는 없어야 한다.
    #[test]
    fn spawn_reuses_freed_id() {
        let mut world = World::new();
        let e_first = world.spawn();
        world.add_component(e_first, Health(99));

        world.despawn(e_first);

        let e_second = world.spawn();
        // 반납된 ID 가 재사용돼야 한다
        assert_eq!(e_first.0, e_second.0);
        // 이전 컴포넌트가 남아있으면 안 된다
        assert!(world.get::<Health>(e_second).is_none());
    }

    /// query2 는 A, B 를 모두 가진 엔티티만 반환해야 한다.
    #[test]
    fn query2_returns_only_entities_with_both() {
        let mut world = World::new();

        // A 만 있는 엔티티
        let ea = world.spawn();
        world.add_component(ea, Position { x: 0.0, y: 0.0 });

        // B 만 있는 엔티티
        let eb = world.spawn();
        world.add_component(eb, Health(10));

        // A + B 를 모두 가진 엔티티
        let eab = world.spawn();
        world.add_component(eab, Position { x: 1.0, y: 1.0 });
        world.add_component(eab, Health(20));

        let results: Vec<_> = world.query2::<Position, Health>().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, eab);
    }

    /// query3 는 A, B, C 를 모두 가진 엔티티만 반환해야 한다.
    #[test]
    fn query3_returns_only_entities_with_all_three() {
        let mut world = World::new();

        // A + B 만 있는 엔티티
        let eab = world.spawn();
        world.add_component(eab, Position { x: 0.0, y: 0.0 });
        world.add_component(eab, Health(5));

        // A + B + C 를 모두 가진 엔티티
        let eabc = world.spawn();
        world.add_component(eabc, Position { x: 1.0, y: 1.0 });
        world.add_component(eabc, Health(10));
        world.add_component(eabc, Velocity { vx: 1.0, vy: 0.0 });

        let results: Vec<_> = world.query3::<Position, Health, Velocity>().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, eabc);
    }
}
