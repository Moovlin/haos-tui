use std::sync::{Condvar, Arc, Mutex};

use tui::{
    backend::CrosstermBackend,
    widgets::{Widget, Block, Borders, Tabs, List, ListItem, ListState},
    layout::{Layout, Constraint,Direction},
    Terminal,
    text::{Spans, Span},
    style::Style,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use haoscli::types::Event as HAEvent;

use haoscli::types::Service;

use log::info;

pub struct UiState {
    pub active: bool,
    pub events: Vec<HAEvent>,
    pub services: Vec<Service>,
}


pub fn draw_ui(state: &mut Arc<Mutex<UiState>>, convar: &mut Arc<Condvar>) {
    info!("Entered draw_ui for the first time");
    enable_raw_mode().expect("Could not enable raw mode");
    let mut std_out = std::io::stdout();
    execute!(std_out, EnterAlternateScreen, EnableMouseCapture).expect("Could not enter the altnerate screen or enable mouse capture");
    let backend = CrosstermBackend::new(std_out);
    let mut terminal = Terminal::new(backend).expect("Could not load the backend");


    let mut paint_ui = || {
        let lock_state = state.lock().unwrap();

        let mut event_list_state = ListState::default();
        event_list_state.select(Some(0));

        let event_list_items: Vec<_> = lock_state.events
            .iter()
            .map(|event| {
                ListItem::new(Spans::from(vec![Span::styled(
                            event.event.clone(),
                            Style::default(),
                            )]))
            })
            .collect();

        let selected_event = lock_state.events
            .get(
                event_list_state
                .selected()
                .expect("An event should be selected"),
            ).expect("exists")
            .clone();

        terminal.draw(|f| {
            let size = f.size();
            /*
            let block = Block::default()
                .title("Bloock")
                .borders(Borders::ALL);
            */
            let list = List::new(event_list_items);
            f.render_widget(list, size)
        }).expect("Failed to draw the terminal UI");
    };

    'ui_loop: loop {
        if !convar.wait(state.lock().unwrap()).unwrap().active {
            info!("Quitting since we were told to");
            break 'ui_loop;
        } else {
            paint_ui();
        }
    }

    disable_raw_mode().expect("couldn't disable raw mode");
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    ).expect("Couldn't close everything out");
}
