//! The player's profiles (in the game's Documents folder, and Steam Cloud's), and their mods' files. Read-only.

use std::path::{Path, PathBuf};

use westplay_mapdata::save;
use westplay_mapdata::sii::Unit;

use crate::game::{Game, STEAM_APP};

/// Every profile's folder, from the game's Documents folder (`game_documents`) and Steam Cloud's (under Steam's
/// `userdata`).
pub fn find(game_documents: &Path, steam_userdata: Option<&Path>) -> Vec<PathBuf> {
    let mut folders = vec![game_documents.join("profiles"), game_documents.join("steam_profiles")];
    if let Some(userdata) = steam_userdata {
        for user in std::fs::read_dir(userdata).into_iter().flatten().flatten() {
            folders.push(user.path().join(STEAM_APP).join("remote").join("profiles"));
        }
    }
    folders
        .iter()
        .flat_map(|folder| std::fs::read_dir(folder).into_iter().flatten().flatten().map(|entry| entry.path()))
        .filter(|dir| read_units(&dir.join("profile.sii")).is_some())
        .collect()
}

fn read_units(path: &Path) -> Option<Vec<Unit>> {
    save::read(&std::fs::read(path).ok()?).ok()
}

/// A profile's name from its folder's: the name's UTF-8 bytes in hexadecimal (`485654202D2053617665`).
pub fn name_from_folder(folder: &str) -> Option<String> {
    if folder.is_empty() || !folder.len().is_multiple_of(2) {
        return None;
    }
    let bytes: Option<Vec<u8>> =
        (0..folder.len()).step_by(2).map(|i| u8::from_str_radix(folder.get(i..i + 2)?, 16).ok()).collect();
    String::from_utf8(bytes?).ok()
}

/// The profile the game has loaded (or last had), by its name, as the game's log says it selects one
/// (`New profile selected: 'Eren'`, in `game.log.txt` in the game's Documents folder, written anew each time
/// the game starts). It says so as the profile loads; a save may not come for a while (a profile not played
/// for years has none newer than another's). `None` when the log doesn't say.
pub fn selected(documents: &Path) -> Option<String> {
    selected_in(&String::from_utf8_lossy(&std::fs::read(documents.join("game.log.txt")).ok()?))
}

fn selected_in(log: &str) -> Option<String> {
    log.lines()
        .rev()
        .find_map(|line| line.trim_end().split_once("New profile selected: '")?.1.strip_suffix('\'').map(str::to_owned))
}

/// A mod's files as a profile lists it (`<package>|<name as shown>`), in the Documents folder's `mod` or the game's
/// Workshop folder: an archive or a folder to mount.
pub fn mod_files(listed: &str, game: Option<&Game>, mod_dir: &Path) -> Option<PathBuf> {
    let package = listed.split_once('|').map_or(listed, |(package, _)| package);
    match workshop_id(package) {
        Some(id) => {
            let folder = game.and_then(Game::workshop).map(|workshop| workshop.join(id.to_string()));
            folder.and_then(|folder| workshop_files(&folder, game.and_then(|game| game.version)))
        }
        None => ["scs", "zip"]
            .iter()
            .map(|extension| mod_dir.join(format!("{package}.{extension}")))
            .chain([mod_dir.join(package)])
            .find(|path| path.exists()),
    }
}

/// The installed mods' files (an archive or a folder each), whether a profile has them active or not: each
/// in the Documents folder's `mod` (and so said), and each Workshop item's for `game`'s version.
pub fn installed_files(game_documents: &Path, game: Option<&Game>) -> Vec<(PathBuf, bool)> {
    let in_mod_folder =
        std::fs::read_dir(game_documents.join("mod")).into_iter().flatten().flatten().filter_map(|entry| {
            let path = entry.path();
            let archive = path.extension().is_some_and(|extension| extension == "scs" || extension == "zip");
            (archive || path.is_dir()).then_some((path, true))
        });
    let workshop = game.and_then(Game::workshop);
    let in_workshop = workshop
        .iter()
        .flat_map(|folder| std::fs::read_dir(folder).into_iter().flatten().flatten())
        .filter_map(|item| Some((workshop_files(&item.path(), game.and_then(|game| game.version))?, false)));
    in_mod_folder.chain(in_workshop).collect()
}

