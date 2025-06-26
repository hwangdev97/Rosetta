mod ascii_art;
mod config;
mod error;
mod onboarding;
mod translator;
mod ui;
mod xcstrings;
mod ai_provider;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::env;
use colored::*;

use crate::config::Config;
use crate::onboarding::Onboarding;
use crate::translator::Translator;
use crate::ui::UI;
use crate::xcstrings::XCStringsFile;

#[derive(Debug, Clone, ValueEnum)]
pub enum TranslationMode {
    /// Supplement missing translations only
    Supplement,
    /// Fresh translation for all keys (overwrites existing)
    Fresh,
}

#[derive(Subcommand)]
enum Commands {
    /// Translate strings to target language
    #[command(arg_required_else_help = true)]
    Translate {
        /// Target language code (e.g., ja, zh-Hans, ko, fr)
        #[arg(help = "Target language code (ja, zh-Hans, zh-Hant, ko, fr, de, es, etc.)")]
        language: Option<String>,

        /// Path to .xcstrings file
        #[arg(
            short,
            long,
            help = "Path to .xcstrings file (auto-detected if not specified)"
        )]
        file: Option<PathBuf>,

        /// OpenRouter API key
        #[arg(
            short = 'k',
            long,
            help = "OpenRouter API key (or set OPENROUTER_API_KEY env var)"
        )]
        api_key: Option<String>,

        /// Translation mode
        #[arg(
            short,
            long,
            default_value = "supplement",
            help = "Translation mode: supplement (skip existing) or fresh (retranslate all)"
        )]
        mode: TranslationMode,

        /// Base URL for OpenRouter API
        #[arg(
            long,
            default_value = "https://openrouter.ai/api/v1",
            help = "OpenRouter API base URL"
        )]
        base_url: String,

        /// Model to use for translation
        #[arg(
            long,
            default_value = "anthropic/claude-3.5-sonnet",
            help = "AI model to use for translation"
        )]
        model: String,

        /// Skip interactive mode (auto-translate all)
        #[arg(long, help = "Skip interactive mode and auto-translate all keys")]
        auto: bool,
    },

    /// Run initial setup and configuration
    Setup,

    /// Show configuration settings
    Config,

    /// Test connection with AI provider
    Test,
}

#[derive(Parser)]
#[command(
    name = "rosetta",
    about = "🌍 Modern iOS localization tool with beautiful CLI",
    long_about = "A modern command-line tool for translating iOS .xcstrings files using OpenRouter API with interactive UI and progress tracking.",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

fn find_project_root() -> Option<PathBuf> {
    let mut current_dir = env::current_dir().ok()?;
    
    loop {
        let xcstrings_path = current_dir.join("Shared/Resources/Localizable.xcstrings");
        if xcstrings_path.exists() {
            return Some(xcstrings_path);
        }
        
        if !current_dir.pop() {
            break;
        }
    }
    
    None
}

