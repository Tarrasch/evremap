use super::event_logic::compute_keys_based_on_state;
use super::types::{EvKeyEvent, KeyEventType};
use crate::mapping::*;
use std::collections::HashSet;

/// The machine you pass in the key events through that gives the "replaced" events one should press instead.
pub struct Machine {
    /// Keys currently pressed down according the input events.
    input_state: HashSet<KeyCode>,

    /// Keys currently pressed down according the output events.
    output_keys: HashSet<KeyCode>,

    /// The (readonly) list of mappings passed at initialization.
    mappings: Vec<Mapping>,
}

impl Machine {
    pub fn new(mappings: &Vec<Mapping>) -> Self {
        return Machine {
            input_state: HashSet::new(),
            mappings: mappings.clone(),
            output_keys: HashSet::new(),
        };
    }

    // Insert an event and get the resulting events to be emitted.
    pub fn insert(&mut self, incoming_event: EvKeyEvent) -> Vec<EvKeyEvent> {
        match incoming_event.key_event_type {
            KeyEventType::Press => {
                self.input_state.insert(incoming_event.ev_key.clone());
            }
            KeyEventType::Release => {
                if !self.input_state.remove(&incoming_event.ev_key) {
                    log::trace!("There was nothing to be removed");
                }
            }
            _ => {}
        }
        let outgoing_events: Vec<EvKeyEvent> = self.get_keys_to_emit(&incoming_event);
        // Update states of local variables.
        for ev_key_event in &outgoing_events {
            match ev_key_event.key_event_type {
                KeyEventType::Press | KeyEventType::Repeat => {
                    self.output_keys.insert(ev_key_event.ev_key.clone());
                }
                KeyEventType::Release => {
                    self.output_keys.remove(&ev_key_event.ev_key);
                }
                _ => {}
            }
        }
        outgoing_events
    }

    fn get_keys_to_emit(&self, event: &EvKeyEvent) -> Vec<EvKeyEvent> {
        match event.key_event_type {
            KeyEventType::Press | KeyEventType::Release => compute_keys_based_on_state(
                &self.mappings,
                &self.input_state,
                &self.output_keys,
                &event.time,
            ),
            KeyEventType::Repeat => {
                match super::event_logic::lookup_mapping(
                    &self.mappings,
                    &self.input_state,
                    event.ev_key,
                ) {
                    Some(Mapping::Remap { output, .. }) => output
                        .iter()
                        .map(|ev_key| EvKeyEvent {
                            time: event.time,
                            ev_key: ev_key.clone(),
                            key_event_type: KeyEventType::Repeat,
                        })
                        .collect(),
                    None => vec![event.clone()],
                }
            }
            KeyEventType::Unknown(_) => {
                vec![event.clone()]
            }
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
        assert_eq!(machine.insert(dummy_event.clone()), vec![dummy_event,]);
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
            assert_eq!(result, vec![$event]);
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
            input: HashSet::from([EV_KEY::KEY_0]),
            output: HashSet::from([EV_KEY::KEY_1]),
        }]);

        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(100),
                ev_key: EV_KEY::KEY_0,
                key_event_type: KeyEventType::Press,
            }),
            vec![EvKeyEvent {
                time: create_timeval(100),
                ev_key: EV_KEY::KEY_1,
                key_event_type: KeyEventType::Press,
            }]
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(200),
                ev_key: EV_KEY::KEY_0,
                key_event_type: KeyEventType::Release,
            }),
            vec![EvKeyEvent {
                time: create_timeval(200),
                ev_key: EV_KEY::KEY_1,
                key_event_type: KeyEventType::Release,
            }]
        );
    }

    #[test]
    fn handles_press_and_release_of_mapping_with_ctrl() {
        let mut machine = Machine::new(&vec![Mapping::Remap {
            input: HashSet::from([EV_KEY::KEY_0, EV_KEY::KEY_LEFTCTRL]),
            output: HashSet::from([EV_KEY::KEY_1]),
        }]);

        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(50),
                ev_key: EV_KEY::KEY_LEFTCTRL,
                key_event_type: KeyEventType::Press,
            }),
            vec![EvKeyEvent {
                time: create_timeval(50),
                ev_key: EV_KEY::KEY_LEFTCTRL,
                key_event_type: KeyEventType::Press,
            }]
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(100),
                ev_key: EV_KEY::KEY_0,
                key_event_type: KeyEventType::Press,
            }),
            vec![EvKeyEvent {
                time: create_timeval(100),
                ev_key: EV_KEY::KEY_1,
                key_event_type: KeyEventType::Press,
            }]
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(200),
                ev_key: EV_KEY::KEY_0,
                key_event_type: KeyEventType::Release,
            }),
            vec![EvKeyEvent {
                time: create_timeval(200),
                ev_key: EV_KEY::KEY_1,
                key_event_type: KeyEventType::Release,
            }]
        );
        assert_eq!(
            machine.insert(EvKeyEvent {
                time: create_timeval(300),
                ev_key: EV_KEY::KEY_LEFTCTRL,
                key_event_type: KeyEventType::Release,
            }),
            vec![EvKeyEvent {
                time: create_timeval(300),
                ev_key: EV_KEY::KEY_LEFTCTRL,
                key_event_type: KeyEventType::Release,
            }]
        );
    }
}
