use anyhow::Result;
use api::flash::FirmwareRelease;
use api::hardware::Hardware;
use api::hardware_physical;
use inquire::{Confirm, Select};

pub fn ask_beta() -> Result<bool> {
    Ok(Confirm::new("Allow Beta?")
        .with_default(true)
        .with_help_message("Enables beta releases")
        .prompt()?)
}

pub fn ask_hardware() -> Result<Hardware> {
    let options = vec![
        hardware_physical::DEFY_WIRED,
        hardware_physical::DEFY_WIRED_BOOTLOADER,
        hardware_physical::DEFY_WIRELESS,
        hardware_physical::DEFY_WIRELESS_BOOTLOADER,
        hardware_physical::RAISE_ANSI,
        hardware_physical::RAISE_ANSI_BOOTLOADER,
        hardware_physical::RAISE_ISO,
        hardware_physical::RAISE_ISO_BOOTLOADER,
    ];

    Ok(Select::new("Product?", options)
        .with_help_message("Select the product")
        .prompt()?)
}

pub fn ask_firmware(
    releases: Vec<FirmwareRelease>,
    hardware: &Hardware,
) -> Result<FirmwareRelease> {
    let hardware_name = hardware.info.product.to_string();

    let options = releases
        .into_iter()
        .filter(|release| release.name == hardware_name)
        .collect();

    Ok(Select::new("Firmware?", options)
        .with_help_message("Select the firmware you want to flash")
        .prompt()?)
}
