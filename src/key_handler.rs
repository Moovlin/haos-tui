use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use crate::ui::{UiState, Pane};

use crossterm::{
    event::{self, Event, KeyCode},
};

use log::{debug, info, trace};

const REFRESH_RATE: u64 = 250;

// Helper function to drop the first paramater and call the function in second paramater and
// optional arguments provided in later arguments
// This is used to drop the state and call the function as such pattern is found redundant while
// calling event handeling closure where unlocked state needs to be droppped before calling the
// corresponding handler
// Macro is ripped right from the https://github.com/sudipghimire533/ytui-music repo. S/O to them. 
macro_rules! drop_and_call {
    // This will call the function in passe in second argument
    // passed function will not accept any argument
    ($state: expr, $callback: expr) => {{
        trace!("Dropping the state");
        drop($state);
        std::mem::drop($state);
        $callback()
    }};
    // This will call the function recived in second argument and pass the later arguments as that
    // function paramater
    ($state: expr, $callback: expr, $($args: expr)*) => {{
        trace!("Dropping the state");
        std::mem::drop($state);
        $callback( $($args)* )
    }};
}


enum KeyDirection {
    Up,
    Down,
    Initial,
}

fn next_index(current: usize, list_size: usize, direction: KeyDirection) -> usize{
    if list_size == 0 {
        return 0;
    } 
    match direction {
        KeyDirection::Up => (current.checked_sub(1).unwrap_or(list_size - 1)) % list_size,
        KeyDirection::Down => (current + 1) % list_size,
        KeyDirection::Initial => current,
    }
     
}

pub async fn key_handler (state_og: &mut Arc<Mutex<UiState>>, notifier: &mut Arc<Condvar>) {
    let quit = || -> bool {
        let mut state =  state_og.lock().expect("Could not quit, we couldn't lock the UI");
        state.active = Pane::None;
        notifier.notify_all();
        true
    };

    let event_list_move = |direction: KeyDirection| {
        let mut state = match state_og.lock(){
            Ok(v) => {info!("Grabbed the state"); v},
            Err(e) => {info!("Couldn't grab the lock to move the list???: {}", e);return},
        };
        trace!("state.events.selected:\t{}", state.events.1.selected().expect("Couldn't get what row was selected"));
        let move_to_index = match state.events.1.selected() {
            None => 0,
            Some (current) => next_index(current, state.events.0.len(), direction),
        };
        state.events.1.select(Some(move_to_index));
        trace!("state.events.selected:\t{}", state.events.1.selected().unwrap());
        drop(state);
        notifier.notify_all();
        
    };

    let handle_up_or_down = |direction: KeyDirection| {
        let state = state_og.lock().expect("Couldn't lock on the UI");
        match state.active {
            Pane::EventPane => {
                info!("Matched the eventPane");
                drop_and_call!(state, event_list_move, direction);
            },
            //Pane::EventPane => {info!("Matched the eventPane"); drop_and_call!(state, event_list_move, direction);},
            Pane::None => _ = quit(),
            _ => ()
        }
    };

    'listener_loop: loop {
        if event::poll(Duration::from_millis(REFRESH_RATE)).unwrap() {
            match event::read().unwrap() {
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Up => {
                            debug!("Pressed Up");
                            handle_up_or_down(KeyDirection::Up);
                        },
                        KeyCode::Down => {
                            debug!("Pressed down");
                            handle_up_or_down(KeyDirection::Down);
                        },
                        KeyCode::Char(ch) => {
                            if ch == 'q' {
                                info!("Quitting");
                                if quit() {
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
