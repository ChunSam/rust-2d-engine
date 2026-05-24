/// 1D 블렌드 트리의 단일 항목: 파라미터 임계값과 재생할 클립 인덱스.
#[derive(Debug, Clone)]
pub struct BlendEntry {
    /// `BlendTree1D::param`이 이 값 이상일 때 이 클립이 선택된다.
    pub threshold: f32,
    pub clip_index: usize,
}

/// 1D 파라미터로 `AnimationPlayer` 클립을 자동 전환하는 컴포넌트.
///
/// `entries`를 threshold 오름차순으로 정렬해두면 param에 따라 가장 가까운 클립이
/// 선택된다. 클립이 바뀌는 순간 `crossfade_duration`만큼 부드럽게 크로스페이드한다.
///
/// # 등록 순서
/// ```text
/// app.add_system(Box::new(BlendTreeSystem));  // 클립 선택
/// app.add_system(Box::new(AnimationSystem));  // 프레임 진행
/// ```
///
/// # 예시
/// ```rust,ignore
/// let tree = BlendTree1D::new(
///     vec![
///         BlendEntry { threshold: 0.0, clip_index: 0 },  // idle
///         BlendEntry { threshold: 0.5, clip_index: 1 },  // walk
///         BlendEntry { threshold: 1.5, clip_index: 2 },  // run
///     ],
///     0.15,  // 크로스페이드 0.15초
/// );
/// world.add_component(entity, tree);
///
/// // 매 프레임 speed 파라미터 갱신
/// world.get_mut::<BlendTree1D>(entity).unwrap().set_param(speed);
/// ```
#[derive(Debug, Clone)]
pub struct BlendTree1D {
    /// threshold 오름차순으로 정렬해야 한다.
    pub entries: Vec<BlendEntry>,
    /// 현재 파라미터 값. `set_param()`으로 갱신한다.
    pub param: f32,
    /// 클립 전환 시 크로스페이드 지속 시간(초). 0이면 즉시 전환.
    pub crossfade_duration: f32,
    // BlendTreeSystem이 중복 요청을 막기 위해 추적하는 마지막 선택 클립
    pub(crate) last_clip: Option<usize>,
}

impl BlendTree1D {
    /// `entries`는 threshold 오름차순으로 전달한다.
    pub fn new(entries: Vec<BlendEntry>, crossfade_duration: f32) -> Self {
        Self {
            entries,
            param: 0.0,
            crossfade_duration,
            last_clip: None,
        }
    }

    /// 파라미터 값을 설정한다. `BlendTreeSystem`이 다음 프레임에 클립을 갱신한다.
    pub fn set_param(&mut self, param: f32) {
        self.param = param;
    }

    /// 현재 param에 따라 선택해야 할 클립 인덱스를 반환한다.
    /// entries가 비어 있으면 `None`.
    pub fn target_clip(&self) -> Option<usize> {
        if self.entries.is_empty() {
            return None;
        }
        // param ≥ threshold인 항목 중 threshold가 가장 큰 것을 선택
        let mut result = &self.entries[0];
        for entry in &self.entries {
            if entry.threshold <= self.param {
                result = entry;
            }
        }
        Some(result.clip_index)
    }
}
