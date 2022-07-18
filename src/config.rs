use std::fs::File;
use std::io::Read;
use toml;
use log;
use anyhow::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub listen_port: Option<u32>,
    pub systems: Vec<System>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct System {
    pub host: Option<String>,
    pub url: Option<String>,
    pub user: Option<String>,
    pub pass: Option<String>,
    pub sn: Option<String>,
}

impl Config {
    pub fn from_file(file: &str) -> Result<Config> {
        let mut f = File::open(file)?;
        let mut s = String::new();
        let _ = f.read_to_string(&mut s);
        let config: Config = toml::from_str(&s)?;
        log::debug!("config {:?}", config);
        Ok(config)
    }
}
