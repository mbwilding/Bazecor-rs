use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_ansi(true)
        .with_max_level(tracing_subscriber::filter::LevelFilter::DEBUG)
        .init();

    let firmware_releases = api::flash::load_available_firmware_versions(true).await?;

    let _fw_defy_wireless = api::flash::download_firmware(
        "default",
        &api::devices::DEFY_WIRELESS,
        &firmware_releases,
        0,
    )
    .await?;

    let _fw_raise_iso =
        api::flash::download_firmware("default", &api::devices::RAISE_ISO, &firmware_releases, 2)
            .await?;

    Ok(())
}
