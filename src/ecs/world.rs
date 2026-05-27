use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub u32);

/// 컴포넌트 저장 단위. `Send + Sync`를 요구해 병렬 쿼리를 가능하게 한다.
type ComponentBox = Box<dyn Any + Send + Sync>;
type CloneComponentFn = Box<dyn Fn(&mut World, Entity, Entity) + Send + Sync>;

// ─── Reflect 레지스트리 헬퍼 ─────────────────────────────────────────────────

fn get_reflect_impl<T: crate::reflect::Reflect + 'static>(
    b: &ComponentBox,
) -> Option<&dyn crate::reflect::Reflect> {
    b.downcast_ref::<T>()
        .map(|t| t as &dyn crate::reflect::Reflect)
}

fn get_reflect_mut_impl<T: crate::reflect::Reflect + 'static>(
    b: &mut ComponentBox,
) -> Option<&mut dyn crate::reflect::Reflect> {
    b.downcast_mut::<T>()
        .map(|t| t as &mut dyn crate::reflect::Reflect)
}

#[derive(Copy, Clone)]
struct ReflectEntry {
    get: fn(&ComponentBox) -> Option<&dyn crate::reflect::Reflect>,
    get_mut: fn(&mut ComponentBox) -> Option<&mut dyn crate::reflect::Reflect>,
    type_name: &'static str,
}

type ArchetypeId = usize;

/// 동일한 컴포넌트 집합을 가진 엔티티 묶음.
/// columns[T][i]는 entities[i]의 T 컴포넌트이다 (항상 동일한 길이).
struct Archetype {
    type_set: Vec<TypeId>, // 정렬된 TypeId 목록
    entities: Vec<Entity>,
    columns: HashMap<TypeId, Vec<ComponentBox>>,
}

impl Archetype {
    fn new(type_set: Vec<TypeId>) -> Self {
        let columns = type_set
            .iter()
            .map(|&t| (t, Vec::<ComponentBox>::new()))
            .collect();
        Self {
            type_set,
            entities: Vec::new(),
            columns,
        }
    }

    fn contains(&self, tid: TypeId) -> bool {
        self.type_set.binary_search(&tid).is_ok()
    }
}

/// ECS의 중심 저장소
///
/// - 엔티티(Entity): 단순한 u32 ID
/// - 컴포넌트: Archetype 기반 밀집 컬럼 스토리지
/// - 리소스: 전역 싱글턴 데이터
pub struct World {
    next_id: u32,
    free_ids: VecDeque<u32>,
    entities: Vec<Entity>,
    archetypes: Vec<Archetype>,
    archetype_index: HashMap<Vec<TypeId>, ArchetypeId>,
    entity_location: HashMap<Entity, (ArchetypeId, usize)>,
    resources: HashMap<TypeId, Box<dyn Any>>,
    reflect_registry: HashMap<TypeId, ReflectEntry>,
    added_this_tick: HashSet<(Entity, TypeId)>,
    changed_this_tick: HashSet<(Entity, TypeId)>,
    /// clone_entity에서 컴포넌트를 복제할 때 사용하는 함수 레지스트리.
    clone_registry: HashMap<TypeId, CloneComponentFn>,
}

impl World {
    pub fn new() -> Self {
        let empty_arch = Archetype::new(vec![]);
        let mut archetype_index = HashMap::new();
        archetype_index.insert(vec![], 0);
        Self {
            next_id: 0,
            free_ids: VecDeque::new(),
            entities: Vec::new(),
            archetypes: vec![empty_arch],
            archetype_index,
            entity_location: HashMap::new(),
            resources: HashMap::new(),
            reflect_registry: HashMap::new(),
            added_this_tick: HashSet::new(),
            changed_this_tick: HashSet::new(),
            clone_registry: HashMap::new(),
        }
    }

    /// 현재 살아있는 엔티티 수를 반환한다.
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// 빈 엔티티를 생성하고 반환한다.
    pub fn spawn(&mut self) -> Entity {
        let id = if let Some(reused) = self.free_ids.pop_front() {
            reused
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        };
        let entity = Entity(id);
        let row = self.archetypes[0].entities.len();
        self.archetypes[0].entities.push(entity);
        self.entity_location.insert(entity, (0, row));
        self.entities.push(entity);
        entity
    }

    /// 엔티티를 제거하고 모든 컴포넌트를 해제한다. 멱등성 보장.
    pub fn despawn(&mut self, entity: Entity) {
        let (arch_id, row) = match self.entity_location.get(&entity) {
            Some(&loc) => loc,
            None => return,
        };

        let arch_len = self.archetypes[arch_id].entities.len();
        let type_set: Vec<TypeId> = self.archetypes[arch_id].type_set.clone();

        for &tid in &type_set {
            self.archetypes[arch_id]
                .columns
                .get_mut(&tid)
                .unwrap()
                .swap_remove(row);
        }
        self.archetypes[arch_id].entities.swap_remove(row);

        if row < arch_len - 1 {
            let swapped = self.archetypes[arch_id].entities[row];
            self.entity_location.insert(swapped, (arch_id, row));
        }

        if let Some(pos) = self.entities.iter().position(|&e| e == entity) {
            self.entities.swap_remove(pos);
        }

        self.entity_location.remove(&entity);
        self.free_ids.push_back(entity.0);
        self.added_this_tick.retain(|(e, _)| *e != entity);
        self.changed_this_tick.retain(|(e, _)| *e != entity);
    }

