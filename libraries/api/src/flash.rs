use crate::hardware::{Hardware, KeyboardType, Product};
use anyhow::{bail, Context, Result};
use log::{debug, error, trace};
use regex::Regex;
use semver::{Version, VersionReq};
use serde::Deserialize;

const FW_MAJOR_VERSION: &str = "1.x";

#[derive(Debug)]
pub struct FirmwareRelease {
    pub name: String,
    pub version: String,
    pub body: String,
    pub assets: Vec<FirmwareAsset>,
}

#[derive(Debug)]
pub struct FirmwareAsset {
    pub name: String,
    pub url: String,
}

#[derive(Debug)]
pub struct Firmware {
    pub firmware: Vec<u8>,
    pub sides: Option<Vec<u8>>,
}

#[derive(Debug)]
pub struct Ctx {
    pub device: Hardware,
    pub collected: Collected,
    pub allow_beta: bool,
}

#[derive(Debug)]
pub struct Collected {
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub name: String,
    pub body: String,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    #[serde(rename = "browser_download_url")]
    pub url: String,
}

#[derive(Debug)]
pub struct GitHubInfo {
    pub firmwares: Vec<FirmwareRelease>,
    pub is_updated: bool,
    pub is_beta: bool,
}

fn parse_version(version_str: &str) -> Version {
    let mut clean_version_str = version_str.trim_start_matches('v');

    if let Some(hyphen_index) = clean_version_str.find('-') {
        clean_version_str = &clean_version_str[..hyphen_index];
    }

    Version::parse(clean_version_str).unwrap()
}

pub async fn github_read(context: Ctx) -> Result<GitHubInfo> {
    let fw_major_version_req = VersionReq::parse(FW_MAJOR_VERSION)?;

    let mut final_releases = Vec::new();
    let mut is_updated = false;
    let mut is_beta = false;

    let fw_releases =
        load_available_firmware_versions(!context.device.bootloader && context.allow_beta).await?;

    final_releases = fw_releases
        .into_iter()
        .filter(|release| {
            release.name == context.device.info.product.to_string() && {
                if context.device.info.product == Product::Defy {
                    return fw_major_version_req.matches(&parse_version(&release.version));
                }
                true
            }
        })
        .collect();

    if final_releases.is_empty() {
        let msg = "No GitHub firmware releases found";
        error!("{}", msg);
        bail!("{}", msg);
    }

    final_releases.sort_by(|a, b| b.version.cmp(&a.version));

    if context.device.bootloader {
        return Ok(GitHubInfo {
            firmwares: final_releases,
            is_updated: false,
            is_beta: false,
        });
    }

    is_updated = context.collected.version == final_releases[0].version;
    is_beta = context.collected.version.contains("beta");

    Ok(GitHubInfo {
        firmwares: final_releases,
        is_updated,
        is_beta,
    })
}

pub async fn load_available_firmware_versions(allow_beta: bool) -> Result<Vec<FirmwareRelease>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/repos/Dygmalab/Firmware-release/releases")
        .header("User-Agent", "Bazecor-Rust")
        .send()
        .await?;

    let gh_releases: Vec<GitHubRelease> = response.json().await?;

    let releases = gh_releases
        .into_iter()
        .filter_map(|release| {
            let release_data = release.name.split(' ').collect::<Vec<&str>>();
            if release_data.len() < 2 {
                return None;
            }
            let name = release_data[0].to_string();
            let version = release_data[1].to_string();
            if !allow_beta && version.contains("beta") {
                return None;
            }
            Some(FirmwareRelease {
                name,
                version,
                body: release.body,
                assets: release
                    .assets
                    .into_iter()
                    .map(|asset| FirmwareAsset {
                        name: asset.name,
                        url: asset.url,
                    })
                    .collect(),
            })
        })
        .collect();

    debug!("Firmware releases: {:#?}", releases);

    Ok(releases)
}

pub async fn download_firmware(
    type_selected: &str,
    info: &Hardware,
    firmware_list: &Vec<FirmwareRelease>,
    selected_firmware: usize,
) -> Result<Firmware> {
    let firmware = &firmware_list[selected_firmware];

    if type_selected == "default" {
        match info.info.product {
            Product::Raise => {
                let file_type_fw = "firmware.hex";
                let matched = firmware
                    .assets
                    .iter()
                    .find(|asset| asset.name == file_type_fw)
                    .context("Firmware not found")?;
                return Ok(Firmware {
                    firmware: obtain_fw_file(file_type_fw, &matched.url).await?,
                    sides: None,
                });
            }
            _ => match info.info.keyboard_type {
                KeyboardType::Wireless => {
                    let file_type_fw = "Wireless_neuron.hex";
                    let matched = firmware
                        .assets
                        .iter()
                        .find(|asset| asset.name == file_type_fw)
                        .context("Firmware not found")?;
                    return Ok(Firmware {
                        firmware: obtain_fw_file(file_type_fw, &matched.url).await?,
                        sides: None,
                    });
                }
                KeyboardType::Wired => {
                    let file_type_fw = "Wired_neuron.uf2";
                    let matched_fw = firmware
                        .assets
                        .iter()
                        .find(|asset| asset.name == file_type_fw)
                        .context("Firmware not found")?;

                    let file_type_fw_sides = "keyscanner.bin";
                    let matched_sides = firmware
                        .assets
                        .iter()
                        .find(|asset| asset.name == file_type_fw_sides)
                        .context("Firmware sides not found")?;

                    return Ok(Firmware {
                        firmware: obtain_fw_file(file_type_fw, &matched_fw.url).await?,
                        sides: Some(obtain_fw_file(file_type_fw_sides, &matched_sides.url).await?),
                    });
                }
                _ => bail!("Invalid keyboard type"),
            },
        }
    }

    bail!("Invalid firmware type")
}

pub async fn obtain_fw_file(file_type: &str, url: &str) -> Result<Vec<u8>> {
    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .header("User-Agent", "Bazecor-Rust")
        .send()
        .await?;

    match file_type {
        "keyscanner.bin" | "Wired_neuron.uf2" => {
            let bytes = response.bytes().await?;
            Ok(bytes.to_vec())
        }
        "Wireless_neuron.hex" | "firmware.hex" => {
            let text = response.text().await?;
            let re = Regex::new(r"[\r\n]+")?;
            let cleaned_text = re.replace_all(&text, "");
            let parts: Vec<&str> = cleaned_text.split(':').skip(1).collect();
            let firmware = &parts.join("");
            trace!("Firmware Hex: {}", firmware);
            let bytes = hex::decode(firmware)?;
            Ok(bytes)
        }
        _ => bail!("Invalid firmware file type"),
    }
}