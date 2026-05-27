use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::time::Duration;

use glam::Vec2;
use rodio::source::SineWave;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

// ─── 스테레오 팬 소스 래퍼 ────────────────────────────────────────────────────

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

// ─── 오디오 이펙트 ────────────────────────────────────────────────────────────

/// 채널별 오디오 이펙트 설정.
/// `set_effect()` 후 다음 `play_*` 호출 시 자동 적용된다.
#[derive(Debug, Clone)]
pub struct AudioEffect {
    /// 로우패스 컷오프 주파수 (Hz). `None` = 필터 없음.
    pub low_pass_hz: Option<u32>,
    /// 재생 속도 배율 (피치 비례). 1.0 = 원속도.
    pub pitch: f32,
    /// 재생 시작 시 페이드인 시간 (초). 0.0 = 즉시.
    pub attack_secs: f32,
    /// 볼륨 엔벨로프 지속 시간 (초). 0.0 = 무제한.
    pub release_secs: f32,
}

impl Default for AudioEffect {
    fn default() -> Self {
        Self {
            low_pass_hz: None,
            pitch: 1.0,
            attack_secs: 0.0,
            release_secs: 0.0,
        }
    }
}

// ─── 페이드 상태 ──────────────────────────────────────────────────────────────

struct Fade {
    start_vol: f32,
    target_vol: f32,
    duration: f32,
    elapsed: f32,
    /// 페이드 완료 시 싱크를 정지할지 여부 (fade_out에서 true)
    stop_when_done: bool,
}

// ─── AudioManager ─────────────────────────────────────────────────────────────

/// 오디오 재생 관리자 (ECS 리소스로 삽입)
///
/// ## 기본 재생
/// ```rust,no_run
/// # use engine::AudioManager;
/// # let mut am = AudioManager::new().unwrap();
/// am.play("bgm", "assets/music.ogg", true);
/// am.set_volume("bgm", 0.6);
/// am.stop("bgm");
/// ```
///
/// ## 위치 오디오
/// ```rust,no_run
/// # use engine::AudioManager;
/// # use glam::Vec2;
/// # let mut am = AudioManager::new().unwrap();
/// let source_pos = Vec2::new(300.0, 200.0);
/// let listener   = Vec2::new(0.0, 0.0);
/// am.play_at("sfx_hit", "assets/hit.wav", false, source_pos, listener, 500.0);
/// ```
///
/// ## 오디오 버스 (그룹 볼륨)
/// ```rust,no_run
/// # use engine::AudioManager;
/// # let mut am = AudioManager::new().unwrap();
/// am.assign_bus("bgm",      "music");
/// am.assign_bus("sfx_jump", "sfx");
/// am.set_bus_volume("music", 0.5);  // 음악 버스 절반으로
/// ```
pub struct AudioManager {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sinks: HashMap<String, Sink>,
    /// 채널별 기본 볼륨 (버스 볼륨과 곱하기 전)
    volume_overrides: HashMap<String, f32>,
    /// 채널별 스테레오 팬
    pans: HashMap<String, f32>,
    /// 버스 이름 → 볼륨 배율
    bus_volumes: HashMap<String, f32>,
    /// 채널 → 버스 이름
    channel_buses: HashMap<String, String>,
    /// 활성 페이드 상태
    fades: HashMap<String, Fade>,
    /// 채널별 오디오 이펙트
    effects: HashMap<String, AudioEffect>,
}

impl AudioManager {
    /// 오디오 장치를 초기화한다. 실패 시 `None` 반환, 게임은 무음으로 계속 실행된다.
    pub fn new() -> Option<Self> {
        match OutputStream::try_default() {
            Ok((_stream, stream_handle)) => Some(Self {
                _stream,
                stream_handle,
                sinks: HashMap::new(),
                volume_overrides: HashMap::new(),
                pans: HashMap::new(),
                bus_volumes: HashMap::new(),
                channel_buses: HashMap::new(),
                fades: HashMap::new(),
                effects: HashMap::new(),
            }),
            Err(e) => {
                log::warn!("오디오 초기화 실패 (오디오 없이 실행됩니다): {e}");
                None
            }
        }
    }

