//! Hardware protocol constants and low-level packet structures for Casper Excalibur laptops.

/// Casper Excalibur ACPI-WMI Device GUID
pub const EXCALIBUR_WMI_GUID: &str = "644C5791-B7B0-4123-A90B-E93876E0DAAD";

/// WMI Command Codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum WmiCommand {
    /// Read request to firmware
    Read = 0xfa00,
    /// Write request to firmware
    Write = 0xfb00,
    /// Query hardware sensor info (Fan speeds, status)
    GetHardwareInfo = 0x0200,
    /// Set LED lighting state for a zone
    SetLed = 0x0100,
    /// Get / Set system power plan
    PowerPlan = 0x0300,
}

/// Raw WMI packet layout matching ACPI buffer structure passed to/from wmidev_block_set/query.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ExcaliburWmiArgs {
    pub a0: u16,
    pub a1: u16,
    pub a2: u32,
    pub a3: u32,
    pub a4: u32,
    pub a5: u32,
    pub a6: u32,
    pub rev0: u32,
    pub rev1: u32,
}

impl ExcaliburWmiArgs {
    /// Creates a write request buffer.
    #[inline]
    pub const fn write_cmd(sub_cmd: u16, zone_or_param: u32, data: u32) -> Self {
        Self {
            a0: WmiCommand::Write as u16,
            a1: sub_cmd,
            a2: zone_or_param,
            a3: data,
            a4: 0,
            a5: 0,
            a6: 0,
            rev0: 0,
            rev1: 0,
        }
    }

    /// Creates a read query request buffer.
    #[inline]
    pub const fn read_cmd(sub_cmd: u16) -> Self {
        Self {
            a0: WmiCommand::Read as u16,
            a1: sub_cmd,
            a2: 0,
            a3: 0,
            a4: 0,
            a5: 0,
            a6: 0,
            rev0: 0,
            rev1: 0,
        }
    }
}
