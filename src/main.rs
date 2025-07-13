mod ascii_art;
mod config;
mod error;
mod key_mappings;
mod onboarding;
mod translator;
mod ui;
mod xcstrings;
mod ai_provider;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum, CommandFactory};
use std::path::PathBuf;
use std::env;
use std::fs;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect, Select};
use chrono::DateTime;

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

    /// Clean up backup files
    Clean {
        /// Directory to search for backup files
        #[arg(
            short,
            long,
            help = "Directory to search for backup files (default: current directory)"
        )]
        directory: Option<PathBuf>,
    },

    /// Run initial setup and configuration
    Setup,

    /// Show configuration settings
    Config,

    /// Test connection with AI provider
    Test,

    /// Display welcome banner, help information and project GitHub address
    Hello,
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
            // Load config (or default) and merge CLI overrides.
            let mut config = Config::load()?.unwrap_or_default();

            // Determine effective values with CLI having priority.
            let effective_api_key = api_key.unwrap_or_else(|| config.api_key.clone());
            let effective_base_url = if base_url == "https://openrouter.ai/api/v1" {
                config.base_url.clone()
            } else {
                base_url
            };
            let effective_model = if model == "anthropic/claude-3.5-sonnet" {
                config.model.clone()
            } else {
                model
            };

            // Resolve language (CLI > config default).
            let language_value = language.as_ref().unwrap_or(&config.default_language).clone();

            // Persist updated default language if changed via CLI.
            if language.is_some() && language_value != config.default_language {
                config.update_default_language(language_value.clone())?;
            }

            translate_command(
                file,
                Some(effective_api_key),
                mode,
                effective_base_url,
                effective_model,
                auto,
                language_value,
            )
            .await?;
        }
        Some(Commands::Clean { directory }) => {
            clean_command(directory)?;
        }
        Some(Commands::Setup) => {
            if let Some(ob) = Onboarding::start().await? {
                let mut config = Config::new(
                    ob.api_key,
                    ob.default_language,
                    ob.project_path,
                );
                // Persist chosen provider (model & key) from onboarding.
                config.update_ai_provider(ob.ai_provider)?;

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
            if let Some(config) = Config::load()? {
                match config.ai_provider.test_connection().await {
                    Ok(true) => println!("{}", "Connection test successful!".green()),
                    Ok(false) => println!("{}", "Connection test failed.".red()),
                    Err(e) => println!("{} {}", "Error:".red(), e),
                }
            } else {
                println!("No configuration found. Run 'rosetta setup' first.");
            }
        }
        Some(Commands::Hello) => {
            // Display banner (ASCII art)
            UI::print_banner();

            // Show GitHub repository link
            println!("{}", "GitHub: https://github.com/hwangdev97/Rosetta".blue().underline());

            // Print CLI help information
            println!();
            // Clap's `print_long_help` returns an io::Result, safe to unwrap here
            Cli::command().print_long_help().unwrap();
            println!();
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
                println!("  clean        Clean up backup files");
                println!("  setup       Run initial setup and configuration");
                println!("  config      Show configuration settings");
                println!("  test        Test connection with AI provider");
                println!("  hello       Display welcome banner, help information and project GitHub address");
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
    language: String,
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

fn clean_command(directory: Option<PathBuf>) -> Result<()> {
    // Print banner
    UI::print_banner();
    
    // Get target directory
    let target_dir = directory.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    
    UI::print_step("Scanning for backup files...");
    UI::print_info("Directory", &target_dir.display().to_string());
    
    // Find all backup files
    let backup_files = find_backup_files(&target_dir)?;
    
    if backup_files.is_empty() {
        UI::print_warning("No backup files found in the specified directory.");
        return Ok(());
    }
    
    println!();
    UI::print_success(&format!("Found {} backup files:", backup_files.len()));
    
    // Display backup files with details
    for (i, backup) in backup_files.iter().enumerate() {
        let file_size = backup.metadata.len();
        let size_str = format_file_size(file_size);
        let modified_time = backup.modified_time.format("%Y-%m-%d %H:%M:%S");
        
        println!("  {}. {} ({}, {})", 
                 (i + 1).to_string().bright_white(),
                 backup.path.display().to_string().cyan(),
                 size_str.bright_black(),
                 modified_time.to_string().bright_black());
    }
    
    println!();
    
    // Ask user for action
    let options = vec![
        "删除全部备份文件",
        "手动选择要删除的文件",
        "取消"
    ];
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("请选择操作")
        .items(&options)
        .default(0)
        .interact()?;
    
    match selection {
        0 => {
            // Delete all
            println!();
            UI::print_warning("即将删除所有备份文件！");
            
            let confirm = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("确认删除全部备份文件吗？")
                .default(false)
                .interact()?;
            
            if confirm {
                let backup_refs: Vec<&BackupFile> = backup_files.iter().collect();
                delete_backup_files(&backup_refs)?;
                UI::print_success(&format!("已删除 {} 个备份文件", backup_files.len()));
            } else {
                UI::print_info("操作", "已取消");
            }
        }
        1 => {
            // Manual selection
            interactive_select_and_delete(&backup_files)?;
        }
        2 => {
            // Cancel
            UI::print_info("操作", "已取消");
        }
        _ => unreachable!(),
    }
    
    Ok(())
}

#[derive(Debug)]
struct BackupFile {
    path: PathBuf,
    metadata: fs::Metadata,
    modified_time: DateTime<chrono::Local>,
}

fn find_backup_files(directory: &PathBuf) -> Result<Vec<BackupFile>> {
    let mut backup_files = Vec::new();
    
    if !directory.exists() {
        anyhow::bail!("Directory does not exist: {}", directory.display());
    }
    
    if !directory.is_dir() {
        anyhow::bail!("Path is not a directory: {}", directory.display());
    }
    
    // Search recursively for backup files
    search_directory_recursive(directory, &mut backup_files)?;
    
    // Sort by modification time (newest first)
    backup_files.sort_by(|a, b| b.modified_time.cmp(&a.modified_time));
    
    Ok(backup_files)
}

fn search_directory_recursive(directory: &PathBuf, backup_files: &mut Vec<BackupFile>) -> Result<()> {
    let entries = fs::read_dir(directory)?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // Recursively search subdirectories
            search_directory_recursive(&path, backup_files)?;
        } else if path.is_file() {
            // Check if it's a backup file
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.contains(".xcstrings.backup_") {
                    let metadata = fs::metadata(&path)?;
                    let modified_time = metadata.modified()?;
                    let modified_time: DateTime<chrono::Local> = modified_time.into();
                    
                    backup_files.push(BackupFile {
                        path: path.clone(),
                        metadata,
                        modified_time,
                    });
                }
            }
        }
    }
    
    Ok(())
}

