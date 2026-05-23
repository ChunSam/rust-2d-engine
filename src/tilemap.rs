use std::collections::HashMap;

use glam::Vec2;

use crate::animation::player::UvRect;
use crate::components::{Sprite, Transform};
use crate::ecs::{Entity, System, World};

// ─── 데이터 타입 ──────────────────────────────────────────────────────────────

/// 타일맵용 텍스처 아틀라스 설정
#[derive(Debug, Clone)]
pub struct TilemapAtlas {
    /// 텍스처 파일 경로
    pub texture: String,
    /// 아틀라스 가로 타일 수
    pub columns: u32,
    /// 아틀라스 세로 타일 수
    pub rows: u32,
}

impl TilemapAtlas {
    pub fn new(texture: impl Into<String>, columns: u32, rows: u32) -> Self {
        Self {
            texture: texture.into(),
            columns,
            rows,
        }
    }

    /// 타일 ID(0부터 시작)의 UV 좌표를 반환한다.
    pub fn uv_for(&self, tile_id: u32) -> UvRect {
        let col = tile_id % self.columns;
        let row = tile_id / self.columns;
        UvRect::from_grid(col, row, self.columns, self.rows)
    }
}

/// 타일맵 컴포넌트.
///
/// 엔티티에 붙이면 `TilemapSystem`이 타일 엔티티를 자동으로 스폰한다.
/// `tiles[row][col]` = 0이면 빈 타일, 1 이상이면 `atlas.uv_for(tile_id - 1)` 사용.
#[derive(Debug, Clone)]
pub struct Tilemap {
    pub atlas: TilemapAtlas,
    /// `tiles[row][col]` 형태. 0 = 빈 칸, 1+ = 타일 ID+1.
    pub tiles: Vec<Vec<u32>>,
    /// 타일 한 변 길이 (픽셀)
    pub tile_size: f32,
    /// 타일맵 좌상단 기준점 (세계 좌표)
    pub origin: Vec2,
}

impl Tilemap {
    pub fn new(atlas: TilemapAtlas, tiles: Vec<Vec<u32>>, tile_size: f32, origin: Vec2) -> Self {
        Self {
            atlas,
            tiles,
            tile_size,
            origin,
        }
    }
}

// ─── 시스템 ──────────────────────────────────────────────────────────────────

/// 타일맵 컴포넌트를 읽어 타일 엔티티를 관리하는 시스템.
///
/// 타일맵 엔티티가 처음 발견되면 타일 엔티티를 스폰한다.
/// 타일맵 엔티티가 사라지면 해당 타일 엔티티를 디스폰한다.
pub struct TilemapSystem {
    /// 타일맵 엔티티 → 스폰된 타일 엔티티 목록
    tile_entities: HashMap<Entity, Vec<Entity>>,
}

impl TilemapSystem {
    pub fn new() -> Self {
        Self {
            tile_entities: HashMap::new(),
        }
    }
}

impl Default for TilemapSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for TilemapSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        // 현재 살아있는 타일맵 엔티티 수집
        let tilemap_entities: Vec<Entity> =
            world.query::<Tilemap>().map(|(e, _)| e).collect();

        // 사라진 타일맵 엔티티의 타일들 디스폰
        let removed: Vec<Entity> = self
            .tile_entities
            .keys()
            .filter(|e| !tilemap_entities.contains(e))
            .copied()
            .collect();
        for map_entity in removed {
            if let Some(tiles) = self.tile_entities.remove(&map_entity) {
                for tile in tiles {
                    world.despawn(tile);
                }
            }
        }

        // 새 타일맵 엔티티 스폰
        for map_entity in tilemap_entities {
            if self.tile_entities.contains_key(&map_entity) {
                continue; // 이미 처리됨
            }

            let (atlas, tiles, tile_size, origin) = {
                let tm = world.get::<Tilemap>(map_entity).unwrap();
                (
                    tm.atlas.clone(),
                    tm.tiles.clone(),
                    tm.tile_size,
                    tm.origin,
                )
            };

            let mut spawned = Vec::new();
            for (row_idx, row) in tiles.iter().enumerate() {
                for (col_idx, &tile_id) in row.iter().enumerate() {
                    if tile_id == 0 {
                        continue;
                    }
                    let actual_id = tile_id - 1;
                    let uv = atlas.uv_for(actual_id);
                    let x = origin.x + col_idx as f32 * tile_size + tile_size * 0.5;
                    let y = origin.y + row_idx as f32 * tile_size + tile_size * 0.5;

                    let tile_entity = world.spawn();
                    world.add_component(
                        tile_entity,
                        Transform {
                            position: Vec2::new(x, y),
                            scale: Vec2::splat(tile_size),
                            rotation: 0.0,
                            z: -1.0,
                        },
                    );
                    // AnimationPlayer 없이 UvRect 컴포넌트로 UV를 직접 제어한다.
                    world.add_component(tile_entity, Sprite::textured(&atlas.texture));
                    world.add_component(tile_entity, uv);
                    spawned.push(tile_entity);
                }
            }
            self.tile_entities.insert(map_entity, spawned);
        }
    }
}
