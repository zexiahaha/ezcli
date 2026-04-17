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
}}

function ezcli-enter-project {{
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    & '{program}' emit --shell powershell enter-project $Name | Invoke-Expression
}}
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
