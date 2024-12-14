use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};
use image::{ImageFormat, ImageReader};
use ini::Ini;
use std::fs::{self, remove_file};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

const VALID_SUFFIXES: [&str; 5] = ["ao", "rg", "mt", "hm", "nm"];
const MAX_FILES_PER_PAGE: usize = 15;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let no_suffix_with_formats = prompt_format_selection(&no_suffix_files);

    let temp_pngs = create_pngs_if_needed(&no_suffix_with_formats)?;
    let cmd_args = build_all_command_args(&suffix_files, &no_suffix_with_formats);

    let execution_result = execute_commands(&nvtools_path, "nvtt_export.exe", &cmd_args);

    // Ensure cleanup always happens, regardless of success or failure
    if let Err(e) = cleanup_temp_files(&temp_pngs) {
        eprintln!("Error during cleanup: {}", e);
    }

    // Propagate the command execution result
    execution_result?;

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

fn get_tif_files(folder_path: &str) -> io::Result<Vec<PathBuf>> {
    fs::read_dir(folder_path)?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("tif") {
                Some(Ok(path))
            } else {
                None
            }
        })
        .collect()
}

fn prompt_file_selection(tif_files: &[PathBuf]) -> Vec<PathBuf> {
    let file_names: Vec<_> = tif_files
        .iter()
        .map(|path| path.display().to_string())
        .collect();

    let defaults = vec![true; file_names.len()];

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick .tif files to transform to .dss")
        .items(&file_names)
        .defaults(&defaults)
        .max_length(MAX_FILES_PER_PAGE)
        .interact()
        .unwrap();

    selections
        .into_iter()
        .map(|i| tif_files[i].clone())
        .collect()
}

fn prompt_format_selection(files: &[PathBuf]) -> Vec<(PathBuf, &'static str)> {
    let formats = ["bc1", "bc2", "bc3", "bc4", "bc5"];

    files
        .iter()
        .map(|file| {
            let selected_format = Select::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "Image `{}` has no suffix. Choose the format to use:",
                    file.with_extension("tif").display()
                ))
                .default(0)
                .items(&formats)
                .interact()
                .unwrap();
            (file.clone(), formats[selected_format])
        })
        .collect()
}

fn segregate_files_by_suffix(files: &[PathBuf]) -> (Vec<(PathBuf, String)>, Vec<PathBuf>) {
    let mut suffix_files = Vec::new();
    let mut no_suffix_files = Vec::new();

    for file in files {
        if let Some(stem) = file.file_stem().and_then(|s| s.to_str()) {
            let parts: Vec<&str> = stem.split('_').collect();
            let suffix = parts.last().unwrap_or(&"");

            if VALID_SUFFIXES.contains(suffix) {
                suffix_files.push((file.clone(), suffix.to_string()));
            } else {
                no_suffix_files.push(file.clone());
            }
        }
    }

    (suffix_files, no_suffix_files)
}

fn create_pngs_if_needed(
    files_with_formats: &[(PathBuf, &'static str)],
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut temp_files = Vec::new();

    for (file, format) in files_with_formats {
        if *format == "bc3" {
            let png_path = file.with_extension("png");
            convert_tiff_to_png(file, &png_path)?;
            temp_files.push(png_path);
        }
    }

    Ok(temp_files)
}

fn build_all_command_args(
    suffix_files: &[(PathBuf, String)],
    no_suffix_with_formats: &[(PathBuf, &'static str)],
) -> Vec<Vec<String>> {
    let mut args_list = Vec::new();

    for (file, format) in no_suffix_with_formats {
        let input = if *format == "bc3" {
            file.with_extension("png")
        } else {
            file.clone()
        };
        let output = file.with_extension("dds");

        args_list.push(build_args(
            format,
            "normal",
            "box",
            "5",
            output.to_str().unwrap(),
            input.to_str().unwrap(),
            &[],
        ));
    }

    for (file, suffix) in suffix_files {
        let (format, extra_args) = match suffix.as_str() {
            "ao" | "rg" | "mt" | "hm" => ("bc4", vec!["--no-mip-gamma-correct"]),
            "nm" => ("bc5", vec!["--no-mip-gamma-correct"]),
            _ => continue,
        };
        let input = file.with_extension("tif");
        let output = file.with_extension("dds");

        args_list.push(build_args(
            format,
            "normal",
            "box",
            "5",
            output.to_str().unwrap(),
            input.to_str().unwrap(),
            &extra_args,
        ));
    }

    args_list
}

fn execute_commands(
    nvtools_path: &str,
    nvtools_filename: &str,
    cmd_args: &[Vec<String>],
) -> io::Result<()> {
    let fullpath = Path::new(nvtools_path).join(nvtools_filename);

    for args in cmd_args {
        println!("Executing: {} {:?}", fullpath.display(), args);
        let status = Command::new(&fullpath).args(args).status()?;
        if !status.success() {
            eprintln!("Command failed with status: {:?}", status);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Command execution failed",
            ));
        }
    }

    Ok(())
}

fn cleanup_temp_files(temp_files: &[PathBuf]) -> io::Result<()> {
    for file in temp_files {
        remove_file(file)?;
    }
    if temp_files.len() > 0 {
        println!("All temporary PNGs deleted.");
    }
    Ok(())
}

fn build_args(
    format: &str,
    quality: &str,
    mip_filter: &str,
    zcmp: &str,
    output: &str,
    input: &str,
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
        output.to_string(),
    ];
    args.extend(extra_args.iter().map(|&arg| arg.to_string()));
    args.push(input.to_string());
    args
}

fn convert_tiff_to_png(input: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let img = ImageReader::open(input)?.with_guessed_format()?.decode()?;
    let output_file = fs::File::create(output)?;
    img.write_to(&mut std::io::BufWriter::new(output_file), ImageFormat::Png)?;
    println!("Temporary PNG created: {}", output.display());
    Ok(())
}