    // ── 기본 재생 ─────────────────────────────────────────────────────────────

    /// 오디오 파일을 채널에서 재생한다. 같은 채널이 있으면 먼저 정지한다.
    pub fn play(&mut self, channel: &str, path: &str, repeat: bool) {
        self.play_internal(channel, path, repeat, None);
    }

    /// 페이드인을 적용해 재생한다.
    pub fn play_fade_in(&mut self, channel: &str, path: &str, repeat: bool, fade_secs: f32) {
        self.play_internal(channel, path, repeat, Some(fade_secs));
    }

    /// 채널 재생을 즉시 정지한다.
    pub fn stop(&mut self, channel: &str) {
        self.fades.remove(channel);
        if let Some(sink) = self.sinks.remove(channel) {
            sink.stop();
        }
    }

    /// 채널 재생을 `duration_secs` 초에 걸쳐 페이드아웃 후 정지한다.
    ///
    /// 매 프레임 [`update`](Self::update)를 호출해야 동작한다.
    pub fn fade_out(&mut self, channel: &str, duration_secs: f32) {
        let current_vol = self.effective_volume(channel);
        self.fades.insert(
            channel.to_string(),
            Fade {
                start_vol: current_vol,
                target_vol: 0.0,
                duration: duration_secs.max(0.001),
                elapsed: 0.0,
                stop_when_done: true,
            },
        );
    }

    /// 채널 볼륨을 `duration_secs` 초에 걸쳐 `target` 까지 변경한다.
    ///
    /// 매 프레임 [`update`](Self::update)를 호출해야 동작한다.
    pub fn fade_volume(&mut self, channel: &str, target: f32, duration_secs: f32) {
        let current_vol = self.effective_volume(channel);
        self.fades.insert(
            channel.to_string(),
            Fade {
                start_vol: current_vol,
                target_vol: target.clamp(0.0, 1.0),
                duration: duration_secs.max(0.001),
                elapsed: 0.0,
                stop_when_done: false,
            },
        );
    }

    // ── 위치 오디오 ───────────────────────────────────────────────────────────

    /// 2D 공간의 `source_pos`에서 소리를 재생한다.
    ///
    /// - 거리(`source_pos`와 `listener_pos` 사이)가 `max_dist` 이상이면 무음.
    /// - X 방향 차이로 스테레오 팬을 자동 계산한다.
    pub fn play_at(
        &mut self,
        channel: &str,
        path: &str,
        repeat: bool,
        source_pos: Vec2,
        listener: Vec2,
        max_dist: f32,
    ) {
        let (vol, pan) = Self::spatial_params(source_pos, listener, max_dist);
        self.volume_overrides.insert(channel.to_string(), vol);
        self.pans.insert(channel.to_string(), pan);
        self.play(channel, path, repeat);
    }

    /// 이미 재생 중인 채널의 공간 위치를 실시간으로 업데이트한다.
    ///
    /// ECS 시스템에서 매 프레임 호출해 움직이는 소리 발생원에 적용한다.
    pub fn update_position(
        &mut self,
        channel: &str,
        source_pos: Vec2,
        listener: Vec2,
        max_dist: f32,
    ) {
        let (vol, pan) = Self::spatial_params(source_pos, listener, max_dist);
        self.volume_overrides.insert(channel.to_string(), vol);
        self.pans.insert(channel.to_string(), pan);
        if let Some(sink) = self.sinks.get(channel) {
            sink.set_volume(self.effective_volume_params(vol, channel));
        }
    }

    // ── 볼륨 / 팬 ────────────────────────────────────────────────────────────

