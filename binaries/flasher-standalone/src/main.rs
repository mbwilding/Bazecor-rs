mod prompts;

use crate::prompts::*;
use anyhow::Result;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_ansi(true)
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    let allow_beta = ask_beta();
    let hardware = ask_hardware();
    let firmware_releases = api::flash::load_available_firmware_versions(allow_beta).await?;
    let firmware_release = ask_firmware(firmware_releases, &hardware);
    info!("Release Notes\n{}", &firmware_release.body);

    if let Some(hw) = &hardware {
        let _firmware = api::flash::download_firmware("default", hw, &firmware_release).await?;
        info!("Firmware downloaded");
    } else {
        warn!("Skipping download as no specific hardware was selected");
    }

    Ok(())
}
