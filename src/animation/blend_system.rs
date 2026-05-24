use crate::animation::blend_tree::BlendTree1D;
use crate::animation::player::AnimationPlayer;
use crate::ecs::{Entity, System, World};

/// 매 프레임 `BlendTree1D`의 param을 평가해 `AnimationPlayer`에 클립 전환을 지시한다.
///
/// `AnimationSystem` **이전에** 등록해야 클립 전환이 같은 프레임에 반영된다.
pub struct BlendTreeSystem;

impl System for BlendTreeSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let entities: Vec<Entity> = world.query::<BlendTree1D>().map(|(e, _)| e).collect();

        for entity in entities {
            // 대상 클립과 크로스페이드 지속 시간을 BlendTree1D에서 추출
            let (target_clip, crossfade_dur, already_requested) = {
                let Some(tree) = world.get_mut::<BlendTree1D>(entity) else {
                    continue;
                };
                let target = tree.target_clip();
                (target, tree.crossfade_duration, tree.last_clip)
            };

            let Some(clip_index) = target_clip else {
                continue;
            };

            // 이미 이 클립을 요청했으면 재요청하지 않는다
            if already_requested == Some(clip_index) {
                continue;
            }

            // AnimationPlayer에 전환 지시
            {
                let Some(player) = world.get_mut::<AnimationPlayer>(entity) else {
                    continue;
                };
                if player.current_clip != clip_index && !player.is_crossfading() {
                    player.play_with_crossfade(clip_index, crossfade_dur);
                } else if player.current_clip == clip_index {
                    // 이미 재생 중 — last_clip만 업데이트
                }
            }

            // 요청 기록 업데이트
            if let Some(tree) = world.get_mut::<BlendTree1D>(entity) {
                tree.last_clip = Some(clip_index);
            }
        }
    }
}
