use crate::app::AppResult;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::borrow::BorrowMut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex, Condvar};
use std::thread;
use std::time::{Duration, Instant};

/// Terminal events.
#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Program Logic tick.
    LogicTick(Duration),
    /// Terminal Render tick
    RenderTick(Duration),
    /// Key press.
    Key(KeyEvent),
    /// Mouse click/scroll.
    Mouse(MouseEvent),
    /// Terminal resize.
    Resize(u16, u16),
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    sender: mpsc::Sender<Event>,
    /// Event receiver channel.
    receiver: mpsc::Receiver<Event>,
    /// Event handler thread.
    handlers: [thread::JoinHandle<()>; 2],

    is_animating: Arc<(Mutex<bool>, Condvar)>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new(tick_rate: u64, render_tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let render_tick_rate = Duration::from_millis(render_tick_rate);
        let (sender, receiver) = mpsc::channel();
        let is_animating = Arc::new((Mutex::new(false), Condvar::new()));
        let handlers = [
            {
                let mut last_tick = Instant::now();
                let sender = sender.clone();
                thread::spawn(move || {
                    loop {
                        let timeout = tick_rate
                            .checked_sub(last_tick.elapsed())
                            .unwrap_or(tick_rate);

                        if event::poll(timeout).expect("no events available") {
                            match event::read().expect("unable to read event") {
                                CrosstermEvent::Key(e) => sender.send(Event::Key(e)),
                                CrosstermEvent::Mouse(e) => sender.send(Event::Mouse(e)),
                                CrosstermEvent::Resize(w, h) => sender.send(Event::Resize(w, h)),
                                _ => Ok(()),
                                // CrosstermEvent::FocusGained => todo!(),
                                // CrosstermEvent::FocusLost => todo!(),
                                // CrosstermEvent::Paste(_) => todo!(),
                            }
                            .expect("failed to send terminal event")
                        }

                        if last_tick.elapsed() >= tick_rate {
                            sender.send(Event::LogicTick(last_tick.elapsed())).expect("failed to send tick event");
                            last_tick = Instant::now();
                        }
                    }
                })
            },
            {
                let is_animating = is_animating.clone();
                let sender = sender.clone();
                let mut last_tick = Instant::now();
                thread::spawn(move || {
                    let (is_animating, cvar) = &*is_animating;
                    loop {
                        drop(cvar.wait(is_animating.lock().unwrap()).unwrap());
                        last_tick = Instant::now();
                        while *is_animating.lock().unwrap() {
                            if last_tick.elapsed() >= render_tick_rate {
                                sender.send(Event::RenderTick(last_tick.elapsed())).expect("failed to send tick event");
                                last_tick = Instant::now();
                            }
                        }
                    }
                })
            }
        ];
        Self {
            sender,
            receiver,
            handlers,
            is_animating
        }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub fn next(&self) -> AppResult<Event> {
        Ok(self.receiver.recv()?)
    }

    pub fn trigger_animation(&self, new_state: bool) {
        let (is_animating, cvar) = &*self.is_animating;
        let mut transitioning = is_animating.lock().unwrap();
        if !*transitioning && new_state {
            cvar.notify_one();
        }
        *transitioning = new_state;
    }
}
