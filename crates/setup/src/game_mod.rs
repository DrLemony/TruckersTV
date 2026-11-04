//! TruckersTV's game mod (`truckerstv.scs`), built on the player's PC from their own game: the cab TV (`cab_tv`) with
//! its screen texture, and the cab's mark in the dashboard scripts of the trucks the TV goes in (`cab_mark`). Nothing
//! of the game's is shipped: the scripts are the player's own copy's, patched, and the rest is TruckersTV's.
//!
//! The texture starts fully transparent, until the plugin fills it.
//!
//! It isn't put in the game's `mod` folder or shown in its Mod Manager: it's written into TruckersTV's own folder
//! (`places::westplay_documents`), and the plugin mounts it into the game itself, first of all mods, in every mode
//! (single player, Convoy, TruckersMP). A copy in the game's `mod` folder is taken out (`uninstall::remove_mod`).
//!
//! The mod is a zip, as `.scs` mods may be.

use std::io::Write;
use std::path::{Path, PathBuf};

use westplay_mapdata::archive::Archive;
use westplay_mapdata::files::{GameFiles, Mount};
use westplay_shared::{CAB_MARK_HEIGHT, CAB_MARK_WIDTH, SCREEN_WIDTH, ScreenLayout, screen_layouts};

use crate::cab_mark;

/// The mod's file, in TruckersTV's own folder (`places::westplay_documents`).
pub const FILE_NAME: &str = "truckerstv.scs";
/// Where the mod's screen materials and textures live in the game's file system.
const SCREEN_DIR: &str = "material/ui/westplay";
/// The screen's draw order in its group: the stock elements use layers 0 to 3.
const SCREEN_LAYER: u32 = 100;

/// Whether the mod built at `built` has the cab TV as it would be made now from `game` (`cab_tv::current`): placed
/// as now, in the trucks the game has.
pub fn cab_tv_current(game: &GameFiles, built: &Path) -> bool {
    let Ok(archive) = Archive::open(built) else { return false };
    let ours = GameFiles::new(vec![Mount::Archive(archive)]);
    let index = westplay_shared::cab_tv_layout();
    let texture = texture_name(&screen_layouts()[index]);
    crate::cab_tv::current(game, &ours, &format!("/{SCREEN_DIR}/{texture}.tobj"), ScreenLayout::texture_height(index))
}

/// What a build made: why the TV is left out, if it is (no truck it goes in), and how many dashboard scripts have the
/// cab's mark.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Built {
    pub no_tv: Option<String>,
    pub cab_marks: usize,
}

/// The mod for `game`: the cab TV and its screen texture, and the cab's mark in `cab_marks`' scripts.
pub fn build(game: &GameFiles, cab_marks: &cab_mark::Marks) -> (Vec<u8>, Built) {
    let mut files: Vec<(String, Vec<u8>)> = vec![
        ("manifest.sii".into(), manifest().into_bytes()),
        ("description.txt".into(), b"TruckersTV cab TV. Requires the TruckersTV app and plugin.\n".to_vec()),
    ];
    let mut built = Built::default();
    let index = westplay_shared::cab_tv_layout();
    let texture = texture_name(&screen_layouts()[index]);
    let height = ScreenLayout::texture_height(index);
    files.push((format!("{SCREEN_DIR}/{texture}.mat"), material(&texture).into_bytes()));
    files.push((format!("{SCREEN_DIR}/{texture}.tobj"), tobj(&format!("/{SCREEN_DIR}/{texture}.dds"))));
    files.push((format!("{SCREEN_DIR}/{texture}.dds"), transparent_dds(SCREEN_WIDTH, height)));
    match crate::cab_tv::files(game, &format!("/{SCREEN_DIR}/{texture}.tobj"), height) {
        Ok(tv) => files.extend(tv),
        Err(why) => built.no_tv = Some(why),
    }
    // The cab's mark: its texture, fully transparent, and the dashboard scripts with it, listed.
    let mark = cab_mark::TEXTURE;
    files.push((format!("{SCREEN_DIR}/{mark}.mat"), material(mark).into_bytes()));
    files.push((format!("{SCREEN_DIR}/{mark}.tobj"), tobj(&format!("/{SCREEN_DIR}/{mark}.dds"))));
    files.push((format!("{SCREEN_DIR}/{mark}.dds"), transparent_dds(CAB_MARK_WIDTH, CAB_MARK_HEIGHT)));
    files.push((cab_mark::LIST.to_owned(), cab_marks.list().into_bytes()));
    files.extend(cab_marks.scripts.iter().cloned());
    built.cab_marks = cab_marks.scripts.len();
    files.sort_by(|a, b| a.0.cmp(&b.0));
    (zip(&files), built)
}

