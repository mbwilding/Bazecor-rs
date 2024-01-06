mod logger;
mod prompts;

use crate::prompts::*;
use anyhow::Result;
use tracing::debug;

#[tokio::main]
async fn main() -> Result<()> {
    logger::init();

    let allow_beta = ask_beta()?;
    let hardware = ask_hardware()?;
    let firmware_releases = api::flash::load_available_firmware_versions(allow_beta).await?;
    let firmware_release = ask_firmware(firmware_releases, &hardware)?;
    debug!("Release Notes\n{}", &firmware_release.body);
    let _firmware = api::flash::download_firmware("default", &hardware, &firmware_release).await?;
    debug!("Firmware downloaded successfully");

    Ok(())
}
