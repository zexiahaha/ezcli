use std::fs;
use std::io;
use std::path::Path;

use windows::Win32::Globalization::GetACP;
use windows::Win32::Globalization::WideCharToMultiByte;
use windows::Win32::System::Console::GetConsoleOutputCP;

fn current_cmd_code_page() -> u32 {
    let ocp = unsafe { GetConsoleOutputCP() };

    if ocp == 0 { unsafe { GetACP() } } else { ocp }
}

pub fn write_cmd_script_with_current_code_page(path: &Path, content: &str) -> io::Result<()> {
    let code_page = current_cmd_code_page();

    let wide: Vec<u16> = content.encode_utf16().collect();

    if wide.is_empty() {
        return fs::write(path, []);
    }

    let require_len = unsafe { WideCharToMultiByte(code_page, 0, &wide, None, None, None) };

    if require_len <= 0 {
        return Err(io::Error::last_os_error());
    }

    let mut bytes = vec![0u8; require_len as usize];

    let written_len =
        unsafe { WideCharToMultiByte(code_page, 0, &wide, Some(&mut bytes), None, None) };

    if written_len <= 0 {
        return Err(io::Error::last_os_error());
    }

    bytes.truncate(written_len as usize);

    fs::write(path, bytes)
}
