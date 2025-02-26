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

pub fn get_key_using_mapping(mappings: &Vec<Mapping>, code: KeyCode) -> KeyCode {
    mappings
        .iter()
        .find(|Mapping::Remap { input, .. }| input == &code)
        .map(|Mapping::Remap { output, .. }| output)
        .unwrap_or(&code)
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    mod get_key_using_mapping {
        use super::*;
        
        #[test]
        fn should_passthrough_when_mappings_list_is_empty() {
            let mappings = vec![];
            let result = get_key_using_mapping(&mappings, KeyCode::KEY_A);
            assert_eq!(result, KeyCode::KEY_A);
        }

        #[test]
        fn should_convert_simple_mapping() {
            let mappings = vec![Mapping::Remap {
                input: KeyCode::KEY_A,
                output: KeyCode::KEY_B,
            }];
            let result = get_key_using_mapping(&mappings, KeyCode::KEY_A);
            assert_eq!(result, KeyCode::KEY_B);
        }
    }
}