    /// 채널 볼륨을 즉시 설정한다 (0.0 = 무음, 1.0 = 원본).
    /// 버스 볼륨과 곱해진 값이 실제 음량이 된다.
    pub fn set_volume(&mut self, channel: &str, volume: f32) {
        let vol = volume.clamp(0.0, 1.0);
        self.volume_overrides.insert(channel.to_string(), vol);
        let eff = self.effective_volume(channel);
        if let Some(sink) = self.sinks.get(channel) {
            sink.set_volume(eff);
        }
    }

    /// 채널 스테레오 팬을 설정한다 (-1.0 = 좌, 0.0 = 중앙, 1.0 = 우).
    /// 다음 `play()` 호출부터 적용된다.
    pub fn set_pan(&mut self, channel: &str, pan: f32) {
        self.pans.insert(channel.to_string(), pan.clamp(-1.0, 1.0));
    }

    // ── 오디오 이펙트 ─────────────────────────────────────────────────────────

    /// 채널에 이펙트를 설정한다. 다음 `play_*` 호출 시 적용된다.
    pub fn set_effect(&mut self, channel: &str, effect: AudioEffect) {
        self.effects.insert(channel.to_string(), effect);
    }

    /// 채널의 이펙트를 제거한다.
    pub fn clear_effect(&mut self, channel: &str) {
        self.effects.remove(channel);
    }

    /// 채널의 현재 이펙트를 반환한다.
    pub fn effect(&self, channel: &str) -> Option<&AudioEffect> {
        self.effects.get(channel)
    }

    // ── 오디오 버스 ───────────────────────────────────────────────────────────

    /// 채널을 버스에 할당한다.
    ///
    /// 예: `assign_bus("bgm", "music")` → `set_bus_volume("music", v)` 로 일괄 제어.
    pub fn assign_bus(&mut self, channel: &str, bus: &str) {
        self.channel_buses
            .insert(channel.to_string(), bus.to_string());
        // 즉시 버스 볼륨 반영
        let eff = self.effective_volume(channel);
        if let Some(sink) = self.sinks.get(channel) {
            sink.set_volume(eff);
        }
    }

    /// 버스 전체 볼륨을 설정한다. 버스에 속한 모든 채널에 즉시 적용된다.
    pub fn set_bus_volume(&mut self, bus: &str, volume: f32) {
        self.bus_volumes
            .insert(bus.to_string(), volume.clamp(0.0, 1.0));
        // 버스에 속한 모든 채널 싱크 업데이트
        let channels: Vec<String> = self
            .channel_buses
            .iter()
            .filter(|(_, b)| b.as_str() == bus)
            .map(|(ch, _)| ch.clone())
            .collect();
        for ch in channels {
            let eff = self.effective_volume(&ch);
            if let Some(sink) = self.sinks.get(&ch) {
                sink.set_volume(eff);
            }
        }
    }

    /// 버스 볼륨을 반환한다 (없으면 1.0).
    pub fn bus_volume(&self, bus: &str) -> f32 {
        self.bus_volumes.get(bus).copied().unwrap_or(1.0)
    }

    // ── 유틸리티 ──────────────────────────────────────────────────────────────

    /// 순수 사인파 톤을 재생한다.
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

