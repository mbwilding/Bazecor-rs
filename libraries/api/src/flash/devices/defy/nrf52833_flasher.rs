use anyhow::{bail, Result};
use dygma_focus::hardware::{Device, Product};
use dygma_focus::Focus;
use rayon::prelude::*;

const _MAX_MS: u16 = 2000;
const _PACKET_SIZE: u16 = 4096;

pub struct Flasher {
    focus: Focus,
}

impl Flasher {
    pub fn new(device: &Device) -> Result<Self> {
        if device.hardware.info.product != Product::Defy {
            bail!("Unsupported device");
        }
        Ok(Self {
            focus: Focus::new_via_device(device)?,
        })
    }

    pub fn flash(&mut self) -> Result<()> {
        Ok(())
    }

    pub async fn write(&mut self, buffer: &[u8]) -> Result<()> {
        for chunk in buffer.chunks(200) {
            self.focus.write_bytes(chunk).await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(file_content))]
    pub fn ihex_decode_lines(file_content: &str) -> Result<Vec<DecodedHex>> {
        file_content
            .par_lines()
            .map(|line| Self::ihex_decode_line(&line[1..]))
            .collect()
    }

    fn ihex_decode_line(line: &str) -> Result<DecodedHex> {
        let byte_count = usize::from_str_radix(&line[0..2], 16)?;
        let address = u16::from_str_radix(&line[2..6], 16)?;
        let record_byte = u8::from_str_radix(&line[6..8], 16)?;

        let record_type = match record_byte {
            0x00 => RecordType::DAT,
            0x02 => RecordType::ESA,
            0x04 => RecordType::ELA,
            _ => RecordType::Unknown(record_byte),
        };

        let byte_data = (8..line.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&line[i..i + 2], 16))
            .collect::<Result<Vec<u8>, _>>()?;

        Ok(DecodedHex {
            str: line.to_string(),
            len: byte_count,
            address,
            record_type,
            data: byte_data,
        })
    }
}

#[derive(Debug)]
pub struct DecodedHex {
    pub str: String,
    pub len: usize,
    pub address: u16,
    pub record_type: RecordType,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum RecordType {
    Unknown(u8),
    DAT,
    ESA,
    ELA,
}
