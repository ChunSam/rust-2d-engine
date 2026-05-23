#[derive(Clone, Copy)]
pub struct DrawRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: [f32; 4],
    pub z: f32,
}

impl DrawRect {
    pub fn new(x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) -> Self {
        Self {
            x,
            y,
            w,
            h,
            color,
            z: 0.0,
        }
    }

    pub fn with_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }
}

#[derive(Default)]
pub struct UiQueue {
    pub items: Vec<DrawRect>,
}

impl UiQueue {
    pub fn push(&mut self, rect: DrawRect) {
        self.items.push(rect);
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_queue_push_and_clear() {
        let mut q = UiQueue::default();
        assert!(q.is_empty());
        q.push(DrawRect::new(0.0, 0.0, 100.0, 50.0, [1.0, 0.0, 0.0, 1.0]));
        assert!(!q.is_empty());
        assert_eq!(q.items.len(), 1);
        q.clear();
        assert!(q.is_empty());
    }

    #[test]
    fn draw_rect_with_z() {
        let r = DrawRect::new(10.0, 20.0, 80.0, 40.0, [1.0; 4]).with_z(0.5);
        assert_eq!(r.x, 10.0);
        assert_eq!(r.z, 0.5);
    }
}