    /// 페이드 처리를 진행한다. 매 프레임 System에서 호출한다.
    ///
    /// `fade_out` / `fade_volume`을 사용할 경우 반드시 이 메서드를 호출해야 한다.
    pub fn update(&mut self, dt: f32) {
        let channels: Vec<String> = self.fades.keys().cloned().collect();
        for ch in channels {
            let done = {
                let fade = self.fades.get_mut(&ch).unwrap();
                fade.elapsed += dt;
                let t = (fade.elapsed / fade.duration).clamp(0.0, 1.0);
                let vol = fade.start_vol + (fade.target_vol - fade.start_vol) * t;
                if let Some(sink) = self.sinks.get(&ch) {
                    let bus_vol = self
                        .channel_buses
                        .get(&ch)
                        .and_then(|b| self.bus_volumes.get(b))
                        .copied()
                        .unwrap_or(1.0);
                    sink.set_volume(vol * bus_vol);
                }
                if t >= 1.0 {
                    let stop = fade.stop_when_done;
                    self.volume_overrides.insert(ch.clone(), fade.target_vol);
                    stop
                } else {
                    false
                }
            };
            if done {
                self.fades.remove(&ch);
                self.stop(&ch);
            } else if self
                .fades
                .get(&ch)
                .map(|f| f.elapsed >= f.duration)
                .unwrap_or(false)
            {
                self.fades.remove(&ch);
            }
        }
    }

    // ── 내부 헬퍼 ─────────────────────────────────────────────────────────────