    /// 엔티티에서 컴포넌트 T를 제거한다. 없어도 panic 하지 않는다.
    pub fn remove_component<T: 'static>(&mut self, entity: Entity) {
        let tid = TypeId::of::<T>();
        let (arch_id, _) = match self.entity_location.get(&entity) {
            Some(&loc) => loc,
            None => return,
        };

        if !self.archetypes[arch_id].contains(tid) {
            return;
        }

        let new_sig: Vec<TypeId> = self.archetypes[arch_id]
            .type_set
            .iter()
            .copied()
            .filter(|&t| t != tid)
            .collect();

        let new_arch_id = self.get_or_create_archetype(new_sig);
        self.move_entity(entity, new_arch_id);
        self.added_this_tick.remove(&(entity, tid));
        self.changed_this_tick.remove(&(entity, tid));
    }

    /// 엔티티에서 컴포넌트 T를 꺼내 반환한다. 없으면 None.
    ///
    /// `remove_component`와 달리 컴포넌트 값을 소유권째 돌려준다.
    /// `BehaviorSystem` 등 컴포넌트를 임시로 빌려서 World와 동시에 사용해야 할 때 쓴다.
    pub fn take_component<T: Send + Sync + 'static>(&mut self, entity: Entity) -> Option<T> {
        let tid = TypeId::of::<T>();
        // Step 1: 실제 값을 Box<()> placeholder로 교체하고 소유권을 확보
        let value: T = {
            let (arch_id, row) = *self.entity_location.get(&entity)?;
            let arch = &mut self.archetypes[arch_id];
            if !arch.contains(tid) {
                return None;
            }
            let col = arch.columns.get_mut(&tid)?;
            // 실제 값을 unit placeholder로 swap
            let placeholder: ComponentBox = Box::new(());
            let extracted = std::mem::replace(&mut col[row], placeholder);
            // Box<dyn Any+Send+Sync> → Box<T> → T
            *extracted.downcast::<T>().ok()?
        }; // archetypes 빌림 해제
           // Step 2: placeholder를 포함한 슬롯을 아키타입에서 제거
        self.remove_component::<T>(entity);
        Some(value)
    }

    /// 엔티티에 컴포넌트를 붙인다. 이미 있으면 교체한다.
    ///
    /// `T: Send + Sync`가 요구된다 — 병렬 쿼리(`par_query*`)에서 스레드 간 공유를 허용하기 위해서다.
    pub fn add_component<T: Send + Sync + 'static>(&mut self, entity: Entity, component: T) {
        let tid = TypeId::of::<T>();
        let (arch_id, _) = match self.entity_location.get(&entity) {
            Some(&loc) => loc,
            None => return,
        };

        if self.archetypes[arch_id].contains(tid) {
            let (a, row) = self.entity_location[&entity];
            self.archetypes[a].columns.get_mut(&tid).unwrap()[row] = Box::new(component);
            self.changed_this_tick.insert((entity, tid));
            return;
        }

        let new_sig: Vec<TypeId> = {
            let arch = &self.archetypes[arch_id];
            let mut sig = arch.type_set.clone();
            let pos = sig.binary_search(&tid).unwrap_err();
            sig.insert(pos, tid);
            sig
        };

        let new_arch_id = self.get_or_create_archetype(new_sig);
        self.move_entity(entity, new_arch_id);

        let (na, _) = self.entity_location[&entity];
        self.archetypes[na]
            .columns
            .get_mut(&tid)
            .unwrap()
            .push(Box::new(component));
        self.added_this_tick.insert((entity, tid));
    }

    /// 엔티티의 컴포넌트를 불변 참조로 가져온다.
    pub fn get<T: 'static>(&self, entity: Entity) -> Option<&T> {
        let &(arch_id, row) = self.entity_location.get(&entity)?;
        self.archetypes[arch_id]
            .columns
            .get(&TypeId::of::<T>())?
            .get(row)?
            .downcast_ref::<T>()
    }

    /// 엔티티의 컴포넌트를 가변 참조로 가져온다.
    pub fn get_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        let &(arch_id, row) = self.entity_location.get(&entity)?;
        self.archetypes[arch_id]
            .columns
            .get_mut(&TypeId::of::<T>())?
            .get_mut(row)?
            .downcast_mut::<T>()
    }

    /// T를 가진 모든 (Entity, &T) 쌍을 순회한다.
    pub fn query<T: 'static>(&self) -> impl Iterator<Item = (Entity, &T)> {
        let tid = TypeId::of::<T>();
        self.archetypes
            .iter()
            .filter(move |arch| arch.contains(tid))
            .flat_map(move |arch| {
                let col = arch.columns.get(&tid).unwrap();
                arch.entities
                    .iter()
                    .zip(col.iter())
                    .map(|(&e, c)| (e, c.downcast_ref::<T>().unwrap()))
            })
    }

    /// A, B 를 모두 가진 엔티티를 순회한다.
    pub fn query2<A: 'static, B: 'static>(&self) -> impl Iterator<Item = (Entity, &A, &B)> {
        let ta = TypeId::of::<A>();
        let tb = TypeId::of::<B>();
        self.archetypes
            .iter()
            .filter(move |arch| arch.contains(ta) && arch.contains(tb))
            .flat_map(move |arch| {
                let ca = arch.columns.get(&ta).unwrap();
                let cb = arch.columns.get(&tb).unwrap();
                arch.entities.iter().enumerate().map(move |(i, &e)| {
                    (
                        e,
                        ca[i].downcast_ref::<A>().unwrap(),
                        cb[i].downcast_ref::<B>().unwrap(),
                    )
                })
            })
    }

    /// A, B, C 를 모두 가진 엔티티를 순회한다.
    pub fn query3<A: 'static, B: 'static, C: 'static>(
        &self,
    ) -> impl Iterator<Item = (Entity, &A, &B, &C)> {
        let ta = TypeId::of::<A>();
        let tb = TypeId::of::<B>();
        let tc = TypeId::of::<C>();
        self.archetypes
            .iter()
            .filter(move |arch| arch.contains(ta) && arch.contains(tb) && arch.contains(tc))
            .flat_map(move |arch| {
                let ca = arch.columns.get(&ta).unwrap();
                let cb = arch.columns.get(&tb).unwrap();
                let cc = arch.columns.get(&tc).unwrap();
                arch.entities.iter().enumerate().map(move |(i, &e)| {
                    (
                        e,
                        ca[i].downcast_ref::<A>().unwrap(),
                        cb[i].downcast_ref::<B>().unwrap(),
                        cc[i].downcast_ref::<C>().unwrap(),
                    )
                })
            })
    }

    /// A, B, C, D 를 모두 가진 엔티티를 순회한다.
    pub fn query4<A: 'static, B: 'static, C: 'static, D: 'static>(
        &self,
    ) -> impl Iterator<Item = (Entity, &A, &B, &C, &D)> {
        let ta = TypeId::of::<A>();
        let tb = TypeId::of::<B>();
        let tc = TypeId::of::<C>();
        let td = TypeId::of::<D>();
        self.archetypes
            .iter()
            .filter(move |arch| {
                arch.contains(ta) && arch.contains(tb) && arch.contains(tc) && arch.contains(td)
            })
            .flat_map(move |arch| {
                let ca = arch.columns.get(&ta).unwrap();
                let cb = arch.columns.get(&tb).unwrap();
                let cc = arch.columns.get(&tc).unwrap();
                let cd = arch.columns.get(&td).unwrap();
                arch.entities.iter().enumerate().map(move |(i, &e)| {
                    (
                        e,
                        ca[i].downcast_ref::<A>().unwrap(),
                        cb[i].downcast_ref::<B>().unwrap(),
                        cc[i].downcast_ref::<C>().unwrap(),
                        cd[i].downcast_ref::<D>().unwrap(),
                    )
                })
            })
    }

    /// A 를 가지면서 B 도 가진 엔티티만 순회한다.
    pub fn query_with<A: 'static, B: 'static>(&self) -> impl Iterator<Item = (Entity, &A)> {
        let ta = TypeId::of::<A>();
        let tb = TypeId::of::<B>();
        self.archetypes
            .iter()
            .filter(move |arch| arch.contains(ta) && arch.contains(tb))
            .flat_map(move |arch| {
                let col = arch.columns.get(&ta).unwrap();
                arch.entities
                    .iter()
                    .zip(col.iter())
                    .map(|(&e, c)| (e, c.downcast_ref::<A>().unwrap()))
            })
    }

    /// A 를 가지면서 B 가 없는 엔티티만 순회한다.
    pub fn query_without<A: 'static, B: 'static>(&self) -> impl Iterator<Item = (Entity, &A)> {
        let ta = TypeId::of::<A>();
        let tb = TypeId::of::<B>();
        self.archetypes
            .iter()
            .filter(move |arch| arch.contains(ta) && !arch.contains(tb))
            .flat_map(move |arch| {
                let col = arch.columns.get(&ta).unwrap();
                arch.entities
                    .iter()
                    .zip(col.iter())
                    .map(|(&e, c)| (e, c.downcast_ref::<A>().unwrap()))
            })
    }

    /// A 를 가진 모든 엔티티를 순회한다. B 는 있으면 Some, 없으면 None.
    pub fn query_opt2<A: 'static, B: 'static>(
        &self,
    ) -> impl Iterator<Item = (Entity, &A, Option<&B>)> {
        let ta = TypeId::of::<A>();
        let tb = TypeId::of::<B>();
        self.archetypes
            .iter()
            .filter(move |arch| arch.contains(ta))
            .flat_map(move |arch| {
                let ca = arch.columns.get(&ta).unwrap();
                let cb = arch.columns.get(&tb);
                arch.entities.iter().enumerate().map(move |(i, &e)| {
                    let a = ca[i].downcast_ref::<A>().unwrap();
                    let b = cb.map(|col| col[i].downcast_ref::<B>().unwrap());
                    (e, a, b)
                })
            })
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    // ── 리소스 ────────────────────────────────────────────────────────────────

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

    /// 리소스 `T`를 World에서 제거하고 소유권을 반환한다.
    /// 리소스가 없으면 `None`.
    pub fn remove_resource<T: 'static>(&mut self) -> Option<T> {
        self.resources
            .remove(&TypeId::of::<T>())
            .and_then(|b| b.downcast::<T>().ok().map(|b| *b))
    }

    // ── Reflect 레지스트리 ────────────────────────────────────────────────────

    /// 타입 T를 Reflect 레지스트리에 등록한다.
    ///
    /// 등록된 타입은 `get_reflect`, `get_reflect_mut`, `reflected_components`로 접근할 수 있으며
    /// egui Inspector 패널에 자동으로 표시된다.
    pub fn register_reflect<T: crate::reflect::Reflect + 'static>(&mut self) {
        self.reflect_registry.insert(
            TypeId::of::<T>(),
            ReflectEntry {
                get: get_reflect_impl::<T>,
                get_mut: get_reflect_mut_impl::<T>,
                type_name: "",
            },
        );
    }

    /// 타입 T를 Reflect 레지스트리에 표시 이름과 함께 등록한다.
    ///
    /// `register_reflect`와 동일하지만 Inspector 컴포넌트 목록에 표시될 이름을 명시한다.
    /// `reflect_registered_types()`로 조회할 때 이 이름이 반환된다.
    pub fn register_reflect_named<T: crate::reflect::Reflect + 'static>(
        &mut self,
        name: &'static str,
    ) {
        self.reflect_registry.insert(
            TypeId::of::<T>(),
            ReflectEntry {
                get: get_reflect_impl::<T>,
                get_mut: get_reflect_mut_impl::<T>,
                type_name: name,
            },
        );
    }

    /// Reflect 레지스트리에 등록된 모든 타입의 `(TypeId, type_name)` 목록을 반환한다.
    pub fn reflect_registered_types(&self) -> Vec<(TypeId, &'static str)> {
        self.reflect_registry
            .iter()
            .map(|(&tid, entry)| (tid, entry.type_name))
            .collect()
    }

    /// 엔티티의 특정 컴포넌트를 `&dyn Reflect`로 가져온다.
    ///
    /// `type_id`에 해당하는 컴포넌트가 없거나 등록되지 않은 타입이면 `None`.
    pub fn get_reflect(
        &self,
        entity: Entity,
        type_id: TypeId,
    ) -> Option<&dyn crate::reflect::Reflect> {
        let entry = self.reflect_registry.get(&type_id)?;
        let &(arch_id, row) = self.entity_location.get(&entity)?;
        let boxed = self.archetypes[arch_id].columns.get(&type_id)?.get(row)?;
        (entry.get)(boxed)
    }

    /// 엔티티의 특정 컴포넌트를 `&mut dyn Reflect`로 가져온다.
    ///
    /// `ReflectEntry`를 Copy로 꺼낸 뒤 Archetype을 가변 접근하므로 borrow 충돌 없음.
    pub fn get_reflect_mut(
        &mut self,
        entity: Entity,
        type_id: TypeId,
    ) -> Option<&mut dyn crate::reflect::Reflect> {
        let entry = *self.reflect_registry.get(&type_id)?; // Copy → borrow 해제
        let &(arch_id, row) = self.entity_location.get(&entity)?;
        let boxed = self.archetypes[arch_id]
            .columns
            .get_mut(&type_id)?
            .get_mut(row)?;
        (entry.get_mut)(boxed)
    }

    /// 엔티티가 가진 컴포넌트 중 Reflect 레지스트리에 등록된 타입의 `TypeId` 목록.
    pub fn reflected_components(&self, entity: Entity) -> Vec<TypeId> {
        let &(arch_id, _) = match self.entity_location.get(&entity) {
            Some(loc) => loc,
            None => return vec![],
        };
        self.archetypes[arch_id]
            .type_set
            .iter()
            .copied()
            .filter(|tid| self.reflect_registry.contains_key(tid))
            .collect()
    }

    /// 엔티티가 살아있는지 확인한다 (despawn 또는 미존재이면 false).
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entity_location.contains_key(&entity)
    }

    /// Commands 버퍼에 쌓인 모든 명령을 즉시 World에 적용한다.
    ///
    /// 시스템 실행이 끝난 뒤(쿼리 이터레이터가 모두 해제된 뒤) 호출해야
    /// borrow 충돌 없이 안전하게 엔티티/컴포넌트를 변경할 수 있다.
    ///
    /// ```rust,ignore
    /// fn run(&mut self, world: &mut World, _dt: f32) {
    ///     let mut cmds = Commands::new();
    ///     cmds.spawn(|world, e| { world.add_component(e, MyTag); });
    ///     world.apply_commands(cmds);
    /// }
    /// ```
    pub fn apply_commands(&mut self, commands: crate::ecs::Commands) {
        commands.apply(self);
    }

    // ── 변경 감지 ─────────────────────────────────────────────────────────────

    /// 이번 틱 변경 추적을 초기화한다. 매 프레임 시작 시 `App`이 호출한다.
    pub fn clear_change_tracking(&mut self) {
        self.added_this_tick.clear();
        self.changed_this_tick.clear();
    }

    /// 이번 틱에 *처음 추가된* 컴포넌트 T를 가진 엔티티만 조회한다.
    pub fn query_added<T: 'static>(&self) -> impl Iterator<Item = (Entity, &T)> {
        let tid = TypeId::of::<T>();
        let entities: Vec<Entity> = self
            .added_this_tick
            .iter()
            .filter(|(_, t)| *t == tid)
            .map(|(e, _)| *e)
            .collect();
        entities
            .into_iter()
            .filter_map(move |e| self.get::<T>(e).map(|c| (e, c)))
    }

    /// 이번 틱에 *교체된* 컴포넌트 T를 가진 엔티티만 조회한다.
    pub fn query_changed<T: 'static>(&self) -> impl Iterator<Item = (Entity, &T)> {
        let tid = TypeId::of::<T>();
        let entities: Vec<Entity> = self
            .changed_this_tick
            .iter()
            .filter(|(_, t)| *t == tid)
            .map(|(e, _)| *e)
            .collect();
        entities
            .into_iter()
            .filter_map(move |e| self.get::<T>(e).map(|c| (e, c)))
    }

    // ── 엔티티 복제 ───────────────────────────────────────────────────────────

    /// 컴포넌트 T를 `clone_entity`에서 복제할 수 있도록 등록한다.
    ///
    /// T는 Clone + Send + Sync + 'static 이어야 한다.
    /// 등록되지 않은 컴포넌트는 `clone_entity` 시 복사되지 않는다.
    pub fn register_clone<T: Clone + Send + Sync + 'static>(&mut self) {
        self.clone_registry.insert(
            TypeId::of::<T>(),
            Box::new(|world, src, dst| {
                if let Some(comp) = world.get::<T>(src) {
                    let cloned = comp.clone();
                    world.add_component(dst, cloned);
                }
            }),
        );
    }

    /// 엔티티를 복제한다. `register_clone`에 등록된 컴포넌트만 복사된다.
    ///
    /// `src`가 alive하지 않으면 빈 엔티티를 반환한다.
    /// 반환값: 새로 생성된 엔티티.
    pub fn clone_entity(&mut self, src: Entity) -> Entity {
        if !self.is_alive(src) {
            return self.spawn();
        }

        // 1. clone_registry에 등록된 TypeId 중 src 엔티티가 보유한 것만 수집
        let tids: Vec<TypeId> = self
            .clone_registry
            .keys()
            .filter(|&&tid| self.has_component_typeid(src, tid))
            .copied()
            .collect();

        // 2. 새 엔티티 생성
        let dst = self.spawn();

        // 3. remove → call → reinsert 패턴으로 borrow 충돌 없이 복제
        for tid in tids {
            self.clone_component_by_typeid(src, dst, tid);
        }

        dst
    }

    /// entity가 주어진 TypeId의 컴포넌트를 보유하고 있는지 확인한다.
    pub(crate) fn has_component_typeid(&self, entity: Entity, tid: TypeId) -> bool {
        match self.entity_location.get(&entity) {
            Some(&(arch_id, _)) => self.archetypes[arch_id].contains(tid),
            None => false,
        }
    }

    /// remove → clone_fn 호출 → reinsert 패턴으로 단일 TypeId 컴포넌트를 복제한다.
    fn clone_component_by_typeid(&mut self, src: Entity, dst: Entity, tid: TypeId) {
        if let Some(clone_fn) = self.clone_registry.remove(&tid) {
            clone_fn(self, src, dst);
            self.clone_registry.insert(tid, clone_fn);
        }
    }

    // ── 내부 헬퍼 ─────────────────────────────────────────────────────────────

    fn get_or_create_archetype(&mut self, sig: Vec<TypeId>) -> ArchetypeId {
        if let Some(&id) = self.archetype_index.get(&sig) {
            return id;
        }
        let id = self.archetypes.len();
        self.archetypes.push(Archetype::new(sig.clone()));
        self.archetype_index.insert(sig, id);
        id
    }

    /// entity를 target_arch_id로 이동한다. 공통 컴포넌트를 이전하며,
    /// 새로 추가되는 컴포넌트 push는 호출자가 담당한다.
    fn move_entity(&mut self, entity: Entity, target_arch_id: ArchetypeId) {
        let (src_arch_id, src_row) = self.entity_location[&entity];
        if src_arch_id == target_arch_id {
            return;
        }

        let src_len = self.archetypes[src_arch_id].entities.len();
        let src_type_set: Vec<TypeId> = self.archetypes[src_arch_id].type_set.clone();

        let mut extracted: HashMap<TypeId, ComponentBox> = HashMap::new();
        for &tid in &src_type_set {
            let comp = self.archetypes[src_arch_id]
                .columns
                .get_mut(&tid)
                .unwrap()
                .swap_remove(src_row);
            extracted.insert(tid, comp);
        }

        self.archetypes[src_arch_id].entities.swap_remove(src_row);

        if src_row < src_len - 1 {
            let swapped = self.archetypes[src_arch_id].entities[src_row];
            self.entity_location.insert(swapped, (src_arch_id, src_row));
        }

        let dst_row = self.archetypes[target_arch_id].entities.len();
        self.archetypes[target_arch_id].entities.push(entity);

        let dst_type_set: Vec<TypeId> = self.archetypes[target_arch_id].type_set.clone();
        for &tid in &dst_type_set {
            if let Some(comp) = extracted.remove(&tid) {
                self.archetypes[target_arch_id]
                    .columns
                    .get_mut(&tid)
                    .unwrap()
                    .push(comp);
            }
        }

        self.entity_location
            .insert(entity, (target_arch_id, dst_row));
    }
}

