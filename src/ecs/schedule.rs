/// 시스템 라벨 타입. 순서·그룹 식별에 사용된다.
pub type SystemLabel = &'static str;

/// `add_system_labeled`에 넘기는 순서/그룹 설정. 빌더 패턴.
#[derive(Default, Clone)]
pub struct SystemConfig {
    pub(crate) label: Option<SystemLabel>,
    pub(crate) before: Vec<SystemLabel>,
    pub(crate) after: Vec<SystemLabel>,
    pub(crate) set: Option<SystemLabel>,
}

impl SystemConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// 이 시스템에 라벨을 붙인다. 다른 시스템이 before/after로 참조할 수 있다.
    pub fn label(mut self, l: SystemLabel) -> Self {
        self.label = Some(l);
        self
    }

    /// 지정한 라벨을 가진 시스템보다 **이전에** 실행되도록 요청한다.
    pub fn before(mut self, l: SystemLabel) -> Self {
        self.before.push(l);
        self
    }

    /// 지정한 라벨을 가진 시스템 **이후에** 실행되도록 요청한다.
    pub fn after(mut self, l: SystemLabel) -> Self {
        self.after.push(l);
        self
    }

    /// 이 시스템을 특정 SystemSet에 배치한다. 해당 set이 비활성화되면 실행을 건너뛴다.
    pub fn in_set(mut self, s: SystemLabel) -> Self {
        self.set = Some(s);
        self
    }
}

/// 시스템 인덱스별 메타데이터 (app.rs가 systems와 평행 보관).
#[derive(Default, Clone)]
pub struct SystemMeta {
    pub label: Option<SystemLabel>,
    pub before: Vec<SystemLabel>,
    pub after: Vec<SystemLabel>,
    pub set: Option<SystemLabel>,
}

impl From<SystemConfig> for SystemMeta {
    fn from(c: SystemConfig) -> Self {
        Self {
            label: c.label,
            before: c.before,
            after: c.after,
            set: c.set,
        }
    }
}

/// 스케줄 계산 오류.
#[derive(Debug, PartialEq)]
pub enum ScheduleError {
    /// 순환 의존성. 사이클에 포함된 시스템 인덱스들.
    Cycle(Vec<usize>),
}

