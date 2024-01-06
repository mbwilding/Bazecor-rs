use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct Hardware {
    pub info: Info,
    pub usb: Usb,
    pub bootloader: bool,
    pub keyboard: Option<Grid>,
    pub keyboard_underglow: Option<Grid>,
    pub rgbw_mode: Option<bool>,
    pub instructions: Languages,
    pub virtual_info: Option<Virtual>,
}

impl Display for Hardware {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.info.display_name)
    }
}

#[derive(Debug)]
pub struct Virtual {
    pub version: VirtualNode,
    pub keymap_custom: VirtualNode,
    pub keymap_default: VirtualNode,
    pub keymap_only_custom: VirtualNode,
    pub settings_default_layer: VirtualNode,
    pub settings_valid: VirtualNode,
    pub settings_version: VirtualNode,
    pub settings_crc: VirtualNode,
    pub eeprom_contents: VirtualNode,
    pub eeprom_free: VirtualNode,
    pub led_at: VirtualNode,
    pub led_set_all: VirtualNode,
    pub led_mode: VirtualNode,
    pub led_fade: Option<VirtualNode>,
    pub led_brightness: VirtualNode,
    pub led_brightness_wireless: Option<VirtualNode>,
    pub led_brightness_ug: VirtualNode,
    pub led_brightness_ug_wireless: Option<VirtualNode>,
    pub led_theme: VirtualNode,
    pub palette: VirtualNode,
    pub colormap_map: VirtualNode,
    pub idle_leds_time_limit: VirtualNode,
    pub idle_leds_wireless: Option<VirtualNode>,
    pub hardware_version: VirtualNode,
    pub hardware_side_power: VirtualNode,
    pub hardware_side_ver: VirtualNode,
    pub hardware_sled_ver: VirtualNode,
    pub hardware_sled_current: VirtualNode,
    pub hardware_layout: VirtualNode,
    pub hardware_joint: VirtualNode,
    pub hardware_keyscan: VirtualNode,
    pub hardware_crc_errors: VirtualNode,
    pub hardware_firmware: VirtualNode,
    pub hardware_chip_id: VirtualNode,
    pub qukeys_hold_timeout: VirtualNode,
    pub qukeys_overlap_threshold: VirtualNode,
    pub superkeys_map: VirtualNode,
    pub superkeys_wait_for: VirtualNode,
    pub superkeys_timeout: VirtualNode,
    pub superkeys_repeat: VirtualNode,
    pub superkeys_hold_start: VirtualNode,
    pub superkeys_overlap: VirtualNode,
    pub macros_map: VirtualNode,
    pub macros_trigger: VirtualNode,
    pub macros_memory: VirtualNode,
    pub help: VirtualNode,
    pub mouse_speed: VirtualNode,
    pub mouse_speed_delay: VirtualNode,
    pub mouse_accel_speed: VirtualNode,
    pub mouse_accel_delay: VirtualNode,
    pub mouse_wheel_speed: VirtualNode,
    pub mouse_wheel_delay: VirtualNode,
    pub mouse_speed_limit: VirtualNode,
    pub layer_activate: VirtualNode,
    pub layer_deactivate: VirtualNode,
    pub layer_is_active: VirtualNode,
    pub layer_move_to: VirtualNode,
    pub layer_state: VirtualNode,
    pub wireless_battery_left_level: Option<VirtualNode>,
    pub wireless_battery_right_level: Option<VirtualNode>,
    pub wireless_battery_left_status: Option<VirtualNode>,
    pub wireless_battery_right_status: Option<VirtualNode>,
    pub wireless_battery_saving_mode: Option<VirtualNode>,
    pub wireless_energy_modes: Option<VirtualNode>,
    pub wireless_energy_disable: Option<VirtualNode>,
    pub wireless_energy_current_mode: Option<VirtualNode>,
    pub wireless_bluetooth_macs: Option<VirtualNode>,
    pub wireless_bluetooth_peer_ids: Option<VirtualNode>,
    pub wireless_bluetooth_remove: Option<VirtualNode>,
    pub wireless_bluetooth_device_name: Option<VirtualNode>,
    pub wireless_bluetooth_list: Option<VirtualNode>,
    pub wireless_rf_power: Option<VirtualNode>,
    pub wireless_rf_stability: Option<VirtualNode>,
    pub wireless_rf_channel_hop: Option<VirtualNode>,
    pub wireless_rf_sync_pairing: Option<VirtualNode>,
}

#[derive(Debug)]
pub struct VirtualNode {
    pub data: &'static str,
    pub erasable: bool,
}

#[derive(Debug)]
pub struct Urls {
    pub homepage: Url,
}

#[derive(Debug)]
pub struct Url {
    pub name: &'static str,
    pub url: &'static str,
}

#[derive(Debug)]
pub struct Info {
    pub vendor: Vendor,
    pub product: Product,
    pub keyboard_type: KeyboardType,
    pub display_name: &'static str,
    pub urls: Urls,
}

#[derive(Debug)]
pub enum Vendor {
    Dygma,
}

#[derive(Debug, PartialEq)]
pub enum Product {
    Defy,
    Raise,
}

impl Display for Product {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Product::Defy => "Defy",
                Product::Raise => "Raise",
            }
        )
    }
}

#[derive(Debug)]
pub enum KeyboardType {
    Wired,
    Wireless,
    ISO,
    ANSI,
}

#[derive(Debug)]
pub struct Usb {
    pub vendor_id: u16,
    pub product_id: u16,
}

#[derive(Debug)]
pub struct Grid {
    pub rows: u8,
    pub columns: u8,
}

#[derive(Debug)]
pub struct Languages {
    pub en: Dialog,
}

#[derive(Debug)]
pub struct Dialog {
    pub update_instructions: &'static str,
}
