use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;
use clocktui::app::{App, AppResult};
use clocktui::event::{Event, EventHandler};
use clocktui::handler::handle_key_events;
use clocktui::tui::Tui;

fn main() -> AppResult<()> {
    // Create an application.q
    let mut app = App::default();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(10);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick(duration) => app.tick(duration),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
