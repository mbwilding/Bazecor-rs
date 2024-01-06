use crate::keyboards::Keyboard;
use anyhow::{anyhow, Result};
use bytes::BytesMut;
use log::error;
use std::time::Duration;
use std::{io, str};
use tokio_serial::{SerialPort, SerialPortBuilderExt, SerialStream};
use tokio_util::codec::{Decoder, Encoder, Framed};

pub mod api;
pub mod color;
pub mod configuration;
pub mod enums;
pub mod keyboards;
pub mod prelude;

pub const MAX_LAYERS: u8 = 10 - 1;

/// The Dygma Focus API.
pub struct Focus {
    pub(crate) serial: Framed<SerialStream, LineCodec>,
}

struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(newline) = src
            .as_ref()
            .windows(5)
            .position(|bytes| bytes == [b'\r', b'\n', b'.', b'\r', b'\n'])
        {
            let line = src.split_to(newline + 2);
            return match str::from_utf8(line.as_ref()) {
                Ok(s) => Ok(Some(s.trim_end_matches("\r\n").to_string())),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid String")),
            };
        }
        Ok(None)
    }
}

impl Encoder<String> for LineCodec {
    type Error = io::Error;

    fn encode(&mut self, _item: String, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
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

        let serial = LineCodec.framed(serial);

        Ok(Self { serial })
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
