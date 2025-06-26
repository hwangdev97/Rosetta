use crate::ascii_art::ROSETTA_LOGO;
use anyhow::Result;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Input, Select};

use crate::config::Config;
use crate::ai_provider::AIProvider;

pub struct OnboardingConfig {
    pub api_key: String,
    pub default_language: String,
    pub project_path: Option<String>,
}

pub struct Onboarding;

impl Onboarding {
    pub async fn start() -> Result<Option<OnboardingConfig>> {
        // Clear screen
        let term = console::Term::stdout();
        term.clear_screen()?;

        // Show logo with a typing effect
        for line in ROSETTA_LOGO.lines() {
            println!("{}", line.bright_cyan());
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        println!("\n{}", "Welcome to Rosetta!".bright_white().bold());
        println!("{}", "Let's set up your localization environment.".bright_black());
        println!();

        // Ask if they want to proceed with setup
        let proceed = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Would you like to set up Rosetta now?")
            .default(true)
            .interact()?;

        if !proceed {
            println!("\n{}", "You can run Rosetta setup anytime by using:".bright_black());
            println!("{}", "  rosetta setup".bright_white());
            return Ok(None);
        }

        println!();
        println!("{}", "🔑 API Configuration".bright_white().bold());
        
        // Select AI provider
        let providers = vec!["OpenAI", "Claude", "Google Gemini"];
        let provider_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select your AI provider")
            .items(&providers)
            .default(0)
            .interact()?;

        // Get API key
        let api_key: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter your API key")
            .interact_text()?;

        // Select model based on provider
        let _provider = match provider_idx {
            0 => {
                let models = vec!["gpt-4-turbo-preview", "gpt-4", "gpt-3.5-turbo"];
                let model_idx = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select OpenAI model")
                    .items(&models)
                    .default(0)
                    .interact()?;
                
                AIProvider::OpenAI {
                    api_key: api_key.clone(),
                    model: models[model_idx].to_string(),
                }
            }
            1 => {
                let models = vec![
                    "claude-3-opus-20240229",
                    "claude-3-sonnet-20240229",
                    "claude-2.1",
                ];
                let model_idx = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select Claude model")
                    .items(&models)
                    .default(0)
                    .interact()?;
                
                AIProvider::Claude {
                    api_key: api_key.clone(),
                    model: models[model_idx].to_string(),
                }
            }
            2 => {
                let models = vec![
                    "gemini-1.5-pro-latest",
                    "gemini-1.0-pro",
                    "gemini-1.5-flash",
                    "gemini-1.5-flash-8b",
                    "gemini-2.0-flash-exp",
                ];
                let model_idx = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select Gemini model")
                    .items(&models)
                    .default(0)
                    .interact()?;
                
                AIProvider::Gemini {
                    api_key: api_key.clone(),
                    model: models[model_idx].to_string(),
                }
            }
            _ => unreachable!(),
        };

        println!();
        println!("{}", "🌍 Language Settings".bright_white().bold());

        // Get default language
        let default_language: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter your default target language (e.g., ja, zh-Hans, ko)")
            .default("en".into())
            .interact_text()?;

        println!();
        println!("{}", "📁 Project Settings".bright_white().bold());

        // Get project path (optional)
        let project_path: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter your project path (optional, press Enter to skip)")
            .allow_empty(true)
            .interact_text()?;

        let project_path = if project_path.is_empty() {
            None
        } else {
            Some(project_path)
        };

        println!();
        println!("{}", "✨ Setup Complete!".bright_green().bold());
        println!("{}", "Your settings have been saved. You can change them anytime using:".bright_black());
        println!("{}", "  rosetta config".bright_white());
        println!();

        Ok(Some(OnboardingConfig {
            api_key,
            default_language,
            project_path,
        }))
    }
}

pub async fn run() -> Result<()> {
    println!("{}", ROSETTA_LOGO);
    println!("Welcome to Rosetta! Let's get you set up.\n");

    let theme = ColorfulTheme::default();

    // Select AI provider
    let providers = vec!["OpenAI", "Claude", "Google Gemini"];
    let provider_idx = Select::with_theme(&theme)
        .with_prompt("Select your AI provider")
        .items(&providers)
        .default(0)
        .interact()?;

    // Get API key
    let api_key: String = Input::with_theme(&theme)
        .with_prompt("Enter your API key")
        .interact_text()?;

    // Select model based on provider
    let provider = match provider_idx {
        0 => {
            let models = vec!["gpt-4-turbo-preview", "gpt-4", "gpt-3.5-turbo"];
            let model_idx = Select::with_theme(&theme)
                .with_prompt("Select OpenAI model")
                .items(&models)
                .default(0)
                .interact()?;
            
            AIProvider::OpenAI {
                api_key,
                model: models[model_idx].to_string(),
            }
        }
        1 => {
            let models = vec![
                "claude-3-opus-20240229",
                "claude-3-sonnet-20240229",
                "claude-2.1",
            ];
            let model_idx = Select::with_theme(&theme)
                .with_prompt("Select Claude model")
                .items(&models)
                .default(0)
                .interact()?;
            
            AIProvider::Claude {
                api_key,
                model: models[model_idx].to_string(),
            }
        }
        2 => {
            let models = vec![
                "gemini-1.5-pro-latest",
                "gemini-1.0-pro",
                "gemini-1.5-flash",
                "gemini-1.5-flash-8b",
                "gemini-2.0-flash-exp",
            ];
            let model_idx = Select::with_theme(&theme)
                .with_prompt("Select Gemini model")
                .items(&models)
                .default(0)
                .interact()?;
            
            AIProvider::Gemini {
                api_key,
                model: models[model_idx].to_string(),
            }
        }
        _ => unreachable!(),
    };

    // Get default language
    let default_language: String = Input::with_theme(&theme)
        .with_prompt("Enter your default target language (e.g., ja, zh-Hans, ko)")
        .default("en".into())
        .interact_text()?;

    // Get project path (optional)
    let project_path: String = Input::with_theme(&theme)
        .with_prompt("Enter your project path (optional, press Enter to skip)")
        .allow_empty(true)
        .interact_text()?;

    let project_path = if project_path.is_empty() {
        None
    } else {
        Some(project_path)
    };

    // Create and save config
    let mut config = Config::default();
    config.update_default_language(default_language)?;
    config.update_project_path(project_path)?;
    config.update_ai_provider(provider)?;

    println!("\n{}", "Configuration saved successfully!".green());
    println!("\nYou can now use Rosetta. Try running:");
    println!("  rosetta translate --help");
    println!("  rosetta config");
    println!("  rosetta test");

    Ok(())
}

pub async fn select_ai_provider() -> Result<AIProvider> {
    let providers = vec![
        "OpenAI (GPT-4, GPT-3.5)",
        "Anthropic (Claude)",
        "Google (Gemini)",
    ];

    let provider_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an AI provider")
        .items(&providers)
        .default(0)
        .interact()?;

    let provider = match provider_idx {
        0 => {
            let models = vec![
                "gpt-4",
                "gpt-3.5-turbo",
            ];
            let model_idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a model")
                .items(&models)
                .default(0)
                .interact()?;

            let api_key = dialoguer::Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter your OpenAI API key")
                .interact()?;

            AIProvider::OpenAI {
                api_key,
                model: models[model_idx].to_string(),
            }
        }
        1 => {
            let models = vec![
                "claude-3-opus-20240229",
                "claude-2.1",
            ];
            let model_idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a model")
                .items(&models)
                .default(0)
                .interact()?;

            let api_key = dialoguer::Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter your Anthropic API key")
                .interact()?;

            AIProvider::Claude {
                api_key,
                model: models[model_idx].to_string(),
            }
        }
        2 => {
            let models = vec![
                "gemini-1.5-pro-latest",
                "gemini-1.0-pro",
                "gemini-1.5-flash",
                "gemini-1.5-flash-8b",
                "gemini-2.0-flash-exp",
            ];
            let model_idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a model")
                .items(&models)
                .default(0)
                .interact()?;

            let api_key = dialoguer::Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter your Google API key")
                .interact()?;

            AIProvider::Gemini {
                api_key,
                model: models[model_idx].to_string(),
            }
        }
        _ => unreachable!(),
    };

    Ok(provider)
}

pub async fn select_target_language() -> Result<String> {
    let languages = vec![
        "Chinese (Simplified) - zh-CN",
        "Chinese (Traditional) - zh-TW",
        "Japanese - ja",
        "Korean - ko",
        "French - fr",
        "German - de",
        "Italian - it",
        "Spanish - es",
        "Portuguese - pt",
        "Russian - ru",
        "Arabic - ar",
        "Hindi - hi",
        "Vietnamese - vi",
        "Thai - th",
        "Indonesian - id",
    ];

    let language_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select target language")
        .items(&languages)
        .default(0)
        .interact()?;

    Ok(languages[language_idx].split(" - ").nth(1).unwrap().to_string())
}

pub async fn select_mode() -> Result<bool> {
    let modes = vec![
        "Interactive mode (translate one string at a time)",
        "Batch mode (translate all strings in a file)",
    ];

    let mode_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select translation mode")
        .items(&modes)
        .default(0)
        .interact()?;

    Ok(mode_idx == 0)
}

pub async fn select_ai_provider_for_test() -> Result<AIProvider> {
    let providers = vec![
        "OpenAI (GPT-4, GPT-3.5)",
        "Anthropic (Claude)",
        "Google (Gemini)",
    ];

    let provider_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an AI provider for testing")
        .items(&providers)
        .default(0)
        .interact()?;

    let provider = match provider_idx {
        0 => AIProvider::OpenAI {
            api_key: "test".to_string(),
            model: "gpt-4".to_string(),
        },
        1 => AIProvider::Claude {
            api_key: "test".to_string(),
            model: "claude-3-opus-20240229".to_string(),
        },
        2 => AIProvider::Gemini {
            api_key: "test".to_string(),
            model: "gemini-1.5-pro-latest".to_string(),
        },
        _ => unreachable!(),
    };

    Ok(provider)
} 