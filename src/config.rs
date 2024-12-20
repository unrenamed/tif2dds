use std::fs::{self, File};
use std::env;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

use dialoguer::{theme::ColorfulTheme, Confirm, Input};
use ini::Ini;

const CONFIG_FILENAME: &str = "tif2dds_config.ini";

pub fn generate_config_file() -> io::Result<Option<PathBuf>> {
    let config_path = get_config_path()?;
    if file_exists_and_user_declines(&config_path)? {
        return Ok(None);
    }
    let nvtoolsdirectory = get_nvtools_directory_path();
    write_config_file(&config_path, &nvtoolsdirectory)?;
    Ok(Some(config_path))
}

pub fn load_config() -> io::Result<Ini> {
    let config_path = get_config_path()?;
    Ok(Ini::load_from_file(config_path).expect("Failed to load the config."))
}

fn file_exists_and_user_declines(path: &Path) -> io::Result<bool> {
    if path.exists() {
        let override_file = Confirm::new()
            .with_prompt("The config file already exists. Do you want to override it?")
            .interact()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        return Ok(!override_file);
    }
    Ok(false)
}

fn get_nvtools_directory_path() -> String {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Please enter the path to the folder containing Nvidia Texture CLI:")
        .validate_with({
            move |input: &String| -> Result<(), &str> {
                if fs::metadata(input).map(|m| m.is_dir()).unwrap_or(false) {
                    Ok(())
                } else {
                    Err("The specified folder does not exist or is not a directory. Please provide a valid path.")
                }
            }
        })
        .interact_text()
        .unwrap()
}

fn write_config_file(path: &Path, nvtoolsdirectory: &str) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "[General]")?;
    writeln!(file, "nvtoolsdirectory = {}", nvtoolsdirectory)?;
    Ok(())
}

fn get_config_path() -> io::Result<PathBuf> {
    let exe_dir = get_executable_directory()?;
    Ok(exe_dir.join(CONFIG_FILENAME))
}

fn get_executable_directory() -> io::Result<PathBuf> {
    let current_exe = env::current_exe()?;
    current_exe
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Executable directory not found."))
}
