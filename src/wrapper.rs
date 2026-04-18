use std::env::home_dir;
use std::fs::{self, create_dir_all};
use std::io;
use std::path::PathBuf;

fn ps_quote(value: &str) -> String {
    value.replace('\'', "''")
}

pub fn render_powershell_wrapper_script(program: &str) -> String {
    let program = ps_quote(program);

    format!(
        r#"
function ezcli-load-cl {{
    & '{program}' emit --shell powershell load-cl | Invoke-Expression
    Write-Host "cl environment loaded" -ForegroundColor Green
}}

function ezcli-enter-project {{
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    & '{program}' emit --shell powershell enter-project $Name | Invoke-Expression
}}
Set-Alias ecl ezcli-load-cl
Set-Alias ep ezcli-enter-project
        "#
    )
}

pub fn get_powershell_wrapper_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home =
        home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "get home dir failed"))?;

    Ok(home.join(".ezcli").join("ezcli-wrapper.ps1"))
}

pub fn save_powershell_wrapper_script(
    program: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let wrapper_path = get_powershell_wrapper_path()?;

    if let Some(parent) = wrapper_path.parent() {
        create_dir_all(parent)?;
    }

    let script = render_powershell_wrapper_script(program);
    fs::write(&wrapper_path, script)?;

    Ok(wrapper_path)
}

pub fn get_powershell_profile_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home =
        home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "get home dir failed"))?;

    Ok(home
        .join("Documents")
        .join("WindowsPowerShell")
        .join("Microsoft.PowerShell_profile.ps1"))
}

pub fn build_powershell_profile_source_line() -> Result<String, Box<dyn std::error::Error>> {
    let wrapper_path = get_powershell_wrapper_path()?;
    let source_line = ps_quote(&wrapper_path.to_string_lossy());

    Ok(format!(". '{}'", source_line))
}

pub fn install_powershell_profile_source_line() -> Result<bool, Box<dyn std::error::Error>> {
    let profile_path = get_powershell_profile_path()?;
    let source_line = build_powershell_profile_source_line()?;

    if let Some(parent) = profile_path.parent() {
        create_dir_all(parent)?;
    }

    let mut content = match fs::read_to_string(&profile_path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.into()),
    };

    if content
        .lines()
        .any(|line| line.trim() == source_line.as_str())
    {
        return Ok(false);
    }

    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }

    content.push_str(&source_line);
    content.push('\n');

    fs::write(profile_path, content)?;

    Ok(true)
}

fn cmd_quote(value: &str) -> String {
    value.replace('"', "\"\"")
}

fn render_cmd_wrapper_script(program: &str, action_args: &str, temp_stem: &str) -> String {
    let program = cmd_quote(program);

    [
        "@echo off".to_string(),
        format!(r#"set "EZCLI_WRAPPER_TMP=%TEMP%\{temp_stem}-%RANDOM%%RANDOM%.cmd""#),
        format!(r#""{program}" emit --shell cmd {action_args} > "%EZCLI_WRAPPER_TMP%""#),
        "if errorlevel 1 goto :cleanup".to_string(),
        r#"call "%EZCLI_WRAPPER_TMP%""#.to_string(),
        ":cleanup".to_string(),
        r#"if exist "%EZCLI_WRAPPER_TMP%" del "%EZCLI_WRAPPER_TMP%""#.to_string(),
        r#"set "EZCLI_WRAPPER_TMP=""#.to_string(),
        String::new(),
    ]
    .join("\r\n")
}

pub fn render_cmd_load_cl_wrapper_script(program: &str) -> String {
    // render_cmd_wrapper_script(program, "load-cl", "ezcli-load-cl")
    let program = cmd_quote(program);

    [
        "@echo off".to_string(),
        format!(r#"set "EZCLI_WRAPPER_TMP=%TEMP%\ezcli-load-cl-%RANDOM%%RANDOM%.cmd""#),
        format!(r#""{program}" emit --shell cmd load-cl > "%EZCLI_WRAPPER_TMP%""#),
        "if errorlevel 1 goto :cleanup".to_string(),
        r#"call "%EZCLI_WRAPPER_TMP%""#.to_string(),
        r#"echo cl environment loaded"#.to_string(),
        ":cleanup".to_string(),
        r#"if exist "%EZCLI_WRAPPER_TMP%" del "%EZCLI_WRAPPER_TMP%""#.to_string(),
        r#"set "EZCLI_WRAPPER_TMP=""#.to_string(),
        String::new(),
    ]
    .join("\r\n")
}

pub fn render_cmd_enter_project_wrapper_script(program: &str) -> String {
    render_cmd_wrapper_script(program, "enter-project %*", "ezcli-enter-project")
}

pub fn render_cmd_ecl_wrapper_script(program: &str) -> String {
    // render_cmd_wrapper_script(program, "load-cl", "ecl")
    let program = cmd_quote(program);

    [
        "@echo off".to_string(),
        format!(r#"set "EZCLI_WRAPPER_TMP=%TEMP%\ecl-%RANDOM%%RANDOM%.cmd""#),
        format!(r#""{program}" emit --shell cmd load-cl > "%EZCLI_WRAPPER_TMP%""#),
        "if errorlevel 1 goto :cleanup".to_string(),
        r#"call "%EZCLI_WRAPPER_TMP%""#.to_string(),
        r#"echo cl environment loaded"#.to_string(),
        ":cleanup".to_string(),
        r#"if exist "%EZCLI_WRAPPER_TMP%" del "%EZCLI_WRAPPER_TMP%""#.to_string(),
        r#"set "EZCLI_WRAPPER_TMP=""#.to_string(),
        String::new(),
    ]
    .join("\r\n")
}

pub fn render_cmd_ep_wrapper_script(program: &str) -> String {
    render_cmd_wrapper_script(program, "enter-project %*", "ep")
}

pub fn get_cmd_load_cl_wrapper_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home =
        home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "get home dir failed"))?;
    Ok(home.join(".ezcli").join("ezcli-load-cl.cmd"))
}

pub fn get_cmd_enter_project_wrapper_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home =
        home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "get home dir failed"))?;
    Ok(home.join(".ezcli").join("ezcli-enter-project.cmd"))
}
pub fn get_cmd_ecl_wrapper_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home =
        home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "get home dir failed"))?;
    Ok(home.join(".ezcli").join("ecl.cmd"))
}

pub fn get_cmd_ep_wrapper_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home =
        home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "get home dir failed"))?;
    Ok(home.join(".ezcli").join("ep.cmd"))
}

pub fn save_cmd_wrapper_scripts(program: &str) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let load_cl_path = get_cmd_load_cl_wrapper_path()?;
    let enter_project_path = get_cmd_enter_project_wrapper_path()?;
    let ecl_path = get_cmd_ecl_wrapper_path()?;
    let ep_path = get_cmd_ep_wrapper_path()?;

    if let Some(parent) = load_cl_path.parent() {
        create_dir_all(parent)?;
    }

    fs::write(&load_cl_path, render_cmd_load_cl_wrapper_script(program))?;
    fs::write(
        &enter_project_path,
        render_cmd_enter_project_wrapper_script(program),
    )?;
    fs::write(&ecl_path, render_cmd_ecl_wrapper_script(program))?;
    fs::write(&ep_path, render_cmd_ep_wrapper_script(program))?;

    Ok(vec![load_cl_path, enter_project_path])
}
