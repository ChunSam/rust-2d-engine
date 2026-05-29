# Entity Generation v2 Plan

> **Status: Cancelled / archived (2026-05-29).** Removed from the planned/scheduled work
> during the vision reset (`docs/VISION.md`): it is a v2-only breaking change, not breadth
> and not example-validated, so it does not match the current direction. The design below
> is preserved for reference in case generation-checked handles are revisited for a future
> v2.0.0.

작성일: 2026-05-29
상태: Cancelled / archived (원래: v2 후보 설계 확정안)

## 요약

현재 `Entity(pub u32)`는 despawn 후 ID를 재사용한다. 오래 보관된 `Entity` 값이 새 엔티티를 가리킬 수 있어 `SelectedEntity`, `Parent`, `Children`, 스크립트 despawn 요청, 게임 코드 캐시에서 의도와 다른 조작이 발생할 수 있다.

v1에서는 이 동작을 문서화하고 `is_alive()` 확인을 권장한다. v2에서는 `Entity`를 세대 번호가 포함된 핸들로 바꿔 stale handle이 자동으로 실패하도록 한다.

## 결정 사항

- `Entity`는 v2에서 tuple struct가 아니라 불투명 핸들로 바꾼다.
  ```rust
  pub struct Entity {
      index: u32,
      generation: u32,
  }
  ```
- 공개 접근은 메서드로 제공한다.
  - `Entity::index(self) -> u32`
  - `Entity::generation(self) -> u32`
  - `Entity::from_raw_parts(index: u32, generation: u32) -> Self`
- `Entity(pub u32)`와 `entity.0` 직접 접근은 제거한다. UI와 로그는 `entity.index()`를 사용하고, 디버그 문자열은 `Debug` 출력 또는 `format!("Entity {}:{}", entity.index(), entity.generation())`를 사용한다.
- `World`는 슬롯별 현재 세대를 보관한다.
  - `next_id: u32`는 `next_index: u32`로 이름을 바꾼다.
  - `free_ids: VecDeque<u32>`는 `free_indices: VecDeque<u32>`로 바꾼다.
  - `generations: Vec<u32>`를 추가해 `generations[index]`가 현재 세대가 되게 한다.
- `spawn()` 동작:
  - 새 슬롯이면 `generation = 0`으로 시작한다.
  - 재사용 슬롯이면 이미 증가된 `generations[index]`를 사용한다.
- `despawn(entity)` 동작:
  - `is_alive(entity)`가 false면 즉시 no-op이다.
  - 제거가 성공하면 해당 index의 generation을 1 증가시키고 free queue에 index를 넣는다.
  - generation이 `u32::MAX`이면 그 index는 재사용하지 않고 은퇴시킨다.
- 모든 `World` API는 세대가 맞는 살아있는 핸들에만 작동한다.
  - `get`, `get_mut`, `add_component`, `remove_component`, `take_component`, `mark_changed`, `clone_entity`, `has_component_typeid`는 stale handle을 없는 엔티티처럼 처리한다.
  - `query*`, `entities()`, `par_query*`는 항상 현재 세대의 살아있는 핸들만 반환한다.
- `entity_location` 키는 `Entity`를 유지한다. 세대가 다르면 키가 다르므로 stale handle 조회는 실패한다.
- change tracking set(`added_this_tick`, `changed_this_tick`)은 `(Entity, TypeId)` 그대로 유지한다. despawn 시 현재 핸들의 항목만 제거하면 stale 세대 항목과 충돌하지 않는다.

## 마이그레이션 영향

- 외부 코드는 `entity.0`을 `entity.index()`로 바꿔야 한다.
- 오래 보관한 `Entity`가 새 엔티티를 조작하던 기존 동작은 v2에서 실패한다. 이는 의도된 breaking fix다.
- `Commands::despawn/insert/remove`는 별도 변경 없이 stale handle no-op 정책을 따른다.
- `Parent(Entity)`, `Children(Vec<Entity>)`, `SelectedEntity`, 이벤트 payload의 `Entity`는 타입은 유지되지만 stale safety가 추가된다.
- `ScriptingSystem::despawn_entity(id)`는 v2에서 현재 `i64 -> Entity(index)` 변환을 제거한다. 스크립트 API는 `despawn_entity(index, generation)` 또는 엔진이 발급한 opaque handle 문자열 중 하나로 바꾼다. 권장안은 Rust API와 같은 `index, generation` 2개 인자다.
- 씬/프리팹 직렬화는 태그와 계층 이름 기반 복원 구조를 유지한다. 런타임 `Entity` 값은 저장 포맷에 넣지 않는다.

## 구현 순서

1. `Entity` 구조체와 접근자 메서드를 추가하고, 내부 코드의 `entity.0` 사용을 `entity.index()`로 교체한다.
2. `World`에 `generations`와 `free_indices`를 추가하고 `spawn/despawn/is_alive`를 세대 검증 기반으로 바꾼다.
3. stale handle no-op 정책이 모든 컴포넌트 API와 commands 경로에서 지켜지는지 단위 테스트를 추가한다.
4. editor/debug UI 표시, hierarchy, prefab, scripting, physics/network event 사용처를 새 접근자 기반으로 갱신한다.
5. `REFERENCE.html`, `README.md`, `docs/CHANGELOG.md`에 v2 breaking change와 migration note를 추가한다.

## 필수 테스트

- `despawn` 후 같은 index가 재사용돼도 이전 `Entity`로 `is_alive`, `get`, `get_mut`, `add_component`, `remove_component`, `despawn`이 모두 실패하거나 no-op이어야 한다.
- 재사용된 새 `Entity`는 같은 index와 증가한 generation을 가져야 하며 정상 쿼리되어야 한다.
- `Commands`에 stale handle을 넣고 `apply_commands`를 호출해도 새 엔티티가 변하지 않아야 한다.
- `Parent`/`Children`에 stale handle이 남아도 hierarchy update가 새 엔티티를 부모/자식으로 오인하지 않아야 한다.
- `clone_entity(stale)`는 빈 새 엔티티를 만들지 않고 `Option<Entity>` 반환으로 바꾸거나, 기존 시그니처를 유지해야 한다면 빈 엔티티 생성 정책을 명시 테스트해야 한다. 권장안은 v2에서 `clone_entity(src) -> Option<Entity>`로 바꾸는 것이다.

## 보류하지 않을 결정

- v2에서는 하위 호환 tuple field를 유지하지 않는다.
- v2에서는 stale handle을 panic으로 처리하지 않는다. 게임 루프 안정성을 위해 현재 no-op 계열 정책을 유지한다.
- v1.x에는 이 변경을 넣지 않는다. 공개 API와 `entity.0` 사용 패턴을 깨므로 v2 전용이다.
