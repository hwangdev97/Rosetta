use crate::error::{Result, TranslatorError};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

pub struct Translator {
    client: Client,
    base_url: String,
    model: String,
    language_map: HashMap<String, String>,
}

impl Translator {
    pub fn new(api_key: String, base_url: String, model: String) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", api_key).parse().unwrap(),
        );
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        let mut language_map = HashMap::new();
        language_map.insert("ja".to_string(), "Japanese".to_string());
        language_map.insert("zh-Hans".to_string(), "Simplified Chinese".to_string());
        language_map.insert("zh-Hant".to_string(), "Traditional Chinese".to_string());
        language_map.insert("ko".to_string(), "Korean".to_string());
        language_map.insert("fr".to_string(), "French".to_string());
        language_map.insert("de".to_string(), "German".to_string());
        language_map.insert("es".to_string(), "Spanish".to_string());
        language_map.insert("pt".to_string(), "Portuguese".to_string());
        language_map.insert("it".to_string(), "Italian".to_string());
        language_map.insert("ru".to_string(), "Russian".to_string());
        language_map.insert("ar".to_string(), "Arabic".to_string());
        language_map.insert("hi".to_string(), "Hindi".to_string());

        Self {
            client,
            base_url,
            model,
            language_map,
        }
    }

    pub async fn translate_text(
        &self,
        text: &str,
        target_language: &str,
        context: Option<&str>,
    ) -> Result<String> {
        let target_lang_name = self
            .language_map
            .get(target_language)
            .map(|s| s.as_str())
            .unwrap_or(target_language);

        let context_part = context
            .map(|c| format!("Context: {}\n", c))
            .unwrap_or_default();

        let prompt = format!(
            r#"Please translate the following text to {}.
This is a localization string for an iOS app about time widgets and clocks.

Original text: "{}"
{}
Requirements:
- Keep the translation natural and appropriate for mobile app users
- Maintain any formatting like %@ placeholders
- For technical terms, use commonly accepted translations
- Keep brand names like "Hands Time" unchanged unless there's a standard localized version

Please provide only the translated text, no explanations."#,
            target_lang_name, text, context_part
        );

        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            max_tokens: 500,
            temperature: 0.3,
        };

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            return Err(TranslatorError::ApiError {
                status,
                message: error_text,
            });
        }

        let chat_response: ChatResponse = response.json().await?;

        let translation = chat_response
            .choices
            .first()
            .ok_or_else(|| TranslatorError::TranslationFailed("No choices in response".to_string()))?
            .message
            .content
            .trim()
            .to_string();

        // Remove surrounding quotes if present
        let translation = if translation.starts_with('"') && translation.ends_with('"') {
            translation[1..translation.len() - 1].to_string()
        } else {
            translation
        };

        if translation.is_empty() {
            return Err(TranslatorError::TranslationFailed(
                "Empty translation received".to_string(),
            ));
        }

        Ok(translation)
    }

    pub async fn batch_translate(
        &self,
        texts: &[String],
        target_language: &str,
    ) -> Vec<Result<String>> {
        let mut results = Vec::new();
        
        for text in texts {
            let result = self.translate_text(text, target_language, None).await;
            results.push(result);
            
            // Small delay to avoid rate limiting
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        
        results
    }
}