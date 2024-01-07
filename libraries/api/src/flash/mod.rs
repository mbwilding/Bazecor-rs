pub mod devices;

use anyhow::Result;
use chrono::format::StrftimeItems;
use chrono::Local;
use log::info;
use serde::Serialize;
use tokio::fs;

pub trait Flasher {
    /// Takes a backup of the device settings and saves a backup file.
    fn backup_settings(&self) -> Result<()>;
}

/// Formats date for create name of backup file.
///
/// Example output: "2019-07-12-19_40_56"
pub fn formatted_date() -> String {
    let now = Local::now();
    let formatted_date = now
        .format_with_items(StrftimeItems::new("%Y-%m-%d-%H_%M_%S"))
        .to_string();
    formatted_date
}

/// Saves backup file to a directory
///
/// Windows: `C:\Users\%username%\AppData\Local\Programs\bazecor`
///
/// Other: The directory where the app is located.
pub async fn save_backup_file<T>(device_name: &str, file_data: &T) -> Result<()>
where
    T: Serialize,
{
    let user_data_path = if cfg!(target_os = "windows") {
        dirs::data_local_dir()
            .unwrap()
            .join("Programs")
            .join("bazecor")
    } else {
        std::env::current_dir().unwrap()
    };

    let file_path =
        user_data_path.join(format!("{}-backup-{}.json", device_name, formatted_date()));

    info!("Saving file to: {:?}", file_path);

    let json = serde_json::to_string(file_data)?;
    fs::write(file_path, json).await?;

    Ok(())
}
