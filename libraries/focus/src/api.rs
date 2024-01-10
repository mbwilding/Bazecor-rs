use crate::helpers::*;
use crate::prelude::*;
use crate::{Focus, MAX_LAYERS};
use anyhow::{anyhow, bail, Result};
use std::str::FromStr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::trace;

/// Public methods
impl Focus {
    /// Writes bytes to the serial port.
    pub async fn write(&mut self, bytes: &[u8]) -> Result<()> {
        trace!("Writing bytes: {:02X?}", bytes);
        let mut stream = self.stream.lock().await;
        stream.write_all(bytes).await?;
        stream.flush().await?;

        Ok(())
    }

    /// Response from serial port
    pub async fn read(&mut self) -> Result<String> {
        let eof_marker = b"\r\n.\r\n";

        self.response_buffer.clear();

        loop {
            let prev_len = self.response_buffer.len();
            self.response_buffer.resize(prev_len + 1024, 0);

            let mut stream = self.stream.lock().await;

            match stream.read(&mut self.response_buffer[prev_len..]).await {
                Ok(0) => continue,
                Ok(size) => {
                    self.response_buffer.truncate(prev_len + size);
                    self.response_buffer.retain(|&x| x != 0);

                    trace!("Received bytes: {:02X?}", &self.response_buffer[..size]);

                    if self.response_buffer.ends_with(eof_marker) {
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => bail!("Error reading from serial port: {:?}", e),
            }
        }

        while let Some(pos) = self
            .response_buffer
            .windows(eof_marker.len())
            .position(|window| window == eof_marker)
        {
            self.response_buffer.drain(pos..pos + eof_marker.len());
        }

        let start = self
            .response_buffer
            .iter()
            .position(|&b| !b.is_ascii_whitespace())
            .unwrap_or(0);

        let end = self
            .response_buffer
            .iter()
            .rposition(|&b| !b.is_ascii_whitespace())
            .map_or(0, |p| p + 1);

        let trimmed_buffer = &self.response_buffer[start..end];

        let response = std::str::from_utf8(trimmed_buffer)
            .map_err(|e| anyhow!("Failed to convert response to UTF-8 string: {:?}", e))?;

        if !response.is_empty() {
            trace!("Command RX: {}", &response);
        } else {
            trace!("Command RX: [Ack]");
        }

        Ok(response.to_string())
    }

    /// Gets the settings from the device.
    #[tracing::instrument(skip(self))]
    pub async fn settings_get(&mut self) -> Result<Settings> {
        Ok(Settings {
            keymap_custom: self.keymap_custom_get().await?,
            keymap_default: self.keymap_default_get().await?,
            keymap_only_custom: self.keymap_only_custom_get().await?,
            settings_default_layer: self.settings_default_layer_get().await?,
            superkeys_map: self.superkeys_map_get().await?,
            superkeys_wait_for: self.superkeys_wait_for_get().await?,
            superkeys_timeout: self.superkeys_timeout_get().await?,
            superkeys_repeat: self.superkeys_repeat_get().await?,
            superkeys_hold_start: self.superkeys_hold_start_get().await?,
            superkeys_overlap: self.superkeys_overlap_get().await?,
            led_mode: self.led_mode_get().await?,
            led_brightness_top: self.led_brightness_top_get().await?,
            led_brightness_underglow: self.led_brightness_underglow_get().await.ok(),
            led_brightness_wireless_top: self.led_brightness_wireless_top_get().await.ok(),
            led_brightness_wireless_underglow: self
                .led_brightness_wireless_underglow_get()
                .await
                .ok(),
            led_fade: self.led_fade_get().await.ok(),
            led_theme: self.led_theme_get().await?,
            palette_rgb: self.palette_rgb_get().await.ok(),
            palette_rgbw: self.palette_rgbw_get().await.ok(),
            color_map: self.color_map_get().await?,
            led_idle_true_sleep: self.led_idle_true_sleep_get().await.ok(),
            led_idle_true_sleep_time: self.led_idle_true_sleep_time_get().await.ok(),
            led_idle_time_limit: self.led_idle_time_limit_get().await?,
            led_idle_wireless: self.led_idle_wireless_get().await.ok(),
            qukeys_hold_timeout: self.qukeys_hold_timeout_get().await?,
            qukeys_overlap_threshold: self.qukeys_overlap_threshold_get().await?,
            macros_map: self.macros_map_get().await?,
            mouse_speed: self.mouse_speed_get().await?,
            mouse_delay: self.mouse_delay_get().await?,
            mouse_acceleration_speed: self.mouse_acceleration_speed_get().await?,
            mouse_acceleration_delay: self.mouse_acceleration_delay_get().await?,
            mouse_wheel_speed: self.mouse_wheel_speed_get().await?,
            mouse_wheel_delay: self.mouse_wheel_delay_get().await?,
            mouse_speed_limit: self.mouse_speed_limit_get().await?,
            wireless_battery_saving_mode: self.wireless_battery_saving_mode_get().await.ok(),
            wireless_rf_power_level: self.wireless_rf_power_level_get().await.ok(),
            wireless_rf_channel_hop: self.wireless_rf_channel_hop_get().await.ok(),
        })
    }

    /// Sets the settings for the device.
    #[tracing::instrument(skip(self, settings))]
    pub async fn settings_set(&mut self, settings: &Settings) -> Result<()> {
        self.keymap_custom_set(&settings.keymap_custom).await?;
        self.keymap_default_set(&settings.keymap_default).await?;
        self.keymap_only_custom_set(settings.keymap_only_custom)
            .await?;
        self.settings_default_layer_set(settings.settings_default_layer)
            .await?;
        self.superkeys_map_set(&settings.superkeys_map).await?;
        self.superkeys_wait_for_set(settings.superkeys_wait_for)
            .await?;
        self.superkeys_timeout_set(settings.superkeys_timeout)
            .await?;
        self.superkeys_repeat_set(settings.superkeys_repeat).await?;
        self.superkeys_hold_start_set(settings.superkeys_hold_start)
            .await?;
        self.superkeys_overlap_set(settings.superkeys_overlap)
            .await?;
        self.led_mode_set(settings.led_mode).await?;
        self.led_brightness_top_set(settings.led_brightness_top)
            .await?;
        if let Some(led_brightness_underglow) = settings.led_brightness_underglow {
            self.led_brightness_underglow_set(led_brightness_underglow)
                .await?;
        }
        if let Some(led_brightness_wireless_top) = settings.led_brightness_wireless_top {
            self.led_brightness_wireless_top_set(led_brightness_wireless_top)
                .await?;
        }
        if let Some(led_brightness_wireless_underglow) = settings.led_brightness_wireless_underglow
        {
            self.led_brightness_wireless_underglow_set(led_brightness_wireless_underglow)
                .await?;
        }
        if let Some(led_fade) = settings.led_fade {
            self.led_fade_set(led_fade).await?;
        }
        self.led_theme_set(&settings.led_theme).await?;
        if let Some(palette) = &settings.palette_rgb {
            self.palette_rgb_set(palette).await?;
        }
        if let Some(palette) = &settings.palette_rgbw {
            self.palette_rgbw_set(palette).await?;
        }
        self.color_map_set(&settings.color_map).await?;
        if let Some(led_idle_true_sleep) = settings.led_idle_true_sleep {
            self.led_idle_true_sleep_set(led_idle_true_sleep).await?;
        }
        if let Some(led_idle_true_sleep_time) = settings.led_idle_true_sleep_time {
            self.led_idle_true_sleep_time_set(led_idle_true_sleep_time)
                .await?;
        }
        self.led_idle_time_limit_set(settings.led_idle_time_limit)
            .await?;
        if let Some(led_idle_wireless) = settings.led_idle_wireless {
            self.led_idle_wireless_set(led_idle_wireless).await?;
        }
        self.qukeys_hold_timeout_set(settings.qukeys_hold_timeout)
            .await?;
        self.qukeys_overlap_threshold_set(settings.qukeys_overlap_threshold)
            .await?;
        self.macros_map_set(&settings.macros_map).await?;
        self.mouse_speed_set(settings.mouse_speed).await?;
        self.mouse_delay_set(settings.mouse_delay).await?;
        self.mouse_acceleration_speed_set(settings.mouse_acceleration_speed)
            .await?;
        self.mouse_acceleration_delay_set(settings.mouse_acceleration_delay)
            .await?;
        self.mouse_wheel_speed_set(settings.mouse_wheel_speed)
            .await?;
        self.mouse_wheel_delay_set(settings.mouse_wheel_delay)
            .await?;
        self.mouse_speed_limit_set(settings.mouse_speed_limit)
            .await?;
        if let Some(wireless_battery_saving_mode) = settings.wireless_battery_saving_mode {
            self.wireless_battery_saving_mode_set(wireless_battery_saving_mode)
                .await?;
        }
        if let Some(wireless_rf_power_level) = settings.wireless_rf_power_level {
            self.wireless_rf_power_level_set(wireless_rf_power_level)
                .await?;
        }
        if let Some(wireless_rf_channel_hop) = settings.wireless_rf_channel_hop {
            self.wireless_rf_channel_hop_set(wireless_rf_channel_hop)
                .await?;
        }

        Ok(())
    }
}

/// Private methods
impl Focus {
    /// Sends a command to the device.
    async fn command_raw(
        &mut self,
        command: &str,
        suffix: Option<char>,
        wait_for_response: bool,
    ) -> Result<()> {
        trace!("Command TX: {}", command);

        if let Some(char) = suffix {
            self.write(format!("{}{}", command, char).as_bytes())
                .await?;
        } else {
            self.write(command.as_bytes()).await?;
        }

        if wait_for_response {
            let _response = self.read().await?;
            // It's not necessary to do anything with the response, but we need to wait for it.
        }

        Ok(())
    }

    /// Sends a command to the device, with a single new line ending.
    async fn command_new_line(&mut self, command: &str, wait_for_response: bool) -> Result<()> {
        self.command_raw(command, Some('\n'), wait_for_response)
            .await
    }

    /// Sends a command to the device, with a single whitespace ending.
    async fn command_whitespace(&mut self, command: &str) -> Result<()> {
        self.command_raw(command, Some(' '), false).await
    }

    /// Sends a command to the device, and returns the response as a string.
    async fn command_response_string(&mut self, command: &str) -> Result<String> {
        self.command_new_line(command, false).await?;

        self.read().await
    }

    /// Sends a command to the device, and returns the response as a numerical value.
    async fn command_response_numerical<T>(&mut self, command: &str) -> Result<T>
    where
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let response = self.command_response_string(command).await?;
        response
            .parse::<T>()
            .map_err(|e| anyhow!("Failed to parse response: {:?}", e))
    }

    /// Sends a command to the device, and returns the response as a duration.
    async fn command_response_duration(
        &mut self,
        command: &str,
        time_unit: TimeUnit,
    ) -> Result<Duration> {
        let response = self.command_response_numerical(command).await?;

        let duration = match time_unit {
            TimeUnit::Milliseconds => Duration::from_millis(response),
            TimeUnit::Seconds => Duration::from_secs(response),
        };

        Ok(duration)
    }

    /// Sends a command to the device, and returns the response as a boolean value.
    async fn command_response_bool(&mut self, command: &str) -> Result<bool> {
        let response = self.command_response_string(command).await?;
        if response.is_empty() {
            bail!("Cannot parse bool: Empty response");
        } else if response == "0" || response == "false" {
            Ok(false)
        } else if response == "1" || response == "true" {
            Ok(true)
        } else {
            bail!("Cannot parse bool: {}", response);
        }
    }

    /// Sends a command to the device, and returns the response as a vector of strings.
    async fn command_response_vec_string(&mut self, command: &str) -> Result<Vec<String>> {
        Ok(self
            .command_response_string(command)
            .await?
            .lines()
            .map(|line| line.replace('\r', ""))
            .collect())
    }
}

/// Public API methods
impl Focus {
    /// Get the version of the firmware.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#version
    pub async fn version(&mut self) -> Result<String> {
        self.command_response_string("version").await
    }

