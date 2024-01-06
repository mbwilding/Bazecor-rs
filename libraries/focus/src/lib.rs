use crate::keyboards::Keyboard;
use anyhow::{anyhow, Result};
use log::error;
use std::str;
use std::time::Duration;
use tokio_serial::{SerialPort, SerialPortBuilderExt, SerialStream};

pub mod api;
pub mod color;
pub mod configuration;
pub mod enums;
pub mod keyboards;
pub mod prelude;

pub const MAX_LAYERS: u8 = 10 - 1;

/// The Dygma Focus API.
pub struct Focus {
    pub(crate) serial: SerialStream,
    pub(crate) response_buffer: Vec<u8>,
}

/// Constructors
impl Focus {
    /// Creates a new instance of the Focus API, connecting to the keyboard via port.
    pub async fn new_via_port(port: &str) -> Result<Self> {
        let port_settings = tokio_serial::new(port, 115_200)
            .data_bits(tokio_serial::DataBits::Eight)
            .flow_control(tokio_serial::FlowControl::None)
            .parity(tokio_serial::Parity::None)
            .stop_bits(tokio_serial::StopBits::One)
            .timeout(Duration::from_secs(5));

        let mut serial = port_settings.open_native_async().map_err(|e| {
            let err_msg = format!("Failed to open serial port: {} ({:?})", &port, e);
            error!("{}", err_msg);
            anyhow!(err_msg)
        })?;

        serial.write_data_terminal_ready(true)?;

        #[cfg(unix)]
        serial
            .set_exclusive(false)
            .expect("Unable to set serial port exclusive to false");

        Ok(Self {
            serial,
            response_buffer: Vec::with_capacity(4096),
        })
    }

    /// Creates a new instance of the Focus API, connecting to the keyboard via keyboard struct.
    pub async fn new_via_keyboard(device: &Keyboard) -> Result<Self> {
        Self::new_via_port(&device.port).await
    }

    /// Creates a new instance of the Focus API, connecting to the keyboard via first available keyboard.
    pub async fn new_first_available() -> Result<Self> {
        Self::new_via_keyboard(Keyboard::find_all_keyboards()?.first().ok_or_else(|| {
            let err_msg = "No supported keyboards found";
            error!("{}", err_msg);
            anyhow!(err_msg)
        })?)
        .await
    }
}
