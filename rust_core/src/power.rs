//! System power plan definitions and Linux platform_profile integration.

/// Excalibur Hardware Power Plans
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PowerPlan {
    /// High Power / Maximum Performance (Turbo)
    HighPower = 1,
    /// Gaming / Balanced High Performance
    Gaming = 2,
    /// Text Mode / Office / Quiet Mode
    TextMode = 3,
    /// Low Power / Battery Saver / Eco Mode
    LowPower = 4,
}

impl PowerPlan {
    pub const MIN: u32 = 1;
    pub const MAX: u32 = 4;

    pub fn from_u32(val: u32) -> Option<Self> {
        match val {
            1 => Some(Self::HighPower),
            2 => Some(Self::Gaming),
            3 => Some(Self::TextMode),
            4 => Some(Self::LowPower),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> u32 {
        *self as u32
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HighPower => "high_power",
            Self::Gaming => "gaming",
            Self::TextMode => "text_mode",
            Self::LowPower => "low_power",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        match trimmed {
            "high_power" | "performance" | "turbo" | "1" => Some(Self::HighPower),
            "gaming" | "balanced" | "game" | "2" => Some(Self::Gaming),
            "text_mode" | "office" | "quiet" | "silent" | "3" => Some(Self::TextMode),
            "low_power" | "eco" | "battery" | "saver" | "4" => Some(Self::LowPower),
            _ => None,
        }
    }

    /// Linux kernel platform_profile_option enum values
    /// Reference: linux/platform_profile.h
    pub const PLATFORM_PROFILE_LOW_POWER: i32 = 0;
    pub const PLATFORM_PROFILE_QUIET: i32 = 2;
    pub const PLATFORM_PROFILE_BALANCED: i32 = 3;
    pub const PLATFORM_PROFILE_PERFORMANCE: i32 = 5;

    /// Converts Linux platform_profile option enum into Excalibur PowerPlan
    pub fn from_platform_profile(profile: i32) -> Option<Self> {
        match profile {
            Self::PLATFORM_PROFILE_LOW_POWER => Some(Self::LowPower),
            Self::PLATFORM_PROFILE_QUIET => Some(Self::TextMode),
            Self::PLATFORM_PROFILE_BALANCED => Some(Self::Gaming),
            Self::PLATFORM_PROFILE_PERFORMANCE => Some(Self::HighPower),
            _ => None,
        }
    }

    /// Converts Excalibur PowerPlan into Linux platform_profile option enum
    pub fn to_platform_profile(&self) -> i32 {
        match self {
            Self::HighPower => Self::PLATFORM_PROFILE_PERFORMANCE,
            Self::Gaming => Self::PLATFORM_PROFILE_BALANCED,
            Self::TextMode => Self::PLATFORM_PROFILE_QUIET,
            Self::LowPower => Self::PLATFORM_PROFILE_LOW_POWER,
        }
    }
}
