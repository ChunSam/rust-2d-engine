//! Phase 27 — 멀티플레이어 릴레이 서버
//!
//! ```
//! cargo run --example mp_server
//! ```
//!
//! 127.0.0.1:9001에서 WebSocket 연결을 수락하고, 한 클라이언트의 위치 메시지를
//! 나머지 모든 클라이언트에게 릴레이한다.
//!
//! # 프로토콜 (JSON 텍스트)
//!
//! | 방향 | 형식 | 의미 |
//! |------|------|------|
//! | Server → Client | `{"type":"hello","id":<N>}` | 연결 ID 할당 |
//! | Client → Server | `{"x":<f32>,"y":<f32>}` | 로컬 플레이어 위치 |
//! | Server → Others | `{"type":"pos","id":<N>,"x":<f32>,"y":<f32>}` | 원격 플레이어 위치 |
//! | Server → Client | `{"type":"bye","id":<N>}` | 플레이어 퇴장 |

use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc, Arc, Mutex,
};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tungstenite::{accept, Message};

type BroadcastMap = Arc<Mutex<HashMap<usize, mpsc::Sender<Message>>>>;
const MAX_JSON_MESSAGE_BYTES: usize = 4096;

#[derive(Debug, Deserialize)]
struct ClientPosition {
    x: f32,
    y: f32,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ServerMessage {
    #[serde(rename = "hello")]
    Hello { id: usize },
    #[serde(rename = "pos")]
    Position { id: usize, x: f32, y: f32 },
    #[serde(rename = "bye")]
    Bye { id: usize },
}

fn main() {
    let addr = "127.0.0.1:9001";
    let listener = TcpListener::bind(addr).expect("bind failed");
    println!("mp_server: listening on ws://{addr}");
    println!("  Client  → Server: {{\"x\":<f32>,\"y\":<f32>}}");
    println!("  Server → Client : {{\"type\":\"hello\",\"id\":<N>}}");
    println!("  Server → Others : {{\"type\":\"pos\",\"id\":<N>,\"x\":<f32>,\"y\":<f32>}}");
    println!("  Server → Client : {{\"type\":\"bye\",\"id\":<N>}}");
    println!();

    let clients: BroadcastMap = Arc::new(Mutex::new(HashMap::new()));
    let next_id = Arc::new(AtomicUsize::new(1));

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("accept error: {e}");
                continue;
            }
        };

        let clients = clients.clone();
        let next_id = next_id.clone();

        thread::spawn(move || {
            let peer = stream.peer_addr().ok();
            let mut ws = match accept(stream) {
                Ok(ws) => ws,
                Err(e) => {
                    eprintln!("WS handshake failed: {e}");
                    return;
                }
            };

            // 5 ms read timeout — 발신 큐를 주기적으로 확인하기 위한 논블로킹 루프
            ws.get_mut()
                .set_read_timeout(Some(Duration::from_millis(5)))
                .ok();

            let id = next_id.fetch_add(1, Ordering::SeqCst);
            let (tx, rx) = mpsc::channel::<Message>();
            clients.lock().unwrap().insert(id, tx);

            println!(
                "[{id}] connected from {peer:?}  (total: {})",
                clients.lock().unwrap().len()
            );

            // 클라이언트에게 할당 ID 전달
            let hello = serde_json::to_string(&ServerMessage::Hello { id })
                .expect("hello message should serialize");
            if ws.send(Message::Text(hello)).is_err() {
                cleanup(&clients, id);
                return;
            }

            'main: loop {
                // 릴레이 발신 큐 소진
                loop {
                    match rx.try_recv() {
                        Ok(msg) => {
                            if ws.send(msg).is_err() {
                                break 'main;
                            }
                        }
                        Err(mpsc::TryRecvError::Empty) => break,
                        Err(mpsc::TryRecvError::Disconnected) => break 'main,
                    }
                }

                // WebSocket 수신
                match ws.read() {
                    Ok(Message::Text(text)) => {
                        if text.len() > MAX_JSON_MESSAGE_BYTES {
                            eprintln!(
                                "[{id}] dropped oversized message: {} > {} bytes",
                                text.len(),
                                MAX_JSON_MESSAGE_BYTES
                            );
                            continue;
                        }
                        let pos = match serde_json::from_str::<ClientPosition>(&text) {
                            Ok(pos) => pos,
                            Err(err) => {
                                eprintln!("[{id}] invalid JSON message: {err}");
                                continue;
                            }
                        };
                        let relay = serde_json::to_string(&ServerMessage::Position {
                            id,
                            x: pos.x,
                            y: pos.y,
                        })
                        .expect("position message should serialize");
                        let relay = Message::Text(relay);
                        let guard = clients.lock().unwrap();
                        for (&cid, sender) in guard.iter() {
                            if cid != id {
                                let _ = sender.send(relay.clone());
                            }
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Ok(_) => {}
                    Err(tungstenite::Error::Io(e))
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            || e.kind() == std::io::ErrorKind::TimedOut =>
                    {
                        // read timeout — 계속 루프
                    }
                    Err(_) => break,
                }
            }

            cleanup(&clients, id);
        });
    }
}

fn cleanup(clients: &BroadcastMap, id: usize) {
    let bye = Message::Text(
        serde_json::to_string(&ServerMessage::Bye { id }).expect("bye message should serialize"),
    );
    let mut guard = clients.lock().unwrap();
    guard.remove(&id);
    for sender in guard.values() {
        let _ = sender.send(bye.clone());
    }
    println!("[{id}] disconnected  (total: {})", guard.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_client_position_json() {
        let pos: ClientPosition = serde_json::from_str(r#"{"x":12.5,"y":-3.25}"#).unwrap();
        assert_eq!(pos.x, 12.5);
        assert_eq!(pos.y, -3.25);
    }

    #[test]
    fn rejects_invalid_client_position_json() {
        let result = serde_json::from_str::<ClientPosition>(r#"{"x":"bad","y":1.0}"#);
        assert!(result.is_err());
    }

    #[test]
    fn serializes_server_messages_with_type_tags() {
        let text = serde_json::to_string(&ServerMessage::Position {
            id: 7,
            x: 1.0,
            y: 2.0,
        })
        .unwrap();
        assert_eq!(text, r#"{"type":"pos","id":7,"x":1.0,"y":2.0}"#);
    }
}
