//! Top-down maze-escape playable example (candidate B from docs/NEXT_WORK.md).
//!
//! Validates `PathGrid` + `BehaviorTree` + `SpatialGrid` working together for
//! the first time in the engine. Engine-side gaps closed alongside this example:
//! `BehaviorTree`/`Sequence`/`Selector` re-exported from `engine::`,
//! `SpatialGrid` mirrored as a `World` resource, and `PathGrid::from_tilemap`.
//!
//! Controls: WASD / Arrows to move, R to restart, Esc to quit.

use engine::{
    behavior::{Blackboard, BlackboardValue},
    App, BehaviorNode, BehaviorStatus, BehaviorSystem, BehaviorTree, Camera, Collider,
    CollisionGridSystem, CollisionLayer, DrawText, Entity, InputState, KeyCode, PathGrid, Selector,
    Sequence, ShouldQuit, SpatialGrid, Sprite, System, TextQueue, Tilemap, TilemapAtlas, Transform,
    WindowConfig, World,
};
use glam::{IVec2, Vec2};

// ─── Layout / constants ──────────────────────────────────────────────────────

const TILE: f32 = 40.0;

// 17 cols × 11 rows. '#' wall, '.' floor, 'P' player spawn, 'E' enemy spawn,
// 'G' goal. Player and goal positions verified to be reachable by hand.
const MAZE: &[&str] = &[
    "#################",
    "#P.....#.......G#",
    "#.####.#.#####..#",
    "#..#...#.#...#..#",
    "##.#.###.#.#.#..#",
    "#..#.....#.#....#",
    "#.####.###.###.##",
    "#......#E.......#",
    "##.####.#.####..#",
    "#...............#",
    "#################",
];

const MAZE_COLS: f32 = 17.0;
const MAZE_ROWS: f32 = 11.0;
const WINDOW_W: u32 = (MAZE_COLS as u32) * TILE as u32;
const WINDOW_H: u32 = (MAZE_ROWS as u32) * TILE as u32;

const PLAYER_HALF: f32 = TILE * 0.35;
const ENEMY_HALF: f32 = TILE * 0.35;
const PLAYER_SPEED: f32 = 180.0;
const ENEMY_SPEED: f32 = 110.0;
const GOAL_RADIUS: f32 = TILE * 0.4;
const ENEMY_CONTACT: f32 = TILE * 0.55;

const WALL_LAYER: u32 = 1 << 0;

// Tile ids used for both rendering (Tilemap is not spawned — we render with
// plain Sprite entities) and `PathGrid::from_tilemap`.
const TILE_FLOOR: u32 = 1;
const TILE_WALL: u32 = 2;

// ─── Marker components / session ─────────────────────────────────────────────

// Marker components — attached to entities so the design is greppable even
// though queries route through `MazeSession` rather than these tags.
#[allow(dead_code)]
struct Player;
#[allow(dead_code)]
struct Enemy;
#[allow(dead_code)]
struct Goal;
#[allow(dead_code)]
struct Wall;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Playing,
    Won,
    Lost,
}

struct MazeSession {
    player: Entity,
    player_spawn: Vec2,
    goal: Entity,
    enemies: Vec<(Entity, Vec2)>, // (entity, spawn position)
    status: Status,
}

// ─── Layout parsing ──────────────────────────────────────────────────────────

struct ParsedMaze {
    tiles: Vec<Vec<u32>>,
    player: Vec2,
    goal: Vec2,
    enemies: Vec<Vec2>,
}

fn parse_maze() -> ParsedMaze {
    let mut tiles: Vec<Vec<u32>> = Vec::with_capacity(MAZE.len());
    let mut player = Vec2::ZERO;
    let mut goal = Vec2::ZERO;
    let mut enemies = Vec::new();

    for (row, line) in MAZE.iter().enumerate() {
        let mut row_tiles = Vec::with_capacity(line.len());
        for (col, ch) in line.chars().enumerate() {
            let tile = match ch {
                '#' => TILE_WALL,
                _ => TILE_FLOOR,
            };
            row_tiles.push(tile);

            let center = tile_center(col as i32, row as i32);
            match ch {
                'P' => player = center,
                'G' => goal = center,
                'E' => enemies.push(center),
                _ => {}
            }
        }
        tiles.push(row_tiles);
    }

    ParsedMaze {
        tiles,
        player,
        goal,
        enemies,
    }
}

