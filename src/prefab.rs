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
//!         },
//!     ],
//! };
//!
//! // 파일 저장 후 로드
//! scene.save(Path::new("levels/level1.ron")).unwrap();
//! let loaded = SceneDef::load(Path::new("levels/level1.ron")).unwrap();
//! let entities = spawn_scene_def(&mut world, &loaded);
//! ```

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
}

// ─── SceneDef ─────────────────────────────────────────────────────────────────

/// 레벨/씬 전체를 기술하는 직렬화 가능 구조체.
///
/// RON 파일 한 장이 하나의 `SceneDef`에 대응한다.
///
/// # RON 예시
/// ```ron
/// SceneDef(
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SceneDef {
    pub entities: Vec<EntityDef>,
}

impl SceneDef {
    /// RON 파일에서 씬 정의를 로드한다.
    pub fn load(path: &Path) -> Result<Self, SaveError> {
        load(path)
    }

    /// 씬 정의를 RON 파일에 저장한다.
    pub fn save(&self, path: &Path) -> Result<(), SaveError> {
        save(path, self)
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
pub fn spawn_scene_def(world: &mut World, scene: &SceneDef) -> Vec<Entity> {
    scene
        .entities
        .iter()
        .map(|def| spawn_entity_def(world, def))
        .collect()
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
                },
                EntityDef {
                    tag: Some("player".into()),
                    transform: Some(Transform::default()),
                    sprite: None,
                },
            ],
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
        };
        let entities = spawn_scene_def(&mut world, &scene);
        assert_eq!(entities.len(), 3);
    }
}
