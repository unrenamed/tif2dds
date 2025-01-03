use std::env;

pub fn register_context_menu_options() -> Result<(), Box<dyn std::error::Error>> {
    let exe_path = env::current_exe()?.display().to_string();
    let script_command = format!(
        r#"powershell.exe -NoProfile -Command "& \"{}\" convert \"%1\"""#,
        exe_path.replace("\\", "\\\\")
    );

    for file_type in [".tif", ".png"] {
        add_context_menu_for_file_type(file_type, "Convert to DDS", &script_command)?;
        println!("Added context menu for file type '{}'.", file_type);
    }

    Ok(())
}

pub fn unregister_context_menu_options() -> Result<(), Box<dyn std::error::Error>> {
    for file_type in [".tif", ".png"] {
        remove_context_menu_for_file_type(file_type, "Convert to DDS")?;
        println!("Removed context menu for file type '{}'.", file_type);
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

    // Set the context menu label and separate it from other menu items
    shell_key.set_value("", &menu_name)?;
    shell_key.set_value("SeparatorBefore", &"")?;
    shell_key.set_value("SeparatorAfter", &"")?;

    // Add the command to execute
    let (command_key, _) = shell_key.create_subkey("Command")?;
    command_key.set_value("", &command)?;

    Ok(())
}

#[cfg(not(windows))]
fn remove_context_menu_for_file_type(
    _extension: &str,
    _menu_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[cfg(windows)]
fn remove_context_menu_for_file_type(
    extension: &str,
    menu_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use winreg::enums::*;
    use winreg::RegKey;

    // Open the HKEY_CLASSES_ROOT registry key
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);

    // Create the context menu entry
    let shell_key_path = format!(r"SystemFileAssociations\{}\Shell\{}", extension, menu_name);
    hkcr.delete_subkey_all(&shell_key_path)?;

    Ok(())
}