    /// Gets the whole custom keymap stored in the keyboard.
    ///
    /// Layers 0 and above, The layers are -1 to Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymapcustom
    pub async fn keymap_custom_get(&mut self) -> Result<Vec<u16>> {
        let data = self.command_response_string("keymap.custom").await?;

        string_to_numerical_vec(&data)
    }

    /// Sets the whole custom keymap stored in the keyboard.
    ///
    /// Layers 0 and above, The layers are -1 to Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymapcustom
    pub async fn keymap_custom_set(&mut self, data: &[u16]) -> Result<()> {
        if self.keymap_custom_get().await? == data {
            return Ok(());
        }

        self.command_new_line(
            &format!("keymap.custom {}", numerical_vec_to_string(data)),
            true,
        )
        .await
    }

    /// Gets the default keymap stored in the keyboard.
    ///
    /// Layers -1 and -2, the layers are -1 to Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymapdefault
    pub async fn keymap_default_get(&mut self) -> Result<Vec<u16>> {
        let data = self.command_response_string("keymap.default").await?;

        string_to_numerical_vec(&data)
    }

    /// Sets the default keymap stored in the keyboard.
    ///
    /// Layers -1 and -2, the layers are -1 to Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymapdefault
    pub async fn keymap_default_set(&mut self, data: &[u16]) -> Result<()> {
        if self.keymap_default_get().await? == data {
            return Ok(());
        }

        self.command_new_line(
            &format!("keymap.default {}", numerical_vec_to_string(data)),
            true,
        )
        .await
    }

