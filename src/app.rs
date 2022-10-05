use std::error;
use std::fmt::Display;
use std::time::Duration;

use figlet_rs::FIGfont;
use tui::backend::Backend;
use tui::layout::{Layout, Direction, Constraint, Alignment};
use tui::terminal::Frame;
use tui::widgets::{Block, Borders, Paragraph, BorderType, Clear};

use chrono::prelude::*;

use crate::event::EventHandler;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone, Default)]
struct TokenBlock {
    pub is_constant: bool,
    pub transition_progress: u128,
    pub transition_timing: u128,
    pub size: usize,
    pub curr_token: String,
    pub new_token: String,
}

#[derive(Debug, Clone)]
struct Token {
    pub format_string: String,
    pub blocks: Vec<TokenBlock>
}

#[derive(Debug, Clone)]
struct AnimatedTime {
    pub format_tokens: Vec<Token>,
    timing: u128
}

impl AnimatedTime {
    pub fn new() -> Self {        
        Self { format_tokens: Vec::new(), timing: 250 }.set_format("%X")
    }

    pub fn set_timing(mut self, timing: u128) -> Self {
        for token in &mut self.format_tokens {
            for block in &mut token.blocks {
                block.transition_timing = self.timing;
            }
        }
        
        self
    }

    pub fn set_format(mut self, format_string: &str) -> Self {
        let max_dt = Local.ymd(3000, 11, 11).and_hms_nano(12, 11, 11, 111111111);
        let min_dt = Local.ymd(2222, 2, 2).and_hms_nano(1, 0, 0, 0);
        // let max_dt = Local::now();
        // let min_dt = Local::now();

        self.format_tokens.clear();

        let mut token = String::new();
        for ch in format_string.to_string().chars() {
            token.push(ch);
            if !token.starts_with('%') || token.len() > 2 || (token.len() == 2 && !"-_0".contains(ch)) {
                let max_dt = max_dt.format(&token).to_string();
                let min_dt = min_dt.format(&token).to_string();

                let mut blocks: Vec<TokenBlock> = Vec::new();
                if min_dt.len() != max_dt.len() {
                    blocks.push(TokenBlock { is_constant: max_dt == min_dt, transition_progress: 0, transition_timing: self.timing, size: min_dt.len().max(max_dt.len()), ..TokenBlock::default() });
                } else {
                    for (min_ch, max_ch) in min_dt.chars().zip(max_dt.chars()) {
                        blocks.push(TokenBlock{ is_constant: min_ch == max_ch, transition_progress: 0, transition_timing: self.timing, size: 1, ..TokenBlock::default()});
                    }
                }
                self.format_tokens.push(Token {format_string: token, blocks});
                token = String::new();
            }
        }

        self.tick_logic();
        for token in &mut self.format_tokens {
            for block in &mut token.blocks {
                block.curr_token = block.new_token.clone();
            }
        }

        self
    }

    pub fn tick_logic(&mut self) {
        let dt = Local::now(); // Add timezone stuff
        for token in &mut self.format_tokens {
            let time_string = dt.format(&token.format_string).to_string();
            let mut time_chars = time_string.chars();
            for block in &mut token.blocks {
                block.new_token = (&mut time_chars).take(block.size).collect();
            }
        }
    }

    pub fn tick_render(&mut self, duration: Duration) -> bool {
        let mut is_transitioning = false;
        let duration = duration.as_millis();
        for token in &mut self.format_tokens {
            for block in &mut token.blocks {
                if block.is_constant {
                    // continue
                } else if block.transition_progress > block.transition_timing {
                    block.transition_progress = 0;
                    block.curr_token = block.new_token.clone();
                } else if block.new_token != block.curr_token {
                    is_transitioning = true;
                    block.transition_progress += duration;
                }
            }
        }
        is_transitioning
    }
}

/// Application.
#[derive(Debug)]
pub struct App {
    pub running: bool,
    animated_time: AnimatedTime,
    direction: u8
}

impl Default for App {
    fn default() -> Self {
        Self { running: true, animated_time: AnimatedTime::new(), direction: 0 }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(transition_timing: u128) -> Self {
        Self { animated_time: AnimatedTime::new().set_timing(transition_timing) , ..App::default() }
    }

    /// Handles the tick event of the terminal.
    pub fn tick_logic(&mut self, duration: Duration, event: &EventHandler) {
        self.animated_time.tick_logic();
        event.trigger_animation(true);
    }

    pub fn tick_render(&mut self, duration: Duration, event: &EventHandler) {
        let is_transitioning = self.animated_time.tick_render(duration);
        event.trigger_animation(is_transitioning);
    }

    /// Renders the user interface widgets.
    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        // This is where you add new widgets.
        // See the following resources:
        // - https://docs.rs/tui/0.16.0/tui/widgets/index.html
        // - https://github.com/fdehau/tui-rs/tree/v0.16.0/examples
        let mut constraints: Vec<Constraint> = Vec::new();
        let mut width: usize = 0;
        for tokens in &self.animated_time.format_tokens {
            for block in &tokens.blocks {
                let size = block.size * match block.is_constant {
                    true => 8,
                    false => 15
                };
                constraints.push(Constraint::Length(size as u16));
                width += size
            }
        }
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints.as_slice())
            .horizontal_margin((frame.size().width - width as u16) / 2)
            .vertical_margin((frame.size().height - 9) / 2)
            .split(frame.size());
        let standard_font = FIGfont::standand().unwrap();
        let transition_box = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded);
        let digit_box = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let mut i = 0;
        for tokens in &self.animated_time.format_tokens {
            for block in &tokens.blocks {
                let figure = match standard_font.convert(&block.curr_token) {
                    Some(figure) => figure,
                    None => standard_font.convert(" ").unwrap()
                };
                frame.render_widget(Paragraph::new(format!("\n\n{}", figure)).alignment(Alignment::Center), chunks[i]);
                if !block.is_constant {
                    frame.render_widget(digit_box.clone(), chunks[i]);
                }
                if block.transition_progress > 0 {
                    let mut direction = Direction::Vertical;
                    if (self.direction % 2) == 1 {
                        direction = Direction::Horizontal;
                    }
                    let (constraint, chunk_index) = {
                        let constraint =  (((100 * block.transition_progress) / block.transition_timing) as u16).min(100);
                        if self.direction > 1 {
                            ([Constraint::Percentage(100 - constraint), Constraint::Percentage(constraint)], 1)
                        } else {
                            ([Constraint::Percentage(constraint), Constraint::Percentage(0)], 0)
                        }
                    };
                    let chunks = Layout::default()
                        .direction(direction)
                        .constraints(constraint)
                        .split(chunks[i]);
                    frame.render_widget(Clear, chunks[chunk_index]);
                    let figure = standard_font.convert(&block.new_token).unwrap().to_string();
                    frame.render_widget(transition_box.clone(), chunks[chunk_index]);
                    frame.render_widget(Paragraph::new(format!("\n\n{}", figure)).alignment(Alignment::Center), chunks[chunk_index]);
                }
                i += 1
            }
        }
    }
}
