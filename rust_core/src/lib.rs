#![no_std]
#![no_main]

pub mod fan;
pub mod ffi;
pub mod led;
pub mod power;
pub mod protocol;

pub use fan::*;
pub use ffi::*;
pub use led::*;
pub use power::*;
pub use protocol::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_led_pack_unpack() {
        let col = RgbColor::new(0x12, 0x34, 0x56);
        let word = pack_led_word(LedMode::Fade as u8, 2, col);
        let (mode, br, unpacked_col) = unpack_led_word(word);
        assert_eq!(mode, LedMode::Fade as u8);
        assert_eq!(br, 2);
        assert_eq!(unpacked_col, col);
    }

    #[test]
    fn test_color_hex_parser() {
        let col = RgbColor::parse_hex(b"FF0080").unwrap();
        assert_eq!(col.r, 0xFF);
        assert_eq!(col.g, 0x00);
        assert_eq!(col.b, 0x80);

        let col_hash = RgbColor::parse_hex(b"#112233\n").unwrap();
        assert_eq!(col_hash.r, 0x11);
        assert_eq!(col_hash.g, 0x22);
        assert_eq!(col_hash.b, 0x33);
    }

    #[test]
    fn test_power_plan_mapping() {
        assert_eq!(
            PowerPlan::from_platform_profile(PowerPlan::PLATFORM_PROFILE_PERFORMANCE),
            Some(PowerPlan::HighPower)
        );
        assert_eq!(
            PowerPlan::HighPower.to_platform_profile(),
            PowerPlan::PLATFORM_PROFILE_PERFORMANCE
        );
    }

    #[test]
    fn test_fanspeed_quirk() {
        // Little endian
        let rpm = decode_fanspeed(0x0DAC, true); // 3500
        assert_eq!(rpm, 3500);

        // Big endian swap (0xAC0D -> 3500)
        let rpm_swapped = decode_fanspeed(0xAC0D, false);
        assert_eq!(rpm_swapped, 3500);
    }
}
