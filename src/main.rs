use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

mod app;
mod events;
mod ui;

use app::App;
use events::{Event, Events};

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    let events = Events::new();

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        match events.next()? {
            Event::Input(key) => {
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    break;
                }
                match key.code {
                    KeyCode::Char(' ') => app.toggle_play(),
                    KeyCode::Up | KeyCode::Char('k') => app.prev_item(),
                    KeyCode::Down | KeyCode::Char('j') => app.next_item(),
                    KeyCode::PageUp => app.volume_up(),
                    KeyCode::PageDown => app.volume_down(),
                    KeyCode::Enter => app.enter_selected(),
                    KeyCode::Tab => app.play_folder(),
                    KeyCode::Backspace => app.go_up(),
                    KeyCode::Left => app.prev_track(),
                    KeyCode::Right => app.next_track(),
                    _ => {}
                }
            }
            Event::Tick => {
                app.on_tick();
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