fn interactive_select_and_delete(backup_files: &[BackupFile]) -> Result<()> {
    println!();
    UI::print_info("提示", "使用 ↑↓ 移动，空格键选择，回车确认");
    println!();
    
    let file_names: Vec<String> = backup_files.iter().map(|backup| {
        let file_size = backup.metadata.len();
        let size_str = format_file_size(file_size);
        let modified_time = backup.modified_time.format("%Y-%m-%d %H:%M:%S");
        
        format!("{} ({}, {})", 
                backup.path.display(),
                size_str,
                modified_time)
    }).collect();
    
    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("选择要删除的备份文件")
        .items(&file_names)
        .interact()?;
    
    if selections.is_empty() {
        UI::print_info("操作", "未选择任何文件");
        return Ok(());
    }
    
    // Show selected files for confirmation
    println!();
    UI::print_warning("即将删除以下文件：");
    
    let selected_files: Vec<&BackupFile> = selections.iter()
        .map(|&i| &backup_files[i])
        .collect();
    
    for (i, backup) in selected_files.iter().enumerate() {
        println!("  {}. {}", 
                 (i + 1).to_string().bright_white(),
                 backup.path.display().to_string().red());
    }
    
    println!();
    
    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("确认删除选中的文件吗？")
        .default(false)
        .interact()?;
    
    if confirm {
        delete_backup_files(&selected_files)?;
        UI::print_success(&format!("已删除 {} 个备份文件", selected_files.len()));
    } else {
        UI::print_info("操作", "已取消");
    }
    
    Ok(())
}

fn delete_backup_files(backup_files: &[&BackupFile]) -> Result<()> {
    let mut deleted_count = 0;
    let mut failed_count = 0;
    
    for backup in backup_files {
        match fs::remove_file(&backup.path) {
            Ok(_) => {
                deleted_count += 1;
                UI::print_success(&format!("已删除: {}", backup.path.display()));
            }
            Err(e) => {
                failed_count += 1;
                UI::print_error(&format!("删除失败: {} - {}", backup.path.display(), e));
            }
        }
    }
    
    if failed_count > 0 {
        UI::print_warning(&format!("删除完成：{} 成功，{} 失败", deleted_count, failed_count));
    }
    
    Ok(())
}

fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}