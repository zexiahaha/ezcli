use std::collections::BTreeMap;
use std::io;
use std::os::windows::process::CommandExt;
use std::process::Command;

use windows::Win32::Globalization::GetACP;
use windows::Win32::Globalization::MULTI_BYTE_TO_WIDE_CHAR_FLAGS;
use windows::Win32::Globalization::MultiByteToWideChar;
use windows::Win32::System::Console::GetConsoleOutputCP;

const ENV_BEFORE_BEGIN: &str = "__EZCLI_ENV_BEFORE_BEGIN__";
const ENV_BEFORE_END: &str = "__EZCLI_ENV_BEFORE_END__";
const ENV_AFTER_BEGIN: &str = "__EZCLI_ENV_AFTER_BEGIN__";
const ENV_AFTER_END: &str = "__EZCLI_ENV_AFTER_END__";

fn build_vcvars_command_line(vc_path: &str, arch: &str) -> String {
    format!(
        r#"echo {before_begin} && set && echo {before_end} && call "{vc_path}" {arch} && echo {after_begin} && set && echo {after_end}"#,
        before_begin = ENV_BEFORE_BEGIN,
        before_end = ENV_BEFORE_END,
        vc_path = vc_path,
        arch = arch,
        after_begin = ENV_AFTER_BEGIN,
        after_end = ENV_AFTER_END
    )
}

fn extract_between_markers<'a>(
    text: &'a str,
    begin_marker: &str,
    end_marker: &str,
) -> io::Result<&'a str> {
    let begin = text.find(begin_marker).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("begin marker not found: {begin_marker}"),
        )
    })?;

    let content_start = begin + begin_marker.len();

    let end = text[content_start..]
        .find(end_marker)
        .map(|offset| content_start + offset)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("end marker not found: {end_marker}"),
            )
        })?;

    Ok((text[content_start..end]).trim_matches(&['\r', '\n'][..]))
}

fn current_cmd_code_page() -> u32 {
    let ocp = unsafe { GetConsoleOutputCP() };

    if ocp == 0 { unsafe { GetACP() } } else { ocp }
}

fn decode_cmd_output_with_current_code_page(bytes: &[u8]) -> io::Result<String> {
    if bytes.is_empty() {
        return Ok(String::new());
    }

    let code_page = current_cmd_code_page();

    let wide_len =
        unsafe { MultiByteToWideChar(code_page, MULTI_BYTE_TO_WIDE_CHAR_FLAGS(0), bytes, None) };

    if wide_len <= 0 {
        return Err(io::Error::last_os_error());
    }

    let mut wide = vec![0u16; wide_len as usize];

    let written_len = unsafe {
        MultiByteToWideChar(
            code_page,
            MULTI_BYTE_TO_WIDE_CHAR_FLAGS(0),
            bytes,
            Some(&mut wide),
        )
    };

    if written_len <= 0 {
        return Err(io::Error::last_os_error());
    }

    wide.truncate(written_len as usize);

    String::from_utf16(&wide).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
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

fn diff_env_vars(
    before: BTreeMap<String, String>,
    after: BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let before_normalized: BTreeMap<String, String> = before
        .into_iter()
        .map(|(key, value)| (key.to_ascii_uppercase(), value))
        .collect();
    after
        .into_iter()
        .filter(
            |(key, after_value)| match before_normalized.get(&key.to_ascii_uppercase()) {
                Some(before_value) => before_value != after_value,
                None => true,
            },
        )
        .collect()
}

pub fn capture_vcvars_env(vc_path: &str, arch: &str) -> io::Result<BTreeMap<String, String>> {
    let output = Command::new("cmd.exe")
        .args(["/d", "/c"])
        .raw_arg(build_vcvars_command_line(vc_path, arch))
        .output()?;

    if !output.status.success() {
        let stderr = decode_cmd_output_with_current_code_page(&output.stderr)
            .unwrap_or_else(|_| String::from_utf8_lossy(&output.stderr).into_owned());
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("vcvarsall failed: {stderr}"),
        ));
    }

    let stdout = decode_cmd_output_with_current_code_page(&output.stdout)?;

    let before_text = extract_between_markers(&stdout, ENV_BEFORE_BEGIN, ENV_BEFORE_END)?;

    let after_text = extract_between_markers(&stdout, ENV_AFTER_BEGIN, ENV_AFTER_END)?;

    let before_value = parse_env_block(before_text);
    let after_value = parse_env_block(after_text);

    Ok(diff_env_vars(before_value, after_value))
}
