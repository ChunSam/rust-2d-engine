//! 씬 직렬화 + 프리팹 시스템 (Phase 16)
//!
//! # 핵심 타입
//! - [`Tag`] — 엔티티 식별용 문자열 컴포넌트
//! - [`EntityDef`] — 하나의 엔티티를 기술하는 직렬화 가능 구조체
//! - [`SceneDef`] — 여러 [`EntityDef`]의 컬렉션 (레벨/씬 전체)
//! - [`Prefab`] — 파일 기반 단일 엔티티 템플릿
//!
//! # 빠른 사용 예
//! ```rust,no_run
//! use engine::prefab::{SceneDef, EntityDef, spawn_scene_def};
//! use engine::{Transform, Sprite, ecs::World};
//! use glam::Vec2;
//! use std::path::Path;
//!
//! let mut world = World::new();
//!
//! // 씬 정의 구성
//! let scene = SceneDef {
//!     entities: vec![
//!         EntityDef {
//!             tag: Some("player".into()),
//!             transform: Some(Transform::new(Vec2::ZERO, Vec2::splat(64.0), 0.0)),
//!             sprite: Some(Sprite::textured("assets/player.png")),
//!             parent: None,
//!         },
//!     ],
//!     ..SceneDef::default()
//! };
//!
//! // 파일 저장 후 로드
//! scene.save(Path::new("levels/level1.ron")).unwrap();
//! let loaded = SceneDef::load(Path::new("levels/level1.ron")).unwrap();
//! let entities = spawn_scene_def(&mut world, &loaded);
//! ```

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::components::{Sprite, Transform};
use crate::ecs::{Entity, World};
use crate::reflect::{Reflect, ReflectValue};
use crate::save::{load, save, SaveError};

// ─── Tag 컴포넌트 ─────────────────────────────────────────────────────────────

/// 엔티티 식별용 문자열 태그 컴포넌트.
///
/// 레벨 로드 후 "player", "enemy" 등의 역할을 구분하거나
/// 특정 엔티티를 쿼리할 때 사용한다.
///
/// # 예
/// ```rust,no_run
/// # use engine::prefab::Tag;
/// # use engine::ecs::World;
/// # let mut world = engine::ecs::World::new();
/// let e = world.spawn();
/// world.add_component(e, Tag("player".into()));
///
/// // 나중에 찾기
/// for (entity, tag) in world.query::<Tag>() {
///     if tag.0 == "player" { /* ... */ }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag(pub String);

impl Reflect for Tag {
    fn fields(&self) -> Vec<(&'static str, ReflectValue)> {
        vec![("tag", ReflectValue::String(self.0.clone()))]
    }
    fn set_field(&mut self, name: &str, val: ReflectValue) -> bool {
        match (name, val) {
            ("tag", ReflectValue::String(s)) => {
                self.0 = s;
                true
            }
            _ => false,
        }
    }
    fn type_name(&self) -> &'static str {
        "Tag"
    }
}

// ─── EntityDef ────────────────────────────────────────────────────────────────

/// 하나의 엔티티를 기술하는 직렬화 가능 구조체.
///
/// 각 필드는 `Option`이므로 필요한 컴포넌트만 지정하면 된다.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityDef {
    /// 엔티티 식별 태그 (선택)
    pub tag: Option<String>,
    /// 위치·크기·회전 (선택)
    pub transform: Option<Transform>,
    /// 텍스처·색상 (선택)
    pub sprite: Option<Sprite>,
    /// 부모 엔티티의 tag 문자열. None이면 루트 엔티티.
    /// 스폰 시 해당 태그를 가진 엔티티에 계층 연결된다.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}

// ─── SceneDef ─────────────────────────────────────────────────────────────────

/// 현재 `SceneDef` RON 포맷 버전. 구조 변경 시 증가시킨다.
pub const SCENE_DEF_VERSION: u32 = 1;