    /// Gets the user setting of hiding the default layers.
    ///
    /// It does not allow you to increment the number of available layers by start using the default ones.
    /// They are there so you can store a backup for two layers in your keyboard.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymaponlycustom
    pub async fn keymap_only_custom_get(&mut self) -> Result<bool> {
        self.command_response_bool("keymap.onlyCustom").await
    }

    /// Sets the user setting of hiding the default layers.
    ///
    /// It does not allow you to increment the number of available layers by start using the default ones.
    /// They are there so you can store a backup for two layers in your keyboard.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymaponlycustom
    pub async fn keymap_only_custom_set(&mut self, state: bool) -> Result<()> {
        if self.keymap_only_custom_get().await? == state {
            return Ok(());
        }

        self.command_new_line(&format!("keymap.onlyCustom {}", state as u8), true)
            .await
    }

    /// Gets the default layer the keyboard will boot with.
    ///
    /// The layer is -1 to Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#settingsdefaultlayer
    pub async fn settings_default_layer_get(&mut self) -> Result<u8> {
        self.command_response_numerical("settings.defaultLayer")
            .await
    }

    /// Sets the default layer the keyboard will boot with.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#settingsdefaultlayer
    pub async fn settings_default_layer_set(&mut self, layer: u8) -> Result<()> {
        if layer > MAX_LAYERS {
            bail!("Layer out of range, max is {}: {}", MAX_LAYERS, layer);
        }

        if self.settings_default_layer_get().await? == layer {
            return Ok(());
        }

        self.command_new_line(&format!("settings.defaultLayer {}", layer), true)
            .await
    }

    /// Gets a boolean value that states true if all checks have been performed on the current settings, and its upload was done in the intended way.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#settingsvalid
    pub async fn settings_valid(&mut self) -> Result<bool> {
        self.command_response_numerical("settings.valid?").await
    }

    /// Gets the current settings version.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#settingsversion
    pub async fn settings_version_get(&mut self) -> Result<String> {
        self.command_response_string("settings.version").await
    }

    /// Sets the current settings version.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#settingsversion
    pub async fn settings_version_set(&mut self, version: &str) -> Result<()> {
        if self.settings_version_get().await? == version {
            return Ok(());
        }

        self.command_new_line(&format!("settings.version {}", version), true)
            .await
    }

    /// Gets the CRC checksum of the layout.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#settingscrc
    pub async fn settings_crc(&mut self) -> Result<String> {
        self.command_response_string("settings.crc").await
    }

    /// Gets the EEPROM's contents.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#eepromcontents
    pub async fn eeprom_contents_get(&mut self) -> Result<String> {
        self.command_response_string("eeprom.contents").await
    }

    /// Sets the EEPROM's contents.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#eepromcontents
    pub async fn eeprom_contents_set(&mut self, data: &str) -> Result<()> {
        if self.eeprom_contents_get().await? == data {
            return Ok(());
        }

        self.command_new_line(&format!("eeprom.contents {}", data), true)
            .await
    }

    /// Gets the EEPROM's free bytes.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#eepromfree
    pub async fn eeprom_free(&mut self) -> Result<String> {
        self.command_response_string("eeprom.free").await
    }

    // TODO: upgrade.start

    pub async fn upgrade_neuron(&mut self) -> Result<()> {
        self.command_new_line("upgrade.neuron", true).await
    }

    // TODO: upgrade.end

    pub async fn upgrade_keyscanner_is_connected(&mut self, side: Side) -> Result<bool> {
        self.command_response_bool(&format!("upgrade.keyscanner.isConnected {}", side as u8))
            .await
    }

    pub async fn upgrade_keyscanner_is_bootloader(&mut self, side: Side) -> Result<bool> {
        self.command_response_bool(&format!("upgrade.keyscanner.isBootloader {}", side as u8))
            .await
    }

