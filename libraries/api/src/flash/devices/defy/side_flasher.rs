use crate::firmware_downloader::FirmwareNode;
use anyhow::Result;
use crc32fast::Hasher;
use rayon::prelude::*;

pub fn prepare_chunks(firmware: &FirmwareNode) -> Result<Vec<Vec<u8>>> {
    let data_size = 256;

    let firmware_sides = &firmware.bytes;

    let chunks = firmware_sides
        .par_chunks(data_size)
        .enumerate()
        .map(|(index, data)| {
            // 8 bytes (Write action) + 256 bytes (Data) + 4 bytes (CRC) = 268 byte chunks
            let chunk_size = 8 + data.len() + 4;
            let mut chunk = Vec::with_capacity(chunk_size);

            // Write action (offset, chunk length)
            let offset = (index * data_size) as u32;
            chunk.extend_from_slice(&offset.to_le_bytes());
            chunk.extend_from_slice(&(data.len() as u32).to_le_bytes());

            // Add the data chunk
            chunk.extend_from_slice(data);

            // Calculate and add CRC32 (SIMD calculation)
            let crc = {
                let mut hasher = Hasher::new();
                hasher.update(data);
                hasher.finalize().to_le_bytes()
            };
            chunk.extend_from_slice(&crc);

            chunk
        })
        .collect();

    Ok(chunks)
}
