pub mod ui;

use tui::{
    backend::CrosstermBackend,
    widgets::{Widget, Block, Borders, Tabs},
    layout::{Layout, Constraint,Direction},
    Terminal,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

pub fn draw_ui(){
    enable_raw_mode()?;
    let mut std_out = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(std_out);
    let mut terminal = Terminal::new(backedn)?;

    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default()
            .title("Bloock")
            .borders(Borders::ALL);
        f.render_widget(block, size);
    })?;
}