    pub async fn upgrade_keyscanner_begin(&mut self, side: Side) -> Result<bool> {
        self.command_response_bool(&format!("upgrade.keyscanner.begin {}", side as u8))
            .await
    }

    // TODO: upgrade.keyscanner.isReady
    // TODO: upgrade.keyscanner.getInfo

    pub async fn upgrade_keyscanner_send_write(&mut self) -> Result<()> {
        self.command_whitespace("upgrade.keyscanner.sendWrite")
            .await
    }

    // TODO: upgrade.keyscanner.validate
    // TODO: upgrade.keyscanner.finish
    // TODO: upgrade.keyscanner.sendStart

    /// Gets the Superkeys map.
    ///
    /// Each action in a Superkey is represented by a key code number that encodes the action, for example if you use the number 44, you are encoding space, etc...
    ///
    /// To know more about keycodes and to find the right one for your actions, check the key map database.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysmap
    pub async fn superkeys_map_get(&mut self) -> Result<Vec<u16>> {
        let data = self.command_response_string("superkeys.map").await?;

        string_to_numerical_vec(&data)
    }

    /// Sets the Superkeys map.
    ///
    /// Each action in a Superkey is represented by a key code number that encodes the action, for example if you use the number 44, you are encoding space, etc...
    ///
    /// To know more about keycodes and to find the right one for your actions, check the key map database.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysmap
    pub async fn superkeys_map_set(&mut self, data: &[u16]) -> Result<()> {
        if self.superkeys_map_get().await? == data {
            return Ok(());
        }

        self.command_new_line(
            &format!("superkeys.map {}", numerical_vec_to_string(data)),
            true,
        )
        .await
    }

    /// Gets the Superkeys wait for duration.
    ///
    /// Wait for value specifies the time between the first and subsequent releases of the HOLD actions meanwhile is held,
    ///
    /// So for example,
    /// if the variable is set to 500ms, you can maintain the hold key, it will emmit a key code corresponding to the action that it triggers,
    /// then it will wait for wait for time for making another key press with that same key code.
    /// This enables the user to delay the hold "machinegun" to be able to release the key and achieve a single keypress from a hold action.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeyswaitfor
    pub async fn superkeys_wait_for_get(&mut self) -> Result<Duration> {
        self.command_response_duration("superkeys.waitfor", TimeUnit::Milliseconds)
            .await
    }

    /// Sets the Superkeys wait for duration.
    ///
    /// Wait for value specifies the time between the first and subsequent releases of the HOLD actions meanwhile is held,
    ///
    /// So for example,
    /// if the variable is set to 500ms, you can maintain the hold key, it will emmit a key code corresponding to the action that it triggers,
    /// then it will wait for wait for time for making another key press with that same key code.
    /// This enables the user to delay the hold "machinegun" to be able to release the key and achieve a single keypress from a hold action.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeyswaitfor
    pub async fn superkeys_wait_for_set(&mut self, duration: Duration) -> Result<()> {
        if self.superkeys_wait_for_get().await? == duration {
            return Ok(());
        }

        self.command_new_line(
            &format!("superkeys.waitfor {}", &duration.as_millis()),
            true,
        )
        .await
    }

    /// Gets the Superkeys timeout of how long it waits for the next tap.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeystimeout
    pub async fn superkeys_timeout_get(&mut self) -> Result<Duration> {
        self.command_response_duration("superkeys.timeout", TimeUnit::Milliseconds)
            .await
    }

    /// Sets the Superkeys timeout of how long it waits for the next tap.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeystimeout
    pub async fn superkeys_timeout_set(&mut self, duration: Duration) -> Result<()> {
        if self.superkeys_timeout_get().await? == duration {
            return Ok(());
        }

        self.command_new_line(
            &format!("superkeys.timeout {}", &duration.as_millis()),
            true,
        )
        .await
    }

    /// Gets the Superkeys repeat duration.
    ///
    /// The repeat value specifies the time between the second and subsequent key code releases when on hold, it only takes effect after the wait for timer has been exceeded.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysrepeat
    pub async fn superkeys_repeat_get(&mut self) -> Result<Duration> {
        self.command_response_duration("superkeys.repeat", TimeUnit::Milliseconds)
            .await
    }

    /// Sets the Superkeys repeat duration.
    ///
    /// The repeat value specifies the time between the second and subsequent key code releases when on hold, it only takes effect after the wait for timer has been exceeded.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysrepeat
    pub async fn superkeys_repeat_set(&mut self, duration: Duration) -> Result<()> {
        if self.superkeys_repeat_get().await? == duration {
            return Ok(());
        }

        self.command_new_line(&format!("superkeys.repeat {}", &duration.as_millis()), true)
            .await
    }

    /// Gets the Superkeys hold start duration.
    ///
    /// The hold start value specifies the minimum time that has to pass between the first key down and any other action to trigger a hold, if held it will emit a hold action.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysholdstart
    pub async fn superkeys_hold_start_get(&mut self) -> Result<Duration> {
        self.command_response_duration("superkeys.holdstart", TimeUnit::Milliseconds)
            .await
    }

    /// Sets the Superkeys hold start duration.
    ///
    /// The hold start value specifies the minimum time that has to pass between the first key down and any other action to trigger a hold, if held it will emit a hold action.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysholdstart
    pub async fn superkeys_hold_start_set(&mut self, duration: Duration) -> Result<()> {
        if self.superkeys_hold_start_get().await? == duration {
            return Ok(());
        }

        self.command_new_line(
            &format!("superkeys.holdstart {}", &duration.as_millis()),
            true,
        )
        .await
    }