/// Writes the mod into `mod_dir` (TruckersTV's own folder), replacing the one there whole: written beside it
/// first, then moved over it, so the plugin never finds half a file.
/// A step that failed names itself and its path (`writing C:\…\TruckersTV\truckerstv.scs.partial: …`), so a
/// player's report says which.
pub fn install(bytes: &[u8], mod_dir: &Path) -> std::io::Result<PathBuf> {
    let step = |what: &str, path: &Path| {
        let what = format!("{what} {}", path.display());
        move |error: std::io::Error| std::io::Error::new(error.kind(), format!("{what}: {error}"))
    };
    std::fs::create_dir_all(mod_dir).map_err(step("making the folder", mod_dir))?;
    let path = mod_dir.join(FILE_NAME);
    let partial = mod_dir.join(format!("{FILE_NAME}.partial"));
    std::fs::write(&partial, bytes).map_err(step("writing", &partial))?;
    std::fs::rename(&partial, &path).map_err(step("moving it over", &path))?;
    Ok(path)
}

/// A layout's screen texture's name: `screen_daf_2021`.
pub fn texture_name(layout: &ScreenLayout) -> String {
    format!("screen_{}", layout.id.replace('-', "_"))
}

/// The mod's manifest. Optional in convoys (`mp_mod_optional`, as Workshop mods mark it): it changes only the
/// player's own truck screen, so players with and without it join each other's convoys.
fn manifest() -> String {
    format!(
        "SiiNunit\n{{\nmod_package : .package_name\n{{\n\tpackage_version: \"{}\"\n\tdisplay_name: \"TruckersTV\"\n\tauthor: \"Western\"\n\tcategory[]: \"ui\"\n\tdescription_file: \"description.txt\"\n\tmp_mod_optional: true\n}}\n}}\n",
        env!("CARGO_PKG_VERSION")
    )
}

/// A UI material showing `texture`'s texture object, beside it (the screen's, and the TV's shop icon).
pub(crate) fn material(texture: &str) -> String {
    format!(
        "effect : \"ui.rfx\" {{\n\ttexture : \"texture\" {{\n\t\tsource : \"{texture}.tobj\"\n\t\tu_address : clamp\n\t\tv_address : clamp\n\t\tmip_filter : none\n\t}}\n}}\n"
    )
}

/// A texture object pointing at `dds_path`: the header of a stock UI texture object (`/material/ui/white.tobj`),
/// then the path.
pub(crate) fn tobj(dds_path: &str) -> Vec<u8> {
    const HEADER: [u8; 40] = [
        0x01, 0x0a, 0xb1, 0x70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01, 0, 0, 0, 0x02, 0, 0x01, 0x01,
        0x02, 0, 0x02, 0x02, 0, 0, 0, 0, 0, 0x01, 0, 0,
    ];
    let mut bytes = HEADER.to_vec();
    bytes.extend((dds_path.len() as u32).to_le_bytes());
    bytes.extend(0u32.to_le_bytes());
    bytes.extend(dds_path.as_bytes());
    bytes
}

/// A fully transparent `width` x `height` texture: a DDS with a DX10 header, B8G8R8A8_UNORM_SRGB (91), the
/// format the plugin copies TruckersTV's frames into.
fn transparent_dds(width: u32, height: u32) -> Vec<u8> {
    screen_dds(width, height, &vec![0; (width * height * 4) as usize])
}

