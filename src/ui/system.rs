use glam::Vec2;
use winit::event::MouseButton;

use crate::ecs::{Entity, Events, System, World};
use crate::input::InputState;
use crate::renderer::{DrawRect, DrawText, TextQueue, UiQueue};
use crate::resources::ViewportSize;

use super::button::{Button, ButtonState};
use super::label::Label;
use super::node::UiNode;
use super::scroll_view::ScrollView;
use super::text_input::TextInput;

/// `UiNode` + `Button` / `Label` / `TextInput` / `ScrollView` 엔티티를 처리하는 시스템.
///
/// 매 프레임 실행 순서:
/// 1. 입력 상태 스냅샷
/// 2. 버튼 히트 테스트 → `ButtonState` 갱신 + `UiEvent` 발행
/// 3. TextInput 패스 — 포커스, 문자 입력, 커서 깜빡임
/// 4. ScrollView 패스 — 휠 스크롤, 아이템 렌더
/// 5. Label 패스
/// 6. 렌더 큐 제출
/// 7. 이벤트 일괄 발행
pub struct UiSystem;

impl System for UiSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        // ── 1. 입력 스냅샷 ───────────────────────────────────────────────────
        let (cursor, just_pressed, just_released, is_held, scroll_delta, chars) =
            match world.resource::<InputState>() {
                Some(input) => (
                    input.cursor(),
                    input.mouse_just_pressed(MouseButton::Left),
                    input.mouse_just_released(MouseButton::Left),
                    input.is_mouse_pressed(MouseButton::Left),
                    input.scroll(),
                    input.text_chars().to_vec(),
                ),
                None => return,
            };

        let viewport = match world.resource::<ViewportSize>() {
            Some(v) => ViewportSize { width: v.width, height: v.height },
            None => return,
        };

        let mut rects: Vec<DrawRect> = Vec::new();
        let mut texts: Vec<DrawText> = Vec::new();
        let mut ui_events: Vec<UiEvent> = Vec::new();

        // ── 2. 버튼 히트 테스트 ──────────────────────────────────────────────
        let button_entities: Vec<Entity> = world
            .query2::<UiNode, Button>()
            .map(|(e, _, _)| e)
            .collect();

        for entity in button_entities {
            let (pos, size, z, visible) = match world.get::<UiNode>(entity) {
                Some(node) => (node.screen_pos(&viewport), node.size, node.z, node.visible),
                None => continue,
            };
            if !visible {
                continue;
            }

            let in_rect = in_bounds(cursor, pos, size);

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
                if prev == ButtonState::Pressed && just_released && in_rect {
                    ui_events.push(UiEvent::ButtonClicked(entity));
                }
                if in_rect && just_pressed {
                    btn.state = ButtonState::Pressed;
                }
            }

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

        // ── 3. TextInput 패스 ────────────────────────────────────────────────
        let text_input_entities: Vec<Entity> = world
            .query2::<UiNode, TextInput>()
            .map(|(e, _, _)| e)
            .collect();

        // 클릭 시 포커스 변경 여부 확인 (unfocus 대상 수집)
        let mut newly_focused: Option<Entity> = None;
        if just_pressed {
            for &entity in &text_input_entities {
                let (pos, size) = match world.get::<UiNode>(entity) {
                    Some(n) => (n.screen_pos(&viewport), n.size),
                    None => continue,
                };
                if in_bounds(cursor, pos, size) {
                    newly_focused = Some(entity);
                    break;
                }
            }
        }

        for &entity in &text_input_entities {
            let (pos, size, z, visible) = match world.get::<UiNode>(entity) {
                Some(n) => (n.screen_pos(&viewport), n.size, n.z, n.visible),
                None => continue,
            };
            if !visible {
                continue;
            }

            // 포커스 전환
            if just_pressed {
                let ti = match world.get_mut::<TextInput>(entity) {
                    Some(t) => t,
                    None => continue,
                };
                let was_focused = ti.focused;
                ti.focused = newly_focused == Some(entity);
                if !was_focused && ti.focused {
                    ui_events.push(UiEvent::TextFocused(entity));
                    ti.cursor_blink = 0.0;
                    ti.cursor_visible = true;
                } else if was_focused && !ti.focused {
                    ui_events.push(UiEvent::TextBlurred(entity));
                }
            }

            // 문자 입력 처리 (focused 상태에서만)
            {
                let focused = world.get::<TextInput>(entity).map_or(false, |t| t.focused);
                if focused {
                    // 커서 깜빡임
                    if let Some(ti) = world.get_mut::<TextInput>(entity) {
                        ti.cursor_blink += dt;
                        if ti.cursor_blink >= 0.5 {
                            ti.cursor_blink -= 0.5;
                            ti.cursor_visible = !ti.cursor_visible;
                        }
                    }

                    // 문자 버퍼 소비
                    for &c in &chars {
                        match c {
                            '\x08' => {
                                if let Some(ti) = world.get_mut::<TextInput>(entity) {
                                    ti.backspace();
                                    let text = ti.text.clone();
                                    ui_events.push(UiEvent::TextChanged(entity, text));
                                }
                            }
                            '\n' => {
                                if let Some(ti) = world.get_mut::<TextInput>(entity) {
                                    let text = ti.text.clone();
                                    ti.focused = false;
                                    ui_events.push(UiEvent::TextSubmitted(entity, text));
                                    ui_events.push(UiEvent::TextBlurred(entity));
                                }
                            }
                            ch => {
                                if let Some(ti) = world.get_mut::<TextInput>(entity) {
                                    ti.insert_char(ch);
                                    let text = ti.text.clone();
                                    ui_events.push(UiEvent::TextChanged(entity, text));
                                }
                            }
                        }
                    }
                }
            }

            // 렌더 명령 수집
            let (bg_color, display_text, text_color, font_size) = {
                let ti = match world.get::<TextInput>(entity) {
                    Some(t) => t,
                    None => continue,
                };
                let display = if ti.text.is_empty() && !ti.focused {
                    ti.placeholder.clone()
                } else if ti.focused && ti.cursor_visible {
                    format!("{}|", ti.text)
                } else {
                    ti.text.clone()
                };
                (ti.current_color(), display, ti.text_color, ti.font_size)
            };

            rects.push(DrawRect::new(pos.x, pos.y, size.x, size.y, bg_color).with_z(z));
            texts.push(DrawText {
                text: display_text,
                position: Vec2::new(pos.x + 6.0, pos.y + (size.y - font_size) / 2.0),
                size: font_size,
                color: text_color,
            });
        }

        // ── 4. ScrollView 패스 ───────────────────────────────────────────────
        let scroll_entities: Vec<Entity> = world
            .query2::<UiNode, ScrollView>()
            .map(|(e, _, _)| e)
            .collect();

        for entity in scroll_entities {
            let (pos, size, z, visible) = match world.get::<UiNode>(entity) {
                Some(n) => (n.screen_pos(&viewport), n.size, n.z, n.visible),
                None => continue,
            };
            if !visible {
                continue;
            }

            // 휠 스크롤
            if scroll_delta != 0.0 && in_bounds(cursor, pos, size) {
                if let Some(sv) = world.get_mut::<ScrollView>(entity) {
                    sv.scroll_offset -= scroll_delta * sv.item_height;
                    sv.clamp_scroll(size.y);
                }
            }

            let (scroll_offset, item_height, font_size, color, bg_color, item_count) = {
                let sv = match world.get::<ScrollView>(entity) {
                    Some(s) => s,
                    None => continue,
                };
                (sv.scroll_offset, sv.item_height, sv.font_size, sv.color, sv.background_color, sv.items.len())
            };

            rects.push(DrawRect::new(pos.x, pos.y, size.x, size.y, bg_color).with_z(z));

            let first = (scroll_offset / item_height).floor() as usize;
            let last = (first + (size.y / item_height).ceil() as usize + 1).min(item_count);

            let sv = match world.get::<ScrollView>(entity) {
                Some(s) => s,
                None => continue,
            };
            for i in first..last {
                let y = pos.y + (i as f32 * item_height) - scroll_offset;
                if y + item_height < pos.y || y > pos.y + size.y {
                    continue;
                }
                texts.push(DrawText {
                    text: sv.items[i].clone(),
                    position: Vec2::new(pos.x + 4.0, y),
                    size: font_size,
                    color,
                });
            }
        }

        // ── 5. Label 패스 ────────────────────────────────────────────────────
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

        // ── 6. 렌더 큐 제출 ──────────────────────────────────────────────────
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

        // ── 7. 이벤트 일괄 발행 ──────────────────────────────────────────────
        if !ui_events.is_empty() {
            if let Some(events) = world.resource_mut::<Events<UiEvent>>() {
                for ev in ui_events {
                    events.send(ev);
                }
            }
        }
    }
}

/// UI 시스템이 발행하는 이벤트.
///
/// `app.register_event::<UiEvent>()` 후 `world.resource::<Events<UiEvent>>()` 로 읽는다.
#[derive(Debug, Clone)]
pub enum UiEvent {
    ButtonClicked(Entity),
    TextChanged(Entity, String),
    TextSubmitted(Entity, String),
    TextFocused(Entity),
    TextBlurred(Entity),
}

// ── 헬퍼 ─────────────────────────────────────────────────────────────────────

fn in_bounds(cursor: Vec2, pos: Vec2, size: Vec2) -> bool {
    cursor.x >= pos.x
        && cursor.x <= pos.x + size.x
        && cursor.y >= pos.y
        && cursor.y <= pos.y + size.y
}
