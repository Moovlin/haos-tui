use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use crate::ui::{UiState, Pane};

use crossterm::event::KeyModifiers;
use crossterm::{
    event::{self, Event, KeyCode},
};

use log::{debug, info};

const REFRESH_RATE: u64 = 100;

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
        debug!("Dropping the state");
        drop($state);
        std::mem::drop($state);
        $callback()
    }};
    // This will call the function recived in second argument and pass the later arguments as that
    // function paramater
    ($state: expr, $callback: expr, $($args: expr)*) => {{
        debug!("Dropping the state");
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
        debug!("state.events.selected:\t{}", state.events.1.selected().expect("Couldn't get what row was selected"));
        let move_to_index = match state.events.1.selected() {
            None => 0,
            Some (current) => next_index(current, state.events.0.len(), direction),
        };
        state.events.1.select(Some(move_to_index));
        debug!("state.events.selected:\t{}", state.events.1.selected().unwrap());
        drop(state);
        notifier.notify_all();
        
    };

    let states_list_move = |direction: KeyDirection| {
        let mut state = state_og.lock().expect("Couldn't grab the state to move.");
        debug!("state.states.selected:\t{}", state.states.1.selected().expect("Couldn't get the row that was selected"));
        let move_to_index = match state.states.1.selected() {
            None => 0,
            Some (current) => next_index(current, state.states.0.len(), direction),
        };
        state.states.1.select(Some(move_to_index));
        debug!("state.states.selected:\t{}", state.states.1.selected().expect("Couldn't get the row that was selected"));
        drop(state);
        notifier.notify_all();
    };

    let services_table_move = |direction: KeyDirection| {
        let mut state = state_og.lock().expect("Couldn't grab the UI state");
        debug!("state.services.selected:\t{}", state.services.1.selected().expect("Couldn't get the row that is selected"));
        let move_to_index = match state.services.1.selected() {
            None => 0,
            Some(current) => next_index(current, state.services.0.len(), direction),
        };
        state.services.1.select(Some(move_to_index));
        debug!("state.services.selected:\t{}", state.services.1.selected().expect("Couldn't get the row that is selected"));
        drop(state);
        notifier.notify_all();

    };

    let handle_up_or_down = |direction: KeyDirection| {
        let state = state_og.lock().expect("Couldn't lock on the UI");
        match state.active {
            Pane::Events => {
                info!("Matched the eventPane");
                drop_and_call!(state, event_list_move, direction);
            },
            Pane::States => {
                info!("Matched the states pane");
                drop_and_call!(state, states_list_move, direction);
            },
            Pane::Services => {
                drop_and_call!(state, services_table_move, direction);
            }
            Pane::None => _ = quit(),
            _ => ()
        }
    };

    let handle_pane_switch = |switch_to_pane: Pane| {
        let mut state = state_og.lock().expect("Couldn't lock on the UI");
        state.active = switch_to_pane;
    };

    'listener_loop: loop {
        if event::poll(Duration::from_millis(REFRESH_RATE)).unwrap() {
            match event::read().unwrap() {
                Event::Key(key) => {
                    let holding_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
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
                                info!("Got quit keypress. Quitting");
                                if quit() {
                                    break 'listener_loop;
                                }
                            } else if ch == 'e' && holding_ctrl {
                                handle_pane_switch(Pane::Events);
                            } else if ch == 's' && holding_ctrl {
                                handle_pane_switch(Pane::Services);
                            } else if ch == 'x' && holding_ctrl {
                                handle_pane_switch(Pane::States);
                            }
                        },
                        _ => {
                            notifier.notify_all();
                        }
                    }
                },
                Event::FocusLost => {
                    debug!("Focus lost");
                },
                Event::Resize(..) => {
                    debug!("Window was resized");
                    notifier.notify_all();
                },
                _ => {}
            }
        }
    }
}
