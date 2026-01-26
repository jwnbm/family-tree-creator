/// 多言語対応モジュール
/// 
/// このモジュールはアプリケーションの多言語対応を提供します。
/// 現在、日本語と英語をサポートしています。

use std::sync::Mutex;

mod ja;
mod en;

static I18N_WARNINGS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// i18n警告をバッファに追加
pub(crate) fn add_warning(message: String) {
    if let Ok(mut warnings) = I18N_WARNINGS.lock() {
        warnings.push(message);
    }
}

/// 警告を取得してバッファをクリア
pub fn take_warnings() -> Vec<String> {
    if let Ok(mut warnings) = I18N_WARNINGS.lock() {
        std::mem::take(&mut *warnings)
    } else {
        Vec::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Japanese,
    English,
}

pub struct Texts;

impl Texts {
    pub fn get(key: &str, lang: Language) -> String {
        match lang {
            Language::Japanese => ja::translate(key),
            Language::English => en::translate(key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_japanese_translation() {
        assert_eq!(Texts::get("title", Language::Japanese), "家系図 (MVP)");
        assert_eq!(Texts::get("save", Language::Japanese), "保存");
        assert_eq!(Texts::get("male", Language::Japanese), "男性");
        assert_eq!(Texts::get("female", Language::Japanese), "女性");
    }

    #[test]
    fn test_english_translation() {
        assert_eq!(Texts::get("title", Language::English), "Family Tree (MVP)");
        assert_eq!(Texts::get("save", Language::English), "Save");
        assert_eq!(Texts::get("male", Language::English), "Male");
        assert_eq!(Texts::get("female", Language::English), "Female");
    }

    #[test]
    fn test_unknown_key() {
        assert_eq!(Texts::get("nonexistent_key", Language::Japanese), "nonexistent_key");
        assert_eq!(Texts::get("nonexistent_key", Language::English), "nonexistent_key");
    }

    #[test]
    fn test_language_equality() {
        assert_eq!(Language::Japanese, Language::Japanese);
        assert_eq!(Language::English, Language::English);
        assert_ne!(Language::Japanese, Language::English);
    }

    #[test]
    fn test_all_common_keys() {
        let keys = vec!["title", "save", "persons", "families", "settings"];
        
        for key in keys {
            let ja = Texts::get(key, Language::Japanese);
            let en = Texts::get(key, Language::English);
            
            assert_ne!(ja, key, "Japanese translation missing for key: {}", key);
            assert_ne!(en, key, "English translation missing for key: {}", key);
        }
    }
}
