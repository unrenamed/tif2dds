use std::env;

pub fn register_context_menu_options() -> Result<(), Box<dyn std::error::Error>> {
    let exe_path = env::current_exe()?.display().to_string();
    let script_command = format!(
        r#"powershell.exe -NoProfile -Command \"& \"{}\" convert \"%1\"\""#,
        exe_path.replace("\\", "\\\\")
    );

    for file_type in [".tif", ".png"] {
        add_context_menu_for_file_type(file_type, "Convert to DDS", &script_command)?;
        println!("Successfully added context menu for file type '{}'.", file_type);
    }

    Ok(())
}

#[cfg(not(windows))]
fn add_context_menu_for_file_type(
    extension: &str,
    menu_name: &str,
    _command: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let shell_key_path = format!(r"SystemFileAssociations\{}\Shell\{}", extension, menu_name);
    println!("{}", shell_key_path);
    Ok(())
}

#[cfg(windows)]
fn add_context_menu_for_file_type(
    extension: &str,
    menu_name: &str,
    command: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use winreg::enums::*;
    use winreg::RegKey;

    // Open the HKEY_CLASSES_ROOT registry key
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);

    // Create the context menu entry
    let shell_key_path = format!(r"SystemFileAssociations\{}\Shell\{}", extension, menu_name);
    let (shell_key, _) = hkcr.create_subkey(&shell_key_path)?;

    // Set the context menu label
    shell_key.set_value("", &menu_name)?;

    // Add the command to execute
    let (command_key, _) = shell_key.create_subkey("Command")?;
    command_key.set_value("", &command)?;

    Ok(())
}
