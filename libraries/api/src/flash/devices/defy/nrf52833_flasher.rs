use anyhow::Result;
use dygma_focus::enums::*;
use dygma_focus::Focus;
use rayon::prelude::*;
use serde::Serialize;
use std::time::Duration;

const _MAX_MS: u16 = 2000;
const _PACKET_SIZE: u16 = 4096;

pub struct Flasher {
    focus: Focus,
}

#[derive(Debug, Serialize)]
pub struct Settings {
    keymap_custom: String,
    keymap_default: String,
    keymap_only_custom: bool,
    settings_default_layer: u8,
    superkeys_map: String,
    superkeys_wait_for: Duration,
    superkeys_timeout: Duration,
    superkeys_repeat: Duration,
    superkeys_hold_start: Duration,
    superkeys_overlap: u8,
    led_mode: LedMode,
    led_brightness_top: u8,
    led_brightness_underglow: Option<u8>,
    led_brightness_wireless_top: Option<u8>,
    led_brightness_wireless_underglow: Option<u8>,
    led_fade: Option<u16>,
    led_theme: String,
    palette: String,
    color_map: String,
    led_idle_true_sleep: Option<bool>,
    led_idle_true_sleep_time: Option<Duration>,
    led_idle_time_limit: Duration,
    led_idle_wireless: Option<bool>,
    qukeys_hold_timeout: Duration,
    qukeys_overlap_threshold: Duration,
    macros_map: String,
    mouse_speed: u8,
    mouse_delay: Duration,
    mouse_acceleration_speed: u8,
    mouse_acceleration_delay: Duration,
    mouse_wheel_speed: u8,
    mouse_wheel_delay: Duration,
    mouse_speed_limit: u8,
    wireless_battery_saving_mode: Option<bool>,
    wireless_rf_power_level: Option<WirelessPowerMode>,
    wireless_rf_channel_hop: Option<bool>,
}

impl Flasher {
    pub fn new() -> Result<Self> {
        Ok(Self {
            focus: Focus::new_first_available()?, // TODO: Revise passing or doing device check / pass in device
        })
    }

    pub async fn write(&mut self, buffer: &[u8]) -> Result<()> {
        for chunk in buffer.chunks(200) {
            self.focus.dygma_write_bytes(chunk).await?;
        }

        Ok(())
    }

    pub fn ihex_decode_lines(file_content: &str) -> Result<Vec<DecodedHex>> {
        file_content
            .par_lines()
            .map(|line| Self::ihex_decode_line(&line[1..]))
            .collect()
    }

    fn ihex_decode_line(line: &str) -> Result<DecodedHex> {
        let byte_count = usize::from_str_radix(&line[0..2], 16)?;
        let address = u16::from_str_radix(&line[2..6], 16)?;
        let record_byte = u8::from_str_radix(&line[6..8], 16)?;

        let record_type = match record_byte {
            0x00 => RecordType::DAT,
            0x02 => RecordType::ESA,
            0x04 => RecordType::ELA,
            _ => RecordType::Unknown(record_byte),
        };

        let byte_data = (8..line.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&line[i..i + 2], 16))
            .collect::<Result<Vec<u8>, _>>()?;

