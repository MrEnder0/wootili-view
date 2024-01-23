use std::fs::File;

use image::imageops::FilterType;
use ron::{
    de::from_reader,
    ser::{to_string_pretty, PrettyConfig},
};
use scorched::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub config_version: u8,
    pub brightness: u8,
    pub reduce_bright_effects: bool,
    pub screen: usize,
    pub display_rgb_preview: bool,
    pub downscale_method_index: u8,
    pub frame_sleep: u64,
    pub red_shift_fix: bool,
}

pub static CONFIG_VERSION: u8 = 1;

pub fn read_config() -> Config {
    let config_file = File::open(crate::paths::config_path().join("config.ron"))
        .log_expect(LogImportance::Error, "Unable to open config file");
    let config: Config = match from_reader(config_file) {
        Ok(x) => x,
        Err(e) => {
            log_this(LogData {
                importance: LogImportance::Error,
                message: format!(
                    "Unable to read config file because of the following error:\n{}",
                    e
                ),
            });

            std::process::exit(0);
        }
    };

    config
}

pub fn gen_config() {
    let data = Config {
        config_version: CONFIG_VERSION,
        brightness: 100,
        reduce_bright_effects: false,
        screen: 0,
        display_rgb_preview: true,
        downscale_method_index: 1,
        frame_sleep: 10,
        red_shift_fix: false,
    };

    let config = PrettyConfig::new()
        .depth_limit(3)
        .separate_tuple_members(true)
        .enumerate_arrays(true);

    let config_str = to_string_pretty(&data, config).expect("Serialization failed");
    std::fs::write(crate::paths::config_path().join("config.ron"), config_str)
        .log_expect(LogImportance::Error, "Unable to write config file");

    log_this(LogData {
        importance: LogImportance::Info,
        message: "Config file has been generated.".to_string(),
    });
}

pub fn config_exists() -> bool {
    std::path::Path::new(
        crate::paths::config_path()
            .join("config.ron")
            .to_str()
            .unwrap(),
    )
    .exists()
}

pub enum ConfigChange {
    Brightness(u8),
    ReduceBrightEffects(bool),
    Screen(usize),
    DisplayRgbPreview(bool),
    DownscaleMethod(FilterType),
    FrameSleep(u64),
    RedShiftFix(bool),
}

pub fn change_config_option(new: ConfigChange) {
    let mut data = read_config();

    match new {
        ConfigChange::Brightness(x) => data.brightness = x,
        ConfigChange::ReduceBrightEffects(x) => data.reduce_bright_effects = x,
        ConfigChange::Screen(x) => data.screen = x,
        ConfigChange::DisplayRgbPreview(x) => data.display_rgb_preview = x,
        ConfigChange::DownscaleMethod(x) => {
            data.downscale_method_index = filter_to_downscale_index(x)
        }
        ConfigChange::FrameSleep(x) => data.frame_sleep = x,
        ConfigChange::RedShiftFix(x) => data.red_shift_fix = x,
    }

    let config = PrettyConfig::new()
        .depth_limit(3)
        .separate_tuple_members(true)
        .enumerate_arrays(true);

    let config_str = to_string_pretty(&data, config).expect("Serialization failed");
    std::fs::write(crate::paths::config_path().join("config.ron"), config_str)
        .log_expect(LogImportance::Error, "Unable to write config file");
}

pub fn downscale_index_to_filter(index: u8) -> FilterType {
    match index {
        0 => FilterType::Nearest,
        1 => FilterType::Triangle,
        2 => FilterType::CatmullRom,
        3 => FilterType::Gaussian,
        4 => FilterType::Lanczos3,
        _ => {
            logf!(
                Warning,
                "Invalid downscale method index {} defaulting to nearest neighbor",
                index.to_string()
            );

            FilterType::Nearest
        }
    }
}

fn filter_to_downscale_index(filter: FilterType) -> u8 {
    match filter {
        FilterType::Nearest => 0,
        FilterType::Triangle => 1,
        FilterType::CatmullRom => 2,
        FilterType::Gaussian => 3,
        FilterType::Lanczos3 => 4,
        _ => {
            logf!(
                Warning,
                "Invalid downscale method {:?} defaulting to nearest neighbor",
                filter
            );

            0
        }
    }
}