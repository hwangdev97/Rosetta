use crate::error::{Result, TranslatorError};
use crate::xcstrings::TranslationContext;
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
        language_map.insert("pt-PT".to_string(), "Portuguese (Portugal)".to_string());
        language_map.insert("pt-BR".to_string(), "Portuguese (Brazil)".to_string());
        language_map.insert("it".to_string(), "Italian".to_string());
        language_map.insert("ru".to_string(), "Russian".to_string());
        language_map.insert("ar".to_string(), "Arabic".to_string());
        language_map.insert("hi".to_string(), "Hindi".to_string());
        language_map.insert("tr".to_string(), "Turkish".to_string());
        language_map.insert("nl".to_string(), "Dutch".to_string());
        language_map.insert("pl".to_string(), "Polish".to_string());
        language_map.insert("sv".to_string(), "Swedish".to_string());
        language_map.insert("no".to_string(), "Norwegian".to_string());
        language_map.insert("da".to_string(), "Danish".to_string());
        language_map.insert("fi".to_string(), "Finnish".to_string());
        language_map.insert("cs".to_string(), "Czech".to_string());
        language_map.insert("ro".to_string(), "Romanian".to_string());
        language_map.insert("uk".to_string(), "Ukrainian".to_string());
        language_map.insert("el".to_string(), "Greek".to_string());
        language_map.insert("he".to_string(), "Hebrew".to_string());
        language_map.insert("id".to_string(), "Indonesian".to_string());
        language_map.insert("th".to_string(), "Thai".to_string());
        language_map.insert("vi".to_string(), "Vietnamese".to_string());
        language_map.insert("ar".to_string(), "Arabic".to_string());
        language_map.insert("hi".to_string(), "Hindi".to_string());
        language_map.insert("ml".to_string(), "Malayalam".to_string());
        language_map.insert("en‑US".to_string(), "English (United States)".to_string());
        language_map.insert("en‑GB".to_string(), "English (United Kingdom)".to_string());
        language_map.insert("en‑AU".to_string(), "English (Australia)".to_string());
        language_map.insert("en‑CA".to_string(), "English (Canada)".to_string());
        language_map.insert("en‑NZ".to_string(), "English (New Zealand)".to_string());
        language_map.insert("en‑ZA".to_string(), "English (South Africa)".to_string());
        language_map.insert("en‑IN".to_string(), "English (India)".to_string());
        language_map.insert("en‑SG".to_string(), "English (Singapore)".to_string());
        language_map.insert("en‑HK".to_string(), "English (Hong Kong)".to_string());
        language_map.insert("en‑IE".to_string(), "English (Ireland)".to_string());
  

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

    /// Enhanced translation method with rich context
    pub async fn translate_with_context(
        &self,
        context: &TranslationContext,
        target_language: &str,
    ) -> Result<String> {
        let target_lang_name = self
            .language_map
            .get(target_language)
            .map(|s| s.as_str())
            .unwrap_or(target_language);

        // Build context information
        let mut context_parts = Vec::new();
        
        // Add key information
        context_parts.push(format!("Key: {}", context.key));
        
        // Add key meaning if available
        if let Some(ref meaning) = context.key_meaning {
            context_parts.push(format!("Key含义: {}", meaning));
        }
        
        // Add comment if available
        if let Some(ref comment) = context.comment {
            context_parts.push(format!("注释: {}", comment));
        }
        
        // Add usage category
        if let Some(ref category) = context.usage_category {
            let category_desc = match category.as_str() {
                "system_permission" => "系统权限说明",
                "app_metadata" => "应用元数据",
                "widget_ui" => "小组件界面",
                "ui_element" => "界面元素",
                "user_message" => "用户提示信息",
                _ => "通用文本",
            };
            context_parts.push(format!("用途类别: {}", category_desc));
        }
        
        // Add existing translations as reference
        if !context.existing_translations.is_empty() {
            context_parts.push("其他语言翻译参考:".to_string());
            for (lang, translation) in &context.existing_translations {
                let lang_name = self.language_map.get(lang).unwrap_or(lang);
                context_parts.push(format!("  - {}: \"{}\"", lang_name, translation));
            }
        }
        
        let context_info = context_parts.join("\n");

        let prompt = format!(
            r#"请将以下iOS应用本地化字符串翻译为{target_lang}。

翻译信息:
{context_info}

原文: "{source_text}"

翻译要求:
- 保持翻译自然流畅，符合{target_lang}使用习惯
- 保留所有格式化占位符（如 %@, %d, {{}}等）
- 技术术语使用标准翻译
- 品牌名称如"Hands Time"保持不变，除非有官方本地化版本
- 参考其他语言的翻译风格保持一致性
- 根据用途类别选择合适的语言风格和正式程度

请只提供翻译结果，不要包含解释。"#,
            target_lang = target_lang_name,
            context_info = context_info,
            source_text = context.source_text
        );

        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            max_tokens: 800,
            temperature: 0.2, // Lower temperature for more consistent translations
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

    /// Batch translation with context
    pub async fn batch_translate_with_context(
        &self,
        contexts: &[TranslationContext],
        target_language: &str,
    ) -> Vec<Result<String>> {
        let mut results = Vec::new();
        
        for context in contexts {
            let result = self.translate_with_context(context, target_language).await;
            results.push(result);
            
            // Small delay to avoid rate limiting
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        }
        
        results
    }
}