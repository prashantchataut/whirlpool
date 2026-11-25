//! Interface strings. The JSON files are the same ones the previous
//! web-based settings used, so every translation carried over unchanged.

use std::collections::HashMap;

use whirlpool_config::Language;
use whirlpool_shell::system::{self, UiLanguage};

const EN: &str = include_str!("../i18n/en.json");
const ES: &str = include_str!("../i18n/es.json");
const FR: &str = include_str!("../i18n/fr.json");
const ID: &str = include_str!("../i18n/id.json");

/// One language's strings, falling back to English and then to the key.
pub struct Texts {
    chosen: HashMap<String, String>,
    english: HashMap<String, String>,
}

fn parse(json: &str) -> HashMap<String, String> {
    serde_json::from_str(json).unwrap_or_default()
}

impl Texts {
    pub fn for_language(language: Language) -> Self {
        let json = match language {
            Language::Es => ES,
            Language::En => EN,
            Language::Fr => FR,
            Language::Id => ID,
            Language::Auto => match system::ui_language() {
                UiLanguage::Spanish => ES,
                UiLanguage::French => FR,
                UiLanguage::English => EN,
            },
        };
        Self {
            chosen: parse(json),
            english: parse(EN),
        }
    }

    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        self.chosen
            .get(key)
            .or_else(|| self.english.get(key))
            .map_or(key, String::as_str)
    }

    /// A string with `{version}` filled in.
    pub fn with_version(&self, key: &str, version: &str) -> String {
        self.get(key).replace("{version}", version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_translation_has_a_key_english_lacks() {
        let english = parse(EN);
        for (name, json) in [("es", ES), ("fr", FR), ("id", ID)] {
            for key in parse(json).keys() {
                assert!(
                    english.contains_key(key),
                    "{name} has {key}, English does not"
                );
            }
        }
    }

    #[test]
    fn every_language_has_the_clock_and_glass_strings() {
        for (name, json) in [("en", EN), ("es", ES), ("fr", FR), ("id", ID)] {
            let map = parse(json);
            for key in [
                "clockSeconds",
                "groupClock",
                "groupMaterial",
                "materialMatte",
                "hubStart",
                "hubShelfEmpty",
            ] {
                assert!(map.contains_key(key), "{name} lacks {key}");
            }
        }
    }

    #[test]
    fn placeholders_match_english() {
        let english = parse(EN);
        let braces = |s: &str| {
            let mut found: Vec<String> = s
                .split('{')
                .skip(1)
                .filter_map(|r| r.split('}').next())
                .map(str::to_owned)
                .collect();
            found.sort();
            found
        };
        for (name, json) in [("es", ES), ("fr", FR), ("id", ID)] {
            for (key, value) in parse(json) {
                if let Some(base) = english.get(&key) {
                    assert_eq!(braces(base), braces(&value), "{name}: {key}");
                }
            }
        }
    }
}
