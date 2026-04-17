use std::collections::BTreeMap;
use std::io;
use std::os::windows::process::CommandExt;
use std::process::Command;

fn build_vcvars_command_line(vc_path: &str, arch: &str) -> String {
    format!(r#"call "{}" {} && set"#, vc_path, arch)
}

fn parse_env_block(output: &str) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();

    for line in output.lines() {
        if let Some((key, value)) = line.split_once('=') {
            if !key.trim().is_empty() {
                values.insert(key.trim().to_string(), value.to_string());
            }
        }
    }

    values
}

pub fn capture_vcvars_env(vc_path: &str, arch: &str) -> io::Result<BTreeMap<String, String>> {
    let output = Command::new("cmd.exe")
        .args(["/d", "/c"])
        .raw_arg(build_vcvars_command_line(vc_path, arch))
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("vcvarsall failed: {stderr}"),
        ));
    }

    Ok(parse_env_block(&String::from_utf8_lossy(&output.stdout)))
}
