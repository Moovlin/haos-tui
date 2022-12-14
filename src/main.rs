use log::info;

use haoscli::types::HomeAssistantConnection;
use tokio::io::Result;

use serde::Deserialize;

use std::{env, fs, thread::spawn};

use std::sync::{Arc, Condvar, Mutex};

mod fetcher;
mod key_handler;
mod ui;
mod ui_types;

use clap::{arg, command};

use crate::fetcher::fetcher;
use crate::key_handler::key_handler;
use crate::ui_types::{UiState};

use log::LevelFilter;

#[derive(Deserialize)]
struct Config {
    url: String,
    token: String,
    client_id: String,
    log_level: LogLevel,
    poll_rate: u64,
}

#[derive(Deserialize)]
enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Config {
    pub fn new(args: Args) -> Self {
        read_toml(args.config_path)
    }
}

fn read_toml(config_path: String) -> Config {
    let config_string = fs::read_to_string(config_path).expect("Could not access the file");
    let config_toml = toml::from_str(&config_string);
    match config_toml {
        Ok(config) => config,
        Err(e) => panic!("Couldn't access file. Threw {}", e),
    }
}

#[derive(Debug, Clone)]
struct Args {
    config_path: String,
}

impl Default for Args {
    fn default() -> Self {
        let home_dir = env::var("HOME")
            .expect("User hasn't defined their HOME variable. Supply a path manually");
        let env_config_path = format!("{}/.config/haos-rs-client/config.toml", home_dir);
        info!("{}", env_config_path);
        Args {
            config_path: env_config_path,
        }
    }
}

fn main() -> Result<()> {
    let matches = command!()
        .arg(
            arg!(-c --config <FILE>)
                .required(false)
                .default_value(Args::default().config_path.as_str()),
        )
        .get_matches();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let args = Args {
        config_path: String::from(
            matches
                .get_one::<String>("config")
                .expect("Did not supply a value for config and a home value wasn't set."),
        ),
    };

    let config = Config::new(args);
    let log_level: LevelFilter = match config.log_level {
        LogLevel::Off => LevelFilter::Off,
        LogLevel::Error => LevelFilter::Error,
        LogLevel::Warn => LevelFilter::Warn,
        LogLevel::Info => LevelFilter::Info,
        LogLevel::Debug => LevelFilter::Debug,
        LogLevel::Trace => LevelFilter::Trace,
    };
    simple_logging::log_to_file("log.txt", log_level)
        .expect("File doesn't exist. This should create the file or smthing I guess.");

    let haos_conn = HomeAssistantConnection::new(config.url, config.client_id);
    haos_conn
        .write()
        .expect("Couldn't get the write lock on the token")
        .set_long_live_token(config.token);

    let locked_state = Arc::new(Mutex::new(UiState::default()));
    locked_state
        .lock()
        .expect("Should be the only person with access to this")
        .events
        .1
        .select(Some(0));
    locked_state
        .lock()
        .expect("Should be the only person with access to this")
        .services
        .1
        .select(Some(0));
    locked_state
        .lock()
        .expect("Should be the only person with access to this")
        .states
        .1
        .select(Some(0));

    let convar = Arc::new(Condvar::new());

    let mut convar_for_painter = Arc::clone(&convar);
    let mut state_for_painter = Arc::clone(&locked_state);

    let convar_for_fetcher = Arc::clone(&convar);
    let mut state_for_fetcher = Arc::clone(&locked_state);

    let mut convar_for_keyhandler = Arc::clone(&convar);
    let mut state_for_keyhandler = Arc::clone(&locked_state);
    //let key_handler_joiner = rt.spawn(key_handler(&mut locked_state, &mut convar));

    let key_handler_joiner = spawn(move || {
        rt.block_on(async move {
            key_handler(&mut state_for_keyhandler, &mut convar_for_keyhandler).await;
        })
    });

    let fetcher_handler = spawn(move || {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                fetcher(
                    &haos_conn,
                    &convar_for_fetcher,
                    &mut state_for_fetcher,
                    config.poll_rate,
                )
                .await;
            })
    });

    ui::draw_ui(&mut state_for_painter, &mut convar_for_painter);

    key_handler_joiner
        .join()
        .expect("We were unable to join the key_handler");
    fetcher_handler
        .join()
        .expect("We were unable to join the fetcher");
    Ok(())
}
