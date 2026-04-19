# ezcli

[English README](README.md)

`ezcli` 是一个面向 Windows shell 的可扩展开发辅助工具。

它当前重点解决开发工作流中的一些常见问题，例如加载 MSVC `cl` 编译环境、切换到已配置的项目目录，并让这些变化真正作用到当前的 PowerShell 或 `cmd` 会话中。它未来仍然会围绕开发工作流继续扩展，但主线会始终保持清晰、显式、可预期，并尽量符合 shell 工具的使用习惯。

## 安装

### 从 crates.io 安装

```powershell
cargo install ezcli-win
```

在 `crates.io` 上的包名是 `ezcli-win`，但安装后的命令仍然是：

```powershell
ezcli
```

### 从本地仓库安装

```powershell
cargo install --path .
```

如果你更习惯手动构建，也可以执行：

```powershell
cargo build --release
```

如果你希望直接运行 `ezcli`，请确保可执行文件所在目录已经加入 `PATH`，或者直接从构建输出目录运行。

## 快速开始

### 1. 配置 `vcvarsall.bat`

执行：

```powershell
ezcli --find-cl
```

这一步会：

- 打开文件选择器，让你选择 `vcvarsall.bat`
- 更新 `%USERPROFILE%\.ezcli\ezcli.toml`
- 安装 `cmd` wrapper
- 生成 PowerShell wrapper
- 询问是否把 PowerShell wrapper 写入当前用户的 profile

### 2. 添加项目

执行：

```powershell
ezcli --add-project handmade
```

在选择完项目目录后，就可以通过下面的命令快速进入：

```powershell
ep handmade
```

### 3. 日常使用

当前推荐的日常命令：

PowerShell:

```powershell
ecl
ep handmade
```

cmd:

```cmd
ecl
ep handmade
```

其中：

- `ecl`：把 MSVC `cl` 环境加载到当前 shell 会话
- `ep <name>`：进入指定项目目录，并把该目录前置到当前会话 `PATH`

## 当前能力

- 加载 MSVC `cl` 环境到当前 shell 会话
- 进入配置中的项目目录
- 在进入项目时把项目路径前置到当前会话 `PATH`
- 同时支持 PowerShell 和 `cmd`
- 提供短命令与长命令两组入口

## 为什么需要 wrapper

普通 CLI 是由 shell 启动的子进程。子进程可以修改自己的环境变量和当前目录，但这些变化不会自动回写到父 shell。

因此，`ezcli` 采用两层设计：

- `emit`：负责生成目标 shell 的脚本文本并输出到标准输出
- `wrapper`：负责让当前 shell 执行这些脚本，从而真正修改当前会话

这就是为什么 `ecl` 和 `ep` 能够影响当前 PowerShell 或 `cmd`，而不是只在一个短命的子进程里生效。

## 项目定位

- 面向 Windows shell 工作流，而不只是一次性执行的 CLI
- 以开发效率为重点，但不绑定在某一种语言或某一种项目类型上
- 适合承载环境加载、项目切换，以及后续更多与开发工作流相关的命令
- 强调显式调用，避免新功能污染已有的日常高频入口

如果未来继续扩展，新的命令也应尽量围绕开发工作流组织，而不是混入无关用途。

## 常用命令

### Wrapper 入口

- `ecl`
- `ep <name>`
- `ezcli-load-cl`
- `ezcli-enter-project <name>`

### CLI 入口

- `ezcli --find-cl`
- `ezcli --show-cl`
- `ezcli --add-project <name>`
- `ezcli --show-project`
- `ezcli --del-project`

### 高级命令

这些命令主要用于调试、理解底层行为，或者手动集成 wrapper：

- `ezcli emit --shell powershell load-cl`
- `ezcli emit --shell powershell enter-project <name>`
- `ezcli emit --shell cmd load-cl`
- `ezcli emit --shell cmd enter-project <name>`
- `ezcli emit --shell powershell init`
- `ezcli emit --shell powershell show-profile`
- `ezcli emit --shell powershell install-profile`
- `ezcli emit --shell cmd install-wrapper`

## 配置文件

默认配置文件路径：

```text
%USERPROFILE%\.ezcli\ezcli.toml
```

当前配置结构大致如下：

```toml
vc_path = "C:\\Program Files\\Microsoft Visual Studio\\...\\vcvarsall.bat"
default_arch = "x64"

[[projects]]
name = "handmade"
path = "D:\\code\\handmade"
```

其中：

- `vc_path`：`vcvarsall.bat` 的路径
- `default_arch`：默认架构参数
- `projects`：项目名与项目路径列表

## 当前限制

- 当前主要面向 Windows
- 当前 wrapper 只覆盖 PowerShell 和 `cmd`
- 反复执行 `ep <name>` 时，当前会话里的 `PATH` 可能出现重复项
- 这份 README 主要记录稳定可用能力，更细的实现细节更适合放到独立文档中

## 当前状态

`ezcli` 已经完成从“直接执行逻辑的 CLI”到“shell wrapper 驱动工具”的核心改造。

PowerShell 和 `cmd` 两条链路都已经打通，`ecl` 与 `ep <name>` 也已经可以作为日常入口使用。

## 后续方向

后续合理的推进方向包括：

- 增加更多使用示例
- 随着功能增长进一步整理命令分组
- 处理 `PATH` 去重
- 在 `docs/` 下补充独立文档
- 在不破坏主线心智模型的前提下增加更多开发工作流能力

## 许可证

本项目使用双许可证：`MIT` 或 `Apache-2.0`。
