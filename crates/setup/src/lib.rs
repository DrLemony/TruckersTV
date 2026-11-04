//! Sets TruckersTV up on the player's PC, for the companion's onboarding: finds the game (every install, its
//! version and DLCs), the player's profiles and the mods each one plays with, and which of them carry a map.
//! Finding is read-only; building TruckersTV's game mod writes only its own file (`game_mod`).

pub mod cab_mark;
pub mod cab_tv;
pub mod game;
pub mod game_mod;
pub mod model;
pub mod pick;
pub mod places;
pub mod plugin;
pub mod profiles;
pub mod record;
pub mod uninstall;
pub mod vdf;