fn get_api_key(provided_key: Option<String>) -> Result<String> {
    if let Some(key) = provided_key {
        return Ok(key);
    }
    
    if let Ok(key) = env::var("OPENROUTER_API_KEY") {
        return Ok(key);
    }
    
    anyhow::bail!(
        "No API key found. Please:\n  • Use --api-key parameter\n  • Set OPENROUTER_API_KEY environment variable"
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Translate {
            language,
            file,
            api_key,
            mode,
            base_url,
            model,
            auto,
        }) => {
            // Load config if available
            let mut config = Config::load()?;
            
            // Use config values as fallbacks
            let api_key = api_key.or_else(|| config.as_ref().map(|c| c.api_key.clone()));
            let base_url = if base_url == "https://openrouter.ai/api/v1" {
                config.as_ref().map(|c| c.base_url.clone()).unwrap_or(base_url)
            } else {
                base_url
            };
            let model = if model == "anthropic/claude-3.5-sonnet" {
                config.as_ref().map(|c| c.model.clone()).unwrap_or(model)
            } else {
                model
            };
            
            if let Some(l) = language {
                config.update_default_language(l)?;
            }
            
            translate_command(file, api_key, mode, base_url, model, auto).await?;
        }
        Some(Commands::Setup) => {
            if let Some(config) = Onboarding::start().await? {
                let config = Config::new(
                    config.api_key,
                    config.default_language,
                    config.project_path,
                );
                config.save()?;
                println!("\n✨ Configuration saved successfully!");
            }
        }
        Some(Commands::Config) => {
            if let Some(config) = Config::load()? {
                config.display();
            } else {
                println!("\n❌ No configuration found. Run 'rosetta setup' to create one.");
            }
        }
        Some(Commands::Test) => {
            let config = Config::load()?;
            match config.ai_provider.test_connection().await {
                Ok(true) => println!("{}", "Connection test successful!".green()),
                Ok(false) => println!("{}", "Connection test failed.".red()),
                Err(e) => println!("{} {}", "Error:".red(), e),
            }
        }
        None => {
            // No command provided, check if config exists
            if Config::load()?.is_none() {
                // No config found, run onboarding
                if let Some(config) = Onboarding::start().await? {
                    let config = Config::new(
                        config.api_key,
                        config.default_language,
                        config.project_path,
                    );
                    config.save()?;
                    println!("\n✨ Configuration saved successfully!");
                }
            } else {
                // Config exists, show help
                println!("\nUsage: rosetta <COMMAND>");
                println!("\nCommands:");
                println!("  translate    Translate strings to target language");
                println!("  setup       Run initial setup and configuration");
                println!("  config      Show configuration settings");
                println!("  test        Test connection with AI provider");
                println!("\nRun 'rosetta --help' for more information.");
            }
        }
    }

    Ok(())
}

async fn translate_command(
    file: Option<PathBuf>,
    api_key: Option<String>,
    mode: TranslationMode,
    base_url: String,
    model: String,
    auto: bool,
) -> Result<()> {
    // Print welcome banner
    UI::print_banner();
    
    // Get file path
    let file_path = match file {
        Some(path) => {
            if !path.exists() {
                anyhow::bail!("File not found: {}", path.display());
            }
            path
        }
        None => {
            UI::print_step("Auto-detecting project file...");
            match find_project_root() {
                Some(path) => {
                    UI::print_success(&format!("Found: {}", path.display()));
                    path
                }
                None => {
                    anyhow::bail!("Could not find Localizable.xcstrings. Use --file to specify path.");
                }
            }
        }
    };
    
    // Get API key
    let api_key = get_api_key(api_key)?;
    
    // Initialize components
    UI::print_step("Initializing translator...");
    let translator = Translator::new(api_key, base_url, model);
    
    UI::print_step("Loading localization file...");
    let mut xcstrings = XCStringsFile::load(&file_path)?;
    
    // Create backup
    UI::print_step("Creating backup...");
    let backup_path = xcstrings.create_backup()?;
    UI::print_success(&format!("Backup: {}", backup_path.display()));
    
    // Get keys to translate
    let keys = xcstrings.get_keys_needing_translation(&language, &mode);
    
    if keys.is_empty() {
        let mode_desc = match mode {
            TranslationMode::Supplement => "supplement translation",
            TranslationMode::Fresh => "fresh translation",
        };
        UI::print_success(&format!(
            "No keys need {} for language '{}'",
            mode_desc, language
        ));
        return Ok(());
    }
    
    // Start translation process
    println!();
    println!("Translation Task");
    UI::print_info("Target", &language);
    UI::print_info("Mode", match mode {
        TranslationMode::Supplement => "Supplement (skip existing)",
        TranslationMode::Fresh => "Fresh (retranslate all)",
    });
    UI::print_info("Keys", &keys.len().to_string());
    println!();
    
    // Translation process
    if auto {
        // Auto mode - translate all without interaction
        UI::auto_translate_all(&mut xcstrings, &translator, &keys, &language, &file_path).await?;
    } else {
        // Interactive mode
        UI::interactive_translate(&mut xcstrings, &translator, &keys, &language, &file_path).await?;
    }
    
    UI::print_success("Translation completed");
    UI::print_info("Backup", &backup_path.display().to_string());
    UI::print_info("Output", &file_path.display().to_string());
    
    Ok(())
}