//! Hardware monitoring and Fan RPM decoding with per-model quirk support.

/// Decodes fan speed RPM from raw 32-bit register value returned by WMI query.
///
/// Newer generations (11th+ gen, G770, G870, G911) return RPM in native little-endian.
/// Older generations (10th gen Intel, G650, G750, G670, G900) return RPM in big-endian bytes.
#[inline]
pub fn decode_fanspeed(raw: u32, has_raw_fanspeed: bool) -> u16 {
    let val = (raw & 0xFFFF) as u16;
    if has_raw_fanspeed {
        val
    } else {
        val.swap_bytes()
    }
}

/// Fan sensor identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanSensor {
    Cpu = 0,
    Gpu = 1,
}

impl FanSensor {
    pub const COUNT: usize = 2;

    pub fn label(&self) -> &'static str {
        match self {
            Self::Cpu => "cpu_fan",
            Self::Gpu => "gpu_fan",
        }
    }
}