/// 레벨/씬 전체를 기술하는 직렬화 가능 구조체.
///
/// RON 파일 한 장이 하나의 `SceneDef`에 대응한다.
///
/// # RON 예시
/// ```ron
/// SceneDef(
///     version: 1,
///     entities: [
///         EntityDef(
///             tag: Some("ground"),
///             transform: Some(Transform(
///                 position: (0.0, -200.0),
///                 scale: (800.0, 32.0),
///                 rotation: 0.0,
///                 z: 0.0,
///             )),
///             sprite: Some(Sprite(
///                 texture: None,
///                 color: (0.3, 0.6, 0.3, 1.0),
///             )),
///         ),
///     ],
/// )
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneDef {
    /// 파일 포맷 버전. 버전 없는 구 형식 파일은 0으로 역직렬화된다.
    #[serde(default)]
    pub version: u32,
    pub entities: Vec<EntityDef>,
}

impl Default for SceneDef {
    fn default() -> Self {
        Self {
            version: SCENE_DEF_VERSION,
            entities: Vec::new(),
        }
    }
}

impl SceneDef {
    /// RON 파일에서 씬 정의를 로드한다.
    ///
    /// 파일 버전이 현재 버전과 다르면 경고를 출력하지만 로드는 계속한다.
    pub fn load(path: &Path) -> Result<Self, SaveError> {
        let scene: SceneDef = load(path)?;
        if scene.version != SCENE_DEF_VERSION {
            log::warn!(
                "씬 파일 버전 불일치: 파일={}, 현재={} ({})",
                scene.version,
                SCENE_DEF_VERSION,
                path.display()
            );
        }
        Ok(scene)
    }

    /// 씬 정의를 RON 파일에 저장한다. 항상 현재 버전으로 기록된다.
    pub fn save(&self, path: &Path) -> Result<(), SaveError> {
        let versioned = SceneDef {
            version: SCENE_DEF_VERSION,
            ..self.clone()
        };
        save(path, &versioned)
    }

    /// 씬 정의에 엔티티를 추가하고 빌더 패턴으로 반환한다.
    pub fn with(mut self, def: EntityDef) -> Self {
        self.entities.push(def);
        self
    }
}

// ─── Prefab ───────────────────────────────────────────────────────────────────

/// 파일 하나에 저장된 단일 엔티티 템플릿.
///
/// 동일한 엔티티를 여러 번 스폰하거나 에디터에서 재사용할 때 유용하다.
///
/// # 예
/// ```rust,no_run
/// use engine::prefab::Prefab;
/// use engine::ecs::World;
/// use std::path::Path;
///
/// let mut world = engine::ecs::World::new();
/// let prefab = Prefab::load(Path::new("prefabs/coin.ron")).unwrap();
/// let _e = prefab.spawn(&mut world);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Prefab {
    pub def: EntityDef,
}

impl Prefab {
    /// RON 파일에서 프리팹을 로드한다.
    pub fn load(path: &Path) -> Result<Self, SaveError> {
        load(path)
    }

    /// 프리팹을 RON 파일에 저장한다.
    pub fn save(&self, path: &Path) -> Result<(), SaveError> {
        save(path, self)
    }

    /// 프리팹을 월드에 스폰하고 생성된 엔티티를 반환한다.
    pub fn spawn(&self, world: &mut World) -> Entity {
        spawn_entity_def(world, &self.def)
    }
}

// ─── 자유 함수 ─────────────────────────────────────────────────────────────────

/// `EntityDef` 하나를 월드에 스폰하고 엔티티를 반환한다.
///
/// `def`에 지정된 컴포넌트만 삽입된다.
pub fn spawn_entity_def(world: &mut World, def: &EntityDef) -> Entity {
    let entity = world.spawn();

    if let Some(tag) = &def.tag {
        world.add_component(entity, Tag(tag.clone()));
    }
    if let Some(transform) = &def.transform {
        world.add_component(entity, transform.clone());
    }
    if let Some(sprite) = &def.sprite {
        world.add_component(entity, sprite.clone());
    }

    entity
}

