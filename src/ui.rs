use std::{sync::{Condvar, Arc, Mutex}, borrow::Cow};

use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem, ListState, TableState, Table, Row, Cell, self},
    layout::{Layout, Constraint,Direction, Rect},
    Terminal,
    text::{Spans, Span},
    style::{Style, Color},
};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use haoscli::types::Event as HAEvent;

use haoscli::types::{Service, State};

use log::{info, debug};

const POPUP_OFFSET: u16 = 5;

/// Enum to determine which pane is currently the active pane. 
#[derive(PartialEq, Debug, Default)]
pub enum Pane {
    #[default]
    Events,
    Services,
    States,
    PopUp(PopUpPane),
    None,
}

/// Enum to determine where we need to look for the selected item to create a pop up for it. 
#[derive(PartialEq, Debug, Default)]
pub enum PopUpPane {
    Events,
    Services,
    States,
    #[default]
    None,
}

/// Struct which holds the state of the UI. For each pane, there is the associated data and then,
/// assuming that the widget is stateful, the state for that widget. 
#[derive(Debug, Default)]
pub struct UiState {
    pub active: Pane,
    pub events: (Vec<HAEvent>, ListState),
    pub services: (Vec<Service>, TableState),
    pub states: (Vec<State>, ListState),
}



/// This function loops until quit is called. It draws each UI element. 
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
            let event_list_element = List::new(event_list_items)
                .highlight_style(Style::default().bg(Color::Yellow))
                .block(Block::default().title("Services").borders(Borders::ALL));
            f.render_stateful_widget(event_list_element, locs[0], &mut lock_state.events.1);

            let services_table_element = Table::new(services_table_rows)
                .style(Style::default())
                .highlight_style(Style::default().bg(Color::Yellow))
                .header(Row::new(vec!["Service Name", "Service Details"]))
                .block(Block::default())
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

            let popup_block: Rect;
            {
                let x = f.size().left() + POPUP_OFFSET;
                let y = f.size().top() + POPUP_OFFSET;

                let width = f.size().right() - POPUP_OFFSET;
                let height = f.size().bottom() - POPUP_OFFSET;

                popup_block = Rect{x, y, width, height};
            }
            // We want to draw the pop up after everything else so it looks pretty
            match lock_state.active {
                Pane::PopUp(PopUpPane::Events) => {
                    debug!("Rendering a pop up for events over the rest of the windows");
                    let event_loc = lock_state.events.1.selected().unwrap();
                    let passing_event = lock_state.events.0.get(event_loc).unwrap();
                    let (popup_list, mut popup_state) = build_event_element(passing_event);
                    f.render_widget(widgets::Clear, popup_block);
                    f.render_stateful_widget(popup_list, popup_block, &mut popup_state);
                },
                Pane::PopUp(PopUpPane::States) => {
                    debug!("Rendering a pop up for states over the rest of the windows");
                    let states_loc = lock_state.states.1.selected().unwrap();
                    let passing_states = lock_state.states.0.get(states_loc).unwrap();
                    let (popup_list, mut popup_state) = build_states_element(passing_states);
                    f.render_widget(widgets::Clear, popup_block);
                    f.render_stateful_widget(popup_list, popup_block, &mut popup_state);
                },
                Pane::PopUp(PopUpPane::Services) => debug!("Rendering a pop up for services over the rest of the windows"),
                _ => debug!("Not building a pop up as it's not marked as active. Current active pane: {:?}", lock_state.active),
            };
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

fn build_event_element<'a>(event: &'a HAEvent) -> (List, ListState){
    let event_list_items: Vec<_> = vec![ListItem::new(
        Spans::from(
            vec![
            Span::styled(Cow::Owned(event.listener_count.to_string()), Style::default())
            ,]
        ))];
    let ret_list = List::new(event_list_items)
        .block(Block::default().borders(Borders::ALL).title(event.event.clone()));
    let mut ret_list_state = ListState::default();
    ret_list_state.select(Some(0));
    (ret_list, ret_list_state)
}

fn build_states_element<'a>(state: &'a State) -> (Table, TableState) {
     let states_table_rows: Vec<Row> = vec!(Row::new(vec!(
         Cell::from(Cow::Owned(state.state.to_string())),
         Cell::from(Cow::Owned(state.last_changed.to_string())),
         Cell::from(Cow::Owned(state.attributes.to_string()))
         )));

     let state_table = Table::new(states_table_rows)
         .style(Style::default())
         .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black))
         .header(Row::new(vec!["State", "Changed Last At", "Attributes"]))
         .block(Block::default().borders(Borders::ALL).title(state.entity_id.clone()))
         .widths(&[
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(80),
         ])
         ;
    let mut ret_table_state = TableState::default();
    ret_table_state.select(Some(0));
    (state_table, ret_table_state)
}

fn build_states_popup() -> Layout {
    Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(90)
        ])
}

struct Positions {}

struct WindowElements {

}


struct StatesUIElement {
    
}
