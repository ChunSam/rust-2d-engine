use crate::ecs::{events::Events, system::System, world::World};

pub const DEFAULT_MAX_MESSAGE_BYTES: usize = 64 * 1024;
/// 기본 송신 큐 최대 메시지 수. 큐가 가득 차면 새 메시지를 드롭하고 warn 로그를 남긴다.
pub const DEFAULT_MAX_PENDING_MESSAGES: usize = 256;

/// 매 프레임 [`NetworkSystem`]이 생성하는 ECS 이벤트.
#[derive(Clone, Debug)]
pub enum NetworkEvent {
    Connected,
    Disconnected { reason: String },
    BinaryMessage(Vec<u8>),
    TextMessage(String),
    MessageTooLarge { len: usize, limit: usize },
    JsonParseError { message: String },
    Error(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetworkConfig {
    pub max_message_bytes: usize,
    /// 송신 큐 최대 메시지 수. 초과 시 `send_text`/`send_bytes`는 메시지를 드롭한다.
    pub max_pending_messages: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_message_bytes: DEFAULT_MAX_MESSAGE_BYTES,
            max_pending_messages: DEFAULT_MAX_PENDING_MESSAGES,
        }
    }
}

// ── 네이티브 구현 ────────────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::{NetworkConfig, NetworkEvent};

    enum OutMsg {
        Binary(Vec<u8>),
        Text(String),
        Close,
    }

    pub struct NetworkClient {
        event_rx: std::sync::mpsc::Receiver<NetworkEvent>,
        msg_tx: std::sync::mpsc::SyncSender<OutMsg>,
    }

    impl NetworkClient {
        /// 백그라운드 스레드에서 WebSocket 연결을 시작한다.
        /// 연결 성공 시 [`NetworkEvent::Connected`], 실패 시 [`NetworkEvent::Error`]가 발행된다.
        pub fn connect(url: &str) -> Self {
            Self::connect_with_config(url, NetworkConfig::default())
        }

        pub fn connect_with_config(url: &str, config: NetworkConfig) -> Self {
            let (event_tx, event_rx) = std::sync::mpsc::channel::<NetworkEvent>();
            let (msg_tx, msg_rx) =
                std::sync::mpsc::sync_channel::<OutMsg>(config.max_pending_messages);
            let url = url.to_string();
            let max_message_bytes = config.max_message_bytes;

            std::thread::spawn(move || {
                let (mut socket, _) = match tungstenite::connect(&url) {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = event_tx.send(NetworkEvent::Error(format!("connect failed: {e}")));
                        return;
                    }
                };

                // 5 ms read timeout → loop가 5 ms마다 발신 채널을 확인한다.
                // Plain TCP와 rustls TLS 양쪽 모두 내부 TcpStream에 직접 설정한다.
                const READ_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(5);
                let stream = socket.get_mut();
                if let tungstenite::stream::MaybeTlsStream::Plain(tcp) = stream {
                    tcp.set_read_timeout(Some(READ_TIMEOUT)).ok();
                } else if let tungstenite::stream::MaybeTlsStream::Rustls(tls) = stream {
                    // rustls::StreamOwned.sock 은 pub 필드 (rustls 0.22+)
                    // wss:// 연결에서도 5 ms 주기로 발신 채널을 확인할 수 있게 한다.
                    tls.sock.set_read_timeout(Some(READ_TIMEOUT)).ok();
                }

                let _ = event_tx.send(NetworkEvent::Connected);

                loop {
                    // 발신 메시지 처리
                    loop {
                        match msg_rx.try_recv() {
                            Ok(OutMsg::Binary(data)) => {
                                if socket.send(tungstenite::Message::Binary(data)).is_err() {
                                    return;
                                }
                            }
                            Ok(OutMsg::Text(text)) => {
                                if socket.send(tungstenite::Message::Text(text)).is_err() {
                                    return;
                                }
                            }
                            Ok(OutMsg::Close) => {
                                socket.close(None).ok();
                                let _ = event_tx.send(NetworkEvent::Disconnected {
                                    reason: "local close".into(),
                                });
                                return;
                            }
                            Err(std::sync::mpsc::TryRecvError::Empty) => break,
                            Err(std::sync::mpsc::TryRecvError::Disconnected) => return,
                        }
                    }

                    // 수신 메시지 처리 (timeout 시 WouldBlock / TimedOut)
                    match socket.read() {
                        Ok(tungstenite::Message::Binary(data)) => {
                            if data.len() > max_message_bytes {
                                let _ = event_tx.send(NetworkEvent::MessageTooLarge {
                                    len: data.len(),
                                    limit: max_message_bytes,
                                });
                                continue;
                            }
                            if event_tx.send(NetworkEvent::BinaryMessage(data)).is_err() {
                                return;
                            }
                        }
                        Ok(tungstenite::Message::Text(text)) => {
                            if text.len() > max_message_bytes {
                                let _ = event_tx.send(NetworkEvent::MessageTooLarge {
                                    len: text.len(),
                                    limit: max_message_bytes,
                                });
                                continue;
                            }
                            if event_tx.send(NetworkEvent::TextMessage(text)).is_err() {
                                return;
                            }
                        }
                        Ok(tungstenite::Message::Close(frame)) => {
                            let reason = frame.map(|f| f.reason.into_owned()).unwrap_or_default();
                            let _ = event_tx.send(NetworkEvent::Disconnected { reason });
                            return;
                        }
                        Ok(_) => {} // Ping / Pong / Frame — tungstenite 내부 처리
                        Err(tungstenite::Error::Io(e))
                            if e.kind() == std::io::ErrorKind::WouldBlock
                                || e.kind() == std::io::ErrorKind::TimedOut =>
                        {
                            // 데이터 없음 — 루프 재시작
                        }
                        Err(e) => {
                            let _ = event_tx.send(NetworkEvent::Error(e.to_string()));
                            let _ = event_tx.send(NetworkEvent::Disconnected {
                                reason: "error".into(),
                            });
                            return;
                        }
                    }
                }
            });

            Self { event_rx, msg_tx }
        }

        pub fn send_bytes(&self, data: &[u8]) {
            if self.msg_tx.try_send(OutMsg::Binary(data.to_vec())).is_err() {
                log::warn!(
                    "network: 송신 큐 만원 — binary 메시지 드롭 ({} bytes)",
                    data.len()
                );
            }
        }

        pub fn send_text(&self, text: impl Into<String>) {
            let text = text.into();
            if self.msg_tx.try_send(OutMsg::Text(text.clone())).is_err() {
                log::warn!(
                    "network: 송신 큐 만원 — text 메시지 드롭 ({} bytes)",
                    text.len()
                );
            }
        }

        /// 송신 큐가 가득 차지 않은 경우에만 전송하고 성공 여부를 반환한다.
        pub fn try_send_bytes(&self, data: &[u8]) -> bool {
            self.msg_tx.try_send(OutMsg::Binary(data.to_vec())).is_ok()
        }

        /// 송신 큐가 가득 차지 않은 경우에만 전송하고 성공 여부를 반환한다.
        pub fn try_send_text(&self, text: impl Into<String>) -> bool {
            self.msg_tx.try_send(OutMsg::Text(text.into())).is_ok()
        }

        pub fn disconnect(&self) {
            let _ = self.msg_tx.try_send(OutMsg::Close);
        }

        pub(super) fn poll(&mut self) -> Vec<NetworkEvent> {
            let mut out = Vec::new();
            while let Ok(ev) = self.event_rx.try_recv() {
                out.push(ev);
            }
            out
        }
    }
}

