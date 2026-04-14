use clap::Parser;
use std::env::home_dir;
use std::fs::create_dir_all;
use std::mem;
use windows::Win32::UI::Controls::Dialogs::{
    GetOpenFileNameW, OFN_FILEMUSTEXIST, OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows::core::PWSTR;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    find_cl: bool,

    #[arg(short, long)]
    show_cl: bool,

    #[arg(short, long)]
    add_project: Option<String>,

    #[arg(short, long)]
    path_to_project: Option<String>,

    #[arg(short, long)]
    del_project: bool,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    if cli.find_cl {
        let mut file_buf = [0u16; 260];

        let mut ofn = OPENFILENAMEW {
            lStructSize: mem::size_of::<OPENFILENAMEW>() as u32,
            lpstrFile: PWSTR(file_buf.as_mut_ptr()),
            nMaxFile: file_buf.len() as u32,
            Flags: OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST,
            ..Default::default()
        };
        let str = unsafe {
            if GetOpenFileNameW(&mut ofn).as_bool() {
                let len = file_buf
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(file_buf.len());
                Some(String::from_utf16_lossy(&file_buf[..len]))
            } else {
                None
            }
        };

        let s = str.unwrap_or_else(|| "".to_string());

        println!("file path is {s}");

        if s.as_str().ends_with("vcvarsall.bat") {
            if let Some(home) = home_dir() {
                let config_file_dir = home.join(".ezcli");
                let config_file = config_file_dir.join("/ezcli_config.json");
            } else {
                println!("get home dir failed!");
            }
        } else {
            println!("current file is not cl vcvarsall.bat!");
        }
    }

    Ok(())
}
