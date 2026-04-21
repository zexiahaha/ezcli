# 更新日志

本文档记录 `ezcli` 各版本的重要变更。

当前采用按版本倒序、按变更类型分组的方式维护。

## [未发布]

- 暂无未发布变更。

## [0.1.2] - 2026-04-21

### Fixed

- 真正修复了 `cmd` / Cmder 环境下 wrapper 在中文路径场景中的可用性问题。
- 修复了临时 `.cmd` 脚本生成受 shell 重定向与编码链路影响的问题。
- 修复了 `ecl` 之后 `TEMP`、`TMP`、`USERPROFILE`、`APPDATA` 与 `LOCALAPPDATA` 等环境变量可能被污染的问题。
- 修复了加载 MSVC 环境后 `ecl` 与 `ep` wrapper 可能不可见的问题。

### Changed

- `cmd` 固定 wrapper 改为 ASCII-only，不再硬编码中文绝对路径。
- `emit load-cl` 与 `emit enter-project` 支持 `--output`，由 `ezcli` 直接写入临时脚本。
- 临时 `cmd` 脚本改为按当前 `cmd` 代码页写入。
- `env_capture` 改为在同一 `cmd.exe` 进程中抓取 before/after 环境并只应用 diff。

## [0.1.1] - 2026-04-20

### Fixed

- 修复了 PowerShell wrapper 与 profile 在中文路径下的乱码与执行异常问题。
- 修复了 PowerShell 中带括号环境变量名的渲染问题，例如 `CommonProgramFiles(x86)`。

### Changed

- PowerShell 相关脚本与 profile 写入统一改为 `UTF-8 + BOM`。

## [0.1.0] - 2026-04-20

### Added

- 首个可用版本发布。
- 提供基于 shell wrapper 的工作流，使环境变更可以真正作用到当前 shell 会话。
- 提供 `ecl` 命令，用于将 MSVC `cl` 环境加载到当前 shell。
- 提供 `ep <name>` 命令，用于快速进入已配置的项目目录。
