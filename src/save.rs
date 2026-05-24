use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{de::DeserializeOwned, Serialize};

/// 저장/로드 에러 타입.
#[derive(Debug)]
pub enum SaveError {
    Io(io::Error),
    Ron(String),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Io(e) => write!(f, "IO error: {}", e),
            SaveError::Ron(s) => write!(f, "RON error: {}", s),
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

/// 디렉토리를 만들고 데이터를 RON 파일로 직렬화해 저장한다.
pub fn save<T: Serialize>(path: &Path, data: &T) -> Result<(), SaveError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let s = ron::ser::to_string_pretty(data, ron::ser::PrettyConfig::default())
        .map_err(|e| SaveError::Ron(e.to_string()))?;
    fs::write(path, s)?;
    Ok(())
}

/// RON 파일을 읽어 역직렬화한다. 파일 없으면 Err(SaveError::Io(NotFound)).
pub fn load<T: DeserializeOwned>(path: &Path) -> Result<T, SaveError> {
    let s = fs::read_to_string(path)?;
    ron::from_str(&s).map_err(|e| SaveError::Ron(e.to_string()))
}

/// 파일이 있으면 로드, 없으면 `T::default()` 반환. 파싱 에러는 그대로 전파.
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
        std::env::temp_dir().join(format!("rust-gameengine-save-test-{}", std::process::id()))
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
        let loaded: Settings = load(&path).expect("load should succeed");

        assert_eq!(original, loaded);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_missing_file_returns_io_error() {
        let result: Result<Settings, SaveError> = load(Path::new("/nonexistent/path/foo.ron"));

        assert!(
            matches!(result, Err(SaveError::Io(_))),
            "expected SaveError::Io, got {:?}",
            result
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
