use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Config {
    pub music_dir: String,
}

impl Config {
    pub fn from_config_file() -> Config {
        let mut config_file_dir: PathBuf = dirs::config_dir().unwrap();
        config_file_dir.push("rsmus/rsmusrc");
        let mut config_file = File::open(config_file_dir).unwrap();
        let mut config_data = String::new();
        config_file.read_to_string(&mut config_data);
        let config: Config = toml::from_str(&config_data).unwrap();
        return config;
    }
}
