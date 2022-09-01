use std::{sync::{Condvar, Arc, Mutex}, borrow::Cow};

use tui::{
    backend::CrosstermBackend,
    widgets::{Widget, Block, Borders, Tabs, List, ListItem, ListState, TableState, Table, Row, Cell},
    layout::{Layout, Constraint,Direction},
    Terminal,
    text::{Spans, Span, Text},
    style::{Style, Color},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use haoscli::types::Event as HAEvent;

use haoscli::types::{Service, State};

use log::{info, debug, trace};


#[derive(PartialEq, Debug)]
pub enum Pane {
    Events,
    Services,
    States,
    None,
}

#[derive(Debug)]
pub struct UiState {
    pub active: Pane,
    pub events: (Vec<HAEvent>, ListState),
    pub services: (Vec<Service>, TableState),
    pub states: (Vec<State>, ListState),
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
                Constraint::Percentage(25),
                Constraint::Percentage(37),
                Constraint::Percentage(37),
            ].as_ref());

    let mut paint_ui = || {
        let mut lock_state = state.lock().unwrap();
        //debug!("{:#?}", lock_state);

        let event_list_items: Vec<_> = lock_state.events.0
            .iter()
            .map(|event| {
                ListItem::new(Spans::from(vec![Span::styled(
                            event.event.clone(),
                            Style::default(),
                            )]))
            })
            .collect();

        let services_table_rows: Vec<_> = lock_state.services.0
            .iter()
            .map(|service| {
                //let cells: Vec<_> = vec![Cell::from(Text::from(Cow::Owned(service.services.to_string())))];
                //let cells: Vec<_> = vec![Cell::from(Cow::Owned(service.services.to_string()))];
                let mut cells: Vec<Cell> = vec!(Cell::from(Cow::Owned(service.domain.to_string())).style(Style::default()));
                cells.push(Cell::from(Cow::Owned(service.services.to_string())));
                Row::new(cells)
            })
        .collect();

        let state_list_items: Vec<_> = lock_state.states.0
            .iter()
            .map(|state| {
                ListItem::new(Spans::from(vec![Span::styled(
                            state.entity_id.clone(),
                            Style::default(),
                            )]))
            })
            .collect();

        //debug!("Service table rows: {:#?}", services_table_rows);

        terminal.draw(|f| {
            let size = f.size();
            let locs = chunks.split(size);
            /*
            let block = Block::default()
                .title("Bloock")
                .borders(Borders::ALL);

            */
            let event_list_element = List::new(event_list_items)
                .highlight_style(Style::default().bg(Color::Yellow))
                .block(Block::default().title("Services").borders(Borders::ALL));
            f.render_stateful_widget(event_list_element, locs[0], &mut lock_state.events.1);

            let services_table_element = Table::new(services_table_rows)
                .style(Style::default())
                .highlight_style(Style::default().bg(Color::Yellow))
                .header(Row::new(vec!["Service Name", "Service Details"]))
                .block(Block::default().title("Services").borders(Borders::ALL))
                .widths(&[
                    Constraint::Percentage(10),
                    Constraint::Percentage(90),
                ]);
            f.render_stateful_widget(services_table_element, locs[1], &mut lock_state.services.1);

            let states_list_element = List::new(state_list_items)
                .block(Block::default().borders(Borders::ALL).title("States"))
                .highlight_style(Style::default().bg(Color::Yellow))
                .style(Style::default());
            f.render_stateful_widget(states_list_element, locs[2], &mut lock_state.states.1);
            
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
