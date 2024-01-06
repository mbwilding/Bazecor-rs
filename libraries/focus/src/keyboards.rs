use anyhow::{anyhow, bail, Result};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use tokio_serial::SerialPortType;

/// Supported device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportedDevice {
    /// The name of the device.
    pub name: &'static str,
    /// The vendor ID of the device.
    pub vendor_id: u16,
    /// The product ID of the device.
    pub product_id: u16,
}

impl SupportedDevice {
    pub const fn new(name: &'static str, vendor_id: u16, product_id: u16) -> Self {
        SupportedDevice {
            name,
            vendor_id,
            product_id,
        }
    }
}

pub const DEVICES: [SupportedDevice; 4] = [
    SupportedDevice::new("Dygma Defy Wired", 0x35ef, 0x0010),
    SupportedDevice::new("Dygma Defy Wireless", 0x35ef, 0x0012),
    SupportedDevice::new("Dygma Raise ANSI", 0x1209, 0x2201),
    SupportedDevice::new("Dygma Raise ISO", 0x1209, 0x2201),
];

/// Dygma keyboard information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Keyboard {
    /// The name of the keyboard.
    pub name: &'static str,
    /// The port of the keyboard.
    pub port: String,
}

impl Keyboard {
    /// Find all supported keyboards.
    pub fn find_all_keyboards() -> Result<Vec<Keyboard>> {
        let ports = match tokio_serial::available_ports() {
            Ok(ports) => ports,
            Err(e) => {
                let err_msg = format!("Failed to enumerate serial ports: {:?}", e);
                error!("{}", err_msg);
                bail!(err_msg)
            }
        };

        debug!("Available serial ports: {:?}", ports);

        let keyboards: Vec<Keyboard> = ports
            .into_iter()
            .filter_map(|port| match &port.port_type {
                SerialPortType::UsbPort(info) => DEVICES
                    .iter()
                    .find(|&device| device.vendor_id == info.vid && device.product_id == info.pid)
                    .map(|device| Keyboard {
                        name: device.name,
                        port: port.port_name,
                    }),
                _ => None,
            })
            .collect();

        debug!("Found keyboards: {:?}", keyboards);

        Ok(keyboards)
    }

    /// Find the first supported keyboard.
    pub fn find_first_keyboard() -> Result<Keyboard> {
        let devices = match Self::find_all_keyboards() {
            Ok(devices) => devices,
            Err(e) => {
                let err_msg = format!("No device found: {:?}", e);
                error!("{}", err_msg);
                bail!(err_msg)
            }
        };

        let keyboard = devices.first().ok_or_else(|| {
            let err_msg = "No supported keyboards found";
            error!("{}", err_msg);
            anyhow!(err_msg)
        })?;

        Ok(keyboard.to_owned())
    }
}
