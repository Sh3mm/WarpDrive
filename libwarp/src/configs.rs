use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};
use std::fs::{create_dir_all, File, read_dir};
use std::io::{Error};
use serde::{Deserialize, Serialize};

fn get_config_path() -> PathBuf {
    #[allow(deprecated)]
    let config_path = env::home_dir().unwrap().canonicalize().unwrap().join(".config/warpcli");
    create_dir_all(&config_path).expect("Unable to create configs folder");
    return config_path.canonicalize().expect("Error getting config path");
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub link_path: String,
    pub local: String,
    pub remote: String,
    pub update_rt: usize
}

impl Config {
    pub fn get_all_names() -> HashSet<String> {
        let configs = get_config_path();
        return read_dir(configs).unwrap().map(|v|{
            v.unwrap().file_name().to_str().unwrap().to_string()
        }).collect::<HashSet<String>>()
    }

    pub fn new(name: &str, local: &str, remote: &str, update_rt: usize) -> Self {
        let config_path = get_config_path().join(name);
        let local_path = Path::new(local).canonicalize().expect("Local path does not exist");

        return Config {
            link_path: config_path.to_str().unwrap().to_string(),
            local: local_path.to_str().unwrap().to_string(),
            remote: remote.to_string(),
            update_rt,
        }
    }

    pub fn load(name: &str) -> Result<Self, Error> {
        let path = get_config_path().join(name).join("info.json");

        let config: Config = serde_json::from_reader(
            File::open(&path)?
        )?;

        return Ok(config)
    }

    pub fn save(self) {
        let link_path = Path::new(&self.link_path);
        create_dir_all(link_path).expect("Unable to create configs/link folder");
        let path = link_path.join("info.json");

        serde_json::to_writer(
            File::create(path).unwrap(),
            &self
        ).expect("Error while saving link info");
    }
}