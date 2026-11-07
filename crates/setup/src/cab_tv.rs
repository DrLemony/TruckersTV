//! TruckersTV's cab TV (owner 2026-11-03: "a TV over the truck's right door, bought in the shop, not there by default",
//! and no mod of the player's own: the mod is mounted by the plugin, under TruckersMP too). A flat panel with a thin
//! black bezel, on a short bracket against the wall over the co-driver's door, its glass showing the `cab-tv` layout's
//! texture (`westplay_shared::CAB_TV`).
//!
//! The game has no slot over a door, so the TV is bought as a bed toy (`toybed`, its hookup's place a fixed point in
//! the cab) and its model is made where the wall over the door is, from that point: read from the truck's interior
//! model (the cab's triangles round the door, [`TRUCKS`]). Only the Iveco S-Way for now (owner 2026-11-03: "start
//! with my truck"); each truck is placed by hand from its own interior. Drawn with the camera in the cab only (an
//! `interior_model`, no `exterior_model`).

use westplay_mapdata::buildings;
use westplay_mapdata::files::GameFiles;
use westplay_shared::{SCREEN_HEIGHT, TEST_TV_MARK, TV_BEZEL_UV, TV_PANEL_VERTICES, TV_VERTEX_SIZE};

use crate::model::{self, Block, Piece};

/// Where the TV's files live in the game's file system.
const DIR: &str = "vehicle/truck/upgrade/westplay_tv";
const ICON_DIR: &str = "material/ui/accessory";
const NAME: &str = "westplay_tv";
/// The slot it's bought at: one TV at a time, of any size.
const SLOT: &str = "toybed";

/// A truck the TV goes in: where its slot's hookup point is in the interior's model.
struct Truck {
    id: &'static str,
    /// The `toybed` locator's position (its rotation none, in the trucks read).
    hookup: [f32; 3],
}

/// The TV in the shop, and where it is (metres, the interior's axes: +x to the co-driver's side, +y up, -z to the
/// windscreen).
struct Tv {
    /// Its accessory's name, the first of its unit name's parts (a token, 12 characters at the most), and its files'.
    unit: &'static str,
    file: &'static str,
    /// As the shop shows it.
    name: &'static str,
    price: u32,
    /// The wall's face behind the TV (x), and the TV's middle on it (y, z).
    wall: f32,
    middle: [f32; 2],
    /// The TV's width along the wall (z), its bezel included; its height follows (a 16:9 glass).
    width: f32,
}

/// The S-Way's interior model, and its `prod_std` interior's tied curtain, a piece of its own in it (the `prod_opt`
/// interior has a copy, 29, of another colour), where a big TV goes: taken out, for that interior alone (owner
/// 2026-11-04: "only for this cabin"). The game's update moving it: `CURTAIN_TRIANGLES` and `CURTAIN_FROM` tell, and
/// the curtain stays.
const INTERIOR: &str = "vehicle/truck/iveco_sway/interior/model/as.pmg";
const CURTAIN: usize = 37;
const CURTAIN_TRIANGLES: usize = 706;
/// How near the windscreen the curtain comes (z).
const CURTAIN_FROM: f32 = 0.1506;

/// The Iveco S-Way's interior (`/vehicle/truck/iveco_sway/interior/model/as.pmd`, every interior but the UK's, read
/// 2026-11-03): the door's window from z -0.75 to 0.05 below y 0.15, the wall over it a panel at x 1.751 to 1.77 from
/// z -0.36 to 0.18 and y 0.40 to 0.90 (the A pillar's trim nearer the cab at -0.40, the curtain's rail behind 0.20).
/// The TV starts there; past the rail, the `prod_std` interior's curtain is out ([`without_curtain`]) for a bigger one.
/// shortcut: `prod_opt`'s curtain and `top_opt`'s bunk side stay, and a big TV may go through them.
const TRUCKS: &[Truck] = &[Truck { id: "iveco.sway", hookup: [0.737_660_5, -0.482_281_54, 0.481_756_9] }];

