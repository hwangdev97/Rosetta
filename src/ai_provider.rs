use anyhow::Result;
use async_openai::{Client as OpenAIClient, types::{CreateChatCompletionRequestArgs, ChatCompletionRequestMessage, Role, ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent}};
use anthropic::{client::Client as AnthropicClient};
use google_generative_ai_rs::v1::{
    api::Client as GeminiClient,
    gemini::{
        request::{Request as GeminiRequest, GenerationConfig, SafetySettings},
        safety::{HarmCategory, HarmBlockThreshold},
        Content, Part, Role as GeminiRole, Model,
    },
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AIError {
    #[error("Invalid API key")]
    InvalidAPIKey,
    #[error("API request failed: {0}")]
    RequestFailed(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Model not available")]
    ModelNotAvailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIProvider {
    OpenAI {
        api_key: String,
        model: String,
    },
    Claude {
        api_key: String,
        model: String,
    },
    Gemini {
        api_key: String,
        model: String,
    },
}

impl AIProvider {
    pub async fn generate(&self, system_prompt: &str, text: &str) -> Result<String> {
        match self {
            AIProvider::OpenAI { api_key, model } => {
                let config = async_openai::config::OpenAIConfig::new().with_api_key(api_key.clone());
                let client = OpenAIClient::with_config(config);
                let request = CreateChatCompletionRequestArgs::default()
                    .model(model)
                    .messages([
                        ChatCompletionRequestMessage::System(
                            ChatCompletionRequestSystemMessage {
                                content: system_prompt.to_string(),
                                name: None,
                                role: Role::System,
                            }
                        ),
                        ChatCompletionRequestMessage::User(
                            ChatCompletionRequestUserMessage {
                                content: ChatCompletionRequestUserMessageContent::Text(text.to_string()),
                                name: None,
                                role: Role::User,
                            }
                        ),
                    ])
                    .build()?;

                let response = client.chat().create(request).await?;
                Ok(response.choices[0].message.content.clone().unwrap_or_default())
            }
            AIProvider::Claude { api_key, model } => {
                let client = AnthropicClient {
                    api_key: api_key.clone(),
                    ..Default::default()
                };

                let request = anthropic::types::MessagesRequest {
                    model: model.to_string(),
                    max_tokens: 1024,
                    system: system_prompt.to_string(),
                    messages: vec![anthropic::types::Message {
                        role: anthropic::types::Role::User,
                        content: vec![anthropic::types::ContentBlock::Text { text: text.to_string() }],
                    }],
                    ..Default::default()
                };

                let response = client.messages(request).await?;
                if let Some(content) = response.content.first() {
                    match content {
                        anthropic::types::ContentBlock::Text { text } => Ok(text.clone()),
                        _ => Err(AIError::RequestFailed("Unexpected response content type".to_string()).into()),
                    }
                } else {
                    Err(AIError::RequestFailed("No response content received".to_string()).into())
                }
            }
            AIProvider::Gemini { api_key, model } => {
                // Map the provided model string onto the crate's `Model` enum. For unknown strings we use `Custom`, available under the `beta` feature.
                let model_enum = match model.as_str() {
                    "gemini-1.0-pro" => Model::Gemini1_0Pro,
                    #[cfg(feature = "beta")]
                    _ => Model::Custom(model.clone()),
                    #[cfg(not(feature = "beta"))]
                    _ => Model::Gemini1_0Pro,
                };

                let client = GeminiClient::new_from_model(model_enum, api_key.clone());

                // Build the request payload expected by the new SDK.
                let request = GeminiRequest {
                    contents: vec![Content {
                        role: GeminiRole::User,
                        parts: vec![Part {
                            text: Some(format!("{}\n{}", system_prompt, text)),
                            inline_data: None,
                            file_data: None,
                            video_metadata: None,
                        }],
                    }],
                    tools: vec![],
                    safety_settings: vec![SafetySettings {
                        category: HarmCategory::HarmCategoryHarassment,
                        threshold: HarmBlockThreshold::BlockNone,
                    }],
                    generation_config: Some(GenerationConfig {
                        temperature: Some(0.7),
                        top_p: Some(0.8),
                        top_k: Some(40),
                        candidate_count: Some(1),
                        max_output_tokens: Some(1024),
                        stop_sequences: None,
                        response_mime_type: None,
                        response_schema: None,
                    }),
                    system_instruction: None,
                };

                // Execute the request (30-second timeout).
                let post_result = client
                    .post(30, &request)
                    .await
                    .map_err(|e| AIError::RequestFailed(e.to_string()))?;

                // Extract the plain (non-streamed) response text.
                if let Some(rest) = post_result.rest() {
                    if let Some(candidate) = rest.candidates.first() {
                        if let Some(part) = candidate.content.parts.first() {
                            if let Some(text) = &part.text {
                                return Ok(text.clone());
                            }
                        }
                    }
                }

                Err(AIError::RequestFailed("No valid response received".to_string()).into())
            }
        }
    }

    pub fn available_models(&self) -> Vec<String> {
        match self {
            AIProvider::OpenAI { .. } => vec![
                "gpt-4o".to_string(),
                "gpt-4-turbo-preview".to_string(),
                "gpt-4".to_string(),
                "gpt-3.5-turbo".to_string(),
            ],
            AIProvider::Claude { .. } => vec![
                "claude-3-opus-20240229".to_string(),
                "claude-3-sonnet-20240229".to_string(),
                "claude-3-haiku-20240307".to_string(),
                "claude-2.1".to_string(),
            ],
            AIProvider::Gemini { .. } => vec![
                // 2.5 family
                "gemini-2.5-pro".to_string(),
                "gemini-2.5-flash".to_string(),
                "gemini-2.5-flash-lite-preview-06-17".to_string(),
                "gemini-2.5-flash-preview-native-audio-dialog".to_string(),
                "gemini-2.5-flash-exp-native-audio-thinking-dialog".to_string(),
                "gemini-2.5-flash-preview-tts".to_string(),
                "gemini-2.5-pro-preview-tts".to_string(),

                // 2.0 family
                "gemini-2.0-flash".to_string(),
                "gemini-2.0-flash-preview-image-generation".to_string(),
                "gemini-2.0-flash-lite".to_string(),

                // 1.5 family
                "gemini-1.5-pro-latest".to_string(),
                "gemini-1.5-pro".to_string(),
                "gemini-1.5-flash-latest".to_string(),
                "gemini-1.5-flash".to_string(),
                "gemini-1.5-flash-8b".to_string(),

                // 1.0
                "gemini-1.0-pro".to_string(),
            ],
        }
    }

    pub async fn translate(&self, text: &str, target_language: &str) -> Result<String> {
        let system_prompt = format!(
            "You are a professional translator. Translate the following text to {}. \
            Maintain the original meaning and style while ensuring the translation sounds natural. \
            Only return the translated text, without any explanations or notes.",
            target_language
        );

        self.generate(&system_prompt, text).await
    }

    pub async fn test_connection(&self) -> Result<bool> {
        let test_text = "Hello, world!";
        let result = self.translate(test_text, "ja").await;
        Ok(result.is_ok())
    }

    /// Return a new AIProvider value with the same provider type and API key, but using the
    /// supplied `model` string. This makes it easy for callers to switch to experimental or
    /// custom-named models without having to construct a fresh enum variant manually.
    ///
    /// Example:
    /// ```rust
    /// let provider = AIProvider::OpenAI { api_key: "sk-...".into(), model: "gpt-4o".into() };
    /// let custom = provider.with_model("my-custom-model".into());
    /// ```
    pub fn with_model(&self, model: String) -> Self {
        match self {
            AIProvider::OpenAI { api_key, .. } => AIProvider::OpenAI {
                api_key: api_key.clone(),
                model,
            },
            AIProvider::Claude { api_key, .. } => AIProvider::Claude {
                api_key: api_key.clone(),
                model,
            },
            AIProvider::Gemini { api_key, .. } => AIProvider::Gemini {
                api_key: api_key.clone(),
                model,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_models() {
        let openai = AIProvider::OpenAI {
            api_key: "test".to_string(),
            model: "gpt-4".to_string(),
        };
        assert!(openai.available_models().contains(&"gpt-4o".to_string()));
        assert!(openai.available_models().contains(&"gpt-4-turbo-preview".to_string()));
        assert!(openai.available_models().contains(&"gpt-4".to_string()));
        assert!(openai.available_models().contains(&"gpt-3.5-turbo".to_string()));

        let claude = AIProvider::Claude {
            api_key: "test".to_string(),
            model: "claude-3-opus-20240229".to_string(),
        };
        assert!(claude.available_models().contains(&"claude-3-opus-20240229".to_string()));
        assert!(claude.available_models().contains(&"claude-3-sonnet-20240229".to_string()));
        assert!(claude.available_models().contains(&"claude-3-haiku-20240307".to_string()));
        assert!(claude.available_models().contains(&"claude-2.1".to_string()));

        let gemini = AIProvider::Gemini {
            api_key: "test".to_string(),
            model: "gemini-1.5-pro-latest".to_string(),
        };
        assert!(gemini.available_models().contains(&"gemini-2.5-pro".to_string()));
        assert!(gemini.available_models().contains(&"gemini-2.5-flash".to_string()));
        assert!(gemini.available_models().contains(&"gemini-2.0-flash".to_string()));
        assert!(gemini.available_models().contains(&"gemini-1.5-pro-latest".to_string()));
        assert!(gemini.available_models().contains(&"gemini-1.5-flash".to_string()));
        assert!(gemini.available_models().contains(&"gemini-1.0-pro".to_string()));
    }

    #[tokio::test]
    async fn test_translate_with_mock() {
        // Test OpenAI with invalid key
        let provider = AIProvider::OpenAI {
            api_key: "invalid_key".to_string(),
            model: "gpt-4".to_string(),
        };
        let result = provider.translate("Hello", "ja").await;
        assert!(result.is_err());

        // Test Claude with invalid key
        let provider = AIProvider::Claude {
            api_key: "invalid_key".to_string(),
            model: "claude-3-opus-20240229".to_string(),
        };
        let result = provider.translate("Hello", "ja").await;
        assert!(result.is_err());

        // Test Gemini with invalid key
        let provider = AIProvider::Gemini {
            api_key: "invalid_key".to_string(),
            model: "gemini-1.5-pro-latest".to_string(),
        };
        let result = provider.translate("Hello", "ja").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_ai_error_display() {
        let error = AIError::InvalidAPIKey;
        assert_eq!(error.to_string(), "Invalid API key");

        let error = AIError::RequestFailed("Network error".to_string());
        assert_eq!(error.to_string(), "API request failed: Network error");

        let error = AIError::RateLimitExceeded;
        assert_eq!(error.to_string(), "Rate limit exceeded");

        let error = AIError::ModelNotAvailable;
        assert_eq!(error.to_string(), "Model not available");
    }
} 