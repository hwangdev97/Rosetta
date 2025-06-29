use anyhow::Result;
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::ai_provider::AIProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub default_language: String,
    pub project_path: Option<String>,
    pub ai_provider: AIProvider,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: String::from("https://openrouter.ai/api/v1"),
            model: String::from("anthropic/claude-3.5-sonnet"),
            default_language: String::from("en"),
            project_path: None,
            ai_provider: AIProvider::OpenAI {
                api_key: String::new(),
                model: String::from("gpt-3.5-turbo"),
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Option<Self>> {
        let config_path = Self::config_path()?;
        if !config_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(config_path)?;
        let config = serde_json::from_str(&content)?;
        Ok(Some(config))
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let mut path = config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        path.push("rosetta");
        path.push("config.json");
        Ok(path)
    }

    pub fn update_ai_provider(&mut self, provider: AIProvider) -> Result<()> {
        // Keep top-level api_key & model in sync for backwards-compat displays.
        match &provider {
            AIProvider::OpenAI { api_key, model } => {
                self.api_key = api_key.clone();
                self.model = model.clone();
                self.base_url = "https://api.openai.com/v1".to_string();
            }
            AIProvider::Claude { api_key, model } => {
                self.api_key = api_key.clone();
                self.model = model.clone();
                self.base_url = "https://api.anthropic.com".to_string();
            }
            AIProvider::Gemini { api_key, model } => {
                self.api_key = api_key.clone();
                self.model = model.clone();
                self.base_url = "https://generativelanguage.googleapis.com/v1beta".to_string();
            }
        }

        self.ai_provider = provider;
        self.save()
    }

    pub fn update_default_language(&mut self, language: String) -> Result<()> {
        self.default_language = language;
        self.save()
    }

    pub fn update_project_path(&mut self, path: Option<String>) -> Result<()> {
        self.project_path = path;
        self.save()
    }

    pub fn new(api_key: String, default_language: String, project_path: Option<String>) -> Self {
        Self {
            api_key,
            base_url: String::from("https://openrouter.ai/api/v1"),
            model: String::from("anthropic/claude-3.5-sonnet"),
            default_language,
            project_path,
            ai_provider: AIProvider::OpenAI {
                api_key: String::new(),
                model: String::from("gpt-3.5-turbo"),
            },
        }
    }

    /// Display the current configuration in a user-friendly format.
    pub fn display(&self) {
        println!("\nCurrent Configuration");
        if let Ok(path) = Self::config_path() {
            println!("  Config path      : {}", path.display());
        }
        println!("  Default language : {}", self.default_language);
        println!("  Project path     : {}", self.project_path.as_deref().unwrap_or("<not set>"));
        println!("  Base URL         : {}", self.base_url);
        println!("  Model            : {}", self.model);
        match &self.ai_provider {
            crate::ai_provider::AIProvider::OpenAI { model, .. } => {
                println!("  Provider         : OpenAI ({})", model);
            }
            crate::ai_provider::AIProvider::Claude { model, .. } => {
                println!("  Provider         : Claude ({})", model);
            }
            crate::ai_provider::AIProvider::Gemini { model, .. } => {
                println!("  Provider         : Gemini ({})", model);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.default_language, "en");
        assert!(config.project_path.is_none());
        
        match config.ai_provider {
            AIProvider::OpenAI { model, .. } => {
                assert_eq!(model, "gpt-3.5-turbo");
            }
            _ => panic!("Default AI provider should be OpenAI"),
        }
    }

    #[test]
    fn test_config_save_load() -> Result<()> {
        // Create a temporary directory for testing
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("config.json");
        
        // Create test config
        let mut config = Config::default();
        config.update_default_language("ja".to_string())?;
        config.update_project_path(Some("/test/path".to_string()))?;
        config.update_ai_provider(AIProvider::Claude {
            api_key: "test_key".to_string(),
            model: "claude-3-opus-20240229".to_string(),
        })?;

        // Save config
        let content = serde_json::to_string_pretty(&config)?;
        fs::write(&config_path, content)?;

        // Load and verify config
        let loaded: Config = serde_json::from_str(&fs::read_to_string(&config_path)?)?;
        assert_eq!(loaded.default_language, "ja");
        assert_eq!(loaded.project_path, Some("/test/path".to_string()));
        
        match loaded.ai_provider {
            AIProvider::Claude { api_key, model } => {
                assert_eq!(api_key, "test_key");
                assert_eq!(model, "claude-3-opus-20240229");
            }
            _ => panic!("AI provider should be Claude"),
        }

        Ok(())
    }

    #[test]
    fn test_config_updates() -> Result<()> {
        let mut config = Config::default();

        // Test language update
        config.update_default_language("fr".to_string())?;
        assert_eq!(config.default_language, "fr");

        // Test project path update
        config.update_project_path(Some("/new/path".to_string()))?;
        assert_eq!(config.project_path, Some("/new/path".to_string()));

        // Test AI provider update
        config.update_ai_provider(AIProvider::Gemini {
            api_key: "gemini_key".to_string(),
            model: "gemini-pro".to_string(),
        })?;

        match config.ai_provider {
            AIProvider::Gemini { api_key, model } => {
                assert_eq!(api_key, "gemini_key");
                assert_eq!(model, "gemini-pro");
            }
            _ => panic!("AI provider should be Gemini"),
        }

        Ok(())
    }
} 