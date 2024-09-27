use super::types::{EvKeyEvent, KeyEventType};
use crate::mapping::*;
use evdev_rs::TimeVal;
use std::cmp::Ordering;
use std::collections::HashSet;

fn is_modifier(key: &KeyCode) -> bool {
    match key {
        KeyCode::KEY_FN
        | KeyCode::KEY_LEFTALT
        | KeyCode::KEY_RIGHTALT
        | KeyCode::KEY_LEFTMETA
        | KeyCode::KEY_RIGHTMETA
        | KeyCode::KEY_LEFTCTRL
        | KeyCode::KEY_RIGHTCTRL
        | KeyCode::KEY_LEFTSHIFT
        | KeyCode::KEY_RIGHTSHIFT => true,
        _ => false,
    }
}

/// Orders modifier keys ahead of non-modifier keys.
/// Unfortunately the underlying type doesn't allow direct
/// comparison, but that's ok for our purposes.
fn modifiers_first(a: &KeyCode, b: &KeyCode) -> Ordering {
    if is_modifier(a) {
        if is_modifier(b) {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    } else if is_modifier(b) {
        Ordering::Greater
    } else {
        // Neither are modifiers
        Ordering::Equal
    }
}

fn modifiers_last(a: &KeyCode, b: &KeyCode) -> Ordering {
    modifiers_first(a, b).reverse()
}

fn apply_mapping_to_held_keys(
    mappings: &Vec<Mapping>,
    currently_pressed_keys: &HashSet<KeyCode>,
) -> HashSet<KeyCode> {
    log::trace!("currently_pressed_keys: {:?}", currently_pressed_keys);
    // Start with the input keys
    let mut keys: HashSet<KeyCode> = currently_pressed_keys.clone();

    // Arash note: I removed the variable "keys_minus_remapped". Having it caused too early "releases" of modifier keys to be emitted.
    for Mapping::Remap { input, output } in mappings {
        if input.is_subset(&keys) {
            for i in input {
                if !is_modifier(i) {
                    keys.remove(i);
                }
            }
            for o in output {
                // Outputs that apply are not visible as
                // inputs for later remap rules
                if !is_modifier(o) {
                    keys.insert(o.clone());
                }
            }
        }
    }

    keys
}

/// Compute the difference between our desired set of keys
/// and the set of keys that are currently pressed in the
/// output device.
/// Release any keys that should not be pressed, and then
/// press any keys that should be pressed.
///
/// When releasing, release modifiers last so that mappings
/// that produce eg: CTRL-C don't emit a random C character
/// when released.
///
/// Similarly, when pressing, emit modifiers first so that
/// we don't emit C and then CTRL for such a mapping.
pub fn compute_keys_based_on_state(
    mappings: &Vec<Mapping>,
    currently_pressed_keys: &HashSet<KeyCode>,
    output_keys: &HashSet<KeyCode>,
    time: &TimeVal,
) -> Vec<EvKeyEvent> {
    let desired_keys = apply_mapping_to_held_keys(mappings, currently_pressed_keys);
    let mut to_release: Vec<KeyCode> = output_keys.difference(&desired_keys).cloned().collect();
    let mut to_press: Vec<KeyCode> = desired_keys.difference(&output_keys).cloned().collect();

    to_release.sort_by(modifiers_last);
    to_press.sort_by(modifiers_first);

    let release_events = to_release.iter().map(|ev_key| EvKeyEvent {
        time: time.clone(),
        ev_key: ev_key.clone(),
        key_event_type: KeyEventType::Release,
    });
    let press_events = to_press.iter().map(|ev_key| EvKeyEvent {
        time: time.clone(),
        ev_key: ev_key.clone(),
        key_event_type: KeyEventType::Press,
    });

    release_events.chain(press_events).collect()
}

pub fn lookup_mapping<'a>(
    mappings: &'a Vec<Mapping>,
    currently_pressed_keys: &HashSet<KeyCode>,
    code: KeyCode,
) -> Option<&'a Mapping> {
    // Arash note: I changed the original logic to a simple linear search. We prioritize the first match rather than the one with the most matching "input".
    mappings.iter().find(|Mapping::Remap { input, .. }| {
        input.contains(&code) && currently_pressed_keys.is_superset(input)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    mod apply_mapping_to_held_keys {
        use super::*;

        #[test]
        fn apply_mapping_to_held_keys_no_mappings() {
            let input_state = HashSet::from([KeyCode::KEY_A, KeyCode::KEY_B]);
            let mappings = vec![];
            let result = apply_mapping_to_held_keys(&mappings, &input_state);
            assert_eq!(result, input_state);
        }

        #[test]
        fn apply_mapping_to_held_keys_single_remap() {
            let input_state = HashSet::from([KeyCode::KEY_A]);
            let mappings = vec![Mapping::Remap {
                input: HashSet::from([KeyCode::KEY_A]),
                output: HashSet::from([KeyCode::KEY_B]),
            }];
            let result = apply_mapping_to_held_keys(&mappings, &input_state);
            let expected_output = HashSet::from([KeyCode::KEY_B]);
            assert_eq!(result, expected_output);
        }

        #[test]
        fn should_not_apply_remap_when_input_key_not_present() {
            let input_state = HashSet::from([KeyCode::KEY_A, KeyCode::KEY_B]);
            let mappings = vec![Mapping::Remap {
                input: HashSet::from([KeyCode::KEY_C]),
                output: HashSet::from([KeyCode::KEY_D]),
            }];
            let result = apply_mapping_to_held_keys(&mappings, &input_state);
            assert_eq!(result, input_state);
        }

        #[test]
        fn apply_mapping_to_held_keys_multiple_remap_mappings() {
            let input_state = HashSet::from([KeyCode::KEY_A, KeyCode::KEY_B]);
            let mappings = vec![
                Mapping::Remap {
                    input: HashSet::from([KeyCode::KEY_A]),
                    output: HashSet::from([KeyCode::KEY_C]),
                },
                Mapping::Remap {
                    input: HashSet::from([KeyCode::KEY_B]),
                    output: HashSet::from([KeyCode::KEY_D]),
                },
            ];
            let result = apply_mapping_to_held_keys(&mappings, &input_state);
            let expected = HashSet::from([KeyCode::KEY_C, KeyCode::KEY_D]);
            assert_eq!(result, expected);
        }
    }

    mod lookup_mapping {
        use super::*;
        #[test]
        fn should_return_none_when_mappings_list_is_empty() {
            let mappings = vec![];
            let currently_pressed_keys = HashSet::from([KeyCode::KEY_A, KeyCode::KEY_B]);
            let result = lookup_mapping(&mappings, &currently_pressed_keys, KeyCode::KEY_A);
            assert!(result.is_none());
        }

        #[test]
        fn should_return_none_when_code_in_input_but_currently_pressed_keys_not_superset() {
            let mappings = vec![Mapping::Remap {
                input: HashSet::from([KeyCode::KEY_A, KeyCode::KEY_B]),
                output: HashSet::from([KeyCode::KEY_C]),
            }];
            let currently_pressed_keys = HashSet::from([KeyCode::KEY_A]);
            let result = lookup_mapping(&mappings, &currently_pressed_keys, KeyCode::KEY_A);
            assert!(result.is_none());
        }

        #[test]
        fn should_return_correct_mapping_when_code_in_input_and_currently_pressed_keys_is_superset()
        {
            let mappings = vec![Mapping::Remap {
                input: HashSet::from([KeyCode::KEY_A, KeyCode::KEY_B]),
                output: HashSet::from([KeyCode::KEY_C]),
            }];
            let currently_pressed_keys =
                HashSet::from([KeyCode::KEY_A, KeyCode::KEY_B, KeyCode::KEY_D]);
            let result = lookup_mapping(&mappings, &currently_pressed_keys, KeyCode::KEY_A);
            assert!(result.is_some());
            assert_eq!(
                result.unwrap(),
                &Mapping::Remap {
                    input: HashSet::from([KeyCode::KEY_A, KeyCode::KEY_B]),
                    output: HashSet::from([KeyCode::KEY_C]),
                }
            );
        }
    }
}
