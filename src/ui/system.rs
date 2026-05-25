use glam::Vec2;
use winit::event::MouseButton;

use crate::ecs::{Entity, Events, System, World};
use crate::input::InputState;
use crate::renderer::{DrawRect, DrawText, TextAlign, TextQueue, UiQueue};
use crate::resources::ViewportSize;

use super::button::{Button, ButtonState};
use super::checkbox::CheckBox;
use super::label::Label;
use super::node::UiNode;
use super::scroll_view::ScrollView;
use super::slider::Slider;
use super::text_input::TextInput;

/// `UiNode` + `Button` / `Label` / `TextInput` / `ScrollView` / `Slider` / `CheckBox` 엔티티를 처리하는 시스템.
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
        let (cursor, just_pressed, just_released, is_held, scroll_delta, chars, ime_preedit) =
            match world.resource::<InputState>() {
                Some(input) => (
                    input.cursor(),
                    input.mouse_just_pressed(MouseButton::Left),
                    input.mouse_just_released(MouseButton::Left),
                    input.is_mouse_pressed(MouseButton::Left),
                    input.scroll(),
                    input.text_chars().to_vec(),
                    input.ime_preedit().to_string(),
                ),
                None => return,
            };

        let viewport = match world.resource::<ViewportSize>() {
            Some(v) => ViewportSize {
                width: v.width,
                height: v.height,
            },
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
                    if is_held {
                        ButtonState::Pressed
                    } else {
                        ButtonState::Hovered
                    }
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
                (
                    btn.current_color(),
                    btn.label.clone(),
                    btn.text_color,
                    btn.font_size,
                )
            };

            rects.push(DrawRect::new(pos.x, pos.y, size.x, size.y, color).with_z(z));

            if !label_text.is_empty() {
                let text_y = pos.y + (size.y - font_size) / 2.0;
                texts.push(
                    DrawText::new(label_text, Vec2::new(pos.x, text_y), font_size, text_color)
                        .with_bounds(Vec2::new(size.x, size.y))
                        .with_align(TextAlign::Center),
                );
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
                let focused = world.get::<TextInput>(entity).is_some_and(|t| t.focused);
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
                    if let Some(ti) = world.get_mut::<TextInput>(entity) {
                        ti.preedit = ime_preedit.clone();
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
                    format!("{}|", ti.text_with_preedit())
                } else {
                    ti.text_with_preedit()
                };
                (ti.current_color(), display, ti.text_color, ti.font_size)
            };

            rects.push(DrawRect::new(pos.x, pos.y, size.x, size.y, bg_color).with_z(z));
            texts.push(
                DrawText::new(
                    display_text,
                    Vec2::new(pos.x + 6.0, pos.y + (size.y - font_size) / 2.0),
                    font_size,
                    text_color,
                )
                .with_bounds(Vec2::new((size.x - 12.0).max(0.0), size.y)),
            );
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
                (
                    sv.scroll_offset,
                    sv.item_height,
                    sv.font_size,
                    sv.color,
                    sv.background_color,
                    sv.items.len(),
                )
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
                texts.push(
                    DrawText::new(
                        sv.items[i].clone(),
                        Vec2::new(pos.x + 4.0, y),
                        font_size,
                        color,
                    )
                    .with_bounds(Vec2::new((size.x - 8.0).max(0.0), item_height)),
                );
            }
        }

        // ── 5. Label 패스 ────────────────────────────────────────────────────
        let label_entities: Vec<Entity> =
            world.query2::<UiNode, Label>().map(|(e, _, _)| e).collect();

        for entity in label_entities {
            let (pos, size, visible) = match world.get::<UiNode>(entity) {
                Some(node) => (node.screen_pos(&viewport), node.size, node.visible),
                None => continue,
            };
            if !visible {
                continue;
            }
            if let Some(label) = world.get::<Label>(entity) {
                let mut text = DrawText::new(label.text.clone(), pos, label.font_size, label.color)
                    .with_bounds(size)
                    .with_align(label.align);
                if label.rich {
                    text = text.rich();
                }
                texts.push(text);
            }
        }

        // ── 6. Slider 패스 ───────────────────────────────────────────────────
        let slider_entities: Vec<Entity> = world
            .query2::<UiNode, Slider>()
            .map(|(e, _, _)| e)
            .collect();

        for entity in slider_entities {
            let (pos, size, z, visible) = match world.get::<UiNode>(entity) {
                Some(n) => (n.screen_pos(&viewport), n.size, n.z, n.visible),
                None => continue,
            };
            if !visible {
                continue;
            }

            let thumb_w = world.get::<Slider>(entity).map_or(14.0, |s| s.thumb_width);
            let track_len = (size.x - thumb_w).max(0.0);

            // 마우스 누름 → 드래그 시작
            if just_pressed && in_bounds(cursor, pos, size) {
                if let Some(slider) = world.get_mut::<Slider>(entity) {
                    let t = ((cursor.x - pos.x - thumb_w / 2.0) / track_len.max(f32::EPSILON))
                        .clamp(0.0, 1.0);
                    slider.set_normalized(t);
                    slider.dragging = true;
                    let v = slider.value;
                    ui_events.push(UiEvent::SliderChanged(entity, v));
                }
            }

            // 드래그 중
            {
                let dragging = world.get::<Slider>(entity).is_some_and(|s| s.dragging);
                if dragging {
                    if is_held {
                        if let Some(slider) = world.get_mut::<Slider>(entity) {
                            let t = ((cursor.x - pos.x - thumb_w / 2.0)
                                / track_len.max(f32::EPSILON))
                            .clamp(0.0, 1.0);
                            let new_val = slider.min + t * (slider.max - slider.min);
                            if (new_val - slider.value).abs() > f32::EPSILON {
                                slider.value = new_val;
                                let v = slider.value;
                                ui_events.push(UiEvent::SliderChanged(entity, v));
                            }
                        }
                    } else if let Some(slider) = world.get_mut::<Slider>(entity) {
                        slider.dragging = false;
                    }
                }
            }

            let (norm, track_col, fill_col, thumb_col, thumb_hover_col) = {
                let s = match world.get::<Slider>(entity) {
                    Some(s) => s,
                    None => continue,
                };
                (
                    s.normalized(),
                    s.track_color,
                    s.fill_color,
                    s.thumb_color,
                    s.thumb_hovered_color,
                )
            };

            let thumb_x = pos.x + norm * track_len;
            let thumb_hovered = in_bounds(
                cursor,
                Vec2::new(thumb_x, pos.y),
                Vec2::new(thumb_w, size.y),
            );

            rects.push(DrawRect::new(pos.x, pos.y, size.x, size.y, track_col).with_z(z));
            rects.push(
                DrawRect::new(
                    pos.x,
                    pos.y,
                    thumb_x - pos.x + thumb_w / 2.0,
                    size.y,
                    fill_col,
                )
                .with_z(z + 0.001),
            );
            let tc = if thumb_hovered {
                thumb_hover_col
            } else {
                thumb_col
            };
            rects.push(DrawRect::new(thumb_x, pos.y, thumb_w, size.y, tc).with_z(z + 0.002));
        }

        // ── 7. CheckBox 패스 ─────────────────────────────────────────────────
        let checkbox_entities: Vec<Entity> = world
            .query2::<UiNode, CheckBox>()
            .map(|(e, _, _)| e)
            .collect();

        for entity in checkbox_entities {
            let (pos, size, z, visible) = match world.get::<UiNode>(entity) {
                Some(n) => (n.screen_pos(&viewport), n.size, n.z, n.visible),
                None => continue,
            };
            if !visible {
                continue;
            }

            // 클릭 → 토글
            if just_pressed && in_bounds(cursor, pos, size) {
                if let Some(cb) = world.get_mut::<CheckBox>(entity) {
                    cb.checked = !cb.checked;
                    let checked = cb.checked;
                    ui_events.push(UiEvent::CheckBoxToggled(entity, checked));
                }
            }

            let (
                checked,
                box_size,
                border_col,
                checked_col,
                unchecked_col,
                label,
                text_color,
                font_size,
            ) = {
                let cb = match world.get::<CheckBox>(entity) {
                    Some(c) => c,
                    None => continue,
                };
                (
                    cb.checked,
                    cb.box_size,
                    cb.border_color,
                    cb.checked_color,
                    cb.unchecked_color,
                    cb.label.clone(),
                    cb.text_color,
                    cb.font_size,
                )
            };

            let box_y = pos.y + (size.y - box_size) / 2.0;
            let pad = 2.0;
            // 테두리
            rects.push(DrawRect::new(pos.x, box_y, box_size, box_size, border_col).with_z(z));
            // 내부 채움
            let inner_col = if checked { checked_col } else { unchecked_col };
            rects.push(
                DrawRect::new(
                    pos.x + pad,
                    box_y + pad,
                    box_size - pad * 2.0,
                    box_size - pad * 2.0,
                    inner_col,
                )
                .with_z(z + 0.001),
            );
            // 레이블 텍스트
            if !label.is_empty() {
                texts.push(
                    DrawText::new(
                        label,
                        Vec2::new(pos.x + box_size + 6.0, pos.y + (size.y - font_size) / 2.0),
                        font_size,
                        text_color,
                    )
                    .with_bounds(Vec2::new((size.x - box_size - 6.0).max(0.0), size.y)),
                );
            }
        }

        // ── 8. 렌더 큐 제출 ──────────────────────────────────────────────────
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

        // ── 9. 이벤트 일괄 발행 ──────────────────────────────────────────────
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
    /// Slider 값이 변경됨. 두 번째 필드는 새 값.
    SliderChanged(Entity, f32),
    /// CheckBox 상태가 토글됨. 두 번째 필드는 새 checked 값.
    CheckBoxToggled(Entity, bool),
}

// ── 헬퍼 ─────────────────────────────────────────────────────────────────────

fn in_bounds(cursor: Vec2, pos: Vec2, size: Vec2) -> bool {
    cursor.x >= pos.x
        && cursor.x <= pos.x + size.x
        && cursor.y >= pos.y
        && cursor.y <= pos.y + size.y
}
