use haoscli::types::{Event, HomeAssistantConnection, Service, State};

use std::{sync::{Arc, RwLock, Condvar, Mutex}, thread, time::Duration, io::Error};

use log::{trace, info};

use crate::ui::{UiState, Pane};

const REQUEST_WAIT: u64 = 10000;

pub async fn fetcher(haos_conn_locked: &Arc<RwLock<HomeAssistantConnection>>, convar: &Arc<Condvar>, state: &mut Arc<Mutex<UiState>>, poll_rate: u64) {
    
    //let events = match rt.block_on(working_haos_conn.get_events()) {Ok(v) => v, Err(_) => panic!("Couldn't access the resouce")};
    loop {
        {
            let ui_state = match state.try_lock() {
                Ok(v) => v,
                Err(e) => {info!("Error getting UI lock in the fetcher: {}", e);continue},
            };
            if ui_state.active == Pane::None {
                break;
            }
            drop(ui_state);
        }
        
        let events: Result<Vec<Event>, Error>;
        {
            let haos_conn = haos_conn_locked.read().expect("Couldn't get the read lock");
            let temp_events = haos_conn.get_events();
            info!("recived response for event update from HAOS");
            events = Ok(temp_events.await.unwrap());
        }
        for event in &events {
            trace!("Event Recieved: {:?}", event);
        }
        let mut state_lock = state.lock().expect("Could not get the lock on the state");
        state_lock.events.0 = events.expect("test");


        let services: Result<Vec<Service>, Error>;
        {
            let haos_conn = haos_conn_locked.read().expect("Couldn't get the read lock to unlock the service");
            let temp_services = haos_conn.get_services();
            info!("Recieved a response for Services from HAOS");
            services = Ok(temp_services.await.expect("Couldn't get the services"));
        }

        for service in &services {
            trace!("Service recieved: {:?}", service);
        }

        state_lock.services.0 = services.expect("Couldn't unwrap the services value");
        

        let states: Result<Vec<State>, Error>;
        {
            let haos_conn = haos_conn_locked.read().expect("Couldn't get the read lock to unlock the state");
            let temp_states = haos_conn.get_states();
            info!("Recieved a respsone for states from Haos");
            states = Ok(temp_states.await.expect("Couldn't get the states"));
        }
        
        state_lock.states.0 = states.expect("Couldn't unwrap the states value");

        drop(state_lock);

        convar.notify_all();

        thread::sleep(Duration::from_millis(poll_rate));
    }

}
