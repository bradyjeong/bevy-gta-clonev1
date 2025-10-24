/// Utilities for asset path resolution across different platforms and build configurations.
///
/// # macOS Bundle Structure
///
/// When the game is packaged as a macOS .app bundle, the directory structure looks like:
/// ```text
/// MyGame.app/
/// ├── Contents/
/// │   ├── MacOS/
/// │   │   └── game_executable    <- Current working directory when running
/// │   └── Resources/
/// │       └── assets/            <- Asset files location
/// ```
///
/// The executable runs from `Contents/MacOS/`, so assets are accessed via `../Resources/assets`.
///
/// In development builds, the executable runs from the project root, so assets are in `./assets`.
///
/// Returns the base path to the assets directory.
///
/// # Platform-Specific Behavior
///
/// ## macOS Bundle (.app)
/// When running inside a macOS .app bundle, returns `"../Resources/assets"` to navigate from
/// `Contents/MacOS/` to `Contents/Resources/assets/`.
///
/// ## Development Build
/// When running from the project directory (cargo run), returns `"assets"` for direct access.
///
/// # Detection Logic
///
/// The function detects macOS bundles by checking if the current executable path contains
/// `.app/Contents/MacOS`. This is a reliable indicator that the game is running from within
/// a properly structured macOS application bundle.
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::*;
/// use crate::util::asset_path::get_assets_base_path;
///
/// // Configure AssetPlugin with correct path
/// App::new()
///     .add_plugins(DefaultPlugins.set(AssetPlugin {
///         file_path: get_assets_base_path(),
///         ..default()
///     }));
///
/// // Load config files from correct location
/// let config_path = format!("{}/config/game.ron", get_assets_base_path());
/// ```
pub fn get_assets_base_path() -> String {
    if cfg!(target_os = "macos")
        && std::env::current_exe()
            .map(|exe| exe.to_string_lossy().contains(".app/Contents/MacOS"))
            .unwrap_or(false)
    {
        "../Resources/assets".to_string()
    } else {
        "assets".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_assets_base_path_returns_valid_string() {
        let path = get_assets_base_path();
        assert!(path == "assets" || path == "../Resources/assets");
    }
}
