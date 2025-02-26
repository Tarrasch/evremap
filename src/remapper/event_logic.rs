use crate::mapping::*;
use std::collections::HashSet;

pub fn as_modifier(key: &KeyCode) -> Option<Modifier> {
    match key {
        KeyCode::KEY_FN => Some(Modifier::Fn),
        KeyCode::KEY_LEFTALT => Some(Modifier::LeftAlt),
        KeyCode::KEY_RIGHTALT => Some(Modifier::RightAlt),
        KeyCode::KEY_LEFTMETA => Some(Modifier::LeftMeta),
        KeyCode::KEY_RIGHTMETA => Some(Modifier::RightMeta),
        KeyCode::KEY_LEFTCTRL => Some(Modifier::LeftCtrl),
        KeyCode::KEY_RIGHTCTRL => Some(Modifier::RightCtrl),
        KeyCode::KEY_LEFTSHIFT => Some(Modifier::LeftShift),
        KeyCode::KEY_RIGHTSHIFT => Some(Modifier::RightShift),
        _ => None,
    }
}

pub fn is_modifier(key: &KeyCode) -> bool {
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

pub fn get_key_using_mapping<'a>(
    mappings: &'a Vec<Mapping>,
    currently_pressed_modifiers: &HashSet<Modifier>,
    code: KeyCode,
) -> KeyCode {
    // Arash note: I changed the original logic to a simple linear search. We prioritize the first match rather than the one with the most matching "input".
    mappings
        .iter()
        .find(
            |Mapping::Remap {
                 input, modifiers, ..
             }| {
                input == &code && currently_pressed_modifiers.is_superset(modifiers)
            },
        )
        .map(|Mapping::Remap { output, .. }| output)
        .unwrap_or(&code)
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    mod get_key_using_mapping {
        use super::*;
        #[test]
        fn should_passthrough_when_mappings_list_is_empty() {
            let mappings = vec![];
            let currently_pressed_modifiers = HashSet::from([Modifier::LeftAlt]);
            let result =
                get_key_using_mapping(&mappings, &currently_pressed_modifiers, KeyCode::KEY_A);
            assert!(result == KeyCode::KEY_A);
        }

        #[test]
        fn should_convert_when_no_modifier_needed() {
            let mappings = vec![Mapping::Remap {
                input: KeyCode::KEY_A,
                modifiers: HashSet::from([]),
                output: KeyCode::KEY_B,
            }];
            let result = get_key_using_mapping(&mappings, &HashSet::from([]), KeyCode::KEY_A);
            assert!(result == KeyCode::KEY_B);
        }

        #[test]
        fn should_not_convert_when_missing_modifier_needed() {
            let mappings = vec![Mapping::Remap {
                input: KeyCode::KEY_A,
                modifiers: HashSet::from([Modifier::LeftAlt]),
                output: KeyCode::KEY_B,
            }];
            let result = get_key_using_mapping(&mappings, &HashSet::from([]), KeyCode::KEY_A);
            assert!(result == KeyCode::KEY_A);
        }

        #[test]
        fn should_convert_when_enough_modifiers_needed() {
            let mappings = vec![Mapping::Remap {
                input: KeyCode::KEY_A,
                modifiers: HashSet::from([Modifier::LeftAlt]),
                output: KeyCode::KEY_B,
            }];
            let result = get_key_using_mapping(
                &mappings,
                &HashSet::from([Modifier::LeftAlt]),
                KeyCode::KEY_A,
            );
            assert!(result == KeyCode::KEY_B);
        }
    }
}
