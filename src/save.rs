use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use rand::RngCore;
use serde::{de::DeserializeOwned, Serialize};

const SAVE_MAGIC: &[u8; 9] = b"R2DAEAD01";
const NONCE_LEN: usize = 12;
const SAVE_KEY_BYTES: [u8; 32] = [
    0x52, 0x32, 0x44, 0x45, 0x2d, 0x53, 0x41, 0x56, 0x45, 0x2d, 0x41, 0x45, 0x41, 0x44, 0x2d, 0x4b,
    0x31, 0x9f, 0x6c, 0x21, 0xb8, 0x43, 0xd0, 0x75, 0xe2, 0x0a, 0x5c, 0x99, 0x13, 0xfe, 0x67, 0x2b,
];

/// AEAD key used to encrypt and authenticate save files.
///
/// A key embedded in a client binary is not a secret against a determined user. Use
/// [`SaveKey`] to separate saves between builds/users and to detect tampering, not
/// as a complete secret-protection boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveKey(pub [u8; 32]);

impl SaveKey {
    /// Built-in compatibility key used by [`save`] and [`load`].
    ///
    /// This default key is public by construction because it ships in the binary.
    pub const DEFAULT: Self = Self(SAVE_KEY_BYTES);
}

/// 저장/로드 에러 타입.
#[derive(Debug)]
pub enum SaveError {
    Io(io::Error),
    Ron(String),
    Corrupted,
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Io(e) => write!(f, "IO error: {e}"),
            SaveError::Ron(s) => write!(f, "RON error: {s}"),
            SaveError::Corrupted => write!(f, "Save file is corrupted or has been tampered with"),
        }
    }
}

impl std::error::Error for SaveError {}

impl From<io::Error> for SaveError {
    fn from(e: io::Error) -> Self {
        SaveError::Io(e)
    }
}

/// OS 표준 데이터 디렉토리 하위의 저장 파일 경로를 반환한다.
///
/// WASM에서는 `{app_name}/{file}` 상대 경로를 반환한다 (파일시스템 미지원).
pub fn save_path(app_name: &str, file: &str) -> PathBuf {
    #[cfg(not(target_arch = "wasm32"))]
    return dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(app_name)
        .join(file);
    #[cfg(target_arch = "wasm32")]
    PathBuf::from(format!("{}/{}", app_name, file))
}

/// 디렉토리를 만들고 데이터를 RON으로 직렬화한 뒤 AEAD 암호화해 저장한다.
///
/// Uses [`SaveKey::DEFAULT`] for backwards compatibility. Prefer [`save_with_key`]
/// when the application can provide its own stable key material.
pub fn save<T: Serialize>(path: &Path, data: &T) -> Result<(), SaveError> {
    save_with_key(path, data, SaveKey::DEFAULT)
}

/// 디렉토리를 만들고 데이터를 지정한 키로 AEAD 암호화해 저장한다.
pub fn save_with_key<T: Serialize>(path: &Path, data: &T, key: SaveKey) -> Result<(), SaveError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let plaintext = ron::ser::to_string_pretty(data, ron::ser::PrettyConfig::default())
        .map_err(|e| SaveError::Ron(e.to_string()))?;
    let encrypted = encrypt_save_bytes(plaintext.as_bytes(), key)?;
    fs::write(path, encrypted)?;
    Ok(())
}

/// 저장 파일을 복호화한 뒤 RON으로 역직렬화한다. 파일 없으면 Err(SaveError::Io(NotFound)).
///
/// Uses [`SaveKey::DEFAULT`] for backwards compatibility. Prefer [`load_with_key`]
/// when loading saves written with [`save_with_key`].
pub fn load<T: DeserializeOwned>(path: &Path) -> Result<T, SaveError> {
    load_with_key(path, SaveKey::DEFAULT)
}

/// 지정한 키로 저장 파일을 복호화한 뒤 RON으로 역직렬화한다.
pub fn load_with_key<T: DeserializeOwned>(path: &Path, key: SaveKey) -> Result<T, SaveError> {
    let bytes = fs::read(path)?;
    let plaintext = decrypt_save_bytes(&bytes, key)?;
    let s = std::str::from_utf8(&plaintext).map_err(|_| SaveError::Corrupted)?;
    ron::from_str(s).map_err(|e| SaveError::Ron(e.to_string()))
}

/// 파일이 있으면 복호화해 로드, 없으면 `T::default()` 반환. 복호화/파싱 에러는 그대로 전파.
pub fn load_or_default<T: DeserializeOwned + Default>(path: &Path) -> Result<T, SaveError> {
    match load(path) {
        Ok(v) => Ok(v),
        Err(SaveError::Io(e)) if e.kind() == io::ErrorKind::NotFound => Ok(T::default()),
        Err(e) => Err(e),
    }
}

/// 저장 파일이 존재하는지 확인한다.
pub fn exists(path: &Path) -> bool {
    path.exists()
}

