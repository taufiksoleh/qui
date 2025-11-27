use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::time::Duration;

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn next(&mut self) -> Result<Option<InputEvent>> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                return Ok(Some(InputEvent::Key(key)));
            }
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    Key(KeyEvent),
}

impl InputEvent {
    pub fn key_code(&self) -> KeyCode {
        match self {
            InputEvent::Key(key) => key.code,
        }
    }
}
