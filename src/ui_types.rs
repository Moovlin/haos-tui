use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, List, ListState, Row, Table, TableState},
};

use std::borrow::Cow;

use haoscli::types::Event as HAEvent;

use haoscli::types::{Service, State};

/// Enum to determine which pane is currently the active pane.
#[derive(PartialEq, Debug, Default, Clone)]
pub enum Pane {
    #[default]
    Events,
    Services,
    States,
    PopUp(PopUpPane),
    Search,
    None,
}

/// Enum to determine where we need to look for the selected item to create a pop up for it.
#[derive(PartialEq, Debug, Default, Clone)]
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
    pub events_popop: (HAEvent, ListState),

    pub services: (Vec<Service>, TableState),
    pub services_popup: (Service, TableState),

    pub services_popup_selected: String, 

    pub states: (Vec<State>, ListState),
    pub states_popup: (State, TableState),


    pub search: String,
    pub input_pane: (String, bool),    // This should really be a struct, ideally, each "pop up"
                                       // should manage it's search state via a more complex struct
                                       // and a trait that allows for input to it/resetting it.
                                       // Should make code more uniform. Honestly, I don't think I
                                       // could do it with a super trait, which sucks.
}

impl UiState {
    pub fn get_selected_service(&self) -> &Service {
        let selected_service = self.services.1.selected().unwrap();
        self.services.0.get(selected_service).unwrap()
    }
}

pub trait BuildPopup {
    fn set_loc(&mut self, popup_loc: Rect);

    fn build_popup(&self) -> Vec<Rect>;
}

pub trait BuildList {
    fn build_list_element(&self) -> (Box<List>, Box<ListState>);
}

pub trait BuildTable {
    fn build_table_element(&self) -> (Table, TableState);
}

pub struct EventsPopUpElement {
    popup_loc: Rect,
}

impl EventsPopUpElement {}

pub struct ServicesPopUpElement<'popup> {
    popup_loc: Rect,
    service: &'popup Service,
}

impl<'popup> ServicesPopUpElement<'popup> {
    pub fn new(popup_loc: Rect, service: &'popup Service) -> Self {
        ServicesPopUpElement { popup_loc, service }
    }
}

impl<'popup> BuildPopup for ServicesPopUpElement<'popup> {
    fn set_loc(&mut self, popup_loc: Rect) {
        self.popup_loc = popup_loc;
    }

    fn build_popup(&self) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(30),
                Constraint::Percentage(60),
            ])
            .split(self.popup_loc)
    }
}

impl<'popup> BuildTable for ServicesPopUpElement<'popup> {
    fn build_table_element(&self) -> (Table, TableState) {
        let value_map = self
            .service
            .services
            .as_object()
            .expect("Couldn't convert the individual service to map");
        let mut services_table_rows: Vec<Row> = Vec::new();
        for (serv, desc) in value_map.iter() {
            let push_row = vec![Cell::from(serv.to_string()), Cell::from(desc.to_string())];

            services_table_rows.push(Row::new(push_row));
        }

        let service_table = Table::new(services_table_rows)
            .style(Style::default())
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black))
            .header(Row::new(vec!["Service", "Description"]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.service.domain.clone()),
            )
            .widths(&[Constraint::Percentage(10), Constraint::Percentage(90)]);
        let mut ret_table_state = TableState::default();
        ret_table_state.select(Some(0));
        (service_table, ret_table_state)
    }
}

pub struct StatesPopUpElement<'popup> {
    //text_bar_loc: Rect,
    //table_loc: Rect,
    popup_loc: Rect,
    state: &'popup State,
}

impl<'popup> StatesPopUpElement<'popup> {
    pub fn new(popup_loc: Rect, state: &'popup State) -> Self {
        StatesPopUpElement { popup_loc, state }
    }
}

impl<'popup> BuildPopup for StatesPopUpElement<'popup> {
    fn set_loc(&mut self, popup_loc: Rect) {
        self.popup_loc = popup_loc
    }

    fn build_popup(&self) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(30),
                Constraint::Percentage(60),
            ])
            .split(self.popup_loc)
    }
}

impl<'popup> BuildTable for StatesPopUpElement<'popup> {
    fn build_table_element(&self) -> (Table, TableState) {
        let states_table_rows: Vec<Row> = vec![Row::new(vec![
            Cell::from(Cow::Owned(self.state.state.to_string())),
            Cell::from(Cow::Owned(self.state.last_changed.to_string())),
            Cell::from(Cow::Owned(self.state.attributes.to_string())),
        ])];

        let state_table = Table::new(states_table_rows)
            .style(Style::default())
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black))
            .header(Row::new(vec!["State", "Changed Last At", "Attributes"]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.state.entity_id.clone()),
            )
            .widths(&[
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(80),
            ]);
        let mut ret_table_state = TableState::default();
        ret_table_state.select(Some(0));
        (state_table, ret_table_state)
    }
}
