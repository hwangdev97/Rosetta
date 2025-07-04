use crate::error::{Result, TranslatorError};
use crate::key_mappings::{infer_key_meaning, categorize_usage};
use crate::TranslationMode;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// New structure for rich translation context
#[derive(Debug, Clone)]
pub struct TranslationContext {
    pub key: String,
    pub key_meaning: Option<String>,
    pub comment: Option<String>,
    pub source_text: String,
    pub existing_translations: HashMap<String, String>,
    pub usage_category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringUnit {
    pub state: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Localization {
    #[serde(rename = "stringUnit", skip_serializing_if = "Option::is_none")]
    pub string_unit: Option<StringUnit>,
    #[serde(rename = "shouldTranslate", skip_serializing_if = "Option::is_none")]
    pub should_translate: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizationEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "shouldTranslate", skip_serializing_if = "Option::is_none")]
    pub should_translate: Option<bool>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub localizations: HashMap<String, Localization>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct XCStringsData {
    #[serde(rename = "sourceLanguage")]
    pub source_language: String,
    pub version: String,
    pub strings: HashMap<String, LocalizationEntry>,
}

pub struct XCStringsFile {
    path: PathBuf,
    pub data: XCStringsData,
}

impl XCStringsFile {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let content = fs::read_to_string(&path)?;
        let data: XCStringsData = serde_json::from_str(&content)
            .map_err(|e| TranslatorError::FileFormatError(format!("Invalid JSON: {}", e)))?;

        Ok(Self { path, data })
    }

    pub fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.path, content)?;
        Ok(())
    }

    pub fn create_backup(&self) -> Result<PathBuf> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = self.path.with_extension(format!("xcstrings.backup_{}", timestamp));
        fs::copy(&self.path, &backup_path)?;
        Ok(backup_path)
    }

    pub fn get_keys_needing_translation(
        &self,
        target_language: &str,
        mode: &TranslationMode,
    ) -> Vec<String> {
        let mut keys = Vec::new();

        for (key, entry) in &self.data.strings {
            // Skip empty keys
            if key.trim().is_empty() {
                continue;
            }

            // Skip if globally marked as should not translate
            if entry.should_translate == Some(false) {
                continue;
            }

            // Check if marked as should not translate in any localization
            if self.is_marked_no_translate_globally(entry) {
                continue;
            }

            // Check based on translation mode
            match mode {
                TranslationMode::Supplement => {
                    // Only include if target language doesn't exist or has no translation
                    if !entry.localizations.contains_key(target_language)
                        || entry
                            .localizations
                            .get(target_language)
                            .and_then(|loc| loc.string_unit.as_ref())
                            .map(|unit| unit.value.is_empty())
                            .unwrap_or(true)
                    {
                        keys.push(key.clone());
                    }
                }
                TranslationMode::Fresh => {
                    // Include all keys that are not marked as should not translate
                    keys.push(key.clone());
                }
            }
        }

        keys
    }

    fn is_marked_no_translate_globally(&self, entry: &LocalizationEntry) -> bool {
        // Check root level shouldTranslate
        if entry.should_translate == Some(false) {
            return true;
        }

        // Check if any localization has shouldTranslate: false
        for localization in entry.localizations.values() {
            if localization.should_translate == Some(false) {
                return true;
            }
        }

        false
    }

    pub fn add_translation(&mut self, key: &str, target_language: &str, translation: &str) -> Result<()> {
        let entry = self.data.strings.get_mut(key)
            .ok_or_else(|| TranslatorError::TranslationFailed(format!("Key not found: {}", key)))?;

        if !entry.localizations.contains_key(target_language) {
            entry.localizations.insert(target_language.to_string(), Localization {
                string_unit: None,
                should_translate: None,
            });
        }

        let localization = entry.localizations.get_mut(target_language).unwrap();
        localization.string_unit = Some(StringUnit {
            state: "translated".to_string(),
            value: translation.to_string(),
        });

        Ok(())
    }

    pub fn mark_as_no_translate(&mut self, key: &str) -> Result<()> {
        let entry = self.data.strings.get_mut(key)
            .ok_or_else(|| TranslatorError::TranslationFailed(format!("Key not found: {}", key)))?;

        entry.should_translate = Some(false);
        Ok(())
    }

    pub fn get_existing_translation(&self, key: &str, target_language: &str) -> Option<String> {
        self.data.strings
            .get(key)?
            .localizations
            .get(target_language)?
            .string_unit
            .as_ref()
            .map(|unit| unit.value.clone())
    }

    pub fn get_keys(&self) -> Vec<String> {
        self.data.strings.keys().cloned().collect()
    }

    /// Get rich translation context for a key
    pub fn get_translation_context(&self, key: &str, source_language: &str) -> Option<TranslationContext> {
        let entry = self.data.strings.get(key)?;
        
        // Get source text from source language; if missing or empty, fall back to key itself
        let source_text = entry
            .localizations
            .get(source_language)
            .and_then(|loc| loc.string_unit.as_ref())
            .map(|unit| unit.value.clone())
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| key.to_string());

        // Get existing translations from other languages
        let mut existing_translations = HashMap::new();
        for (lang, localization) in &entry.localizations {
            if lang != source_language {
                if let Some(string_unit) = &localization.string_unit {
                    if !string_unit.value.trim().is_empty() {
                        existing_translations.insert(lang.clone(), string_unit.value.clone());
                    }
                }
            }
        }

        // Infer key meaning
        let key_meaning = infer_key_meaning(key);
        
        // Categorize usage
        let usage_category = categorize_usage(key, key_meaning.as_deref());

        Some(TranslationContext {
            key: key.to_string(),
            key_meaning,
            comment: entry.comment.clone(),
            source_text,
            existing_translations,
            usage_category,
        })
    }

    /// Get translation contexts for multiple keys
    pub fn get_translation_contexts(
        &self,
        keys: &[String],
        source_language: &str,
    ) -> Vec<TranslationContext> {
        keys.iter()
            .filter_map(|key| self.get_translation_context(key, source_language))
            .collect()
    }

    /// Get keys needing translation with rich context
    pub fn get_keys_with_context_needing_translation(
        &self,
        target_language: &str,
        mode: &TranslationMode,
    ) -> Vec<TranslationContext> {
        let keys = self.get_keys_needing_translation(target_language, mode);
        self.get_translation_contexts(&keys, &self.data.source_language)
    }
}