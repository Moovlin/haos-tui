use std::{sync::{RwLock, Weak}, collections::HashMap};


use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Debug)]
pub struct HomeAssistantConnection {
   pub url: String, 
   pub token: Token,
   pub client_id: String, 
   pub retries: i32,
   pub lock: Weak<RwLock<Self>>,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Token {
    LongLivedToken(String),
    None,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
   pub event: String, 
   pub listener_count: i32,

}


#[derive(Debug, Serialize, Deserialize)]
pub struct Service {
    pub domain: String,
    //pub services: HashMap<String, serde_json::Map<String, serde_json::Value>>,
    pub services: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestEntityObject<'a> {
    pub entity_id: &'a str,
}


