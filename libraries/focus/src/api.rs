use crate::prelude::*;
use crate::{Focus, MAX_LAYERS};
use anyhow::{anyhow, bail, Result};
use log::debug;
use std::str::FromStr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Private methods
impl Focus {
    /// Sends a command to the device, with no response.
    async fn command(&mut self, command: &str) -> Result<()> {
        debug!("Command TX: {}", command);

        self.serial
            .write_all(format!("{}\n", command).as_bytes())
            .await?;

        Ok(())
    }

    /// Sends a command to the device, and returns the response as a string.
    async fn command_response_string(&mut self, command: &str) -> Result<String> {
        self.command(command).await?;

        let eof_marker = b"\r\n.\r\n";

        self.response_buffer.clear();

        loop {
            let prev_len = self.response_buffer.len();
            self.response_buffer.resize(prev_len + 1024, 0);
            match self
                .serial
                .read(&mut self.response_buffer[prev_len..])
                .await
            {
                Ok(0) => continue,
                Ok(size) => {
                    self.response_buffer.truncate(prev_len + size);

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

        debug!("Command RX: {}", &response);

        Ok(response.to_string())
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
        Ok(response == "1" || response == "true")
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
    pub async fn version_get(&mut self) -> Result<String> {
        self.command_response_string("version").await
    }

    /// Gets the whole custom keymap stored in the keyboard.
    ///
    /// Layers 0 and above, The layers are -1 to Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymapcustom
    pub async fn keymap_custom_get(&mut self) -> Result<String> {
        self.command_response_string("keymap.custom").await
    }

    /// Sets the whole custom keymap stored in the keyboard.
    ///
    /// Layers 0 and above, The layers are -1 to Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymapcustom
    pub async fn keymap_custom_set(&mut self, data: &str) -> Result<()> {
        if self.keymap_custom_get().await? == data {
            return Ok(());
        }

        self.command(&format!("keymap.custom {}", data)).await
    }

    /// Gets the default keymap stored in the keyboard.
    ///
    /// Layers -1 and -2, the layers are -1 to Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymapdefault
    pub async fn keymap_default_get(&mut self) -> Result<String> {
        self.command_response_string("keymap.default").await
    }

    /// Sets the default keymap stored in the keyboard.
    ///
    /// Layers -1 and -2, the layers are -1 to Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymapdefault
    pub async fn keymap_default_set(&mut self, data: &str) -> Result<()> {
        if self.keymap_default_get().await? == data {
            return Ok(());
        }

        self.command(&format!("keymap.default {}", data)).await
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

        self.command(&format!("keymap.onlyCustom {}", state as u8))
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

        self.command(&format!("settings.defaultLayer {}", layer))
            .await
    }

    /// Gets a boolean value that states true if all checks have been performed on the current settings, and its upload was done in the intended way.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#settingsvalid
    pub async fn settings_valid_get(&mut self) -> Result<bool> {
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

        self.command(&format!("settings.version {}", version)).await
    }

    /// Gets the CRC checksum of the layout.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#settingscrc
    pub async fn settings_crc_get(&mut self) -> Result<String> {
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

        self.command(&format!("eeprom.contents {}", data)).await
    }

    /// Gets the EEPROM's free bytes.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#eepromfree
    pub async fn eeprom_free_get(&mut self) -> Result<String> {
        self.command_response_string("eeprom.free").await
    }

    // TODO: upgrade.start

    /// Upgrade: Neuron.
    pub async fn upgrade_neuron(&mut self) -> Result<()> {
        self.command("upgrade.neuron").await
    }

    // TODO: upgrade.end

    /// Gets the status of the Keyscanner: is connected?
    pub async fn upgrade_keyscanner_is_connected_get(&mut self, side: Side) -> Result<bool> {
        self.command_response_bool(&format!("upgrade.keyscanner.isConnected {}", side as u8))
            .await
    }

    /// Gets the status of the Keyscanner: is bootloader?
    pub async fn upgrade_keyscanner_is_bootloader_get(&mut self, side: Side) -> Result<bool> {
        self.command_response_bool(&format!("upgrade.keyscanner.isBootloader {}", side as u8))
            .await
    }

    /// Gets the status of the Keyscanner: begin?
    pub async fn upgrade_keyscanner_begin_get(&mut self, side: Side) -> Result<bool> {
        self.command_response_bool(&format!("upgrade.keyscanner.begin {}", side as u8))
            .await
    }

    // TODO: upgrade.keyscanner.isReady
    // TODO: upgrade.keyscanner.getInfo
    // TODO: upgrade.keyscanner.sendWrite
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
    pub async fn superkeys_map_get(&mut self) -> Result<String> {
        self.command_response_string("superkeys.map").await
    }

    /// Sets the Superkeys map.
    ///
    /// Each action in a Superkey is represented by a key code number that encodes the action, for example if you use the number 44, you are encoding space, etc...
    ///
    /// To know more about keycodes and to find the right one for your actions, check the key map database.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysmap
    pub async fn superkeys_map_set(&mut self, data: &str) -> Result<()> {
        if self.superkeys_map_get().await? == data {
            return Ok(());
        }

        self.command(&format!("superkeys.map {}", data)).await
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

        self.command(&format!("superkeys.waitfor {}", &duration.as_millis()))
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

        self.command(&format!("superkeys.timeout {}", &duration.as_millis()))
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

        self.command(&format!("superkeys.repeat {}", &duration.as_millis()))
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

        self.command(&format!("superkeys.holdstart {}", &duration.as_millis()))
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

        self.command(&format!("superkeys.overlap {}", percentage))
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

        self.command(&format!(
            "led.at {} {} {} {}",
            led, color.r, color.g, color.b
        ))
        .await
    }

    /// Sets the color of all the LEDs.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledsetall
    pub async fn led_all_set(&mut self, color: &RGB) -> Result<()> {
        self.command(&format!("led.setAll {} {} {}", color.r, color.g, color.b,))
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

        self.command(&format!("led.mode {}", mode as u8)).await
    }

    /// Gets the LED brightness.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightness
    pub async fn led_brightness_get(&mut self) -> Result<u8> {
        self.command_response_numerical("led.brightness").await
    }

    /// Sets the LED brightness.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightness
    pub async fn led_brightness_set(&mut self, brightness: u8) -> Result<()> {
        if self.led_brightness_get().await? == brightness {
            return Ok(());
        }

        self.command(&format!("led.brightness {}", brightness))
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

        self.command(&format!("led.brightnessUG {}", brightness))
            .await
    }

    /// Gets the wireless LED brightness.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightness
    pub async fn led_brightness_wireless_get(&mut self) -> Result<u8> {
        self.command_response_numerical("led.brightness.wireless")
            .await
    }

    /// Sets the wireless LED brightness.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightness
    pub async fn led_brightness_wireless_set(&mut self, brightness: u8) -> Result<()> {
        if self.led_brightness_wireless_get().await? == brightness {
            return Ok(());
        }

        self.command(&format!("led.brightness.wireless {}", brightness))
            .await
    }

    /// Gets the wireless underglow LED brightness.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightnessug
    pub async fn led_brightness_underglow_wireless_get(&mut self) -> Result<u8> {
        self.command_response_numerical("led.brightnessUG.wireless")
            .await
    }

    /// Sets the wireless underglow LED brightness.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightnessug
    pub async fn led_brightness_underglow_wireless_set(&mut self, brightness: u8) -> Result<()> {
        if self.led_brightness_underglow_wireless_get().await? == brightness {
            return Ok(());
        }

        self.command(&format!("led.brightnessUG.wireless {}", brightness))
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

        self.command(&format!("led.fade {}", fade)).await
    }

    /// Gets the LED theme.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledtheme
    pub async fn led_theme_get(&mut self) -> Result<String> {
        self.command_response_string("led.theme").await
    }

    /// Sets the LED theme.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledtheme
    pub async fn led_theme_set(&mut self, data: &str) -> Result<()> {
        if self.led_theme_get().await? == data {
            return Ok(());
        }

        self.command(&format!("led.theme {}", data)).await
    }

    /// Gets the palette.
    ///
    /// The color palette is used by the color map to establish each color that can be assigned to the keyboard.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#palette
    pub async fn palette_get(&mut self) -> Result<String> {
        self.command_response_string("palette").await
    }

    /// Sets the palette.
    ///
    /// The color palette is used by the color map to establish each color that can be assigned to the keyboard.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#palette
    pub async fn palette_set(&mut self, data: &str) -> Result<()> {
        if self.palette_get().await? == data {
            return Ok(());
        }

        self.command(&format!("palette {}", data)).await
    }

    /// Gets the color map.
    ///
    /// This command reads the color map that assigns each color listed in the palette to individual LEDs, mapping them to the keyboard's current layout.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#colormapmap
    pub async fn color_map_get(&mut self) -> Result<String> {
        self.command_response_string("colormap.map").await
    }

    /// Sets the color map.
    ///
    /// This command writes the color map that assigns each color listed in the palette to individual LEDs, mapping them to the keyboard's current layout.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#colormapmap
    pub async fn color_map_set(&mut self, data: &str) -> Result<()> {
        if self.color_map_get().await? == data {
            return Ok(());
        }

        self.command(&format!("colormap.map {}", data)).await
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

        self.command(&format!("idleleds.true_sleep {}", state as u8))
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

        self.command(&format!("idleleds.true_sleep_time {}", seconds))
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

        self.command(&format!("idleleds.time_limit {}", seconds))
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

        self.command(&format!("idleleds.wireless {}", state as u8))
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

        self.command(&format!("hardware.version {}", data)).await
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

        self.command(&format!("qukeys.holdTimeout {}", &duration.as_millis()))
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

        self.command(&format!(
            "qukeys.overlapThreshold {}",
            &duration.as_millis()
        ))
        .await
    }

    /// Gets the macros map.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#macrosmap
    pub async fn macros_map_get(&mut self) -> Result<String> {
        self.command_response_string("macros.map").await
    }

    /// Sets the macros map.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#macrosmap
    pub async fn macros_map_set(&mut self, data: &str) -> Result<()> {
        if self.macros_map_get().await? == data {
            return Ok(());
        }

        self.command(&format!("macros.map {}", data)).await
    }

    /// Triggers a macro.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#macrostrigger
    pub async fn macros_trigger(&mut self, macro_id: u8) -> Result<()> {
        self.command(&format!("macros.trigger {}", macro_id)).await
    }

    /// Gets the macros memory size in bytes.
    pub async fn macros_memory_get(&mut self) -> Result<u16> {
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

        self.command(&format!("mouse.speed {}", speed)).await
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

        self.command(&format!("mouse.speedDelay {}", &duration.as_millis()))
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

        self.command(&format!("mouse.accelSpeed {}", speed)).await
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

        self.command(&format!("mouse.accelDelay {}", &duration.as_millis()))
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

        self.command(&format!("mouse.wheelSpeed {}", speed)).await
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

        self.command(&format!("mouse.wheelDelay {}", &duration.as_millis()))
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

        self.command(&format!("mouse.speedLimit {}", limit)).await
    }

    /// Activate a certain layer remotely just by sending its order number.
    ///
    /// The layer is -1 to Bazecor.
    ///
    /// This does not affect the memory usage as the value is stored in RAM.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#layeractivate
    pub async fn layer_activate(&mut self, layer: u8) -> Result<()> {
        self.command(&format!("layer.activate {}", layer)).await
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
            self.command(&format!("layer.deactivate {}", layer)).await?
        }

        self.command("layer.deactivate").await
    }

    /// Gets the state of the provided layer.
    ///
    /// The layer is -1 to Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#layerisactive
    pub async fn layer_is_active_get(&mut self, layer: u8) -> Result<bool> {
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
        self.command(&format!("layer.moveTo {}", layer)).await
    }

    /// Gets the status for up to 32 layers.
    ///
    /// It will return a vector of bools with the respective index matching each layer, -1 from Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#layerstate
    pub async fn layer_state_get(&mut self) -> Result<Vec<bool>> {
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

        self.command(&format!("wireless.battery.savingMode {}", state as u8))
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

        self.command(&format!("wireless.rf.power {}", wireless_power_mode as u8))
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

        self.command(&format!("wireless.rf.channelHop {}", state as u8))
            .await
    }

    /// Gets the sync pairing state.
    pub async fn wireless_rf_sync_pairing_get(&mut self) -> Result<bool> {
        self.command_response_bool("wireless.rf.syncPairing").await
    }
}