/// 저장 파일을 삭제한다. 파일이 없으면 Ok(()).
pub fn delete(path: &Path) -> Result<(), SaveError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(SaveError::Io(e)),
    }
}

fn cipher(key: SaveKey) -> ChaCha20Poly1305 {
    ChaCha20Poly1305::new(Key::from_slice(&key.0))
}

fn encrypt_save_bytes(plaintext: &[u8], key: SaveKey) -> Result<Vec<u8>, SaveError> {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let ciphertext = cipher(key)
        .encrypt(Nonce::from_slice(&nonce_bytes), plaintext)
        .map_err(|_| SaveError::Corrupted)?;

    let mut out = Vec::with_capacity(SAVE_MAGIC.len() + NONCE_LEN + ciphertext.len());
    out.extend_from_slice(SAVE_MAGIC);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

fn decrypt_save_bytes(bytes: &[u8], key: SaveKey) -> Result<Vec<u8>, SaveError> {
    let header_len = SAVE_MAGIC.len() + NONCE_LEN;
    if bytes.len() <= header_len || !bytes.starts_with(SAVE_MAGIC) {
        return Err(SaveError::Corrupted);
    }

    let nonce = Nonce::from_slice(&bytes[SAVE_MAGIC.len()..header_len]);
    cipher(key)
        .decrypt(nonce, &bytes[header_len..])
        .map_err(|_| SaveError::Corrupted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Settings {
        sfx: f32,
        music: f32,
        hi_score: u32,
    }

    fn unique_test_dir() -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "rust-gameengine-save-test-{}-{}",
            std::process::id(),
            id
        ))
    }

    #[test]
    fn save_load_roundtrip() {
        let dir = unique_test_dir();
        let path = dir.join("settings.ron");

        let original = Settings {
            sfx: 0.8,
            music: 0.5,
            hi_score: 9999,
        };

        save(&path, &original).expect("save should succeed");
        let raw = fs::read(&path).expect("saved file should exist");
        assert!(
            !String::from_utf8_lossy(&raw).contains("hi_score"),
            "saved file should not contain plaintext RON fields"
        );

        let loaded: Settings = load(&path).expect("load should succeed");

        assert_eq!(original, loaded);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_missing_file_returns_io_error() {
        let result: Result<Settings, SaveError> = load(Path::new("/nonexistent/path/foo.ron"));

        assert!(
            matches!(result, Err(SaveError::Io(_))),
            "expected SaveError::Io, got {result:?}"
        );
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
    struct Counter {
        value: u32,
    }

    #[test]
    fn load_or_default_returns_default_when_missing() {
        let path = PathBuf::from("/nonexistent/path/counter.ron");
        let result: Result<Counter, SaveError> = load_or_default(&path);
        assert_eq!(result.unwrap(), Counter::default());
    }

    #[test]
    fn load_or_default_returns_saved_value() {
        let dir = unique_test_dir();
        let path = dir.join("counter.ron");
        let data = Counter { value: 42 };
        save(&path, &data).unwrap();
        let loaded: Counter = load_or_default(&path).unwrap();
        assert_eq!(loaded, data);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn tampered_file_returns_corrupted() {
        let dir = unique_test_dir();
        let path = dir.join("settings.ron");
        let data = Settings {
            sfx: 1.0,
            music: 0.5,
            hi_score: 7,
        };

        save(&path, &data).unwrap();
        let mut raw = fs::read(&path).unwrap();
        let last = raw.len() - 1;
        raw[last] ^= 0x01;
        fs::write(&path, raw).unwrap();

        let loaded: Result<Settings, SaveError> = load(&path);
        assert!(
            matches!(loaded, Err(SaveError::Corrupted)),
            "expected SaveError::Corrupted, got {loaded:?}"
        );

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_load_with_key_roundtrip_and_wrong_key_fails() {
        let dir = unique_test_dir();
        let path = dir.join("keyed-settings.ron");
        let data = Settings {
            sfx: 0.2,
            music: 0.9,
            hi_score: 123,
        };
        let key = SaveKey([7; 32]);
        let wrong_key = SaveKey([8; 32]);

        save_with_key(&path, &data, key).unwrap();
        let loaded: Settings = load_with_key(&path, key).unwrap();
        assert_eq!(loaded, data);

        let wrong: Result<Settings, SaveError> = load_with_key(&path, wrong_key);
        assert!(
            matches!(wrong, Err(SaveError::Corrupted)),
            "expected wrong key to fail authentication, got {wrong:?}"
        );

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn exists_and_delete() {
        let dir = unique_test_dir();
        let path = dir.join("flag.ron");
        let data = Counter { value: 1 };

        assert!(!exists(&path));
        save(&path, &data).unwrap();
        assert!(exists(&path));
        delete(&path).unwrap();
        assert!(!exists(&path));
        // 이미 없는 파일 삭제 → Ok
        delete(&path).unwrap();
        fs::remove_dir_all(&dir).ok();
    }
}
