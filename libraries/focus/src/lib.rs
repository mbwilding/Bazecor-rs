use crate::hardware::Pair;
use anyhow::{anyhow, bail, Result};
use log::{error, trace};
use std::str;
use std::time::Duration;
use tokio_serial::{SerialPort, SerialPortBuilderExt, SerialPortType, SerialStream};

pub mod api;
pub mod color;
pub mod configuration;
pub mod enums;
pub mod hardware;
pub mod prelude;

pub const MAX_LAYERS: u8 = 10 - 1;

/// The Dygma Focus API.
pub struct Focus {
    pub(crate) serial: SerialStream,
    pub(crate) response_buffer: Vec<u8>,
}

/// Constructors
impl Focus {
    /// Find all supported keyboards.
    pub fn find_all_keyboards() -> Result<Vec<Pair>> {
        let ports = match tokio_serial::available_ports() {
            Ok(ports) => ports,
            Err(e) => {
                let err_msg = format!("Failed to enumerate serial ports: {:?}", e);
                error!("{}", err_msg);
                bail!(err_msg)
            }
        };

        trace!("Available serial ports: {:?}", ports);

        let pairs: Vec<Pair> = ports
            .into_iter()
            .filter_map(|port| match &port.port_type {
                SerialPortType::UsbPort(info) => {
                    let matching_devices: Vec<Pair> =
                        hardware::types::hardware_physical::DEVICES_PHYSICAL
                            .iter()
                            .filter_map(|device| {
                                if device.usb.vendor_id == info.vid
                                    && device.usb.product_id == info.pid
                                {
                                    Some(Pair {
                                        hardware: device.to_owned(),
                                        port: port.port_name.to_owned(),
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect();

                    if matching_devices.is_empty() {
                        None
                    } else {
                        Some(matching_devices)
                    }
                }
                _ => None,
            })
            .flatten()
            .collect();

        trace!("Found keyboards: {:?}", pairs);

        Ok(pairs)
    }

    /// Find the first supported keyboard.
    pub fn find_first_keyboard() -> Result<Pair> {
        let devices = match Self::find_all_keyboards() {
            Ok(devices) => devices,
            Err(e) => {
                let err_msg = format!("No device found: {:?}", e);
                error!("{}", err_msg);
                bail!(err_msg)
            }
        };

        let keyboard = devices.into_iter().nth(0).ok_or_else(|| {
            let err_msg = "No supported keyboards found";
            error!("{}", err_msg);
            anyhow!(err_msg)
        })?;

        Ok(keyboard)
    }

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
    pub async fn new_via_hardware(device: &Pair) -> Result<Self> {
        Self::new_via_port(&device.port).await
    }

    /// Creates a new instance of the Focus API, connecting to the keyboard via first available keyboard.
    pub async fn new_first_available() -> Result<Self> {
        Self::new_via_hardware(Self::find_all_keyboards()?.first().ok_or_else(|| {
            let err_msg = "No supported keyboards found";
            error!("{}", err_msg);
            anyhow!(err_msg)
        })?)
        .await
    }
}
