use clap::Parser;
use serde::{Deserialize, Serialize};
use std::env::home_dir;
use std::fs::{self, create_dir_all};
use std::mem;
use windows::Win32::UI::Controls::Dialogs::{
    GetOpenFileNameW, OFN_FILEMUSTEXIST, OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows::core::PWSTR;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub vcvars_path: String,
    pub default_arch: String,
    pub projects: Vec<Project>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Project {
    pub name: String,
    pub path: String,
}

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

        let cl_str = str.unwrap_or_else(|| "".to_string());

        println!("file path is {cl_str}");

        if cl_str.as_str().ends_with("vcvarsall.bat") {
            if let Some(home) = home_dir() {
                let config_file_dir = home.join(".ezcli");
                let config_file = config_file_dir.join("/ezcli.toml");

                if !config_file_dir.exists() {
                    create_dir_all(config_file_dir)?;
                }
                if !config_file.exists() {
                    let default_config = Config {
                        vcvars_path: cl_str,
                        default_arch: "x64".to_string(),
                        projects: vec![],
                    };

                    let toml_content = toml::to_string_pretty(&default_config)?;

                    fs::write(&config_file, toml_content)?;

                    println!("already create new ezcli.toml!");
                } else {
                }
            } else {
                println!("get home dir failed!");
            }
        } else {
            println!("current file is not cl vcvarsall.bat!");
        }
    }

    Ok(())
}
