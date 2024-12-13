use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};
use ini::Ini;
use std::{fs, io, path::Path, process::Command};

const VALID_SUFFIXES: [&str; 5] = ["ao", "rg", "mt", "hm", "nm"];
const MAX_FILES_PER_PAGE: usize = 15;

fn main() -> io::Result<()> {
    let nvtools_path = load_nvtools_path("tif2dds_config.ini")?;
    let tif_files = get_tif_files(".")?;

    if tif_files.is_empty() {
        println!("No .tif files found in the folder.");
        return Ok(());
    }

    let selected_files = prompt_file_selection(&tif_files);
    if selected_files.is_empty() {
        println!("You did not select anything :(");
        return Ok(());
    }

    let (suffix_files, no_suffix_files) = segregate_files_by_suffix(&selected_files);
    let no_suffix_files_with_format = prompt_format_selection(no_suffix_files);
    let cmd_args = prepare_command_args(&suffix_files, &no_suffix_files_with_format);

    execute_commands(&nvtools_path, "nvtt_export.exe", cmd_args)?;

    Ok(())
}

fn load_nvtools_path(config_path: &str) -> io::Result<String> {
    let conf = Ini::load_from_file(config_path).expect("No config file is found.");
    if let Some(section) = conf.section(Some("General")) {
        if let Some(dir_path) = section.get("nvtoolsdirectory") {
            return Ok(dir_path.to_string());
        }
    }
    println!("Config is not valid");
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "Invalid config",
    ))
}

fn get_tif_files(folder_path: &str) -> io::Result<Vec<String>> {
    let mut tif_files = vec![];

    for entry in fs::read_dir(folder_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("tif") {
            tif_files.push(path.display().to_string());
        }
    }

    Ok(tif_files)
}

fn prompt_file_selection(tif_files: &[String]) -> Vec<String> {
    let defaults = vec![true; tif_files.len()];

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick .tif files to transform to .dss")
        .items(tif_files)
        .defaults(&defaults)
        .max_length(MAX_FILES_PER_PAGE)
        .interact()
        .unwrap();

    selections
        .into_iter()
        .map(|i| tif_files[i].clone())
        .collect()
}

fn prompt_format_selection(files: Vec<String>) -> Vec<(String, &'static str)> {
    let transform_formats = ["bc1", "bc2", "bc3", "bc4", "bc5"];
    let mut final_files = vec![];

    for file in files {
        let selected_format = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("`{file}` has no suffix. Choose the format to use"))
            .default(0)
            .items(&transform_formats)
            .interact()
            .unwrap();

        final_files.push((file, transform_formats[selected_format]));
    }

    final_files
}

fn segregate_files_by_suffix(files: &[String]) -> (Vec<(String, String)>, Vec<String>) {
    let mut suffix_files = vec![];
    let mut no_suffix_files = vec![];

    for file_path in files {
        if let Some(filename) = Path::new(file_path)
            .file_stem()
            .and_then(|stem| stem.to_str())
        {
            let path_parts: Vec<&str> = filename.split('_').collect();
            let suffix = path_parts.last().unwrap_or(&"");

            if VALID_SUFFIXES.contains(suffix) {
                suffix_files.push((filename.to_string(), suffix.to_string()));
            } else {
                no_suffix_files.push(filename.to_string());
            }
        }
    }

    (suffix_files, no_suffix_files)
}

fn prepare_command_args(
    suffix_files: &[(String, String)],
    no_suffix_files_with_format: &[(String, &'static str)],
) -> Vec<Vec<String>> {
    let mut cmd_args = vec![];

    for (file, format) in no_suffix_files_with_format {
        cmd_args.push(build_args(
            format,
            "normal",
            "box",
            "5",
            &format!("{}.dds", file),
            &format!("{}.tif", file),
            &[],
        ));
    }

    for (file, suffix) in suffix_files {
        let (format, extra_args) = match suffix.as_str() {
            "ao" | "rg" | "mt" | "hm" => ("bc4", vec!["--no-mip-gamma-correct"]),
            "nm" => ("bc5", vec!["--no-mip-gamma-correct"]),
            _ => continue,
        };

        cmd_args.push(build_args(
            format,
            "normal",
            "box",
            "5",
            &format!("{}.dds", file),
            &format!("{}.tif", file),
            &extra_args,
        ));
    }

    cmd_args
}

fn execute_commands(
    nvtools_path: &str,
    nvtools_filename: &str,
    cmd_args: Vec<Vec<String>>,
) -> io::Result<()> {
    let fullpath = Path::new(nvtools_path).join(nvtools_filename);

    for args in cmd_args {
        println!("Executing: {} {:?}", fullpath.display(), args);

        let status = Command::new(fullpath.display().to_string())
            .args(&args)
            .status()
            .expect("Failed to execute command");

        if status.success() {
            println!("Command executed successfully!");
        } else {
            eprintln!("Command failed with status: {:?}", status);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Command execution failed",
            ));
        }
    }

    Ok(())
}

fn build_args(
    format: &str,
    quality: &str,
    mip_filter: &str,
    zcmp: &str,
    output_file: &str,
    input_file: &str,
    extra_args: &[&str],
) -> Vec<String> {
    let mut args = vec![
        "--format".to_string(),
        format.to_string(),
        "--quality".to_string(),
        quality.to_string(),
        "--mips".to_string(),
        "--mip-filter".to_string(),
        mip_filter.to_string(),
        "--zcmp".to_string(),
        zcmp.to_string(),
        "--output".to_string(),
        output_file.to_string(),
        input_file.to_string(),
    ];
    for arg in extra_args {
        args.push(arg.to_string());
    }
    args
}
