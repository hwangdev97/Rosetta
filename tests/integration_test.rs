use anyhow::Result;
use rosetta::{
    ai_provider::AIProvider,
    config::Config,
};
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_config_and_translation_flow() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("config.json");

    // Create test config with OpenAI provider
    let mut config = Config::default();
    config.update_default_language("ja".to_string())?;
    config.update_project_path(Some(temp_dir.path().to_string_lossy().into_owned()))?;
    config.update_ai_provider(AIProvider::OpenAI {
        api_key: "test_key".to_string(),
        model: "gpt-3.5-turbo".to_string(),
    })?;

    // Save config
    let content = serde_json::to_string_pretty(&config)?;
    fs::write(&config_path, content)?;

    // Create a test strings file
    let strings_content = r#"{
        "strings": {
            "hello_world": {
                "extractionState": "manual",
                "localizations": {
                    "en": {
                        "stringUnit": {
                            "state": "translated",
                            "value": "Hello, World!"
                        }
                    }
                }
            }
        }
    }"#;
    
    let strings_path = temp_dir.path().join("Localizable.xcstrings");
    fs::write(&strings_path, strings_content)?;

    // Test translation (this will fail with test key, which is expected)
    let result = config.ai_provider.translate("Hello, World!", "ja").await;
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_ai_provider_selection() {
    let providers = vec![
        AIProvider::OpenAI {
            api_key: "key1".to_string(),
            model: "gpt-4".to_string(),
        },
        AIProvider::Claude {
            api_key: "key2".to_string(),
            model: "claude-3-opus-20240229".to_string(),
        },
        AIProvider::Gemini {
            api_key: "key3".to_string(),
            model: "gemini-pro".to_string(),
        },
        AIProvider::Grok {
            api_key: "key4".to_string(),
            model: "grok-1".to_string(),
        },
    ];

    for provider in providers {
        let models = provider.available_models();
        assert!(!models.is_empty());

        match provider {
            AIProvider::OpenAI { model, .. } => {
                assert!(models.contains(&model));
            }
            AIProvider::Claude { model, .. } => {
                assert!(models.contains(&model));
            }
            AIProvider::Gemini { model, .. } => {
                assert!(models.contains(&model));
            }
            AIProvider::Grok { model, .. } => {
                assert!(models.contains(&model));
            }
        }
    }
}

#[tokio::test]
async fn test_connection_check() -> Result<()> {
    let provider = AIProvider::OpenAI {
        api_key: "invalid_key".to_string(),
        model: "gpt-4".to_string(),
    };

    let result = provider.test_connection().await;
    assert!(result.is_ok());
    assert!(!result.unwrap());

    Ok(())
} 