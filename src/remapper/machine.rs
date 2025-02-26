use super::event_logic::{as_modifier, get_key_using_mapping, is_modifier};
use super::types::{EvKeyEvent, KeyEventType};
use crate::mapping::*;
use std::collections::{HashMap, HashSet};

/// The machine you pass in the key events through that gives the "replaced" events one should press instead.
pub struct Machine {
    /// Keys currently pressed down according the input events.
    currently_pressed_modifiers: HashSet<Modifier>,

    /// Keys currently pressed down and the keycode it was mapped to at the time of being pressed.
    currently_pressed_keys: HashMap<KeyCode, KeyCode>,

    /// The (readonly) list of mappings passed at initialization.
    mappings: Vec<Mapping>,
}

impl Machine {
    pub fn new(mappings: &Vec<Mapping>) -> Self {
        return Machine {
            currently_pressed_modifiers: HashSet::new(),
            currently_pressed_keys: HashMap::new(),
            mappings: mappings.clone(),
        };
    }

    // Insert an event and get the resulting events to be emitted.
    pub fn insert(&mut self, incoming_event: EvKeyEvent) -> EvKeyEvent {
        match incoming_event.key_event_type {
            KeyEventType::Press => {
                if let Some(modifier) = as_modifier(&incoming_event.ev_key) {
                    self.currently_pressed_modifiers.insert(modifier);
                    incoming_event
                } else {
                    let translated_key = get_key_using_mapping(
                        &self.mappings,
                        &self.currently_pressed_modifiers,
                        incoming_event.ev_key,
                    );
                    self.currently_pressed_keys
                        .insert(incoming_event.ev_key, translated_key);
                    EvKeyEvent {
                        time: incoming_event.time,
                        ev_key: translated_key,
                        key_event_type: incoming_event.key_event_type,
                    }
                }
            }
            KeyEventType::Repeat => {
                if is_modifier(&incoming_event.ev_key) {
                    incoming_event
                } else {
                    EvKeyEvent {
                        time: incoming_event.time,
                        ev_key: self
                            .currently_pressed_keys
                            .get(&incoming_event.ev_key)
                            .unwrap_or(&incoming_event.ev_key)
                            .clone(),
                        key_event_type: incoming_event.key_event_type,
                    }
                }
            }
            KeyEventType::Release => {
                if let Some(modifier) = as_modifier(&incoming_event.ev_key) {
                    self.currently_pressed_modifiers.remove(&modifier);
                    incoming_event
                } else {
                    EvKeyEvent {
                        time: incoming_event.time,
                        ev_key: self
                            .currently_pressed_keys
                            .remove(&incoming_event.ev_key)
                            .unwrap_or(incoming_event.ev_key),
                        key_event_type: incoming_event.key_event_type,
                    }
                }
            }
            _ => incoming_event,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use evdev_rs::enums::EV_KEY;
    use evdev_rs::TimeVal;
    use std::collections::HashSet;

    #[allow(dead_code)]
    fn init_logger() {
        let mut builder = env_logger::Builder::new();
        builder.filter_level(log::LevelFilter::Trace);
        let env = env_logger::Env::new()
            .filter("EVREMAP_LOG")
            .write_style("EVREMAP_LOG_STYLE");
        builder.parse_env(env);
        builder.init();
    }

    #[test]
    fn machine_without_config_is_passthrough_for_press() {
        let dummy_time = TimeVal {
            tv_sec: 0,
            tv_usec: 0,
        };
        let mut machine = Machine::new(&vec![]);
        let dummy_event = EvKeyEvent {
            time: dummy_time,
            ev_key: EV_KEY::KEY_1,
            key_event_type: KeyEventType::Press,
        };
        assert_eq!(machine.insert(dummy_event.clone()), dummy_event);
    }

    fn create_timeval(sec: i64) -> TimeVal {
        TimeVal {
            tv_sec: sec,
            tv_usec: 0,
        }
    }

    macro_rules! assert_machine_insertion_yields_same_event {
        ($machine:ident, $event:expr) => {
            let result = $machine.insert($event.clone());
            assert_eq!(result, $event);
        };
    }

    #[test]
    fn machine_without_config_also_emits_releases() {
        let mut machine = Machine::new(&vec![]);
        assert_machine_insertion_yields_same_event!(
            machine,
            EvKeyEvent {
                time: create_timeval(100),
                ev_key: EV_KEY::KEY_0,
                key_event_type: KeyEventType::Press,
            }
        );
        assert_machine_insertion_yields_same_event!(
            machine,
            EvKeyEvent {
                time: create_timeval(200),
                ev_key: EV_KEY::KEY_0,
                key_event_type: KeyEventType::Release,
            }
        );
    }

    #[test]
    fn machine_without_config_handles_repeats() {
        let mut machine = Machine::new(&vec![]);
        assert_machine_insertion_yields_same_event!(
            machine,
            EvKeyEvent {
                time: create_timeval(100),
                ev_key: EV_KEY::KEY_0,
                key_event_type: KeyEventType::Press,
            }
        );
        assert_machine_insertion_yields_same_event!(
            machine,
            EvKeyEvent {
                time: create_timeval(200),
                ev_key: EV_KEY::KEY_0,
                key_event_type: KeyEventType::Repeat,
            }
        );
        assert_machine_insertion_yields_same_event!(
            machine,
            EvKeyEvent {
                time: create_timeval(210),
                ev_key: EV_KEY::KEY_0,
                key_event_type: KeyEventType::Repeat,
            }
        );
        assert_machine_insertion_yields_same_event!(
            machine,
            EvKeyEvent {
                time: create_timeval(220),
                ev_key: EV_KEY::KEY_0,
                key_event_type: KeyEventType::Repeat,
            }
        );
        assert_machine_insertion_yields_same_event!(
            machine,
            EvKeyEvent {
                time: create_timeval(320),
                ev_key: EV_KEY::KEY_0,
                key_event_type: KeyEventType::Release,
            }
        );
    }

    #[test]
    fn machine_without_config_passthrough_for_two_presses() {
        let mut machine = Machine::new(&vec![]);
        assert_machine_insertion_yields_same_event!(
            machine,
            EvKeyEvent {
                time: create_timeval(100),
                ev_key: EV_KEY::KEY_0,
                key_event_type: KeyEventType::Press,
            }
        );
        assert_machine_insertion_yields_same_event!(
            machine,
            EvKeyEvent {
                time: create_timeval(200),
                ev_key: EV_KEY::KEY_0,
                key_event_type: KeyEventType::Release,
            }
        );
        assert_machine_insertion_yields_same_event!(
            machine,
            EvKeyEvent {
                time: create_timeval(300),
                ev_key: EV_KEY::KEY_1,
                key_event_type: KeyEventType::Press,
            }
        );
        assert_machine_insertion_yields_same_event!(
            machine,
            EvKeyEvent {
                time: create_timeval(400),
                ev_key: EV_KEY::KEY_1,
                key_event_type: KeyEventType::Release,
            }
        );
    }

    #[test]
    fn machine_without_config_passthrough_two_quick_presses() {
        let mut machine = Machine::new(&vec![]);
        assert_machine_insertion_yields_same_event!(
            machine,
            EvKeyEvent {
                time: create_timeval(100),
                ev_key: EV_KEY::KEY_A,
                key_event_type: KeyEventType::Press,
            }
        );
        assert_machine_insertion_yields_same_event!(
            machine,
            EvKeyEvent {
                time: create_timeval(110),
                ev_key: EV_KEY::KEY_B,
                key_event_type: KeyEventType::Press,
            }
        );
        assert_machine_insertion_yields_same_event!(
            machine,
            EvKeyEvent {
                time: create_timeval(200),
                ev_key: EV_KEY::KEY_A,
                key_event_type: KeyEventType::Release,
            }
        );
        assert_machine_insertion_yields_same_event!(
            machine,
            EvKeyEvent {
                time: create_timeval(210),
                ev_key: EV_KEY::KEY_B,
                key_event_type: KeyEventType::Release,
            }
        );
    }

    #[test]
    fn handles_press_and_release_of_mapping_without_modifier() {
        let mut machine = Machine::new(&vec![Mapping::Remap {
            input: EV_KEY::KEY_0,
            modifiers: HashSet::new(),
            output: EV_KEY::KEY_1,
        }]);

        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(100),
                ev_key: EV_KEY::KEY_0,
                key_event_type: KeyEventType::Press,
            }),
            EvKeyEvent {
                time: create_timeval(100),
                ev_key: EV_KEY::KEY_1,
                key_event_type: KeyEventType::Press,
            }
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(200),
                ev_key: EV_KEY::KEY_0,
                key_event_type: KeyEventType::Release,
            }),
            EvKeyEvent {
                time: create_timeval(200),
                ev_key: EV_KEY::KEY_1,
                key_event_type: KeyEventType::Release,
            }
        );
    }

    #[test]
    fn handles_press_and_release_of_mapping_with_ctrl() {
        let mut machine = Machine::new(&vec![Mapping::Remap {
            input: EV_KEY::KEY_0,
            modifiers: HashSet::from([Modifier::LeftCtrl]),
            output: EV_KEY::KEY_1,
        }]);

        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(50),
                ev_key: EV_KEY::KEY_LEFTCTRL,
                key_event_type: KeyEventType::Press,
            }),
            EvKeyEvent {
                time: create_timeval(50),
                ev_key: EV_KEY::KEY_LEFTCTRL,
                key_event_type: KeyEventType::Press,
            }
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(100),
                ev_key: EV_KEY::KEY_0,
                key_event_type: KeyEventType::Press,
            }),
            EvKeyEvent {
                time: create_timeval(100),
                ev_key: EV_KEY::KEY_1,
                key_event_type: KeyEventType::Press,
            }
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(200),
                ev_key: EV_KEY::KEY_0,
                key_event_type: KeyEventType::Release,
            }),
            EvKeyEvent {
                time: create_timeval(200),
                ev_key: EV_KEY::KEY_1,
                key_event_type: KeyEventType::Release,
            }
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(300),
                ev_key: EV_KEY::KEY_LEFTCTRL,
                key_event_type: KeyEventType::Release,
            }),
            EvKeyEvent {
                time: create_timeval(300),
                ev_key: EV_KEY::KEY_LEFTCTRL,
                key_event_type: KeyEventType::Release,
            }
        );
    }

    #[test]
    fn handles_arashs_arrow_up_key_binding_release_key_first() {
        let mut machine = Machine::new(&vec![Mapping::Remap {
            input: EV_KEY::KEY_K,
            modifiers: HashSet::from([Modifier::RightAlt]),
            output: EV_KEY::KEY_UP,
        }]);

        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(50),
                ev_key: EV_KEY::KEY_RIGHTALT,
                key_event_type: KeyEventType::Press,
            }),
            EvKeyEvent {
                time: create_timeval(50),
                ev_key: EV_KEY::KEY_RIGHTALT,
                key_event_type: KeyEventType::Press,
            }
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(100),
                ev_key: EV_KEY::KEY_K,
                key_event_type: KeyEventType::Press,
            }),
            EvKeyEvent {
                time: create_timeval(100),
                ev_key: EV_KEY::KEY_UP,
                key_event_type: KeyEventType::Press,
            }
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(200),
                ev_key: EV_KEY::KEY_K,
                key_event_type: KeyEventType::Release,
            }),
            EvKeyEvent {
                time: create_timeval(200),
                ev_key: EV_KEY::KEY_UP,
                key_event_type: KeyEventType::Release,
            }
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(300),
                ev_key: EV_KEY::KEY_RIGHTALT,
                key_event_type: KeyEventType::Release,
            }),
            EvKeyEvent {
                time: create_timeval(300),
                ev_key: EV_KEY::KEY_RIGHTALT,
                key_event_type: KeyEventType::Release,
            }
        );
    }

    #[test]
    fn handles_arashs_arrow_up_key_binding_release_modifier_first() {
        let mut machine = Machine::new(&vec![Mapping::Remap {
            input: EV_KEY::KEY_K,
            modifiers: HashSet::from([Modifier::RightAlt]),
            output: EV_KEY::KEY_UP,
        }]);

        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(50),
                ev_key: EV_KEY::KEY_RIGHTALT,
                key_event_type: KeyEventType::Press,
            }),
            EvKeyEvent {
                time: create_timeval(50),
                ev_key: EV_KEY::KEY_RIGHTALT,
                key_event_type: KeyEventType::Press,
            }
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(100),
                ev_key: EV_KEY::KEY_K,
                key_event_type: KeyEventType::Press,
            }),
            EvKeyEvent {
                time: create_timeval(100),
                ev_key: EV_KEY::KEY_UP,
                key_event_type: KeyEventType::Press,
            }
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(200),
                ev_key: EV_KEY::KEY_RIGHTALT,
                key_event_type: KeyEventType::Release,
            }),
            EvKeyEvent {
                time: create_timeval(200),
                ev_key: EV_KEY::KEY_RIGHTALT,
                key_event_type: KeyEventType::Release,
            }
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(300),
                ev_key: EV_KEY::KEY_K,
                key_event_type: KeyEventType::Release,
            }),
            EvKeyEvent {
                time: create_timeval(300),
                ev_key: EV_KEY::KEY_UP,
                key_event_type: KeyEventType::Release,
            }
        );
    }

    #[test]
    fn handles_repeating() {
        let mut machine = Machine::new(&vec![Mapping::Remap {
            input: EV_KEY::KEY_T,
            modifiers: HashSet::from([Modifier::RightAlt]),
            output: EV_KEY::KEY_RIGHT,
        }]);

        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(50),
                ev_key: EV_KEY::KEY_RIGHTALT,
                key_event_type: KeyEventType::Press,
            }),
            EvKeyEvent {
                time: create_timeval(50),
                ev_key: EV_KEY::KEY_RIGHTALT,
                key_event_type: KeyEventType::Press,
            }
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(100),
                ev_key: EV_KEY::KEY_T,
                key_event_type: KeyEventType::Press,
            }),
            EvKeyEvent {
                time: create_timeval(100),
                ev_key: EV_KEY::KEY_RIGHT,
                key_event_type: KeyEventType::Press,
            }
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(110),
                ev_key: EV_KEY::KEY_T,
                key_event_type: KeyEventType::Repeat,
            }),
            EvKeyEvent {
                time: create_timeval(110),
                ev_key: EV_KEY::KEY_RIGHT,
                key_event_type: KeyEventType::Repeat,
            }
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(120),
                ev_key: EV_KEY::KEY_T,
                key_event_type: KeyEventType::Repeat,
            }),
            EvKeyEvent {
                time: create_timeval(120),
                ev_key: EV_KEY::KEY_RIGHT,
                key_event_type: KeyEventType::Repeat,
            }
        );
    }

    #[test]
    fn keeps_repeating_key_without_modifiers_once_started() {
        let mut machine = Machine::new(&vec![Mapping::Remap {
            input: EV_KEY::KEY_T,
            modifiers: HashSet::from([Modifier::RightAlt]),
            output: EV_KEY::KEY_RIGHT,
        }]);

        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(100),
                ev_key: EV_KEY::KEY_T,
                key_event_type: KeyEventType::Press,
            }),
            EvKeyEvent {
                time: create_timeval(100),
                ev_key: EV_KEY::KEY_T,
                key_event_type: KeyEventType::Press,
            }
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(200),
                ev_key: EV_KEY::KEY_RIGHTALT,
                key_event_type: KeyEventType::Press,
            }),
            EvKeyEvent {
                time: create_timeval(200),
                ev_key: EV_KEY::KEY_RIGHTALT,
                key_event_type: KeyEventType::Press,
            }
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(210),
                ev_key: EV_KEY::KEY_T,
                key_event_type: KeyEventType::Repeat,
            }),
            EvKeyEvent {
                time: create_timeval(210),
                ev_key: EV_KEY::KEY_T,
                key_event_type: KeyEventType::Repeat,
            }
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(220),
                ev_key: EV_KEY::KEY_T,
                key_event_type: KeyEventType::Repeat,
            }),
            EvKeyEvent {
                time: create_timeval(220),
                ev_key: EV_KEY::KEY_T,
                key_event_type: KeyEventType::Repeat,
            }
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(300),
                ev_key: EV_KEY::KEY_T,
                key_event_type: KeyEventType::Release,
            }),
            EvKeyEvent {
                time: create_timeval(300),
                ev_key: EV_KEY::KEY_T,
                key_event_type: KeyEventType::Release,
            }
        );
        // Pressing down again we now keep the modifier keys.
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(400),
                ev_key: EV_KEY::KEY_T,
                key_event_type: KeyEventType::Press,
            }),
            EvKeyEvent {
                time: create_timeval(400),
                ev_key: EV_KEY::KEY_RIGHT,
                key_event_type: KeyEventType::Press,
            }
        );
    }
}
