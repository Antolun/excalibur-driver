//! RGB Keyboard & Corner LED lighting protocol and state management.

/// Hardware Zone IDs
pub const ZONE_LEFT: u8 = 0x05;
pub const ZONE_MIDDLE: u8 = 0x04;
pub const ZONE_RIGHT: u8 = 0x03;
pub const ZONE_ALL_KBD: u8 = 0x06;
pub const ZONE_CORNERS: u8 = 0x07;

pub const KBD_ZONE_COUNT: usize = 3;
pub const TOTAL_ZONE_COUNT: usize = 4;
pub const MAX_BRIGHTNESS: u8 = 2;

pub const ZONE_IDS: [u8; TOTAL_ZONE_COUNT] = [
    ZONE_LEFT,
    ZONE_MIDDLE,
    ZONE_RIGHT,
    ZONE_CORNERS,
];

pub const ZONE_NAMES: [&str; TOTAL_ZONE_COUNT] = [
    "excalibur::kbd_backlight-left",
    "excalibur::kbd_backlight-middle",
    "excalibur::kbd_backlight-right",
    "excalibur::kbd_backlight-corners",
];

/// Animation modes supported by the Excalibur EC/WMI controller
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LedMode {
    Off = 0,
    Static = 1,
    Blink = 2,
    Fade = 3,
    Heartbeat = 4,
    Wave = 5,
    Random = 6,
    Rainbow = 7,
}

impl LedMode {
    pub const ALL_MODES_STR: &'static str = "off static blink fade heartbeat wave random rainbow";

    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::Off),
            1 => Some(Self::Static),
            2 => Some(Self::Blink),
            3 => Some(Self::Fade),
            4 => Some(Self::Heartbeat),
            5 => Some(Self::Wave),
            6 => Some(Self::Random),
            7 => Some(Self::Rainbow),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Static => "static",
            Self::Blink => "blink",
            Self::Fade => "fade",
            Self::Heartbeat => "heartbeat",
            Self::Wave => "wave",
            Self::Random => "random",
            Self::Rainbow => "rainbow",
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut s = bytes;
        while let Some((&first, rest)) = s.split_first() {
            if first == b' ' || first == b'\t' || first == b'\n' || first == b'\r' {
                s = rest;
            } else {
                break;
            }
        }
        while let Some((&last, rest)) = s.split_last() {
            if last == b' ' || last == b'\t' || last == b'\n' || last == b'\r' {
                s = rest;
            } else {
                break;
            }
        }

        const MODES: [(&[u8], LedMode); 17] = [
            (b"off", LedMode::Off),
            (b"0", LedMode::Off),
            (b"static", LedMode::Static),
            (b"1", LedMode::Static),
            (b"blink", LedMode::Blink),
            (b"2", LedMode::Blink),
            (b"fade", LedMode::Fade),
            (b"breathe", LedMode::Fade),
            (b"3", LedMode::Fade),
            (b"heartbeat", LedMode::Heartbeat),
            (b"4", LedMode::Heartbeat),
            (b"wave", LedMode::Wave),
            (b"5", LedMode::Wave),
            (b"random", LedMode::Random),
            (b"6", LedMode::Random),
            (b"rainbow", LedMode::Rainbow),
            (b"7", LedMode::Rainbow),
        ];

        let mut i = 0;
        while i < MODES.len() {
            if s.eq_ignore_ascii_case(MODES[i].0) {
                return Some(MODES[i].1);
            }
            i += 1;
        }
        None
    }
}

/// Represents an RGB color (24-bit)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const WHITE: Self = Self { r: 0xFF, g: 0xFF, b: 0xFF };
    pub const BLACK: Self = Self { r: 0x00, g: 0x00, b: 0x00 };
    pub const RED:   Self = Self { r: 0xFF, g: 0x00, b: 0x00 };
    pub const GREEN: Self = Self { r: 0x00, g: 0xFF, b: 0x00 };
    pub const BLUE:  Self = Self { r: 0x00, g: 0x00, b: 0xFF };

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parses hex string like "FF0000", "#FF0000", "0xFF0000"
    pub fn parse_hex(bytes: &[u8]) -> Option<Self> {
        let mut s = bytes;
        // Trim whitespace
        while let Some((&first, rest)) = s.split_first() {
            if first == b' ' || first == b'\t' || first == b'\n' || first == b'\r' {
                s = rest;
            } else {
                break;
            }
        }
        while let Some((&last, rest)) = s.split_last() {
            if last == b' ' || last == b'\t' || last == b'\n' || last == b'\r' {
                s = rest;
            } else {
                break;
            }
        }

        // Strip prefix '#' or '0x' / '0X'
        if s.starts_with(b"#") {
            s = &s[1..];
        } else if s.starts_with(b"0x") || s.starts_with(b"0X") {
            s = &s[2..];
        }

        if s.len() != 6 {
            return None;
        }

        let hex_val = |b: u8| -> Option<u8> {
            match b {
                b'0'..=b'9' => Some(b - b'0'),
                b'a'..=b'f' => Some(b - b'a' + 10),
                b'A'..=b'F' => Some(b - b'A' + 10),
                _ => None,
            }
        };

        let h0 = hex_val(s[0])?;
        let h1 = hex_val(s[1])?;
        let h2 = hex_val(s[2])?;
        let h3 = hex_val(s[3])?;
        let h4 = hex_val(s[4])?;
        let h5 = hex_val(s[5])?;

        Some(Self {
            r: (h0 << 4) | h1,
            g: (h2 << 4) | h3,
            b: (h4 << 4) | h5,
        })
    }
}

/// 32-bit Packed LED Word for Excalibur EC/WMI hardware.
///
/// Bit layout:
///   Bits [31:28]  mode   (0..7)
///   Bits [27:24]  alpha  (brightness level 0..2)
///   Bits [23:16]  red    (0..255)
///   Bits [15: 8]  green  (0..255)
///   Bits [ 7: 0]  blue   (0..255)
#[inline]
pub const fn pack_led_word(mode: u8, brightness: u8, color: RgbColor) -> u32 {
    let m = (mode as u32 & 0x0F) << 28;
    let a = (brightness as u32 & 0x0F) << 24;
    let r = (color.r as u32) << 16;
    let g = (color.g as u32) << 8;
    let b = color.b as u32;
    m | a | r | g | b
}

/// Unpacks a 32-bit LED word into its constituent parts
#[inline]
pub const fn unpack_led_word(val: u32) -> (u8, u8, RgbColor) {
    let mode = ((val >> 28) & 0x0F) as u8;
    let brightness = ((val >> 24) & 0x0F) as u8;
    let color = RgbColor {
        r: ((val >> 16) & 0xFF) as u8,
        g: ((val >> 8) & 0xFF) as u8,
        b: (val & 0xFF) as u8,
    };
    (mode, brightness, color)
}

/// State representation for a single lighting zone
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZoneState {
    pub zone_id: u8,
    pub mode: u8,
    pub brightness: u8,
    pub color: RgbColor,
}

impl ZoneState {
    pub const fn new(zone_id: u8) -> Self {
        Self {
            zone_id,
            mode: LedMode::Static as u8,
            brightness: 0,
            color: RgbColor::WHITE,
        }
    }

    #[inline]
    pub const fn packed_word(&self) -> u32 {
        pack_led_word(self.mode, self.brightness, self.color)
    }
}
