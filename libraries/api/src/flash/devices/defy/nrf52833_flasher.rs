use anyhow::Result;
use dygma_focus::Focus;

const _MAX_MS: u16 = 2000;
const _PACKET_SIZE: u16 = 4096;
const WRITE_SIZE: usize = 200;
const _TYPE_DAT: u8 = 0x00;
const _TYPE_ESA: u8 = 0x02;
const _TYPE_ELA: u8 = 0x04;

pub struct Flasher {
    pub focus: Focus,
}

impl Flasher {
    pub fn new() -> Result<Self> {
        Ok(Self {
            focus: Focus::new_first_available()?, // TODO: Revise passing or doing device check / pass in device
        })
    }

    pub async fn write(&mut self, buffer: Vec<u8>) -> Result<()> {
        let mut total = buffer.len();
        let mut buffer_total = 0;

        while buffer_total < buffer.len() {
            let buffer_size = {
                if buffer_total < WRITE_SIZE {
                    total
                } else {
                    WRITE_SIZE
                }
            };

            let buffer_slice = &buffer[buffer_total..buffer_total + buffer_size];
            self.focus.dygma_write_bytes(buffer_slice).await?;

            buffer_total += buffer_size;
            total -= buffer_size;
        }

        Ok(())
    }
}
