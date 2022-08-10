use std::{fmt::Display, sync::Arc, sync::RwLock, sync::Weak, thread::Result};

use std::error::Error;
use serde::{Serialize, Deserialize};

use types::{HomeAssistantConnection, Token};
pub mod types;


impl HomeAssistantConnection {
    pub fn new(url: String, client_id: String) -> Arc<RwLock<Self>> {
        let token = Token::None;
        let ret = Arc::new(RwLock::new(Self {
            url,
            token,
            client_id,
            lock: Weak::new(),
        }));
        
        ret.write().unwrap().lock = Arc::downgrade(&ret);

        ret
    }

    pub fn set_long_live_token(&mut self, token: String){
        self.token = Token::LongLivedToken(token);
    }

    pub async fn get_events(&self) -> Result<Vec<types::Event>>{
        let api = format!("{}/api/events", self.url);
        let str_token = self.get_token();
        let req = reqwest::Client::new().get(api.as_str()).header("content-type", "application/json").bearer_auth(str_token);

        let resp = req.send().await.unwrap();
        let resp_json: Vec<types::Event> = resp.json().await.unwrap();

        Ok(resp_json)
    }

    pub async fn fire_event(self, event_type: String, event_data: Option<impl serde::Serialize>) -> Result<String>{
        let api = format!("{}/api/events/{}", self.url, event_type);
        let str_token = self.get_token();
        let mut req = reqwest::Client::new().post(api.as_str()).header("content-type", "application/json").bearer_auth(str_token);

        if let Some(data) = event_data {
            req = req.json(&data);
        }

        let resp = req.send().await.unwrap();

        #[derive(Serialize, Deserialize, Debug)]
        struct Response {
            message: String,
        }
            
        let resp_json: Response = resp.json().await.unwrap();
        Ok(resp_json.message)

    }

    fn get_token(&self) -> &str {
        let str_token = match &self.token {Token::LongLivedToken(str) => Ok(str), Token::None => Err("no") }.unwrap();
        str_token
    }

}
