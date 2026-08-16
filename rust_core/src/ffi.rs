//! C-compatible Foreign Function Interface (FFI) and Panic Handler for Linux Kernel integration.

use core::ffi::c_char;
use core::panic::PanicInfo;
use core::slice;

use crate::fan::decode_fanspeed;
use crate::led::{
    pack_led_word, unpack_led_word, LedMode, RgbColor, TOTAL_ZONE_COUNT, ZONE_IDS,
};
use crate::power::PowerPlan;
use crate::protocol::ExcaliburWmiArgs;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// Returns the sysfs device name for a given zone index (0..3)
#[no_mangle]
pub extern "C" fn excalibur_core_get_zone_name(index: usize) -> *const c_char {
    if index < TOTAL_ZONE_COUNT {
        match index {
            0 => b"excalibur::kbd_backlight-left\0".as_ptr() as *const c_char,
            1 => b"excalibur::kbd_backlight-middle\0".as_ptr() as *const c_char,
            2 => b"excalibur::kbd_backlight-right\0".as_ptr() as *const c_char,
            3 => b"excalibur::kbd_backlight-corners\0".as_ptr() as *const c_char,
            _ => core::ptr::null(),
        }
    } else {
        core::ptr::null()
    }
}

/// Returns the hardware zone ID for a given zone index (0..3)
#[no_mangle]
pub extern "C" fn excalibur_core_get_zone_id(index: usize) -> u8 {
    if index < TOTAL_ZONE_COUNT {
        ZONE_IDS[index]
    } else {
        0
    }
}

/// Packs mode, brightness, and RGB components into 32-bit hardware LED word.
#[no_mangle]
pub extern "C" fn excalibur_core_pack_led(
    mode: u8,
    brightness: u8,
    r: u8,
    g: u8,
    b: u8,
) -> u32 {
    pack_led_word(mode, brightness, RgbColor::new(r, g, b))
}

/// Unpacks a 32-bit hardware LED word into mode, brightness, and RGB components.
#[no_mangle]
pub unsafe extern "C" fn excalibur_core_unpack_led(
    packed: u32,
    mode: *mut u8,
    brightness: *mut u8,
    r: *mut u8,
    g: *mut u8,
    b: *mut u8,
) {
    let (m, br, col) = unpack_led_word(packed);
    if !mode.is_null() {
        *mode = m;
    }
    if !brightness.is_null() {
        *brightness = br;
    }
    if !r.is_null() {
        *r = col.r;
    }
    if !g.is_null() {
        *g = col.g;
    }
    if !b.is_null() {
        *b = col.b;
    }
}

/// Parses a hex color string (e.g. "FF0000", "#00FF88", "0x123456") into RGB bytes.
/// Returns 0 on success, -EINVAL (-22) on failure.
#[no_mangle]
pub unsafe extern "C" fn excalibur_core_parse_color_hex(
    str_ptr: *const c_char,
    len: usize,
    r: *mut u8,
    g: *mut u8,
    b: *mut u8,
) -> i32 {
    if str_ptr.is_null() || len == 0 {
        return -22; // -EINVAL
    }

    let bytes = slice::from_raw_parts(str_ptr as *const u8, len);
    if let Some(col) = RgbColor::parse_hex(bytes) {
        if !r.is_null() {
            *r = col.r;
        }
        if !g.is_null() {
            *g = col.g;
        }
        if !b.is_null() {
            *b = col.b;
        }
        0
    } else {
        -22 // -EINVAL
    }
}

/// Parses an animation mode string (e.g. "fade", "rainbow", "static").
/// Returns 0 on success, -EINVAL (-22) on failure.
#[no_mangle]
pub unsafe extern "C" fn excalibur_core_parse_mode_str(
    str_ptr: *const c_char,
    len: usize,
    mode_out: *mut u8,
) -> i32 {
    if str_ptr.is_null() || len == 0 || mode_out.is_null() {
        return -22; // -EINVAL
    }

    let bytes = slice::from_raw_parts(str_ptr as *const u8, len);
    if let Some(mode) = LedMode::from_bytes(bytes) {
        *mode_out = mode as u8;
        0
    } else {
        -22 // -EINVAL
    }
}

