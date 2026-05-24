use std::collections::HashMap;

use crate::animation::player::AnimationPlayer;
use crate::ecs::{Entity, System, World};

// ─── 파라미터 ─────────────────────────────────────────────────────────────────

/// 상태 머신이 보유하는 파라미터 값
#[derive(Debug, Clone)]
pub enum AnimParam {
    Bool(bool),
    Float(f32),
    /// 한 프레임만 유효한 트리거. `fire_trigger()` 로 활성화하고 매 프레임 소비된다.
    Trigger(bool),
}

// ─── 전환 조건 ────────────────────────────────────────────────────────────────

/// 상태 전환이 일어나기 위해 충족해야 하는 단일 조건
#[derive(Debug, Clone)]
pub enum TransitionCond {
    /// 불 파라미터가 기대값과 일치할 때
    BoolEq(String, bool),
    /// 실수 파라미터가 임계값을 초과할 때
    FloatGt(String, f32),
    /// 실수 파라미터가 임계값 미만일 때
    FloatLt(String, f32),
    /// 트리거 파라미터가 활성화됐을 때
    Trigger(String),
    /// 현재 클립이 끝(non-looping 마지막 프레임)에 도달했을 때
    AnimationEnd,
}

// ─── 전환 ─────────────────────────────────────────────────────────────────────

/// 하나의 상태 전환 엣지: 대상 상태 + 충족해야 할 조건 목록 (AND)
#[derive(Debug, Clone)]
pub struct AnimTransition {
    /// 전환될 상태 이름
    pub to: String,
    /// 모두 충족해야 전환이 일어난다
    pub conditions: Vec<TransitionCond>,
}

// ─── 상태 노드 ────────────────────────────────────────────────────────────────

/// 상태 머신의 한 노드: `AnimationPlayer`의 클립 인덱스와 전환 목록
#[derive(Debug, Clone)]
pub struct AnimState {
    /// 이 상태에서 재생할 `AnimationPlayer` 클립 인덱스
    pub clip_index: usize,
    /// 이 상태에서 평가될 전환 엣지들 (등록 순서대로 우선 평가)
    pub transitions: Vec<AnimTransition>,
}

// ─── 상태 머신 컴포넌트 ───────────────────────────────────────────────────────

/// 엔티티에 붙이는 애니메이션 상태 머신 컴포넌트.
///
/// `AnimationPlayer`와 같은 엔티티에 추가한 뒤, `StateMachineSystem`을
/// `AnimationSystem` **이후에** 등록하면 된다.
///
/// # 등록 순서
/// ```text
/// app.add_system(Box::new(AnimationSystem));     // 프레임 진행
/// app.add_system(Box::new(StateMachineSystem));  // 상태 전환
/// ```
///
/// # 예시
/// ```rust,ignore
/// let mut sm = AnimationStateMachine::new("idle", 0);
/// sm.add_state("run", 1)
///   .add_state("jump", 2);
/// sm.set_bool("is_running", false);
/// sm.add_trigger("jump");
/// sm.add_transition("idle", "run",  vec![TransitionCond::BoolEq("is_running".into(), true)]);
/// sm.add_transition("run",  "idle", vec![TransitionCond::BoolEq("is_running".into(), false)]);
/// sm.add_transition("idle", "jump", vec![TransitionCond::Trigger("jump".into())]);
/// sm.add_transition("jump", "idle", vec![TransitionCond::AnimationEnd]);
/// world.add_component(entity, sm);
/// ```
#[derive(Debug, Clone)]
pub struct AnimationStateMachine {
    states: HashMap<String, AnimState>,
    current: String,
    params: HashMap<String, AnimParam>,
}

impl AnimationStateMachine {
    /// 초기 상태 이름과 해당 클립 인덱스로 상태 머신을 생성한다.
    pub fn new(initial_state: impl Into<String>, initial_clip: usize) -> Self {
        let initial_state = initial_state.into();
        let mut states = HashMap::new();
        states.insert(
            initial_state.clone(),
            AnimState {
                clip_index: initial_clip,
                transitions: Vec::new(),
            },
        );
        Self {
            states,
            current: initial_state,
            params: HashMap::new(),
        }
    }

    // ── 상태/전환 등록 ──────────────────────────────────────────────────────────

    /// 새 상태를 추가한다. 이미 존재하는 이름이면 무시한다.
    pub fn add_state(&mut self, name: impl Into<String>, clip_index: usize) -> &mut Self {
        self.states.entry(name.into()).or_insert(AnimState {
            clip_index,
            transitions: Vec::new(),
        });
        self
    }

    /// `from` 상태에서 `to` 상태로의 전환 엣지를 등록한다.
    /// `from` 상태가 없으면 아무것도 하지 않는다.
    pub fn add_transition(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        conditions: Vec<TransitionCond>,
    ) -> &mut Self {
        let from = from.into();
        let to = to.into();
        if let Some(state) = self.states.get_mut(&from) {
            state.transitions.push(AnimTransition { to, conditions });
        }
        self
    }