    /// Gets the Superkeys overlap percentage.
    ///
    /// The overlap value specifies the percentage of overlap when fast typing that is allowed to happen before triggering a hold action to the overlapped key pressed after the super key.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysoverlap
    pub async fn superkeys_overlap_get(&mut self) -> Result<u8> {
        self.command_response_numerical("superkeys.overlap").await
    }

    /// Sets the Superkeys overlap percentage.
    ///
    /// The overlap value specifies the percentage of overlap when fast typing that is allowed to happen before triggering a hold action to the overlapped key pressed after the super key.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysoverlap
    pub async fn superkeys_overlap_set(&mut self, percentage: u8) -> Result<()> {
        if percentage > 80 {
            bail!("Percentage must be 80 or below: {}", percentage);
        }

        if self.superkeys_overlap_get().await? == percentage {
            return Ok(());
        }

        self.command_new_line(&format!("superkeys.overlap {}", percentage), true)
            .await
    }

    /// Gets the color of a specific LED.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledat
    pub async fn led_at_get(&mut self, led: u8) -> Result<RGB> {
        let response = self
            .command_response_string(&format!("led.at {}", led))
            .await?;

        if response.is_empty() {
            bail!("Empty response");
        }

        let parts = response.split_whitespace().collect::<Vec<&str>>();

        if parts.len() != 3 {
            bail!("Response does not contain exactly three parts");
        }

        let r = parts[0].parse()?;
        let g = parts[1].parse()?;
        let b = parts[2].parse()?;

        Ok(RGB { r, g, b })
    }

    /// Sets the color of a specific LED.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledat
    pub async fn led_at_set(&mut self, led: u8, color: &RGB) -> Result<()> {
        if &self.led_at_get(led).await? == color {
            return Ok(());
        }

        self.command_new_line(
            &format!("led.at {} {} {} {}", led, color.r, color.g, color.b),
            true,
        )
        .await
    }

    /// Sets the color of all the LEDs.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledsetall
    pub async fn led_all(&mut self, color: &RGB) -> Result<()> {
        self.command_new_line(
            &format!("led.setAll {} {} {}", color.r, color.g, color.b,),
            true,
        )
        .await
    }

    /// Gets the LED mode.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledmode
    pub async fn led_mode_get(&mut self) -> Result<LedMode> {
        self.command_response_numerical("led.mode").await
    }

    /// Sets the LED mode.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledmode
    pub async fn led_mode_set(&mut self, mode: LedMode) -> Result<()> {
        if self.led_mode_get().await? == mode {
            return Ok(());
        }

        self.command_new_line(&format!("led.mode {}", mode as u8), true)
            .await
    }

    /// Gets the top LED brightness.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightness
    pub async fn led_brightness_top_get(&mut self) -> Result<u8> {
        self.command_response_numerical("led.brightness").await
    }

    /// Sets the top LED brightness.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightness
    pub async fn led_brightness_top_set(&mut self, brightness: u8) -> Result<()> {
        if self.led_brightness_top_get().await? == brightness {
            return Ok(());
        }

        self.command_new_line(&format!("led.brightness {}", brightness), true)
            .await
    }

    /// Gets the underglow LED brightness.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightnessug
    pub async fn led_brightness_underglow_get(&mut self) -> Result<u8> {
        self.command_response_numerical("led.brightnessUG").await
    }

    /// Sets the underglow LED brightness.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightnessug
    pub async fn led_brightness_underglow_set(&mut self, brightness: u8) -> Result<()> {
        if self.led_brightness_underglow_get().await? == brightness {
            return Ok(());
        }

        self.command_new_line(&format!("led.brightnessUG {}", brightness), true)
            .await
    }

    /// Gets the wireless top LED brightness.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightness
    pub async fn led_brightness_wireless_top_get(&mut self) -> Result<u8> {
        self.command_response_numerical("led.brightness.wireless")
            .await
    }

    /// Sets the wireless top LED brightness.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightness
    pub async fn led_brightness_wireless_top_set(&mut self, brightness: u8) -> Result<()> {
        if self.led_brightness_wireless_top_get().await? == brightness {
            return Ok(());
        }

        self.command_new_line(&format!("led.brightness.wireless {}", brightness), true)
            .await
    }

    /// Gets the wireless underglow LED brightness.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightnessug
    pub async fn led_brightness_wireless_underglow_get(&mut self) -> Result<u8> {
        self.command_response_numerical("led.brightnessUG.wireless")
            .await
    }

    /// Sets the wireless underglow LED brightness.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightnessug
    pub async fn led_brightness_wireless_underglow_set(&mut self, brightness: u8) -> Result<()> {
        if self.led_brightness_wireless_underglow_get().await? == brightness {
            return Ok(());
        }

        self.command_new_line(&format!("led.brightnessUG.wireless {}", brightness), true)
            .await
    }

    /// Gets the LED fade.
    pub async fn led_fade_get(&mut self) -> Result<u16> {
        self.command_response_numerical("led.fade").await
    }

    /// Sets the LED fade.
    pub async fn led_fade_set(&mut self, fade: u16) -> Result<()> {
        if self.led_fade_get().await? == fade {
            return Ok(());
        }

        self.command_new_line(&format!("led.fade {}", fade), true)
            .await
    }

    /// Gets the LED theme.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledtheme
    pub async fn led_theme_get(&mut self) -> Result<Vec<RGB>> {
        let data = self.command_response_string("led.theme").await?;

        string_to_rgb_vec(&data)
    }

    /// Sets the LED theme.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledtheme
    pub async fn led_theme_set(&mut self, data: &[RGB]) -> Result<()> {
        if self.led_theme_get().await? == data {
            return Ok(());
        }

        self.command_new_line(&format!("led.theme {}", &rgb_vec_to_string(data)), true)
            .await
    }

