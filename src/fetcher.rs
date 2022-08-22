use haoscli::types::{Event, HomeAssistantConnection};

use std::{sync::{Arc, RwLock, Condvar, Mutex}, thread, time::Duration, io::Error};

use log::info;

use crate::ui::{UiState, Pane};

const REQUEST_WAIT: u64 = 10000;

pub async fn fetcher(haos_conn_locked: &Arc<RwLock<HomeAssistantConnection>>, convar: &Arc<Condvar>, state: &mut Arc<Mutex<UiState>>) {
    
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
            info!("recived response from HAOS");
            events = Ok(temp_events.await.unwrap());
        }
        let mut state_lock = state.lock().expect("Could not get the lock on the state");
        state_lock.events.0 = events.expect("test");

        for evnt in state_lock.events.0.iter() {
            info!("Event: {}, listeners: {}", evnt.event, evnt.listener_count);
        }
        
       drop(state_lock);

        convar.notify_all();

        thread::sleep(Duration::from_millis(REQUEST_WAIT));
    }

}