        Ok(DecodedHex {
            str: line.to_string(),
            len: byte_count,
            address,
            record_type,
            data: byte_data,
        })
    }

    pub async fn settings_backup(&mut self) -> Result<Settings> {
        Ok(Settings {
            keymap_custom: self.focus.keymap_custom_get().await?,
            keymap_default: self.focus.keymap_default_get().await?,
            keymap_only_custom: self.focus.keymap_only_custom_get().await?,
            settings_default_layer: self.focus.settings_default_layer_get().await?,
            superkeys_map: self.focus.superkeys_map_get().await?,
            superkeys_wait_for: self.focus.superkeys_wait_for_get().await?,
            superkeys_timeout: self.focus.superkeys_timeout_get().await?,
            superkeys_repeat: self.focus.superkeys_repeat_get().await?,
            superkeys_hold_start: self.focus.superkeys_hold_start_get().await?,
            superkeys_overlap: self.focus.superkeys_overlap_get().await?,
            led_mode: self.focus.led_mode_get().await?,
            led_brightness_top: self.focus.led_brightness_top_get().await?,
            led_brightness_underglow: self.focus.led_brightness_underglow_get().await.ok(),
            led_brightness_wireless_top: self.focus.led_brightness_wireless_top_get().await.ok(),
            led_brightness_wireless_underglow: self
                .focus
                .led_brightness_wireless_underglow_get()
                .await
                .ok(),
            led_fade: self.focus.led_fade_get().await.ok(),
            led_theme: self.focus.led_theme_get().await?,
            palette: self.focus.palette_get().await?,
            color_map: self.focus.color_map_get().await?,
            led_idle_true_sleep: self.focus.led_idle_true_sleep_get().await.ok(),
            led_idle_true_sleep_time: self.focus.led_idle_true_sleep_time_get().await.ok(),
            led_idle_time_limit: self.focus.led_idle_time_limit_get().await?,
            led_idle_wireless: self.focus.led_idle_wireless_get().await.ok(),
            qukeys_hold_timeout: self.focus.qukeys_hold_timeout_get().await?,
            qukeys_overlap_threshold: self.focus.qukeys_overlap_threshold_get().await?,
            macros_map: self.focus.macros_map_get().await?,
            mouse_speed: self.focus.mouse_speed_get().await?,
            mouse_delay: self.focus.mouse_delay_get().await?,
            mouse_acceleration_speed: self.focus.mouse_acceleration_speed_get().await?,
            mouse_acceleration_delay: self.focus.mouse_acceleration_delay_get().await?,
            mouse_wheel_speed: self.focus.mouse_wheel_speed_get().await?,
            mouse_wheel_delay: self.focus.mouse_wheel_delay_get().await?,
            mouse_speed_limit: self.focus.mouse_speed_limit_get().await?,
            wireless_battery_saving_mode: self.focus.wireless_battery_saving_mode_get().await.ok(),
            wireless_rf_power_level: self.focus.wireless_rf_power_level_get().await.ok(),
            wireless_rf_channel_hop: self.focus.wireless_rf_channel_hop_get().await.ok(),
        })
    }

    pub async fn settings_restore(&mut self, settings: &Settings) -> Result<()> {
        self.focus
            .keymap_custom_set(&settings.keymap_custom)
            .await?;
        self.focus
            .keymap_default_set(&settings.keymap_default)
            .await?;
        self.focus
            .keymap_only_custom_set(settings.keymap_only_custom)
            .await?;
        self.focus
            .settings_default_layer_set(settings.settings_default_layer)
            .await?;
        self.focus
            .superkeys_map_set(&settings.superkeys_map)
            .await?;
        self.focus
            .superkeys_wait_for_set(settings.superkeys_wait_for)
            .await?;
        self.focus
            .superkeys_timeout_set(settings.superkeys_timeout)
            .await?;
        self.focus
            .superkeys_repeat_set(settings.superkeys_repeat)
            .await?;
        self.focus
            .superkeys_hold_start_set(settings.superkeys_hold_start)
            .await?;
        self.focus
            .superkeys_overlap_set(settings.superkeys_overlap)
            .await?;
        self.focus.led_mode_set(settings.led_mode).await?;
        self.focus
            .led_brightness_top_set(settings.led_brightness_top)
            .await?;
        if let Some(led_brightness_underglow) = settings.led_brightness_underglow {
            self.focus
                .led_brightness_underglow_set(led_brightness_underglow)
                .await?;
        }
        if let Some(led_brightness_wireless_top) = settings.led_brightness_wireless_top {
            self.focus
                .led_brightness_wireless_top_set(led_brightness_wireless_top)
                .await?;
        }
        if let Some(led_brightness_wireless_underglow) = settings.led_brightness_wireless_underglow
        {
            self.focus
                .led_brightness_wireless_underglow_set(led_brightness_wireless_underglow)
                .await?;
        }
        if let Some(led_fade) = settings.led_fade {
            self.focus.led_fade_set(led_fade).await?;
        }
        self.focus.led_theme_set(&settings.led_theme).await?;
        self.focus.palette_set(&settings.palette).await?;
        self.focus.color_map_set(&settings.color_map).await?;
        if let Some(led_idle_true_sleep) = settings.led_idle_true_sleep {
            self.focus
                .led_idle_true_sleep_set(led_idle_true_sleep)
                .await?;
        }
        if let Some(led_idle_true_sleep_time) = settings.led_idle_true_sleep_time {
            self.focus
                .led_idle_true_sleep_time_set(led_idle_true_sleep_time)
                .await?;
        }
        self.focus
            .led_idle_time_limit_set(settings.led_idle_time_limit)
            .await?;
        if let Some(led_idle_wireless) = settings.led_idle_wireless {
            self.focus.led_idle_wireless_set(led_idle_wireless).await?;
        }
        self.focus
            .qukeys_hold_timeout_set(settings.qukeys_hold_timeout)
            .await?;
        self.focus
            .qukeys_overlap_threshold_set(settings.qukeys_overlap_threshold)
            .await?;
        self.focus.macros_map_set(&settings.macros_map).await?;
        self.focus.mouse_speed_set(settings.mouse_speed).await?;
        self.focus.mouse_delay_set(settings.mouse_delay).await?;
        self.focus
            .mouse_acceleration_speed_set(settings.mouse_acceleration_speed)
            .await?;
        self.focus
            .mouse_acceleration_delay_set(settings.mouse_acceleration_delay)
            .await?;
        self.focus
            .mouse_wheel_speed_set(settings.mouse_wheel_speed)
            .await?;
        self.focus
            .mouse_wheel_delay_set(settings.mouse_wheel_delay)
            .await?;
        self.focus
            .mouse_speed_limit_set(settings.mouse_speed_limit)
            .await?;
        if let Some(wireless_battery_saving_mode) = settings.wireless_battery_saving_mode {
            self.focus
                .wireless_battery_saving_mode_set(wireless_battery_saving_mode)
                .await?;
        }
        if let Some(wireless_rf_power_level) = settings.wireless_rf_power_level {
            self.focus
                .wireless_rf_power_level_set(wireless_rf_power_level)
                .await?;
        }
        if let Some(wireless_rf_channel_hop) = settings.wireless_rf_channel_hop {
            self.focus
                .wireless_rf_channel_hop_set(wireless_rf_channel_hop)
                .await?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct DecodedHex {
    pub str: String,
    pub len: usize,
    pub address: u16,
    pub record_type: RecordType,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum RecordType {
    Unknown(u8),
    DAT,
    ESA,
    ELA,
}