    /// Gets the palette as RGB.
    ///
    /// The color palette is used by the color map to establish each color that can be assigned to the keyboard.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#palette
    pub async fn palette_rgb_get(&mut self) -> Result<Vec<RGB>> {
        let data = self.command_response_string("palette").await?;

        string_to_rgb_vec(&data)
    }

    /// Sets the palette as RGB.
    ///
    /// The color palette is used by the color map to establish each color that can be assigned to the keyboard.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#palette
    pub async fn palette_rgb_set(&mut self, data: &[RGB]) -> Result<()> {
        if self.palette_rgb_get().await? == data {
            return Ok(());
        }

        self.command_new_line(&format!("palette {}", rgb_vec_to_string(data)), true)
            .await
    }

    /// Gets the palette as RGBW.
    ///
    /// The color palette is used by the color map to establish each color that can be assigned to the keyboard.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#palette
    pub async fn palette_rgbw_get(&mut self) -> Result<Vec<RGBW>> {
        let data = self.command_response_string("palette").await?;

        string_to_rgbw_vec(&data)
    }

    /// Sets the palette as RGBW.
    ///
    /// The color palette is used by the color map to establish each color that can be assigned to the keyboard.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#palette
    pub async fn palette_rgbw_set(&mut self, data: &[RGBW]) -> Result<()> {
        if self.palette_rgbw_get().await? == data {
            return Ok(());
        }

        self.command_new_line(&format!("palette {}", rgbw_vec_to_string(data)), true)
            .await
    }

    /// Gets the color map.
    ///
    /// This command reads the color map that assigns each color listed in the palette to individual LEDs, mapping them to the keyboard's current layout.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#colormapmap
    pub async fn color_map_get(&mut self) -> Result<Vec<u8>> {
        let data = self.command_response_string("colormap.map").await?;

        string_to_numerical_vec(&data)
    }

    /// Sets the color map.
    ///
    /// This command writes the color map that assigns each color listed in the palette to individual LEDs, mapping them to the keyboard's current layout.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#colormapmap
    pub async fn color_map_set(&mut self, data: &[u8]) -> Result<()> {
        if self.color_map_get().await? == data {
            return Ok(());
        }

        self.command_new_line(
            &format!("colormap.map {}", numerical_vec_to_string(data)),
            true,
        )
        .await
    }

    /// Gets the idle LED true sleep state.
    pub async fn led_idle_true_sleep_get(&mut self) -> Result<bool> {
        self.command_response_bool("idleleds.true_sleep").await
    }

    /// Sets the idle LED true sleep state.
    pub async fn led_idle_true_sleep_set(&mut self, state: bool) -> Result<()> {
        if self.led_idle_true_sleep_get().await? == state {
            return Ok(());
        }

        self.command_new_line(&format!("idleleds.true_sleep {}", state as u8), true)
            .await
    }

    /// Gets the idle LED true sleep time.
    pub async fn led_idle_true_sleep_time_get(&mut self) -> Result<Duration> {
        self.command_response_duration("idleleds.true_sleep_time", TimeUnit::Seconds)
            .await
    }

    /// Sets the idle LED true sleep time.
    pub async fn led_idle_true_sleep_time_set(&mut self, duration: Duration) -> Result<()> {
        let seconds = duration.as_secs();

        if seconds > 65_000 {
            bail!("Seconds must be 65000 or below: {}", seconds);
        }

        if self.led_idle_true_sleep_time_get().await? == duration {
            return Ok(());
        }

        self.command_new_line(&format!("idleleds.true_sleep_time {}", seconds), true)
            .await
    }

    /// Gets the idle LED time limit.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#idleledstime_limit
    pub async fn led_idle_time_limit_get(&mut self) -> Result<Duration> {
        self.command_response_duration("idleleds.time_limit", TimeUnit::Seconds)
            .await
    }

    /// Sets the idle LED time limit.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#idleledstime_limit
    pub async fn led_idle_time_limit_set(&mut self, duration: Duration) -> Result<()> {
        let seconds = duration.as_secs();

        if seconds > 65_000 {
            bail!("Duration must be 65000 seconds or below, got: {}", seconds);
        }

        if self.led_idle_time_limit_get().await? == duration {
            return Ok(());
        }

        self.command_new_line(&format!("idleleds.time_limit {}", seconds), true)
            .await
    }

    /// Gets the idle LED wireless state.
    pub async fn led_idle_wireless_get(&mut self) -> Result<bool> {
        self.command_response_bool("idleleds.wireless").await
    }

    /// Sets the idle LED wireless state.
    pub async fn led_idle_wireless_set(&mut self, state: bool) -> Result<()> {
        if self.led_idle_wireless_get().await? == state {
            return Ok(());
        }

        self.command_new_line(&format!("idleleds.wireless {}", state as u8), true)
            .await
    }

    /// Gets the keyboard model name.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#hardwareversion
    pub async fn hardware_version_get(&mut self) -> Result<String> {
        self.command_response_string("hardware.version").await
    }

    /// Sets the keyboard model name.
    pub async fn hardware_version_set(&mut self, data: &str) -> Result<()> {
        if self.hardware_version_get().await? == data {
            return Ok(());
        }

        self.command_new_line(&format!("hardware.version {}", data), true)
            .await
    }

    // TODO: hardware.side_power https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#hardwareside_power
    // TODO: hardware.side_ver https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#hardwareside_ver
    // TODO: hardware.keyscanInterval
    // TODO: hardware.firmware https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#hardwarefirmware
    // TODO: hardware.chip_id
    // TODO: hardware.chip_info

    /// Gets the Qukeys hold timeout.
    ///
    /// https://kaleidoscope.readthedocs.io/en/latest/plugins/Kaleidoscope-Qukeys.html
    pub async fn qukeys_hold_timeout_get(&mut self) -> Result<Duration> {
        self.command_response_duration("qukeys.holdTimeout", TimeUnit::Milliseconds)
            .await
    }

