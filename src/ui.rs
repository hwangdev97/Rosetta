use crate::ai_provider::AIProvider;
use crate::config::Config;
use crate::translator::Translator;
use crate::xcstrings::XCStringsFile;
use colored::Colorize;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::time::Duration;
use anyhow::Result;
use crate::ascii_art::ROSETTA_LOGO;

pub struct UI {
    pub provider: AIProvider,
}

impl UI {
    pub fn new(provider: AIProvider) -> Self {
        Self { provider }
    }

    pub fn print_banner() {
        println!();
        // ASCII art logo in bright white
        for line in ROSETTA_LOGO.lines() {
            println!("{}", line.bright_white());
        }
        println!("{}", "\n \n \nModern CLI tool for .xcstrings translation".bright_black());
        println!();
    }

    pub fn print_header(message: &str) {
        println!();
        println!("{}", message.bright_white().bold());
        println!();
    }

    pub fn print_step(message: &str) {
        println!("• {}", message.white());
    }

    pub fn print_substep(message: &str) {
        println!("  □ {}", message.bright_black());
    }

    pub fn print_success(message: &str) {
        println!("✓ {}", message.green());
    }

    pub fn print_warning(message: &str) {
        println!("! {}", message.yellow());
    }

    pub fn print_error(message: &str) {
        println!("✗ {}", message.red());
    }

    pub fn print_info(label: &str, value: &str) {
        println!("  {}: {}", label.bright_black(), value.white());
    }

