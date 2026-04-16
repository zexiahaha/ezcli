use clap::Parser;
use colored::Colorize;
use inquire::{Select, error::InquireError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::env::home_dir;
use std::fs::{self, create_dir_all};
use std::io;
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
use winreg::RegKey;
use winreg::enums::*;

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
    show_project: bool,

    #[arg(short, long)]
    del_project: bool,

    #[arg(short = 'c', long)]
    switch_project: bool,
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

        println!("file path is {}", cl_str.green().bold());

        if cl_str.as_str().ends_with("vcvarsall.bat") {
            save_cl_bat(&cl_str)?;
            add_ezcli_to_path()?;
            let config_file = get_config_path().ok_or("get config path failed!".red())?;
            let config_file_dir = config_file.parent().unwrap();

            if !config_file_dir.exists() {
                create_dir_all(config_file_dir).map_err(|_| "create config dir failed!".red())?;
            }
            if !config_file.exists() {
                let default_config = Config {
                    vc_path: cl_str,
                    default_arch: "x64".to_string(),
                    projects: vec![],
                };

                let toml_content = toml::to_string_pretty(&default_config)
                    .map_err(|_| "toml to_string_pretty failed!".red())?;

                fs::write(&config_file, toml_content)?;

                println!("already create new ezcli.toml!");
            } else {
                let mut config = load_config().map_err(|_| "load config file failed!".red())?;
                config.vc_path = cl_str;
                save_config(&config).map_err(|_| "save config file failed!".red())?;
            }
        } else {
            println!("current file is not cl vcvarsall.bat!");
        }
    }

    if cli.show_cl {
        let config = load_config().map_err(|_| "load config file failed!".red())?;
        println!("cl at {}", config.vc_path.as_str().green());
    }

    if cli.load_cl {
        let config = load_config().map_err(|_| "load config file failed!".red())?;
        if !config.vc_path.is_empty() {
            let vc = config.vc_path;
            let arch = config.default_arch;

            let command_str = if env::var("ConEmuDir").is_ok() {
                println!("{}", "cmder environment detected!".blue());
                format!(
                    r#"/s /k "call "%ConEmuDir%\..\init.bat" && call "{}" {}""#,
                    vc, arch
                )
            } else {
                println!("{}", "raw cmd environment".blue());
                format!(r#"/s /k "call "{}" {}""#, vc, arch)
            };

            Command::new("cmd.exe")
                .raw_arg(command_str)
                .status()
                .unwrap();
        } else {
            println!("{}", "no vc_path exists, please run find_cl!".red());
        }
    }

    if let Some(name) = cli.add_project.as_deref() {
        println!("add new project: {}", name.green());
        println!("{}", "please find project path".yellow());

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

        println!("project_path_str: {}", &project_path_str);

        let mut config = load_config().map_err(|_| "load config file failed!".red())?;

        save_project_bat(name, &project_path_str)?;

        add_project(&mut config, name, &project_path_str);
    }

    if cli.show_project {
        let config = load_config().map_err(|_| "load config file failed!".red())?;
        let projects_map: HashMap<String, String> = config
            .projects
            .iter()
            .map(|p| (p.name.clone(), p.path.clone()))
            .collect();
        let project_names: Vec<&str> = projects_map.keys().map(|s| s.as_str()).collect();

        let ans: Result<&str, InquireError> =
            Select::new("select to show path", project_names).prompt();

        match ans {
            Ok(choice) => {
                let path = projects_map
                    .get(choice)
                    .ok_or("can't find the project".red())?;
                println!("\n project {} path is {}", choice.green(), path.green());
            }
            Err(_) => {
                println!(
                    "{}",
                    "There was an error when select project to show, please try again"
                        .red()
                        .bold()
                )
            }
        }
    }

    if cli.del_project {
        let config = load_config().map_err(|_| "load config file failed!".red())?;
        let project_names: Vec<&str> = config.projects.iter().map(|p| p.name.as_str()).collect();
        let options: Vec<&str> = project_names;

        let ans: Result<&str, InquireError> = Select::new("select to delete", options).prompt();

        match ans {
            Ok(choice) => {
                let answer = confirm_continue(
                    format!("will delete {} project, continue?", choice.green()).as_str(),
                );

                if answer {
                    let mut config = load_config().map_err(|_| "load config file failed!".red())?;
                    delete_project(&mut config, choice)?;
                }
            }
            Err(_) => {
                println!(
                    "{}",
                    "There was an error when select project to delete, please try again"
                        .red()
                        .bold()
                )
            }
        }
    }

    if cli.switch_project {
        let config = load_config().map_err(|_| "load config file failed!".red())?;
        let projects_map: HashMap<String, String> = config
            .projects
            .iter()
            .map(|p| (p.name.clone(), p.path.clone()))
            .collect();
        let project_names: Vec<&str> = projects_map.keys().map(|s| s.as_str()).collect();

        let ans: Result<&str, InquireError> =
            Select::new("select to switch", project_names).prompt();

        match ans {
            Ok(choice) => {
                let path = projects_map
                    .get(choice)
                    .ok_or("can't find the project".red())?;
                println!("\n project {} path is {}", choice.green(), path.green());
            }
            Err(_) => {
                println!(
                    "{}",
                    "There was an error when select project to switch, please try again"
                        .red()
                        .bold()
                )
            }
        }
    }
    Ok(())
}