/// A `width` x `height` texture of `pixels` (blue, green, red, alpha each, row by row), laid out as the screens'
/// ([`transparent_dds`]).
pub(crate) fn screen_dds(width: u32, height: u32, pixels: &[u8]) -> Vec<u8> {
    let mut header = [0u8; 148];
    header[0..4].copy_from_slice(b"DDS ");
    for (at, value) in [(4, 124u32), (8, 0x1007), (12, height), (16, width), (76, 32), (80, 0x4), (108, 0x1000)] {
        header[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    header[84..88].copy_from_slice(b"DX10");
    for (at, value) in [(128, 91u32), (132, 3), (136, 0), (140, 1), (144, 0)] {
        header[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    let mut bytes = header.to_vec();
    bytes.extend(pixels);
    bytes
}

/// A top-level `ui::kind : name {` block of a UI script: its name, and its lines (header to brace).
struct Block<'a> {
    name: &'a str,
    start: usize,
    end: usize,
}

fn blocks<'a>(lines: &[&'a str]) -> Vec<Block<'a>> {
    let mut blocks = Vec::new();
    let mut current: Option<(&str, usize)> = None;
    for (index, line) in lines.iter().enumerate() {
        if current.is_none() {
            current = block_header(line).map(|name| (name, index));
        } else if line.trim() == "}" {
            let (name, start) = current.take().unwrap();
            blocks.push(Block { name, start, end: index });
        }
    }
    blocks
}

/// `ui::group : _nameless.0abc {` → `_nameless.0abc`.
fn block_header(line: &str) -> Option<&str> {
    let rest = line.trim_end().strip_suffix('{')?.trim_end();
    let (kind, name) = rest.split_once(" : ")?;
    let kind_ok = kind
        .strip_prefix("ui::")
        .is_some_and(|word| !word.is_empty() && word.chars().all(|c| c.is_alphanumeric() || c == '_'));
    (kind_ok && !name.is_empty() && !name.contains(char::is_whitespace)).then_some(name)
}

/// A field's value in a block (`id: 100` → `100`).
fn field<'a>(lines: &[&'a str], block: &Block, name: &str) -> Option<&'a str> {
    lines[block.start + 1..block.end].iter().find_map(|line| {
        let (key, value) = line.trim().split_once(':').unwrap_or((line.trim(), ""));
        (key == name).then(|| value.trim())
    })
}

/// Adds the cab's mark (`texture`, `cab_mark`) to a dashboard script: a child of its window (its one block with no
/// parent), a unit square in the window's corner, fully transparent. The rest of the script stays as it is.
pub fn patch_dashboard_script(text: &str, texture: &str) -> Result<String, String> {
    if text.contains("_nameless.westplay.cab") {
        return Err("has the mark already".into());
    }
    // Its left and bottom corner, a unit in from both (its coordinates may go up the screen or down it).
    add_to_window(text, "_nameless.westplay.cab", texture, |[left, _, top, bottom]| {
        let inward = if top >= bottom { 1.0 } else { -1.0 };
        [left, left + 1.0, bottom + inward, bottom]
    })
}

