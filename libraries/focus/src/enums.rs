use bazecor_proc_macros::*;
use serde::{Deserialize, Serialize};

/// Time units for use with converting from string.
pub(crate) enum TimeUnit {
    Milliseconds,
    Seconds,
}

/// The LED mode states.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, NumStrEnum)]
pub enum LedMode {
    /// The default mode. The LEDs will be set to the color of the layer you are on.
    PerLayer = 0,
    /// Multi-colored rainbow effect.
    RainbowMulti = 1,
    /// Single-colored rainbow effect.
    RainbowSingle = 2,
    /// All LEDs will be off until pressed, they will light up when pressed and cycle colors back to off.
    Stalker = 3,
    /// All LEDs to red.
    Red = 4,
    /// All LEDs to green.
    Green = 5,
    /// All LEDs to blue.
    Blue = 6,
    /// All LEDs to white.
    White = 7,
    /// All LEDs to off.
    Off = 8,
    /// The inner three LEDs on both sides will be green, the rest will be off.
    GreenInner = 9,
    /// Emulates the bluetooth connect sequence.
    Bluetooth = 10,
}

/// The wireless power mode states.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, NumStrEnum)]
pub enum WirelessPowerMode {
    /// Low power mode. The battery will last longer but the wireless range will be shorter.
    Low = 0,
    /// Medium power mode. The battery will last a bit less but the wireless range will be longer.
    Medium = 1,
    /// High power mode. The battery will last the least but the wireless range will be the longest.
    High = 2,
}

/// The device side.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Right = 0,
    Left = 1,
}
