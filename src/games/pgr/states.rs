use std::path::PathBuf;

use serde::{Serialize, Deserialize};

use anime_game_core::prelude::*;
use anime_game_core::pgr::prelude::*;

use crate::config::ConfigExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LauncherState {
    Launch,

    #[cfg(feature = "components")]
    WineNotInstalled,

    PrefixNotExists,

    // Always contains `VersionDiff::Diff`
    GameUpdateAvailable(VersionDiff),

    /// Always contains `VersionDiff::NotInstalled`
    GameNotInstalled(VersionDiff)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateUpdating {
    Game
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherStateParams<F: Fn(StateUpdating)> {
    pub wine_prefix: PathBuf,
    pub game_path: PathBuf,
    pub status_updater: F
}

impl LauncherState {
    pub fn get<F: Fn(StateUpdating)>(params: LauncherStateParams<F>) -> anyhow::Result<Self> {
        tracing::debug!("Trying to get launcher state");

        // Check prefix existence
        if !params.wine_prefix.join("drive_c").exists() {
            return Ok(Self::PrefixNotExists);
        }

        // Check game installation status
        (params.status_updater)(StateUpdating::Game);

        let game = Game::new(&params.game_path, ());

        let diff = game.try_get_diff()?;

        match diff {
            VersionDiff::Latest(_) => Ok(Self::Launch),
            VersionDiff::Outdated { .. } => Ok(Self::GameUpdateAvailable(diff)),
            VersionDiff::NotInstalled { .. } => Ok(Self::GameNotInstalled(diff))
        }
    }

    #[cfg(feature = "config")]
    pub fn get_from_config<T: Fn(StateUpdating)>(status_updater: T) -> anyhow::Result<Self> {
        tracing::debug!("Trying to get launcher state");

        let config = crate::pgr::config::Config::get()?;

        match &config.game.wine.selected {
            #[cfg(feature = "components")]
            Some(selected) if !config.game.wine.builds.join(selected).exists() => return Ok(Self::WineNotInstalled),

            None => return Ok(Self::WineNotInstalled),

            _ => ()
        }

        Self::get(LauncherStateParams {
            wine_prefix: config.get_wine_prefix_path(),
            game_path: config.game.path,

            status_updater
        })
    }
}