// ─── 병렬 쿼리 (native only — WASM은 단일 스레드) ──────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
impl World {
    /// T를 가진 모든 엔티티에 클로저를 **병렬**로 적용한다 (읽기 전용).
    ///
    /// 결과를 수집할 때는 클로저 내에서 `Mutex` 또는 채널을 사용하거나,
    /// 반환값이 필요하면 `par_query_map`을 사용한다.
    ///
    /// ```text
    /// world.par_query_for_each::<Transform, _>(|e, t| {
    ///     println!("{e:?} pos={}", t.position);
    /// });
    /// ```
    pub fn par_query_for_each<T, F>(&self, f: F)
    where
        T: Send + Sync + 'static,
        F: Fn(Entity, &T) + Send + Sync,
    {
        use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
        let tid = TypeId::of::<T>();
        self.archetypes
            .par_iter()
            .filter(|arch| arch.contains(tid))
            .for_each(|arch| {
                let col = arch.columns.get(&tid).unwrap();
                arch.entities
                    .par_iter()
                    .zip(col.par_iter())
                    .for_each(|(&e, c)| f(e, c.downcast_ref::<T>().unwrap()));
            });
    }

    /// T를 가진 모든 엔티티에 매핑 클로저를 **병렬**로 적용하고 결과를 `Vec<R>`로 반환한다.
    ///
    /// ```text
    /// let positions: Vec<(Entity, Vec2)> =
    ///     world.par_query_map::<Transform, _, _>(|e, t| (e, t.position));
    /// ```
    pub fn par_query_map<T, R, F>(&self, f: F) -> Vec<R>
    where
        T: Send + Sync + 'static,
        R: Send,
        F: Fn(Entity, &T) -> R + Send + Sync,
    {
        use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
        let tid = TypeId::of::<T>();
        self.archetypes
            .par_iter()
            .filter(|arch| arch.contains(tid))
            .flat_map(|arch| {
                let col = arch.columns.get(&tid).unwrap();
                arch.entities
                    .par_iter()
                    .zip(col.par_iter())
                    .map(|(&e, c)| f(e, c.downcast_ref::<T>().unwrap()))
            })
            .collect()
    }

