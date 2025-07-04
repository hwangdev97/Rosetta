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
use comfy_table::{Table, presets::UTF8_FULL, ContentArrangement};

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

            // Clear screen for clean interface only when stdout is an interactive TTY
            // In non-interactive environments (e.g. logging/CI) the escape sequence printed
            // by `clear_screen()` can appear as a flood of blank lines.  To avoid the
            // "large empty gap" reported by users, we clear the screen **only** if the
            // output is attached to a real terminal.
            let term = Term::stdout();
            if console::user_attended() {
                // Safe to clear in interactive mode.
                let _ = term.clear_screen();
            }
            
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

            // Load user config (fallback to default) to get batch size
            let mut config = crate::config::Config::load()?.unwrap_or_default();
            let default_batch_size = config.batch_size;

            // Build choice list dynamically
            let mut choices: Vec<String> = Vec::new();
            choices.push("Translate".to_string());
            choices.push("Mark as no translation needed".to_string());

            if remaining > 0 {
                choices.push(format!("Batch translate next {}", default_batch_size));
                choices.push("Batch translate next custom number".to_string());
                choices.push("Skip".to_string());
            }
            if remaining == 0 {
                choices.push("Skip".to_string());
            }
            // Save & exit is always available
            choices.push("Save and exit".to_string());

            // Create slice of &str references for dialoguer
            let choice_refs: Vec<&str> = choices.iter().map(|s| s.as_str()).collect();

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Action")
                .items(&choice_refs)
                .default(0)
                .interact()?;

            // Map selections to actions depending on whether `remaining > 0`
            match (selection, remaining > 0) {
                (0, _) => {
                    // Translate single key
                    if Self::translate_single_key(xcstrings, translator, key, target_language).await? {
                        xcstrings.save()?;
                        Self::print_success("Translation saved");
                        tokio::time::sleep(Duration::from_millis(800)).await;
                    }
                    current += 1;
                }
                (1, _) => {
                    // Mark as no translation needed
                    xcstrings.mark_as_no_translate(key)?;
                    xcstrings.save()?;
                    Self::print_success("Marked as no translation needed");
                    tokio::time::sleep(Duration::from_millis(800)).await;
                    current += 1;
                }
                (2, true) => {
                    // Batch translate with default batch size from config
                    let batch_size = std::cmp::min(default_batch_size, remaining + 1);
                    let batch_keys = &keys[current..current + batch_size];

                    if Self::batch_translate_confirm(batch_keys, target_language).await? {
                        Self::batch_translate_keys(xcstrings, translator, batch_keys, target_language).await?;
                        current += batch_size;
                    }
                }
                (3, true) => {
                    // Batch translate with custom number (and persist to config)
                    let input: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Enter custom batch size")
                        .validate_with(|val: &String| -> std::result::Result<(), &str> {
                            if val.trim().parse::<usize>().ok().filter(|v| *v > 0).is_some() {
                                Ok(())
                            } else {
                                Err("Please enter a positive integer")
                            }
                        })
                        .interact_text()?;

                    let new_batch_size = input.trim().parse::<usize>().unwrap_or(default_batch_size);
                    // Update config and save
                    config.update_batch_size(new_batch_size)?;

                    let size = std::cmp::min(new_batch_size, remaining + 1);
                    let batch_keys = &keys[current..current + size];

                    if Self::batch_translate_confirm(batch_keys, target_language).await? {
                        Self::batch_translate_keys(xcstrings, translator, batch_keys, target_language).await?;
                        current += size;
                    }
                }
                (4, true) => {
                    // Skip current key
                    current += 1;
                }
                (5, true) | (3, false) => {
                    // Save and exit
                    break;
                }
                (2, false) => {
                    // Skip when no remaining items
                    current += 1;
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
        
        // Get rich translation context
        let context = match xcstrings.get_translation_context(key, &xcstrings.data.source_language) {
            Some(ctx) => ctx,
            None => {
                Self::print_warning("Could not get translation context for this key");
                return Ok(false);
            }
        };

        // Show context information for user
        println!();
        println!("Translation Context:");
        if let Some(ref meaning) = context.key_meaning {
            Self::print_info("Key meaning", meaning);
        }
        if let Some(ref comment) = context.comment {
            Self::print_info("Comment", comment);
        }
        if let Some(ref category) = context.usage_category {
            Self::print_info("Category", category);
        }
        if !context.existing_translations.is_empty() {
            println!("  {}: ", "Other languages".bright_black());
            for (lang, trans) in &context.existing_translations {
                println!("    {}: {}", lang.bright_black(), trans.cyan());
            }
        }
        println!();

        let result = translator.translate_with_context(&context, target_language).await;

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

        Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
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
        // Get rich translation contexts for all keys
        let contexts = xcstrings.get_translation_contexts(keys, &xcstrings.data.source_language);
        
        let pb = ProgressBar::new(contexts.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{bar:40.cyan/blue} {pos:>3}/{len:3} {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  "),
        );

        let mut results: Vec<(String, Result<String, String>)> = Vec::new();

        // Unicode-safe helper to truncate long keys without splitting multibyte characters.
        fn ellipsize_utf8(s: &str, max_chars: usize) -> String {
            if s.chars().count() <= max_chars {
                return s.to_string();
            }

            // Find the byte index of the char boundary at `max_chars`.
            let mut char_indices = s.char_indices();
            let mut boundary = s.len();
            for _ in 0..max_chars {
                if let Some((idx, _)) = char_indices.next() {
                    boundary = idx;
                } else {
                    break;
                }
            }
            // Safety: `boundary` is guaranteed to be at a char boundary.
            format!("{}...", &s[..boundary])
        }

        for (_i, context) in contexts.iter().enumerate() {
            let display_key = ellipsize_utf8(&context.key, 40);
            pb.set_message(display_key);

            let result = match translator.translate_with_context(context, target_language).await {
                Ok(t) => Ok(t),
                Err(e) => Err(e.to_string()),
            };

            results.push((context.key.clone(), result));

            pb.inc(1);

            // Rate limiting to avoid hitting API limits  
            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        pb.finish_and_clear();

        // Build preview table
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["Key", "Translation (preview)", "Status"]);

        let mut success_count = 0;
        let mut failed_count = 0;

        for (k, res) in &results {
            match res {
                Ok(t) => {
                    success_count += 1;
                    table.add_row(vec![
                        ellipsize_utf8(k, 40),
                        ellipsize_utf8(t, 60),
                        "Success".green().to_string(),
                    ]);
                }
                Err(err_msg) => {
                    failed_count += 1;
                    table.add_row(vec![
                        ellipsize_utf8(k, 40),
                        "-".to_string(),
                        format!("Error: {}", err_msg).red().to_string(),
                    ]);
                }
            }
        }

        println!("\n{}", table);

        println!("\nSummary: {} successes, {} failures", success_count, failed_count);

        let proceed = Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Save successful translations to .xcstrings?")
            .default(true)
            .interact()?;

        if proceed {
            for (k, res) in results {
                if let Ok(trans) = res {
                    // Ignore individual save errors, collect later if needed
                    let _ = xcstrings.add_translation(&k, target_language, &trans);
                }
            }

            xcstrings.save()?;
            Self::print_success("Translations saved.");
        } else {
            Self::print_warning("Translations were not saved.");
        }

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
    println!("Batch Size:        {}", config.batch_size);
    
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