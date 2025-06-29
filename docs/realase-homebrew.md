Search files...

Ran tool



Read file: .github/workflows/release-homebrew.yml

Ran tool

已在仓库根目录创建新的 GitHub Action 工作流 `.github/workflows/release-homebrew.yml`  
核心逻辑：

1. 触发条件  
   • 仅当推送符合 `v*.*.*` 格式的 tag（如 `v1.2.3`）时执行。

2. Job：`bump-formula`  
   • 先解析出纯版本号（去掉 `refs/tags/` 前缀）。  
   • 调用 dawidd6/action-homebrew-bump-formula@v5  
     - 使用你在 `secrets.HOMEBREW_PAT` 中保存的 PAT（需 repo + workflow 权限）  
     - 自动在 `hwangdev97/homebrew-tools` tap 里更新 `rosetta` 公式，URL 指向本仓库该 tag 的源码压缩包。  
     - 会自动创建（或更新）PR 并填好 SHA-256。

这样当你：

```bash
git tag v1.2.3
git push origin v1.2.3
```

Action 会自动跑完并在 tap 仓库生成更新 PR，合并后用户就能直接：

```bash
brew update
brew install hwangdev97/tools/rosetta
```

补充事项  
• 请在 **本仓库 Settings → Secrets → Actions** 中新增 `HOMEBREW_PAT`，其权限至少包含 `repo` 与 `workflow`。  
• 如需修改 tap 或 formula 名称，只需编辑 workflow 中 `tap` / `formula` 字段。


在 Homebrew 里“注册”开源 CLI 有两条路线，取决于您希望用户如何安装：

1. 私有/个人 Tap  
   `brew tap hwangdev97/tools && brew install rosetta`
2. 官方 **homebrew-core**  
   直接 `brew install rosetta`

下面分别说明如何“注册”。

────────────────────────────────────────
路线 A：发布到自己的 Tap（最快、最灵活）
────────────────────────────────────────
1. 建仓库  
   • GitHub 上创建公开仓库：`homebrew-tools`  
   • 目录结构：`Formula/rosetta.rb`

2. 准备公式  
   • 参考您项目里的 `rosetta.rb`  
   • 确保 `url` 指向 GitHub 发行版 tarball  
     `https://github.com/hwangdev97/Rosetta/archive/v0.1.4.tar.gz`  
   • 计算 SHA-256：  
     ```bash
     shasum -a 256 v0.1.4.tar.gz
     ```

3. 提交后测试  
   ```bash
   brew tap hwangdev97/tools
   brew install --build-from-source hwangdev97/tools/rosetta
   ```

4. 自动化更新  
   您已经用 GitHub Action 配置了 `dawidd6/action-homebrew-bump-formula`，每次打 tag 就会在 tap 仓库发 PR 更新公式；合并后用户 `brew update` 即可得到新版本。

优点：流程可控、审核宽松；缺点：用户需先 `brew tap`。

────────────────────────────────────────
路线 B：提交到 Homebrew/homebrew-core（让用户免 tap）
────────────────────────────────────────
官方“注册”就是把公式合并进 [homebrew/homebrew-core](https://github.com/Homebrew/homebrew-core)。步骤略严格，但一旦通过，用户可直接 `brew install rosetta`。

⚠️ 前置条件  
• 开源项目 **≥30 ⭐**（软要求，但小众项目常被要求达到这一活跃度）  
• 至少发布一个 _稳定_ tag（如 v0.1.4）  
• 构建时间 ≤30 分钟，生成产物 ≤100 MB

1. 本地准备环境  
   ```bash
   brew install git hub  # hub 可选，方便提交 PR
   brew update-reset     # 确保 Homebrew 最新
   ```

2. Fork & clone  
   ```bash
   git clone https://github.com/your-name/homebrew-core
   cd homebrew-core
   git remote add upstream https://github.com/Homebrew/homebrew-core
   git fetch upstream
   git checkout -b rosetta
   ```

3. 生成公式草稿  
   ```bash
   brew create https://github.com/hwangdev97/Rosetta/archive/v0.1.4.tar.gz \
     --no-fetch \
     --tap=homebrew/core
   # 会打开编辑器，让你填 desc、homepage、sha256、build 命令（cargo install ...）及 test do 块
   ```

4. 本地验证  
   ```bash
   brew audit --new-formula rosetta
   brew install --build-from-source rosetta
   brew test rosetta
   ```

   - `audit` 必须无严重错误  
   - `test` 区块需最少运行 `rosetta --version`，确保二进制可执行

5. 提交 PR  
   ```bash
   git add Formula/rosetta.rb
   git commit -m "rosetta 0.1.4 (new formula)"
   git push origin rosetta
   # 打开 PR 指向 homebrew/homebrew-core
   ```

6. 审核与 CI  
   • Homebrew CI 会在 Intel & Apple Silicon 架构上编译、跑 `brew test`  
   • 维护者可能提出修改意见（命名、依赖、测试、描述等）  
   • 通过后合并，Bottles 会自动上传到 GitHub Releases，几小时内全球可用

以后更新版本时，只需跑命令：

```bash
brew bump-formula-pr --version 0.1.5 --url https://github...v0.1.5.tar.gz rosetta
```

或依旧使用 GitHub Actions（`dawidd6/action-homebrew-bump-formula`）监听新 tag，直接 PR 到 **homebrew-core**。

────────────────────────────────────────
常见坑 & 小贴士
────────────────────────────────────────
• **Rust 项目**：公式里通常用  
  ```ruby
  system "cargo", "install", *std_cargo_args
  ```  
  Homebrew CI 会自己安装 Rust。

• **瓶子（Bottle）**：Homebrew CI 会自动生成，无须手动上传。

• **依赖**：若编译依赖非系统库，需要 `depends_on`，但尽量减少外部依赖以避免审核阻碍。

• **测试块**：必须写；哪怕只是运行 `rosetta --help` 并断言输出中包含某关键字。

────────────────────────────────────────

总结  
• 想立刻开始分发——保持个人 tap，流程已自动化。  
• 想“官方”发布——按路线 B 提交 PR 到 `homebrew/homebrew-core`，通过审核后用户无需 tap。

如需更详细示例或遇到审核问题，随时告诉我！