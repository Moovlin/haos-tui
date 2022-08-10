use std::sync::{RwLock, Weak};


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