fn confirm_continue(prompt: &str) -> bool {
    loop {
        println!("{prompt} y/n");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect(format!("{}", "read input line failed!".red().bold()).as_str());

        match input.trim().to_lowercase().as_str() {
            "y" => return true,
            "n" => return false,
            _ => println!("{}", "please input y or n \n".yellow()),
        }
    }
}

pub fn save_cl_bat(cl_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let cl_bat_str = format!(
        r#"
@echo off
call "{}" x64
"#,
        cl_path
    );
    let home = home_dir().ok_or("get home dir failed".red())?;
    let cli_dir = home.join(".ezcli");
    let cl_bat_path = cli_dir.join("cl_l.bat");

    if !cli_dir.exists() {
        create_dir_all(cli_dir).map_err(|_| "create config dir failed!".red())?;
    }

    fs::write(cl_bat_path, cl_bat_str)?;

    Ok(true)
}

pub fn save_project_bat(
    name: &str,
    project_path: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let project_bat_str = format!(
        r#"
@echo off
set path={};%path%
cd /d "{}"
"#,
        project_path, project_path
    );
    let home = home_dir().ok_or("get home dir failed".red())?;

    let cli_dir = home.join(".ezcli");
    let project_bat_path = cli_dir.join(format!("{}_l.bat", name));

    if !cli_dir.exists() {
        create_dir_all(cli_dir).map_err(|_| "create config dir failed!".red())?;
    }

    fs::write(project_bat_path, project_bat_str)?;

    Ok(true)
}

pub fn add_ezcli_to_path() -> Result<bool, Box<dyn std::error::Error>> {
    let home = home_dir().ok_or("get home dir failed".red())?;
    let cli_dir = home.join(".ezcli");
    let cli_dir_str = cli_dir.to_str().unwrap_or("");

    if !cli_dir.exists() {
        create_dir_all(cli_dir_str).map_err(|_| "create config dir failed!".red())?;
    }

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    let env_key = hklm.open_subkey_with_flags(
        "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment",
        KEY_READ | KEY_WRITE,
    )?;

    let current: String = env_key.get_value("PATH").unwrap_or_default();

    if !current
        .split(';')
        .any(|p| p.eq_ignore_ascii_case(cli_dir_str))
    {
        let new_path = if current.is_empty() {
            cli_dir_str.to_string()
        } else {
            format!("{};{}", current, cli_dir_str)
        };

        env_key.set_value("PATH", &new_path)?;

        println!("already let {} write into HKLM PATH", cli_dir_str.green());
    } else {
        println!("HKLM PATH already has {}, skip", cli_dir_str.green());
    }

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env_key = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;

    let current: String = env_key.get_value("PATH").unwrap_or_default();

    if !current
        .split(';')
        .any(|p| p.eq_ignore_ascii_case(cli_dir_str))
    {
        let new_path = if current.is_empty() {
            cli_dir_str.to_string()
        } else {
            format!("{};{}", current, cli_dir_str)
        };
        env_key.set_value("PATH", &new_path)?;

        println!("already let {} write into HKCU PATH", cli_dir_str.green());
    } else {
        println!("HKCU PATH already has {}, skip", cli_dir_str.green());
    }

    Ok(true)
}

pub fn get_config_path() -> Option<PathBuf> {
    let home = home_dir()?;
    Some(home.join(".ezcli").join("ezcli.toml"))
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = get_config_path().ok_or("get config path failed!".red())?;
    // println!("config_path: {:?}", &config_path.to_str());
    let content = fs::read_to_string(&config_path)?;
    let data = toml::from_str(&content).map_err(|_| "toml from_str failed!".red())?;
    Ok(data)
}

pub fn save_config(config: &Config) -> Result<bool, Box<dyn std::error::Error>> {
    let config_path = get_config_path().ok_or("get config path failed!".red())?;
    let content =
        toml::to_string_pretty(config).map_err(|_| "toml to_string_pretty failed!".red())?;
    fs::write(config_path, content)?;
    Ok(true)
}

pub fn add_project(config: &mut Config, name: &str, path: &str) {
    let exists = config.projects.iter().any(|p| p.name == name);
    if exists {
        println!("project {} already exists!", name.green().bold());
    }

    config.projects.push(Project {
        name: name.to_string(),
        path: path.to_string(),
    });

    let _ = save_config(&config);
}

pub fn delete_project(config: &mut Config, name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let old_len = config.projects.len();
    config.projects.retain(|p| p.name != name);

    let home = home_dir().ok_or("get home dir failed".red())?;

    let cli_dir = home.join(".ezcli");
    let project_bat_path = cli_dir.join(format!("{}_l.bat", name));

    fs::remove_file(project_bat_path)?;

    let _ = save_config(&config);
    Ok(old_len != config.projects.len())
}

pub fn update_project_path(config: &mut Config, name: &str, new_path: &str) -> bool {
    match config.projects.iter_mut().find(|p| p.name == name) {
        Some(proj) => {
            proj.path = new_path.to_string();

            let _ = save_config(&config);
            true
        }
        None => false,
    }
}

pub fn find_project<'a>(config: &'a Config, name: &'a str) -> Option<&'a Project> {
    config.projects.iter().find(|p| p.name == name)
}
