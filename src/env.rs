use lazy_static::lazy_static;
use serde::Deserialize;
use std::fs::File;

const CONFIG_PATH: &str = "./config.json";

lazy_static! {
    static ref CONFIG: Config = Config::init();
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub listen: String,
    pub base_url: String,
    pub allow_origin: String,
}

impl Config {
    pub fn init() -> Self {
        let file = File::open(CONFIG_PATH).expect("config.json file missing");
        serde_json::from_reader(file).expect("config.json file in wrong format")
    }

    pub fn get() -> Self {
        CONFIG.to_owned()
    }
}
