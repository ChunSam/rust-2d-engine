//! Sokoban playable example (candidate C from docs/NEXT_WORK.md).
//!
//! A turn-based grid puzzle: push every box onto a goal. Validates discrete grid
//! movement, multi-level progression, undo/redo, and progress save/load working
//! together for the first time in the engine. Engine-side gap closed alongside
//! this example: a reusable, genre-agnostic `History<T>` undo utility
//! (`src/history.rs`) — previously the only undo lived inside the editor and was
//! private. Grid/board logic stays example-local (genre-specific), and progress
//! persistence reuses the existing `save` module unchanged.
//!
//! Rendering uses immediate-mode filled rects (`DebugDrawQueue`, drained every
//! frame) rather than persistent ECS sprite entities, because the board is fully
//! reconstructed from snapshot state each move and levels swap in place — no
//! entity churn, trivial level switching.
//!
//! Controls: WASD / Arrows to push, U undo, Y redo, R restart level,
//! N next / P previous level, Esc quit. Solve a level to auto-unlock the next;
//! progress (furthest level reached) is saved and resumed on the next launch.

use engine::{
    save, App, Camera, DebugDrawQueue, DebugRect, DrawText, History, InputState, KeyCode,
    ShouldQuit, System, TextQueue, WindowConfig, World,
};
use glam::{IVec2, Vec2};
use serde::{Deserialize, Serialize};

// ─── Layout / constants ──────────────────────────────────────────────────────

const TILE: f32 = 56.0;
// Window is sized to the largest level; smaller levels are centered.
const GRID_COLS: i32 = 9;
const GRID_ROWS: i32 = 6;
const WINDOW_W: u32 = GRID_COLS as u32 * TILE as u32;
const WINDOW_H: u32 = GRID_ROWS as u32 * TILE as u32 + 56; // + HUD strip

const APP_NAME: &str = "skeleton_sokoban";
const SAVE_FILE: &str = "progress.ron";

// Sokoban notation: '#' wall, ' ' floor, '@' player, '$' box, '.' goal,
// '*' box-on-goal, '+' player-on-goal. Every level here is hand-verified solvable.
const LEVELS: &[&[&str]] = &[
    // L1 — one box, push up.
    &[
        "#######", //
        "###.###", //
        "#  $  #", //
        "#  @  #", //
        "#######", //
    ],
    // L2 — two independent boxes, one pushed left, one pushed right.
    &[
        "#########", //
        "#.$@ $ .#", //
        "#########", //
    ],
    // L3 — two boxes pushed straight up into the top row.
    &[
        "#######", //
        "#.   .#", //
        "#     #", //
        "#$   $#", //
        "#  @  #", //
        "#######", //
    ],
];

// ─── Persistent progress ─────────────────────────────────────────────────────

/// Saved progress: the furthest level the player has unlocked (0-based).
/// Reuses the engine `save` module as-is — no new persistence API needed.
#[derive(Serialize, Deserialize, Default)]
struct Progress {
    max_level: usize,
}

// ─── Static level + mutable snapshot state ───────────────────────────────────

/// Immutable level geometry — walls and goals never move, so they live outside
/// the undo snapshot.
struct Level {
    walls: Vec<IVec2>,
    goals: Vec<IVec2>,
    cols: i32,
    rows: i32,
    spawn: LevelState,
}

impl Level {
    fn is_wall(&self, c: IVec2) -> bool {
        self.walls.contains(&c)
    }
    fn is_goal(&self, c: IVec2) -> bool {
        self.goals.contains(&c)
    }
}

/// The only mutable, undoable part of a level: where the player and boxes are.
/// Cloned wholesale into `History` before each move — see `src/history.rs`.
#[derive(Clone, PartialEq, Eq)]
struct LevelState {
    player: IVec2,
    boxes: Vec<IVec2>,
}

impl LevelState {
    fn box_index_at(&self, c: IVec2) -> Option<usize> {
        self.boxes.iter().position(|&b| b == c)
    }
}

fn parse_level(rows: &[&str]) -> Level {
    let mut walls = Vec::new();
    let mut goals = Vec::new();
    let mut boxes = Vec::new();
    let mut player = IVec2::ZERO;
    let cols = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0) as i32;

    for (y, line) in rows.iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            let c = IVec2::new(x as i32, y as i32);
            match ch {
                '#' => walls.push(c),
                '.' => goals.push(c),
                '$' => boxes.push(c),
                '*' => {
                    goals.push(c);
                    boxes.push(c);
                }
                '@' => player = c,
                '+' => {
                    goals.push(c);
                    player = c;
                }
                _ => {}
            }
        }
    }

    Level {
        walls,
        goals,
        cols,
        rows: rows.len() as i32,
        spawn: LevelState { player, boxes },
    }
}

