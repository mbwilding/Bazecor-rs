use api::devices;
use api::flash::FirmwareRelease;
use api::hardware::Hardware;
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
        devices::DEFY_WIRED,
        devices::DEFY_WIRED_BOOTLOADER,
        devices::DEFY_WIRELESS,
        devices::DEFY_WIRELESS_BOOTLOADER,
        devices::RAISE_ANSI,
        devices::RAISE_ANSI_BOOTLOADER,
        devices::RAISE_ISO,
        devices::RAISE_ISO_BOOTLOADER,
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
