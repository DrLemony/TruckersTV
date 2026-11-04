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
use westplay_shared::SCREEN_HEIGHT;

use crate::model::{self, Block, Piece};

/// Where the TV's files live in the game's file system.
const DIR: &str = "vehicle/truck/upgrade/westplay_tv";
const ICON_DIR: &str = "material/ui/accessory";
const NAME: &str = "westplay_tv";
/// The slot it's bought at: one TV at a time, of any size.
const SLOT: &str = "toybed";

/// A truck the TV goes in: where its slot's hookup point is in the interior's model, and the TVs sold for it.
struct Truck {
    id: &'static str,
    /// The `toybed` locator's position (its rotation none, in the trucks read).
    hookup: [f32; 3],
    tvs: &'static [Tv],
}

/// A TV in the shop, and where it is on the wall over the right door (metres, the interior's axes: +x to the
/// co-driver's side, +y up, -z to the windscreen).
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

/// The wall's nearest face behind the XL and the XXL (read 2026-11-04: at z 0.48, y 0.51).
const WALL_XL: f32 = 1.741;

/// The S-Way's interior model, and its `prod_std` interior's tied curtain, a piece of its own in it (the `prod_opt`
/// interior has a copy, 29, of another colour), where the XXL goes: taken out, for that interior alone (owner
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
/// The XL goes on past that rail, as the player marked (owner 2026-11-04), from the small one's top left corner down
/// to y 0.36 and back to z 0.44, where the `top_opt` interior's bunk side starts. The XXL fills what the player marked
/// in the `prod_std` interior, the curtain out ([`without_curtain`]): to z 0.65, its bunk's side, and down to y 0.27.
/// shortcut: the shop sells both in every interior; `prod_opt`'s curtain comes through them and `top_opt`'s bunk side
/// through the XXL, until each is asked for.
const TRUCKS: &[Truck] = &[Truck {
    id: "iveco.sway",
    hookup: [0.737_660_5, -0.482_281_54, 0.481_756_9],
    tvs: &[
        Tv {
            unit: "wp_tv",
            file: NAME,
            name: "TruckersTV TV",
            price: 1500,
            wall: 1.751,
            middle: [0.662, -0.09],
            width: 0.54,
        },
        Tv {
            unit: "wp_tv_xl",
            file: "westplay_tv_xl",
            name: "TruckersTV TV XL",
            price: 3000,
            wall: WALL_XL,
            middle: [0.5888, 0.04],
            width: 0.80,
        },
        Tv {
            unit: "wp_tv_xxl",
            file: "westplay_tv_xxl",
            name: "TruckersTV TV XXL",
            price: 4500,
            wall: WALL_XL,
            middle: [0.5578, 0.14],
            width: 1.02,
        },
    ],
}];

/// The panel's depth, its bezel's width, how far its bracket holds it off the wall, and how far in front of the
/// panel's face the glass is (no fighting over which shows). Metres.
const DEPTH: f32 = 0.03;
const BEZEL: f32 = 0.012;
const STAND_OFF: f32 = 0.016;
const GLASS_AHEAD: f32 = 0.0008;
/// The bezel's look: the source model's colour texture's black patch (its glass's border).
const BLACK: [f32; 2] = [0.125, 0.375];

/// The trucks the TV goes in, by their folders in the game.
pub fn trucks() -> impl Iterator<Item = &'static str> {
    TRUCKS.iter().map(|truck| truck.id)
}