// ── WASM 구현 ────────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
mod wasm_impl {
    use super::{NetworkConfig, NetworkEvent};
    use std::{cell::RefCell, rc::Rc};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    pub struct NetworkClient {
        socket: Option<web_sys::WebSocket>,
        buffer: Rc<RefCell<Vec<NetworkEvent>>>,
        // 클로저를 살아있게 유지
        _on_open: Option<Closure<dyn FnMut()>>,
        _on_message: Option<Closure<dyn FnMut(web_sys::MessageEvent)>>,
        _on_error: Option<Closure<dyn FnMut(web_sys::Event)>>,
        _on_close: Option<Closure<dyn FnMut(web_sys::CloseEvent)>>,
    }

    impl NetworkClient {
        pub fn connect(url: &str) -> Self {
            Self::connect_with_config(url, NetworkConfig::default())
        }

        pub fn connect_with_config(url: &str, config: NetworkConfig) -> Self {
            let buffer: Rc<RefCell<Vec<NetworkEvent>>> = Rc::new(RefCell::new(Vec::new()));
            let max_message_bytes = config.max_message_bytes;

            let ws = match web_sys::WebSocket::new(url) {
                Ok(ws) => ws,
                Err(e) => {
                    buffer.borrow_mut().push(NetworkEvent::Error(format!(
                        "WebSocket::new failed: {}",
                        js_value_to_string(&e)
                    )));
                    return Self {
                        socket: None,
                        buffer,
                        _on_open: None,
                        _on_message: None,
                        _on_error: None,
                        _on_close: None,
                    };
                }
            };
            ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

            let buf = buffer.clone();
            let on_open = Closure::<dyn FnMut()>::new(move || {
                buf.borrow_mut().push(NetworkEvent::Connected);
            });
            ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));

