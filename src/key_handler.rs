use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use crate::ui::UiState;

use crossterm::{
    event::{self, Event, KeyCode},
};

use log::{debug, info};

const REFRESH_RATE: u64 = 250;

pub async fn key_handler (state_og: &mut Arc<Mutex<UiState>>, notifier: &mut Arc<Condvar>) {
    let quit = || -> bool {
        let mut state =  state_og.lock().expect("Could not quit, we couldn't lock the UI");
        state.active = false;
        notifier.notify_all();
        true
    };

    'listener_loop: loop {
        if event::poll(Duration::from_millis(REFRESH_RATE)).unwrap() {
            match event::read().unwrap() {
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Up => {
                            debug!("We saw an up");
                        },
                        KeyCode::Char(ch) => {
                            if ch == 'q' {
                                info!("Quitting");
                                if(quit()) {
                                    break 'listener_loop;
                                }
                            }
                        }
                        _ => {
                            notifier.notify_all();
                        }
                    }
                },
                Event::FocusLost => {
                    debug!("Focus lost");
                }
                _ => {}
            }
        }
    }
}