/// 위상정렬로 실행 순서를 계산한다.
///
/// - 입력: 각 시스템의 메타데이터 (인덱스 순서 = 삽입 순서)
/// - 엣지: `after(X)` → "X 라벨을 가진 모든 시스템"이 self보다 먼저.
///   `before(Y)` → self가 "Y 라벨 시스템"보다 먼저.
/// - 동순위 타이브레이커는 삽입 순서(인덱스 오름차순)로 결정적.
/// - 성공: `Ok(실행할 인덱스 순서)`. 순환: `Err(Cycle(남은 인덱스))`.
pub fn compute_order(metas: &[SystemMeta]) -> Result<Vec<usize>, ScheduleError> {
    use std::collections::HashMap;

    let n = metas.len();

    // 라벨 → 그 라벨을 가진 인덱스들
    let mut by_label: HashMap<SystemLabel, Vec<usize>> = HashMap::new();
    for (i, m) in metas.iter().enumerate() {
        if let Some(l) = m.label {
            by_label.entry(l).or_default().push(i);
        }
    }

    // 엣지 집합 (from → to). 중복 방지 위해 HashSet.
    let mut edges: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();

    for (i, m) in metas.iter().enumerate() {
        // after(a): a 라벨들이 i보다 먼저
        for a in &m.after {
            if let Some(srcs) = by_label.get(a) {
                for &s in srcs {
                    if s != i {
                        edges.insert((s, i));
                    }
                }
            }
        }
        // before(b): i가 b 라벨들보다 먼저
        for b in &m.before {
            if let Some(dsts) = by_label.get(b) {
                for &d in dsts {
                    if d != i {
                        edges.insert((i, d));
                    }
                }
            }
        }
    }

    // 진입차수
    let mut indeg = vec![0usize; n];
    for &(_, to) in &edges {
        indeg[to] += 1;
    }

    // Kahn 알고리즘 — 결정적: 진입차수 0인 것 중 인덱스 가장 작은 것부터
    let mut order = Vec::with_capacity(n);
    let mut available: Vec<usize> = (0..n).filter(|&i| indeg[i] == 0).collect();

    while let Some(&next) = available.iter().min() {
        available.retain(|&x| x != next);
        order.push(next);
        for &(from, to) in &edges {
            if from == next {
                indeg[to] -= 1;
                if indeg[to] == 0 {
                    available.push(to);
                }
            }
        }
    }

    if order.len() != n {
        let remaining: Vec<usize> = (0..n).filter(|i| !order.contains(i)).collect();
        return Err(ScheduleError::Cycle(remaining));
    }

    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta_default() -> SystemMeta {
        SystemMeta::default()
    }

    fn meta_label(label: &'static str) -> SystemMeta {
        SystemMeta {
            label: Some(label),
            ..Default::default()
        }
    }

    fn meta_label_after(label: &'static str, after: &'static str) -> SystemMeta {
        SystemMeta {
            label: Some(label),
            after: vec![after],
            ..Default::default()
        }
    }

    fn meta_label_before(label: &'static str, before: &'static str) -> SystemMeta {
        SystemMeta {
            label: Some(label),
            before: vec![before],
            ..Default::default()
        }
    }

    fn meta_after(after: &'static str) -> SystemMeta {
        SystemMeta {
            after: vec![after],
            ..Default::default()
        }
    }

    /// 1. 제약 없는 시스템 3개 → 삽입 순서 유지
    #[test]
    fn no_constraints_keeps_insertion_order() {
        let metas = vec![meta_default(), meta_default(), meta_default()];
        let order = compute_order(&metas).unwrap();
        assert_eq!(order, vec![0, 1, 2]);
    }

    /// 2. after 제약 — sys1(label "b", after "a"), sys0(label "a") → 0이 1보다 먼저
    #[test]
    fn after_orders_correctly() {
        // 인덱스 0: label "a"
        // 인덱스 1: label "b", after "a"
        let metas = vec![meta_label("a"), meta_label_after("b", "a")];
        let order = compute_order(&metas).unwrap();
        let pos0 = order.iter().position(|&x| x == 0).unwrap();
        let pos1 = order.iter().position(|&x| x == 1).unwrap();
        assert!(
            pos0 < pos1,
            "label 'a'(idx 0)이 label 'b'(idx 1)보다 먼저여야 함"
        );
    }

    /// 3. before 제약 — sys0(label "a", before "b"), sys1(label "b") → 0이 1보다 먼저
    #[test]
    fn before_orders_correctly() {
        // 인덱스 0: label "a", before "b"
        // 인덱스 1: label "b"
        let metas = vec![meta_label_before("a", "b"), meta_label("b")];
        let order = compute_order(&metas).unwrap();
        let pos0 = order.iter().position(|&x| x == 0).unwrap();
        let pos1 = order.iter().position(|&x| x == 1).unwrap();
        assert!(
            pos0 < pos1,
            "label 'a'(idx 0)이 label 'b'(idx 1)보다 먼저여야 함"
        );
    }

    /// 4. 순환 의존성 감지
    #[test]
    fn cycle_detected() {
        // 인덱스 0: label "a", after "b"
        // 인덱스 1: label "b", after "a"
        let metas = vec![meta_label_after("a", "b"), meta_label_after("b", "a")];
        let result = compute_order(&metas);
        assert!(
            matches!(result, Err(ScheduleError::Cycle(_))),
            "순환 의존성은 Err(Cycle(..))을 반환해야 함"
        );
    }

    /// 5. 공유 라벨 배리어 — 두 시스템이 label "render", 다른 시스템이 after "render"
    #[test]
    fn shared_label_barrier() {
        // 인덱스 0: label "render"
        // 인덱스 1: label "render"
        // 인덱스 2: after "render" (두 render 시스템 모두 2보다 먼저여야 함)
        let metas = vec![
            meta_label("render"),
            meta_label("render"),
            meta_after("render"),
        ];
        let order = compute_order(&metas).unwrap();
        let pos0 = order.iter().position(|&x| x == 0).unwrap();
        let pos1 = order.iter().position(|&x| x == 1).unwrap();
        let pos2 = order.iter().position(|&x| x == 2).unwrap();
        assert!(
            pos0 < pos2,
            "render(idx 0)이 after_render(idx 2)보다 먼저여야 함"
        );
        assert!(
            pos1 < pos2,
            "render(idx 1)이 after_render(idx 2)보다 먼저여야 함"
        );
    }
}
