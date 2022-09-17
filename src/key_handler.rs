use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use crate::ui_types::{Pane, PopUpPane, UiState, ServicesPopUpElement, BuildTable};

use crossterm::event::KeyModifiers;
use crossterm::event::{self, Event, KeyCode};

use haoscli::types::Service;
use log::{debug, info, warn};
use tui::layout::Rect;
use tui::widgets::TableState;

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

/// which direction the keypress is.
enum KeyDirection {
    Up,
    Down,
    Initial,
}

/// Helper function to determine the next index for indexable widgets.
fn next_index(current: usize, list_size: usize, direction: KeyDirection) -> usize {
    if list_size == 0 {
        return 0;
    }
    match direction {
        KeyDirection::Up => (current.checked_sub(1).unwrap_or(list_size - 1)) % list_size,
        KeyDirection::Down => (current + 1) % list_size,
        KeyDirection::Initial => current,
    }
}

/// Async function which handles the key press management and then updates the UI state for
/// drawing.
pub async fn key_handler(state_og: &mut Arc<Mutex<UiState>>, notifier: &mut Arc<Condvar>) {
    let quit = || -> bool {
        let mut state = state_og
            .lock()
            .expect("Could not quit, we couldn't lock the UI");
        state.active = Pane::None;
        notifier.notify_all();
        true
    };

    let event_list_move = |direction: KeyDirection| {
        let mut state = match state_og.lock() {
            Ok(v) => {
                info!("Grabbed the state");
                v
            }
            Err(e) => {
                info!("Couldn't grab the lock to move the list???: {}", e);
                return;
            }
        };
        debug!(
            "state.events.selected:\t{}",
            state
                .events
                .1
                .selected()
                .expect("Couldn't get what row was selected")
        );
        let move_to_index = match state.events.1.selected() {
            None => 0,
            Some(current) => next_index(current, state.events.0.len(), direction),
        };
        state.events.1.select(Some(move_to_index));
        debug!(
            "state.events.selected:\t{}",
            state.events.1.selected().unwrap()
        );
        drop(state);
        notifier.notify_all();
    };

    let states_list_move = |direction: KeyDirection| {
        let mut state = state_og.lock().expect("Couldn't grab the state to move.");
        debug!(
            "state.states.selected:\t{}",
            state
                .states
                .1
                .selected()
                .expect("Couldn't get the row that was selected")
        );
        let move_to_index = match state.states.1.selected() {
            None => 0,
            Some(current) => next_index(current, state.states.0.len(), direction),
        };
        state.states.1.select(Some(move_to_index));
        debug!(
            "state.states.selected:\t{}",
            state
                .states
                .1
                .selected()
                .expect("Couldn't get the row that was selected")
        );
        drop(state);
        notifier.notify_all();
    };

    let services_table_move = |direction: KeyDirection| {
        let mut state = state_og.lock().expect("Couldn't grab the UI state");
        debug!(
            "state.services.selected:\t{}",
            state
                .services
                .1
                .selected()
                .expect("Couldn't get the row that is selected")
        );
        let move_to_index = match state.services.1.selected() {
            None => 0,
            Some(current) => next_index(current, state.services.0.len(), direction),
        };
        state.services.1.select(Some(move_to_index));
        debug!(
            "state.services.selected:\t{}",
            state
                .services
                .1
                .selected()
                .expect("Couldn't get the row that is selected")
        );
        drop(state);
        notifier.notify_all();
    };

    let services_popup_table_move = |direction: KeyDirection| {
        let mut state = state_og.lock().expect("Couldn't grab the UI State");


        debug!("state.services_popup.0.services:\t{:?}", state.services_popup.0.services);
        debug!("state.services_popup.1.selected():\t{:?}", state.services_popup.1.selected());

        let state_map = state.services_popup.0.services.as_object().unwrap();
        
        let move_to_index = match state.services_popup.1.selected() {
            None => 0,
            Some(current) => next_index(current, state_map.len(), direction),
        };

        state.services_popup_selected = state_map.keys().nth(move_to_index).unwrap().into();
        debug!("state.services_popup_selected:\t{}", state.services_popup_selected);
        state.services_popup.1.select(Some(move_to_index));
        debug!("state.services_popup.1.selected():\t{:?}", state.services_popup.1.selected());
        drop(state);
        notifier.notify_all();
    };

    let handle_up_or_down = |direction: KeyDirection| {
        let state = state_og.lock().expect("Couldn't lock on the UI");
        match state.active {
            Pane::Events => {
                info!("Matched the eventPane");
                drop_and_call!(state, event_list_move, direction);
            }
            Pane::States => {
                info!("Matched the states pane");
                drop_and_call!(state, states_list_move, direction);
            }
            Pane::Services => {
                drop_and_call!(state, services_table_move, direction);
            }
            Pane::PopUp(PopUpPane::Services) => {
                drop_and_call!(state, services_popup_table_move, direction);
            }
            Pane::None => _ = quit(),
            _ => (),
        }
    };

    let handle_pane_switch = |switch_to_pane: Pane| {
        let mut state = state_og.lock().expect("Couldn't lock on the UI");
        state.active = switch_to_pane;
        notifier.notify_all();
    };

    // Need a way to handle hitting enter to bring up the correct pop up for a given service.
    let handle_enter = || {
        // Match on what the active pane is and then mark the active as the pane.
        //let mut state = state_og.lock().map_err(|_| {warn!("Couldn't lock the state")});
        let mut state = state_og.lock().expect("Couldn't lock on the UI");
        match state.active {
            Pane::Events => state.active = Pane::PopUp(PopUpPane::Events),
            Pane::Services => {
                state.active = Pane::PopUp(PopUpPane::Services);
                let sel_service: &Service = state.get_selected_service();
                let mut popup_state = TableState::default();
                popup_state.select(Some(0));
                state.services_popup = (sel_service.clone(), popup_state);
            },
            Pane::States => state.active = Pane::PopUp(PopUpPane::States),
            Pane::PopUp(_) => state.input_pane.1 = true,
            Pane::Search => todo!("Not implemented"),
            Pane::None => debug!("Trying to hit enter when we have no active pane, ignoring as we should be closing."),
        };
        debug!("Active pane should be a popup: {:?}", state.active);
        notifier.notify_all();
    };

    /*
    let handle_popup_enter = || {
        let mut state = state_og.lock().expect("Couldn't Lock the UI");
        match state.active {
            Pane::PopUp(PopUpPane::Services) => {
                
            }, 
        }
    };
    */

    // Need a way to exit a popup, we'll set the pane back to the non-pop up version of whatever we
    // opened last.
    let handle_escape = || {
        let mut state = state_og.lock().expect("Couldn't lock on the UI");
        match state.active {
            Pane::PopUp(PopUpPane::Events) => state.active = Pane::Events,
            Pane::PopUp(PopUpPane::Services) => state.active = Pane::Services,
            Pane::PopUp(PopUpPane::States) => state.active = Pane::States,
            Pane::PopUp(PopUpPane::None) => debug!("tf???"),
            _ => debug!("Ignoring escape press for non-pop up panes"),
        }
        state.input_pane = (String::from(""), false); // THIS IS BAD BUT HEY I'M WORKING TOWARD AN
                                                      // MVP. WE WILL HAVE TO ACCEPT THIS AS A
                                                      // REALITY. 
        notifier.notify_all();
    };

    let activate_search = || {
        let mut state = state_og.lock().expect("Couldn't lock on the state");
        match state.active {
            Pane::PopUp(_) => state.active = Pane::Search,
            _ => (),
        }
        notifier.notify_all();
    };

    let handle_popup_input = |ch| {
        debug!("Handling popup input");
        let mut state = state_og.lock().expect("Couldn't lock the state");
        state.input_pane.0.push(ch);
        notifier.notify_all();
    };

    let handle_backspace = || {
        debug!("Handling the backspace");
        let mut state = state_og.lock().expect("Couldn't lock the state");
        state.input_pane.0.pop();
        notifier.notify_all();
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
                        }
                        KeyCode::Down => {
                            debug!("Pressed down");
                            handle_up_or_down(KeyDirection::Down);
                        }
                        KeyCode::Enter => {
                            debug!("Pressed Enter");
                            handle_enter();
                        }
                        KeyCode::Esc => {
                            debug!("Pressed Escape");
                            handle_escape();
                        }
                        KeyCode::Backspace => {
                            debug!("Pressed backspace");
                            handle_backspace();
                        }
                        KeyCode::Char(ch) => {
                            let active_pane = state_og.lock().expect("Could be anything").active.clone();
                            let in_pop_up = match active_pane {
                                Pane::PopUp(PopUpPane::Events) => true,
                                Pane::PopUp(PopUpPane::States) => true,
                                Pane::PopUp(PopUpPane::Services) => true,
                                Pane::PopUp(PopUpPane::None) => false,
                                _ => false, 
                            };
                            if in_pop_up {
                                debug!("The active pane is in the pop up");
                                handle_popup_input(ch);
                            } else if ch == 'q' {
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
                            } else if ch == '/' {
                                activate_search();
                            }
                        }
                        _ => {
                            notifier.notify_all();
                        }
                    }
                }
                Event::FocusLost => {
                    debug!("Focus lost");
                }
                Event::Resize(..) => {
                    debug!("Window was resized");
                    notifier.notify_all();
                }
                _ => {}
            }
        }
    }
}
