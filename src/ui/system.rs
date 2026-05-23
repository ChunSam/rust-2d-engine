use glam::Vec2;
use winit::event::MouseButton;

use crate::ecs::{Entity, Events, System, World};
use crate::input::InputState;
use crate::renderer::{DrawRect, DrawText, TextQueue, UiQueue};
use crate::resources::ViewportSize;

use super::button::{Button, ButtonState};
use super::label::Label;
use super::node::UiNode;

/// `UiNode` + `Button` 또는 `UiNode` + `Label` 엔티티를 처리하는 시스템.
///
/// 매 프레임 실행 순서:
/// 1. 입력 상태 스냅샷 (불변 읽기 → 로컬 복사)
/// 2. 버튼 히트 테스트 → `ButtonState` 갱신 + `UiEvent` 발행
/// 3. 버튼 배경 DrawRect → `UiQueue` 제출
/// 4. 버튼 레이블 / 독립 Label DrawText → `TextQueue` 제출
pub struct UiSystem;

impl System for UiSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        // ── 1. 입력 스냅샷 ───────────────────────────────────────────────────
        let (cursor, just_pressed, just_released) = match world.resource::<InputState>() {
            Some(input) => (
                input.cursor(),
                input.mouse_just_pressed(MouseButton::Left),
                input.mouse_just_released(MouseButton::Left),
            ),
            None => return,
        };
        let is_held = match world.resource::<InputState>() {
            Some(input) => input.is_mouse_pressed(MouseButton::Left),
            None => return,
        };

        let viewport = match world.resource::<ViewportSize>() {
            Some(v) => ViewportSize { width: v.width, height: v.height },
            None => return,
        };

        // ── 2. 버튼 히트 테스트 ──────────────────────────────────────────────
        // 엔티티 목록을 먼저 수집해 World 불변 빌림을 해제한다.
        let button_entities: Vec<Entity> = world
            .query2::<UiNode, Button>()
            .map(|(e, _, _)| e)
            .collect();

        // 클릭 이벤트를 나중에 일괄 발행하기 위해 수집한다.
        let mut clicked: Vec<Entity> = Vec::new();
        // 렌더링 명령도 수집 후 일괄 제출한다.
        let mut rects: Vec<DrawRect> = Vec::new();
        let mut texts: Vec<DrawText> = Vec::new();

        for entity in button_entities {
            let (pos, size, z, visible) = match world.get::<UiNode>(entity) {
                Some(node) => (node.screen_pos(&viewport), node.size, node.z, node.visible),
                None => continue,
            };
            if !visible {
                continue;
            }

            let in_rect = cursor.x >= pos.x
                && cursor.x <= pos.x + size.x
                && cursor.y >= pos.y
                && cursor.y <= pos.y + size.y;

            // 상태 전환
            let btn = match world.get_mut::<Button>(entity) {
                Some(b) => b,
                None => continue,
            };
            if btn.state != ButtonState::Disabled {
                let prev = btn.state;
                btn.state = if in_rect {
                    if is_held { ButtonState::Pressed } else { ButtonState::Hovered }
                } else {
                    ButtonState::Normal
                };
                // Pressed → Released while inside = Click
                if prev == ButtonState::Pressed && just_released && in_rect {
                    clicked.push(entity);
                }
                // just_pressed outside doesn't count
                if in_rect && just_pressed {
                    btn.state = ButtonState::Pressed;
                }
            }

            // 렌더 명령 수집
            let (color, label_text, text_color, font_size) = {
                let btn = world.get::<Button>(entity).unwrap();
                (btn.current_color(), btn.label.clone(), btn.text_color, btn.font_size)
            };

            rects.push(DrawRect::new(pos.x, pos.y, size.x, size.y, color).with_z(z));

            if !label_text.is_empty() {
                let text_x = pos.x + size.x / 2.0;
                let text_y = pos.y + (size.y - font_size) / 2.0;
                texts.push(DrawText {
                    text: label_text,
                    position: Vec2::new(text_x, text_y),
                    size: font_size,
                    color: text_color,
                });
            }
        }

        // ── 3. Label 처리 ────────────────────────────────────────────────────
        let label_entities: Vec<Entity> = world
            .query2::<UiNode, Label>()
            .map(|(e, _, _)| e)
            .collect();

        for entity in label_entities {
            let (pos, visible) = match world.get::<UiNode>(entity) {
                Some(node) => (node.screen_pos(&viewport), node.visible),
                None => continue,
            };
            if !visible {
                continue;
            }
            if let Some(label) = world.get::<Label>(entity) {
                texts.push(DrawText {
                    text: label.text.clone(),
                    position: pos,
                    size: label.font_size,
                    color: label.color,
                });
            }
        }

        // ── 4. 렌더 큐 제출 ──────────────────────────────────────────────────
        if let Some(ui_queue) = world.resource_mut::<UiQueue>() {
            for rect in rects {
                ui_queue.push(rect);
            }
        }
        if let Some(text_queue) = world.resource_mut::<TextQueue>() {
            for text in texts {
                text_queue.push(text);
            }
        }

        // ── 5. 이벤트 발행 ───────────────────────────────────────────────────
        if !clicked.is_empty() {
            if let Some(events) = world.resource_mut::<Events<UiEvent>>() {
                for entity in clicked {
                    events.send(UiEvent::ButtonClicked(entity));
                }
            }
        }
    }
}

/// UI 시스템이 발행하는 이벤트.
///
/// `app.register_event::<UiEvent>()` 후 `world.resource::<Events<UiEvent>>()` 로 읽는다.
#[derive(Debug, Clone, Copy)]
pub enum UiEvent {
    ButtonClicked(Entity),
}
