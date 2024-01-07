use anyhow::Result;
use dygma_focus::hardware::Device;
use dygma_focus::Focus;
use log::info;

// TODO: Remove, redundant as it is the same as the Focus API
pub async fn prepare_neuron(device: &Device) -> Result<()> {
    let mut focus = Focus::new_via_device(device).await?;

    info!("Upgrading the Neuron");

    focus.upgrade_neuron().await
    // sleep 10ms
}
