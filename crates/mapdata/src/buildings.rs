//! What's read of the game's models (`.pmg`): the places things are fitted in them, and their triangles.

/// A `.pmg`'s first word: the format's version (0x15), then "gmP".
const PMG_MAGIC: u32 = 0x506d_6715;
/// Where a `.pmg` says how many pieces it has (its meshes, one a material) and where their records are.
pub const PIECE_COUNT_AT: usize = 4;
pub const PIECES_AT: usize = 84;
/// A piece's record: its index count (word 0), its vertex count (1), its vertices' stride (15), where its
/// vertices' positions start (16: three floats each), where its indices start (24: u16, a triangle list).
pub const PIECE_SIZE: usize = 100;
fn word(data: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(data.get(at..at.checked_add(4)?)?.try_into().ok()?))
}

/// Where a `.pmg` says how many parts it has (its named groups of pieces) and where their records are: each
/// 24 bytes, its name (a token), its piece count and its first piece, its locators' count and first.
const PART_COUNT_AT: usize = 8;
const PARTS_AT: usize = 76;
const PART_SIZE: usize = 24;

/// A model's locator: a named place in it where something else is fitted (a truck interior's `int_rearwall`,
/// where the rear wall the player chose goes).
#[derive(Clone, Debug, PartialEq)]
pub struct Locator {
    pub name: String,
    /// Metres, the model's own axes (x right, y up, z back).
    pub position: [f32; 3],
    /// How what's fitted there is turned: a quaternion, w first.
    pub rotation: [f32; 4],
}

/// Where a `.pmg`'s locator records are (44 bytes each: a name, then the position at +8, the scale at +20 and the
/// rotation at +24); each part's record says which of them are its.
const LOCATORS_AT: usize = 80;
const LOCATOR_SIZE: usize = 44;

/// Every part's locators of a `.pmg`, in its order; `None` if it isn't a `.pmg` this reads.
pub fn model_locators(pmg: &[u8]) -> Option<Vec<Locator>> {
    let word = |at: usize| word(pmg, at);
    if word(0)? != PMG_MAGIC {
        return None;
    }
    let float = |at: usize| word(at).map(f32::from_bits);
    let (parts, parts_at, locators_at) =
        (word(PART_COUNT_AT)? as usize, word(PARTS_AT)? as usize, word(LOCATORS_AT)? as usize);
    let mut locators = Vec::new();
    for part in 0..parts {
        let record = parts_at.checked_add(part.checked_mul(PART_SIZE)?)?;
        let (count, first) = (word(record + 16)? as usize, word(record + 20)? as usize);
        for index in first..first.checked_add(count)? {
            let at = locators_at.checked_add(index.checked_mul(LOCATOR_SIZE)?)?;
            let name = crate::token::token_to_string(u64::from(word(at)?) | u64::from(word(at + 4)?) << 32)?;
            let position = [float(at + 8)?, float(at + 12)?, float(at + 16)?];
            let rotation = [float(at + 24)?, float(at + 28)?, float(at + 32)?, float(at + 36)?];
            if position.iter().chain(&rotation).any(|v| !v.is_finite()) {
                return None;
            }
            locators.push(Locator { name, position, rotation });
        }
    }
    Some(locators)
}

/// A model's triangles (each three corners: x, y, z, metres, its own axes); `None` if its pieces don't add up.
pub fn model_triangles(pmg: &[u8]) -> Option<Vec<[[f32; 3]; 3]>> {
    shown_triangles(pmg, None)
}

/// [`model_triangles`] of only the pieces `shown`, or all.
pub fn shown_triangles(pmg: &[u8], shown: Option<&[bool]>) -> Option<Vec<[[f32; 3]; 3]>> {
    let word = |at: usize| word(pmg, at);
    if word(0)? != PMG_MAGIC {
        return None;
    }
    let (count, table) = (word(PIECE_COUNT_AT)? as usize, word(PIECES_AT)? as usize);
    let mut triangles = Vec::new();
    for piece in 0..count {
        if shown.is_some_and(|shown| !shown.get(piece).copied().unwrap_or(true)) {
            continue;
        }
        let record = table.checked_add(piece.checked_mul(PIECE_SIZE)?)?;
        let field = |index: usize| word(record + 4 * index).map(|value| value as usize);
        let (indices, vertices, stride, positions, index_at) =
            (field(0)?, field(1)?, field(15)?, field(16)?, field(24)?);
        if stride < 12 || indices % 3 != 0 {
            return None;
        }
        let vertex = |index: usize| -> Option<[f32; 3]> {
            if index >= vertices {
                return None;
            }
            let at = positions.checked_add(index.checked_mul(stride)?)?;
            Some([0, 4, 8].map(|offset| word(at + offset).map(f32::from_bits).unwrap_or(f32::NAN)))
                .filter(|corner| corner.iter().all(|v| v.is_finite()))
        };
        let index = |which: usize| -> Option<usize> {
            let at = index_at.checked_add(which * 2)?;
            Some(u16::from_le_bytes(pmg.get(at..at + 2)?.try_into().ok()?) as usize)
        };
        for first in (0..indices).step_by(3) {
            triangles.push([vertex(index(first)?)?, vertex(index(first + 1)?)?, vertex(index(first + 2)?)?]);
        }
    }
    Some(triangles)
}