    pub async fn interactive_translate(
        xcstrings: &mut XCStringsFile,
        translator: &Translator,
        keys: &[String],
        target_language: &str,
        _file_path: &Path,
    ) -> Result<()> {
        let total = keys.len();
        let mut current = 0;

        while current < total {
            let key = &keys[current];
            let remaining = total - current - 1;

            // Clear screen for clean interface
            let term = Term::stdout();
            term.clear_screen().ok();
            
            // Show progress in Claude Code style
            println!();
            println!("Translation Progress");
            Self::print_info("Status", &format!("{}/{} keys", current + 1, total));
            Self::print_info("Progress", &format!("{}%", (current as f64 / total as f64 * 100.0) as u8));
            println!();

            // Show current key
            println!("Key:");
            println!("  {}", key.bright_white());
            println!();

            // Show existing translation if any
            if let Some(existing) = xcstrings.get_existing_translation(key, target_language) {
                println!("Existing translation:");
                println!("  {}", existing.cyan());
                println!();
            }

            // Simple choice options
            let choices = if remaining > 0 {
                vec![
                    "Translate",
                    "Mark as no translation needed", 
                    "Batch translate next 30",
                    "Skip",
                    "Save and exit",
                ]
            } else {
                vec![
                    "Translate",
                    "Mark as no translation needed",
                    "Skip", 
                    "Save and exit",
                ]
            };

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Action")
                .items(&choices)
                .default(0)
                .interact()?;

            match selection {
                0 => {
                    // Translate
                    if Self::translate_single_key(xcstrings, translator, key, target_language).await? {
                        xcstrings.save()?;
                        Self::print_success("Translation saved");
                        tokio::time::sleep(Duration::from_millis(800)).await;
                    }
                    current += 1;
                }
                1 => {
                    // Mark as no translation needed
                    xcstrings.mark_as_no_translate(key)?;
                    xcstrings.save()?;
                    Self::print_success("Marked as no translation needed");
                    tokio::time::sleep(Duration::from_millis(800)).await;
                    current += 1;
                }
                2 if remaining > 0 => {
                    // Batch translate
                    let batch_size = std::cmp::min(30, remaining + 1);
                    let batch_keys = &keys[current..current + batch_size];
                    
                    if Self::batch_translate_confirm(batch_keys, target_language).await? {
                        Self::batch_translate_keys(xcstrings, translator, batch_keys, target_language).await?;
                        current += batch_size;
                    }
                }
                2 | 3 if remaining == 0 => {
                    // Skip (when no remaining items)
                    current += 1;
                }
                3 if remaining > 0 => {
                    // Skip (when there are remaining items)
                    current += 1;
                }
                4 if remaining > 0 => {
                    // Save and exit (when there are remaining items)
                    break;
                }
                3 if remaining == 0 => {
                    // Save and exit (when no remaining items)
                    break;
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    async fn translate_single_key(
        xcstrings: &mut XCStringsFile,
        translator: &Translator,
        key: &str,
        target_language: &str,
    ) -> Result<bool> {
        println!("Translating...");
        
        let result = translator.translate_text(key, target_language, None).await;

        match result {
            Ok(translation) => {
                println!();
                println!("Translation:");
                println!("  {}", translation.bright_white());
                println!();

                let confirm = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Accept translation?")
                    .default(true)
                    .interact()?;

                if confirm {
                    xcstrings.add_translation(key, target_language, &translation)?;
                    return Ok(true);
                } else {
                    let custom_translation: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Custom translation (empty to skip)")
                        .allow_empty(true)
                        .interact_text()?;

                    if !custom_translation.trim().is_empty() {
                        xcstrings.add_translation(key, target_language, &custom_translation)?;
                        return Ok(true);
                    }
                }
            }
            Err(e) => {
                Self::print_error(&format!("Translation failed: {}", e));
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }

        Ok(false)
    }

    async fn batch_translate_confirm(keys: &[String], target_language: &str) -> Result<bool> {
        println!();
        println!("Batch translate {} keys to {}", keys.len(), target_language.cyan());
        println!();

        Self::print_substep("Keys to translate:");
        for key in keys.iter().take(10) {
            println!("    {}", key.bright_black());
        }

        if keys.len() > 10 {
            println!("    {} ... and {} more", "...".bright_black(), (keys.len() - 10));
        }
        
        println!();

        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Proceed with batch translation")
            .default(true)
            .interact()
            .map_err(Into::into)
    }

    pub async fn auto_translate_all(
        xcstrings: &mut XCStringsFile,
        translator: &Translator,
        keys: &[String],
        target_language: &str,
        _file_path: &Path,
    ) -> Result<()> {
        Self::batch_translate_keys(xcstrings, translator, keys, target_language).await
    }

    async fn batch_translate_keys(
        xcstrings: &mut XCStringsFile,
        translator: &Translator,
        keys: &[String],
        target_language: &str,
    ) -> Result<()> {
        let pb = ProgressBar::new(keys.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{bar:40.cyan/blue} {pos:>3}/{len:3} {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );

        let mut success_count = 0;
        let mut failed_keys = Vec::new();

        for (i, key) in keys.iter().enumerate() {
            let display_key = if key.len() > 40 { 
                format!("{}...", &key[..37])
            } else { 
                key.clone() 
            };
            pb.set_message(display_key);

            match translator.translate_text(key, target_language, None).await {
                Ok(translation) => {
                    if let Err(_) = xcstrings.add_translation(key, target_language, &translation) {
                        failed_keys.push(key.clone());
                    } else {
                        success_count += 1;
                    }
                }
                Err(_) => {
                    failed_keys.push(key.clone());
                }
            }

            pb.inc(1);

            // Save periodically
            if (i + 1) % 10 == 0 {
                xcstrings.save()?;
            }

            // Rate limiting
            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        pb.finish_and_clear();

        // Final save
        xcstrings.save()?;

        // Results summary
        println!();
        println!("Batch translation completed");
        Self::print_info("Successful", &success_count.to_string());
        Self::print_info("Failed", &failed_keys.len().to_string());
        
        if !failed_keys.is_empty() && failed_keys.len() <= 5 {
            println!();
            Self::print_substep("Failed keys:");
            for key in &failed_keys {
                println!("    {}", key.bright_black());
            }
        }

        println!();
        Ok(())
    }

    pub fn display_provider_info(&self) {
        match &self.provider {
            AIProvider::OpenAI { api_key, model } => {
                println!("AI Provider:       {}", "OpenAI".bright_cyan());
                println!("API Key:           {}", mask_api_key(&api_key));
                println!("Model:             {}", model);
            }
            AIProvider::Claude { api_key, model } => {
                println!("AI Provider:       {}", "Claude".bright_cyan());
                println!("API Key:           {}", mask_api_key(&api_key));
                println!("Model:             {}", model);
            }
            AIProvider::Gemini { api_key, model } => {
                println!("AI Provider:       {}", "Google Gemini".bright_cyan());
                println!("API Key:           {}", mask_api_key(&api_key));
                println!("Model:             {}", model);
            }
        }
    }

    pub fn get_provider_name(&self) -> &'static str {
        match &self.provider {
            AIProvider::OpenAI { .. } => "OpenAI",
            AIProvider::Claude { .. } => "Claude",
            AIProvider::Gemini { .. } => "Google Gemini",
        }
    }

    pub fn get_model_name(&self) -> String {
        match &self.provider {
            AIProvider::OpenAI { model, .. } => model.clone(),
            AIProvider::Claude { model, .. } => model.clone(),
            AIProvider::Gemini { model, .. } => model.clone(),
        }
    }

    pub fn get_api_key(&self) -> String {
        match &self.provider {
            AIProvider::OpenAI { api_key, .. } => api_key.clone(),
            AIProvider::Claude { api_key, .. } => api_key.clone(),
            AIProvider::Gemini { api_key, .. } => api_key.clone(),
        }
    }

    pub async fn translate_interactive(
        &self,
        _xcstrings: &mut XCStringsFile,
        _translator: &Translator,
        _target_language: &str,
        _file_path: &Path,
    ) -> anyhow::Result<()> {
        // ... existing code ...
        Ok(())
    }

    pub async fn translate_batch(
        &self,
        _xcstrings: &mut XCStringsFile,
        _translator: &Translator,
        _target_language: &str,
        _file_path: &Path,
    ) -> anyhow::Result<()> {
        // ... existing code ...
        Ok(())
    }

    pub async fn translate_untranslated(
        &self,
        _xcstrings: &mut XCStringsFile,
        _translator: &Translator,
        _target_language: &str,
        _file_path: &Path,
    ) -> anyhow::Result<()> {
        // ... existing code ...
        Ok(())
    }

    pub async fn translate_all(
        &self,
        _xcstrings: &mut XCStringsFile,
        _translator: &Translator,
        _target_language: &str,
    ) -> anyhow::Result<()> {
        // ... existing code ...
        Ok(())
    }
}

pub fn display_config(config: &Config) {
    println!("\n{}", "Current Configuration".bright_white().bold());
    println!("Default Language:  {}", config.default_language);
    
    if let Some(path) = &config.project_path {
        println!("Project Path:      {}", path);
    } else {
        println!("Project Path:      {}", "Not set".yellow());
    }

    match &config.ai_provider {
        AIProvider::OpenAI { api_key, model } => {
            println!("AI Provider:       {}", "OpenAI".bright_blue());
            println!("API Key:           {}", mask_api_key(api_key));
            println!("Model:             {}", model);
        }
        AIProvider::Claude { api_key, model } => {
            println!("AI Provider:       {}", "Claude".bright_magenta());
            println!("API Key:           {}", mask_api_key(api_key));
            println!("Model:             {}", model);
        }
        AIProvider::Gemini { api_key, model } => {
            println!("AI Provider:       {}", "Google Gemini".bright_green());
            println!("API Key:           {}", mask_api_key(api_key));
            println!("Model:             {}", model);
        }
    }
}

fn mask_api_key(api_key: &str) -> String {
    if api_key.len() <= 8 {
        return "*".repeat(api_key.len());
    }
    format!("{}{}",
        &api_key[..4],
        "*".repeat(api_key.len() - 4)
    )
}