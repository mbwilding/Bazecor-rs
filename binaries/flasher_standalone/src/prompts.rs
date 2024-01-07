use anyhow::Result;
use dygma_api::firmware_downloader::FirmwareRelease;
use dygma_focus::hardware::{Device, Hardware};
use inquire::{Confirm, Select};

pub fn ask_beta() -> Result<bool> {
    Ok(Confirm::new("Allow Beta?")
        .with_default(true)
        .with_help_message("Enables beta releases")
        .prompt()?)
}

pub fn ask_connected_device(options: Vec<Device>) -> Result<Device> {
    Ok(Select::new("Which connected device?", options)
        .with_help_message("Select the connected device")
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
