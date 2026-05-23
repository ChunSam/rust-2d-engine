use crate::components::AnimationPlayer;
use crate::ecs::{System, World};

/// 매 프레임 AnimationPlayer 컴포넌트의 타이머를 진행하고 현재 프레임을 갱신한다.
pub struct AnimationSystem;

impl System for AnimationSystem {
    fn run(&mut self, world: &mut World, dt: f32) {
        // query 이터레이터가 world를 빌리는 동안 get_mut을 호출할 수 없으므로
        // entity ID를 먼저 수집한 뒤 두 번째 루프에서 변경한다.
        let entities: Vec<_> = world.query::<AnimationPlayer>().map(|(e, _)| e).collect();

        for entity in entities {
            if let Some(player) = world.get_mut::<AnimationPlayer>(entity) {
                let Some(clip) = player.clips.get(player.current_clip) else {
                    continue;
                };
                let frame_duration = 1.0 / clip.fps.max(0.001);
                let looping = clip.looping;
                let frame_count = clip.frames.len();

                player.timer += dt;
                while player.timer >= frame_duration {
                    player.timer -= frame_duration;
                    player.current_frame += 1;
                    if player.current_frame >= frame_count {
                        player.current_frame = if looping {
                            0
                        } else {
                            frame_count.saturating_sub(1)
                        };
                    }
                }
            }
        }
    }
}
