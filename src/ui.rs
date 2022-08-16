use std::sync::{Condvar, Arc, Mutex};

use tui::{
    backend::CrosstermBackend,
    widgets::{Widget, Block, Borders, Tabs,List},
    layout::{Layout, Constraint,Direction},
    Terminal,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use haoscli::types::Event as HAEvent;

use haoscli::types::Service;

pub struct UiState {
    pub events: Vec<HAEvent>,
    pub services: Vec<Service>,
}


pub fn draw_ui(state: &mut Arc<Mutex<UiState>>, convar: &mut Arc<Condvar>) {
    enable_raw_mode().expect("Could not enable raw mode");
    let mut std_out = std::io::stdout();
    execute!(std_out, EnterAlternateScreen, EnableMouseCapture).expect("Could not enter the altnerate screen or enable mouse capture");
    let backend = CrosstermBackend::new(std_out);
    let mut terminal = Terminal::new(backend).expect("Could not load the backend");


    let locked_state = state.lock().unwrap();

    terminal.draw(|f| {
        let size = f.size();
        /*
        let block = Block::default()
            .title("Bloock")
            .borders(Borders::ALL);
        */
        let list = List::new(locked_state.events);
        f.render_widget(list, size)
    }).expect("Failed to draw the terminal UI");

    std::thread::sleep_ms(5000);

    disable_raw_mode().expect("couldn't disable raw mode");
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    ).expect("Couldn't close everything out");
}