/// `SceneDef`의 모든 엔티티를 월드에 스폰하고 엔티티 목록을 반환한다.
///
/// `EntityDef.parent`가 설정된 경우, 해당 태그의 엔티티를 부모로 연결한다.
/// RON 파일에 `parent` 키가 없는 구버전 씬도 그대로 로드된다 (하위 호환).
pub fn spawn_scene_def(world: &mut World, scene: &SceneDef) -> Vec<Entity> {
    // 1패스: 모든 엔티티 생성 + tag → Entity 맵 구축
    let mut tag_to_entity: HashMap<String, Entity> = HashMap::new();
    let entities: Vec<Entity> = scene
        .entities
        .iter()
        .map(|def| {
            let e = spawn_entity_def(world, def);
            if let Some(tag) = &def.tag {
                tag_to_entity.insert(tag.clone(), e);
            }
            e
        })
        .collect();

    // 2패스: parent 태그가 있는 엔티티에 계층 연결
    for (def, &child) in scene.entities.iter().zip(entities.iter()) {
        if let Some(parent_tag) = &def.parent {
            if let Some(&parent) = tag_to_entity.get(parent_tag) {
                crate::hierarchy::attach(world, child, parent);
            }
        }
    }

    entities
}

/// 엔티티 목록을 위상 정렬하여 루트 → 자식 순으로 반환한다.
///
/// 씬 저장 시 부모가 자식보다 먼저 나와야 `spawn_scene_def()`의 2패스 attach가 동작한다.
pub fn topological_sort_entities(entities: &[Entity], world: &World) -> Vec<Entity> {
    use std::collections::VecDeque;

    // 부모 → 자식 인접 맵
    let mut children_map: HashMap<Entity, Vec<Entity>> = HashMap::new();
    let entity_set: std::collections::HashSet<Entity> = entities.iter().copied().collect();
    let mut roots: Vec<Entity> = Vec::new();

    for &e in entities {
        match world.get::<crate::hierarchy::Parent>(e) {
            Some(p) if entity_set.contains(&p.0) => {
                children_map.entry(p.0).or_default().push(e);
            }
            _ => roots.push(e),
        }
    }

    // BFS: 루트부터 자식 순으로 수집
    let mut result = Vec::with_capacity(entities.len());
    let mut queue: VecDeque<Entity> = roots.into_iter().collect();
    while let Some(e) = queue.pop_front() {
        result.push(e);
        if let Some(kids) = children_map.get(&e) {
            for &kid in kids {
                queue.push_back(kid);
            }
        }
    }
    result
}