    /// Sets the Qukeys hold timeout.
    ///
    /// https://kaleidoscope.readthedocs.io/en/latest/plugins/Kaleidoscope-Qukeys.html
    pub async fn qukeys_hold_timeout_set(&mut self, duration: Duration) -> Result<()> {
        if self.qukeys_hold_timeout_get().await? == duration {
            return Ok(());
        }

        self.command_new_line(
            &format!("qukeys.holdTimeout {}", &duration.as_millis()),
            true,
        )
        .await
    }

    /// Gets the Qukeys overlap threshold.
    ///
    /// https://kaleidoscope.readthedocs.io/en/latest/plugins/Kaleidoscope-Qukeys.html
    pub async fn qukeys_overlap_threshold_get(&mut self) -> Result<Duration> {
        self.command_response_duration("qukeys.overlapThreshold", TimeUnit::Milliseconds)
            .await
    }

    /// Sets the Qukeys overlap threshold.
    ///
    /// https://kaleidoscope.readthedocs.io/en/latest/plugins/Kaleidoscope-Qukeys.html
    pub async fn qukeys_overlap_threshold_set(&mut self, duration: Duration) -> Result<()> {
        if self.qukeys_overlap_threshold_get().await? == duration {
            return Ok(());
        }

        self.command_new_line(
            &format!("qukeys.overlapThreshold {}", &duration.as_millis()),
            true,
        )
        .await
    }

    /// Gets the macros map.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#macrosmap
    pub async fn macros_map_get(&mut self) -> Result<Vec<u8>> {
        let data = self.command_response_string("macros.map").await?;

        string_to_numerical_vec(&data)
    }

    /// Sets the macros map.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#macrosmap
    pub async fn macros_map_set(&mut self, data: &[u8]) -> Result<()> {
        if self.macros_map_get().await? == data {
            return Ok(());
        }

        self.command_new_line(
            &format!("macros.map {}", numerical_vec_to_string(data)),
            true,
        )
        .await
    }

    /// Triggers a macro.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#macrostrigger
    pub async fn macros_trigger(&mut self, macro_id: u8) -> Result<()> {
        self.command_new_line(&format!("macros.trigger {}", macro_id), true)
            .await
    }

    /// Gets the macros memory size in bytes.
    pub async fn macros_memory(&mut self) -> Result<u16> {
        self.command_response_numerical("macros.memory").await
    }

    /// Gets all the available commands in the current version of the serial protocol.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#help
    pub async fn help_get(&mut self) -> Result<Vec<String>> {
        self.command_response_vec_string("help").await
    }

    /// Gets the virtual mouse speed.
    pub async fn mouse_speed_get(&mut self) -> Result<u8> {
        self.command_response_numerical("mouse.speed").await
    }

    /// Sets the virtual mouse speed.
    pub async fn mouse_speed_set(&mut self, speed: u8) -> Result<()> {
        if speed > 127 {
            bail!("Speed out of range, max is {}: {}", 127, speed);
        }

        if self.mouse_speed_get().await? == speed {
            return Ok(());
        }

        self.command_new_line(&format!("mouse.speed {}", speed), true)
            .await
    }

    /// Gets the virtual mouse delay.
    pub async fn mouse_delay_get(&mut self) -> Result<Duration> {
        self.command_response_duration("mouse.speedDelay", TimeUnit::Milliseconds)
            .await
    }

    /// Sets the virtual mouse delay.
    pub async fn mouse_delay_set(&mut self, duration: Duration) -> Result<()> {
        if self.mouse_delay_get().await? == duration {
            return Ok(());
        }

        self.command_new_line(&format!("mouse.speedDelay {}", &duration.as_millis()), true)
            .await
    }

    /// Gets the virtual mouse acceleration speed.
    pub async fn mouse_acceleration_speed_get(&mut self) -> Result<u8> {
        self.command_response_numerical("mouse.accelSpeed").await
    }

    /// Sets the virtual mouse acceleration speed.
    pub async fn mouse_acceleration_speed_set(&mut self, speed: u8) -> Result<()> {
        if self.mouse_acceleration_speed_get().await? == speed {
            return Ok(());
        }

        self.command_new_line(&format!("mouse.accelSpeed {}", speed), true)
            .await
    }

    /// Gets the virtual mouse acceleration delay.
    pub async fn mouse_acceleration_delay_get(&mut self) -> Result<Duration> {
        self.command_response_duration("mouse.accelDelay", TimeUnit::Milliseconds)
            .await
    }

    /// Sets the virtual mouse acceleration delay.
    pub async fn mouse_acceleration_delay_set(&mut self, duration: Duration) -> Result<()> {
        if self.mouse_acceleration_delay_get().await? == duration {
            return Ok(());
        }

        self.command_new_line(&format!("mouse.accelDelay {}", &duration.as_millis()), true)
            .await
    }

    /// Gets the virtual mouse wheel speed.
    pub async fn mouse_wheel_speed_get(&mut self) -> Result<u8> {
        self.command_response_numerical("mouse.wheelSpeed").await
    }

    /// Sets the virtual mouse wheel speed.
    pub async fn mouse_wheel_speed_set(&mut self, speed: u8) -> Result<()> {
        if self.mouse_wheel_speed_get().await? == speed {
            return Ok(());
        }

        self.command_new_line(&format!("mouse.wheelSpeed {}", speed), true)
            .await
    }

    /// Gets the virtual mouse wheel delay.
    pub async fn mouse_wheel_delay_get(&mut self) -> Result<Duration> {
        self.command_response_duration("mouse.wheelDelay", TimeUnit::Milliseconds)
            .await
    }

    /// Sets the virtual mouse wheel delay.
    pub async fn mouse_wheel_delay_set(&mut self, duration: Duration) -> Result<()> {
        if self.mouse_wheel_delay_get().await? == duration {
            return Ok(());
        }

        self.command_new_line(&format!("mouse.wheelDelay {}", &duration.as_millis()), true)
            .await
    }

