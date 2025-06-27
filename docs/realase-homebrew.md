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