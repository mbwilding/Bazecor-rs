mod logger;
mod prompts;

use crate::prompts::*;
use anyhow::Result;
use clap::Parser;
use dygma_api::flash::devices::defy;
use dygma_focus::prelude::*;
use tracing::{debug, error, info};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long)]
    beta: Option<bool>,
    #[clap(short, long)]
    latest: Option<bool>,
    #[clap(short, long)]
    debug: Option<bool>,
}

#[tokio::main]
async fn main() -> Result<()> {
    logger::init();

    let mut focus = Focus::new_first_available()?;
    let side = Side::Right;

    let resp = focus.upgrade_keyscanner_is_connected(side).await?;
    info!("Upgrade keyscanner is connected: {:?}", resp); // Unused in original code

    let resp = focus.upgrade_keyscanner_is_bootloader(side).await?;
    info!("Upgrade keyscanner is bootloader: {:?}", resp);

    let resp = focus.upgrade_keyscanner_is_ready().await?;
    info!("Upgrade keyscanner is ready: {:?}", resp);

    let resp = focus.upgrade_keyscanner_begin(side).await?;
    info!("Upgrade keyscanner begin: {:?}", resp);

    let resp = focus.upgrade_keyscanner_get_info().await?;
    info!("Upgrade keyscanner get info:\n{:?}", resp);

    let resp = focus.upgrade_keyscanner_finish().await?;
    info!("Upgrade keyscanner finish: {:?}", resp);

    return Ok(());
    //

    let cli = Cli::parse();

    let device = if !cli.debug.unwrap_or(false) {
        let devices = Focus::find_all_devices()?;
        match devices.len() {
            0 => {
                error!("No devices found, please connect a device and try again");
                std::process::exit(1);
            }
            1 => devices[0].clone(),
            _ => ask_connected_device(devices)?,
        }
    } else {
        Device {
            hardware: ask_hardware()?,
            serial_port: "debug".to_string(),
        }
    };

    debug!(
        "Device: {} [{}]",
        &device.hardware.info.display_name, &device.serial_port
    );

    let allow_beta = if let Some(beta) = cli.beta {
        beta
    } else {
        ask_beta()?
    };

    let firmware_releases =
        dygma_api::firmware_downloader::load_available_firmware_versions(allow_beta).await?;

    let use_latest = cli.latest.unwrap_or(false);
    let firmware_release = ask_firmware(firmware_releases, &device.hardware, use_latest)?;
    info!(
        "Release: {} {}\n{}",
        &firmware_release.name, &firmware_release.version, &firmware_release.body
    );
    let firmwares =
        dygma_api::firmware_downloader::download_firmware(&device.hardware, &firmware_release)
            .await?;
    debug!("Firmware downloaded");

    if cli.debug.unwrap_or(false) {
        return Ok(());
    }

    // Testing `Defy flash`
    if let Some(hex_raw) = firmwares.firmware.hex_raw {
        let mut flasher = defy::nrf52833_flasher::Flasher::new(&device)?;
        flasher.flash(&hex_raw).await?;
    }

    Ok(())
}
