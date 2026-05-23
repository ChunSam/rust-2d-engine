use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

use rodio::source::SineWave;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

/// 오디오 재생 관리자 (ECS 리소스로 삽입)
///
/// # 설계 이유
/// `_stream` 필드: `OutputStream`이 drop되면 모든 Sink의 소리가 즉시 멈춘다.
/// 사용하지 않지만 반드시 `AudioManager`와 같이 살아있어야 하므로 `_` 접두사로 소유만 한다.
pub struct AudioManager {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sinks: HashMap<String, Sink>,
}

impl AudioManager {
    /// 오디오 장치를 초기화한다.
    ///
    /// 오디오 장치가 없거나 초기화에 실패하면 `None`을 반환한다.
    /// 게임은 오디오 없이도 계속 실행된다.
    pub fn new() -> Option<Self> {
        match OutputStream::try_default() {
            Ok((_stream, stream_handle)) => Some(Self {
                _stream,
                stream_handle,
                sinks: HashMap::new(),
            }),
            Err(e) => {
                log::warn!("오디오 초기화 실패 (오디오 없이 실행됩니다): {e}");
                None
            }
        }
    }

    /// 오디오 파일을 `channel`에서 재생한다.
    ///
    /// 같은 채널에 이미 재생 중인 소리가 있으면 먼저 정지한다.
    /// - `channel`: 채널 이름 (예: `"bgm"`, `"sfx_jump"`)
    /// - `path`: WAV 파일 경로
    /// - `repeat`: `true`이면 무한 반복
    pub fn play(&mut self, channel: &str, path: &str, repeat: bool) {
        // 같은 채널 정지
        if let Some(old) = self.sinks.remove(channel) {
            old.stop();
        }

        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                log::warn!("오디오 파일을 열 수 없습니다 '{path}': {e}");
                return;
            }
        };

        let sink = match Sink::try_new(&self.stream_handle) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("오디오 싱크 생성 실패: {e}");
                return;
            }
        };

        let source = match Decoder::new(BufReader::new(file)) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("오디오 디코딩 실패 '{path}': {e}");
                return;
            }
        };

        if repeat {
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }

        self.sinks.insert(channel.to_string(), sink);
    }

    /// 채널 재생을 정지한다.
    pub fn stop(&mut self, channel: &str) {
        if let Some(sink) = self.sinks.remove(channel) {
            sink.stop();
        }
    }

    /// 순수 사인파 톤을 재생한다. 오디오 파일 없이 간단한 SFX 생성 용도.
    ///
    /// - `channel`: 채널 이름 (같은 채널이면 이전 소리를 중단)
    /// - `freq`: 주파수 (Hz)
    /// - `duration_secs`: 재생 시간
    /// - `volume`: 음량 배율 (0.0 ~ 1.0)
    pub fn play_tone(&mut self, channel: &str, freq: f32, duration_secs: f32, volume: f32) {
        if let Some(old) = self.sinks.remove(channel) {
            old.stop();
        }
        let sink = match Sink::try_new(&self.stream_handle) {
            Ok(s) => s,
            Err(_) => return,
        };
        let source = SineWave::new(freq)
            .take_duration(Duration::from_secs_f32(duration_secs))
            .amplify(volume);
        sink.append(source);
        self.sinks.insert(channel.to_string(), sink);
    }

    /// 채널 볼륨을 설정한다 (0.0 = 무음, 1.0 = 원본).
    pub fn set_volume(&mut self, channel: &str, volume: f32) {
        if let Some(sink) = self.sinks.get(channel) {
            sink.set_volume(volume);
        }
    }
}
