use std::{fs, io, path::{Path, PathBuf}, time::Duration};

use futuremod_data::plugin::{PluginInfo, PluginInfoContent};
use iced::Color;
use palette::{Hsl, FromColor, rgb::Rgb, Mix};
use anyhow::{anyhow, bail};

// Yoinked from https://github.com/iced-rs/iced/blob/master/style/src/theme/palette.rs because functions are not public

pub fn to_hsl(color: Color) -> Hsl {
    Hsl::from_color(Rgb::from(color))
}

pub fn from_hsl(hsl: Hsl) -> Color {
    Rgb::from_color(hsl).into()
}

pub fn darken(color: Color, amount: f32) -> Color {
    let mut hsl = to_hsl(color);

    hsl.lightness = if hsl.lightness - amount < 0.0 {
        0.0
    } else {
        hsl.lightness - amount
    };

    from_hsl(hsl)
}

pub fn lighten(color: Color, amount: f32) -> Color {
    let mut hsl = to_hsl(color);

    hsl.lightness = if hsl.lightness + amount > 1.0 {
        1.0
    } else {
        hsl.lightness + amount
    };

    from_hsl(hsl)
}

pub fn is_dark(color: Color) -> bool {
    to_hsl(color).lightness < 0.6
}

#[allow(unused)]
pub fn deviate(color: Color, amount: f32) -> Color {
    if is_dark(color) {
        lighten(color, amount)
    } else {
        darken(color, amount)
    }
}

#[allow(unused)]
pub fn mix(a: Color, b: Color, factor: f32) -> Color {
    let a_lin = Rgb::from(a).into_linear();
    let b_lin = Rgb::from(b).into_linear();

    let mixed = a_lin.mix(b_lin, factor);
    Rgb::from_linear(mixed).into()
}

pub fn alpha(a: Color, alpha: f32) -> Color {
    let mut a = a.clone();
    a.a = alpha;

    a
}

/// Waits for the given duration of milliseconds.
#[allow(unused)]
pub async fn wait_for_ms(duration: u64) {
    tokio::time::sleep(Duration::from_millis(duration)).await
}

/// Check if the given folder contains a valid plugin.
pub fn is_plugin_folder(folder: &PathBuf) -> Result<bool, io::Error> {
    if !folder.exists() || folder.is_file() {
        return Ok(false);
    }

    let mut contains_manifest = false;
    let mut contains_main = false;

    for entry in fs::read_dir(folder)? {
        let entry = entry?;

        // Skip directories
        if entry.file_type()?.is_dir() {
            continue
        }

        match entry.file_name().to_str() {
            Some("info.toml") => contains_manifest = true,
            Some("main.lua") | Some("main.luau") => contains_main = true,
            _ => (),
        }

        if contains_main && contains_manifest {
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn get_plugin_info_of_local_folder(folder: &PathBuf) -> Result<PluginInfo, anyhow::Error> {
    let expected_info_path = Path::join(&folder, "info.toml");

    if !expected_info_path.exists() {
        bail!("Folder does not contain a manifest file");
    }

    let content = fs::read_to_string(expected_info_path)
        .map_err(|e| anyhow!("Could not get plugin info: {}", e))?;

    let plugin_info = toml::from_str::<PluginInfoContent>(&content).map_err(|e| anyhow!("Invalid manifest file: {}", e))?;

    Ok(PluginInfo{
        path: folder.clone(),
        authors: plugin_info.authors,
        name: plugin_info.name,
        version: plugin_info.version,
        dependencies: plugin_info.dependencies,
        description: plugin_info.description,
    })
}