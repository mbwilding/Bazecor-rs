use anyhow::{bail, Result};
use dygma_focus::hardware::{Device, Product};
use dygma_focus::Focus;
use log::info;
use rayon::prelude::*;
use std::usize;
use tracing::trace;

const PACKET_SIZE: usize = 4096;

pub struct Flasher {
    focus: Focus,
}

impl Flasher {
    pub fn new(device: &Device) -> Result<Self> {
        if device.hardware.info.product != Product::Defy {
            bail!("Unsupported device");
        } else if device.hardware.bootloader {
            bail!("Device is in bootloader mode");
        }
        Ok(Self {
            focus: Focus::new_via_device(device)?,
        })
    }

    // TODO: Refactor to reduce allocations
    #[tracing::instrument(skip(self, file_content))]
    pub async fn flash(&mut self, file_content: &str) -> Result<()> {
        let decoded = Self::ihex_decode_lines(file_content)?;

        let mut decoded_hexes = Vec::new();
        let mut total = 0;
        let mut segment = 0;
        let mut linear = 0;

        for mut hex in decoded {
            let hex_length = hex.len as usize * 2;
            match hex.record_type {
                RecordType::Unknown(_) => {}
                RecordType::ESA => {
                    segment = u32::from_str_radix(&hex.str[8..8 + hex_length], 16)? * 16;
                    linear = 0;

                    continue;
                }
                RecordType::ELA => {
                    linear = u32::from_str_radix(&hex.str[8..8 + hex_length], 16)? * 65536;
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

                    decoded_hexes.push(hex);
                }
            }
        }

        let mut hex_count = 0;
        let mut address = decoded_hexes[0].address;

        // ERASE device
        let s = format!("E{}#", num_to_hex(address));
        trace!("{}", &s);
        self.write(s.as_bytes()).await?;
        self.focus.read_string().await?;

        while total > 0 {
            let buffer_size = std::cmp::min(total, PACKET_SIZE);

            let mut accumulated_length = 0;
            let start_hex_count = hex_count;
            let decoded_hex_length = decoded_hexes[hex_count].len as usize;
            while hex_count < decoded_hexes.len()
                && accumulated_length + decoded_hex_length <= buffer_size
            {
                accumulated_length += decoded_hex_length;
                hex_count += 1;
            }

            if start_hex_count == hex_count {
                break;
            }

            let data_range = &decoded_hexes[start_hex_count..hex_count];
            for decoded_hex in data_range {
                self.local_write(address, decoded_hex).await?;

                address += decoded_hex.len as u32;
                total -= decoded_hex.len as usize;
            }
        }

        trace!("S#");
        self.write("S#".as_bytes()).await?;

        trace!("Wait for ACK");
        self.focus.read_string().await?;

        info!("Finished flashing");

        Ok(())
    }

    async fn local_write(&mut self, address: u32, decoded_hex: &DecodedHex) -> Result<()> {
        let length_as_hex = num_to_hex(decoded_hex.len as u32);

        let s = format!("U{}#", &length_as_hex);
        trace!("{}", &s);
        self.write(s.as_bytes()).await?;

        trace!("Writing buffer");
        trace!("Writing bytes: {:02X?}", &decoded_hex.data);
        self.write(&decoded_hex.data).await?;

        let s = format!("W{},{}#", num_to_hex(address), &length_as_hex);
        trace!("{}", &s);
        self.write(s.as_bytes()).await?;

        trace!("Wait for ACK");
        self.focus.read_string().await?;

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
            address: address as u32,
            record_type,
            data: byte_data,
        })
    }
}

fn num_to_hex(address: u32) -> String {
    format!("{:08x}", address)
}

#[derive(Debug)]
pub struct DecodedHex {
    pub str: String,
    pub len: u8,
    pub address: u32,
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