/// Returns human-readable name of animation mode as C string pointer
#[no_mangle]
pub extern "C" fn excalibur_core_mode_name(mode: u8) -> *const c_char {
    match LedMode::from_u8(mode) {
        Some(LedMode::Off) => b"off\0".as_ptr() as *const c_char,
        Some(LedMode::Static) => b"static\0".as_ptr() as *const c_char,
        Some(LedMode::Blink) => b"blink\0".as_ptr() as *const c_char,
        Some(LedMode::Fade) => b"fade\0".as_ptr() as *const c_char,
        Some(LedMode::Heartbeat) => b"heartbeat\0".as_ptr() as *const c_char,
        Some(LedMode::Wave) => b"wave\0".as_ptr() as *const c_char,
        Some(LedMode::Random) => b"random\0".as_ptr() as *const c_char,
        Some(LedMode::Rainbow) => b"rainbow\0".as_ptr() as *const c_char,
        None => b"unknown\0".as_ptr() as *const c_char,
    }
}

/// Returns space-separated list of all available modes as C string pointer
#[no_mangle]
pub extern "C" fn excalibur_core_available_modes() -> *const c_char {
    b"off static blink fade heartbeat wave random rainbow\0".as_ptr() as *const c_char
}

/// Decodes fan speed RPM from 32-bit register value
#[no_mangle]
pub extern "C" fn excalibur_core_decode_fanspeed(raw: u32, has_raw_fanspeed: bool) -> u16 {
    decode_fanspeed(raw, has_raw_fanspeed)
}

/// Returns fan label for hwmon channel (0=cpu_fan, 1=gpu_fan)
#[no_mangle]
pub extern "C" fn excalibur_core_fan_label(channel: usize) -> *const c_char {
    match channel {
        0 => b"cpu_fan\0".as_ptr() as *const c_char,
        1 => b"gpu_fan\0".as_ptr() as *const c_char,
        _ => core::ptr::null(),
    }
}

/// Validates power plan integer (1..4)
#[no_mangle]
pub extern "C" fn excalibur_core_validate_power_plan(plan: u32) -> bool {
    PowerPlan::from_u32(plan).is_some()
}

/// Converts Linux platform_profile option enum into Excalibur PowerPlan integer (1..4).
/// Returns -EINVAL (-22) on unmapped profile.
#[no_mangle]
pub extern "C" fn excalibur_core_platform_profile_to_plan(profile: i32) -> i32 {
    match PowerPlan::from_platform_profile(profile) {
        Some(plan) => plan.as_u32() as i32,
        None => -22, // -EINVAL
    }
}

/// Converts Excalibur PowerPlan integer (1..4) into Linux platform_profile option enum.
/// Returns -EINVAL (-22) on invalid plan.
#[no_mangle]
pub extern "C" fn excalibur_core_plan_to_platform_profile(plan: u32) -> i32 {
    match PowerPlan::from_u32(plan) {
        Some(p) => p.to_platform_profile(),
        None => -22, // -EINVAL
    }
}

/// Populates a WMI write arguments buffer
#[no_mangle]
pub unsafe extern "C" fn excalibur_core_create_write_args(
    sub_cmd: u16,
    param: u32,
    data: u32,
    out: *mut ExcaliburWmiArgs,
) {
    if !out.is_null() {
        *out = ExcaliburWmiArgs::write_cmd(sub_cmd, param, data);
    }
}

/// Populates a WMI read arguments buffer
#[no_mangle]
pub unsafe extern "C" fn excalibur_core_create_read_args(
    sub_cmd: u16,
    out: *mut ExcaliburWmiArgs,
) {
    if !out.is_null() {
        *out = ExcaliburWmiArgs::read_cmd(sub_cmd);
    }
}
