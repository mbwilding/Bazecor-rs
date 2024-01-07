use crate::firmware_downloader::Firmware;
use anyhow::{Context, Result};
use crc32fast::Hasher;
use dygma_focus::hardware::Device;
use dygma_focus::Focus;
use log::info;
use rayon::prelude::*;

// TODO: Remove, redundant as it is the same as the Focus API
pub async fn prepare_neuron(device: &Device) -> Result<()> {
    let mut focus = Focus::new_via_device(device).await?;

    info!("Upgrading the Neuron");

    focus.upgrade_neuron().await
    // sleep 10ms
}

// TODO: Just fleshing out the idea of parallel processing chunks
fn prepare_chunks(firmware: &Firmware) -> Result<Vec<Vec<u8>>> {
    let chunk_size = 256;

    let data = firmware.sides.as_deref().context("No firmware sides")?;

    let chunks = data
        .par_chunks(chunk_size)
        .enumerate()
        .map(|(index, chunk)| {
            // 8 bytes for offset and length, 4 bytes for CRC
            let blob_size = 8 + chunk.len() + 4;
            let mut blob = Vec::with_capacity(blob_size);

            // Write action (offset, chunk length)
            let offset = (index * chunk_size) as u32;
            blob.extend_from_slice(&offset.to_le_bytes());
            blob.extend_from_slice(&(chunk.len() as u32).to_le_bytes());

            // Add the data chunk
            blob.extend_from_slice(chunk);

            // Calculate and add CRC
            let crc = {
                let mut hasher = Hasher::new();
                hasher.update(chunk);
                hasher.finalize().to_le_bytes()
            };
            blob.extend_from_slice(&crc);

            blob
        })
        .collect();

    Ok(chunks)
}
