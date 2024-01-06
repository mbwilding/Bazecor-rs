mod logger;
mod prompts;

use crate::prompts::*;
use anyhow::Result;
use tracing::{debug, error, info};

#[tokio::main]
async fn main() -> Result<()> {
    logger::init();

    let pairs = dygma_focus::Focus::find_all_keyboards()?;
    let pair = match pairs.len() {
        0 => {
            error!("No devices found, please connect your device and try again");
            std::process::exit(1);
        }
        1 => pairs[0].clone(),
        _ => ask_connected_device(pairs)?,
    };

    debug!(
        "Device: {} | {}",
        &pair.hardware.info.display_name, &pair.serial_port
    );

    let allow_beta = ask_beta()?;
    let firmware_releases = api::flash::load_available_firmware_versions(allow_beta).await?;
    let firmware_release = ask_firmware(firmware_releases, &pair.hardware)?;
    info!("Release Notes\n{}", &firmware_release.body);
    let _firmware =
        api::flash::download_firmware("default", &pair.hardware, &firmware_release).await?;
    debug!("Firmware downloaded successfully");

    Ok(())
}
