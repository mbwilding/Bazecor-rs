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
    PerLayer,
    /// Multi-colored rainbow effect.
    RainbowMulti,
    /// Single-colored rainbow effect.
    RainbowSingle,
    /// All LEDs will be off until pressed, they will light up when pressed and cycle colors back to off.
    Stalker,
    /// All LEDs to red.
    Red,
    /// All LEDs to green.
    Green,
    /// All LEDs to blue.
    Blue,
    /// All LEDs to white.
    White,
    /// All LEDs to off.
    Off,
    /// The inner three LEDs on both sides will be green, the rest will be off.
    GreenInner,
    /// Emulates the bluetooth connect sequence.
    Bluetooth,
}

/// The wireless power mode states.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, NumStrEnum)]
pub enum WirelessPowerMode {
    /// Low power mode. The battery will last longer but the wireless range will be shorter.
    Low,
    /// Medium power mode. The battery will last a bit less but the wireless range will be longer.
    Medium,
    /// High power mode. The battery will last the least but the wireless range will be the longest.
    High,
}