// ─── Session ─────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Playing,
    Solved,
    AllClear,
}

struct Session {
    levels: Vec<Level>,
    index: usize,
    state: LevelState,
    history: History<LevelState>,
    status: Status,
    moves: u32,
    max_level: usize,
}

impl Session {
    fn level(&self) -> &Level {
        &self.levels[self.index]
    }

    /// Attempt a step in `dir`, pushing a box if one is in the way. Records an
    /// undo snapshot only when the move actually changes the board.
    fn try_move(&mut self, dir: IVec2) {
        if self.status != Status::Playing {
            return;
        }
        let level = &self.levels[self.index];
        let target = self.state.player + dir;
        if level.is_wall(target) {
            return;
        }

        let before = self.state.clone();
        if let Some(bi) = self.state.box_index_at(target) {
            let beyond = target + dir;
            if level.is_wall(beyond) || self.state.box_index_at(beyond).is_some() {
                return; // box is blocked — no move
            }
            self.state.boxes[bi] = beyond;
        }
        self.state.player = target;

        self.history.record(before);
        self.moves += 1;
        self.check_solved();
    }

    fn check_solved(&mut self) {
        let level = &self.levels[self.index];
        let solved = level.goals.iter().all(|g| self.state.boxes.contains(g));
        if solved {
            self.status = if self.index + 1 >= self.levels.len() {
                Status::AllClear
            } else {
                Status::Solved
            };
        }
    }

    fn load_level(&mut self, index: usize) {
        self.index = index.min(self.levels.len() - 1);
        self.state = self.levels[self.index].spawn.clone();
        self.history.clear();
        self.moves = 0;
        self.status = Status::Playing;
    }

    fn restart(&mut self) {
        let i = self.index;
        self.load_level(i);
    }

    /// Advance to the next level (called after a solve), unlocking it for resume.
    fn next_level(&mut self) {
        if self.index + 1 < self.levels.len() {
            let next = self.index + 1;
            self.max_level = self.max_level.max(next);
            self.load_level(next);
            save_progress(self.max_level);
        }
    }

    fn prev_level(&mut self) {
        if self.index > 0 {
            let prev = self.index - 1;
            self.load_level(prev);
        }
    }
}

fn save_progress(max_level: usize) {
    // Best-effort: on wasm or a read-only home dir this simply no-ops.
    let path = save::save_path(APP_NAME, SAVE_FILE);
    let _ = save::save(&path, &Progress { max_level });
}

fn load_progress() -> usize {
    let path = save::save_path(APP_NAME, SAVE_FILE);
    save::load_or_default::<Progress>(&path)
        .map(|p| p.max_level)
        .unwrap_or(0)
}

// ─── Coordinate helpers ──────────────────────────────────────────────────────

/// Top-left pixel of a cell, centering the current level inside the window.
fn cell_origin(level: &Level, c: IVec2) -> Vec2 {
    let off_x = (WINDOW_W as f32 - level.cols as f32 * TILE) * 0.5;
    let off_y = (GRID_ROWS as f32 * TILE - level.rows as f32 * TILE) * 0.5 + 48.0;
    Vec2::new(off_x + c.x as f32 * TILE, off_y + c.y as f32 * TILE)
}

// ─── Systems ─────────────────────────────────────────────────────────────────

struct InputSystem;
impl System for InputSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let Some(input) = world.resource::<InputState>() else {
            return;
        };
        let p = |k: KeyCode| input.just_pressed(k);
        let left = p(KeyCode::KeyA) || p(KeyCode::ArrowLeft);
        let right = p(KeyCode::KeyD) || p(KeyCode::ArrowRight);
        let up = p(KeyCode::KeyW) || p(KeyCode::ArrowUp);
        let down = p(KeyCode::KeyS) || p(KeyCode::ArrowDown);
        let undo = p(KeyCode::KeyU);
        let redo = p(KeyCode::KeyY);
        let restart = p(KeyCode::KeyR);
        let next = p(KeyCode::KeyN) || p(KeyCode::Enter) || p(KeyCode::Space);
        let prev = p(KeyCode::KeyP);
        let quit = p(KeyCode::Escape);

        if quit {
            if let Some(q) = world.resource_mut::<ShouldQuit>() {
                q.0 = true;
            }
            return;
        }

        let Some(s) = world.resource_mut::<Session>() else {
            return;
        };

        // Level navigation works regardless of status; movement only while Playing.
        if restart {
            s.restart();
            return;
        }
        if next && s.status == Status::Solved {
            s.next_level();
            return;
        }
        if next {
            // Manual skip is only allowed within already-unlocked levels.
            if s.index < s.max_level {
                let i = s.index + 1;
                s.load_level(i);
            }
            return;
        }
        if prev {
            s.prev_level();
            return;
        }
        if undo {
            let mut state = s.state.clone();
            if s.history.undo(&mut state) {
                s.state = state;
                s.status = Status::Playing;
                s.moves = s.moves.saturating_sub(1);
            }
            return;
        }
        if redo {
            let mut state = s.state.clone();
            if s.history.redo(&mut state) {
                s.state = state;
                s.moves += 1;
                s.check_solved();
            }
            return;
        }

        let dir = if left {
            IVec2::new(-1, 0)
        } else if right {
            IVec2::new(1, 0)
        } else if up {
            IVec2::new(0, -1)
        } else if down {
            IVec2::new(0, 1)
        } else {
            return;
        };
        s.try_move(dir);
    }

    fn name(&self) -> &'static str {
        "InputSystem"
    }
}

