//! "Start on boot up" support: registers the app in the per-user Windows
//! startup list (HKCU\\...\\Run) so it launches into the tray at sign-in.

#[cfg(target_os = "windows")]
mod imp {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const VALUE_NAME: &str = "L5P-Keyboard-RGB";

    pub fn is_enabled() -> bool {
        RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(RUN_KEY)
            .and_then(|key| key.get_value::<String, _>(VALUE_NAME))
            .is_ok()
    }

    pub fn set_enabled(enabled: bool) -> std::io::Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu.create_subkey(RUN_KEY)?;

        if enabled {
            let exe = std::env::current_exe()?;
            // Start hidden in the tray with the saved profile/settings.
            let command = format!("\"{}\" --gui -w", exe.display());
            key.set_value(VALUE_NAME, &command)
        } else {
            match key.delete_value(VALUE_NAME) {
                Ok(()) => Ok(()),
                // Already absent counts as disabled.
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(err) => Err(err),
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub use imp::{is_enabled, set_enabled};

#[cfg(not(target_os = "windows"))]
pub fn is_enabled() -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
pub fn set_enabled(_enabled: bool) -> std::io::Result<()> {
    Ok(())
}
