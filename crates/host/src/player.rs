//! What TruckersTV takes from the player's game files, watched while it runs: from the game's config, its
//! language; for a truck whose speakers come with its parts, the speakers of the parts the newest save has on it
//! (fitted.rs), found again when the truck or its parts change. Read off the host's own thread (decrypting and
//! reading a save takes a moment), and only when a newer save appears.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant, SystemTime};

use westplay_mapdata::locale;
use westplay_mapdata::save;
use westplay_setup::profiles;
use westplay_setup::{game, places};
use westplay_shared::TruckInfo;

use crate::fitted::{self, SavedTruck};
use crate::speakers::{self, Speaker};

/// How often to look for a newer save.
const CHECK_EVERY: Duration = Duration::from_secs(10);

/// What each newer save says.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Player {
    /// The game's language (`tr_tr`…), from its config.
    pub language: Option<String>,
}

/// The speakers of the parts of the truck the SDK names (`None`: not found in the save, or no part with any).
#[derive(Clone, Debug, PartialEq)]
pub struct PartSpeakers {
    pub truck: TruckInfo,
    pub speakers: Option<Vec<Speaker>>,
}

pub struct PlayerWatch {
    pub updates: Receiver<Player>,
    pub speakers: Receiver<PartSpeakers>,
    trucks: Sender<TruckInfo>,
}

impl PlayerWatch {
    /// The truck the player is in now (the SDK's), for its parts' speakers.
    pub fn truck(&self, truck: TruckInfo) {
        let _ = self.trucks.send(truck);
    }

    pub fn start() -> Self {
        let (sender, updates) = mpsc::channel();
        let (speakers_sender, speakers) = mpsc::channel();
        let (trucks, truck_updates) = mpsc::channel::<TruckInfo>();
        let _ = std::thread::Builder::new().name("westplay-player".into()).spawn(move || {
            let documents = places::game_documents();
            // For the mods a profile plays: its Workshop folder and version, if the game is found.
            let game = game::find().into_iter().next();
            // Steam Cloud keeps the game's profiles under Steam's `userdata`.
            let userdata = places::steam().map(|dir| dir.join("userdata")).filter(|dir| dir.is_dir());
            let mut last: Option<(PathBuf, SystemTime)> = None;
            // The truck the player is in, the save's trucks and the profile's mods' files (in load order), and
            // what the parts' speakers were last found from.
            let mut truck: Option<TruckInfo> = None;
            let mut saved: Vec<SavedTruck> = Vec::new();
            let mut mod_files: Vec<PathBuf> = Vec::new();
            let mut found_from: Option<(TruckInfo, Option<SavedTruck>, Vec<PathBuf>)> = None;
            loop {
                // The newest save of the profile the game has selected (its log), else the newest of all.
                let latest = documents.as_deref().and_then(|dir| {
                    let selected = profiles::selected(dir);
                    let of_selected = selected.as_deref().and_then(|name| {
                        save::latest_of(dir, userdata.as_deref(), |folder| {
                            profiles::name_from_folder(folder).as_deref() == Some(name)
                        })
                    });
                    of_selected.or_else(|| save::latest(dir, userdata.as_deref()))
                });
                let stamped = latest.and_then(|path| Some((std::fs::metadata(&path).ok()?.modified().ok()?, path)));
                if let Some((modified, path)) = stamped
                    && last.as_ref() != Some(&(path.clone(), modified))
                {
                    match read_units(&path) {
                        Ok(units) => {
                            let mods = save::profile_of(&path)
                                .and_then(|profile| read_units(&profile).ok())
                                .map(|units| save::active_mods(&units))
                                .unwrap_or_default();
                            let mod_dir = documents.as_deref().map(|dir| dir.join("mod")).unwrap_or_default();
                            saved = fitted::saved_trucks(&units);
                            mod_files = mods
                                .iter()
                                .filter_map(|listed| profiles::mod_files(listed, game.as_ref(), &mod_dir))
                                .collect();
                            let player = Player { language: documents.as_deref().and_then(game_language) };
                            if sender.send(player).is_err() {
                                return;
                            }
                        }
                        Err(error) => log!("save {}: {error}", path.display()),
                    }
                    last = Some((path, modified));
                }
                // The speakers of the truck's parts, found again when the truck, its parts or the mods change.
                if let (Some(truck), Some(game)) = (truck, game.as_ref())
                    && speakers::by_parts(truck.id.as_str())
                {
                    let saved_truck = fitted::find(&saved, &truck).cloned();
                    let from = (truck, saved_truck, mod_files.clone());
                    if found_from.as_ref() != Some(&from) {
                        let speakers = part_speakers(&truck, from.1.as_ref(), &game.dir, &mod_files);
                        found_from = Some(from);
                        if speakers_sender.send(PartSpeakers { truck, speakers }).is_err() {
                            return;
                        }
                    }
                }
                // Then the next truck, or a while.
                match truck_updates.recv_timeout(CHECK_EVERY) {
                    Ok(first) => truck = Some(truck_updates.try_iter().last().unwrap_or(first)),
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => return,
                }
            }
        });
        Self { updates, speakers, trucks }
    }
}

/// The speakers of `truck`'s parts as `saved` (the save's of it) has them, read from the game in `game_dir` and the
/// mods' `mod_files`; `None` (said in the log) if it isn't in the save or none of its parts has speakers.
fn part_speakers(
    truck: &TruckInfo,
    saved: Option<&SavedTruck>,
    game_dir: &Path,
    mod_files: &[PathBuf],
) -> Option<Vec<Speaker>> {
    let name = format!("{} ({})", truck.id.as_str(), truck.plate.as_str());
    let Some(saved) = saved else {
        log!("speakers: {name} isn't in the save yet");
        return None;
    };
    let started = Instant::now();
    let found =
        fitted::mount(game_dir, mod_files, &saved.model).and_then(|files| fitted::speakers(&files, saved, truck.head));
    let took = started.elapsed().as_secs_f32();
    match found {
        Ok(Some(fitted)) => {
            log!(
                "speakers: {name}, {} from its parts in {took:.1} s: {}",
                fitted.speakers.len(),
                fitted.parts.join(", ")
            );
            Some(fitted.speakers)
        }
        Ok(None) => {
            log!("speakers: {name}: none of its parts has speakers known ({took:.1} s)");
            None
        }
        Err(error) => {
            log!("speakers: {name}: its parts couldn't be read: {error}");
            None
        }
    }
}

fn read_units(path: &Path) -> Result<Vec<westplay_mapdata::sii::Unit>, String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    save::read(&data).map_err(|e| e.to_string())
}

/// The game's language, from its config (`uset g_lang_init "tr_tr"`).
fn game_language(game_documents: &Path) -> Option<String> {
    locale::game_language(&std::fs::read_to_string(game_documents.join("config.cfg")).ok()?)
}