// ─── 단위 테스트 ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Sprite, Transform};
    use glam::Vec2;
    use std::fs;

    fn tmp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir()
            .join(format!("engine-prefab-test-{}", std::process::id()))
            .join(name)
    }

    #[test]
    fn entity_def_spawn_inserts_components() {
        let mut world = World::new();

        let def = EntityDef {
            tag: Some("hero".into()),
            transform: Some(Transform::new(
                Vec2::new(10.0, 20.0),
                Vec2::splat(64.0),
                0.5,
            )),
            sprite: Some(Sprite::colored(1.0, 0.0, 0.0)),
            parent: None,
        };

        let entity = spawn_entity_def(&mut world, &def);

        let tag = world.get::<Tag>(entity).expect("Tag should be present");
        assert_eq!(tag.0, "hero");

        let tf = world
            .get::<Transform>(entity)
            .expect("Transform should be present");
        assert_eq!(tf.position, Vec2::new(10.0, 20.0));

        let sp = world
            .get::<Sprite>(entity)
            .expect("Sprite should be present");
        assert_eq!(sp.color[0], 1.0);
    }

    #[test]
    fn empty_entity_def_spawn_no_components() {
        let mut world = World::new();
        let entity = spawn_entity_def(&mut world, &EntityDef::default());
        assert!(world.get::<Tag>(entity).is_none());
        assert!(world.get::<Transform>(entity).is_none());
        assert!(world.get::<Sprite>(entity).is_none());
    }

    #[test]
    fn scene_def_roundtrip() {
        let path = tmp_path("scene1.ron");

        let scene = SceneDef {
            entities: vec![
                EntityDef {
                    tag: Some("ground".into()),
                    transform: Some(Transform::new(
                        Vec2::new(0.0, -200.0),
                        Vec2::new(800.0, 32.0),
                        0.0,
                    )),
                    sprite: Some(Sprite::colored(0.3, 0.6, 0.3)),
                    parent: None,
                },
                EntityDef {
                    tag: Some("player".into()),
                    transform: Some(Transform::default()),
                    sprite: None,
                    parent: None,
                },
            ],
            ..Default::default()
        };

        scene.save(&path).expect("save should succeed");
        let loaded = SceneDef::load(&path).expect("load should succeed");

        assert_eq!(loaded.entities.len(), 2);
        assert_eq!(loaded.entities[0].tag.as_deref(), Some("ground"));
        assert_eq!(loaded.entities[1].tag.as_deref(), Some("player"));
        assert!(loaded.entities[1].sprite.is_none());

        let tf = loaded.entities[0].transform.as_ref().unwrap();
        assert_eq!(tf.position, Vec2::new(0.0, -200.0));

        fs::remove_file(&path).ok();
        fs::remove_dir(path.parent().unwrap()).ok();
    }

    #[test]
    fn prefab_roundtrip_and_spawn() {
        let path = tmp_path("coin.ron");

        let prefab = Prefab {
            def: EntityDef {
                tag: Some("coin".into()),
                transform: Some(Transform::new(
                    Vec2::new(100.0, 50.0),
                    Vec2::splat(32.0),
                    0.0,
                )),
                sprite: Some(Sprite::textured("assets/coin.png")),
                parent: None,
            },
        };

        prefab.save(&path).expect("save prefab");
        let loaded = Prefab::load(&path).expect("load prefab");

        assert_eq!(loaded.def.tag.as_deref(), Some("coin"));
        let sp = loaded.def.sprite.as_ref().unwrap();
        assert_eq!(sp.texture.as_deref(), Some("assets/coin.png"));

        let mut world = World::new();
        let entity = loaded.spawn(&mut world);
        let tag = world.get::<Tag>(entity).unwrap();
        assert_eq!(tag.0, "coin");

        fs::remove_file(&path).ok();
        fs::remove_dir(path.parent().unwrap()).ok();
    }

    #[test]
    fn spawn_scene_def_returns_correct_count() {
        let mut world = World::new();
        let scene = SceneDef {
            entities: vec![
                EntityDef::default(),
                EntityDef::default(),
                EntityDef::default(),
            ],
            ..Default::default()
        };
        let entities = spawn_scene_def(&mut world, &scene);
        assert_eq!(entities.len(), 3);
    }

    #[test]
    fn scene_hierarchy_roundtrip() {
        use crate::hierarchy::Parent;

        let path = tmp_path("hierarchy_scene.ron");

        // parent → child 계층 씬 저장
        let scene = SceneDef {
            entities: vec![
                EntityDef {
                    tag: Some("parent".into()),
                    transform: Some(Transform::default()),
                    sprite: None,
                    parent: None,
                },
                EntityDef {
                    tag: Some("child".into()),
                    transform: Some(Transform::default()),
                    sprite: None,
                    parent: Some("parent".into()),
                },
            ],
            ..Default::default()
        };

        scene.save(&path).expect("save should succeed");
        let loaded = SceneDef::load(&path).expect("load should succeed");

        // RON에 parent 필드가 보존됐는지 확인
        assert_eq!(loaded.entities[1].parent.as_deref(), Some("parent"));

        // 스폰 후 Parent 컴포넌트 확인
        let mut world = World::new();
        let entities = spawn_scene_def(&mut world, &loaded);
        let parent_entity = entities[0];
        let child_entity = entities[1];
        let p = world
            .get::<Parent>(child_entity)
            .expect("child should have Parent component");
        assert_eq!(p.0, parent_entity);

        fs::remove_file(&path).ok();
        fs::remove_dir(path.parent().unwrap()).ok();
    }

    #[test]
    fn topological_sort_roots_before_children() {
        use crate::hierarchy::{attach, Parent};

        let mut world = World::new();
        let parent = world.spawn();
        let child = world.spawn();
        attach(&mut world, child, parent);

        let entities = vec![child, parent]; // 역순으로 제공
        let sorted = topological_sort_entities(&entities, &world);

        // 부모가 먼저 나와야 함
        assert_eq!(sorted[0], parent);
        assert_eq!(sorted[1], child);

        // Parent 컴포넌트가 있는 child의 parent.0이 parent임을 재확인
        let p = world.get::<Parent>(child).unwrap();
        assert_eq!(p.0, parent);
    }
}
