use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShellKind {
    Cmd,
    Powershell,
}

#[derive(Clone, Default, Eq, PartialEq)]
pub struct ScriptPlan {
    pub set_env: BTreeMap<String, String>,
    pub prepend_path: Vec<PathBuf>,
    pub cwd: Option<PathBuf>,
}

fn ps_quote(value: &str) -> String {
    value.replace('\'', "''")
}

pub fn render_powershell_script(plan: &ScriptPlan) -> String {
    let mut lines = Vec::new();

    for (key, value) in &plan.set_env {
        lines.push(format!("$env:{key} = '{}'", ps_quote(value)));
    }

    if !plan.prepend_path.is_empty() {
        let joined = plan
            .prepend_path
            .iter()
            .map(|path| ps_quote(&path.to_string_lossy()))
            .collect::<Vec<_>>()
            .join(";");

        lines.push(format!("$env:Path = '{joined};' + $env:Path"));
    }

    if let Some(path) = &plan.cwd {
        lines.push(format!(
            "Set-Location -LiteralPath '{}'",
            ps_quote(&path.to_string_lossy())
        ));
    }

    format!("{}", lines.join("\n"))
}

fn cmd_quote(value: &str) -> String {
    value.replace('"', "\"\"")
}

pub fn render_cmd_script(plan: &ScriptPlan) -> String {
    let mut lines = Vec::new();

    for (key, value) in &plan.set_env {
        lines.push(format!("set \"{key}={}\"", cmd_quote(value)));
    }

    if !plan.prepend_path.is_empty() {
        let joined = plan
            .prepend_path
            .iter()
            .map(|path| cmd_quote(&path.to_string_lossy()))
            .collect::<Vec<_>>()
            .join(";");

        lines.push(format!("set \"PATH={joined};%PATH%\""));
    }

    if let Some(path) = &plan.cwd {
        lines.push(format!("cd /d \"{}\"", cmd_quote(&path.to_string_lossy())));
    }

    format!("{}\r\n", lines.join("\r\n"))
}
