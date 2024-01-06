use anyhow::{bail, Error, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// The LED RGB color.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RGB {
    /// Red component of the color.
    pub r: u8,
    /// Green component of the color.
    pub g: u8,
    /// Blue component of the color.
    pub b: u8,
}

impl FromStr for RGB {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<u8> = s
            .split_whitespace()
            .map(|part| part.parse::<u8>())
            .collect::<Result<Vec<_>, _>>()?;
        if parts.len() == 3 {
            Ok(Self {
                r: parts[0],
                g: parts[1],
                b: parts[2],
            })
        } else {
            bail!("Invalid color format");
        }
    }
}

/// The LED RGBW color.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RGBW {
    /// Red component of the color.
    pub r: u8,
    /// Green component of the color.
    pub g: u8,
    /// Blue component of the color.
    pub b: u8,
    /// White component of the color.
    pub w: u8,
}

impl FromStr for RGBW {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<u8> = s
            .split_whitespace()
            .map(|part| part.parse::<u8>())
            .collect::<Result<Vec<_>, _>>()?;
        if parts.len() == 4 {
            Ok(Self {
                r: parts[0],
                g: parts[1],
                b: parts[2],
                w: parts[3],
            })
        } else {
            bail!("Invalid color format");
        }
    }
}
