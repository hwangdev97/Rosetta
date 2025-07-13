# Rosetta 🌍（中文说明）

[English](./README.md)

---

## 简介

> 🎯 **这是一个 vibe coding 项目** - 一个用现代 Rust 为 iOS 本地化构建的个人激情项目。欢迎 fork、修改，让它成为你自己的！如果你觉得有用，请给个 star ⭐

一个现代化、超级快速的 CLI 工具，使用多个 AI 提供商翻译 iOS `.xcstrings` 文件，拥有精美的终端界面。

---

## 目录
- [特性](#特性)
- [安装](#安装)
- [快速开始](#快速开始)
- [如何执行翻译](#如何执行翻译)
- [支持的语言](#支持的语言)
- [备份清理](#备份清理)
- [Vibe Coding 项目](#vibe-coding-项目)
- [License](#license)
- [致谢](#致谢)

---

### ✨ 特性

- **🚀 超级快速**: 用 Rust 编写，性能最大化
- **🎨 精美界面**: 清爽的终端 UI，带进度跟踪
- **🤖 多 AI 支持**: OpenAI GPT、Anthropic Claude、Google Gemini
- **📱 iOS 原生**: 专为 `.xcstrings` 本地化文件设计
- **⚡ 交互模式**: 实时反馈，选择要翻译的内容
- **🔄 批处理**: 一次翻译多个键值
- **💾 自动备份**: 翻译前自动备份文件
- **🔍 智能检测**: 自动检测项目结构和文件
- **🌐 多语言**: 支持 30+ 种语言，包括中日韩语言

---

## 安装

**📦 从 Release 下载（推荐）**
```bash
# 从 GitHub releases 下载最新二进制文件
# https://github.com/hwangdev97/Rosetta/releases
```

**🍺 通过 Homebrew**
```bash
brew tap hwangdev97/tools
brew install rosetta
```

**⚙️ 从源码构建**
```bash
git clone https://github.com/hwangdev97/Rosetta.git
cd Rosetta
chmod +x build.sh
./build.sh
```

**📦 通过 Cargo**
```bash
cargo install --git https://github.com/hwangdev97/Rosetta.git
```

---

## 快速开始

### 设置

1. **配置 AI 提供商**
```bash
rosetta setup
```
选择你喜欢的 AI 提供商并输入 API 密钥：
- **OpenAI**: 从 [OpenAI Platform](https://platform.openai.com) 获取 API 密钥
- **Anthropic Claude**: 从 [Anthropic Console](https://console.anthropic.com) 获取 API 密钥
- **Google Gemini**: 从 [Google AI Studio](https://makersuite.google.com) 获取 API 密钥

2. **验证设置**
```bash
rosetta config  # 查看当前配置
rosetta test    # 测试 AI 提供商连接
```

---

## 如何执行翻译

**基础翻译**
```bash
# 翻译为日语
rosetta translate ja

# 翻译为简体中文
rosetta translate zh-Hans

# 翻译为韩语
rosetta translate ko
```

**高级选项**
```bash
# 指定自定义 .xcstrings 文件路径
rosetta translate ja --file /path/to/Localizable.xcstrings

# 全新翻译（重新翻译所有键值）
rosetta translate ja --mode fresh

# 自动翻译所有键值，无需交互
rosetta translate ja --auto

# 使用特定 AI 模型
rosetta translate ja --model gpt-4
```

**交互模式（默认）**
```
翻译任务
  目标: ja
  模式: 补充（跳过已有）
  键值: 25

键值: "Good morning, how are you today?"
❯ 翻译
  标记为无需翻译
  批量翻译接下来 30 个
  跳过
  保存并退出
```

---

## 支持的语言

| 代码 | 语言 | 代码 | 语言 |
|------|------|------|------|
| `ja` | 日语 | `fr` | 法语 |
| `zh-Hans` | 简体中文 | `de` | 德语 |
| `zh-Hant` | 繁体中文 | `es` | 西班牙语 |
| `ko` | 韩语 | `pt-PT` | 葡萄牙语（葡萄牙）|
| `it` | 意大利语 | `pt-BR` | 葡萄牙语（巴西）|
| `ru` | 俄语 | `ar` | 阿拉伯语 |
| `hi` | 印地语 | `tr` | 土耳其语 |
| `nl` | 荷兰语 | `pl` | 波兰语 |
| `sv` | 瑞典语 | `no` | 挪威语 |
| `da` | 丹麦语 | `fi` | 芬兰语 |
| `cs` | 捷克语 | `ro` | 罗马尼亚语 |
| `uk` | 乌克兰语 | `el` | 希腊语 |
| `he` | 希伯来语 | `id` | 印尼语 |
| `th` | 泰语 | `vi` | 越南语 |
| `ml` | 马拉雅拉姆语 | `en-US` | 英语（美国）|
| `en-GB` | 英语（英国）| `en-AU` | 英语（澳大利亚）|

---

## 备份清理

**一键清理备份文件：**

```bash
rosetta clean
```
- 自动递归扫描当前（或指定）目录下所有 `.xcstrings.backup_*` 文件
- 列出所有找到的备份文件（含大小、时间）
- 支持：
  - 一键删除全部
  - 交互式选择删除（↑↓移动，空格选择，回车确认，二次确认）
  - 取消操作

**可选：**
```bash
rosetta clean --directory /your/project/path
```

---

## Vibe Coding 项目

这是一个为 iOS 开发社区用心构建的**个人激情项目**。它的设计理念是：

- **🎯 实用**: 解决真实的本地化痛点
- **🛠️ 可定制**: 容易 fork 和自定义
- **🌟 启发性**: 展示现代 Rust 工具的可能性

**想要贡献或自定义？**
1. Fork 这个仓库
2. 进行你的修改
3. 提交 pull request 或者自己保留！

代码库结构良好且有文档，便于：
- 添加新的 AI 提供商
- 实现自定义翻译逻辑
- 扩展语言支持
- 改进用户界面

---

## License

MIT License - 详见 [LICENSE](LICENSE) 文件

## 致谢

- [OpenAI](https://openai.com) 提供 GPT 模型
- [Anthropic](https://anthropic.com) 提供 Claude
- [Google](https://ai.google.dev) 提供 Gemini
- Rust 社区提供优秀的 crates

---

Vibe coding with cursor 🧑‍💻 by [Hwang](https://hwang.fun) 