/// The TV (owner 2026-11-07: "one TV, named TV", once the test TV): its size and place the player's, from the
/// companion. Metres, a [`Tv`]'s axes: the wall behind it (`x`), its middle (`y`, `z`), its width (its height
/// follows). It always faces -x. Sold under the first TV's accessory, so a truck that has that one keeps it. The game reads the model as it starts; with it running the plugin moves it where it's told
/// ([`TestTv::glass`], the plugin's test_tv.rs), its bezel growing with it, until the next start makes it exact.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TestTv {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub width: f32,
}

/// The TV's bounds, whatever its place (the cab's axes): all the cab, for the game to draw it wherever the player
/// has moved it to.
const TEST_REACH: [[f32; 3]; 2] = [[-2.6, -1.6, -2.0], [2.6, 2.0, 2.0]];

/// Where it starts: over the right door.
impl Default for TestTv {
    fn default() -> Self {
        Self { x: 1.751, y: 0.662, z: -0.09, width: 0.54 }
    }
}

impl TestTv {
    /// Where its glass is to be in the S-Way, for the plugin: its middle and half width, on its model's axes.
    pub fn glass(self) -> [f32; 4] {
        let (tv, hookup) = (self.tv(), TRUCKS[0].hookup);
        let [y, z] = tv.middle;
        let front = tv.wall - STAND_OFF - DEPTH - GLASS_AHEAD;
        [front - hookup[0], y - hookup[1], z - hookup[2], (tv.width - 2.0 * BEZEL) / 2.0]
    }

    /// As a TV in the shop, kept to the companion's sliders: within the cab, and no smaller than its bracket.
    fn tv(self) -> Tv {
        Tv {
            unit: "wp_tv",
            file: NAME,
            name: "TV",
            price: 1500,
            wall: self.x.clamp(-1.76, 1.76),
            middle: [self.y.clamp(-0.6, 1.0), self.z.clamp(-1.0, 1.0)],
            width: self.width.clamp(0.3, 1.6),
        }
    }
}

/// The panel's depth, its bezel's width, how far its bracket holds it off the wall, and how far in front of the
/// panel's face the glass is (no fighting over which shows). Metres.
const DEPTH: f32 = 0.03;
const BEZEL: f32 = 0.012;
const STAND_OFF: f32 = 0.016;
const GLASS_AHEAD: f32 = 0.0008;

/// The trucks the TV goes in, by their folders in the game.
pub fn trucks() -> impl Iterator<Item = &'static str> {
    TRUCKS.iter().map(|truck| truck.id)
}

/// The TV's files for the trucks `game` has (none: `Err`, nothing to make): its model per truck, its materials and
/// texture, its icon, and each truck's accessory. `screen_texture` is the `cab-tv` layout's texture object (a game
/// path), `texture_height` its texture's height, `test` the test TV's size and place.
pub fn files(
    game: &GameFiles,
    screen_texture: &str,
    texture_height: u32,
    test: TestTv,
) -> Result<Vec<(String, Vec<u8>)>, String> {
    let trucks: Vec<&Truck> = TRUCKS
        .iter()
        .filter(|truck| {
            game.dirs(&format!("def/vehicle/truck/{}/accessory", truck.id))
                .is_ok_and(|slots| slots.iter().any(|slot| slot == SLOT))
        })
        .collect();
    if trucks.is_empty() {
        return Err("no truck the TV goes in".into());
    }
    let mut files = vec![
        (format!("{DIR}/body.mat"), model::body_material_in(DIR, NAME).into_bytes()),
        (format!("{DIR}/screen.mat"), model::screen_material(screen_texture).into_bytes()),
        (format!("{DIR}/{NAME}.tobj"), crate::game_mod::tobj(&format!("/{DIR}/{NAME}.dds"))),
        (format!("{DIR}/{NAME}.dds"), model::SOURCE_DDS.to_vec()),
        (format!("{ICON_DIR}/{NAME}.mat"), crate::game_mod::material(NAME).into_bytes()),
        (format!("{ICON_DIR}/{NAME}.tobj"), crate::game_mod::tobj(&format!("/{ICON_DIR}/{NAME}.dds"))),
        (format!("{ICON_DIR}/{NAME}.dds"), model::icon()?),
    ];
    let test = test.tv();
    for truck in trucks {
        let model = model_name(truck, &test);
        files.push((format!("{DIR}/{model}.pmg"), self::model(truck, &test, texture_height)?));
        files.push((format!("{DIR}/{model}.pmd"), model::descriptor_in(DIR)?));
        files.push((definition_path(truck, &test), accessory(truck, &test).into_bytes()));
    }
    if let Some(interior) = game.read(INTERIOR).ok().flatten().and_then(without_curtain) {
        files.push((INTERIOR.to_owned(), interior));
    }
    Ok(files)
}