/// A Workshop mod's item id, from its package name (`mod_workshop_package.00000000ABCDEF01`).
pub fn workshop_id(package: &str) -> Option<u64> {
    u64::from_str_radix(package.strip_prefix("mod_workshop_package.")?, 16).ok()
}

/// What of a Workshop item to mount for the game's `version`: the package its `versions.sii` names for that
/// version, else one for any version (`universal`, `latest`…), as a folder or an archive (`sound.scs`,
/// `latest.zip`); an item without the file is its own folder.
fn workshop_files(item: &Path, version: Option<crate::game::Version>) -> Option<PathBuf> {
    let Some(units) = read_units(&item.join("versions.sii")) else {
        return item.is_dir().then(|| item.to_path_buf());
    };
    let packages = packages_by_version(&units);
    let (package, _) = version
        .and_then(|version| packages.iter().find(|(_, versions)| versions.iter().any(|v| version.matches(v))))
        .or_else(|| packages.iter().find(|(_, versions)| versions.is_empty()))?;
    [item.join(package), item.join(format!("{package}.scs")), item.join(format!("{package}.zip"))]
        .into_iter()
        .find(|path| path.exists())
}

/// A Workshop item's packages (`versions.sii`): each one's folder and the game versions it's for (none: any,
/// its `universal` one).
pub fn packages_by_version(units: &[Unit]) -> Vec<(String, Vec<String>)> {
    units
        .iter()
        .filter(|unit| unit.class == "package_version_info")
        .filter_map(|unit| {
            let versions = unit.all("compatible_versions").map(str::to_owned).collect();
            Some((unit.get("package_name")?.to_owned(), versions))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_a_profile_from_its_folder() {
        assert_eq!(name_from_folder("485654202D2053617665").as_deref(), Some("HVT - Save"));
        assert_eq!(name_from_folder("C384"), Some("Ä".into()));
        assert_eq!(name_from_folder("4"), None);
        assert_eq!(name_from_folder("zz"), None);
    }


    #[test]
    fn reads_a_workshop_items_id() {
        assert_eq!(workshop_id("mod_workshop_package.00000000449D50B1"), Some(1_151_160_497));
        assert_eq!(workshop_id("Jaspers_DAF_2021_v1.2"), None);
    }


    #[test]
    fn picks_a_workshop_items_package_for_the_game_version() {
        let units = westplay_mapdata::sii::parse(
            "SiiNunit\n{\npackage_version_info : .144.content {\n package_name: \"144_content\"\n compatible_versions[]: \"1.44.*\"\n}\npackage_version_info : .universal {\n package_name: \"universal\"\n}\n}",
        );
        let packages = packages_by_version(&units);
        assert_eq!(packages, vec![("144_content".into(), vec!["1.44.*".into()]), ("universal".into(), vec![])]);
    }


    #[test]
    fn tells_the_profile_the_game_selected_from_its_log() {
        let log = "00:00:40.610 : [fs] device mounted to mod pool.\r\n\
                   00:01:02.000 : New profile selected: 'W'\r\n\
                   00:02:34.591 : Requested profile is already current profile.\r\n\
                   00:02:34.628 : New profile selected: 'Eren'\r\n\
                   00:02:40.000 : [mods] something else\r\n";
        // The last one selected; a name with a quote in it kept whole.
        assert_eq!(selected_in(log).as_deref(), Some("Eren"));
        assert_eq!(selected_in("New profile selected: 'Ali's'").as_deref(), Some("Ali's"));
        assert_eq!(selected_in("00:00:01.000 : [sys] running"), None);
    }


    #[test]
    fn a_mod_that_isnt_there_has_no_files() {
        assert_eq!(mod_files("missing_mod|A missing mod", None, Path::new("nowhere")), None);
    }
}
