use haoscli::types::{Event, HomeAssistantConnection, Service, State, RequestEntityObject, RequestStateStruct};

use std::{
    io::Error,
    sync::{Arc, Condvar, Mutex, RwLock},
    thread,
    time::Duration,
};

use log::{info, trace, warn};

use crate::ui::{Pane, UiState};

pub async fn fetcher(
    haos_conn_locked: &Arc<RwLock<HomeAssistantConnection>>,
    convar: &Arc<Condvar>,
    state: &mut Arc<Mutex<UiState>>,
    poll_rate: u64,
) {
    //let events = match rt.block_on(working_haos_conn.get_events()) {Ok(v) => v, Err(_) => panic!("Couldn't access the resouce")};
    loop {
        {
            let ui_state = match state.try_lock() {
                Ok(v) => v,
                Err(e) => {
                    warn!("Error getting UI lock in the fetcher: {}", e);
                    continue;
                }
            };
            if ui_state.active == Pane::None {
                break;
            }
            drop(ui_state);
        }

        let mut state_lock = state.lock().expect("Could not get the lock on the state");

        // Going to check that we have marked "input as ready" as ready. 
        //let state_update: Result<State, Error>;
        if state_lock.input_pane.1 {
            let haos_conn = haos_conn_locked.read().expect("Couldn't get the read lock");

            // Getting the seleted state
            let selected_state = state_lock.states.0.get(state_lock.states.1.selected().expect("Couldn't get the selected value")).expect("Couldn't select the selected state from the list of states");
            
            let set_state = RequestStateStruct{state: serde_json::from_str(state_lock.input_pane.0.as_str()).expect("Couldn't parse the text as a json") };

            _ = haos_conn.set_state(selected_state, set_state).await.expect("Couldn't send the data to the end point");

            state_lock.input_pane.0 = String::from("");
            state_lock.input_pane.1 = false;
        }

        drop(state_lock);

        let mut state_lock = state.lock().expect("Could not get the lock on the state");
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
        state_lock.events.0 = events.expect("test");


        let services: Result<Vec<Service>, Error>;
        {
            let haos_conn = haos_conn_locked
                .read()
                .expect("Couldn't get the read lock to unlock the service");
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
            let haos_conn = haos_conn_locked
                .read()
                .expect("Couldn't get the read lock to unlock the state");
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
