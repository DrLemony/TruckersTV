//! The test TV moved while the game runs, as the player slides it in the companion (owner 2026-11-04).
//!
//! The game keeps a model's vertices as they are in its file, plain floats, in one big vertex buffer of its own
//! (read 2026-11-04: `DrawIndexed` of the glass, 36 bytes a vertex, a default-usage buffer holding every interior's).
//! So the vertices there are simply written anew, once for each move: nothing is added to the game's frame. The game
//! loading the model again (another truck, the shop) puts it at another `base`: it's found again, and moved again.
//!
//! The glass is told by the draw that has the screen's texture bound, the test TV's from the others' by its mark
//! ([`TEST_TV_MARK`]), and the panel by its look, near it in the buffer. None of it runs, not even the hook, until
//! the player moves the TV.

use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use westplay_shared::{TEST_TV_MARK, TV_BEZEL_UV, TV_GLASS_VERTICES, TV_PANEL_VERTICES, TV_VERTEX_SIZE};
use windows::Win32::Graphics::Direct3D11::{
    D3D11_BOX, D3D11_BUFFER_DESC, D3D11_CPU_ACCESS_READ, D3D11_DEVICE_CONTEXT_IMMEDIATE, D3D11_MAP_READ,
    D3D11_MAPPED_SUBRESOURCE, D3D11_USAGE_STAGING, ID3D11Buffer, ID3D11DeviceContext,
};
use windows::core::Interface;

use crate::{bridge, d3d, log};

/// The glass's indices: its one draw's count.
const GLASS_INDICES: u32 = 6;
const COLOUR_AT: usize = 24;
const UV_AT: usize = 28;
/// How far from the glass the panel is looked for (bytes, either way).
const PANEL_WITHIN: u32 = 16 * 1024;

static HOOKED: AtomicBool = AtomicBool::new(false);
/// The screen's texture was bound: the next draw is its glass.
static SCREEN_NEXT: AtomicBool = AtomicBool::new(false);
static PLACED: Mutex<Option<Placed>> = Mutex::new(None);

/// A TV as last found in the game's vertex buffer.
struct Placed {
    start: u32,
    base: i32,
    /// `test_tv_moves` as last carried out.
    moves: u32,
    /// Its vertices, as last written (empty: not the test TV), and where they are (bytes).
    glass: Vec<u8>,
    glass_at: u32,
    panel: Vec<u8>,
    panel_at: u32,
}

/// The screen's texture is being bound on `context`.
///
/// # Safety
/// `context` is the game's, live in this call, on its render thread.
pub unsafe fn screen_bound(context: *mut c_void) {
    let moved = bridge::get().is_some_and(|bridge| bridge.host.test_tv_moves.load(Ordering::Acquire) != 0);
    if !moved {
        return;
    }
    if !HOOKED.swap(true, Ordering::Relaxed) {
        // SAFETY: the game's live context (the caller's contract).
        if let Err(error) = unsafe { d3d::hook_draw_indexed(context) } {
            return log!("test TV: can't be moved: {error}");
        }
    }
    SCREEN_NEXT.store(true, Ordering::Relaxed);
}

/// Whether the draw about to be made may be the glass's: the draw hook's one check.
pub fn screen_next() -> bool {
    SCREEN_NEXT.swap(false, Ordering::Relaxed)
}