fn tile_center(col: i32, row: i32) -> Vec2 {
    Vec2::new(
        col as f32 * TILE + TILE * 0.5,
        row as f32 * TILE + TILE * 0.5,
    )
}

fn world_to_tile(pos: Vec2) -> IVec2 {
    IVec2::new((pos.x / TILE) as i32, (pos.y / TILE) as i32)
}

// ─── BehaviorTree leaves ─────────────────────────────────────────────────────

fn player_entity(world: &World) -> Option<Entity> {
    world.resource::<MazeSession>().map(|s| s.player)
}

fn player_position(world: &World) -> Option<Vec2> {
    let player = player_entity(world)?;
    world.get::<Transform>(player).map(|t| t.position)
}

/// Returns Success if every tile on the straight Bresenham line between the
/// enemy and the player is walkable, Failure otherwise.
struct HasLineOfSight;
impl BehaviorNode for HasLineOfSight {
    fn tick(&mut self, world: &mut World, entity: Entity, _dt: f32) -> BehaviorStatus {
        let Some(player_pos) = player_position(world) else {
            return BehaviorStatus::Failure;
        };
        let Some(enemy_pos) = world.get::<Transform>(entity).map(|t| t.position) else {
            return BehaviorStatus::Failure;
        };
        let Some(grid) = world.resource::<PathGrid>() else {
            return BehaviorStatus::Failure;
        };

        let from = world_to_tile(enemy_pos);
        let to = world_to_tile(player_pos);
        if line_clear(grid, from, to) {
            BehaviorStatus::Success
        } else {
            BehaviorStatus::Failure
        }
    }
}

/// Straight-line chase. Single-tick Success so the parent Selector re-evaluates
/// LoS next frame.
struct MoveTowardPlayer;
impl BehaviorNode for MoveTowardPlayer {
    fn tick(&mut self, world: &mut World, entity: Entity, dt: f32) -> BehaviorStatus {
        let Some(player_pos) = player_position(world) else {
            return BehaviorStatus::Failure;
        };
        move_enemy_toward(world, entity, player_pos, dt);
        BehaviorStatus::Success
    }
}

/// Run A*, then write the next tile's world center into the Blackboard.
struct ComputePathToPlayer;
impl BehaviorNode for ComputePathToPlayer {
    fn tick(&mut self, world: &mut World, entity: Entity, _dt: f32) -> BehaviorStatus {
        let Some(player_pos) = player_position(world) else {
            return BehaviorStatus::Failure;
        };
        let Some(enemy_pos) = world.get::<Transform>(entity).map(|t| t.position) else {
            return BehaviorStatus::Failure;
        };
        let Some(grid) = world.resource::<PathGrid>() else {
            return BehaviorStatus::Failure;
        };

        let start = world_to_tile(enemy_pos);
        let goal = world_to_tile(player_pos);
        let Some(path) = engine::find_path(grid, start, goal) else {
            return BehaviorStatus::Failure;
        };
        let Some(next) = path.into_iter().next() else {
            // Already on the player's tile — fall through and let direct chase
            // close the remaining sub-tile distance.
            return BehaviorStatus::Success;
        };
        let next_world = tile_center(next.x, next.y);

        if let Some(bb) = world.get_mut::<Blackboard>(entity) {
            bb.set_vec2("path_target", next_world);
        }
        BehaviorStatus::Success
    }
}

