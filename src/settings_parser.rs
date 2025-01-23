use getset::Getters;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

#[derive(Serialize, Deserialize, Getters)]
pub struct Settings {
    #[getset(get = "pub")]
    soundcloud_username: String,
    #[getset(get = "pub")]
    soundcloud_password: String,
}

pub fn parse_settings() -> Result<Settings, String> {
    //open settings file
    let mut file = match File::open("../settings/settings.json") {
        Ok(some) => some,
        Err(e) => {
            return Err(format!(
                "Error opening file \"../settings/settings.json\": {}",
                e
            ))
        }
    };

    //read file as one string
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_size) => (),
        Err(e) => {
            return Err(format!(
                "Error reading contents of settings.json file: {}",
                e
            ));
        }
    };

    //parse file contents as json
    let parsed_settings: Settings = match serde_json::from_str(&contents) {
        Ok(some) => some,
        Err(e) => {
            return Err(format!(
                "Error parsing settings.json file contents as json with binding schema: {}",
                e
            ));
        }
    };

    return Ok(parsed_settings);
}
