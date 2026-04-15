use clap::Parser;
use serde::{Deserialize, Serialize};
use std::env;
use std::env::home_dir;
use std::fs::{self, create_dir_all};
use std::mem;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use windows::Win32::System::Console::GetConsoleWindow;
use windows::Win32::UI::Controls::Dialogs::{
    GetOpenFileNameW, OFN_FILEMUSTEXIST, OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows::Win32::UI::Shell::{
    BIF_BROWSEINCLUDEURLS, BIF_NEWDIALOGSTYLE, BIF_NONEWFOLDERBUTTON, BROWSEINFOW,
    SHBrowseForFolderW, SHGetPathFromIDListW,
};
use windows::core::PWSTR;

const MAX_PATH: u32 = 260;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub vc_path: String,
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
    load_cl: bool,

    #[arg(short, long)]
    add_project: Option<String>,

    #[arg(short = 'w', long)]
    show_project: Option<String>,

    #[arg(short, long)]
    del_project: bool,

    #[arg(short = 'c', long)]
    switch_project: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.find_cl {
        let mut file_buf = [0u16; MAX_PATH as usize];

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
            let config_file = get_config_path();
            let config_file_dir = config_file.parent().unwrap();

            if !config_file_dir.exists() {
                create_dir_all(config_file_dir)?;
            }
            if !config_file.exists() {
                let default_config = Config {
                    vc_path: cl_str,
                    default_arch: "x64".to_string(),
                    projects: vec![],
                };

                let toml_content = toml::to_string_pretty(&default_config)?;

                fs::write(&config_file, toml_content)?;

                println!("already create new ezcli.toml!");
            } else {
                let mut config = load_config();
                config.vc_path = cl_str;
                save_config(&config);
            }
        } else {
            println!("current file is not cl vcvarsall.bat!");
        }
    }

    if cli.show_cl {
        let config = load_config();
        println!("{}", config.vc_path.as_str());
    }

    if cli.load_cl {
        let config = load_config();
        if !config.vc_path.is_empty() {
            let vc = config.vc_path;
            let arch = config.default_arch;

            let command_str = if env::var("ConEmuDir").is_ok() {
                println!("cmder environment detected!");
                format!(
                    r#"/s /k "call "%ConEmuDir%\..\init.bat" && call "{}" {}""#,
                    vc, arch
                )
            } else {
                println!("raw cmd environment");
                format!(r#"/s /k "call "{}" {}""#, vc, arch)
            };

            Command::new("cmd.exe")
                .raw_arg(command_str)
                .status()
                .unwrap();
        } else {
            println!("no vc_path exists, please run find_cl!");
        }
    }

    if let Some(name) = cli.add_project.as_deref() {
        println!("add new project: {name}");
        println!("please find project path");

        let str = unsafe {
            let hwnd = GetConsoleWindow();

            let mut bi = BROWSEINFOW {
                hwndOwner: hwnd,
                ulFlags: BIF_NEWDIALOGSTYLE | BIF_BROWSEINCLUDEURLS | BIF_NONEWFOLDERBUTTON,
                ..Default::default()
            };

            let pidl = SHBrowseForFolderW(&mut bi);

            let mut buffer = [0u16; MAX_PATH as usize];
            if pidl.is_null() || !SHGetPathFromIDListW(pidl, &mut buffer).as_bool() {
                None
            } else {
                if SHGetPathFromIDListW(pidl, &mut buffer).as_bool() {
                    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
                    Some(String::from_utf16_lossy(&buffer[..len]))
                } else {
                    None
                }
            }
        };

        let project_path_str = str.unwrap_or_else(|| "".to_string());

        let mut config = load_config();

        add_project(&mut config, name, &project_path_str);
    }
    Ok(())
}

pub fn get_config_path() -> PathBuf {
    let home = if let Some(home) = home_dir() {
        println!("home: {}", home.to_str().unwrap_or("home dir print failed"));
        home.join(".ezcli").join("ezcli.toml")
    } else {
        println!("get home dir failed!");
        PathBuf::new()
    };
    home
}

pub fn load_config() -> Config {
    let config_path = get_config_path();
    println!("config_path: {:?}", &config_path.to_str());
    let content = fs::read_to_string(&config_path).expect("read config failed!");
    toml::from_str(&content).expect("parse config failed!")
}

pub fn save_config(config: &Config) {
    let config_path = get_config_path();
    let content = toml::to_string_pretty(config).unwrap();
    fs::write(config_path, content).expect("save config failed!");
}

pub fn add_project(config: &mut Config, name: &str, path: &str) {
    let exists = config.projects.iter().any(|p| p.name == name);
    if exists {
        println!("project {name} already exists!");
    }

    config.projects.push(Project {
        name: name.to_string(),
        path: path.to_string(),
    });
}

pub fn delete_project(config: &mut Config, name: &str) -> bool {
    let old_len = config.projects.len();
    config.projects.retain(|p| p.name != name);
    old_len != config.projects.len()
}

pub fn update_project_path(config: &mut Config, name: &str, new_path: &str) -> bool {
    match config.projects.iter_mut().find(|p| p.name == name) {
        Some(proj) => {
            proj.path = new_path.to_string();
            true
        }
        None => false,
    }
}

pub fn find_project<'a>(config: &'a Config, name: &'a str) -> Option<&'a Project> {
    config.projects.iter().find(|p| p.name == name)
}