/// Move one step toward the cached blackboard target.
struct FollowPathStep;
impl BehaviorNode for FollowPathStep {
    fn tick(&mut self, world: &mut World, entity: Entity, dt: f32) -> BehaviorStatus {
        let target = match world.get::<Blackboard>(entity).and_then(|bb| {
            bb.entries().find_map(|(k, v)| {
                if k == "path_target" {
                    if let BlackboardValue::Vec2(p) = v {
                        Some(*p)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        }) {
            Some(p) => p,
            None => return BehaviorStatus::Failure,
        };

        move_enemy_toward(world, entity, target, dt);
        BehaviorStatus::Success
    }
}

fn move_enemy_toward(world: &mut World, entity: Entity, target: Vec2, dt: f32) {
    let Some(current) = world.get::<Transform>(entity).map(|t| t.position) else {
        return;
    };
    let delta = target - current;
    let dist = delta.length();
    if dist < 1.0 {
        return;
    }
    let step = (delta / dist) * ENEMY_SPEED * dt;
    let proposed = current + step;
    let resolved = resolve_walls(world, current, proposed, ENEMY_HALF);
    if let Some(t) = world.get_mut::<Transform>(entity) {
        t.position = resolved;
    }
}

fn line_clear(grid: &PathGrid, from: IVec2, to: IVec2) -> bool {
    // Bresenham, return false on the first blocked tile encountered.
    let mut x0 = from.x;
    let mut y0 = from.y;
    let x1 = to.x;
    let y1 = to.y;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if !grid.is_walkable(x0, y0) {
            return false;
        }
        if x0 == x1 && y0 == y1 {
            return true;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

// ─── Wall collision via SpatialGrid resource ─────────────────────────────────

/// Reject motion that would push the AABB into a wall — checked per axis so
/// sliding along walls feels natural.
fn resolve_walls(world: &World, current: Vec2, proposed: Vec2, half: f32) -> Vec2 {
    let Some(grid) = world.resource::<SpatialGrid>() else {
        return proposed;
    };
    let mask = CollisionLayer(WALL_LAYER);

    let mut resolved = current;

    let candidate_x = Vec2::new(proposed.x, current.y);
    if !overlaps_wall(grid, candidate_x, half, mask) {
        resolved.x = proposed.x;
    }

    let candidate_y = Vec2::new(resolved.x, proposed.y);
    if !overlaps_wall(grid, candidate_y, half, mask) {
        resolved.y = proposed.y;
    }

    resolved
}

fn overlaps_wall(grid: &SpatialGrid, center: Vec2, half: f32, mask: CollisionLayer) -> bool {
    let min = center - Vec2::splat(half);
    let max = center + Vec2::splat(half);
    !grid.query_aabb(min, max, mask).is_empty()
}

// ─── Systems ─────────────────────────────────────────────────────────────────

struct PlayerInputSystem;
impl System for PlayerInputSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        let (axis, restart, quit) = world
            .resource::<InputState>()
            .map(|input| {
                let mut x = 0.0;
                let mut y = 0.0;
                if input.is_pressed(KeyCode::KeyA) || input.is_pressed(KeyCode::ArrowLeft) {
                    x -= 1.0;
                }
                if input.is_pressed(KeyCode::KeyD) || input.is_pressed(KeyCode::ArrowRight) {
                    x += 1.0;
                }
                if input.is_pressed(KeyCode::KeyW) || input.is_pressed(KeyCode::ArrowUp) {
                    y -= 1.0;
                }
                if input.is_pressed(KeyCode::KeyS) || input.is_pressed(KeyCode::ArrowDown) {
                    y += 1.0;
                }
                (
                    Vec2::new(x, y),
                    input.just_pressed(KeyCode::KeyR),
                    input.just_pressed(KeyCode::Escape),
                )
            })
            .unwrap_or((Vec2::ZERO, false, false));

        if quit {
            if let Some(q) = world.resource_mut::<ShouldQuit>() {
                q.0 = true;
            }
        }
        if restart {
            reset(world);
            return;
        }

        let Some((player, status)) = world
            .resource::<MazeSession>()
            .map(|s| (s.player, s.status))
        else {
            return;
        };
        if status != Status::Playing {
            return;
        }

        let direction = if axis.length_squared() > 0.0 {
            axis.normalize()
        } else {
            Vec2::ZERO
        };
        let current = match world.get::<Transform>(player) {
            Some(t) => t.position,
            None => return,
        };
        let proposed = current + direction * PLAYER_SPEED * dt;
        let resolved = resolve_walls(world, current, proposed, PLAYER_HALF);
        if let Some(t) = world.get_mut::<Transform>(player) {
            t.position = resolved;
        }
    }

    fn name(&self) -> &'static str {
        "PlayerInputSystem"
    }
}

struct WinLoseSystem;
impl System for WinLoseSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let Some((player, goal, enemies, status)) = world.resource::<MazeSession>().map(|s| {
            (
                s.player,
                s.goal,
                s.enemies.iter().map(|(e, _)| *e).collect::<Vec<_>>(),
                s.status,
            )
        }) else {
            return;
        };
        if status != Status::Playing {
            return;
        }

        let Some(player_pos) = world.get::<Transform>(player).map(|t| t.position) else {
            return;
        };
        let Some(goal_pos) = world.get::<Transform>(goal).map(|t| t.position) else {
            return;
        };

        if (player_pos - goal_pos).length() <= GOAL_RADIUS + PLAYER_HALF {
            if let Some(s) = world.resource_mut::<MazeSession>() {
                s.status = Status::Won;
            }
            return;
        }

        for enemy in enemies {
            if let Some(enemy_pos) = world.get::<Transform>(enemy).map(|t| t.position) {
                if (player_pos - enemy_pos).length() <= ENEMY_CONTACT {
                    if let Some(s) = world.resource_mut::<MazeSession>() {
                        s.status = Status::Lost;
                    }
                    return;
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "WinLoseSystem"
    }
}

struct HudSystem;
impl System for HudSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let Some(status) = world.resource::<MazeSession>().map(|s| s.status) else {
            return;
        };
        if let Some(tq) = world.resource_mut::<TextQueue>() {
            tq.push(DrawText::new(
                "WASD/Arrows: move   R: restart   Esc: quit",
                Vec2::new(8.0, 8.0),
                18.0,
                [235, 245, 255, 230],
            ));
            match status {
                Status::Playing => {}
                Status::Won => tq.push(DrawText::new(
                    "Escaped! Press R to play again.",
                    Vec2::new(WINDOW_W as f32 * 0.5 - 170.0, WINDOW_H as f32 * 0.5 - 20.0),
                    26.0,
                    [120, 255, 170, 255],
                )),
                Status::Lost => tq.push(DrawText::new(
                    "Caught. Press R to retry.",
                    Vec2::new(WINDOW_W as f32 * 0.5 - 140.0, WINDOW_H as f32 * 0.5 - 20.0),
                    26.0,
                    [255, 140, 140, 255],
                )),
            }
        }
    }

    fn name(&self) -> &'static str {
        "HudSystem"
    }
}

// ─── Reset ───────────────────────────────────────────────────────────────────

fn reset(world: &mut World) {
    let Some((player, player_spawn, enemy_specs)) = world
        .resource::<MazeSession>()
        .map(|s| (s.player, s.player_spawn, s.enemies.clone()))
    else {
        return;
    };
    if let Some(t) = world.get_mut::<Transform>(player) {
        t.position = player_spawn;
    }
    for (entity, spawn) in &enemy_specs {
        if let Some(t) = world.get_mut::<Transform>(*entity) {
            t.position = *spawn;
        }
        if let Some(bb) = world.get_mut::<Blackboard>(*entity) {
            // Drop the cached path step so the BT recomputes immediately.
            bb.set_vec2("path_target", *spawn);
        }
        if let Some(tree) = world.get_mut::<BehaviorTree>(*entity) {
            tree.reset();
        }
    }
    if let Some(s) = world.resource_mut::<MazeSession>() {
        s.status = Status::Playing;
    }
}

// ─── BT builder ──────────────────────────────────────────────────────────────

fn enemy_behavior_tree() -> BehaviorTree {
    BehaviorTree::new(Box::new(Selector::new(vec![
        Box::new(Sequence::new(vec![
            Box::new(HasLineOfSight),
            Box::new(MoveTowardPlayer),
        ])),
        Box::new(Sequence::new(vec![
            Box::new(ComputePathToPlayer),
            Box::new(FollowPathStep),
        ])),
    ])))
}

// ─── Entry point ─────────────────────────────────────────────────────────────

fn main() {
    let mut app = App::new();
    app.world.insert_resource(WindowConfig {
        title: "skeleton-engine maze escape".to_string(),
        width: WINDOW_W,
        height: WINDOW_H,
        clear_color: [0.06, 0.08, 0.12, 1.0],
    });

    let parsed = parse_maze();

    // Build the PathGrid via the new helper — this is the validation point for
    // `PathGrid::from_tilemap`.
    let tilemap = Tilemap::new(
        TilemapAtlas::new("unused.png", 1, 1),
        parsed.tiles.clone(),
        TILE,
        Vec2::ZERO,
    );
    let path_grid = PathGrid::from_tilemap(&tilemap, |id| id == TILE_WALL);
    app.world.insert_resource(path_grid);

    // Spawn floor + wall sprites manually (no TilemapSystem) so we can attach
    // colliders only to wall entities.
    for (row, row_tiles) in parsed.tiles.iter().enumerate() {
        for (col, &tile) in row_tiles.iter().enumerate() {
            let center = tile_center(col as i32, row as i32);
            let entity = app.world.spawn();
            match tile {
                TILE_WALL => {
                    app.world.add_component(
                        entity,
                        Transform {
                            position: center,
                            scale: Vec2::splat(TILE),
                            rotation: 0.0,
                            z: 0.0,
                        },
                    );
                    app.world
                        .add_component(entity, Sprite::colored(0.25, 0.28, 0.38));
                    app.world.add_component(
                        entity,
                        Collider::Aabb {
                            half_extents: Vec2::splat(TILE * 0.5),
                        },
                    );
                    app.world.add_component(entity, CollisionLayer(WALL_LAYER));
                    app.world.add_component(entity, Wall);
                }
                _ => {
                    app.world.add_component(
                        entity,
                        Transform {
                            position: center,
                            scale: Vec2::splat(TILE),
                            rotation: 0.0,
                            z: -1.0,
                        },
                    );
                    app.world
                        .add_component(entity, Sprite::colored(0.14, 0.16, 0.22));
                }
            }
        }
    }

    // Goal marker
    let goal = app.world.spawn();
    app.world.add_component(
        goal,
        Transform {
            position: parsed.goal,
            scale: Vec2::splat(TILE * 0.65),
            rotation: 0.0,
            z: 0.5,
        },
    );
    app.world
        .add_component(goal, Sprite::colored(0.45, 0.95, 0.55));
    app.world.add_component(goal, Goal);

    // Player
    let player = app.world.spawn();
    app.world.add_component(
        player,
        Transform {
            position: parsed.player,
            scale: Vec2::splat(TILE * 0.7),
            rotation: 0.0,
            z: 1.0,
        },
    );
    app.world
        .add_component(player, Sprite::colored(0.95, 0.85, 0.35));
    app.world.add_component(player, Player);

    // Enemies
    let mut enemies = Vec::new();
    for spawn in &parsed.enemies {
        let entity = app.world.spawn();
        app.world.add_component(
            entity,
            Transform {
                position: *spawn,
                scale: Vec2::splat(TILE * 0.7),
                rotation: 0.0,
                z: 1.0,
            },
        );
        app.world
            .add_component(entity, Sprite::colored(0.95, 0.4, 0.4));
        app.world.add_component(entity, Enemy);
        app.world.add_component(entity, Blackboard::new());
        app.world.add_component(entity, enemy_behavior_tree());
        enemies.push((entity, *spawn));
    }

    app.world.insert_resource(MazeSession {
        player,
        player_spawn: parsed.player,
        goal,
        enemies,
        status: Status::Playing,
    });

    // Fixed camera showing the whole maze.
    app.world.insert_resource(Camera::new(Vec2::ZERO, 1.0));

    // System order: rebuild spatial grid → player moves (queries grid) →
    // BT ticks (queries PathGrid + player pos) → win/lose → HUD.
    app.add_system(CollisionGridSystem::new(TILE * 2.0));
    app.add_system(PlayerInputSystem);
    app.add_system(BehaviorSystem);
    app.add_system(WinLoseSystem);
    app.add_system(HudSystem);

    app.run();
}
