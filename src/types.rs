use std::{sync::{RwLock, Weak}, collections::HashMap};

use chrono::{DateTime, Utc};


use serde::{Serialize, Deserialize};

/// Struct related to the HomeAssistant instance
/// Currently only handles long term token and uses the REST end points. 
#[derive(Debug)]
pub struct HomeAssistantConnection {
    /// The URL which you are connecting to
    pub url: String, 
    /// The token which you are using to connect to aforementioned the home assistant instance
    pub token: Token,
    /// The client id we are using to connect to this Homeassistant instance
    pub client_id: String, 
    /// When a request fails, how many times should we retry
    pub retries: i32,
    /// A weak reference to ourself so that we can lock
    pub lock: Weak<RwLock<Self>>,
}

/// An enum for the token. This is created for holding purposes. 
#[derive(Debug)]
#[non_exhaustive]
pub enum Token {
    LongLivedToken(String),
    None,
}

/// Struct to hold data about an event listing
#[derive(Debug, Serialize, Deserialize)]
pub struct Event  {
   pub event: String, 
   pub listener_count: i32,

}

/// Struct to hold the service information. 
#[derive(Debug, Serialize, Deserialize)]
pub struct Service {
    /// The domain of the service, IE: Lights, alarms, etc. 
    pub domain: String,
    //pub services: HashMap<String, serde_json::Map<String, serde_json::Value>>,
    
    /// Holds the json value. 
    pub services: serde_json::Value,
}

/// Used to create a request information about an entity, passes in the entity id. 
#[derive(Debug, Serialize, Deserialize)]
pub struct RequestEntityObject<'a> {
    pub entity_id: &'a str,
}

/// Holds the state informaiton about the Entities in the HAOS instance. 
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct State {
    pub entity_id: String, 
    pub state: String,
    pub last_changed: DateTime<Utc>,
    pub attributes: serde_json::Value,
}

/// Used to get the request state. Hashmap for the values & whatnot. 
pub struct RequestStateStruct {
    pub state: HashMap<String, String>,
}