/// The interior's model `pmg` with the curtain out, if it's where it was read.
fn without_curtain(mut pmg: Vec<u8>) -> Option<Vec<u8>> {
    let pieces = u32::from_le_bytes(pmg.get(buildings::PIECE_COUNT_AT..)?.first_chunk().copied()?) as usize;
    let mut only = vec![false; pieces.max(CURTAIN + 1)];
    only[CURTAIN] = true;
    let curtain = buildings::shown_triangles(&pmg, Some(&only))?;
    let nearest = curtain.iter().flatten().map(|corner| corner[2]).fold(f32::MAX, f32::min);
    if curtain.len() != CURTAIN_TRIANGLES || (nearest - CURTAIN_FROM).abs() > 0.001 {
        return None;
    }
    blank_piece(&mut pmg, CURTAIN)?;
    Some(pmg)
}

/// `piece` of the model `pmg` drawn as nothing: its triangles all one corner (a piece's count left as it is, for the
/// game's reading of it).
fn blank_piece(pmg: &mut [u8], piece: usize) -> Option<()> {
    let word = |at: usize| Some(u32::from_le_bytes(pmg.get(at..at + 4)?.try_into().ok()?) as usize);
    let record = word(buildings::PIECES_AT)? + piece * buildings::PIECE_SIZE;
    let (count, at) = (word(record)?, word(record + 4 * 24)?);
    let indices = pmg.get_mut(at..at + 2 * count)?;
    let first = [*indices.first()?, *indices.get(1)?];
    indices.chunks_exact_mut(2).for_each(|index| index.copy_from_slice(&first));
    Some(())
}

/// Whether the mod built at `built` (its files, mounted) has the TV as [`files`] would make it now for `game`: the
/// same trucks, each model as placed now.
pub fn current(game: &GameFiles, built: &GameFiles, screen_texture: &str, texture_height: u32, test: TestTv) -> bool {
    let wanted = files(game, screen_texture, texture_height, test).unwrap_or_default();
    let has_all = wanted
        .iter()
        .filter(|(path, _)| path.ends_with(".pmg") || path.starts_with("def/"))
        .all(|(path, bytes)| built.read(path).ok().flatten().is_some_and(|found| found == *bytes));
    // A truck the TV went in that the game no longer has (its DLC gone): made again without it.
    let test = test.tv();
    let none_more = TRUCKS
        .iter()
        .map(|truck| definition_path(truck, &test))
        .filter(|definition| !wanted.iter().any(|(path, _)| path == definition))
        .all(|definition| built.read(&definition).ok().flatten().is_none());
    has_all && none_more
}

fn model_name(truck: &Truck, tv: &Tv) -> String {
    format!("{}_{}", tv.file, truck.id.replace('.', "_"))
}

fn definition_path(truck: &Truck, tv: &Tv) -> String {
    format!("def/vehicle/truck/{}/accessory/{SLOT}/{}.sii", truck.id, tv.file)
}

/// A truck's TV, defined as the game's dashboard sets are: drawn with the camera in the cab only.
fn accessory(truck: &Truck, tv: &Tv) -> String {
    let (id, model, Tv { unit, name, price, .. }) = (truck.id, model_name(truck, tv), tv);
    format!(
        "SiiNunit\n{{\naccessory_addon_data : {unit}.{id}.{SLOT}\n{{\n\tname: \"{name}\"\n\tprice: {price}\n\tunlock: 0\n\ticon: \"{NAME}\"\n\tpart_type: aftermarket\n\n\tinterior_model: \"/{DIR}/{model}.pmd\"\n}}\n}}\n"
    )
}

