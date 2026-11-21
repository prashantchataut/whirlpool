//! Interface strings. The JSON files are the same ones the previous
//! web-based settings used, so every translation carried over unchanged.

use std::collections::HashMap;

use muelle_config::Language;
use muelle_shell::system::{self, UiLanguage};

const EN: &str = include_str!("../i18n/en.json");
const ES: &str = include_str!("../i18n/es.json");
const FR: &str = include_str!("../i18n/fr.json");

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
