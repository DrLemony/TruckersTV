//! The TV's model parts: the source model (`assets/tablet`, Art_bor's, CC BY 4.0, an iPad's body and glass, whose
//! pieces `cab_tv` builds the TV from), its materials, and the game's model format.

use westplay_mapdata::token::string_to_token;

/// The source model and its body's colour texture; Kedi739's descriptor, the pattern the TV's is laid out as.
const SOURCE_PMG: &[u8] = include_bytes!("../../../assets/tablet/ipad.pmg");

const SOURCE_PMD: &[u8] = include_bytes!("../../../assets/tablet/Kedi739.pmd");

pub(crate) const SOURCE_DDS: &[u8] = include_bytes!("../../../assets/tablet/ipad.dds");

/// The shop's icon: TruckersTV's own (256 x 256).
const ICON_PNG: &[u8] = include_bytes!("../../../app/icons/128x128@2x.png");

/// The source model's pieces: its body and its glass.
pub(crate) const BODY: usize = 0;

pub(crate) const SCREEN: usize = 1;


const PIECES: usize = 1;


const BOUNDS: usize = 8;


const LOCATORS_AT: usize = 20;

const PIECES_AT: usize = 21;

const STRINGS_AT: usize = 22;

const VERTICES_AT: usize = 24;

const INDICES_AT: usize = 26;

const HEADER_SIZE: usize = 0x70;

const PART_SIZE: usize = 24;

const PIECE_SIZE: usize = 100;

const PIECE_INDICES: usize = 0;

const PIECE_VERTICES: usize = 1;

const PIECE_MATERIAL: usize = 4;

const PIECE_BOUNDS: usize = 5;

const PIECE_STRIDE: usize = 15;

const PIECE_POSITIONS: usize = 16;

const PIECE_NORMALS: usize = 17;

const PIECE_UVS: usize = 18;

const PIECE_COLOURS: usize = 19;

const PIECE_INDEX_AT: usize = 24;

/// A body's material in `dir`, on its colour texture (`name`: [`SOURCE_DDS`]).
pub(crate) fn body_material_in(dir: &str, name: &str) -> String {
    format!(
        "effect : \"eut2.dif.spec.rfx\" {{\n\tadditional_ambient : 0.000000\n\tdiffuse : {{ 1.000000 , 1.000000 , 1.000000 }}\n\treflection : 0.000000\n\tshininess : 80.000000\n\tspecular : {{ 1.000000 , 1.000000 , 1.000000 }}\n\ttexture : \"texture_base\" {{\n\t\tsource : \"/{dir}/{name}.tobj\"\n\t\tsampler : default\n\t}}\n}}\n"
    )
}

/// Its screen: the game's own screen material (`/automat/d8/d8187d0c1f48c659.mat`, the Scania's too: lit
/// at night as a screen is), on TruckersTV's texture.
pub(crate) fn screen_material(texture: &str) -> String {
    format!(
        "effect : \"eut2.dif.lum.rfx\" {{\n\tadditional_ambient : 0.000000\n\tdiffuse : {{ 0.132868 , 0.132868 , 0.132868 }}\n\tluminance_night : 10.000000\n\tluminance_output : 2000.000000\n\treflection : 0.000000\n\tshininess : 40.000000\n\tspecular : {{ 0.010023 , 0.010023 , 0.010023 }}\n\ttexture : \"texture_base\" {{\n\t\tsource : \"{texture}\"\n\t\tsampler : default\n\t}}\n}}\n"
    )
}

fn word(data: &[u8], index: usize) -> Result<u32, String> {
    data.get(index * 4..index * 4 + 4)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
        .ok_or_else(|| format!("no word {index}"))
}

fn float(data: &[u8], at: usize) -> Result<f32, String> {
    data.get(at..at + 4)
        .map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()))
        .ok_or_else(|| format!("no float at {at}"))
}