    // ── 파라미터 읽기/쓰기 ─────────────────────────────────────────────────────

    /// 불 파라미터를 설정하거나 업데이트한다.
    pub fn set_bool(&mut self, name: impl Into<String>, value: bool) {
        self.params.insert(name.into(), AnimParam::Bool(value));
    }

    /// 불 파라미터 값을 읽는다. 없거나 타입이 다르면 `None`.
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        match self.params.get(name) {
            Some(AnimParam::Bool(v)) => Some(*v),
            _ => None,
        }
    }

    /// 실수 파라미터를 설정하거나 업데이트한다.
    pub fn set_float(&mut self, name: impl Into<String>, value: f32) {
        self.params.insert(name.into(), AnimParam::Float(value));
    }

    /// 실수 파라미터 값을 읽는다. 없거나 타입이 다르면 `None`.
    pub fn get_float(&self, name: &str) -> Option<f32> {
        match self.params.get(name) {
            Some(AnimParam::Float(v)) => Some(*v),
            _ => None,
        }
    }

    /// 트리거 파라미터를 등록한다 (초기값 false).
    pub fn add_trigger(&mut self, name: impl Into<String>) {
        self.params
            .entry(name.into())
            .or_insert(AnimParam::Trigger(false));
    }

    /// 트리거를 활성화한다. 같은 프레임 안에서 `StateMachineSystem`이 소비한다.
    pub fn fire_trigger(&mut self, name: &str) {
        if let Some(AnimParam::Trigger(v)) = self.params.get_mut(name) {
            *v = true;
        }
    }

    /// 현재 활성 상태 이름을 반환한다.
    pub fn current_state(&self) -> &str {
        &self.current
    }

    // ── 내부 평가 ──────────────────────────────────────────────────────────────

    fn check_condition(&self, cond: &TransitionCond, anim_finished: bool) -> bool {
        match cond {
            TransitionCond::BoolEq(name, expected) => {
                matches!(self.params.get(name.as_str()), Some(AnimParam::Bool(v)) if v == expected)
            }
            TransitionCond::FloatGt(name, threshold) => {
                matches!(self.params.get(name.as_str()), Some(AnimParam::Float(v)) if v > threshold)
            }
            TransitionCond::FloatLt(name, threshold) => {
                matches!(self.params.get(name.as_str()), Some(AnimParam::Float(v)) if v < threshold)
            }
            TransitionCond::Trigger(name) => {
                matches!(
                    self.params.get(name.as_str()),
                    Some(AnimParam::Trigger(true))
                )
            }
            TransitionCond::AnimationEnd => anim_finished,
        }
    }

    /// 현재 상태에서 조건을 만족하는 첫 번째 전환을 찾아 `(대상 상태, 클립 인덱스)` 반환.
    fn evaluate(&self, anim_finished: bool) -> Option<(String, usize)> {
        let state = self.states.get(&self.current)?;
        for transition in &state.transitions {
            if transition
                .conditions
                .iter()
                .all(|c| self.check_condition(c, anim_finished))
            {
                let next_clip = self.states.get(&transition.to)?.clip_index;
                return Some((transition.to.clone(), next_clip));
            }
        }
        None
    }

    /// 모든 트리거 파라미터를 소비(false)한다.
    fn consume_triggers(&mut self) {
        for param in self.params.values_mut() {
            if let AnimParam::Trigger(v) = param {
                *v = false;
            }
        }
    }
}

// ─── 시스템 ───────────────────────────────────────────────────────────────────

/// 매 프레임 `AnimationStateMachine`의 전환 조건을 평가하고, 충족 시 `AnimationPlayer`에
/// 새 클립을 재생하도록 지시한다.
///
/// `AnimationSystem` **이후에** 등록해야 `is_finished()` 판정이 같은 프레임에 반영된다.
pub struct StateMachineSystem;

impl System for StateMachineSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let entities: Vec<Entity> = world
            .query::<AnimationStateMachine>()
            .map(|(e, _)| e)
            .collect();

        for entity in entities {
            let anim_finished = world
                .get_mut::<AnimationPlayer>(entity)
                .map(|p| p.is_finished())
                .unwrap_or(false);

            let transition = world
                .get_mut::<AnimationStateMachine>(entity)
                .and_then(|sm| sm.evaluate(anim_finished));

            if let Some((next_state, clip_index)) = transition {
                if let Some(sm) = world.get_mut::<AnimationStateMachine>(entity) {
                    sm.current = next_state;
                    sm.consume_triggers();
                }
                if let Some(player) = world.get_mut::<AnimationPlayer>(entity) {
                    player.play(clip_index);
                }
            } else {
                // 전환이 없어도 트리거는 한 프레임만 유효하므로 소비
                if let Some(sm) = world.get_mut::<AnimationStateMachine>(entity) {
                    sm.consume_triggers();
                }
            }
        }
    }
}
