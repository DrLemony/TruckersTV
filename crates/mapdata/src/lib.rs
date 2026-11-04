//! Reads the player's own copy of the game: its `.scs` archives ([`archive`], mounted by [`files`]), definitions and
//! saves. Nothing read here is shipped: the TV is fitted from each player's own game files.
//!
//! The layout of the game's archives was worked out by the ts-map project
//! (<https://github.com/dariowouters/ts-map>, MIT licence, © Dario Wouters), whose readers this crate's follow; see
//! THIRD_PARTY_NOTICES.md.

pub mod archive;
pub mod buildings;
/// CityHash64, shared with the plugin.
pub use westplay_shared::cityhash;
pub mod files;
pub mod gdeflate;
pub mod locale;
pub mod reader;
pub mod save;
pub mod sii;
pub mod token;

#[cfg(test)]
mod test_dir;
