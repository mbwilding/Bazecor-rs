use anyhow::{bail, Context, Result};
use dygma_focus::hardware::{DeviceType, Hardware, Product};
use log::{debug, error, trace};
use regex::Regex;
use semver::{Version, VersionReq};
use serde::Deserialize;
use std::fmt::Display;
use tokio::join;

const FW_MAJOR_VERSION: &str = "1.x";
const USER_AGENT: &str = "Bazecor-Rust";
const GITHUB_USER: &str = "Dygmalab";
const GITHUB_REPOSITORY: &str = "Firmware-release";

#[derive(Debug, Clone)]
pub struct FirmwareRelease {
    pub name: String,
    pub version: String,
    pub body: String,
    pub assets: Vec<FirmwareAsset>,
    pub beta: bool,
}

impl Display for FirmwareRelease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.version)
    }
}

#[derive(Debug, Clone)]
pub struct FirmwareAsset {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct Firmware {
    pub firmware: FirmwareNode,
    pub sides: Option<FirmwareNode>,
}

#[derive(Debug, Clone)]
pub struct FirmwareNode {
    pub name: String,
    pub bytes: Vec<u8>,
    pub hex_raw: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Ctx {
    pub device: Hardware,
    pub collected: Collected,
    pub allow_beta: bool,
}

#[derive(Debug, Clone)]
pub struct Collected {
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    pub name: String,
    pub body: String,
    #[serde(rename = "prerelease")]
    pub beta: bool,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    #[serde(rename = "browser_download_url")]
    pub url: String,
}

#[derive(Debug, Clone)]
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

    let fw_releases =
        load_available_firmware_versions(!context.device.bootloader && context.allow_beta).await?;

    let mut final_releases = fw_releases
        .into_iter()
        .filter(|release| {
            release.name == context.device.info.product.to_string() && {
                if context.device.info.product == Product::Defy {
                    return fw_major_version_req.matches(&parse_version(&release.version));
                }
                true
            }
        })
        .collect::<Vec<_>>();

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

    let latest = &final_releases[0];
    let is_updated = context.collected.version == latest.version;
    let is_beta = latest.beta;

    Ok(GitHubInfo {
        firmwares: final_releases,
        is_updated,
        is_beta,
    })
}

pub async fn load_available_firmware_versions(allow_beta: bool) -> Result<Vec<FirmwareRelease>> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "https://api.github.com/repos/{}/{}/releases",
            GITHUB_USER, GITHUB_REPOSITORY
        ))
        .header("User-Agent", USER_AGENT)
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
            let is_beta = release.beta || version.contains("-beta");
            if !allow_beta && is_beta {
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
                beta: is_beta,
            })
        })
        .collect();

    trace!("Firmware releases: {:#?}", releases);

    Ok(releases)
}

pub async fn download_firmware(
    hardware: &Hardware,
    firmware_release: &FirmwareRelease,
) -> Result<Firmware> {
    match hardware.info.product {
        Product::Raise => download_firmware_raise(firmware_release).await,
        Product::Defy => match hardware.info.device_type {
            DeviceType::Wireless => {
                download_firmware_defy(firmware_release, "Wireless_neuron.hex").await
            }
            DeviceType::Wired => download_firmware_defy(firmware_release, "Wired_neuron.uf2").await,
            _ => bail!("Invalid device type"),
        },
    }
}

async fn download_firmware_raise(firmware_release: &FirmwareRelease) -> Result<Firmware> {
    let firmware_file = "firmware.hex";
    let matched = firmware_release
        .assets
        .iter()
        .find(|asset| asset.name == firmware_file)
        .context("Firmware not found")?;

    let fw = obtain_firmware_file(firmware_file, &matched.url).await?;

    Ok(Firmware {
        firmware: fw,
        sides: None,
    })
}

async fn download_firmware_defy(
    firmware_release: &FirmwareRelease,
    firmware_file_name: &str,
) -> Result<Firmware> {
    let matched_fw = firmware_release
        .assets
        .iter()
        .find(|asset| asset.name == firmware_file_name)
        .context("Firmware not found")?;

    let firmware_sides_file_name = "keyscanner.bin";
    let matched_sides = firmware_release
        .assets
        .iter()
        .find(|asset| asset.name == firmware_sides_file_name)
        .context("Firmware sides not found")?;

    let (firmware, sides) = join!(
        obtain_firmware_file(firmware_file_name, &matched_fw.url),
        obtain_firmware_file(firmware_sides_file_name, &matched_sides.url)
    );

    Ok(Firmware {
        firmware: firmware?,
        sides: Some(sides?),
    })
}

pub async fn obtain_firmware_file(firmware_file_name: &str, url: &str) -> Result<FirmwareNode> {
    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?;

    debug!("Downloading firmware [{}]: {}", firmware_file_name, url);

    if firmware_file_name.ends_with(".hex") {
        let text = response.text().await?;
        let regex = Regex::new(r"[\r\n]+")?;
        let single_line = regex.replace_all(&text, "");
        let parts: Vec<&str> = single_line.split(':').skip(1).collect();
        let firmware = &parts.join("");
        let bytes = hex::decode(firmware)?;
        let firmware_node = FirmwareNode {
            name: firmware_file_name.to_string(),
            bytes,
            hex_raw: Some(text),
        };

        Ok(firmware_node)
    } else {
        let bytes = response.bytes().await?.to_vec();
        let firmware_node = FirmwareNode {
            name: firmware_file_name.to_string(),
            bytes,
            hex_raw: None,
        };

        Ok(firmware_node)
    }
}
