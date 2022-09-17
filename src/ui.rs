use std::{
    borrow::Cow,
    io,
    sync::{Arc, Condvar, Mutex},
};

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{self, Block, Borders, Cell, List, ListItem, ListState, Row, Table, TableState, Paragraph, StatefulWidget, Widget},
    Terminal,
};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use haoscli::types::Event as HAEvent;

use haoscli::types::{Service, State};

use crate::ui_types::{ServicesPopUpElement, EventsPopUpElement, StatesPopUpElement, BuildPopup, BuildList, BuildTable, Pane, PopUpPane, UiState};


use log::{debug, info};

const POPUP_OFFSET: u16 = 5;

fn reset_terminal() -> Result<(), ()> {
    disable_raw_mode().expect("couldn't disable raw mode");
    crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture,)
        .expect("Couldn't execute all the commands.");
    Ok(())
}


/// This function loops until quit is called. It draws each UI element.
pub fn draw_ui(state: &mut Arc<Mutex<UiState>>, convar: &mut Arc<Condvar>) {
    info!("Entered draw_ui for the first time");
    enable_raw_mode().expect("Could not enable raw mode");
    let mut std_out = std::io::stdout();
    execute!(std_out, EnterAlternateScreen, EnableMouseCapture)
        .expect("Could not enter the altnerate screen or enable mouse capture");
    let backend = CrosstermBackend::new(std_out);
    let mut terminal = Terminal::new(backend).expect("Could not load the backend");

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        reset_terminal().unwrap();
        original_hook(panic);
    }));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(25),
                Constraint::Percentage(37),
                Constraint::Percentage(37),
            ]
            .as_ref(),
        );

    let mut paint_ui = || {
        let mut lock_state = state.lock().unwrap();
        //debug!("{:#?}", lock_state);

        let event_list_items: Vec<_> = lock_state
            .events
            .0
            .iter()
            .map(|event| {
                ListItem::new(Spans::from(vec![Span::styled(
                    event.event.clone(),
                    Style::default(),
                )]))
            })
            .collect();

        let services_table_rows: Vec<_> = lock_state
            .services
            .0
            .iter()
            .map(|service| {
                let mut cells: Vec<Cell> = vec![
                    Cell::from(Cow::Owned(service.domain.to_string())).style(Style::default())
                ];
                cells.push(Cell::from(Cow::Owned(service.services.to_string())));
                Row::new(cells)
            })
            .collect();

        let state_list_items: Vec<_> = lock_state
            .states
            .0
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
                    /*
                    // Building the block & then table
                    let states_loc = lock_state.states.1.selected().unwrap();
                    let passing_states = lock_state.states.0.get(states_loc).unwrap();
                    let popup = StatesPopUpElement::new(popup_block, passing_states);
                    f.render_widget(widgets::Clear, popup_block);
                    f.render_stateful_widget(popup_list, popup.popup_loc, &mut popup_state);
                    */
                    let states_loc = lock_state.states.1.selected().unwrap();
                    let passing_states = lock_state.states.0.get(states_loc).unwrap();
                    let popup = StatesPopUpElement::new(popup_block, passing_states);
                    let (popup_table, mut popup_state) = popup.build_table_element();
                    let screen_locs = popup.build_popup();

                    f.render_widget(widgets::Clear, popup_block);
                    
                    // Building the table
                    f.render_stateful_widget(popup_table, screen_locs[1], &mut popup_state);

                    // Building the text input
                    let text = Paragraph::new(lock_state.input_pane.0.clone());
                    f.render_widget(text, screen_locs[2]);

                    
                },
                Pane::PopUp(PopUpPane::Services) => {
                    debug!("Rendering a pop up for services over the rest of the windows");
                    let services_loc = lock_state.services.1.selected().unwrap();
                    let passing_service = lock_state.services.0.get(services_loc).unwrap().clone();
                    // I think what's happening with the UI is that I'm creating a new state each
                    // paint. I think this is why it's happening. 
                    let popup = ServicesPopUpElement::new(popup_block, &passing_service);
                    let (popup_table, _) = popup.build_table_element();
                    let screen_locs = popup.build_popup();
                    //lock_state.services_popup = (passing_service.clone(), popup_state);

                    f.render_widget(widgets::Clear, popup_block);
                    debug!{"painting_ui:service_popup_selected:\t{:?}", lock_state.services_popup.1.selected()};
                    f.render_stateful_widget(popup_table, screen_locs[1], &mut lock_state.services_popup.1);
                    let text = Paragraph::new(lock_state.input_pane.0.clone());
                    f.render_widget(text, screen_locs[2]);
                },
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
    )
    .expect("Couldn't close everything out");
}

fn build_event_element(event: &'_ HAEvent) -> (List, ListState) {
    let event_list_items: Vec<_> = vec![ListItem::new(Spans::from(vec![Span::styled(
        Cow::Owned(event.listener_count.to_string()),
        Style::default(),
    )]))];
    let ret_list = List::new(event_list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(event.event.clone()),
    );
    let mut ret_list_state = ListState::default();
    ret_list_state.select(Some(0));
    (ret_list, ret_list_state)
}
