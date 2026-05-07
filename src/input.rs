use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

pub struct Input;

#[derive(Debug)]
pub enum Action {
    Move { dx: f64, dz: f64 },
    Jump,
    Look { dyaw: f64, dpitch: f64 },
    Place,
    Break,
    SelectBlock(usize),
    Save,
    Quit,
    None,
}

impl Input {
    pub fn poll() -> Action {
        // Drain all pending events, return the last meaningful one
        let mut action = Action::None;
        while event::poll(Duration::ZERO).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                action = Self::map_key(key);
            }
        }
        action
    }

    fn map_key(key: KeyEvent) -> Action {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Action::Quit;
        }

        match key.code {
            KeyCode::Esc => Action::Quit,
            KeyCode::Char('w') => Action::Move { dx: 0.0, dz: -1.0 },
            KeyCode::Char('s') => Action::Move { dx: 0.0, dz: 1.0 },
            KeyCode::Char('a') => Action::Move { dx: -1.0, dz: 0.0 },
            KeyCode::Char('d') => Action::Move { dx: 1.0, dz: 0.0 },
            KeyCode::Char(' ') => Action::Jump,
            KeyCode::Char('e') => Action::Place,
            KeyCode::Char('q') => Action::Break,
            KeyCode::Char(c @ '1'..='5') => Action::SelectBlock(c as usize - '1' as usize),
            KeyCode::F(5) => Action::Save,
            KeyCode::Left => Action::Look { dyaw: -0.1, dpitch: 0.0 },
            KeyCode::Right => Action::Look { dyaw: 0.1, dpitch: 0.0 },
            KeyCode::Up => Action::Look { dyaw: 0.0, dpitch: -0.05 },
            KeyCode::Down => Action::Look { dyaw: 0.0, dpitch: 0.05 },
            _ => Action::None,
        }
    }
}