/// Adds `texture`'s image to a UI script as the last child of its window (its one block with no parent), as `element`,
/// where `place` puts it (left, right, top, bottom) from the window's own coordinates (the same).
fn add_to_window(
    text: &str,
    element: &str,
    texture: &str,
    place: impl Fn([f64; 4]) -> [f64; 4],
) -> Result<String, String> {
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let lines: Vec<&str> = text.split(newline).collect();
    let blocks = blocks(&lines);
    let roots: Vec<&Block> = blocks.iter().filter(|block| field(&lines, block, "my_parent") == Some("null")).collect();
    let [window] = roots[..] else { return Err(format!("{} blocks without a parent, not one", roots.len())) };
    let count: usize =
        field(&lines, window, "my_children").unwrap_or("0").parse().map_err(|_| "my_children isn't a count")?;
    let coord = |name| -> Result<f64, String> {
        let text = field(&lines, window, name).ok_or_else(|| format!("the window has no {name}"))?;
        text.parse().map_err(|_| format!("the window's {name} {text}"))
    };
    let [left, right, top, bottom] =
        place([coord("coords_l")?, coord("coords_r")?, coord("coords_t")?, coord("coords_b")?]);
    let at = |value: f64| format!("{}", value.round() as i64);

    let mut body = Vec::new();
    for line in &lines[window.start + 1..window.end] {
        let trimmed = line.trim();
        let line = if trimmed.starts_with("my_children:") {
            format!(" my_children: {}", count + 1)
        } else {
            (*line).to_owned()
        };
        let after = if count == 0 {
            trimmed.starts_with("my_children:")
        } else {
            trimmed.starts_with(&format!("my_children[{}]:", count - 1))
        };
        body.push(line);
        if after {
            body.push(format!(" my_children[{count}]: {element}"));
        }
    }
    let image = [
        String::new(),
        format!("ui::text : {element} {{"),
        format!(" text: \"<img src=/{SCREEN_DIR}/{texture}.mat xscale=stretch yscale=stretch>\""),
        format!(" coords_l: {}", at(left)),
        format!(" coords_r: {}", at(right)),
        format!(" coords_t: {}", at(top)),
        format!(" coords_b: {}", at(bottom)),
        " area_l: 1".into(),
        " area_r: 0".into(),
        " area_t: 0".into(),
        " area_b: 1".into(),
        " id: 0".into(),
        format!(" layer: {SCREEN_LAYER}"),
        " tab: -1".into(),
        " pointer: -1".into(),
        format!(" my_parent: {}", window.name),
        "}".into(),
    ];
    let last_end = blocks.last().map(|block| block.end).ok_or("no blocks")?;
    let mut patched: Vec<String> = lines[..=window.start].iter().map(|line| (*line).to_owned()).collect();
    patched.extend(body);
    patched.extend(lines[window.end..=last_end].iter().map(|line| (*line).to_owned()));
    patched.extend(image);
    patched.extend(lines[last_end + 1..].iter().map(|line| (*line).to_owned()));
    Ok(patched.join(newline))
}

