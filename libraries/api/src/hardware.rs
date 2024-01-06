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
