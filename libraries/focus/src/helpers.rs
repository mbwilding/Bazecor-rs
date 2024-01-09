use crate::color::*;
use anyhow::{anyhow, Result};
use std::str::FromStr;

#[allow(dead_code)]
pub(crate) fn string_to_numerical_vec<T: FromStr>(str: &str) -> Result<Vec<T>>
where
    <T as FromStr>::Err: std::fmt::Debug,
{
    str.split_ascii_whitespace()
        .map(|part| {
            part.parse::<T>()
                .map_err(|e| anyhow!("Failed to parse response: {:?}", e))
        })
        .collect()
}

#[allow(dead_code)]
pub(crate) fn numerical_vec_to_string<T: ToString>(data: &[T]) -> String {
    data.iter()
        .map(|num| num.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

#[allow(dead_code)]
pub(crate) fn string_to_rgb_vec(str: &str) -> Result<Vec<RGB>> {
    str.split_ascii_whitespace()
        .collect::<Vec<&str>>()
        .chunks(3)
        .map(|chunk| {
            let r = chunk[0].parse()?;
            let g = chunk[1].parse()?;
            let b = chunk[2].parse()?;

            Ok(RGB { r, g, b })
        })
        .collect()
}

#[allow(dead_code)]
pub(crate) fn rgb_vec_to_string(data: &[RGB]) -> String {
    data.iter()
        .map(|rgb| format!("{} {} {}", rgb.r, rgb.g, rgb.b))
        .collect::<Vec<String>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_numerical_vec() {
        let input = "41 30 31 32 33 34 35 0 0";
        let expected = vec![41, 30, 31, 32, 33, 34, 35, 0, 0];
        let actual = string_to_numerical_vec::<u8>(input).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_numerical_vec_to_string() {
        let input: Vec<u8> = vec![41, 30, 31, 32, 33, 34, 35, 0, 0];
        let expected = "41 30 31 32 33 34 35 0 0";
        let actual = numerical_vec_to_string(&input);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_string_to_rgb_vec() {
        let input = "\
        255 196 0 \
        0 0 254 \
        24 0 0 \
        0 0 255 \
        231 255 0 \
        0 0 254 \
        234 0 0 \
        52 255 0 \
        255 0 232 \
        0 0 77 \
        168 87 125 \
        0 235 19 \
        20 0 36 \
        219 85 0 \
        126 129 255 \
        9 0 0 \
        0 0 0 \
        0 255 172 \
        0 0 0 \
        0 0 0 \
        255 58 0 \
        0";
        let input = "\
        255 196 0 0 \
        0 254 24 0 \
        0 0 0 255 \
        231 255 0 0 \
        0 254 234 0 \
        0 52 255 0 \
        255 0 232 0 \
        0 77 168 87 \
        125 0 235 19 \
        20 0 36 219 \
        85 0 126 129 \
        255 9 0 0 \
        0 0 0 0 \
        255 172 0 0 \
        0 0 0 0 \
        255 58 0 0";
        let expected = vec![
            RGB {
                r: 255,
                g: 196,
                b: 0,
            },
            RGB { r: 0, g: 0, b: 254 },
            RGB { r: 24, g: 0, b: 0 },
            RGB { r: 0, g: 0, b: 255 },
            RGB {
                r: 231,
                g: 255,
                b: 0,
            },
            RGB { r: 0, g: 0, b: 254 },
            RGB { r: 234, g: 0, b: 0 },
            RGB {
                r: 52,
                g: 255,
                b: 0,
            },
            RGB {
                r: 255,
                g: 0,
                b: 232,
            },
            RGB { r: 0, g: 0, b: 77 },
            RGB {
                r: 168,
                g: 87,
                b: 125,
            },
            RGB {
                r: 0,
                g: 235,
                b: 19,
            },
            RGB { r: 20, g: 0, b: 36 },
            RGB {
                r: 219,
                g: 85,
                b: 0,
            },
            RGB {
                r: 126,
                g: 129,
                b: 255,
            },
            RGB { r: 9, g: 0, b: 0 },
            RGB { r: 0, g: 0, b: 0 },
            RGB {
                r: 0,
                g: 255,
                b: 172,
            },
            RGB { r: 0, g: 0, b: 0 },
            RGB { r: 0, g: 0, b: 0 },
            RGB {
                r: 255,
                g: 58,
                b: 0,
            },
            RGB { r: 0, g: 0, b: 0 },
        ];

        let actual = string_to_rgb_vec(input).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_rgb_vec_to_string() {
        let input = vec![
            RGB {
                r: 41,
                g: 30,
                b: 31,
            },
            RGB {
                r: 32,
                g: 33,
                b: 34,
            },
            RGB {
                r: 35,
                g: 212,
                b: 10,
            },
        ];
        let expected = "41 30 31 32 33 34 35 212 10";

        let result = rgb_vec_to_string(&input);
        assert_eq!(expected, result);
    }
}
