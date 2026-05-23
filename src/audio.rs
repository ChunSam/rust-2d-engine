use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::time::Duration;

use rodio::source::SineWave;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

// ─── 스테레오 팬 소스 래퍼 (내부 전용) ───────────────────────────────────────────

/// 스테레오 팬을 적용하는 Source 래퍼.
/// left_vol = (1 - pan).clamp(0,1), right_vol = (1 + pan).clamp(0,1)
struct PannedSource<S: Source<Item = f32>> {
    inner: S,
    left_vol: f32,
    right_vol: f32,
    current_channel: u16,
    total_channels: u16,
}

impl<S: Source<Item = f32>> PannedSource<S> {
    fn new(inner: S, pan: f32) -> Self {
        let total_channels = inner.channels();
        Self {
            left_vol: (1.0 - pan).clamp(0.0, 1.0),
            right_vol: (1.0 + pan).clamp(0.0, 1.0),
            inner,
            current_channel: 0,
            total_channels,
        }
    }
}

impl<S: Source<Item = f32> + Clone> Clone for PannedSource<S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            left_vol: self.left_vol,
            right_vol: self.right_vol,
            current_channel: self.current_channel,
            total_channels: self.total_channels,
        }
    }
}

impl<S: Source<Item = f32>> Iterator for PannedSource<S> {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let sample = self.inner.next()?;
        let channels = self.total_channels;
        let vol = if channels < 2 {
            (self.left_vol + self.right_vol) * 0.5
        } else if self.current_channel == 0 {
            self.left_vol
        } else {
            self.right_vol
        };
        self.current_channel = (self.current_channel + 1) % channels.max(1);
        Some(sample * vol)
    }
}

impl<S: Source<Item = f32>> Source for PannedSource<S> {
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }
    fn channels(&self) -> u16 {
        self.inner.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }
    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}

/// 오디오 재생 관리자 (ECS 리소스로 삽입)
///
/// # 설계 이유
/// `_stream` 필드: `OutputStream`이 drop되면 모든 Sink의 소리가 즉시 멈춘다.
/// 사용하지 않지만 반드시 `AudioManager`와 같이 살아있어야 하므로 `_` 접두사로 소유만 한다.
pub struct AudioManager {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sinks: HashMap<String, Sink>,
    /// 채널별 볼륨 오버라이드. `play()` 시 자동 적용된다.
    volume_overrides: HashMap<String, f32>,
    /// 채널별 스테레오 팬 (-1.0 = 좌, 0.0 = 중앙, 1.0 = 우).
    pans: HashMap<String, f32>,
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
                volume_overrides: HashMap::new(),
                pans: HashMap::new(),
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
        if let Some(old) = self.sinks.remove(channel) {
            old.stop();
        }

        let sink = match Sink::try_new(&self.stream_handle) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("오디오 싱크 생성 실패: {e}");
                return;
            }
        };

        if let Some(&vol) = self.volume_overrides.get(channel) {
            sink.set_volume(vol);
        }

        let pan = self.pans.get(channel).copied().unwrap_or(0.0);

        if pan.abs() > 0.001 {
            // pan 적용 시 메모리 버퍼링 (Clone 필요)
            let bytes = match std::fs::read(path) {
                Ok(b) => b,
                Err(e) => {
                    log::warn!("오디오 파일을 열 수 없습니다 '{path}': {e}");
                    return;
                }
            };
            let source = match Decoder::new(Cursor::new(bytes)) {
                Ok(s) => s,
                Err(e) => {
                    log::warn!("오디오 디코딩 실패 '{path}': {e}");
                    return;
                }
            };
            let panned = PannedSource::new(source.convert_samples::<f32>(), pan);
            if repeat {
                sink.append(panned.repeat_infinite());
            } else {
                sink.append(panned);
            }
        } else {
            let file = match File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    log::warn!("오디오 파일을 열 수 없습니다 '{path}': {e}");
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
    /// 현재 재생 중인 싱크에 즉시 적용되며, 이후 `play()` 호출에도 유지된다.
    pub fn set_volume(&mut self, channel: &str, volume: f32) {
        self.volume_overrides.insert(channel.to_string(), volume);
        if let Some(sink) = self.sinks.get(channel) {
            sink.set_volume(volume);
        }
    }

    /// 채널 스테레오 팬을 설정한다 (-1.0 = 좌, 0.0 = 중앙, 1.0 = 우).
    /// 다음 `play()` 호출부터 적용된다.
    pub fn set_pan(&mut self, channel: &str, pan: f32) {
        self.pans.insert(channel.to_string(), pan.clamp(-1.0, 1.0));
    }
}
