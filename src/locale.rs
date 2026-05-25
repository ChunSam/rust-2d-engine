use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Locale text direction metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TextDirection {
    #[default]
    LeftToRight,
    RightToLeft,
}

/// Per-locale translation data and rendering metadata.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LocaleData {
    #[serde(default)]
    pub translations: HashMap<String, String>,
    #[serde(default)]
    pub font: Option<String>,
    #[serde(default)]
    pub direction: TextDirection,
}

/// Serializable bundle accepted by `LocaleResource::from_ron_str`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LocaleBundle {
    #[serde(default)]
    pub default_locale: String,
    #[serde(default)]
    pub locales: HashMap<String, LocaleData>,
}

/// Runtime localization resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleResource {
    current_locale: String,
    default_locale: String,
    locales: HashMap<String, LocaleData>,
}

impl Default for LocaleResource {
    fn default() -> Self {
        Self {
            current_locale: "en".to_string(),
            default_locale: "en".to_string(),
            locales: HashMap::new(),
        }
    }
}

impl LocaleResource {
    /// Creates an empty resource with the same current/default locale.
    pub fn new(default_locale: impl Into<String>) -> Self {
        let default_locale = default_locale.into();
        Self {
            current_locale: default_locale.clone(),
            default_locale,
            locales: HashMap::new(),
        }
    }

    /// Parses a RON localization bundle from a caller-provided string.
    pub fn from_ron_str(input: &str) -> Result<Self, ron::error::SpannedError> {
        let bundle: LocaleBundle = ron::from_str(input)?;
        Ok(Self::from_bundle(bundle))
    }

    /// Builds a resource from an already parsed bundle.
    pub fn from_bundle(bundle: LocaleBundle) -> Self {
        let default_locale = if bundle.default_locale.is_empty() {
            bundle
                .locales
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| "en".to_string())
        } else {
            bundle.default_locale
        };

        Self {
            current_locale: default_locale.clone(),
            default_locale,
            locales: bundle.locales,
        }
    }

    pub fn current_locale(&self) -> &str {
        &self.current_locale
    }

    pub fn default_locale(&self) -> &str {
        &self.default_locale
    }

    /// Switches locale if it exists. Returns whether the switch succeeded.
    pub fn set_locale(&mut self, locale: impl Into<String>) -> bool {
        let locale = locale.into();
        if self.locales.contains_key(&locale) {
            self.current_locale = locale;
            true
        } else {
            false
        }
    }

    /// Inserts or replaces one locale.
    pub fn insert_locale(&mut self, locale: impl Into<String>, data: LocaleData) {
        self.locales.insert(locale.into(), data);
    }

    /// Looks up a translation for the current locale, then the default locale, then the key itself.
    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.locales
            .get(&self.current_locale)
            .and_then(|locale| locale.translations.get(key))
            .or_else(|| {
                self.locales
                    .get(&self.default_locale)
                    .and_then(|locale| locale.translations.get(key))
            })
            .map(String::as_str)
            .unwrap_or(key)
    }

    pub fn font(&self) -> Option<&str> {
        self.locales
            .get(&self.current_locale)
            .and_then(|locale| locale.font.as_deref())
    }

    pub fn direction(&self) -> TextDirection {
        self.locales
            .get(&self.current_locale)
            .map(|locale| locale.direction)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
(
    default_locale: "en",
    locales: {
        "en": (
            translations: {
                "menu.start": "Start",
                "menu.quit": "Quit",
            },
            font: Some("Inter.ttf"),
            direction: LeftToRight,
        ),
        "ko": (
            translations: {
                "menu.start": "시작",
                "menu.quit": "종료",
            },
            font: Some("NotoSansKR.otf"),
            direction: LeftToRight,
        ),
        "ja": (
            translations: {
                "menu.start": "開始",
            },
            font: Some("NotoSansJP.otf"),
            direction: LeftToRight,
        ),
        "ar": (
            translations: {
                "menu.start": "ابدأ",
            },
            font: Some("NotoSansArabic.ttf"),
            direction: RightToLeft,
        ),
    },
)
"#;

    #[test]
    fn switching_locale_changes_translation() {
        let mut locale = LocaleResource::from_ron_str(SAMPLE).unwrap();

        assert_eq!(locale.t("menu.start"), "Start");
        assert!(locale.set_locale("ko"));
        assert_eq!(locale.t("menu.start"), "시작");
        assert!(locale.set_locale("ja"));
        assert_eq!(locale.t("menu.start"), "開始");
    }

    #[test]
    fn missing_key_falls_back_to_default_then_key() {
        let mut locale = LocaleResource::from_ron_str(SAMPLE).unwrap();
        assert!(locale.set_locale("ja"));

        assert_eq!(locale.t("menu.quit"), "Quit");
        assert_eq!(locale.t("menu.missing"), "menu.missing");
    }

    #[test]
    fn exposes_font_and_direction_metadata() {
        let mut locale = LocaleResource::from_ron_str(SAMPLE).unwrap();

        assert_eq!(locale.font(), Some("Inter.ttf"));
        assert_eq!(locale.direction(), TextDirection::LeftToRight);

        assert!(locale.set_locale("ar"));
        assert_eq!(locale.font(), Some("NotoSansArabic.ttf"));
        assert_eq!(locale.direction(), TextDirection::RightToLeft);
    }

    #[test]
    fn set_unknown_locale_keeps_current_locale() {
        let mut locale = LocaleResource::from_ron_str(SAMPLE).unwrap();

        assert!(!locale.set_locale("missing"));
        assert_eq!(locale.current_locale(), "en");
    }
}