    /// A, B를 모두 가진 엔티티에 클로저를 **병렬**로 적용한다 (읽기 전용).
    pub fn par_query2_for_each<A, B, F>(&self, f: F)
    where
        A: Send + Sync + 'static,
        B: Send + Sync + 'static,
        F: Fn(Entity, &A, &B) + Send + Sync,
    {
        use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
        let ta = TypeId::of::<A>();
        let tb = TypeId::of::<B>();
        self.archetypes
            .par_iter()
            .filter(move |arch| arch.contains(ta) && arch.contains(tb))
            .for_each(|arch| {
                let ca = arch.columns.get(&ta).unwrap();
                let cb = arch.columns.get(&tb).unwrap();
                arch.entities
                    .par_iter()
                    .zip(ca.par_iter())
                    .zip(cb.par_iter())
                    .for_each(|((&e, a), b)| {
                        f(
                            e,
                            a.downcast_ref::<A>().unwrap(),
                            b.downcast_ref::<B>().unwrap(),
                        );
                    });
            });
    }

    /// A, B를 모두 가진 엔티티에 매핑 클로저를 **병렬**로 적용하고 결과를 `Vec<R>`로 반환한다.
    pub fn par_query2_map<A, B, R, F>(&self, f: F) -> Vec<R>
    where
        A: Send + Sync + 'static,
        B: Send + Sync + 'static,
        R: Send,
        F: Fn(Entity, &A, &B) -> R + Send + Sync,
    {
        use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
        let ta = TypeId::of::<A>();
        let tb = TypeId::of::<B>();
        self.archetypes
            .par_iter()
            .filter(move |arch| arch.contains(ta) && arch.contains(tb))
            .flat_map(|arch| {
                let ca = arch.columns.get(&ta).unwrap();
                let cb = arch.columns.get(&tb).unwrap();
                arch.entities
                    .par_iter()
                    .zip(ca.par_iter())
                    .zip(cb.par_iter())
                    .map(|((&e, a), b)| {
                        f(
                            e,
                            a.downcast_ref::<A>().unwrap(),
                            b.downcast_ref::<B>().unwrap(),
                        )
                    })
            })
            .collect()
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
        assert!(positions.iter().all(|(e, _)| *e != e1));
    }