struct RenderSystem;
impl System for RenderSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        // Snapshot what we need, then borrow the draw queues.
        let Some(s) = world.resource::<Session>() else {
            return;
        };
        let level = s.level();

        let mut rects: Vec<DebugRect> = Vec::new();
        let push = |rects: &mut Vec<DebugRect>, c: IVec2, inset: f32, color: [f32; 4], z: f32| {
            let origin = cell_origin(level, c) + Vec2::splat(inset);
            let size = TILE - inset * 2.0;
            rects.push(DebugRect {
                min: origin,
                max: origin + Vec2::splat(size),
                color,
                z,
            });
        };

        // Floor under the whole level footprint.
        for y in 0..level.rows {
            for x in 0..level.cols {
                let c = IVec2::new(x, y);
                if !level.is_wall(c) {
                    push(&mut rects, c, 0.0, [0.13, 0.15, 0.20, 1.0], 0.0);
                }
            }
        }
        // Walls.
        for &w in &level.walls {
            push(&mut rects, w, 0.0, [0.30, 0.33, 0.42, 1.0], 0.1);
        }
        // Goals.
        for &g in &level.goals {
            push(&mut rects, g, TILE * 0.32, [0.95, 0.78, 0.30, 0.85], 0.2);
        }
        // Boxes — green when seated on a goal.
        for &b in &s.state.boxes {
            let color = if level.is_goal(b) {
                [0.40, 0.85, 0.45, 1.0]
            } else {
                [0.75, 0.52, 0.30, 1.0]
            };
            push(&mut rects, b, TILE * 0.12, color, 0.3);
        }
        // Player.
        push(
            &mut rects,
            s.state.player,
            TILE * 0.18,
            [0.40, 0.70, 0.95, 1.0],
            0.4,
        );

        let (status, index, moves, level_count) = (s.status, s.index, s.moves, s.levels.len());

        if let Some(q) = world.resource_mut::<DebugDrawQueue>() {
            q.items.extend(rects);
        }

        if let Some(tq) = world.resource_mut::<TextQueue>() {
            tq.push(DrawText::new(
                format!(
                    "Level {}/{}   Moves {}   WASD push  U undo  Y redo  R restart  Esc quit",
                    index + 1,
                    level_count,
                    moves
                ),
                Vec2::new(10.0, 12.0),
                17.0,
                [225, 235, 250, 230],
            ));
            match status {
                Status::Playing => {}
                Status::Solved => tq.push(DrawText::new(
                    "Solved! Press N / Enter for the next level.",
                    Vec2::new(WINDOW_W as f32 * 0.5 - 200.0, WINDOW_H as f32 - 30.0),
                    20.0,
                    [120, 255, 170, 255],
                )),
                Status::AllClear => tq.push(DrawText::new(
                    "All levels cleared! Press R to replay.",
                    Vec2::new(WINDOW_W as f32 * 0.5 - 185.0, WINDOW_H as f32 - 30.0),
                    20.0,
                    [255, 220, 130, 255],
                )),
            }
        }
    }

    fn name(&self) -> &'static str {
        "RenderSystem"
    }
}

// ─── Entry point ─────────────────────────────────────────────────────────────

fn main() {
    let mut app = App::new();
    app.world.insert_resource(WindowConfig {
        title: "skeleton-engine sokoban".to_string(),
        width: WINDOW_W,
        height: WINDOW_H,
        clear_color: [0.05, 0.06, 0.09, 1.0],
    });

    let levels: Vec<Level> = LEVELS.iter().map(|rows| parse_level(rows)).collect();

    // Resume at the furthest unlocked level from a previous session.
    let max_level = load_progress().min(levels.len() - 1);
    let start_state = levels[max_level].spawn.clone();

    app.world.insert_resource(Session {
        levels,
        index: max_level,
        state: start_state,
        history: History::new(),
        status: Status::Playing,
        moves: 0,
        max_level,
    });

    app.world.insert_resource(Camera::new(Vec2::ZERO, 1.0));

    app.add_system(InputSystem);
    app.add_system(RenderSystem);

    app.run();
}
