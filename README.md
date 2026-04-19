# ezcli

[中文说明](README.zh.md)

`ezcli` is an extensible Windows shell helper for developer workflows.

It currently focuses on tasks such as loading the MSVC `cl` environment and switching into configured project directories, while making those changes apply to the current PowerShell or `cmd` session. The long-term direction is still developer workflow tooling, with a strong preference for behavior that is explicit, predictable, and shell-friendly.

## Install

### From crates.io

```powershell
cargo install ezcli-win
```

The package name on `crates.io` is `ezcli-win`, but the installed command is still:

```powershell
ezcli
```

### From the local repository

```powershell
cargo install --path .
```

If you prefer, you can also build it manually:

```powershell
cargo build --release
```

If you want to run `ezcli` directly, make sure the executable directory is available in `PATH`, or run it from the build output directory.

## Quick start

### 1. Configure `vcvarsall.bat`

Run:

```powershell
ezcli --find-cl
```

This step will:

- open a file picker so you can select `vcvarsall.bat`
- update `%USERPROFILE%\.ezcli\ezcli.toml`
- install the `cmd` wrappers
- generate the PowerShell wrapper
- ask whether the PowerShell wrapper should be added to the current user's profile

### 2. Add a project

Run:

```powershell
ezcli --add-project handmade
```

After choosing the project directory, you can enter it quickly with:

```powershell
ep handmade
```

### 3. Daily usage

Recommended daily commands:

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

Where:

- `ecl` loads the MSVC `cl` environment into the current shell session
- `ep <name>` enters a configured project directory and prepends it to the current session `PATH`

## What it does today

- Load the MSVC `cl` environment into the current shell session
- Enter a configured project directory
- Prepend that project path to the current session `PATH`
- Support both PowerShell and `cmd`
- Provide both short commands and long command names

## Why wrappers are needed

A normal CLI runs as a child process of the shell. It can modify its own environment variables and working directory, but those changes do not flow back into the parent shell automatically.

Because of that, `ezcli` uses a two-layer design:

- `emit`: generates shell script text and writes it to standard output
- `wrapper`: makes the current shell execute that script so the current session is actually updated

That is why `ecl` and `ep` can affect the current PowerShell or `cmd` session instead of only taking effect inside a short-lived child process.

## Positioning

- Built for Windows shell workflows instead of being just a one-shot CLI
- Focused on developer productivity without being tied to a single language or a single project type
- Suitable for environment loading, project switching, and future workflow-related commands
- Designed around explicit invocation so new features do not pollute existing daily commands

If the tool grows over time, new commands should still be organized around developer workflows instead of mixing in unrelated concerns.

## Common commands

### Wrapper entry points

- `ecl`
- `ep <name>`
- `ezcli-load-cl`
- `ezcli-enter-project <name>`

### CLI entry points

- `ezcli --find-cl`
- `ezcli --show-cl`
- `ezcli --add-project <name>`
- `ezcli --show-project`
- `ezcli --del-project`

### Advanced commands

These are mainly useful for debugging, understanding the lower-level behavior, or integrating wrappers manually:

- `ezcli emit --shell powershell load-cl`
- `ezcli emit --shell powershell enter-project <name>`
- `ezcli emit --shell cmd load-cl`
- `ezcli emit --shell cmd enter-project <name>`
- `ezcli emit --shell powershell init`
- `ezcli emit --shell powershell show-profile`
- `ezcli emit --shell powershell install-profile`
- `ezcli emit --shell cmd install-wrapper`

## Configuration

Default config file path:

```text
%USERPROFILE%\.ezcli\ezcli.toml
```

The current config structure looks roughly like this:

```toml
vc_path = "C:\\Program Files\\Microsoft Visual Studio\\...\\vcvarsall.bat"
default_arch = "x64"

[[projects]]
name = "handmade"
path = "D:\\code\\handmade"
```

Where:

- `vc_path` is the path to `vcvarsall.bat`
- `default_arch` is the default architecture argument
- `projects` stores project names and project paths

## Current limitations

- The tool currently targets Windows
- Wrappers currently exist only for PowerShell and `cmd`
- Repeatedly running `ep <name>` may introduce duplicate entries into the current session `PATH`
- This README focuses on stable usage; more detailed implementation notes are better kept in separate documents

## Current status

`ezcli` has already completed the core transition from a direct-action CLI into a shell-wrapper-driven tool.

Both the PowerShell and `cmd` paths are working, and `ecl` plus `ep <name>` are already usable as daily commands.

## Next directions

Reasonable next steps include:

- adding more usage examples
- organizing commands more clearly as the tool grows
- adding `PATH` deduplication
- writing separate documents under `docs/`
- extending the tool with more developer workflow features without breaking the main mental model

## License

Licensed under either `MIT` or `Apache-2.0`.
