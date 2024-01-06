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
