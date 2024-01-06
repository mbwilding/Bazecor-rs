use anyhow::Result;
use api::flash::FirmwareRelease;
use api::hardware::Hardware;
use inquire::{Confirm, Select};
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
    info!("Selected Release: {:#?}", &firmware_release);

    if let Some(hw) = &hardware {
        let _firmware = api::flash::download_firmware("default", hw, &firmware_release).await?;
        info!("Firmware downloaded");
    } else {
        warn!("Skipping download as no specific hardware was selected");
    }

    Ok(())
}

pub fn ask_beta() -> bool {
    Confirm::new("Allow Beta?")
        .with_default(true)
        .with_help_message("Enables beta releases")
        .prompt()
        .unwrap()
}

pub fn ask_hardware() -> Option<Hardware> {
    let options = vec![
        "All",
        "Defy Wired",
        "Defy Wired Bootloader",
        "Defy Wireless",
        "Defy Wireless Bootloader",
        "Raise ANSI",
        "Raise ANSI Bootloader",
        "Raise ISO",
        "Raise ISO Bootloader",
    ];

    let hardware = Select::new("Product?", options)
        .with_help_message("Select the product")
        .prompt()
        .unwrap();

    match hardware {
        "Defy Wired" => Some(api::devices::DEFY_WIRED),
        "Defy Wired Bootloader" => Some(api::devices::DEFY_WIRED_BOOTLOADER),
        "Defy Wireless" => Some(api::devices::DEFY_WIRELESS),
        "Defy Wireless Bootloader" => Some(api::devices::DEFY_WIRED_BOOTLOADER),
        "Raise ANSI" => Some(api::devices::RAISE_ANSI),
        "Raise ANSI Bootloader" => Some(api::devices::RAISE_ANSI_BOOTLOADER),
        "Raise ISO" => Some(api::devices::RAISE_ISO),
        "Raise ISO Bootloader" => Some(api::devices::RAISE_ISO_BOOTLOADER),
        _ => None,
    }
}

pub fn ask_firmware(
    releases: Vec<FirmwareRelease>,
    hardware: &Option<Hardware>,
) -> FirmwareRelease {
    let releases = match hardware {
        None => releases,
        Some(product) => {
            let hardware_name = product.info.product.to_string();
            releases
                .into_iter()
                .filter(|release| release.name == hardware_name)
                .collect()
        }
    };

    Select::new("Firmware?", releases)
        .with_help_message("Select the firmware you want to flash")
        .prompt()
        .unwrap()
}
