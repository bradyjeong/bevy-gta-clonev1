use bevy::prelude::*;

/// Application state machine for world generation and gameplay
/// AssetLoading (with splash screen) -> WorldGeneration -> InGame
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    AssetLoading,
    WorldGeneration,
    InGame,
}
