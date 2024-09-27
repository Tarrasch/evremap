use evdev_rs::{InputEvent, TimeVal};
use evdev_rs::enums::{EventCode, EV_KEY};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KeyEventType {
    Release,
    Press,
    Repeat,
    Unknown(i32),
}

impl KeyEventType {
    pub fn from_value(value: i32) -> Self {
        match value {
            0 => KeyEventType::Release,
            1 => KeyEventType::Press,
            2 => KeyEventType::Repeat,
            _ => KeyEventType::Unknown(value),
        }
    }

    pub fn value(&self) -> i32 {
        match self {
            Self::Release => 0,
            Self::Press => 1,
            Self::Repeat => 2,
            Self::Unknown(n) => *n,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EvKeyEvent {
    /// The time at which event occured
    pub time: TimeVal,
    pub ev_key: EV_KEY,
    pub key_event_type: KeyEventType,
}

impl EvKeyEvent {
    pub fn as_input_event(&self) -> InputEvent {
        InputEvent {
            time: self.time,
            event_code: EventCode::EV_KEY(self.ev_key),
            value: self.key_event_type.value(),
        }
    }
}
