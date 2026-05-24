use crate::animation::player::{AnimationPlayer, BlendWeight};
use crate::ecs::{Entity, System, World};

/// 매 프레임 `AnimationPlayer` 타이머를 진행하고 `UvRect`/`BlendWeight` 컴포넌트를 동기화한다.
///
/// 크로스페이드 중에는 두 클립을 병렬로 진행하고, 진행도가 50%를 넘는 순간 to_clip의 UV를 출력한다.
/// `BlendWeight` 컴포넌트는 항상 갱신되며(전환 없으면 1.0), 게임 코드에서 알파 보간 등에 활용할 수 있다.
pub struct AnimationSystem;

impl System for AnimationSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        let entities: Vec<Entity> = world.query::<AnimationPlayer>().map(|(e, _)| e).collect();

        for entity in entities {
            let (uv, weight) = {
                let Some(player) = world.get_mut::<AnimationPlayer>(entity) else {
                    continue;
                };

                // ── 크로스페이드 진행 ───────────────────────────────────────────
                if let Some(cf) = player.crossfade.as_mut() {
                    cf.elapsed += dt;

                    // to_clip 프레임 진행
                    if let Some(to_clip) = player.clips.get(cf.to_clip) {
                        if !to_clip.frames.is_empty() {
                            let frame_dur = 1.0 / to_clip.fps;
                            cf.to_timer += dt;
                            while cf.to_timer >= frame_dur {
                                cf.to_timer -= frame_dur;
                                let n = player.clips[cf.to_clip].frames.len();
                                if player.clips[cf.to_clip].looping {
                                    cf.to_frame = (cf.to_frame + 1) % n;
                                } else {
                                    cf.to_frame = (cf.to_frame + 1).min(n - 1);
                                }
                            }
                        }
                    }

                    // 전환 완료 여부 확인
                    if cf.elapsed >= cf.duration {
                        let to_clip = cf.to_clip;
                        let to_frame = cf.to_frame;
                        player.current_clip = to_clip;
                        player.current_frame = to_frame;
                        player.timer = 0.0;
                        player.crossfade = None;
                    }
                }

                // ── 현재 클립(from) 프레임 진행 ──────────────────────────────
                let Some(clip) = player.clips.get(player.current_clip) else {
                    continue;
                };
                if clip.frames.is_empty() {
                    continue;
                }
                let frame_dur = 1.0 / clip.fps;
                player.timer += dt;
                if player.timer >= frame_dur {
                    player.timer -= frame_dur;
                    let n = player.clips[player.current_clip].frames.len();
                    if player.clips[player.current_clip].looping {
                        player.current_frame = (player.current_frame + 1) % n;
                    } else {
                        player.current_frame = (player.current_frame + 1).min(n - 1);
                    }
                }

                // ── 출력 UV 결정 ─────────────────────────────────────────────
                // 진행도 ≥ 0.5에서 to_clip 프레임으로 전환
                let weight = player.blend_weight();
                let uv = if let Some(cf) = &player.crossfade {
                    if weight >= 0.5 {
                        player.clips[cf.to_clip]
                            .frames
                            .get(cf.to_frame)
                            .copied()
                            .unwrap_or(crate::animation::player::UvRect::FULL)
                    } else {
                        player.current_uv()
                    }
                } else {
                    player.current_uv()
                };

                (uv, weight)
            };

            // AnimationPlayer 빌림 해제 후 컴포넌트 기록
            world.add_component(entity, uv);
            world.add_component(entity, BlendWeight(weight));
        }
    }
}
