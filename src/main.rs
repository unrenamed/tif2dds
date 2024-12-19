use dialoguer::{theme::ColorfulTheme, Select};
use image::{ImageFormat, ImageReader};
use ini::Ini;
use std::fs::{self, remove_file};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, io};

const VALID_SUFFIXES: [&str; 7] = ["ao", "rg", "mt", "hm", "nm", "lm", "dirt"];

#[derive(Debug, Copy, Clone, PartialEq)]
enum ImageFileFormat {
    Bc1,
    Bc3,
    Bc4,
    Bc5,
}

impl ImageFileFormat {
    fn as_str(&self) -> &str {
        match self {
            ImageFileFormat::Bc1 => "bc1",
            ImageFileFormat::Bc3 => "bc3",
            ImageFileFormat::Bc4 => "bc4",
            ImageFileFormat::Bc5 => "bc5",
        }
    }
}

impl std::fmt::Display for ImageFileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
struct ImageFile {
    file_path: PathBuf,
    file_extension: Option<String>,
    file_suffix: Option<String>,
}

#[derive(Debug)]
struct ImageFileWithFormat<'a> {
    image_file: &'a ImageFile,
    image_format: ImageFileFormat,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        return Err("No files selected.".into());
    }

    println!("Selected file paths:");
    args.iter().for_each(|file| println!("{}", file));

    let image_files = collect_image_files(&args)?;
    if image_files.is_empty() {
        return Err("No .tif or .png files found.".into());
    }

    let nvtools_path = load_nvtools_path()?;
    let files_with_format = prompt_format_selection(&image_files);
    let temp_pngs = generate_pngs_if_required(&files_with_format)?;
    let cmd_args = generate_command_args(&files_with_format);
    let execution_result = execute_commands(&nvtools_path, "nvtt_export.exe", &cmd_args);

    // Ensure cleanup always happens, regardless of success or failure
    if let Err(e) = cleanup_temp_files(&temp_pngs) {
        eprintln!("Error during cleanup: {}", e);
    }

    // Propagate the command execution result
    execution_result?;

    Ok(())
}

fn load_nvtools_path() -> io::Result<String> {
    let current_exe = env::current_exe()?;
    let exe_dir = current_exe.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Executable directory not found.")
    })?;
    let config_path = exe_dir.join("tif2dds_config.ini");
    let conf = Ini::load_from_file(config_path).expect("No config file is found.");

    conf.section(Some("General"))
        .and_then(|section| section.get("nvtoolsdirectory"))
        .map(|dir| dir.to_string())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid config"))
}

fn collect_image_files(args: &[String]) -> Result<Vec<ImageFile>, io::Error> {
    Ok(args
        .iter()
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let ext = path.extension()?.to_str()?.to_lowercase();
            if ext == "tif" || ext == "png" {
                Some((path, ext))
            } else {
                None
            }
        })
        .map(|(path, extension)| {
            let suffix = path
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| s.split('_').last())
                .filter(|&s| VALID_SUFFIXES.contains(&s))
                .map(String::from);

            ImageFile {
                file_path: path,
                file_extension: Some(extension),
                file_suffix: suffix,
            }
        })
        .collect())
}

fn prompt_format_selection<'a>(files: &'a [ImageFile]) -> Vec<ImageFileWithFormat<'a>> {
    let formats = [ImageFileFormat::Bc1, ImageFileFormat::Bc3];

    files
        .iter()
        .filter(|file| file.file_suffix.is_none())
        .map(|file| {
            let selected_format = Select::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "Image `{}` has no suffix. Choose the format to use:",
                    if let Some(extension) = &file.file_extension {
                        file.file_path
                            .with_extension(extension)
                            .display()
                            .to_string()
                    } else {
                        file.file_path.display().to_string()
                    }
                ))
                .default(0)
                .items(&formats)
                .interact()
                .unwrap();

            ImageFileWithFormat {
                image_file: file,
                image_format: formats[selected_format],
            }
        })
        .collect()
}

fn generate_pngs_if_required(
    files_with_format: &[ImageFileWithFormat],
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut temp_files = Vec::new();

    for file in files_with_format {
        if let Some(ext) = &file.image_file.file_extension {
            if ext == "png" || file.image_format != ImageFileFormat::Bc3 {
                continue;
            }

            let png_path = file.image_file.file_path.with_extension("png");
            convert_tiff_to_png(&file.image_file.file_path, &png_path)?;
            temp_files.push(png_path);
        }
    }

    Ok(temp_files)
}

fn generate_command_args(files_with_format: &[ImageFileWithFormat]) -> Vec<Vec<String>> {
    // Helper function to determine the input path
    fn get_input_path(
        file: &ImageFile,
        format: &ImageFileFormat,
        suffix: Option<&str>,
    ) -> std::path::PathBuf {
        match suffix {
            Some(_) => file.file_extension.as_deref().map_or_else(
                || file.file_path.clone(),
                |extension| file.file_path.with_extension(extension),
            ),
            None if format == &ImageFileFormat::Bc3 => file.file_path.with_extension("png"),
            _ => file.file_path.clone(),
        }
    }

    // Helper function to determine the output path
    fn get_output_path(file: &ImageFile) -> std::path::PathBuf {
        file.file_path.with_extension("dds")
    }

    // Helper function to determine the format and extra arguments based on suffix
    fn get_format_and_args(suffix: &str) -> Option<(ImageFileFormat, Vec<&'static str>)> {
        match suffix {
            "ao" | "rg" | "mt" | "hm" | "lm" => {
                Some((ImageFileFormat::Bc4, vec!["--no-mip-gamma-correct"]))
            }
            "nm" => Some((ImageFileFormat::Bc5, vec!["--no-mip-gamma-correct"])),
            "dirt" => Some((ImageFileFormat::Bc1, vec![])),
            _ => None,
        }
    }

    let mut args_list = Vec::new();

    for file_with_format in files_with_format {
        let input = get_input_path(
            &file_with_format.image_file,
            &file_with_format.image_format,
            file_with_format.image_file.file_suffix.as_deref(),
        );
        let output = get_output_path(&file_with_format.image_file);

        if let Some(suffix) = &file_with_format.image_file.file_suffix {
            if let Some((format, extra_args)) = get_format_and_args(suffix) {
                args_list.push(build_args(
                    &format,
                    "normal",
                    "box",
                    "5",
                    output.to_str().unwrap(),
                    input.to_str().unwrap(),
                    &extra_args,
                ));
            }
        } else {
            args_list.push(build_args(
                &file_with_format.image_format,
                "normal",
                "box",
                "5",
                output.to_str().unwrap(),
                input.to_str().unwrap(),
                &[],
            ));
        }
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
    format: &ImageFileFormat,
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
