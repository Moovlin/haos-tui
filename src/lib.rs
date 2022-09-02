use std::{sync::Arc, sync::RwLock, sync::Weak, thread::Result};

use log::{debug, info, warn, trace};

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
        let req = self.build_base_get_request("/events");
        let resp = req.send().await.unwrap();
        match resp.error_for_status_ref() {
            Ok(_) => (),
            Err(err) => {
                warn!("err.status: {}", err.status().unwrap_or_default());
                assert_eq!(err.status(), Some(reqwest::StatusCode::UNAUTHORIZED));
            },
        };

        let resp_json: Vec<types::Event> = resp.json().await.expect("Could not convert to json");

        Ok(resp_json)
    }

    pub async fn fire_event(&self, event_type: String, event_data: Option<impl serde::Serialize + std::fmt::Display>) -> Result<String>{
        let api = format!("{}/api/events/{}", self.url, event_type);
        let str_token = self.get_token();
        let req = reqwest::Client::new().post(api.as_str()).header("content-type", "application/json").bearer_auth(str_token);

        if let Some(data) = event_data {
            debug!("We're just dropping this data for now: {}", &data);
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


    pub async fn get_services(&self) -> Result<Vec<types::Service>> {
        let req = self.build_base_get_request("/services");
        let resp = match req.send().await {Ok(v) => v, Err(_) => panic!("Could not get services response")};

        let resp_json: Vec<types::Service> = match resp.json().await {Ok(v) => v, Err(e) => panic!("Couldn't parse the json response:\t{}", e)};
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

    pub async fn get_states(&self) -> Result<Vec<types::State>> {
        let req = self.build_base_get_request("/states");
        let resp = match req.send().await {Ok(v) => v, Err(e) => panic!("Couldn't not post to service: {}", e)};
        let resp_json: Vec<types::State> = resp.json().await.expect("Couldn't convert the list of entities from json to a vec of State structs");
        for resp in &resp_json {
            trace!("{:?}", resp);
        }
        Ok(resp_json)
    }

    pub async fn set_state(&self, state: types::State, payload: types::RequestStateStruct) -> Result<types::State> {
        let req = self.build_base_put_request(format!("/states/{}", state.entity_id.as_str()).as_str()).json(&payload.state);

        let resp = match req.send().await {Ok(v) => v, Err(e) => panic!("Couldn't set the state: {}", e)};
        info!("Set state for {} responded with HTTP code: {}", state.entity_id, resp.status());
        let resp_json: types::State = resp.json().await.expect("Couldn't convery the response of the state to a json");

        Ok(resp_json)
    }

    fn build_base_put_request(&self, end_point: &str) -> reqwest::RequestBuilder {
        let api = format!("{}/api{}", self.url, end_point);
        debug!("api: {}", api);
        let str_token = self.get_token();
        reqwest::Client::new().get(api.as_str()).header("content-type", "application/json").bearer_auth(str_token)
    }

    fn build_base_get_request(&self, end_point: &str) ->  reqwest::RequestBuilder{
        let api = format!("{}/api{}", self.url, end_point);
        debug!("api: {}", api);
        let str_token = self.get_token();
        reqwest::Client::new().get(api.as_str()).header("content-type", "application/json").bearer_auth(str_token)
    }

    fn get_token(&self) -> &str {
        let str_token = match &self.token {Token::LongLivedToken(str) => Ok(str), Token::None => Err("no") };
        match str_token { Ok(v) => v, Err(e) => e}
    }
}
