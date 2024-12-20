mod config;
mod cli;
mod conversion;
mod winreg;

type AppResult = Result<(), Box<dyn std::error::Error>>;

fn main() -> AppResult {
    let matches = cli::setup_cli().get_matches();

    match matches.subcommand() {
        Some(("install", _)) => handle_install()?,
        Some(("convert", sub_matches)) => handle_convert(sub_matches)?,
        _ => unreachable!(),
    }

    Ok(())
}

fn handle_install() -> AppResult {
    println!("Starting the installation process...");

    match config::generate_config_file() {
        Ok(path) if path.is_some() => println!(
            "Configuration file created successfully at {}",
            path.unwrap().display()
        ),
        Err(e) => eprintln!("Failed to create the configuration file. Details: {}", e),
        _ => {}
    }

    println!("Registering the context menu options...");
    winreg::register_context_menu_options()?;

    println!("Installation process completed successfully!");
    Ok(())
}

fn handle_convert(sub_matches: &clap::ArgMatches) -> AppResult {
    let paths: Vec<_> = sub_matches
        .get_many::<std::path::PathBuf>("PATH")
        .into_iter()
        .flatten()
        .collect();

    if paths.is_empty() {
        return Err("No files were selected for conversion.".into());
    }

    println!("Files selected for conversion:");
    for file in &paths {
        println!("- \"{}\"", file.display());
    }

    conversion::convert_images_to_dds(&paths)?;
    println!("Conversion completed successfully!");
    Ok(())
}
