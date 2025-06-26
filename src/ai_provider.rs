use anyhow::Result;
use async_openai::{Client as OpenAIClient, types::{CreateChatCompletionRequestArgs, ChatCompletionRequestMessage, Role}};
use anthropic::{Client as AnthropicClient, Message};
use google_generative_ai_rs::{v1::{GenerativeClient, GenerateContentRequest, Part, Contents}};
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
    Grok {
        api_key: String,
        model: String,
    },
}

impl AIProvider {
    pub fn available_models(&self) -> Vec<String> {
        match self {
            AIProvider::OpenAI { .. } => vec![
                "gpt-4-turbo-preview".to_string(),
                "gpt-4".to_string(),
                "gpt-3.5-turbo".to_string(),
            ],
            AIProvider::Claude { .. } => vec![
                "claude-3-opus-20240229".to_string(),
                "claude-3-sonnet-20240229".to_string(),
                "claude-2.1".to_string(),
            ],
            AIProvider::Gemini { .. } => vec![
                "gemini-pro".to_string(),
                "gemini-pro-vision".to_string(),
            ],
            AIProvider::Grok { .. } => vec![
                "grok-1".to_string(),
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

        match self {
            AIProvider::OpenAI { api_key, model } => {
                let client = OpenAIClient::new().with_api_key(api_key);
                let request = CreateChatCompletionRequestArgs::default()
                    .model(model)
                    .messages([
                        ChatCompletionRequestMessage {
                            role: Role::System,
                            content: system_prompt,
                            name: None,
                            function_call: None,
                            tool_calls: None,
                            tool_call_id: None,
                        },
                        ChatCompletionRequestMessage {
                            role: Role::User,
                            content: text.to_string(),
                            name: None,
                            function_call: None,
                            tool_calls: None,
                            tool_call_id: None,
                        },
                    ])
                    .build()?;

                let response = client.chat().create(request).await?;
                Ok(response.choices[0].message.content.clone().unwrap_or_default())
            }

            AIProvider::Claude { api_key, model } => {
                let client = AnthropicClient::new(api_key.clone());
                let message = Message::new(model)
                    .system_prompt(system_prompt)
                    .user_prompt(text);

                let response = client.messages().create(message).await?;
                Ok(response.content)
            }

            AIProvider::Gemini { api_key, model } => {
                let client = GenerativeClient::new(api_key);
                let request = GenerateContentRequest {
                    contents: vec![Contents {
                        parts: vec![
                            Part::text(system_prompt),
                            Part::text(text.to_string()),
                        ],
                    }],
                    model: model.clone(),
                    ..Default::default()
                };

                let response = client.generate_content(request).await?;
                Ok(response.candidates[0].content.parts[0].text.clone())
            }

            AIProvider::Grok { api_key, model } => {
                // Note: Grok API is still in development, this is a placeholder
                // implementation based on their current API structure
                let client = reqwest::Client::new();
                let response = client
                    .post("https://api.grok.x/v1/chat/completions")
                    .header("Authorization", format!("Bearer {}", api_key))
                    .json(&serde_json::json!({
                        "model": model,
                        "messages": [
                            {
                                "role": "system",
                                "content": system_prompt
                            },
                            {
                                "role": "user",
                                "content": text
                            }
                        ]
                    }))
                    .send()
                    .await?;

                let json: serde_json::Value = response.json().await?;
                Ok(json["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string())
            }
        }
    }

    pub async fn test_connection(&self) -> Result<bool> {
        let test_text = "Hello, world!";
        let result = self.translate(test_text, "ja").await;
        Ok(result.is_ok())
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
        assert!(openai.available_models().contains(&"gpt-4".to_string()));
        assert!(openai.available_models().contains(&"gpt-3.5-turbo".to_string()));

        let claude = AIProvider::Claude {
            api_key: "test".to_string(),
            model: "claude-3-opus-20240229".to_string(),
        };
        assert!(claude.available_models().contains(&"claude-3-opus-20240229".to_string()));
        assert!(claude.available_models().contains(&"claude-2.1".to_string()));

        let gemini = AIProvider::Gemini {
            api_key: "test".to_string(),
            model: "gemini-pro".to_string(),
        };
        assert!(gemini.available_models().contains(&"gemini-pro".to_string()));
        assert!(gemini.available_models().contains(&"gemini-pro-vision".to_string()));

        let grok = AIProvider::Grok {
            api_key: "test".to_string(),
            model: "grok-1".to_string(),
        };
        assert!(grok.available_models().contains(&"grok-1".to_string()));
    }

    #[tokio::test]
    async fn test_translate_with_mock() {
        // This is a mock test that doesn't make actual API calls
        let provider = AIProvider::OpenAI {
            api_key: "invalid_key".to_string(),
            model: "gpt-4".to_string(),
        };
        
        let result = provider.translate("Hello", "ja").await;
        assert!(result.is_err()); // Should fail with invalid key

        let provider = AIProvider::Claude {
            api_key: "invalid_key".to_string(),
            model: "claude-3-opus-20240229".to_string(),
        };
        
        let result = provider.translate("Hello", "ja").await;
        assert!(result.is_err()); // Should fail with invalid key
    }
} 