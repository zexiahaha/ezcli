# Releasing `ezcli`

本文档记录 `ezcli` 目前推荐的发版流程，目标是让每次发布都可重复、可检查、可回溯。

## 版本约定

- 版本号使用语义化版本风格，例如 `0.1.3`
- Git tag 统一使用带 `v` 的格式，例如 `v0.1.3`
- GitHub Release 的标题与 tag 保持一致，例如 `v0.1.3`

## 发版前检查

发版前先确认以下几点：

- 当前改动已经完成，范围明确
- `CHANGELOG.md` 已补充本次版本内容
- 如有需要，`CHANGELOG.zh.md` 也已同步
- `Cargo.toml` 中的 `version` 已更新到目标版本
- 工作区干净，没有不想纳入发版的临时修改

可用命令：

```powershell
git status --short
```

## 更新版本与日志

1. 在 `Cargo.toml` 中更新版本号
2. 将 `CHANGELOG.md` 中的 `Unreleased` 内容整理到正式版本段落
3. 补充发布日期
4. 如有中文版本日志，保持 `CHANGELOG.zh.md` 同步

建议本次版本日志只写：

- Added：新增能力
- Changed：行为调整或实现改进
- Fixed：用户可感知的问题修复

不要把每一个开发过程中的小提交都写进 changelog。

## 提交发版内容

确认本次发版相关文件后，提交一次明确的版本提交。

可用命令：

```powershell
git add Cargo.toml CHANGELOG.md CHANGELOG.zh.md
git commit -m "0.1.3"
```

如果本次发版还包含其他需要一起提交的文件，也一并加入。

## 创建 Git tag

发版提交完成后，为该提交创建注释 tag：

```powershell
git tag -a v0.1.3 -m "Release v0.1.3"
```

检查 tag：

```powershell
git show v0.1.3 --no-patch
```

## 推送提交与 tag

先推送分支，再推送 tag：

```powershell
git push origin master
git push origin v0.1.3
```

如果默认分支以后改名为 `main`，把上面的 `master` 换成 `main`。

## 创建 GitHub Release

在 GitHub 仓库的 Releases 页面：

1. 点击 `Draft a new release`
2. 选择已有 tag，例如 `v0.1.3`
3. Release title 填 `v0.1.3`
4. Release notes 直接复制 `CHANGELOG.md` 对应版本内容
5. 正文里不要重复复制 `## [0.1.3] - YYYY-MM-DD` 这一行
6. 如果这是当前最新版本，勾选 `Set as the latest release`
7. 点击 `Publish release`

建议：

- Release 正文优先使用手写 changelog
- 不完全依赖 GitHub 自动生成 release notes

## 发版后核对

发布完成后至少确认：

- `Cargo.toml` 版本正确
- GitHub 上已经有对应 tag
- GitHub Releases 页面已经出现对应版本
- Release 标题、tag、changelog 内容一致
- 如需验证安装，可额外检查 `cargo install` 或下载源码是否正常

## `ezcli` 推荐发版命令模板

下面以发布 `0.1.3` 为例：

```powershell
git status --short
git add Cargo.toml CHANGELOG.md CHANGELOG.zh.md
git commit -m "0.1.3"
git tag -a v0.1.3 -m "Release v0.1.3"
git push origin master
git push origin v0.1.3
```

## 简短 checklist

每次发版按下面顺序过一遍即可：

- 更新 `Cargo.toml` 版本号
- 更新 `CHANGELOG.md`
- 同步 `CHANGELOG.zh.md`
- 检查工作区是否干净
- 提交版本提交
- 创建 `vX.Y.Z` tag
- 推送分支
- 推送 tag
- 在 GitHub 创建 Release
- 核对 Release 页面内容
