//! The game's camera zoomed in by a keybind of the player's ([`Keybind::Zoom`]): pressed once, it holds the game's
//! own `camzoom` (the middle mouse button's zoom, in the cab to look at the screen up close); pressed again, it
//! lets go. Through the SDK's input API, as a semantical device (the game's controls by name: `controls.sii`'s
//! `semantical.camzoom`), not the game's memory: the game zooms as for its own button.
//!
//! [`Keybind::Zoom`]: westplay_shared::Keybind::Zoom

use std::ffi::{c_char, c_void};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use crate::scs::{self, SCS_RESULT_OK, SCS_RESULT_UNSUPPORTED, SCS_VALUE_TYPE_BOOL, ScsResult, SdkInitParamsV100};
use crate::{hooks, log};

const SCS_INPUT_VERSION_1_00: u32 = scs::make_version(1, 0);
const SCS_RESULT_NOT_FOUND: ScsResult = -4;
const SCS_INPUT_DEVICE_TYPE_SEMANTICAL: u32 = 2;
const SCS_INPUT_EVENT_CALLBACK_FLAG_FIRST_AFTER_ACTIVATION: u32 = 2;

/// Whether the player wants the camera zoomed (the keybind's toggle).
static WANTED: AtomicBool = AtomicBool::new(false);
/// What the game was last told: 0 not zoomed, 1 zoomed, [`UNTOLD`] nothing since the device became active.
static TOLD: AtomicU8 = AtomicU8::new(UNTOLD);
const UNTOLD: u8 = 2;

/// The keybind pressed: zoomed in if it wasn't, out if it was.
pub fn toggle() {
    let zoomed = !WANTED.fetch_xor(true, Ordering::AcqRel);
    log!("zoom {}", if zoomed { "in" } else { "out" });
}

/// `scs_input_init_params_v100_t` (40 bytes on x64).
#[repr(C)]
pub struct InputInitParamsV100 {
    common: SdkInitParamsV100,
    register_device: Option<unsafe extern "system" fn(device: *const Device) -> ScsResult>,
}

/// `scs_input_device_input_t` (24 bytes on x64).
#[repr(C)]
struct DeviceInput {
    name: *const c_char,
    display_name: *const c_char,
    value_type: u32,
    _padding: u32,
}

/// `scs_input_device_t` (56 bytes on x64).
#[repr(C)]
pub struct Device {
    name: *const c_char,
    display_name: *const c_char,
    device_type: u32,
    input_count: u32,
    inputs: *const DeviceInput,
    callback_context: *mut c_void,
    input_active_callback: Option<unsafe extern "system" fn(active: u8, context: *mut c_void)>,
    input_event_callback: Option<unsafe extern "system" fn(*mut Event, flags: u32, context: *mut c_void) -> ScsResult>,
}

/// `scs_input_event_t` (28 bytes): its input's index, and its value (a bool's in the first byte).
#[repr(C)]
pub struct Event {
    input_index: u32,
    value: [u8; 24],
}

/// The device's one input, kept for the game (static strings).
struct Inputs([DeviceInput; 1]);
// SAFETY: only pointers to static, never-written C strings.
unsafe impl Sync for Inputs {}
static INPUTS: Inputs = Inputs([DeviceInput {
    name: c"camzoom".as_ptr(),
    display_name: c"Zoom".as_ptr(),
    value_type: SCS_VALUE_TYPE_BOOL,
    _padding: 0,
}]);

/// Called by the game when it activates the input API: registers the device.
///
/// # Safety
/// Called by the game with a valid `params` pointer for the negotiated `version`, on its main thread.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn scs_input_init(version: u32, params: *const InputInitParamsV100) -> ScsResult {
    // SAFETY: as the game calls this (the contract above).
    hooks::contained("scs_input_init", SCS_RESULT_UNSUPPORTED, || unsafe { init(version, params) })
}

/// # Safety
/// As [`scs_input_init`].
unsafe fn init(version: u32, params: *const InputInitParamsV100) -> ScsResult {
    if version != SCS_INPUT_VERSION_1_00 {
        return SCS_RESULT_UNSUPPORTED;
    }
    // SAFETY: the game's parameters for this version, alive through the call (the SDK's contract).
    let Some(register) = (unsafe { params.as_ref() }).and_then(|params| params.register_device) else {
        return SCS_RESULT_UNSUPPORTED;
    };
    log::init();
    let device = Device {
        name: c"westplay".as_ptr(),
        display_name: c"TruckersTV".as_ptr(),
        device_type: SCS_INPUT_DEVICE_TYPE_SEMANTICAL,
        input_count: INPUTS.0.len() as u32,
        inputs: INPUTS.0.as_ptr(),
        callback_context: std::ptr::null_mut(),
        input_active_callback: None,
        input_event_callback: Some(next_event),
    };
    // SAFETY: the game's own function, with a device laid out as the SDK's, its strings and inputs static.
    let result = unsafe { register(&device) };
    log!("zoom input device registered: {result}");
    result
}

/// Called by the game when it deactivates the input API.
#[unsafe(no_mangle)]
pub extern "system" fn scs_input_shutdown() {
    TOLD.store(UNTOLD, Ordering::Release);
}

/// The game asking for the device's next event, each frame until there's none: the zoom, when the player's
/// wish differs from what the game was told (told again each time the device becomes active).
unsafe extern "system" fn next_event(event: *mut Event, flags: u32, _context: *mut c_void) -> ScsResult {
    if flags & SCS_INPUT_EVENT_CALLBACK_FLAG_FIRST_AFTER_ACTIVATION != 0 {
        TOLD.store(UNTOLD, Ordering::Release);
    }
    let wanted = u8::from(WANTED.load(Ordering::Acquire));
    if TOLD.swap(wanted, Ordering::AcqRel) == wanted {
        return SCS_RESULT_NOT_FOUND;
    }
    // SAFETY: the game's event to fill, valid through the call.
    if let Some(event) = unsafe { event.as_mut() } {
        event.input_index = 0;
        event.value = [0; 24];
        event.value[0] = wanted;
    }
    SCS_RESULT_OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn laid_out_as_the_sdk() {
        assert_eq!(size_of::<InputInitParamsV100>(), 40);
        assert_eq!(size_of::<DeviceInput>(), 24);
        assert_eq!(size_of::<Device>(), 56);
        assert_eq!(size_of::<Event>(), 28);
    }

    #[test]
    fn tells_the_game_each_change_once() {
        let mut storage = Event { input_index: 9, value: [0; 24] };
        let event: *mut Event = &mut storage;
        // SAFETY: `storage`, alive through the test and reached only through `event`.
        let next = |flags| unsafe { next_event(event, flags, std::ptr::null_mut()) };
        let told = || unsafe { ((*event).input_index, (*event).value[0]) };
        // Becoming active: told it's not zoomed, then nothing more.
        assert_eq!(next(SCS_INPUT_EVENT_CALLBACK_FLAG_FIRST_AFTER_ACTIVATION), SCS_RESULT_OK);
        assert_eq!(told(), (0, 0));
        assert_eq!(next(0), SCS_RESULT_NOT_FOUND);
        toggle();
        assert_eq!(next(0), SCS_RESULT_OK);
        assert_eq!(told(), (0, 1));
        assert_eq!(next(0), SCS_RESULT_NOT_FOUND);
        toggle();
        assert_eq!(next(0), SCS_RESULT_OK);
        assert_eq!(told(), (0, 0));
        assert_eq!(next(0), SCS_RESULT_NOT_FOUND);
    }
}
