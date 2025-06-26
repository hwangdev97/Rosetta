use crate::ascii_art::ROSETTA_LOGO;
use anyhow::Result;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::path::PathBuf;

use crate::config::Config;
use crate::ai_provider::AIProvider;

pub struct OnboardingConfig {
    pub api_key: String,
    pub default_language: String,
    pub project_path: PathBuf,
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
        let providers = vec!["OpenAI", "Claude", "Google Gemini", "Grok"];
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
        let provider = match provider_idx {
            0 => {
                let models = vec!["gpt-4-turbo-preview", "gpt-4", "gpt-3.5-turbo"];
                let model_idx = Select::with_theme(&ColorfulTheme::default())
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
                let model_idx = Select::with_theme(&ColorfulTheme::default())
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
                let models = vec!["gemini-pro", "gemini-pro-vision"];
                let model_idx = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select Gemini model")
                    .items(&models)
                    .default(0)
                    .interact()?;
                
                AIProvider::Gemini {
                    api_key,
                    model: models[model_idx].to_string(),
                }
            }
            3 => AIProvider::Grok {
                api_key,
                model: "grok-1".to_string(),
            },
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
    println!("{}", ascii_art::LOGO);
    println!("Welcome to Rosetta! Let's get you set up.\n");

    let theme = ColorfulTheme::default();

    // Select AI provider
    let providers = vec!["OpenAI", "Claude", "Google Gemini", "Grok"];
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
            let models = vec!["gemini-pro", "gemini-pro-vision"];
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
        3 => AIProvider::Grok {
            api_key,
            model: "grok-1".to_string(),
        },
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