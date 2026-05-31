//! Generic undo/redo history — a reusable building block for grid puzzles,
//! turn-based games, level editors, or anything with discrete reversible state.
//!
//! This is a *snapshot* history: callers hand it a full clone of the state they
//! want to be able to revert to. That trades memory for simplicity — there is no
//! per-action diff to author, and `undo`/`redo` can never desync from the state
//! because they swap whole snapshots. For small game state (a Sokoban board, a
//! cursor position, a few entities) this is the right call; for large state where
//! cloning is expensive, prefer a command/diff log instead.
//!
//! The engine's in-editor undo (`src/app.rs`) is command-based and private to the
//! editor; this module is the public, genre-agnostic counterpart games can use.
//!
//! ```
//! use engine::History;
//!
//! let mut board = vec![0u8; 4];
//! let mut history = History::new();
//!
//! history.record(board.clone()); // snapshot before mutating
//! board[1] = 9;
//!
//! assert!(history.undo(&mut board));
//! assert_eq!(board, vec![0, 0, 0, 0]);
//!
//! assert!(history.redo(&mut board));
//! assert_eq!(board, vec![0, 9, 0, 0]);
//! ```

/// A bounded-or-unbounded undo/redo stack over snapshots of `T`.
///
/// Call [`record`](Self::record) with a clone of the current state *before* you
/// mutate it. [`undo`](Self::undo) then restores the most recent snapshot, and
/// [`redo`](Self::redo) re-applies an undone change. Recording a new snapshot
/// after an undo clears the redo branch, matching the usual editor behaviour.
#[derive(Debug, Clone)]
pub struct History<T> {
    past: Vec<T>,
    future: Vec<T>,
    /// Maximum number of undo snapshots retained. `None` = unbounded.
    capacity: Option<usize>,
}

impl<T> Default for History<T> {
    fn default() -> Self {
        Self {
            past: Vec::new(),
            future: Vec::new(),
            capacity: None,
        }
    }
}

impl<T: Clone> History<T> {
    /// An empty, unbounded history.
    pub fn new() -> Self {
        Self::default()
    }

    /// An empty history that retains at most `capacity` undo snapshots; older
    /// snapshots are dropped once the limit is exceeded. A `capacity` of 0 means
    /// nothing is ever retained, so [`undo`](Self::undo) is always a no-op.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            past: Vec::new(),
            future: Vec::new(),
            capacity: Some(capacity),
        }
    }

    /// Push a snapshot of the pre-mutation state and clear the redo branch.
    ///
    /// Call this immediately *before* applying a change you want to be undoable.
    pub fn record(&mut self, snapshot: T) {
        self.future.clear();
        match self.capacity {
            Some(0) => {}
            Some(cap) => {
                self.past.push(snapshot);
                let overflow = self.past.len().saturating_sub(cap);
                if overflow > 0 {
                    self.past.drain(0..overflow);
                }
            }
            None => self.past.push(snapshot),
        }
    }

    /// Restore the most recent snapshot into `current`, moving the previous value
    /// onto the redo branch. Returns `false` (leaving `current` untouched) when
    /// there is nothing to undo.
    pub fn undo(&mut self, current: &mut T) -> bool {
        match self.past.pop() {
            Some(prev) => {
                let replaced = std::mem::replace(current, prev);
                self.future.push(replaced);
                true
            }
            None => false,
        }
    }

    /// Re-apply the most recently undone change into `current`. Returns `false`
    /// (leaving `current` untouched) when there is nothing to redo.
    pub fn redo(&mut self, current: &mut T) -> bool {
        match self.future.pop() {
            Some(next) => {
                let replaced = std::mem::replace(current, next);
                self.past.push(replaced);
                true
            }
            None => false,
        }
    }

    /// Whether [`undo`](Self::undo) would change state.
    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    /// Whether [`redo`](Self::redo) would change state.
    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    /// Number of undo snapshots currently retained.
    pub fn undo_depth(&self) -> usize {
        self.past.len()
    }

    /// Drop all recorded undo and redo snapshots.
    pub fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_then_redo_round_trips() {
        let mut state = 0i32;
        let mut h = History::new();

        h.record(state); // 0
        state = 1;
        h.record(state); // 1
        state = 2;

        assert_eq!(h.undo_depth(), 2);
        assert!(h.undo(&mut state));
        assert_eq!(state, 1);
        assert!(h.undo(&mut state));
        assert_eq!(state, 0);
        assert!(!h.undo(&mut state)); // exhausted
        assert_eq!(state, 0);

        assert!(h.redo(&mut state));
        assert_eq!(state, 1);
        assert!(h.redo(&mut state));
        assert_eq!(state, 2);
        assert!(!h.redo(&mut state));
    }

    #[test]
    fn recording_clears_redo_branch() {
        let mut state = 0i32;
        let mut h = History::new();

        h.record(state);
        state = 1;
        assert!(h.undo(&mut state));
        assert_eq!(state, 0);
        assert!(h.can_redo());

        // A fresh edit invalidates the redo future.
        h.record(state);
        state = 5;
        assert!(!h.can_redo());
        assert!(h.undo(&mut state));
        assert_eq!(state, 0);
    }

    #[test]
    fn capacity_drops_oldest_snapshots() {
        let mut state = 0i32;
        let mut h = History::with_capacity(2);
        for v in 1..=5 {
            h.record(state);
            state = v;
        }
        // Only the two most recent snapshots (3, 4) survive.
        assert_eq!(h.undo_depth(), 2);
        assert!(h.undo(&mut state));
        assert_eq!(state, 4);
        assert!(h.undo(&mut state));
        assert_eq!(state, 3);
        assert!(!h.undo(&mut state));
    }

    #[test]
    fn zero_capacity_never_retains() {
        let mut state = 0i32;
        let mut h = History::with_capacity(0);
        h.record(state);
        state = 1;
        assert!(!h.can_undo());
        assert!(!h.undo(&mut state));
        assert_eq!(state, 1);
    }
}
