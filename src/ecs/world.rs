use std::any::{Any, TypeId};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub u32);

/// 컴포넌트 저장 단위. `Send + Sync`를 요구해 병렬 쿼리를 가능하게 한다.
type ComponentBox = Box<dyn Any + Send + Sync>;

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
            },
        );
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
    /// 반환값이 필요하면 [`par_query_map`]을 사용한다.
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
                        f(e, a.downcast_ref::<A>().unwrap(), b.downcast_ref::<B>().unwrap());
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
                        f(e, a.downcast_ref::<A>().unwrap(), b.downcast_ref::<B>().unwrap())
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
}
