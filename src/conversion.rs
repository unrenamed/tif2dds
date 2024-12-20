use dialoguer::{theme::ColorfulTheme, Select};
use image::{ImageFormat, ImageReader};
use std::fs::{self, remove_file};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config;

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
    extra_arguments: Vec<String>,
}

pub fn convert_images_to_dds(args: &[&PathBuf]) -> Result<(), Box<dyn std::error::Error>> {
    let image_files = collect_image_files(&args)?;
    let nvtools_path = load_nvtools_path()?;
    let files_with_format = prepare_files_with_format(&image_files);
    let temp_pngs = generate_pngs_if_required(&files_with_format)?;
    let cmd_args = generate_command_args(&files_with_format);
    let execution_result = execute_commands(&nvtools_path, "nvtt_export.exe", &cmd_args);

    // Ensure cleanup always happens, regardless of success or failure
    if let Err(e) = cleanup_temp_files(&temp_pngs) {
        eprintln!(
            "Failed to delete temporary file. You may need to remove it manually. Details: {}",
            e
        );
    }

    // Propagate the command execution result
    execution_result?;
    Ok(())
}

fn load_nvtools_path() -> io::Result<String> {
    let conf = config::load_config()?;
    conf.section(Some("General"))
        .and_then(|section| section.get("nvtoolsdirectory"))
        .map(|dir| dir.to_string())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid config"))
}

fn collect_image_files(args: &[&PathBuf]) -> Result<Vec<ImageFile>, io::Error> {
    Ok(args
        .iter()
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
                file_path: (*path).clone(),
                file_extension: Some(extension),
                file_suffix: suffix,
            }
        })
        .collect())
}

fn prepare_files_with_format<'a>(files: &'a [ImageFile]) -> Vec<ImageFileWithFormat<'a>> {
    files
        .iter()
        .map(|file| {
            if let Some(suffix) = &file.file_suffix {
                ImageFileWithFormat {
                    image_file: file,
                    image_format: get_nvtools_format(&suffix),
                    extra_arguments: get_nvtools_arguments(suffix),
                }
            } else {
                ImageFileWithFormat {
                    image_file: file,
                    image_format: prompt_format_selection(file),
                    extra_arguments: vec![],
                }
            }
        })
        .collect()
}

// Helper function to determine the format based on suffix
fn get_nvtools_format(suffix: &str) -> ImageFileFormat {
    match suffix {
        "ao" | "rg" | "mt" | "hm" | "lm" => ImageFileFormat::Bc4,
        "nm" => ImageFileFormat::Bc5,
        "dirt" => ImageFileFormat::Bc1,
        _ => unreachable!(),
    }
}

// Helper function to determine the extra arguments based on suffix
fn get_nvtools_arguments(suffix: &str) -> Vec<String> {
    match suffix {
        "ao" | "rg" | "mt" | "hm" | "lm" => vec!["--no-mip-gamma-correct".to_string()],
        "nm" => vec!["--no-mip-gamma-correct".to_string()],
        _ => vec![],
    }
}

fn prompt_format_selection<'a>(file: &'a ImageFile) -> ImageFileFormat {
    let formats = [ImageFileFormat::Bc1, ImageFileFormat::Bc3];

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

    formats[selected_format]
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

    let mut args_list = Vec::new();

    for file_with_format in files_with_format {
        let input = get_input_path(
            &file_with_format.image_file,
            &file_with_format.image_format,
            file_with_format.image_file.file_suffix.as_deref(),
        );
        let output = get_output_path(&file_with_format.image_file);

        args_list.push(build_args(
            &file_with_format.image_format,
            "normal",
            "box",
            "5",
            output.to_str().unwrap(),
            input.to_str().unwrap(),
            &file_with_format.extra_arguments,
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
    format: &ImageFileFormat,
    quality: &str,
    mip_filter: &str,
    zcmp: &str,
    output: &str,
    input: &str,
    extra_args: &[String],
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
    args.extend(extra_args.iter().map(|arg| arg.to_string()));
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
