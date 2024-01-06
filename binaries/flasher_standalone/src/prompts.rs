use api::flash::FirmwareRelease;
use api::hardware::Hardware;
use api::hardware_physical;
use inquire::{Confirm, Select};

pub fn ask_beta() -> bool {
    Confirm::new("Allow Beta?")
        .with_default(true)
        .with_help_message("Enables beta releases")
        .prompt()
        .unwrap()
}

pub fn ask_hardware() -> Hardware {
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

    Select::new("Product?", options)
        .with_help_message("Select the product")
        .prompt()
        .unwrap()
}

pub fn ask_firmware(releases: Vec<FirmwareRelease>, hardware: &Hardware) -> FirmwareRelease {
    let hardware_name = hardware.info.product.to_string();

    let options = releases
        .into_iter()
        .filter(|release| release.name == hardware_name)
        .collect();

    Select::new("Firmware?", options)
        .with_help_message("Select the firmware you want to flash")
        .prompt()
        .unwrap()
}