    #[test]
    fn despawn_is_idempotent() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, Health(50));
        world.despawn(e);
        world.despawn(e);
    }

    #[test]
    fn spawn_reuses_freed_id() {
        let mut world = World::new();
        let e_first = world.spawn();
        world.add_component(e_first, Health(99));

        world.despawn(e_first);

        let e_second = world.spawn();
        assert_eq!(e_first.0, e_second.0);
        assert!(world.get::<Health>(e_second).is_none());
    }

    #[test]
    fn query2_returns_only_entities_with_both() {
        let mut world = World::new();

        let ea = world.spawn();
        world.add_component(ea, Position { x: 0.0, y: 0.0 });

        let eb = world.spawn();
        world.add_component(eb, Health(10));

        let eab = world.spawn();
        world.add_component(eab, Position { x: 1.0, y: 1.0 });
        world.add_component(eab, Health(20));

        let results: Vec<_> = world.query2::<Position, Health>().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, eab);
    }

    #[test]
    fn remove_component_keeps_entity_alive() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, Position { x: 1.0, y: 2.0 });
        world.add_component(e, Health(50));

        world.remove_component::<Position>(e);

        assert!(world.get::<Position>(e).is_none());
        assert_eq!(world.get::<Health>(e).unwrap().0, 50);
        assert!(world.entities().contains(&e));
    }

    #[test]
    fn remove_component_nonexistent_is_noop() {
        let mut world = World::new();
        let e = world.spawn();
        world.remove_component::<Position>(e);
        world.add_component(e, Health(10));
        world.remove_component::<Velocity>(e);
        assert_eq!(world.get::<Health>(e).unwrap().0, 10);
    }

    #[test]
    fn query4_returns_only_entities_with_all_four() {
        struct Tag;

        let mut world = World::new();

        let eabc = world.spawn();
        world.add_component(eabc, Position { x: 0.0, y: 0.0 });
        world.add_component(eabc, Health(1));
        world.add_component(eabc, Velocity { vx: 0.0, vy: 0.0 });

        let eabcd = world.spawn();
        world.add_component(eabcd, Position { x: 1.0, y: 1.0 });
        world.add_component(eabcd, Health(2));
        world.add_component(eabcd, Velocity { vx: 1.0, vy: 0.0 });
        world.add_component(eabcd, Tag);

        let results: Vec<_> = world.query4::<Position, Health, Velocity, Tag>().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, eabcd);
    }

    #[test]
    fn query_opt2_returns_a_with_optional_b() {
        let mut world = World::new();

        let ea = world.spawn();
        world.add_component(ea, Position { x: 1.0, y: 0.0 });

        let eab = world.spawn();
        world.add_component(eab, Position { x: 2.0, y: 0.0 });
        world.add_component(eab, Health(50));

        let eb = world.spawn();
        world.add_component(eb, Health(99));

        let results: Vec<_> = world.query_opt2::<Position, Health>().collect();
        assert_eq!(results.len(), 2);

        let ea_result = results.iter().find(|(e, _, _)| *e == ea).unwrap();
        assert!(ea_result.2.is_none());

        let eab_result = results.iter().find(|(e, _, _)| *e == eab).unwrap();
        assert_eq!(eab_result.2.unwrap().0, 50);
    }

    #[test]
    fn query3_returns_only_entities_with_all_three() {
        let mut world = World::new();

        let eab = world.spawn();
        world.add_component(eab, Position { x: 0.0, y: 0.0 });
        world.add_component(eab, Health(5));

        let eabc = world.spawn();
        world.add_component(eabc, Position { x: 1.0, y: 1.0 });
        world.add_component(eabc, Health(10));
        world.add_component(eabc, Velocity { vx: 1.0, vy: 0.0 });

        let results: Vec<_> = world.query3::<Position, Health, Velocity>().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, eabc);
    }

    #[test]
    fn query_with_returns_only_entities_with_both() {
        let mut world = World::new();

        let e_pos_only = world.spawn();
        world.add_component(e_pos_only, Position { x: 0.0, y: 0.0 });

        let e_health_only = world.spawn();
        world.add_component(e_health_only, Health(10));

        let e_both = world.spawn();
        world.add_component(e_both, Position { x: 1.0, y: 1.0 });
        world.add_component(e_both, Health(50));

        let results: Vec<_> = world.query_with::<Position, Health>().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, e_both);
        assert_eq!(results[0].1.x, 1.0);
    }

    #[test]
    fn query_without_returns_only_entities_lacking_filter() {
        let mut world = World::new();

        let e_pos_only = world.spawn();
        world.add_component(e_pos_only, Position { x: 5.0, y: 0.0 });

        let e_both = world.spawn();
        world.add_component(e_both, Position { x: 2.0, y: 0.0 });
        world.add_component(e_both, Health(30));

        let results: Vec<_> = world.query_without::<Position, Health>().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, e_pos_only);
        assert_eq!(results[0].1.x, 5.0);
    }

    #[test]
    fn query_with_and_without_mixed_entities() {
        let mut world = World::new();

        // has Position + Health + Velocity
        let e_all = world.spawn();
        world.add_component(e_all, Position { x: 1.0, y: 0.0 });
        world.add_component(e_all, Health(100));
        world.add_component(e_all, Velocity { vx: 1.0, vy: 0.0 });

        // has Position + Health (no Velocity)
        let e_ph = world.spawn();
        world.add_component(e_ph, Position { x: 2.0, y: 0.0 });
        world.add_component(e_ph, Health(50));

        // has Position only
        let e_p = world.spawn();
        world.add_component(e_p, Position { x: 3.0, y: 0.0 });

        // has Health only
        let e_h = world.spawn();
        world.add_component(e_h, Health(10));

        // query_with::<Position, Health> → e_all, e_ph
        let with_results: Vec<Entity> = world
            .query_with::<Position, Health>()
            .map(|(e, _)| e)
            .collect();
        assert_eq!(with_results.len(), 2);
        assert!(with_results.contains(&e_all));
        assert!(with_results.contains(&e_ph));

        // query_without::<Position, Health> → e_p
        let without_results: Vec<Entity> = world
            .query_without::<Position, Health>()
            .map(|(e, _)| e)
            .collect();
        assert_eq!(without_results.len(), 1);
        assert_eq!(without_results[0], e_p);

        // query_with::<Position, Velocity> → e_all only
        let with_vel: Vec<Entity> = world
            .query_with::<Position, Velocity>()
            .map(|(e, _)| e)
            .collect();
        assert_eq!(with_vel.len(), 1);
        assert_eq!(with_vel[0], e_all);
    }

    #[test]
    fn archetype_reuse_across_entities() {
        let mut world = World::new();

        let e1 = world.spawn();
        world.add_component(e1, Position { x: 1.0, y: 0.0 });
        world.add_component(e1, Health(10));

        let e2 = world.spawn();
        world.add_component(e2, Position { x: 2.0, y: 0.0 });
        world.add_component(e2, Health(20));

        // 같은 컴포넌트 집합 → 같은 Archetype에 배치되어야 한다
        let (arch1, _) = world.entity_location[&e1];
        let (arch2, _) = world.entity_location[&e2];
        assert_eq!(arch1, arch2);
        assert_eq!(world.query2::<Position, Health>().count(), 2);
    }

    #[test]
    fn add_component_replaces_existing() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, Health(10));
        world.add_component(e, Health(99)); // replace
        assert_eq!(world.get::<Health>(e).unwrap().0, 99);
        assert_eq!(world.query::<Health>().count(), 1);
    }

    #[test]
    fn change_tracking_added() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, 42u32);
        let added: Vec<_> = world.query_added::<u32>().collect();
        assert_eq!(added.len(), 1);
        assert_eq!(*added[0].1, 42);
        // changed 에는 없어야 함
        assert_eq!(world.query_changed::<u32>().count(), 0);
        // clear 후 없어짐
        world.clear_change_tracking();
        assert_eq!(world.query_added::<u32>().count(), 0);
    }

    #[test]
    fn change_tracking_changed() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, 1u32);
        world.clear_change_tracking();
        // 교체
        world.add_component(e, 2u32);
        assert_eq!(world.query_added::<u32>().count(), 0);
        let changed: Vec<_> = world.query_changed::<u32>().collect();
        assert_eq!(changed.len(), 1);
        assert_eq!(*changed[0].1, 2);
    }

    #[test]
    fn change_tracking_despawn_clears() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, 99u32);
        assert_eq!(world.query_added::<u32>().count(), 1);
        world.despawn(e);
        assert_eq!(world.query_added::<u32>().count(), 0);
    }

    #[test]
    fn change_tracking_remove_clears() {
        let mut world = World::new();
        let e = world.spawn();
        world.add_component(e, 7u32);
        assert_eq!(world.query_added::<u32>().count(), 1);
        world.remove_component::<u32>(e);
        assert_eq!(world.query_added::<u32>().count(), 0);
    }

    #[test]
    fn clone_entity_copies_components() {
        let mut world = World::new();
        world.register_clone::<u32>();
        world.register_clone::<f32>();

        let src = world.spawn();
        world.add_component(src, 42u32);
        world.add_component(src, 3.125f32);

        let dst = world.clone_entity(src);
        assert_ne!(src, dst);
        assert_eq!(*world.get::<u32>(dst).unwrap(), 42);
        assert!((world.get::<f32>(dst).unwrap() - 3.125).abs() < 1e-5);
        // src still intact
        assert_eq!(*world.get::<u32>(src).unwrap(), 42);
    }

    #[test]
    fn clone_entity_skips_unregistered_types() {
        #[derive(Debug)]
        struct NotCloneable;
        unsafe impl Send for NotCloneable {}
        unsafe impl Sync for NotCloneable {}

        let mut world = World::new();
        world.register_clone::<u32>();

        let src = world.spawn();
        world.add_component(src, 99u32);
        world.add_component(src, NotCloneable);

        let dst = world.clone_entity(src);
        assert_eq!(*world.get::<u32>(dst).unwrap(), 99);
        assert!(world.get::<NotCloneable>(dst).is_none()); // not copied
    }

    #[test]
    fn clone_entity_dead_src_returns_empty() {
        let mut world = World::new();
        let src = world.spawn();
        world.despawn(src);

        let dst = world.clone_entity(src);
        assert!(world.is_alive(dst));
        assert_eq!(world.entity_count(), 1); // only dst
    }

    #[test]
    fn world_remove_resource_removes_and_returns() {
        let mut world = World::new();
        world.insert_resource(42u32);
        assert_eq!(world.resource::<u32>().copied(), Some(42));

        let v = world.remove_resource::<u32>();
        assert_eq!(v, Some(42));
        assert_eq!(world.resource::<u32>(), None);
    }

    #[test]
    fn world_remove_resource_missing_returns_none() {
        let mut world = World::new();
        let v = world.remove_resource::<u32>();
        assert_eq!(v, None);
    }
}