    /// Gets the virtual mouse speed limit.
    pub async fn mouse_speed_limit_get(&mut self) -> Result<u8> {
        self.command_response_numerical("mouse.speedLimit").await
    }

    /// Sets the virtual mouse speed limit.
    pub async fn mouse_speed_limit_set(&mut self, limit: u8) -> Result<()> {
        if self.mouse_speed_limit_get().await? == limit {
            return Ok(());
        }

        self.command_new_line(&format!("mouse.speedLimit {}", limit), true)
            .await
    }

    /// Activate a certain layer remotely just by sending its order number.
    ///
    /// The layer is -1 to Bazecor.
    ///
    /// This does not affect the memory usage as the value is stored in RAM.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#layeractivate
    pub async fn layer_activate(&mut self, layer: u8) -> Result<()> {
        self.command_new_line(&format!("layer.activate {}", layer), true)
            .await
    }

    /// Deactivate the last layer that the keyboard switched to.
    /// This same function is the way the shift to layer key works on the keyboard.
    ///
    /// Just provide the layer number to make the keyboard go back one layer. The layer is -1 to Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#layerdeactivate
    pub async fn layer_deactivate(&mut self, layer: Option<u8>) -> Result<()> {
        if let Some(layer) = layer {
            if layer > MAX_LAYERS {
                bail!("Layer out of range, max is {}: {}", MAX_LAYERS, layer);
            }
            self.command_new_line(&format!("layer.deactivate {}", layer), true)
                .await?
        }

        self.command_new_line("layer.deactivate", true).await
    }

    /// Gets the state of the provided layer.
    ///
    /// The layer is -1 to Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#layerisactive
    pub async fn layer_is_active(&mut self, layer: u8) -> Result<bool> {
        if layer > MAX_LAYERS {
            bail!("Layer out of range, max is {}: {}", MAX_LAYERS, layer);
        }
        self.command_response_bool(&format!("layer.isActive {}", layer))
            .await
    }

    /// Switch to a certain layer.
    ///
    /// The layer is -1 to Bazecor.
    ///
    /// The difference between this command and the layer_activate alternative, is that the layer_activate adds to the layer switching history, but moveTo will erase that memory and return it to an array length 1 and holding the current layer the keyboard moved to.
    ///
    /// This does not affect the memory usage as the value is stored in RAM.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#layermoveto
    pub async fn layer_move_to(&mut self, layer: u8) -> Result<()> {
        self.command_new_line(&format!("layer.moveTo {}", layer), true)
            .await
    }

    /// Gets the status for up to 32 layers.
    ///
    /// It will return a vector of bools with the respective index matching each layer, -1 from Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#layerstate
    pub async fn layer_state(&mut self) -> Result<Vec<bool>> {
        let response = self.command_response_string("layer.state").await?;
        let parts = response.split_whitespace().collect::<Vec<&str>>();
        let nums = parts.iter().map(|&part| part == "1").collect();

        Ok(nums)
    }

    /// Gets the battery level of the left keyboard as a percentage.
    pub async fn wireless_battery_level_left_get(&mut self) -> Result<u8> {
        self.command_response_numerical("wireless.battery.left.level")
            .await
    }

    /// Gets the battery level of the right keyboard as a percentage.
    pub async fn wireless_battery_level_right_get(&mut self) -> Result<u8> {
        self.command_response_numerical("wireless.battery.right.level")
            .await
    }

    /// Gets the battery status of the left keyboard.
    pub async fn wireless_battery_status_left_get(&mut self) -> Result<u8> {
        self.command_response_numerical("wireless.battery.left.status")
            .await
    }

    /// Gets the battery status of the right keyboard.
    pub async fn wireless_battery_status_right_get(&mut self) -> Result<u8> {
        self.command_response_numerical("wireless.battery.right.status")
            .await
    }

    /// Gets the battery saving mode state.
    pub async fn wireless_battery_saving_mode_get(&mut self) -> Result<bool> {
        self.command_response_bool("wireless.battery.savingMode")
            .await
    }

    /// Sets the battery saving mode state.
    pub async fn wireless_battery_saving_mode_set(&mut self, state: bool) -> Result<()> {
        if self.wireless_battery_saving_mode_get().await? == state {
            return Ok(());
        }

        self.command_new_line(
            &format!("wireless.battery.savingMode {}", state as u8),
            true,
        )
        .await
    }

    /// Gets the RF power level.
    pub async fn wireless_rf_power_level_get(&mut self) -> Result<WirelessPowerMode> {
        self.command_response_numerical("wireless.rf.power").await
    }

    /// Sets the RF power level.
    pub async fn wireless_rf_power_level_set(
        &mut self,
        wireless_power_mode: WirelessPowerMode,
    ) -> Result<()> {
        if self.wireless_rf_power_level_get().await? == wireless_power_mode {
            return Ok(());
        }

        self.command_new_line(
            &format!("wireless.rf.power {}", wireless_power_mode as u8),
            true,
        )
        .await
    }

    /// Gets the RF channel hop state.
    pub async fn wireless_rf_channel_hop_get(&mut self) -> Result<bool> {
        self.command_response_bool("wireless.rf.channelHop").await
    }

    /// Sets the RF channel hop state.
    pub async fn wireless_rf_channel_hop_set(&mut self, state: bool) -> Result<()> {
        if self.wireless_rf_channel_hop_get().await? == state {
            return Ok(());
        }

        self.command_new_line(&format!("wireless.rf.channelHop {}", state as u8), true)
            .await
    }

    /// Gets the sync pairing state.
    pub async fn wireless_rf_sync_pairing(&mut self) -> Result<bool> {
        self.command_response_bool("wireless.rf.syncPairing").await
    }
}
