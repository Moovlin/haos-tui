use std::{fmt::Display, sync::Arc, sync::RwLock, sync::Weak, thread::Result};

use log::{debug, info, error};
use reqwest::Response;

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
            retries: 30,
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

    pub async fn fire_event(&self, event_type: String, event_data: Option<impl serde::Serialize + std::fmt::Display>) -> Result<String>{
        let api = format!("{}/api/events/{}", self.url, event_type);
        let str_token = self.get_token();
        let mut req = reqwest::Client::new().post(api.as_str()).header("content-type", "application/json").bearer_auth(str_token);

        if let Some(data) = event_data {
            debug!("{}", &data);
            //req = req.json(&data);
        }

        let resp = match req.send().await {
            Ok(v) => v,
            Err(_) => panic!("Couldn't send the message"),
        };


        #[derive(Serialize, Deserialize, Debug)]
        struct Response {
            message: String,
        }
            
        let resp_json: Response = match resp.json().await {Ok(v) => v, Err(_) => panic!("Couldn't parse the json response")};
        Ok(resp_json.message)

    }

    fn get_token(&self) -> &str {
        let str_token = match &self.token {Token::LongLivedToken(str) => Ok(str), Token::None => Err("no") };
        match str_token { Ok(v) => v, Err(e) => e}
    }

    pub async fn get_services(&self) -> Result<Vec<types::Service>> {
        let api = format!("{}/api/services", self.url);
        let str_token = self.get_token();
        let req = reqwest::Client::new().get(api.as_str()).header("content-type", "application/json").bearer_auth(str_token);
        
        let resp = match req.send().await {Ok(v) => v, Err(_) => panic!("Could not get services response")};

        let resp_json: Vec<types::Service> = match resp.json().await {Ok(v) => v, Err(e) => panic!("Couldn't parse the json response:{}", e)};
        //let resp_string = match resp.text().await {Ok(v) => v, Err(_) => panic!("Couldn't get the text of the response")};
        //info!("{}", &resp_string);
        //let resp_json: Vec<types::Service> = match serde_json::from_str(resp_string.as_str()) {Ok(v) => v, Err(_) => panic!("Couldn't parse the json")};
        Ok(resp_json)
    }

    pub async fn set_service(&self, service: types::Service, entity: Option<&'_ types::RequestEntityObject<'_>>) -> Result<serde_json::Value> {
        let api = format!("{}/api/services/{}/{}", self.url, service.domain, service.services.as_str().unwrap());
        
        let str_token = self.get_token();
        let mut req = reqwest::Client::new().post(api.as_str()).header("content-type", "application/json").bearer_auth(str_token);
        match entity {
            Some(v) => req = req.json(&v),
            None => (),
        }
        let resp = match req.send().await {Ok(v) => v, Err(e) => panic!("Could not post to service: {}", e)};
        info!("{:?}", resp);

        let resp_json: serde_json::Value = match resp.json().await {Ok(v) => v, Err(_) => panic!("Couldn't parse the json response")};

        Ok(resp_json)
    }

}
