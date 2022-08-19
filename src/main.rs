use log::{info, warn, debug, error};

use string_builder::Builder;

use tokio::io::Result;
use haoscli::types::{HomeAssistantConnection, Event, self};


use serde::Deserialize;

use serde_json::{to_string_pretty, json};

use std::{fs, env,option::Option, thread::spawn};

use std::sync::{Arc, Mutex, Condvar};


mod ui;
mod key_handler;
mod fetcher;


use clap::{arg, command};

use crate::key_handler::key_handler;
use crate::fetcher::fetcher;

use log::LevelFilter;


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
        let home_dir = env::var("HOME").expect("User hasn't defined their HOME variable. Supply a path manually");
        let env_config_path = format!("{}/.config/haos-rs-client/config.toml", home_dir);
        info!("{}", env_config_path);
        Args {config_path: env_config_path}
    }
}

fn main() -> Result<()>{
    simple_logging::log_to_file("log.txt", LevelFilter::Trace).expect("File doesn't exist. This should create the file or smthing I guess.");
    let matches = command!()
        .arg(arg!(-c --config <FILE>).required(false).default_value(Args::default().config_path.as_str()))
        .get_matches();
    
    /*
    let rt = tokio::runtime::Builder::new_current_thread().enable_all()
        .build()?;
    */
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let args = Args {config_path: String::from(matches.get_one::<String>("config").expect("Did not supply a value for config and a home value wasn't set."))};
    
    let config = Config::new(args);
    let haos_conn = HomeAssistantConnection::new(config.url, config.client_id);

    
    /*
    let working_haos_conn = match haos_conn.read() { Ok(v) => v, Err(e) => panic!("Couldn't get read lock: {}", e)};
    info!("Firing an event to test that things work. This is annoying but hey, I'm provind a point here");
    //let resp = rt.block_on(haos_conn.write().expect("Could not get write lock").fire_event(String::from("garbage_collection_loaded"), Some("{\"data\": \"\"}")));
    let resp = match rt.block_on(working_haos_conn.fire_event(String::from("garbage_collection_loaded"), Some("{\"data\": \"\"}"))) {
        Ok(v) => v,
        Err(e) => panic!("Couldn't fire the event: {:?}", e),
    };
    info!("{}", resp);
    drop(working_haos_conn);
    */

    let working_haos_conn = match haos_conn.read() {Ok(v) => v, Err(e) => panic!("Couldn't get read lock: {}", e)};
    info!("Getting the service list to test that things work.");
    let services = match rt.block_on(working_haos_conn.get_services()) {Ok(v) => v, Err(_) => panic!("Couldn't get the services")};
    let mut string_builder: Builder = Builder::default();
    for service in services.iter() {
        string_builder.append(format!("{}\n", service.domain));
            //string_builder.append(serde_json::to_string_pretty(service.services.as_object().unwrap()).unwrap());
            string_builder.append("\n");
    }
    let output_string = match string_builder.string() {Ok(v) => v, Err(e) => panic!("Couldn't build string: {}", e)};
    info!("{}", output_string);
    drop(working_haos_conn);

    /*
    let working_haos_conn = match haos_conn.read() {Ok(v) => v, Err(e) => panic!("Couldn't get read lock: {}",e)};
    let test_service = types::Service {domain: String::from("light"), services: json!("turn_on")};
    let test_entity = types::RequestEntityObject{entity_id: "light.floor_lamp_level_light_color_on_off"};

    let resp = match rt.block_on(working_haos_conn.set_service(test_service, Some(&test_entity))) {
        Ok(v) => v,
        Err(_) => panic!("Couldn't get a response"),
    };

    info!("{}", resp);
    drop(working_haos_conn);
    */

    let mut locked_state = Arc::new(Mutex::new(
            ui::UiState {
                active: true,
                events: vec!(Event{event: String::from(""), listener_count: -1}),
                services,
            }
            ));


    let mut convar = Arc::new(Condvar::new());

    let mut convar_for_painter = Arc::clone(&convar);
    let mut state_for_painter = Arc::clone(&locked_state);


    let mut convar_for_fetcher = Arc::clone(&convar);
    let mut state_for_fetcher = Arc::clone(&locked_state);

    let mut convar_for_keyhandler = Arc::clone(&convar);
    let mut state_for_keyhandler = Arc::clone(&locked_state);
    //let key_handler_joiner = rt.spawn(key_handler(&mut locked_state, &mut convar));

    let key_handler_joiner = spawn(move || {
        rt.block_on(async move {
            key_handler(&mut state_for_keyhandler, &mut convar_for_keyhandler).await;
        } )});

    let fetcher_handler = spawn(move || {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
            fetcher(&haos_conn, &mut convar_for_fetcher, &mut state_for_fetcher).await;
        }) });


    ui::draw_ui(&mut state_for_painter, &mut convar_for_painter);

    key_handler_joiner.join().expect("We were unable to join the key_handler");
    fetcher_handler.join().expect("We were unable to join the fetcher");
    Ok(())
}
