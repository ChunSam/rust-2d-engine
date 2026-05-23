use glam::Vec2;
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer as GlyphonTextRenderer, Viewport,
};
use wgpu::{
    CommandEncoder, Device, LoadOp, MultisampleState, Operations, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp, TextureFormat, TextureView,
};

use crate::ecs::World;

/// 한 줄 텍스트 그리기 명령. `position`은 좌상단 픽셀 좌표.
#[derive(Debug, Clone)]
pub struct DrawText {
    pub text: String,
    pub position: Vec2,
    /// 폰트 픽셀 크기
    pub size: f32,
    /// RGBA (0~255)
    pub color: [u8; 4],
}

/// 매 프레임 텍스트 그리기 요청을 모으는 큐.
///
/// `World` 리소스로 삽입된다. 게임 시스템이 `push` 로 항목을 추가하면
/// `TextRenderer::render` 가 소비하고 `clear` 한다.
#[derive(Default)]
pub struct TextQueue {
    items: Vec<DrawText>,
}

impl TextQueue {
    /// 텍스트 항목을 큐에 추가한다.
    pub fn push(&mut self, item: DrawText) {
        self.items.push(item);
    }

    /// 모든 항목을 제거한다.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// 항목 이터레이터.
    pub fn iter(&self) -> impl Iterator<Item = &DrawText> {
        self.items.iter()
    }

    /// 큐에 들어 있는 항목 수.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 큐가 비어 있는지 여부.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// glyphon 0.6 기반 텍스트 렌더러.
///
/// ## 소유권 배치
/// - `Cache` 를 먼저 만들고 `TextAtlas` / `Viewport` 에 공유한다.
///   (`TextAtlas::new` 가 `&Cache` 를 필요로 하며, `TextRenderer` 가 `Cache`
///   소유권을 보존해야 한다 — CLAUDE.md 결정 사항.)
/// - `Viewport::update(queue, Resolution{w,h})` 로 매 프레임 GPU 유니폼을 갱신한다.
pub struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    /// `Cache` を先に作り atlas / viewport と共有する (glyphon 0.6 要件).
    /// `TextAtlas` が内部で `Cache` を `clone()` するため、フィールドとして
    /// 保持しなくても動くが、所有権を明示的に残す (CLAUDE.md 決定事項).
    #[allow(dead_code)]
    cache: Cache,
    atlas: TextAtlas,
    viewport: Viewport,
    renderer: GlyphonTextRenderer,
}

impl TextRenderer {
    /// GPU 리소스를 초기화한다.
    ///
    /// `font_data` 가 비어 있지 않으면 해당 TTF/OTF 바이트를 fontdb 에 로드한다.
    /// 비어 있으면 glyphon 의 시스템 폰트 폴백을 사용한다.
    pub fn new(device: &Device, queue: &Queue, format: TextureFormat, font_data: &[u8]) -> Self {
        let mut font_system = FontSystem::new();
        if !font_data.is_empty() {
            font_system.db_mut().load_font_data(font_data.to_vec());
        }

        let swash_cache = SwashCache::new();

        // 2. Cache 먼저, 그 다음 Atlas / Viewport
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);

        // 3. TextRenderer (glyphon 내부 GlyphonTextRenderer)
        let renderer =
            GlyphonTextRenderer::new(&mut atlas, device, MultisampleState::default(), None);

        Self {
            font_system,
            swash_cache,
            cache,
            atlas,
            viewport,
            renderer,
        }
    }

    /// ECS `World` 에서 `TextQueue` 를 꺼내 텍스트를 렌더링한다.
    ///
    /// - 큐가 비어 있으면 렌더 패스를 열지 않고 즉시 반환한다.
    /// - 스프라이트 pass 이후에 `LoadOp::Load` 로 합성한다.
    /// - 렌더 후 큐를 비운다.
    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        world: &mut World,
        w: u32,
        h: u32,
    ) {
        // 큐에서 항목을 꺼낸다. 비어 있으면 조기 반환.
        let items: Vec<DrawText> = match world.resource_mut::<TextQueue>() {
            Some(q) if !q.is_empty() => {
                let taken = q.items.clone();
                q.clear();
                taken
            }
            _ => return,
        };

        // Viewport 갱신 (매 프레임 해상도를 GPU 유니폼에 씀)
        self.viewport.update(
            queue,
            Resolution {
                width: w,
                height: h,
            },
        );

        // 각 DrawText 를 glyphon Buffer 로 변환
        // - `Buffer::set_size` 는 cosmic-text 에서 `(font_system, Option<f32>, Option<f32>)` 를 받는다.
        // - `set_text` 도 `(font_system, text, attrs, shaping)` 형태.
        let buffers: Vec<(Buffer, DrawText)> = items
            .into_iter()
            .map(|d| {
                let metrics = Metrics::new(d.size, d.size * 1.2); // line_height = 1.2× size
                let mut buf = Buffer::new(&mut self.font_system, metrics);
                buf.set_size(
                    &mut self.font_system,
                    Some(w as f32 - d.position.x),
                    Some(h as f32 - d.position.y),
                );
                buf.set_text(
                    &mut self.font_system,
                    &d.text,
                    Attrs::new().family(Family::SansSerif),
                    Shaping::Advanced,
                );
                buf.shape_until_scroll(&mut self.font_system, false);
                (buf, d)
            })
            .collect();

        let text_areas: Vec<TextArea<'_>> = buffers
            .iter()
            .map(|(buf, d)| TextArea {
                buffer: buf,
                left: d.position.x,
                top: d.position.y,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: w as i32,
                    bottom: h as i32,
                },
                default_color: Color::rgba(d.color[0], d.color[1], d.color[2], d.color[3]),
                custom_glyphs: &[],
            })
            .collect();

        // prepare — 글리프 래스터라이즈 + GPU 버퍼 업로드
        let _ = self.renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            text_areas,
            &mut self.swash_cache,
        );

        // 텍스트 렌더 패스 — LoadOp::Load 로 스프라이트 위에 합성
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("text pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let _ = self.renderer.render(&self.atlas, &self.viewport, &mut pass);
        }

        // 다음 프레임을 위해 아틀라스 미사용 글리프 정리
        self.atlas.trim();
    }
}

// ─── 단위 테스트 (GPU 없이 실행 가능한 부분만) ──────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_draw_text(text: &str) -> DrawText {
        DrawText {
            text: text.into(),
            position: Vec2::new(0.0, 0.0),
            size: 24.0,
            color: [255, 255, 255, 255],
        }
    }

    #[test]
    fn text_queue_push_and_clear() {
        let mut q = TextQueue::default();
        assert!(q.is_empty());
        q.push(make_draw_text("hello"));
        assert_eq!(q.len(), 1);
        assert!(!q.is_empty());
        q.clear();
        assert!(q.is_empty());
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn text_queue_iter_preserves_order() {
        let mut q = TextQueue::default();
        q.push(make_draw_text("first"));
        q.push(make_draw_text("second"));
        q.push(make_draw_text("third"));

        let texts: Vec<&str> = q.iter().map(|d| d.text.as_str()).collect();
        assert_eq!(texts, ["first", "second", "third"]);
    }

    #[test]
    fn drawtext_fields_preserved() {
        let d = DrawText {
            text: "안녕".into(),
            position: Vec2::new(10.0, 20.0),
            size: 24.0,
            color: [255, 0, 0, 255],
        };
        assert_eq!(d.text, "안녕");
        assert_eq!(d.position, Vec2::new(10.0, 20.0));
        assert_eq!(d.size, 24.0);
        assert_eq!(d.color, [255, 0, 0, 255]);
    }
}
