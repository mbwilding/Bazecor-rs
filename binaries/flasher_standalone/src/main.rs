mod logger;
mod prompts;

use crate::prompts::*;
use anyhow::Result;
use clap::Parser;
use dygma_api::flash::devices::defy::side_flasher::prepare_chunks;
use dygma_focus::hardware::Device;
use dygma_focus::Focus;
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
    let firmware =
        dygma_api::firmware_downloader::download_firmware(&device.hardware, &firmware_release)
            .await?;
    debug!("Firmware downloaded successfully");

    if firmware.sides.is_some() {
        let chunks = prepare_chunks(&firmware)?;
        debug!(
            "Firmware side chunks prepared successfully: {} chunks",
            chunks.len()
        );
    }

    if cli.debug.unwrap_or(false) {
        return Ok(());
    }

    // TODO: Flash

    // TODO: This is just testing Focus changes
    let mut focus = Focus::new_via_device(&device)?;
    let _version = focus.version().await?;
    focus.layer_move_to(0).await?;

    Ok(())
}
