use crossterm::event::{self, Event as CEvent, KeyEvent, KeyEventKind};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct Events {
    rx: mpsc::Receiver<Event<KeyEvent>>,
    _input_handle: thread::JoinHandle<()>,
    _tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            tick_rate: Duration::from_millis(250),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let _input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                loop {
                    if let Ok(true) = event::poll(Duration::from_millis(100)) {
                        if let Ok(CEvent::Key(key)) = event::read() {
                            // Only send the event if it's a Press
                            if key.kind == KeyEventKind::Press {
                                if let Err(_) = tx.send(Event::Input(key)) {
                                    return;
                                }
                            }
                        }
                    }
                }
            })
        };
        let _tick_handle = {
            thread::spawn(move || {
                loop {
                    if let Err(_) = tx.send(Event::Tick) {
                        break;
                    }
                    thread::sleep(config.tick_rate);
                }
            })
        };
        Events {
            rx,
            _input_handle,
            _tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<KeyEvent>, mpsc::RecvError> {
        self.rx.recv()
    }
}
