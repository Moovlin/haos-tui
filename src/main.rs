use log::{info, warn, debug, error};

use tokio::io::Result;
use haoscli::types::{HomeAssistantConnection, self};

use serde::Deserialize;

use std::{fs, env};
//use toml::{toml, from_str};

use clap::{arg, command};

#[derive(Deserialize)]
struct Config {
    url: String,
    token: String,
    client_id: String,
}

impl Config {
    pub fn new(args: Args) -> Self{
        read_toml(args.config_path)
    }
}

fn read_toml(config_path: String) -> Config {
    let config_string = fs::read_to_string(config_path).expect("Could not access the file");
    toml::from_str(&config_string).unwrap()
}

#[derive(Debug, Clone)]
struct Args {
    config_path: String, 
}

impl Default for Args {
    fn default() -> Self {
        let home_dir = env::var("HOME").expect("User hasn't defined their HOME variable. Supply a path manually");
        let env_config_path = format!("{}/.config/haos-rs-client/config.toml", home_dir);
        info!("{}", env_config_path);
        Args {config_path: env_config_path}
    }
}

fn main() -> Result<()>{
    env_logger::init();
    let matches = command!()
        .arg(arg!(-c --config <FILE>).required(false).default_value(Args::default().config_path.as_str()))
        .get_matches();
    
    //let rt = tokio::runtime::Runtime::new().unwrap();
    let rt = tokio::runtime::Builder::new_current_thread().enable_all()
        .build()?;
    let args = Args {config_path: String::from(matches.get_one::<String>("config").unwrap())};
    
    let config = Config::new(args);
    let haos_conn = HomeAssistantConnection::new(config.url, config.client_id);
    haos_conn.write().unwrap().set_long_live_token(config.token);
    let events = rt.block_on(haos_conn.try_read().unwrap().get_events()).unwrap();
    for evnt in events {
        info!("Event: {}, listeners: {}", evnt.event, evnt.listener_count);
    }

    Ok(())
}