/// The TV's files for the trucks `game` has (none: `Err`, nothing to make): its model per truck, its materials and
/// texture, its icon, and each truck's accessory. `screen_texture` is the `cab-tv` layout's texture object (a game
/// path), `texture_height` its texture's height.
pub fn files(game: &GameFiles, screen_texture: &str, texture_height: u32) -> Result<Vec<(String, Vec<u8>)>, String> {
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
    for truck in trucks {
        for tv in truck.tvs {
            let model = model_name(truck, tv);
            files.push((format!("{DIR}/{model}.pmg"), self::model(truck, tv, texture_height)?));
            files.push((format!("{DIR}/{model}.pmd"), model::descriptor_in(DIR)?));
            files.push((definition_path(truck, tv), accessory(truck, tv).into_bytes()));
        }
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
pub fn current(game: &GameFiles, built: &GameFiles, screen_texture: &str, texture_height: u32) -> bool {
    let wanted = files(game, screen_texture, texture_height).unwrap_or_default();
    let has_all = wanted
        .iter()
        .filter(|(path, _)| path.ends_with(".pmg") || path.starts_with("def/"))
        .all(|(path, bytes)| built.read(path).ok().flatten().is_some_and(|found| found == *bytes));
    // A truck the TV went in that the game no longer has (its DLC gone): made again without it.
    let none_more = TRUCKS
        .iter()
        .flat_map(|truck| truck.tvs.iter().map(|tv| definition_path(truck, tv)))
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
    look.extend(BLACK[0].to_le_bytes());
    look.extend(BLACK[1].to_le_bytes());
    let body = model::source_piece(model::BODY)?;
    let panel = model::blocks_looking(&body, &boxes, |p| p, &look)?;
    let glass = glass(
        from_hookup([front - GLASS_AHEAD, y, z]),
        glass_width / 2.0,
        (height - 2.0 * BEZEL) / 2.0,
        SCREEN_HEIGHT as f32 / texture_height as f32,
    )?;
    model::assemble(vec![panel, glass])
}

/// The glass: a quad about `middle` facing -x, `half_width` along z and `half_height` along y, of the screen's
/// material; its texture coordinates across the frame, `frame_bottom` the frame's last row's (the texture's rows
/// past it, which tell the layouts apart, not shown).
fn glass(middle: [f32; 3], half_width: f32, half_height: f32, frame_bottom: f32) -> Result<Piece, String> {
    let source = model::source_piece(model::SCREEN)?;
    let colour = source.vertices.get(24..28).ok_or("the source glass has no vertex")?.to_vec();
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
        let tv = &truck.tvs[0];
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
    fn a_blanked_piece_draws_nothing_and_the_rest_stays() {
        let mut pmg = model(&TRUCKS[0], &TRUCKS[0].tvs[0], HEIGHT).unwrap();
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
    fn the_xxl_fills_the_wall_the_curtain_left() {
        let truck = &TRUCKS[0];
        let points: Vec<[f32; 3]> = in_cab(truck, &truck.tvs[2]).into_iter().flatten().collect();
        let least = |k: usize| points.iter().map(|p| p[k]).fold(f32::MAX, f32::min);
        let most = |k: usize| points.iter().map(|p| p[k]).fold(f32::MIN, f32::max);
        // Clear of the pillar's trim, the bunk's side, the roof's curve and the door's frame.
        assert!(least(2) >= -0.3701 && most(2) <= 0.6501, "{} {}", least(2), most(2));
        assert!(least(1) >= 0.26 && most(1) <= 0.851, "{} {}", least(1), most(1));
        assert!((most(0) - WALL_XL).abs() < 1e-4);
    }

    #[test]
    fn the_xl_grows_from_the_small_ones_top_left_corner() {
        let truck = &TRUCKS[0];
        let corners = |tv| {
            let points: Vec<[f32; 3]> = in_cab(truck, tv).into_iter().flatten().collect();
            let least = |k: usize| points.iter().map(|p| p[k]).fold(f32::MAX, f32::min);
            let most = |k: usize| points.iter().map(|p| p[k]).fold(f32::MIN, f32::max);
            (least(2), most(2), least(1), most(1), most(0))
        };
        let (left, _, _, top, _) = corners(&truck.tvs[0]);
        let (xl_left, xl_right, xl_bottom, xl_top, xl_back) = corners(&truck.tvs[1]);
        assert!((xl_left - left).abs() < 1e-3 && (xl_top - top).abs() < 2e-3, "{xl_left} {xl_top}");
        // Short of the bunk's side and the panel's foot, on the wall's nearest face there.
        assert!(xl_right <= 0.44 + 1e-3 && xl_bottom >= 0.355, "{xl_right} {xl_bottom}");
        assert!((xl_back - WALL_XL).abs() < 1e-4);
    }

    #[test]
    fn each_tv_is_its_own_accessory_in_the_one_slot() {
        let truck = &TRUCKS[0];
        let [small, xl] = [&truck.tvs[0], &truck.tvs[1]].map(|tv| accessory(truck, tv));
        assert!(small.contains("wp_tv.iveco.sway.toybed") && small.contains("price: 1500"));
        assert!(xl.contains("wp_tv_xl.iveco.sway.toybed") && xl.contains("name: \"TruckersTV TV XL\""));
        assert!(xl.contains("/westplay_tv_xl_iveco_sway.pmd"));
        assert!(truck.tvs.iter().all(|tv| tv.unit.len() <= 12));
    }

    #[test]
    fn its_glass_faces_into_the_cab_showing_the_frame_upright() {
        let pmg = model(&TRUCKS[0], &TRUCKS[0].tvs[0], HEIGHT).unwrap();
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
        let made = files(&game, "/s.tobj", HEIGHT).unwrap();
        let defs: Vec<&String> = made.iter().map(|(path, _)| path).filter(|path| path.starts_with("def/")).collect();
        assert_eq!(
            defs,
            [
                "def/vehicle/truck/iveco.sway/accessory/toybed/westplay_tv.sii",
                "def/vehicle/truck/iveco.sway/accessory/toybed/westplay_tv_xl.sii",
                "def/vehicle/truck/iveco.sway/accessory/toybed/westplay_tv_xxl.sii"
            ]
        );
        let empty = GameFiles::new(Vec::new());
        assert!(files(&empty, "/s.tobj", HEIGHT).is_err());
        // Built with it, current; for a game without the truck, not (the TV to be taken out).
        let built_dir = root.join("built");
        for (path, bytes) in &made {
            let at = built_dir.join(path);
            std::fs::create_dir_all(at.parent().unwrap()).unwrap();
            std::fs::write(at, bytes).unwrap();
        }
        let built = GameFiles::new(vec![westplay_mapdata::files::Mount::Folder(built_dir)]);
        assert!(current(&game, &built, "/s.tobj", HEIGHT));
        assert!(!current(&empty, &built, "/s.tobj", HEIGHT));
        assert!(current(&empty, &empty, "/s.tobj", HEIGHT));
        let _ = std::fs::remove_dir_all(&root);
    }
}
