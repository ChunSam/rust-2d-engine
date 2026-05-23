use crate::animation::player::AnimationPlayer;
use crate::ecs::{Entity, System, World};

/// 매 프레임 `AnimationPlayer` 타이머를 진행하고 `UvRect` 컴포넌트를 동기화한다.
///
/// `UvRect`를 직접 써서 렌더러가 `AnimationPlayer`를 알 필요 없도록 한다.
pub struct AnimationSystem;

impl System for AnimationSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        // 엔티티 목록 먼저 수집 (이터레이터 중 world 재빌림 방지)
        let entities: Vec<Entity> = world.query::<AnimationPlayer>().map(|(e, _)| e).collect();

        for entity in entities {
            let uv = {
                let Some(player) = world.get_mut::<AnimationPlayer>(entity) else {
                    continue;
                };
                let Some(clip) = player.clips.get(player.current_clip) else {
                    continue;
                };
                if clip.frames.is_empty() {
                    continue;
                }

                let frame_duration = 1.0 / clip.fps;
                player.timer += dt;

                if player.timer >= frame_duration {
                    player.timer -= frame_duration;
                    let frame_count = player.clips[player.current_clip].frames.len();
                    if player.clips[player.current_clip].looping {
                        player.current_frame = (player.current_frame + 1) % frame_count;
                    } else {
                        player.current_frame = (player.current_frame + 1).min(frame_count - 1);
                    }
                }
                player.current_uv()
            };
            // AnimationPlayer 빌림 해제 후 UvRect 컴포넌트에 기록
            world.add_component(entity, uv);
        }
    }
}
