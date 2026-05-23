use glam::Vec2;

use crate::ecs::{Entity, System, World};
use crate::renderer::{DrawRect, UiQueue};
use crate::resources::ViewportSize;

use super::node::{Anchor, UiNode};

/// 자식 엔티티 배치 방향
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayoutDir {
    Vertical,
    Horizontal,
}

/// 자식 엔티티를 자동 배치하는 레이아웃 컨테이너.
///
/// `UiNode` 와 함께 엔티티에 붙여 사용한다.
/// `LayoutSystem` 이 매 프레임 `children` 의 `UiNode` 를 재배치한다.
/// `UiSystem` 이 배경 사각형을 렌더링한다.
pub struct Panel {
    pub children: Vec<Entity>,
    pub gap: f32,
    pub direction: LayoutDir,
    pub padding: f32,
    pub background_color: [f32; 4],
}

impl Panel {
    pub fn new(direction: LayoutDir) -> Self {
        Self {
            children: Vec::new(),
            gap: 8.0,
            direction,
            padding: 8.0,
            background_color: [0.12, 0.12, 0.18, 0.9],
        }
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
}

/// Panel 자식 엔티티 위치를 UiSystem 실행 전에 갱신하는 시스템.
///
/// `app.add_system(Box::new(LayoutSystem))` 을 `UiSystem` 보다 먼저 등록해야 한다.
pub struct LayoutSystem;

impl System for LayoutSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let viewport = match world.resource::<ViewportSize>() {
            Some(v) => ViewportSize { width: v.width, height: v.height },
            None => return,
        };

        // Step 1: 패널 데이터 수집 — 이터레이터를 살아있는 채로 get_mut 불가하므로 먼저 collect
        let panel_data: Vec<(Vec<Entity>, f32, LayoutDir, f32, Vec2)> = world
            .query2::<UiNode, Panel>()
            .map(|(_, node, panel)| {
                let pos = node.screen_pos(&viewport);
                (panel.children.clone(), panel.gap, panel.direction, panel.padding, pos)
            })
            .collect();

        // Step 2: 수집 후 이터레이터 해제 → get_mut 안전
        for (children, gap, direction, padding, panel_pos) in panel_data {
            let start_x = panel_pos.x + padding;
            let start_y = panel_pos.y + padding;
            let mut cursor_x = start_x;
            let mut cursor_y = start_y;

            for child_entity in children {
                let child_size = match world.get::<UiNode>(child_entity) {
                    Some(n) => n.size,
                    None => continue,
                };
                if let Some(child_node) = world.get_mut::<UiNode>(child_entity) {
                    child_node.anchor = Anchor::TopLeft;
                    match direction {
                        LayoutDir::Vertical => {
                            child_node.offset = Vec2::new(start_x, cursor_y);
                            cursor_y += child_size.y + gap;
                        }
                        LayoutDir::Horizontal => {
                            child_node.offset = Vec2::new(cursor_x, start_y);
                            cursor_x += child_size.x + gap;
                        }
                    }
                }
            }
        }

        // Step 3: 패널 배경 렌더링 (자식보다 낮은 z로)
        let panel_entities: Vec<Entity> = world
            .query2::<UiNode, Panel>()
            .map(|(e, _, _)| e)
            .collect();

        let mut rects: Vec<DrawRect> = Vec::new();
        for entity in panel_entities {
            let (pos, size, z, visible) = match world.get::<UiNode>(entity) {
                Some(n) => (n.screen_pos(&viewport), n.size, n.z, n.visible),
                None => continue,
            };
            if !visible {
                continue;
            }
            let bg_color = match world.get::<Panel>(entity) {
                Some(p) => p.background_color,
                None => continue,
            };
            rects.push(DrawRect::new(pos.x, pos.y, size.x, size.y, bg_color).with_z(z - 0.01));
        }

        if let Some(ui_queue) = world.resource_mut::<UiQueue>() {
            for rect in rects {
                ui_queue.push(rect);
            }
        }
    }
}
