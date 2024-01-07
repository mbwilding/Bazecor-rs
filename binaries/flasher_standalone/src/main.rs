mod logger;
mod prompts;

use crate::prompts::*;
use anyhow::Result;
use tracing::{debug, error, info};

#[tokio::main]
async fn main() -> Result<()> {
    logger::init();

    let devices = dygma_focus::Focus::find_all_devices()?;
    let device = match devices.len() {
        0 => {
            error!("No devices found, please connect a device and try again");
            std::process::exit(1);
        }
        1 => devices[0].clone(),
        _ => ask_connected_device(devices)?,
    };

    debug!(
        "Device: {} | {}",
        &device.hardware.info.display_name, &device.serial_port
    );

    let allow_beta = ask_beta()?;
    let firmware_releases =
        dygma_api::firmware_downloader::load_available_firmware_versions(allow_beta).await?;
    let firmware_release = ask_firmware(firmware_releases, &device.hardware)?;
    info!("Release Notes\n{}", &firmware_release.body);
    let _firmware = dygma_api::firmware_downloader::download_firmware(
        "default",
        &device.hardware,
        &firmware_release,
    )
    .await?;
    debug!("Firmware downloaded successfully");

    Ok(())
}