/// `tv`'s model for `truck`, from its slot's hookup point: the panel and its bracket (the source body's
/// material, black) and the glass, facing into the cab (-x), showing the texture's first SCREEN_HEIGHT rows (of
/// `texture_height`), the first at its top, the frame's left toward the windscreen (-z) as the player sees it.
fn model(truck: &Truck, tv: &Tv, texture_height: u32) -> Result<Vec<u8>, String> {
    let glass_width = tv.width - 2.0 * BEZEL;
    let height = glass_width * 9.0 / 16.0 + 2.0 * BEZEL;
    let [y, z] = tv.middle;
    let back = tv.wall - STAND_OFF;
    let front = back - DEPTH;
    // Points from the cab's axes to the model's: the hookup's rotation is none.
    let from_hookup = |p: [f32; 3]| [0, 1, 2].map(|k| p[k] - truck.hookup[k]);
    let axes = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let span = |low: [f32; 3], high: [f32; 3]| -> Block {
        (from_hookup([0, 1, 2].map(|k| (low[k] + high[k]) / 2.0)), axes, [0, 1, 2].map(|k| (high[k] - low[k]) / 2.0))
    };
    let boxes = [
        // The panel, and its bracket to the wall behind its middle.
        span([front, y - height / 2.0, z - tv.width / 2.0], [back, y + height / 2.0, z + tv.width / 2.0]),
        span([back, y - 0.08, z - 0.10], [tv.wall, y + 0.08, z + 0.10]),
    ];
    let mut look = Vec::with_capacity(12);
    look.extend([128, 128, 128, 255]);
    // The bezel's look: the source model's colour texture's black patch (its glass's border).
    look.extend(TV_BEZEL_UV[0].to_le_bytes());
    look.extend(TV_BEZEL_UV[1].to_le_bytes());
    let body = model::source_piece(model::BODY)?;
    let panel = model::blocks_looking(&body, &boxes, |p| p, &look)?;
    if panel.vertices.len() != TV_PANEL_VERTICES * TV_VERTEX_SIZE {
        return Err("the panel isn't as the plugin finds it".into());
    }
    let glass = glass(
        from_hookup([front - GLASS_AHEAD, y, z]),
        glass_width / 2.0,
        (height - 2.0 * BEZEL) / 2.0,
        SCREEN_HEIGHT as f32 / texture_height as f32,
    )?;
    model::assemble(vec![panel, glass], Some(TEST_REACH.map(from_hookup)))
}

/// The glass: a quad about `middle` facing -x, `half_width` along z and `half_height` along y, of the screen's
/// material; its texture coordinates across the frame, `frame_bottom` the frame's last row's (the texture's rows
/// past it, which tell the layouts apart, not shown); marked for the plugin to find it ([`TEST_TV_MARK`]).
fn glass(middle: [f32; 3], half_width: f32, half_height: f32, frame_bottom: f32) -> Result<Piece, String> {
    let source = model::source_piece(model::SCREEN)?;
    let mut colour = source.vertices.get(24..28).ok_or("the source glass has no vertex")?.to_vec();
    colour[0] = TEST_TV_MARK;
    // Across the face: `a` (+z) then `b` (+y), `a` x `b` the face's normal (-x), as the panel's boxes' faces are.
    let normal = [-1.0f32, 0.0, 0.0];
    let mut vertices = Vec::new();
    for (sa, sb) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        let p = [middle[0], middle[1] + half_height * sb, middle[2] + half_width * sa];
        for value in p.into_iter().chain(normal) {
            vertices.extend(value.to_le_bytes());
        }
        vertices.extend(&colour);
        // Its left (u 0) toward the windscreen, its top (v 0) up.
        vertices.extend(((sa + 1.0) / 2.0).to_le_bytes());
        vertices.extend(((1.0 - sb) / 2.0 * frame_bottom).to_le_bytes());
    }
    let order: [u16; 6] = if model::counter_clockwise()? { [0, 1, 2, 0, 2, 3] } else { [0, 2, 1, 0, 3, 2] };
    let indices = order.iter().flat_map(|corner| corner.to_le_bytes()).collect();
    let mut record = source.record;
    model::set(&mut record, 0, 6);
    model::set(&mut record, 4, 4);
    model::set(&mut record, 16, 1);
    Ok(Piece { record, vertices, indices })
}

#[cfg(test)]
mod tests {
    use super::*;
    use westplay_mapdata::buildings;

    const HEIGHT: u32 = SCREEN_HEIGHT + 14;

