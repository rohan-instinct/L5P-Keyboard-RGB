use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use crate::manager::{custom_effect::CustomEffect, profile::Profile};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Settings {
    pub profiles: Vec<Profile>,
    pub effects: Vec<CustomEffect>,
    // Up to 0.19.5
    #[serde(alias = "ui_state")]
    pub current_profile: Profile,
}

impl Settings {
    pub fn new(profiles: Vec<Profile>, effects: Vec<CustomEffect>, current_profile: Profile) -> Self {
        Self { profiles, effects, current_profile }
    }

    /// Load the settings from the configured path or generate default ones if an error occurs
    pub fn load() -> Self {
        let mut persist: Self = Self::default();

        // Candidates: exe-adjacent, AppData fallback, legacy working-dir file.
        // The NEWEST existing file wins, so a fallback save (e.g. exe folder
        // not writable) is never shadowed by a stale exe-adjacent copy.
        let mut newest: Option<(PathBuf, std::time::SystemTime)> = None;
        for path in [Self::get_location(), Self::appdata_location(), PathBuf::from("./settings.json")] {
            if path.exists() && path.is_file() {
                let modified = fs::metadata(&path).and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                if newest.as_ref().map_or(true, |(_, t)| modified > *t) {
                    newest = Some((path, modified));
                }
            }
        }

        if let Some((path, _)) = newest {
            if let Ok(string) = fs::read_to_string(&path) {
                persist = serde_json::from_str(&string).unwrap_or_default();
            }
        }

        persist
    }

    /// Save the settings. Tries the exe-adjacent location first, then a
    /// per-user AppData fallback, so an unwritable install folder (e.g.
    /// Program Files) never silently discards changes. Returns whether the
    /// settings were written anywhere.
    pub fn save(&mut self) -> bool {
        let stringified_json = serde_json::to_string(&self).unwrap();

        for path in [Self::get_location(), Self::appdata_location()] {
            if let Some(parent) = path.parent() {
                if fs::create_dir_all(parent).is_err() {
                    continue;
                }
            }
            if let Ok(mut file) = File::create(&path) {
                if file.write_all(stringified_json.as_bytes()).is_ok() {
                    return true;
                }
            }
        }

        eprintln!("L5P-Keyboard-RGB: could not save settings - no writable location found");
        false
    }

    /// Per-user fallback storage: %APPDATA%\L5P-Keyboard-RGB\settings.json
    /// on Windows; the exe-adjacent path elsewhere (APPDATA unset).
    fn appdata_location() -> PathBuf {
        if let Ok(appdata) = env::var("APPDATA") {
            let dir = PathBuf::from(appdata).join("L5P-Keyboard-RGB");
            return dir.join("settings.json");
        }

        Self::get_location()
    }

    fn get_location() -> PathBuf {
        // Honor the explicit override first.
        if let Ok(maybe_path) = env::var("LEGION_KEYBOARD_CONFIG") {
            let path = PathBuf::from(maybe_path);
            if path.exists() && path.is_file() {
                return path;
            }
        }

        // Default: store settings next to the executable so the location is
        // stable regardless of the working directory the app is launched from
        // (Start Menu shortcuts, Task Scheduler, explorer, CLI all agree).
        if let Ok(exe) = env::current_exe() {
            if let Some(dir) = exe.parent() {
                return dir.join("settings.json");
            }
        }

        PathBuf::from("./settings.json")
    }
}
