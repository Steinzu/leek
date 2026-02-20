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

        let tx_input = tx.clone();
        thread::spawn(move || {
            loop {
                if let Ok(true) = event::poll(Duration::from_millis(100)) {
                    if let Ok(CEvent::Key(key)) = event::read() {
                        if key.kind == KeyEventKind::Press {
                            if tx_input.send(Event::Input(key)).is_err() {
                                return;
                            }
                        }
                    }
                }
            }
        });

        thread::spawn(move || {
            loop {
                if tx.send(Event::Tick).is_err() {
                    break;
                }
                thread::sleep(config.tick_rate);
            }
        });

        Events { rx }
    }

    pub fn next(&self) -> Result<Event<KeyEvent>, mpsc::RecvError> {
        self.rx.recv()
    }
}
