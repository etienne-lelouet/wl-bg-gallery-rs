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
    pub bg_duration_seconds: Option<u64>,
}

enum ReadConfigError {
    OpenError(String),
    ReadError(String),
    ParseError(String)
}

impl ParsedConfig {
    fn empty() -> ParsedConfig{
	return ParsedConfig {
	    path: None,
	    bg_duration_seconds: None,
	};
    }

    fn read_config(default_config_path: String) -> Result<ParsedConfig, ReadConfigError> {
	let mut file = match File::open(&default_config_path) {
            Ok(file) => file,
            Err(error) => {
		println!();
		return Err(ReadConfigError::OpenError(format!("failed to open {}: {}", default_config_path, error)));
	    },
	};

	let mut config_str = String::new();
	if let Err(error) = file.read_to_string(&mut config_str) {
	    println!("failed to read {}: {}", default_config_path, error);
	    return Err(ReadConfigError::ReadError(format!("failed to read {}: {}", default_config_path, error)));
	}

	match toml::from_str(config_str.as_str()) {
            Ok(parsed_config) => Ok(parsed_config),
            Err(error) => {
		println!("failed to deserialize {}: {}", default_config_path, error);
		return Err(ReadConfigError::ParseError(format!("failed to parse {}: {}", default_config_path, error)));
	    }
	}
    }
}

const DEFAULT_PATH: &str = "~/Pictures/wallpaper";
const DEFAULT_BG_DURATION_SECONDS: u64 = 15;

#[derive(Debug)]
pub struct Config {
    pub path: String,
    pub bg_duration_seconds: u64,
    pub authorized_formats: Vec<ImageFormat>,
}

pub fn expand_tilde(mut str: String) -> String {
    if str.len() <= 0 {
	return str;
    }

    if str.chars().nth(0).unwrap() != '~' {
	return str;
    }

    let mut home = match std::env::var("HOME") {
        Ok(home) => home,
        Err(_) => {
	    return str;
	}
    };
    home = format!("{home}/");
    str.replace_range(0..1, &home);
    return str;
}

impl Config {
    fn default() -> Config{
	return Config {
	    path: expand_tilde(String::from(DEFAULT_PATH)),
	    bg_duration_seconds: DEFAULT_BG_DURATION_SECONDS,
	    authorized_formats: vec!(
		ImageFormat::Jpeg,
		ImageFormat::WebP,
		ImageFormat::Png,
		ImageFormat::Tiff
	    ),
	};
    }

    fn from(config: &mut Config, parsed_config: ParsedConfig) {
	if let Some(path) = parsed_config.path {
	    config.path = expand_tilde(path);
	}

	if let Some(bg_duration_seconds) = parsed_config.bg_duration_seconds {
	    config.bg_duration_seconds = bg_duration_seconds;
	}

    }

    pub fn get_config() -> Config {
	let mut config = Config::default();
	let etc_parsed_conf = ParsedConfig::read_config(String::from("/etc/wl-bg-gallery/config.toml"));
	match etc_parsed_conf {
	    Ok(etc_parsed_conf) => Config::from(&mut config, etc_parsed_conf),
	    Err(error) => {
		match error {
		    ReadConfigError::ReadError(_) => panic!("failed to read config"),
		    ReadConfigError::ParseError(_) => panic!("failed to read config"),
		    ReadConfigError::OpenError(_) => (),
		}
	    }
	};

	let home_parsed_conf = ParsedConfig::read_config(expand_tilde(String::from("~/.config/wl-bg-gallery/config.toml")));
	match home_parsed_conf {
	    Ok(home_parsed_conf) => Config::from(&mut config, home_parsed_conf),
	    Err(error) => {
		match error {
		    ReadConfigError::ReadError(_) => panic!("failed to read config"),
		    ReadConfigError::ParseError(_) => panic!("failed to read config"),
		    ReadConfigError::OpenError(_) => (),
		}
	    }
	};

	return config;

    }
}
