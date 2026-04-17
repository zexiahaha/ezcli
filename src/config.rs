use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::env::home_dir;
use std::fs;
use std::path::PathBuf;

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
    let exists = config.projects.iter_mut().find(|p| p.name == name);
    if let Some(project) = exists {
        println!("project {} already exists!", name.green().bold());

        project.path = path.to_string();
    } else {
        config.projects.push(Project {
            name: name.to_string(),
            path: path.to_string(),
        });
    }

    let _ = save_config(&config);

    println!("update {} path success!", name);
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

#[allow(dead_code)]
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
