# Rosetta 🌍

[中文说明 (Chinese)](./README.zh-CN.md)

---

## Introduction

> 🎯 **This is a vibe coding project** — A personal passion project built with modern Rust for iOS localization. Feel free to fork, modify, and make it your own! If you find it useful, give it a star ⭐

A modern, blazing-fast CLI tool for translating iOS `.xcstrings` files using multiple AI providers with a beautiful terminal interface.

---

## Features

- 🚀 **Blazing Fast**: Written in Rust for maximum performance
- 🎨 **Beautiful Interface**: Clean terminal UI with progress tracking
- 🤖 **Multi-AI Support**: OpenAI GPT, Anthropic Claude, Google Gemini
- 📱 **iOS Native**: Specifically designed for `.xcstrings` localization files
- ⚡ **Interactive Mode**: Choose what to translate with real-time feedback
- 🔄 **Batch Processing**: Translate multiple keys at once
- 💾 **Auto-Backup**: Automatically backs up your files before translation
- 🔍 **Smart Detection**: Auto-detects project structure and files
- 🌐 **Multi-Language**: Supports 30+ languages including CJK languages

---

## Installation

**Download from Releases (Recommended)**
```bash
# Download the latest binary from GitHub releases
# https://github.com/hwangdev97/Rosetta/releases
```

**Via Homebrew**
```bash
brew tap hwangdev97/tools
brew install rosetta
```

**Build from Source**
```bash
git clone https://github.com/hwangdev97/Rosetta.git
cd Rosetta
chmod +x build.sh
./build.sh
```

**Via Cargo**
```bash
cargo install --git https://github.com/hwangdev97/Rosetta.git
```

---

## Quick Start

### Setup

1. **Configure AI Provider**
```bash
rosetta setup
```
Choose your preferred AI provider and enter your API key:
- **OpenAI**: Get API key from [OpenAI Platform](https://platform.openai.com)
- **Anthropic Claude**: Get API key from [Anthropic Console](https://console.anthropic.com)
- **Google Gemini**: Get API key from [Google AI Studio](https://makersuite.google.com)

2. **Verify Setup**
```bash
rosetta config  # View current configuration
rosetta test    # Test AI provider connection
```

---

## Usage

### Basic Translation
```bash
# Translate to Japanese
rosetta translate ja

# Translate to Simplified Chinese
rosetta translate zh-Hans

# Translate to Korean
rosetta translate ko
```

### Advanced Options
```bash
# Specify custom .xcstrings file path
rosetta translate ja --file /path/to/Localizable.xcstrings

# Fresh translation (retranslate all keys)
rosetta translate ja --mode fresh

# Auto-translate all keys without interaction
rosetta translate ja --auto

# Use specific AI model
rosetta translate ja --model gpt-4
```

### Interactive Mode (Default)
```
Translation Task
  Target: ja
  Mode: Supplement (skip existing)
  Keys: 25

Key: "Good morning, how are you today?"
❯ Translate
  Mark as no translation needed
  Batch translate next 30
  Skip
  Save and exit
```

---

## Supported Languages

| Code | Language | Code | Language |
|------|----------|------|----------|
| `ja` | Japanese | `fr` | French |
| `zh-Hans` | Simplified Chinese | `de` | German |
| `zh-Hant` | Traditional Chinese | `es` | Spanish |
| `ko` | Korean | `pt-PT` | Portuguese (Portugal) |
| `it` | Italian | `pt-BR` | Portuguese (Brazil) |
| `ru` | Russian | `ar` | Arabic |
| `hi` | Hindi | `tr` | Turkish |
| `nl` | Dutch | `pl` | Polish |
| `sv` | Swedish | `no` | Norwegian |
| `da` | Danish | `fi` | Finnish |
| `cs` | Czech | `ro` | Romanian |
| `uk` | Ukrainian | `el` | Greek |
| `he` | Hebrew | `id` | Indonesian |
| `th` | Thai | `vi` | Vietnamese |
| `ml` | Malayalam | `en-US` | English (US) |
| `en-GB` | English (UK) | `en-AU` | English (Australia) |

---

## Commands

### `rosetta translate`
Translate `.xcstrings` files to your target language. See above for options.

### `rosetta clean`
Easily remove backup files:
```bash
rosetta clean
```
- Scans for all `.xcstrings.backup_*` files in the current (or specified) directory (recursively).
- Lists all found backup files with size and date.
- You can:
  - Delete all backups at once
  - Select files to delete interactively (use ↑↓ to move, space to select, enter to confirm, and a final confirmation before deletion)
  - Cancel the operation

**Optional:**
```bash
rosetta clean --directory /path/to/your/project
```

---

## Contributing

Want to contribute or customize?
1. Fork the repository
2. Make your changes
3. Submit a pull request or keep it for yourself!

The codebase is well-structured and documented, making it easy to:
- Add new AI providers
- Implement custom translation logic
- Extend language support
- Improve the UI

---

## License

MIT License — see [LICENSE](LICENSE) for details.

## Acknowledgements

- [OpenAI](https://openai.com) for GPT models
- [Anthropic](https://anthropic.com) for Claude
- [Google](https://ai.google.dev) for Gemini
- The Rust community for awesome crates

---

Vibe coding with cursor 🧑‍💻 by [Hwang](https://hwang.fun)