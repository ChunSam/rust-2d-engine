use crate::animation::player::UvRect;
use crate::asset::{Handle, ImageAsset};

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

#[derive(Clone)]
pub struct DrawImage {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: [f32; 4],
    pub z: f32,
    pub texture: Option<String>,
    pub image_handle: Option<Handle<ImageAsset>>,
    pub uv: UvRect,
}

impl DrawImage {
    pub fn textured(x: f32, y: f32, w: f32, h: f32, path: impl Into<String>) -> Self {
        Self {
            x,
            y,
            w,
            h,
            color: [1.0; 4],
            z: 0.0,
            texture: Some(path.into()),
            image_handle: None,
            uv: UvRect::FULL,
        }
    }

    pub fn with_handle(x: f32, y: f32, w: f32, h: f32, handle: Handle<ImageAsset>) -> Self {
        Self {
            x,
            y,
            w,
            h,
            color: [1.0; 4],
            z: 0.0,
            texture: None,
            image_handle: Some(handle),
            uv: UvRect::FULL,
        }
    }

    pub fn textured_with_handle(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        path: impl Into<String>,
        handle: Option<Handle<ImageAsset>>,
    ) -> Self {
        Self {
            x,
            y,
            w,
            h,
            color: [1.0; 4],
            z: 0.0,
            texture: Some(path.into()),
            image_handle: handle,
            uv: UvRect::FULL,
        }
    }

    pub fn colored(x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) -> Self {
        Self {
            x,
            y,
            w,
            h,
            color,
            z: 0.0,
            texture: None,
            image_handle: None,
            uv: UvRect::FULL,
        }
    }

    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    pub fn with_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    pub fn with_uv(mut self, uv: UvRect) -> Self {
        self.uv = uv;
        self
    }

    pub fn texture_key(&self) -> Option<String> {
        self.image_handle
            .as_ref()
            .map(|h| h.path().to_string())
            .or_else(|| self.texture.clone())
    }
}

#[derive(Default)]
pub struct UiImageQueue {
    pub items: Vec<DrawImage>,
}

impl UiImageQueue {
    pub fn push(&mut self, image: DrawImage) {
        self.items.push(image);
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
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

    #[test]
    fn draw_image_uses_path_fallback() {
        let img = DrawImage::textured(0.0, 0.0, 16.0, 16.0, "fallback.png");
        assert_eq!(img.texture_key().as_deref(), Some("fallback.png"));
    }
}
