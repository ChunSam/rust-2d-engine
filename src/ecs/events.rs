/// 프레임 경계 이벤트 버스.
///
/// `App::register_event::<E>()` 로 등록하면 World 리소스로 삽입된다.
/// 시스템은 `world.resource_mut::<Events<E>>().send(e)` 로 이벤트를 보내고,
/// 같은 프레임의 이후 시스템(또는 다음 프레임)에서 `world.resource::<Events<E>>().read()` 로 읽는다.
/// 매 프레임 종료 시 App이 자동으로 `flush()` 를 호출해 큐를 비운다.
pub struct Events<E: 'static> {
    items: Vec<E>,
}

impl<E: 'static> Default for Events<E> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

impl<E: 'static> Events<E> {
    /// 이벤트를 현재 프레임 큐에 추가한다.
    pub fn send(&mut self, event: E) {
        self.items.push(event);
    }

    /// 현재 프레임의 이벤트 슬라이스를 반환한다.
    ///
    /// 이 슬라이스는 `flush()` 가 호출될 때까지 (= 프레임 종료 시까지) 유효하다.
    pub fn read(&self) -> &[E] {
        &self.items
    }

    /// 프레임 종료 시 App이 호출한다. 외부에서 직접 호출할 필요는 없다.
    pub fn flush(&mut self) {
        self.items.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_and_read() {
        let mut events: Events<u32> = Events::default();
        assert!(events.read().is_empty());

        events.send(1);
        events.send(2);
        assert_eq!(events.read(), &[1, 2]);
    }

    #[test]
    fn flush_clears_queue() {
        let mut events: Events<u32> = Events::default();
        events.send(42);
        events.flush();
        assert!(events.read().is_empty());
    }
}
