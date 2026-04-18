mod config;
mod env_capture;
mod render;
mod wrapper;

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use config::{
    Config, Project, add_project, delete_project, find_project, get_config_path, load_config,
    save_config,
};
use env_capture::capture_vcvars_env;
use inquire::{Select, error::InquireError};
use render::{ScriptPlan, ShellKind, render_cmd_script, render_powershell_script};
use std::path::PathBuf;
use wrapper::{
    build_powershell_profile_source_line, get_powershell_profile_path,
    install_powershell_profile_source_line, render_powershell_wrapper_script,
    save_cmd_wrapper_scripts, save_powershell_wrapper_script,
};

use std::collections::HashMap;
use std::env;
use std::env::home_dir;
use std::fs::{self, create_dir_all};
use std::io;
use std::mem;
use std::os::windows::process::CommandExt;
use std::process::Command;
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::Win32::UI::Controls::Dialogs::{
    GetOpenFileNameW, OFN_FILEMUSTEXIST, OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows::Win32::UI::Shell::{FILEOPENDIALOGOPTIONS, FOS_PICKFOLDERS, IFileOpenDialog};
use windows::core::PWSTR;
use winreg::RegKey;
use winreg::enums::*;

const MAX_PATH: u32 = 260;

struct ComInitializer;
impl ComInitializer {
    fn new() -> Self {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        ComInitializer
    }
}
impl Drop for ComInitializer {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum ShellArg {
    Cmd,
    Powershell,
}

#[derive(Debug, Eq, PartialEq, Subcommand)]
enum EmitAction {
    LoadCl,
    EnterProject { name: String },
    Init,
    InstallWrapper,
    ShowProfile,
    InstallProfile,
}

#[derive(Debug, Eq, PartialEq, Subcommand)]
enum Commands {
    Emit {
        #[arg(long, value_enum)]
        shell: ShellArg,

        #[command(subcommand)]
        action: EmitAction,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(Commands::Emit { shell, action }) = cli.command {
        if let Err(error) = handle_emit(shell, action) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return Ok(());
    }

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
            println!("{}", "config updated:".green().bold());
            println!("  vcvarsall.bat saved to ezcli config");

            println!();
            println!("{}", "cmd setup:".green().bold());
            let cmd_wrapper_path = install_wrapper(ShellArg::Cmd)?;
            for wrapper_path in cmd_wrapper_path {
                println!(
                    "cmd wrapper save to {}",
                    wrapper_path.display().to_string().green()
                );
            }
            println!("open a new cmd.exe, then you can run:");
            println!("  {}", "ezcli-load-cl".green());
            println!("  {} {}", "ezcli-enter-project".green(), "<name>".yellow());
            println!("if cmd was already open before PATH changed, reopen it first");

            println!();
            println!("{}", "powershell setup:".green().bold());
            let powershell_wrapper_path = install_wrapper(ShellArg::Powershell)?;
            if let Some(wrapper_path) = powershell_wrapper_path.first() {
                println!(
                    "powershell wrapper save to {}",
                    wrapper_path.display().to_string().green()
                );
            }

            println!();

            let answer = confirm_continue("write ezcli wrapper into Powershell profile?");

            if answer {
                let message = install_profile(ShellArg::Powershell)?;
                println!("  {}", message.green());
            } else {
                let profile_path = get_powershell_profile_path()?;
                let source_line = build_powershell_profile_source_line()?;

                println!("  {}", "skip writing Powershell profile".yellow());
                println!(
                    "  Powershell profile path: {}",
                    profile_path.display().to_string().green()
                );
                println!(
                    "  you can manually add this line to your Powershell profile: {}",
                    source_line.green()
                );
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

        let project_path_str = select_folder_modern().unwrap_or("".to_string());

        println!("project_path_str: {}", &project_path_str.green());

        let mut config = load_config().map_err(|_| "load config file failed!".red())?;

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

pub fn select_folder_modern() -> Option<String> {
    unsafe {
        let _com_init = ComInitializer::new();

        let dialog_result: Result<IFileOpenDialog, windows::core::Error> =
            CoCreateInstance(&windows::Win32::UI::Shell::FileOpenDialog, None, CLSCTX_ALL);

        let dialog = dialog_result.ok()?;

        let options: FILEOPENDIALOGOPTIONS = Default::default();
        dialog.GetOptions().ok()?;
        dialog.SetOptions(options | FOS_PICKFOLDERS).ok()?;

        if dialog.Show(None).is_err() {
            return None;
        }

        let item_result = dialog.GetResult();
        let item = item_result.ok()?;

        let display_name = item
            .GetDisplayName(windows::Win32::UI::Shell::SIGDN_FILESYSPATH)
            .ok()?;

        let path = display_name.to_string().ok()?;

        Some(path)
    }
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

fn to_shell_kind(shell: ShellArg) -> ShellKind {
    match shell {
        ShellArg::Cmd => ShellKind::Cmd,
        ShellArg::Powershell => ShellKind::Powershell,
    }
}

fn build_load_cl_script(
    shell: ShellArg,
    captured: std::collections::BTreeMap<String, String>,
) -> String {
    let plan = ScriptPlan {
        set_env: captured,
        prepend_path: Vec::new(),
        cwd: None,
    };

    match to_shell_kind(shell) {
        ShellKind::Cmd => render_cmd_script(&plan),
        ShellKind::Powershell => render_powershell_script(&plan),
    }
}

fn build_enter_project_script(shell: ShellArg, project: &Project) -> String {
    let plan = ScriptPlan {
        set_env: Default::default(),
        prepend_path: vec![PathBuf::from(&project.path)],
        cwd: Some(PathBuf::from(&project.path)),
    };

    match to_shell_kind(shell) {
        ShellKind::Cmd => render_cmd_script(&plan),
        ShellKind::Powershell => render_powershell_script(&plan),
    }
}

fn build_init_script(shell: ShellArg) -> Result<String, Box<dyn std::error::Error>> {
    let program = env::current_exe()?;
    let program = program.to_string_lossy().into_owned();

    match to_shell_kind(shell) {
        ShellKind::Powershell => Ok(render_powershell_wrapper_script(&program)),
        ShellKind::Cmd => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "cmd init is not supported; use `ezcli emit --shell cmd install-wrapper` instead",
        )
        .into()),
    }
}

fn install_wrapper(shell: ShellArg) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let program = env::current_exe()?;
    let program = program.to_string_lossy().into_owned();

    match to_shell_kind(shell) {
        ShellKind::Powershell => Ok(vec![save_powershell_wrapper_script(&program)?]),
        ShellKind::Cmd => save_cmd_wrapper_scripts(&program),
    }
}

fn show_profile(shell: ShellArg) -> Result<String, Box<dyn std::error::Error>> {
    match to_shell_kind(shell) {
        ShellKind::Powershell => {
            let profile_path = get_powershell_profile_path()?;
            let source_line = build_powershell_profile_source_line()?;

            Ok(format!(
                "profile path: {}\nsource line: {}",
                profile_path.display(),
                source_line
            ))
        }
        ShellKind::Cmd => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "cmd has no profile support; use `ezcli emit --shell cmd install-wrapper` instead",
        )
        .into()),
    }
}

fn install_profile(shell: ShellArg) -> Result<String, Box<dyn std::error::Error>> {
    match to_shell_kind(shell) {
        ShellKind::Powershell => {
            let changed = install_powershell_profile_source_line()?;
            let profile_path = get_powershell_profile_path()?;

            if changed {
                Ok(format!(
                    "powershell profile updated: {}",
                    profile_path.display()
                ))
            } else {
                Ok(format!(
                    "powershell profile already configured: {}",
                    profile_path.display(),
                ))
            }
        }
        ShellKind::Cmd => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "cmd has no profile support; use `ezcli emit --shell cmd install-wrapper` instead",
        )
        .into()),
    }
}

fn handle_emit(shell: ShellArg, action: EmitAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        EmitAction::LoadCl => {
            let config = load_config()?;
            let captured = capture_vcvars_env(&config.vc_path, &config.default_arch)?;
            print!("{}", build_load_cl_script(shell, captured));
            Ok(())
        }
        EmitAction::EnterProject { name } => {
            let config = load_config()?;
            let project = find_project(&config, &name).ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("project not found: {name}"),
                )
            })?;

            print!("{}", build_enter_project_script(shell, project));
            Ok(())
        }
        EmitAction::Init => {
            print!("{}", build_init_script(shell)?);
            Ok(())
        }
        EmitAction::InstallWrapper => {
            let wrapper_paths = install_wrapper(shell)?;
            for wrapper_path in wrapper_paths {
                println!("powershell wrapper saved to {}", wrapper_path.display());
            }
            Ok(())
        }
        EmitAction::ShowProfile => {
            print!("{}", show_profile(shell)?);
            Ok(())
        }
        EmitAction::InstallProfile => {
            print!("{}", install_profile(shell)?);
            Ok(())
        }
    }
}