    /// The model's triangles, each corner back in the cab's axes.
    fn in_cab(truck: &Truck, tv: &Tv) -> Vec<[[f32; 3]; 3]> {
        let pmg = model(truck, tv, HEIGHT).unwrap();
        buildings::model_triangles(&pmg)
            .unwrap()
            .into_iter()
            .map(|triangle| triangle.map(|p| [0, 1, 2].map(|k| p[k] + truck.hookup[k])))
            .collect()
    }

    #[test]
    fn the_sways_tv_hangs_on_the_wall_over_the_right_door() {
        let truck = &TRUCKS[0];
        let tv = &TestTv::default().tv();
        let points: Vec<[f32; 3]> = in_cab(truck, tv).into_iter().flatten().collect();
        let least = |k: usize| points.iter().map(|p| p[k]).fold(f32::MAX, f32::min);
        let most = |k: usize| points.iter().map(|p| p[k]).fold(f32::MIN, f32::max);
        // Against the wall, never through it; within the panel over the door, clear of the pillar and the rail.
        assert!((most(0) - tv.wall).abs() < 1e-4, "{}", most(0));
        assert!(least(0) > 1.69);
        assert!(least(2) >= -0.36 - 1e-4 && most(2) <= 0.18 + 1e-4, "{} {}", least(2), most(2));
        assert!(least(1) > 0.45 && most(1) < 0.88, "{} {}", least(1), most(1));
    }

    #[test]
    fn the_test_tv_goes_where_it_is_told_keeping_its_shape() {
        let truck = &TRUCKS[0];
        let size = |test: TestTv| {
            let points: Vec<[f32; 3]> = in_cab(truck, &test.tv()).into_iter().flatten().collect();
            let least = |k: usize| points.iter().map(|p| p[k]).fold(f32::MAX, f32::min);
            let most = |k: usize| points.iter().map(|p| p[k]).fold(f32::MIN, f32::max);
            (most(0), (least(1) + most(1)) / 2.0, (least(2) + most(2)) / 2.0, most(2) - least(2), most(1) - least(1))
        };
        let (back, y, z, width, height) = size(TestTv { x: 1.2, y: 0.3, z: 0.1, width: 1.0 });
        assert!((back - 1.2).abs() < 1e-4 && (y - 0.3).abs() < 1e-4 && (z - 0.1).abs() < 1e-4);
        assert!((width - 1.0).abs() < 1e-4);
        // Its glass 16:9 at any size.
        assert!(((width - 2.0 * BEZEL) / (height - 2.0 * BEZEL) - 16.0 / 9.0).abs() < 1e-3);
        // Asked for too much, the most there is.
        let tv = TestTv { x: 9.0, y: -9.0, z: 9.0, width: 0.0 }.tv();
        assert_eq!((tv.wall, tv.middle, tv.width), (1.76, [-0.6, 1.0], 0.3));
        assert!(tv.unit.len() <= 12);
    }

    #[test]
    fn the_plugin_is_told_where_the_model_has_the_test_tv_s_glass() {
        let truck = &TRUCKS[0];
        let test = TestTv { x: 1.2, y: 0.3, z: 0.1, width: 1.0 };
        let pmg = model(truck, &test.tv(), HEIGHT).unwrap();
        let triangles = buildings::model_triangles(&pmg).unwrap();
        let corners: Vec<[f32; 3]> = triangles[triangles.len() - 2..].iter().flatten().copied().collect();
        let least = |k: usize| corners.iter().map(|p| p[k]).fold(f32::MAX, f32::min);
        let most = |k: usize| corners.iter().map(|p| p[k]).fold(f32::MIN, f32::max);
        let [x, y, z, half] = test.glass();
        assert!((least(0) - x).abs() < 1e-5 && (most(0) - x).abs() < 1e-5);
        assert!(((least(1) + most(1)) / 2.0 - y).abs() < 1e-5 && ((least(2) + most(2)) / 2.0 - z).abs() < 1e-5);
        assert!(((most(2) - least(2)) / 2.0 - half).abs() < 1e-5);
    }

