use std::{fs::File, io::Read};

use image::ImageFormat;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub enum FitType {
    BestFit,
    NextFile
}

#[derive(Deserialize)]
struct ParsedConfig {
    pub path: Option<String>,
    pub fit_type: Option<FitType>,
    pub bg_duration_seconds: Option<u64>
}

impl ParsedConfig {
    fn empty() -> ParsedConfig{
	return ParsedConfig {
	    path: None,
	    fit_type: None,
	    bg_duration_seconds: None
	};
    }

    fn read_config(default_config_path: String) -> ParsedConfig {
	let mut file = match File::open(&default_config_path) {
            Ok(file) => file,
            Err(error) => {
		println!("failed to open {}: {}", default_config_path, error);
		return ParsedConfig::empty();
	    },
	};

	let mut config_str = String::new();
	if let Err(error) = file.read_to_string(&mut config_str) {
	    println!("failed to read {}: {}", default_config_path, error);
	    return ParsedConfig::empty();
	}

	match toml::from_str(config_str.as_str()) {
            Ok(parsed_config) => parsed_config,
            Err(error) => {
		println!("failed to deserialize {}: {}", default_config_path, error);
		ParsedConfig::empty()
	    }
	}
    }
}

const DEFAULT_PATH: &str = "~/Pictures/wallpaper";
const DEFAULT_BG_DURATION_SECONDS: u64 = 15;
const DEFAULT_FIT_TYPE: FitType = FitType::NextFile;

#[derive(Debug)]
pub struct Config {
    pub path: String,
    pub fit_type: FitType,
    pub bg_duration_seconds: u64,
    pub authorized_formats: Vec<ImageFormat>
}

impl Config {
    fn default() -> Config{
	return Config {
	    path: String::from(DEFAULT_PATH),
	    fit_type: DEFAULT_FIT_TYPE,
	    bg_duration_seconds: DEFAULT_BG_DURATION_SECONDS,
	    authorized_formats: vec!(
		ImageFormat::Jpeg,
		ImageFormat::WebP,
		ImageFormat::Png
	    )
	};
    }

    fn from(parsed_config: ParsedConfig) -> Config {
	let mut config = Config::default();

	if let Some(path) = parsed_config.path {
	    config.path = String::from(path);
	}

	if let Some(fit_type) = parsed_config.fit_type {
	    config.fit_type = fit_type;
	}

	if let Some(bg_duration_seconds) = parsed_config.bg_duration_seconds {
	    config.bg_duration_seconds = bg_duration_seconds;
	}
	return config;
    }

    pub fn get_config() -> Config{
	let parsed_config = ParsedConfig::read_config(
	    String::from(".config")
	);
	return Config::from(parsed_config);
    }
}
