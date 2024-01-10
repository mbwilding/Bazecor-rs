use anyhow::{bail, Result};
use dygma_focus::hardware::{Device, Product};
use dygma_focus::Focus;
use rayon::prelude::*;
use std::usize;

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

    #[tracing::instrument(skip(self, file_content))]
    pub async fn flash(&mut self, file_content: &str) -> Result<()> {
        let decoded = Self::ihex_decode_lines(file_content)?;

        let mut data_objects = Vec::new();
        let mut total = 0usize;
        let mut segment = 0;
        let mut linear = 0;
        let mut aux_data = Vec::new();

        for mut hex in decoded {
            let hex_length = hex.len as usize * 2;
            match hex.record_type {
                RecordType::Unknown(_) => {}
                RecordType::ESA => {
                    segment = u64::from_str_radix(&hex.str[8..8 + hex_length], 16)? * 16;
                    linear = 0;
                    continue;
                }
                RecordType::ELA => {
                    linear = u64::from_str_radix(&hex.str[8..8 + hex_length], 16)? * 65536;
                    segment = 0;
                    continue;
                }
                RecordType::DAT => {
                    total += hex.len as usize;
                    if segment > 0 {
                        hex.address += segment;
                    }
                    if linear > 0 {
                        hex.address += linear;
                    }
                    aux_data.push(hex.data.clone());
                    data_objects.push(hex);
                }
            }
        }

        let _total_saved = total;
        let _hex_count = 0;
        let _address = &data_objects[0].address;

        Ok(())
    }

    #[tracing::instrument(skip(self, buffer))]
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
        let byte_count = u8::from_str_radix(&line[0..2], 16)?;
        let address = u16::from_str_radix(&line[2..6], 16)?;
        let record_byte = u8::from_str_radix(&line[6..8], 16)?;

        let record_type = match record_byte {
            0x00 => RecordType::DAT,
            0x02 => RecordType::ESA,
            0x04 => RecordType::ELA,
            _ => RecordType::Unknown(record_byte),
        };

        let byte_data = (8..8 + byte_count * 2)
            .step_by(2)
            .map(|i| {
                let i = i as usize;
                u8::from_str_radix(&line[i..i + 2], 16)
            })
            .collect::<Result<Vec<u8>, _>>()?;

        Ok(DecodedHex {
            str: line.to_string(),
            len: byte_count,
            address: address as u64,
            record_type,
            data: byte_data,
        })
    }
}

#[derive(Debug)]
pub struct DecodedHex {
    pub str: String,
    pub len: u8,
    pub address: u64,
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