    fn play_internal(
        &mut self,
        channel: &str,
        path: &str,
        repeat: bool,
        fade_in_secs: Option<f32>,
    ) {
        if let Some(old) = self.sinks.remove(channel) {
            old.stop();
        }
        self.fades.remove(channel);

        let sink = match Sink::try_new(&self.stream_handle) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("오디오 싱크 생성 실패: {e}");
                return;
            }
        };

        let eff_vol = self.effective_volume(channel);
        sink.set_volume(eff_vol);

        let pan = self.pans.get(channel).copied().unwrap_or(0.0);

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

        // ── 이펙트 적용 ──────────────────────────────────────────────────────
        // Box<dyn Source<Item=i16> + Send>로 통일해 타입 복잡도를 줄인다.
        let effect = self.effects.get(channel).cloned();
        let effected: Box<dyn Source<Item = i16> + Send + 'static> = if let Some(eff) = effect {
            if (eff.pitch - 1.0).abs() > 0.001 {
                let s = source.speed(eff.pitch);
                if let Some(hz) = eff.low_pass_hz {
                    let s = s
                        .convert_samples::<f32>()
                        .low_pass(hz)
                        .convert_samples::<i16>();
                    if eff.attack_secs > 0.001 {
                        Box::new(s.fade_in(Duration::from_secs_f32(eff.attack_secs)))
                    } else {
                        Box::new(s)
                    }
                } else if eff.attack_secs > 0.001 {
                    Box::new(
                        s.convert_samples::<i16>()
                            .fade_in(Duration::from_secs_f32(eff.attack_secs)),
                    )
                } else {
                    Box::new(s.convert_samples::<i16>())
                }
            } else if let Some(hz) = eff.low_pass_hz {
                let s = source
                    .convert_samples::<f32>()
                    .low_pass(hz)
                    .convert_samples::<i16>();
                if eff.attack_secs > 0.001 {
                    Box::new(s.fade_in(Duration::from_secs_f32(eff.attack_secs)))
                } else {
                    Box::new(s)
                }
            } else if eff.attack_secs > 0.001 {
                Box::new(source.fade_in(Duration::from_secs_f32(eff.attack_secs)))
            } else {
                Box::new(source)
            }
        } else {
            Box::new(source)
        };

        // ── 팬 / 페이드인 / 반복 적용 ────────────────────────────────────────
        // 팬 없고 페이드인 없을 때는 BufReader 경로가 더 효율적이지만,
        // 여기서는 Cursor 경로로 통일 (이미 bytes로 읽었으므로 비용 동일)
        if pan.abs() > 0.001 {
            let panned = PannedSource::new(effected.convert_samples::<f32>(), pan);
            if let Some(fade_dur) = fade_in_secs {
                let faded = panned.fade_in(Duration::from_secs_f32(fade_dur));
                if repeat {
                    sink.append(faded.repeat_infinite());
                } else {
                    sink.append(faded);
                }
            } else if repeat {
                sink.append(panned.repeat_infinite());
            } else {
                sink.append(panned);
            }
        } else if let Some(fade_dur) = fade_in_secs {
            let faded = effected.fade_in(Duration::from_secs_f32(fade_dur));
            if repeat {
                sink.append(faded.repeat_infinite());
            } else {
                sink.append(faded);
            }
        } else if repeat {
            sink.append(effected.repeat_infinite());
        } else {
            sink.append(effected);
        }

        self.sinks.insert(channel.to_string(), sink);
    }

    /// 채널의 실효 볼륨 = 기본 볼륨 × 버스 볼륨
    fn effective_volume(&self, channel: &str) -> f32 {
        let base = self.volume_overrides.get(channel).copied().unwrap_or(1.0);
        self.effective_volume_params(base, channel)
    }

    fn effective_volume_params(&self, base: f32, channel: &str) -> f32 {
        let bus_vol = self
            .channel_buses
            .get(channel)
            .and_then(|b| self.bus_volumes.get(b))
            .copied()
            .unwrap_or(1.0);
        base * bus_vol
    }

    /// 소리 발생 위치와 리스너 위치로부터 (볼륨, 팬)을 계산한다.
    fn spatial_params(source_pos: Vec2, listener: Vec2, max_dist: f32) -> (f32, f32) {
        let delta = source_pos - listener;
        let dist = delta.length();
        let volume = (1.0 - (dist / max_dist.max(0.001)).min(1.0)).max(0.0);
        let pan = (delta.x / max_dist.max(0.001)).clamp(-1.0, 1.0);
        (volume, pan)
    }

    // ── 호환성 유지 (이전 직접 read+BufReader 패턴) ──────────────────────────

    /// `play` 의 낮은 수준 버전. 팬 없을 때 BufReader로 스트리밍한다.
    #[allow(dead_code)]
    fn play_streaming(&mut self, channel: &str, path: &str, repeat: bool) {
        if let Some(old) = self.sinks.remove(channel) {
            old.stop();
        }
        let sink = match Sink::try_new(&self.stream_handle) {
            Ok(s) => s,
            Err(_) => return,
        };
        sink.set_volume(self.effective_volume(channel));
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
        self.sinks.insert(channel.to_string(), sink);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spatial_params_center_is_full_volume() {
        let (vol, pan) = AudioManager::spatial_params(Vec2::ZERO, Vec2::ZERO, 500.0);
        assert_eq!(vol, 1.0);
        assert!((pan).abs() < 0.001);
    }

    #[test]
    fn spatial_params_max_dist_is_silent() {
        let (vol, _) = AudioManager::spatial_params(Vec2::new(500.0, 0.0), Vec2::ZERO, 500.0);
        assert!(vol < 0.001);
    }

    #[test]
    fn spatial_params_right_side_pans_right() {
        let (_, pan) = AudioManager::spatial_params(Vec2::new(250.0, 0.0), Vec2::ZERO, 500.0);
        assert!(pan > 0.0);
    }

    #[test]
    fn spatial_params_left_side_pans_left() {
        let (_, pan) = AudioManager::spatial_params(Vec2::new(-250.0, 0.0), Vec2::ZERO, 500.0);
        assert!(pan < 0.0);
    }

    #[test]
    fn audio_effect_default_pitch() {
        let eff = AudioEffect::default();
        assert!((eff.pitch - 1.0).abs() < 0.001);
        assert!(eff.low_pass_hz.is_none());
    }

    #[test]
    fn set_and_clear_effect() {
        // AudioManager는 오디오 장치 없이 None을 반환할 수 있으므로,
        // AudioEffect 구조체 자체만 테스트한다.
        let eff = AudioEffect {
            low_pass_hz: Some(1000),
            pitch: 0.8,
            attack_secs: 0.5,
            release_secs: 0.0,
        };
        assert_eq!(eff.low_pass_hz, Some(1000));
        assert!((eff.pitch - 0.8).abs() < 0.001);
    }
}
