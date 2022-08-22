use std::sync::{Condvar, Arc, Mutex};

use tui::{
    backend::CrosstermBackend,
    widgets::{Widget, Block, Borders, Tabs, List, ListItem, ListState, TableState},
    layout::{Layout, Constraint,Direction},
    Terminal,
    text::{Spans, Span},
    style::{Style, Color},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use haoscli::types::Event as HAEvent;

use haoscli::types::Service;

use log::{info, debug};


#[derive(PartialEq)]
pub enum Pane {
    EventPane,
    ServicesPane,
    None,
}

pub struct UiState {
    pub active: Pane,
    pub events: (Vec<HAEvent>, ListState),
    pub services: (Vec<Service>, TableState),
}


pub fn draw_ui(state: &mut Arc<Mutex<UiState>>, convar: &mut Arc<Condvar>) {
    info!("Entered draw_ui for the first time");
    enable_raw_mode().expect("Could not enable raw mode");
    let mut std_out = std::io::stdout();
    execute!(std_out, EnterAlternateScreen, EnableMouseCapture).expect("Could not enter the altnerate screen or enable mouse capture");
    let backend = CrosstermBackend::new(std_out);
    let mut terminal = Terminal::new(backend).expect("Could not load the backend");

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ].as_ref());

    let mut paint_ui = || {
        let mut lock_state = state.lock().unwrap();

        let event_list_items: Vec<_> = lock_state.events.0
            .iter()
            .map(|event| {
                ListItem::new(Spans::from(vec![Span::styled(
                            event.event.clone(),
                            Style::default(),
                            )]))
            })
            .collect();

        terminal.draw(|f| {
            let size = f.size();
            let locs = chunks.split(size);
            /*
            let block = Block::default()
                .title("Bloock")
                .borders(Borders::ALL);
            */
            let list = List::new(event_list_items).highlight_style(Style::default().bg(Color::Yellow));
            f.render_stateful_widget(list, locs[0], &mut lock_state.events.1);
        }).expect("Failed to draw the terminal UI");
    };

    'ui_loop: loop {
        if convar.wait(state.lock().unwrap()).unwrap().active == Pane::None {
            info!("Quitting since we were told to");
            break 'ui_loop;
        } else {
            debug!("Repainting the UI");
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