    #[test]
    fn a_blanked_piece_draws_nothing_and_the_rest_stays() {
        let mut pmg = model(&TRUCKS[0], &TestTv::default().tv(), HEIGHT).unwrap();
        let before = buildings::model_triangles(&pmg).unwrap();
        blank_piece(&mut pmg, 1).unwrap();
        let after = buildings::model_triangles(&pmg).unwrap();
        assert_eq!(after.len(), before.len());
        let glass = before.len() - 2;
        assert_eq!(after[..glass], before[..glass]);
        assert!(after[glass..].iter().all(|[a, b, c]| a == b && b == c));
    }

    #[test]
    #[ignore = "reads the installed game"]
    fn takes_only_the_curtain_out_of_the_game_s_interior() {
        let game = crate::game::find().into_iter().next().expect("the game installed");
        let files = GameFiles::install(&game.dir).unwrap();
        let pmg = files.read(INTERIOR).unwrap().unwrap();
        let before = buildings::model_triangles(&pmg).unwrap();
        let after = buildings::model_triangles(&without_curtain(pmg).unwrap()).unwrap();
        let changed = before.iter().zip(&after).filter(|(old, new)| old != new).count();
        assert_eq!((before.len(), changed), (after.len(), CURTAIN_TRIANGLES));
    }

    #[test]
    fn the_tv_is_sold_as_tv_under_the_first_tvs_accessory() {
        let tv = accessory(&TRUCKS[0], &TestTv::default().tv());
        assert!(tv.contains("wp_tv.iveco.sway.toybed") && tv.contains("name: \"TV\"") && tv.contains("price: 1500"));
        assert!(tv.contains("/westplay_tv_iveco_sway.pmd"));
    }

    #[test]
    fn its_glass_faces_into_the_cab_showing_the_frame_upright() {
        let pmg = model(&TRUCKS[0], &TestTv::default().tv(), HEIGHT).unwrap();
        // Two pieces: the panel (8 faces' worth of boxes) and the glass (one quad).
        let triangles = buildings::model_triangles(&pmg).unwrap();
        assert_eq!(triangles.len(), 2 * 6 * 2 + 2);
        let glass = &triangles[triangles.len() - 2..];
        // Wound so its face is toward -x, as the panel's front is.
        let ccw = model::counter_clockwise().unwrap();
        for [a, b, c] in glass {
            let n = model::cross([0, 1, 2].map(|k| b[k] - a[k]), [0, 1, 2].map(|k| c[k] - a[k]));
            assert!(if ccw { n[0] < 0.0 } else { n[0] > 0.0 }, "{n:?}");
        }
    }

    #[test]
    fn it_goes_only_in_the_trucks_the_game_has() {
        let root = std::env::temp_dir().join(format!("westplay-cab-tv-{}", std::process::id()));
        let accessory = root.join("def/vehicle/truck/iveco.sway/accessory/toybed");
        std::fs::create_dir_all(&accessory).unwrap();
        std::fs::write(accessory.join("x.sii"), "").unwrap();
        let game = GameFiles::new(vec![westplay_mapdata::files::Mount::Folder(root.clone())]);
        let test = TestTv::default();
        let made = files(&game, "/s.tobj", HEIGHT, test).unwrap();
        let defs: Vec<&String> = made.iter().map(|(path, _)| path).filter(|path| path.starts_with("def/")).collect();
        assert_eq!(defs, ["def/vehicle/truck/iveco.sway/accessory/toybed/westplay_tv.sii"]);
        let empty = GameFiles::new(Vec::new());
        assert!(files(&empty, "/s.tobj", HEIGHT, test).is_err());
        // Built with it, current; for a game without the truck, not (the TV to be taken out).
        let built_dir = root.join("built");
        for (path, bytes) in &made {
            let at = built_dir.join(path);
            std::fs::create_dir_all(at.parent().unwrap()).unwrap();
            std::fs::write(at, bytes).unwrap();
        }
        let built = GameFiles::new(vec![westplay_mapdata::files::Mount::Folder(built_dir)]);
        assert!(current(&game, &built, "/s.tobj", HEIGHT, test));
        assert!(!current(&empty, &built, "/s.tobj", HEIGHT, test));
        assert!(current(&empty, &empty, "/s.tobj", HEIGHT, test));
        // The TV moved: made again.
        assert!(!current(&game, &built, "/s.tobj", HEIGHT, TestTv { y: 0.1, ..test }));
        let _ = std::fs::remove_dir_all(&root);
    }
}