/// A zip of `files` (path, contents), deflated, as the game reads `.scs` mods.
pub(crate) fn zip(files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let (mut data, mut directory) = (Vec::new(), Vec::new());
    for (path, contents) in files {
        let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(contents).expect("deflating to memory");
        let packed = encoder.finish().expect("deflating to memory");
        let mut crc = flate2::Crc::new();
        crc.update(contents);
        let local = data.len() as u32;
        // Version 2.0, no flags, deflated, no time.
        let common = |out: &mut Vec<u8>| {
            out.extend(20u16.to_le_bytes());
            out.extend(0u16.to_le_bytes());
            out.extend(8u16.to_le_bytes());
            out.extend([0u8; 4]);
            out.extend(crc.sum().to_le_bytes());
            out.extend((packed.len() as u32).to_le_bytes());
            out.extend((contents.len() as u32).to_le_bytes());
            out.extend((path.len() as u16).to_le_bytes());
            out.extend(0u16.to_le_bytes());
        };
        data.extend(b"PK\x03\x04");
        common(&mut data);
        data.extend(path.as_bytes());
        data.extend(&packed);
        directory.extend(b"PK\x01\x02");
        directory.extend(20u16.to_le_bytes());
        common(&mut directory);
        // No comment, disk 0, no attributes.
        directory.extend([0u8; 10]);
        directory.extend(local.to_le_bytes());
        directory.extend(path.as_bytes());
    }
    let offset = data.len() as u32;
    let size = directory.len() as u32;
    data.extend(directory);
    data.extend(b"PK\x05\x06");
    data.extend([0u8; 4]);
    data.extend((files.len() as u16).to_le_bytes());
    data.extend((files.len() as u16).to_le_bytes());
    data.extend(size.to_le_bytes());
    data.extend(offset.to_le_bytes());
    data.extend(0u16.to_le_bytes());
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installs_the_mod_whole_and_says_which_step_failed_where() {
        let root = std::env::temp_dir().join(format!("westplay-install-{}", std::process::id()));
        let mod_dir = root.join("Euro Truck Simulator 2").join("mod");
        let path = install(b"mod", &mod_dir).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"mod");
        assert!(!mod_dir.join("truckerstv.scs.partial").exists());
        // A file where the folder should be: the step and its path in the error.
        let blocked = root.join("blocked");
        std::fs::write(&blocked, b"").unwrap();
        let error = install(b"mod", &blocked.join("mod")).unwrap_err().to_string();
        assert!(error.starts_with("making the folder ") && error.contains("blocked"), "{error}");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Optional in convoys: a friend without TruckersTV can join, and be joined.
    #[test]
    fn the_mod_is_optional_in_convoys() {
        let units = westplay_mapdata::sii::parse(&manifest());
        let package = units.iter().find(|unit| unit.class == "mod_package").expect("a mod_package");
        assert_eq!(package.get("mp_mod_optional"), Some("true"));
    }

    #[test]
    fn marks_a_dashboard_in_its_windows_corner_once() {
        let script = "SiiNunit\r\n{\r\nui::window : _nameless.w {\r\n my_children: 0\r\n coords_l: 10\r\n coords_r: 500\r\n coords_t: 0\r\n coords_b: 300\r\n my_parent: null\r\n}\r\n\r\n}\r\n";
        let patched = patch_dashboard_script(script, "cab_mark").unwrap();
        assert!(patched.contains(" my_children: 1\r\n my_children[0]: _nameless.westplay.cab\r\n"), "{patched}");
        // Its coordinates going down the screen: the square a unit up from the bottom corner.
        assert!(
            patched.contains(" coords_l: 10\r\n coords_r: 11\r\n coords_t: 299\r\n coords_b: 300\r\n"),
            "{patched}"
        );
        assert!(
            patched.ends_with("my_parent: _nameless.w\r\n}\r\n\r\n}\r\n") && !patched.contains("\r\r"),
            "{patched}"
        );
        assert!(patch_dashboard_script(&patched, "cab_mark").is_err());
        // Two windows: which is the dashboard's isn't known.
        let two = script.replace("\r\n\r\n}", "\r\nui::window : _nameless.v {\r\n my_parent: null\r\n}\r\n}");
        assert!(patch_dashboard_script(&two, "cab_mark").is_err());
    }

    #[test]
    fn makes_a_transparent_texture_of_each_layouts_size() {
        let dds = transparent_dds(1280, 721);
        assert_eq!(&dds[..4], b"DDS ");
        assert_eq!(u32::from_le_bytes(dds[12..16].try_into().unwrap()), 721);
        assert_eq!(u32::from_le_bytes(dds[128..132].try_into().unwrap()), 91);
        assert_eq!(dds.len(), 148 + 1280 * 721 * 4);
        assert!(dds[148..].iter().all(|&byte| byte == 0));
    }

    /// The real thing: the mod built from the game installed on this PC (written to the temp folder, not the
    /// game's). `cargo test -p westplay-setup -- --ignored --nocapture builds_the_mod`
    #[test]
    #[ignore = "reads the PC's game"]
    fn builds_the_mod_from_the_installed_game() {
        let game = crate::game::find().into_iter().next().expect("the game installed");
        let files = GameFiles::install(&game.dir).unwrap();
        let (bytes, built) = build(&files, &cab_mark::Marks::default());
        let out = std::env::temp_dir().join(FILE_NAME);
        std::fs::write(&out, &bytes).unwrap();
        println!("{} bytes, no TV: {:?}: {}", bytes.len(), built.no_tv, out.display());
        assert_eq!(built.no_tv, None);
    }

    #[test]
    fn its_zip_reads_back() {
        let files = vec![("a/b.txt".to_owned(), b"hello".to_vec()), ("c.txt".to_owned(), vec![7u8; 5000])];
        let path = std::env::temp_dir().join(format!("westplay-mod-zip-{}.scs", std::process::id()));
        std::fs::write(&path, zip(&files)).unwrap();
        let archive = westplay_mapdata::archive::Archive::open(&path).unwrap();
        assert_eq!(archive.read("a/b.txt").unwrap().as_deref(), Some(&b"hello"[..]));
        assert_eq!(archive.read("c.txt").unwrap(), Some(vec![7u8; 5000]));
        drop(archive);
        let _ = std::fs::remove_file(path);
    }
}
