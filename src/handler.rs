use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // exit application on ESC or q
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.running = false;
        }

        // exit application on Ctrl-D
        KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.running = false;
            }
        }
        _ => {}
    }
    Ok(())
}