            let buf = buffer.clone();
            let on_message = Closure::<dyn FnMut(web_sys::MessageEvent)>::new(
                move |ev: web_sys::MessageEvent| {
                    let data = ev.data();
                    if let Some(text) = data.as_string() {
                        if text.len() > max_message_bytes {
                            buf.borrow_mut().push(NetworkEvent::MessageTooLarge {
                                len: text.len(),
                                limit: max_message_bytes,
                            });
                        } else {
                            buf.borrow_mut().push(NetworkEvent::TextMessage(text));
                        }
                    } else {
                        let array = js_sys::Uint8Array::new(&data);
                        let bytes = array.to_vec();
                        if bytes.len() > max_message_bytes {
                            buf.borrow_mut().push(NetworkEvent::MessageTooLarge {
                                len: bytes.len(),
                                limit: max_message_bytes,
                            });
                        } else {
                            buf.borrow_mut().push(NetworkEvent::BinaryMessage(bytes));
                        }
                    }
                },
            );
            ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

            let buf = buffer.clone();
            let on_error = Closure::<dyn FnMut(web_sys::Event)>::new(move |_ev: web_sys::Event| {
                buf.borrow_mut()
                    .push(NetworkEvent::Error("WebSocket error".into()));
            });
            ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));

            let buf = buffer.clone();
            let on_close =
                Closure::<dyn FnMut(web_sys::CloseEvent)>::new(move |ev: web_sys::CloseEvent| {
                    buf.borrow_mut().push(NetworkEvent::Disconnected {
                        reason: ev.reason(),
                    });
                });
            ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));

            Self {
                socket: Some(ws),
                buffer,
                _on_open: Some(on_open),
                _on_message: Some(on_message),
                _on_error: Some(on_error),
                _on_close: Some(on_close),
            }
        }

        pub fn send_bytes(&self, data: &[u8]) {
            if !self.try_send_bytes(data) {
                log::warn!("network: binary 메시지 전송 실패 ({} bytes)", data.len());
            }
        }

        pub fn send_text(&self, text: impl Into<String>) {
            let text = text.into();
            if !self.try_send_text(text.clone()) {
                log::warn!("network: text 메시지 전송 실패 ({} bytes)", text.len());
            }
        }

        pub fn try_send_bytes(&self, data: &[u8]) -> bool {
            match &self.socket {
                Some(socket) => socket.send_with_u8_array(data).is_ok(),
                None => false,
            }
        }

        pub fn try_send_text(&self, text: impl Into<String>) -> bool {
            let text = text.into();
            match &self.socket {
                Some(socket) => socket.send_with_str(&text).is_ok(),
                None => false,
            }
        }

        pub fn disconnect(&self) {
            if let Some(socket) = &self.socket {
                socket.close().ok();
            }
        }

        /// `web_sys::WebSocket::OPEN(1)` 상태인지 확인
        pub fn is_connected(&self) -> bool {
            match &self.socket {
                Some(socket) => socket.ready_state() == web_sys::WebSocket::OPEN,
                None => false,
            }
        }

        pub(super) fn poll(&mut self) -> Vec<NetworkEvent> {
            std::mem::take(&mut *self.buffer.borrow_mut())
        }
    }

    fn js_value_to_string(value: &JsValue) -> String {
        value.as_string().unwrap_or_else(|| format!("{value:?}"))
    }
}

// ── 플랫폼별 re-export ────────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
pub use native::NetworkClient;

#[cfg(target_arch = "wasm32")]
pub use wasm_impl::NetworkClient;

// ── NetworkSystem ─────────────────────────────────────────────────────────────

/// 매 프레임 [`NetworkClient`] 수신 버퍼를 폴링해 [`Events<NetworkEvent>`]로 전달한다.
///
/// 등록 방법:
/// ```text
/// app.world.insert_resource(NetworkClient::connect("ws://localhost:9001"));
/// app.world.register_event::<NetworkEvent>();
/// app.add_system(NetworkSystem);
/// ```
pub struct NetworkSystem;

impl System for NetworkSystem {
    fn run(&mut self, world: &mut World, _dt: f32) {
        let incoming: Vec<NetworkEvent> = {
            match world.resource_mut::<NetworkClient>() {
                Some(c) => c.poll(),
                None => return,
            }
        };
        if incoming.is_empty() {
            return;
        }
        if let Some(bus) = world.resource_mut::<Events<NetworkEvent>>() {
            for ev in incoming {
                bus.send(ev);
            }
        }
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;

    #[test]
    fn network_config_defaults() {
        let cfg = NetworkConfig::default();
        assert_eq!(cfg.max_message_bytes, DEFAULT_MAX_MESSAGE_BYTES);
        assert_eq!(cfg.max_pending_messages, DEFAULT_MAX_PENDING_MESSAGES);
    }

    #[test]
    fn network_bounded_channel_drops_on_full() {
        // SyncSender with capacity 1: first send succeeds, second fails (full).
        let (tx, _rx) = std::sync::mpsc::sync_channel::<u8>(1);
        assert!(tx.try_send(1).is_ok());
        assert!(
            tx.try_send(2).is_err(),
            "queue should be full after capacity is reached"
        );
    }
}
