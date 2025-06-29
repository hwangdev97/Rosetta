pub mod ai_provider;
pub mod ascii_art;
pub mod config;
pub mod error;
pub mod key_mappings;
pub mod onboarding;
pub mod translator;
pub mod ui;
pub mod xcstrings;

use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum TranslationMode {
    /// Supplement missing translations only
    Supplement,
    /// Fresh translation for all keys (overwrites existing)
    Fresh,
}

pub use ai_provider::AIProvider;
pub use config::Config;
pub use error::TranslatorError as Error;
pub use translator::Translator; 