/// The draw after the screen's texture was bound, about to be made: the TV moved, if it's the test TV and the player
/// moved it since.
///
/// # Safety
/// `context` is the game's, live in this call, on its render thread.
pub unsafe fn on_draw(context: *mut c_void, count: u32, start: u32, base: i32) {
    let Some(host) = bridge::get().map(|bridge| &bridge.host) else { return };
    let moves = host.test_tv_moves.load(Ordering::Acquire);
    let mut placed = PLACED.lock().unwrap_or_else(|e| e.into_inner());
    let same = placed.as_ref().is_some_and(|placed| placed.start == start && placed.base == base);
    if count != GLASS_INDICES || (same && placed.as_ref().is_some_and(|placed| placed.moves == moves)) {
        return;
    }
    // SAFETY: the game's live context (the caller's contract): its vertex buffer read, and written where the test
    // TV's vertices were found.
    unsafe {
        let Some(context) = ID3D11DeviceContext::from_raw_borrowed(&context) else { return };
        if context.GetType() != D3D11_DEVICE_CONTEXT_IMMEDIATE {
            return;
        }
        let (mut buffer, mut stride, mut offset) = (None, 0, 0);
        context.IAGetVertexBuffers(0, 1, Some(&mut buffer), Some(&mut stride), Some(&mut offset));
        let Some(buffer) = buffer else { return };
        if stride as usize != TV_VERTEX_SIZE || base < 0 {
            return;
        }
        if !same {
            *placed = Some(find(context, &buffer, offset + base as u32 * stride, start, base));
        }
        let Some(placed) = placed.as_mut() else { return };
        placed.moves = moves;
        if placed.glass.is_empty() {
            return;
        }
        let glass = host.test_tv_glass.each_ref().map(|value| f32::from_bits(value.load(Ordering::Relaxed)));
        moved(&mut placed.glass, &mut placed.panel, glass);
        write(context, &buffer, placed.glass_at, &placed.glass);
        write(context, &buffer, placed.panel_at, &placed.panel);
    }
}

/// The TV whose glass is at `glass_at` in `buffer`: none of its vertices if it isn't the test TV.
///
/// # Safety
/// `context` is live, immediate, and of `buffer`'s device.
unsafe fn find(context: &ID3D11DeviceContext, buffer: &ID3D11Buffer, glass_at: u32, start: u32, base: i32) -> Placed {
    let mut placed = Placed { start, base, moves: 0, glass: Vec::new(), glass_at, panel: Vec::new(), panel_at: 0 };
    let size = (TV_GLASS_VERTICES * TV_VERTEX_SIZE) as u32;
    // SAFETY: as `read` requires (the caller's contract).
    let Some(glass) = (unsafe { read(context, buffer, glass_at, size) }) else { return placed };
    if !is_test_glass(&glass) {
        return placed;
    }
    let from = glass_at.saturating_sub(PANEL_WITHIN);
    // SAFETY: as above.
    let near = unsafe { read(context, buffer, from, glass_at - from + size + PANEL_WITHIN) }.unwrap_or_default();
    match find_panel(&near) {
        Some(at) => {
            placed.panel_at = from + at as u32;
            placed.panel = near[at..at + TV_PANEL_VERTICES * TV_VERTEX_SIZE].to_vec();
            log!("test TV: found, its glass at vertex {base}, its panel {} bytes from it", glass_at as i64 - placed.panel_at as i64);
        }
        None => log!("test TV: found, its glass at vertex {base}, no panel near it: the glass alone moves"),
    }
    placed.glass = glass;
    placed
}

fn is_test_glass(glass: &[u8]) -> bool {
    glass.len() == TV_GLASS_VERTICES * TV_VERTEX_SIZE
        && glass.chunks_exact(TV_VERTEX_SIZE).all(|vertex| vertex[COLOUR_AT] == TEST_TV_MARK)
}

/// Where in `near` the panel's vertices are: as many in a row as it has, all of the bezel's look.
fn find_panel(near: &[u8]) -> Option<usize> {
    let is_bezel = |at: usize| {
        let uv = near.get(at + UV_AT..at + TV_VERTEX_SIZE);
        uv.is_some_and(|uv| uv[..4] == TV_BEZEL_UV[0].to_le_bytes() && uv[4..] == TV_BEZEL_UV[1].to_le_bytes())
    };
    (0..near.len()).step_by(4).find(|at| (0..TV_PANEL_VERTICES).all(|vertex| is_bezel(at + vertex * TV_VERTEX_SIZE)))
}

fn position(vertex: &[u8]) -> [f32; 3] {
    [0, 4, 8].map(|at| f32::from_le_bytes([vertex[at], vertex[at + 1], vertex[at + 2], vertex[at + 3]]))
}

