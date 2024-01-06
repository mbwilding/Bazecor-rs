use crate::hardware::*;

pub const DEFY_WIRED_VIRTUAL: Hardware = {
    Hardware {
        info: Info {
            vendor: Vendor::Dygma,
            product: Product::Defy,
            keyboard_type: KeyboardType::Wired,
            display_name: "Dygma Defy Wired",
            urls: Urls {
                homepage: Url {
                    name: "Homepage",
                    url: "https://www.dygma.com/defy/",
                },
            },
        },
        usb: Usb {
            vendor_id: 13807,
            product_id: 16,
        },
        bootloader: false,
        keyboard: Some(Grid {
            rows: 5,
            columns: 16,
        }),
        keyboard_underglow: Some(Grid {
            rows: 2,
            columns: 89,
        }),
        rgbw_mode: Some(true),
        instructions: Languages {
            en: Dialog {
                update_instructions: "To update the firmware, the keyboard needs a special reset. When the countdown starts, press and hold the Escape key. Soon after the countdown finished, the Neuron's light should start a blue pulsing pattern, and the flashing will proceed. At this point, you should release the Escape key.",
            },
        },
        virtual_info: Some(Virtual {}),
    }
};

pub const DEFY_WIRELESS_VIRTUAL: Hardware = {
    Hardware {
        info: Info {
            vendor: Vendor::Dygma,
            product: Product::Defy,
            keyboard_type: KeyboardType::Wireless,
            display_name: "Dygma Defy Wireless",
            urls: Urls {
                homepage: Url {
                    name: "Homepage",
                    url: "https://www.dygma.com/defy/",
                },
            },
        },
        usb: Usb {
            vendor_id: 13807,
            product_id: 18,
        },
        bootloader: false,
        keyboard: Some(Grid {
            rows: 5,
            columns: 16,
        }),
        keyboard_underglow: Some(Grid {
            rows: 2,
            columns: 89,
        }),
        rgbw_mode: Some(true),
        instructions: Languages {
            en: Dialog {
                update_instructions: "To update the firmware, the keyboard needs a special reset. When the countdown starts, press and hold the Escape key. Soon after the countdown finished, the Neuron's light should start a blue pulsing pattern, and the flashing will proceed. At this point, you should release the Escape key.",
            },
        },
        virtual_info: Some(Virtual {}),
    }
};

pub const RAISE_ANSI_VIRTUAL: Hardware = {
    Hardware {
        info: Info {
            vendor: Vendor::Dygma,
            product: Product::Raise,
            keyboard_type: KeyboardType::ANSI,
            display_name: "Dygma Raise ANSI",
            urls: Urls {
                homepage: Url {
                    name: "Homepage",
                    url: "https://www.dygma.com/raise/",
                },
            },
        },
        usb: Usb {
            vendor_id: 4617,
            product_id: 8705,
        },
        bootloader: false,
        keyboard: Some(Grid {
            rows: 5,
            columns: 16,
        }),
        keyboard_underglow: Some(Grid {
            rows: 6,
            columns: 22,
        }),
        rgbw_mode: Some(true),
        instructions: Languages {
            en: Dialog {
                update_instructions: "To update the firmware, the keyboard needs a special reset. When the countdown starts, press and hold the Escape key. Soon after the countdown finished, the Neuron's light should start a blue pulsing pattern, and the flashing will proceed. At this point, you should release the Escape key.",
            },
        },
        virtual_info: Some(Virtual {}),
    }
};

pub const RAISE_ISO_VIRTUAL: Hardware = {
    Hardware {
        info: Info {
            vendor: Vendor::Dygma,
            product: Product::Raise,
            keyboard_type: KeyboardType::ISO,
            display_name: "Dygma Raise ISO",
            urls: Urls {
                homepage: Url {
                    name: "Homepage",
                    url: "https://www.dygma.com/raise/",
                },
            },
        },
        usb: Usb {
            vendor_id: 4617,
            product_id: 8705,
        },
        bootloader: false,
        keyboard: Some(Grid {
            rows: 5,
            columns: 16,
        }),
        keyboard_underglow: Some(Grid {
            rows: 6,
            columns: 22,
        }),
        rgbw_mode: Some(true),
        instructions: Languages {
            en: Dialog {
                update_instructions: "To update the firmware, the keyboard needs a special reset. When the countdown starts, press and hold the Escape key. Soon after the countdown finished, the Neuron's light should start a blue pulsing pattern, and the flashing will proceed. At this point, you should release the Escape key.",
            },
        },
        virtual_info: Some(Virtual {}),
    }
};