pub(crate) fn set(data: &mut [u8], at: usize, value: u32) {
    data[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

fn set_float(data: &mut [u8], at: usize, value: f32) {
    data[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

/// A piece of the source model: its record, and its vertices' and indices' bytes.
pub(crate) struct Piece {
    pub(crate) record: Vec<u8>,
    pub(crate) vertices: Vec<u8>,
    pub(crate) indices: Vec<u8>,
}

pub(crate) fn source_piece(index: usize) -> Result<Piece, String> {
    let table = word(SOURCE_PMG, PIECES_AT)? as usize;
    let record = SOURCE_PMG.get(table + index * PIECE_SIZE..table + (index + 1) * PIECE_SIZE).ok_or("no piece")?;
    let field = |field: usize| word(record, field).map(|value| value as usize);
    let (count, stride, positions) = (field(PIECE_VERTICES)?, field(PIECE_STRIDE)?, field(PIECE_POSITIONS)?);
    // Its vertices interleaved: position, normal (+12), colour (+24), texture coordinate (+28).
    let interleaved = (field(PIECE_NORMALS)?, field(PIECE_COLOURS)?, field(PIECE_UVS)?)
        == (positions + 12, positions + 24, positions + 28)
        && stride == 36
        && (20..24).all(|field| word(record, field) == Ok(u32::MAX));
    if !interleaved {
        return Err(format!("piece {index} isn't laid out as read"));
    }
    let vertices = SOURCE_PMG.get(positions..positions + count * stride).ok_or("vertices past the end")?;
    let at = field(PIECE_INDEX_AT)?;
    let indices = SOURCE_PMG.get(at..at + field(PIECE_INDICES)? * 2).ok_or("indices past the end")?;
    Ok(Piece { record: record.to_vec(), vertices: vertices.to_vec(), indices: indices.to_vec() })
}

/// The box of a piece's record: its least and greatest corners.
fn piece_box(piece: &Piece) -> Result<([f32; 3], [f32; 3]), String> {
    let at = |i: usize| float(&piece.record, (PIECE_BOUNDS + 4 + i) * 4);
    Ok(([at(0)?, at(1)?, at(2)?], [at(3)?, at(4)?, at(5)?]))
}

/// A piece's bounds (a box round its vertices, or `reach`, and a sphere round that) from its vertices as they are.
fn set_bounds(piece: &mut Piece, reach: Option<[[f32; 3]; 2]>) -> Result<(), String> {
    let (mut least, mut most) = ([f32::MAX; 3], [f32::MIN; 3]);
    for vertex in piece.vertices.chunks_exact(36) {
        for axis in 0..3 {
            let value = float(vertex, axis * 4)?;
            least[axis] = least[axis].min(value);
            most[axis] = most[axis].max(value);
        }
    }
    if let Some([low, high]) = reach {
        (least, most) = (low, high);
    }
    let centre = [0, 1, 2].map(|axis| (least[axis] + most[axis]) / 2.0);
    let radius = (0..3).map(|axis| (most[axis] - centre[axis]).powi(2)).sum::<f32>().sqrt();
    for (i, value) in centre.into_iter().chain([radius]).chain(least).chain(most).enumerate() {
        set_float(&mut piece.record, (PIECE_BOUNDS + i) * 4, value);
    }
    Ok(())
}

pub(crate) fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// A box: its centre, three axes (each the cross of the two before it), half its size along each.
pub(crate) type Block = ([f32; 3], [[f32; 3]; 3], [f32; 3]);

/// Which way the source's triangles wind about their vertices' normals: counter-clockwise or not.
pub(crate) fn counter_clockwise() -> Result<bool, String> {
    let source = source_piece(BODY)?;
    let index = |i: usize| u16::from_le_bytes([source.indices[i * 2], source.indices[i * 2 + 1]]) as usize * 36;
    let corner = |i: usize| -> Result<[f32; 3], String> {
        let at = index(i);
        Ok([float(&source.vertices, at)?, float(&source.vertices, at + 4)?, float(&source.vertices, at + 8)?])
    };
    let (a, b, c) = (corner(0)?, corner(1)?, corner(2)?);
    let facing = cross([0, 1, 2].map(|k| b[k] - a[k]), [0, 1, 2].map(|k| c[k] - a[k]));
    let normal = [
        float(&source.vertices, index(0) + 12)?,
        float(&source.vertices, index(0) + 16)?,
        float(&source.vertices, index(0) + 20)?,
    ];
    Ok((0..3).map(|k| facing[k] * normal[k]).sum::<f32>() > 0.0)
}

/// [`blocks`], each vertex's colour and texture coordinate `look` (12 bytes: a patch of [`SOURCE_DDS`]).
pub(crate) fn blocks_looking(
    body: &Piece,
    boxes: &[Block],
    turn: impl Fn([f32; 3]) -> [f32; 3],
    look: &[u8],
) -> Result<Piece, String> {
    let counter_clockwise = counter_clockwise()?;
    let (mut vertices, mut indices) = (Vec::new(), Vec::new());
    for &(centre, axes, half) in boxes {
        for (k, sign) in [0, 1, 2].into_iter().flat_map(|k| [(k, 1.0f32), (k, -1.0)]) {
            let n = axes[k].map(|v| v * sign);
            // Two axes across the face, `a` x `b` along its normal.
            let (a, b, ha, hb) = if sign > 0.0 {
                (axes[(k + 1) % 3], axes[(k + 2) % 3], half[(k + 1) % 3], half[(k + 2) % 3])
            } else {
                (axes[(k + 2) % 3], axes[(k + 1) % 3], half[(k + 2) % 3], half[(k + 1) % 3])
            };
            let first = (vertices.len() / 36) as u16;
            for (sa, sb) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                let p: [f32; 3] = [0, 1, 2].map(|i| centre[i] + n[i] * half[k] + a[i] * ha * sa + b[i] * hb * sb);
                for value in turn(p).into_iter().chain(turn(n)) {
                    vertices.extend(value.to_le_bytes());
                }
                vertices.extend(look);
            }
            let order: [u16; 6] = if counter_clockwise { [0, 1, 2, 0, 2, 3] } else { [0, 2, 1, 0, 3, 2] };
            for corner in order {
                indices.extend((first + corner).to_le_bytes());
            }
        }
    }
    let mut record = body.record.clone();
    set(&mut record, PIECE_INDICES * 4, (indices.len() / 2) as u32);
    set(&mut record, PIECE_VERTICES * 4, (vertices.len() / 36) as u32);
    set(&mut record, PIECE_MATERIAL * 4, 0);
    Ok(Piece { record, vertices, indices })
}

/// A model of `pieces` (each a record, its vertices and indices, its material set): the source's header (its parts'
/// table where it was), the part, the pieces' records, their vertices, their indices, each piece's bounds and the
/// model's from its vertices as they are.
/// With a box `reach` (least and greatest corners), that's the bounds, whatever the vertices: a model moved once it's
/// loaded, for the game not to leave it out for where it was.
pub(crate) fn assemble(mut pieces: Vec<Piece>, reach: Option<[[f32; 3]; 2]>) -> Result<Vec<u8>, String> {
    for piece in &mut pieces {
        set_bounds(piece, reach)?;
    }
    // The header (its parts' table where it was), the part, the pieces' records, their vertices, their indices.
    let table = HEADER_SIZE + PART_SIZE;
    let vertices_at = table + pieces.len() * PIECE_SIZE;
    let vertices_size: usize = pieces.iter().map(|piece| piece.vertices.len()).sum();
    let indices_at = vertices_at + vertices_size;
    let indices_size: usize = pieces.iter().map(|piece| piece.indices.len()).sum();
    let mut out = SOURCE_PMG[..table].to_vec();
    set(&mut out, PIECES * 4, pieces.len() as u32);
    set(&mut out, HEADER_SIZE + 8, pieces.len() as u32);
    for index in [LOCATORS_AT, PIECES_AT] {
        set(&mut out, index * 4, table as u32);
    }
    for (index, value) in [
        (STRINGS_AT, vertices_at),
        (VERTICES_AT, vertices_at),
        (VERTICES_AT + 1, vertices_size),
        (INDICES_AT, indices_at),
        (INDICES_AT + 1, indices_size),
    ] {
        set(&mut out, index * 4, value as u32);
    }
    // The model's bounds: the pieces' boxes', and a sphere round them.
    let (mut least, mut most) = ([f32::MAX; 3], [f32::MIN; 3]);
    for piece in &pieces {
        let (low, high) = piece_box(piece)?;
        for axis in 0..3 {
            least[axis] = least[axis].min(low[axis]);
            most[axis] = most[axis].max(high[axis]);
        }
    }
    let centre = [0, 1, 2].map(|axis| (least[axis] + most[axis]) / 2.0);
    let radius = (0..3).map(|axis| (most[axis] - centre[axis]).powi(2)).sum::<f32>().sqrt();
    for (i, value) in centre.into_iter().chain([radius]).chain(least).chain(most).enumerate() {
        set_float(&mut out, (BOUNDS + i) * 4, value);
    }
    let (mut vertex_at, mut index_at) = (vertices_at, indices_at);
    for piece in &mut pieces {
        for (field, offset) in [(PIECE_POSITIONS, 0), (PIECE_NORMALS, 12), (PIECE_COLOURS, 24), (PIECE_UVS, 28)] {
            set(&mut piece.record, field * 4, (vertex_at + offset) as u32);
        }
        set(&mut piece.record, PIECE_INDEX_AT * 4, index_at as u32);
        vertex_at += piece.vertices.len();
        index_at += piece.indices.len();
        out.extend(&piece.record);
    }
    for piece in &pieces {
        out.extend(&piece.vertices);
    }
    for piece in &pieces {
        out.extend(&piece.indices);
    }
    Ok(out)
}

/// [`descriptor`], its materials `body.mat` and `screen.mat` in `dir`.
pub(crate) fn descriptor_in(dir: &str) -> Result<Vec<u8>, String> {
    // Counts: version, materials, looks, pieces, variants, parts, attributes, a variant's values' bytes; then the
    // strings' bytes and where the looks, variants, parts' ranges, variants' values, attributes, materials and
    // strings are.
    let header: Vec<u32> = (0..16).map(|index| word(SOURCE_PMD, index)).collect::<Result<_, _>>()?;
    if header[..8] != [4, 4, 1, 0, 1, 1, 1, 4] || header[14] != 0x6c {
        return Err("the source descriptor isn't laid out as read".into());
    }
    let default = string_to_token("default").ok_or("no token")?;
    let materials = [format!("/{dir}/body.mat"), format!("/{dir}/screen.mat")];
    let strings_at = 0x6c + 4 * materials.len();
    let mut out = SOURCE_PMD[..0x6c].to_vec();
    for at in [header[9] as usize, header[10] as usize] {
        out[at..at + 8].copy_from_slice(&default.to_le_bytes());
    }
    let mut strings = Vec::new();
    for material in &materials {
        out.extend(((strings_at + strings.len()) as u32).to_le_bytes());
        strings.extend(material.as_bytes());
        strings.push(0);
    }
    set(&mut out, 4, materials.len() as u32);
    set(&mut out, 8 * 4, strings.len() as u32);
    set(&mut out, 15 * 4, strings_at as u32);
    out.extend(strings);
    Ok(out)
}

/// The shop's icon: TruckersTV's, as a texture the mod's screens are made of (`game_mod::screen_dds`).
pub(crate) fn icon() -> Result<Vec<u8>, String> {
    let mut reader =
        png::Decoder::new(std::io::Cursor::new(ICON_PNG)).read_info().map_err(|error| error.to_string())?;
    let mut pixels = vec![0; reader.output_buffer_size().ok_or("no size")?];
    let info = reader.next_frame(&mut pixels).map_err(|error| error.to_string())?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return Err("the icon isn't 8-bit RGBA".into());
    }
    let mut bgra = pixels[..info.buffer_size()].to_vec();
    for pixel in bgra.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    Ok(crate::game_mod::screen_dds(info.width, info.height, &bgra))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn its_descriptor_names_its_two_materials_as_a_toys_look() {
        let pmd = descriptor_in("/a/b").unwrap();
        assert_eq!(word(&pmd, 1).unwrap(), 2);
        let default = string_to_token("default").unwrap().to_le_bytes();
        assert_eq!(&pmd[0x40..0x48], default);
        assert_eq!(&pmd[0x48..0x50], default);
        assert_eq!(pmd.len(), 0x6c + 8 + word(&pmd, 8).unwrap() as usize);
    }
}