/// `glass` and `panel` (vertices) moved and scaled together for the glass's middle to be `target`'s first three (its
/// model's axes), and its half width (along z) the last. Its depth (x) stays as it is.
fn moved(glass: &mut [u8], panel: &mut [u8], target: [f32; 4]) {
    let corners: Vec<[f32; 3]> = glass.chunks_exact(TV_VERTEX_SIZE).map(position).collect();
    let middle = [0, 1, 2].map(|axis| corners.iter().map(|corner| corner[axis]).sum::<f32>() / corners.len() as f32);
    let half = corners.iter().map(|corner| (corner[2] - middle[2]).abs()).fold(0.0, f32::max);
    if half <= 0.0 {
        return;
    }
    let scale = target[3] / half;
    for vertex in glass.chunks_exact_mut(TV_VERTEX_SIZE).chain(panel.chunks_exact_mut(TV_VERTEX_SIZE)) {
        let [x, y, z] = position(vertex);
        let new = [x - middle[0] + target[0], target[1] + (y - middle[1]) * scale, target[2] + (z - middle[2]) * scale];
        for (at, value) in new.into_iter().enumerate() {
            vertex[at * 4..at * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
    }
}

/// `bytes` into `buffer` at `at`.
///
/// # Safety
/// `context` is live, immediate, and of `buffer`'s device; `buffer` isn't immutable and holds `bytes` at `at`.
unsafe fn write(context: &ID3D11DeviceContext, buffer: &ID3D11Buffer, at: u32, bytes: &[u8]) {
    if bytes.is_empty() {
        return;
    }
    let region = D3D11_BOX { left: at, right: at + bytes.len() as u32, top: 0, bottom: 1, front: 0, back: 1 };
    // SAFETY: as the caller says; `bytes` is the region's size.
    unsafe { context.UpdateSubresource(buffer, 0, Some(&region), bytes.as_ptr().cast(), 0, 0) }
}

/// `length` bytes of `buffer` from `at` (fewer at its end), through a staging copy: waits for the GPU, so only as a TV
/// is found.
///
/// # Safety
/// `context` is live, immediate, and of `buffer`'s device.
unsafe fn read(context: &ID3D11DeviceContext, buffer: &ID3D11Buffer, at: u32, length: u32) -> Option<Vec<u8>> {
    // SAFETY: a staging buffer of ours, `length` long, mapped for reading and let go once copied out.
    unsafe {
        let mut desc = D3D11_BUFFER_DESC::default();
        buffer.GetDesc(&mut desc);
        let length = length.min(desc.ByteWidth.saturating_sub(at));
        if length == 0 {
            return None;
        }
        let device = context.GetDevice().ok()?;
        let staging = D3D11_BUFFER_DESC {
            ByteWidth: length,
            Usage: D3D11_USAGE_STAGING,
            CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
            ..Default::default()
        };
        let mut copy = None;
        device.CreateBuffer(&staging, None, Some(&mut copy)).ok()?;
        let copy = copy?;
        let region = D3D11_BOX { left: at, right: at + length, top: 0, bottom: 1, front: 0, back: 1 };
        context.CopySubresourceRegion(&copy, 0, 0, 0, 0, buffer, 0, Some(&region));
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        context.Map(&copy, 0, D3D11_MAP_READ, 0, Some(&mut mapped)).ok()?;
        let bytes = std::slice::from_raw_parts(mapped.pData as *const u8, length as usize).to_vec();
        context.Unmap(&copy, 0);
        Some(bytes)
    }
}

#[cfg(test)]
mod tests {
    use windows::Win32::Foundation::HMODULE;
    use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_WARP;
    use windows::Win32::Graphics::Direct3D11::{
        D3D11_BIND_VERTEX_BUFFER, D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION, D3D11_SUBRESOURCE_DATA,
        D3D11_USAGE_DEFAULT, D3D11CreateDevice, ID3D11Device,
    };

    use super::*;

    fn vertex(p: [f32; 3], mark: u8, uv: [f32; 2]) -> Vec<u8> {
        let mut bytes: Vec<u8> = p.into_iter().flat_map(f32::to_le_bytes).collect();
        bytes.extend([-1.0f32, 0.0, 0.0].into_iter().flat_map(f32::to_le_bytes));
        bytes.extend([mark, 0x80, 0x80, 0xff]);
        bytes.extend(uv.into_iter().flat_map(f32::to_le_bytes));
        bytes
    }

    /// A glass 2 wide and 1 high about (1, 2, 3).
    fn glass(mark: u8) -> Vec<u8> {
        [[2.0, 1.5], [4.0, 1.5], [4.0, 2.5], [2.0, 2.5]]
            .into_iter()
            .flat_map(|[z, y]| vertex([1.0, y, z], mark, [0.0, 1.0]))
            .collect()
    }

    /// The panel: every vertex at its corner, just past the glass.
    fn panel() -> Vec<u8> {
        (0..TV_PANEL_VERTICES).flat_map(|_| vertex([1.1, 1.4, 1.9], 0x80, TV_BEZEL_UV)).collect()
    }

    #[test]
    fn moves_and_scales_the_glass_and_the_panel_together() {
        let (mut glass, mut panel) = (glass(TEST_TV_MARK), panel());
        moved(&mut glass, &mut panel, [5.0, 6.0, 7.0, 2.0]);
        let corners: Vec<[f32; 3]> = glass.chunks_exact(TV_VERTEX_SIZE).map(position).collect();
        assert_eq!(corners, [[5.0, 5.0, 5.0], [5.0, 5.0, 9.0], [5.0, 7.0, 9.0], [5.0, 7.0, 5.0]]);
        // Twice as big about the new middle; only moved in depth.
        let corner = position(&panel);
        assert!((corner[0] - 5.1).abs() < 1e-5 && (corner[1] - 4.8).abs() < 1e-5 && (corner[2] - 4.8).abs() < 1e-5);
        // Its look stays.
        assert_eq!(glass[COLOUR_AT], TEST_TV_MARK);
        assert_eq!(panel[UV_AT..UV_AT + 4], TV_BEZEL_UV[0].to_le_bytes());
    }

    #[test]
    fn finds_the_panel_near_the_glass_at_any_alignment() {
        let mut near = vec![0u8; 100];
        near.extend(panel());
        near.extend(glass(TEST_TV_MARK));
        assert_eq!(find_panel(&near), Some(100));
        assert_eq!(find_panel(&near[..near.len() - TV_GLASS_VERTICES * TV_VERTEX_SIZE - TV_VERTEX_SIZE]), None);
    }

    #[test]
    fn only_the_test_tv_s_glass_is_taken() {
        assert!(is_test_glass(&glass(TEST_TV_MARK)));
        assert!(!is_test_glass(&glass(0x80)));
        assert!(!is_test_glass(&glass(TEST_TV_MARK)[TV_VERTEX_SIZE..]));
    }

    #[test]
    fn finds_it_in_a_vertex_buffer_and_writes_it_there_anew() {
        let mut bytes = vec![7u8; 360];
        let panel_at = bytes.len();
        bytes.extend(panel());
        let glass_at = bytes.len() as u32;
        bytes.extend(glass(TEST_TV_MARK));
        bytes.extend([9u8; 360]);
        let (mut device, mut context) = (None, None);
        // SAFETY: a software device of the test's own, and all it's given is its own and live throughout.
        unsafe {
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_WARP,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_FLAG(0),
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut context),
            )
            .unwrap();
            let (device, context): (ID3D11Device, ID3D11DeviceContext) = (device.unwrap(), context.unwrap());
            let desc = D3D11_BUFFER_DESC {
                ByteWidth: bytes.len() as u32,
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as u32,
                ..Default::default()
            };
            let data = D3D11_SUBRESOURCE_DATA { pSysMem: bytes.as_ptr().cast(), ..Default::default() };
            let mut buffer = None;
            device.CreateBuffer(&desc, Some(&data), Some(&mut buffer)).unwrap();
            let buffer = buffer.unwrap();

            let mut placed = find(&context, &buffer, glass_at, 0, 0);
            assert_eq!((placed.panel_at as usize, placed.glass.len()), (panel_at, 144));
            moved(&mut placed.glass, &mut placed.panel, [5.0, 6.0, 7.0, 1.0]);
            write(&context, &buffer, placed.glass_at, &placed.glass);
            write(&context, &buffer, placed.panel_at, &placed.panel);
            let now = read(&context, &buffer, 0, bytes.len() as u32).unwrap();
            assert_eq!(now[glass_at as usize..glass_at as usize + 4], 5.0f32.to_le_bytes());
            assert_eq!(now[..360], bytes[..360]);
            assert_eq!(now[now.len() - 360..], bytes[bytes.len() - 360..]);

            // Another TV's glass is left alone.
            let other = find(&context, &buffer, 0, 0, 0);
            assert!(other.glass.is_empty());
        }
    }
}
