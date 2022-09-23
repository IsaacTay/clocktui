use std::cmp::min;
use std::sync::Arc;
use std::{error, thread};
use std::time::Duration;

use crossterm::event::Event;
use figlet_rs::FIGfont;
use tui::backend::Backend;
use tui::layout::{Layout, Direction, Constraint, Alignment};
use tui::terminal::Frame;
use tui::widgets::{Block, Borders, Paragraph, BorderType, Clear};

use chrono::{self, Timelike};

use crate::event::EventHandler;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone, Copy)]
struct Digit {
    pub transition: u32,
    pub transition_timing: u32,
    pub new_time: u32,
    pub current_time: u32,
}

impl Default for Digit {
    fn default() -> Self {
        Self {transition: 501, transition_timing: 500, new_time: 0, current_time: 0}
    }
}

impl Digit {
    pub fn new(transition_timing: u32) -> Self {
        Self { transition: transition_timing, transition_timing, new_time: 0, current_time: 0}
    }
}

const DIGITS: usize = 6;

/// Application.
#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub transitioning: bool,
    digits: [Digit; DIGITS],
    direction: [u8; DIGITS]
}

impl Default for App {
    fn default() -> Self {
        Self { running: true, transitioning: true, digits: [Digit::default(); DIGITS], direction: [0; DIGITS] }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(transition_timing: u32) -> Self {
        Self { running: true, transitioning: true, digits: [Digit::new(transition_timing); DIGITS], direction: [0; DIGITS] }
    }

    /// Handles the tick event of the terminal.
    pub fn logic_tick(&mut self, duration: Duration, event: &EventHandler) {
        let t = chrono::offset::Local::now();
        let t = [t.hour(), t.minute(), t.second()];
        for (i, t) in t.iter().enumerate() {
            self.digits[2*i].new_time = t/10;
            self.digits[1+2*i].new_time = t%10;
        }
        event.transitioning(true);
    }

    pub fn render_tick(&mut self, duration: Duration, event: &EventHandler) {
        let mut transitioning = false;
        for (i, d) in self.digits.as_mut().iter_mut().enumerate() {
            if d.transition > d.transition_timing {
                d.transition = 0;
                self.direction[i] = (self.direction[i] + 1) % 4;
                d.current_time = d.new_time;
            }
            if d.new_time != d.current_time {
                transitioning = true;
                d.transition += duration.as_millis() as u32;
            }
        }
        event.transitioning(transitioning);
    }

    /// Renders the user interface widgets.
    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        // This is where you add new widgets.
        // See the following resources:
        // - https://docs.rs/tui/0.16.0/tui/widgets/index.html
        // - https://github.com/fdehau/tui-rs/tree/v0.16.0/examples
        let mut constraints: Vec<Constraint> = Vec::new();
        for (i, _) in self.digits.into_iter().enumerate() {
            constraints.push(Constraint::Length(15));
            if i % 3 == 2 {
                constraints.push(Constraint::Length(10));
            }
        }
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints.as_slice())
            .horizontal_margin((frame.size().width - 15 * 6) / 2)
            .vertical_margin((frame.size().height - 9) / 2)
            .split(frame.size());
        let standard_font = FIGfont::standand().unwrap();
        let transition_box = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded);
        let digit_box = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        for (i, digit) in self.digits.into_iter().enumerate() {
            let figure = standard_font.convert(&format!("{}", digit.current_time)).unwrap();
            frame.render_widget(Paragraph::new(format!("\n{}", figure)).alignment(Alignment::Center).block(digit_box.clone()), chunks[i]);
            if self.digits[i].transition > 0 {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(((100 * digit.transition) / digit.transition_timing) as u16), Constraint::Percentage(0)])
                    .split(chunks[i]);
                frame.render_widget(Clear, chunks[0]);
                let figure = standard_font.convert(&format!("{}", digit.new_time)).unwrap().to_string();
                frame.render_widget(Paragraph::new(format!("\n{}", figure)).alignment(Alignment::Center).block(transition_box.clone()), chunks[0]);
            }
        }
    }
}
