use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use termion::event::Key;

pub struct KeyTranslatorPlugin;

impl Plugin for KeyTranslatorPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<TranslatedKey>()
            .insert_resource(ModifierState::default())
            .add_system(key_translator);
    }
}

#[derive(Clone, Debug)]
pub struct TranslatedKey {
    pub key: Key,
    pub pressed: bool,
}

fn key_translator(
    mut modifier_state: ResMut<ModifierState>,
    mut input_events: EventReader<KeyboardInput>,
    mut output_events: EventWriter<TranslatedKey>,
) {
    for ev in input_events.iter() {
        if let Some(code) = ev.key_code {
            if (code == KeyCode::LShift) || (code == KeyCode::RShift) {
                modifier_state.shift = ev.state.is_pressed();
                continue;
            }
            if (code == KeyCode::LAlt) || (code == KeyCode::RAlt) {
                modifier_state.alt = ev.state.is_pressed();
                continue;
            }
            if (code == KeyCode::LControl) || (code == KeyCode::RControl) {
                modifier_state.control = ev.state.is_pressed();
                continue;
            }
            if let Some(key) = keycode_to_key(&modifier_state, code) {
                output_events.send(TranslatedKey {
                    key,
                    pressed: ev.state.is_pressed(),
                });
            }
        }
    }
}

#[derive(Clone, Resource, Default)]
struct ModifierState {
    shift: bool,
    alt: bool,
    control: bool,
}

fn keycode_to_key(
    modifier_state: &ModifierState,
    keycode: KeyCode,
) -> Option<Key> {
    let ModifierState { shift, alt, control } = modifier_state.clone();

    if control && !alt {
        return Some(Key::Ctrl(keycode_to_letter(keycode, shift)?));
    }
    if !control && alt {
        return Some(Key::Alt(keycode_to_letter(keycode, shift)?));
    }
    if control && alt {
        return None;
    }
    if let Some(character) = keycode_to_letter(keycode, shift) {
        return Some(Key::Char(character));
    }

    match keycode {
        KeyCode::Back         => Some(Key::Backspace),
        KeyCode::Left         => Some(Key::Left),
        KeyCode::Right        => Some(Key::Right),
        KeyCode::Up           => Some(Key::Up),
        KeyCode::Down         => Some(Key::Down),
        KeyCode::Home         => Some(Key::Home),
        KeyCode::End          => Some(Key::End),
        KeyCode::PageUp       => Some(Key::PageUp),
        KeyCode::PageDown     => Some(Key::PageDown),
        KeyCode::Tab if shift => Some(Key::BackTab),
        KeyCode::Delete       => Some(Key::Delete),
        KeyCode::Insert       => Some(Key::Insert),
        KeyCode::Escape       => Some(Key::Esc),
        _                     => None,
    }
}

fn keycode_to_letter(keycode: KeyCode, shift: bool) -> Option<char> {
    if !shift {
        match keycode {
            KeyCode::Key1 => Some('1'),
            KeyCode::Key2 => Some('2'),
            KeyCode::Key3 => Some('3'),
            KeyCode::Key4 => Some('4'),
            KeyCode::Key5 => Some('5'),
            KeyCode::Key6 => Some('6'),
            KeyCode::Key7 => Some('7'),
            KeyCode::Key8 => Some('8'),
            KeyCode::Key9 => Some('9'),
            KeyCode::Key0 => Some('0'),
            KeyCode::A => Some('a'),
            KeyCode::B => Some('b'),
            KeyCode::C => Some('c'),
            KeyCode::D => Some('d'),
            KeyCode::E => Some('e'),
            KeyCode::F => Some('f'),
            KeyCode::G => Some('g'),
            KeyCode::H => Some('h'),
            KeyCode::I => Some('i'),
            KeyCode::J => Some('j'),
            KeyCode::K => Some('k'),
            KeyCode::L => Some('l'),
            KeyCode::M => Some('m'),
            KeyCode::N => Some('n'),
            KeyCode::O => Some('o'),
            KeyCode::P => Some('p'),
            KeyCode::Q => Some('q'),
            KeyCode::R => Some('r'),
            KeyCode::S => Some('s'),
            KeyCode::T => Some('t'),
            KeyCode::U => Some('u'),
            KeyCode::V => Some('v'),
            KeyCode::W => Some('w'),
            KeyCode::X => Some('x'),
            KeyCode::Y => Some('y'),
            KeyCode::Z => Some('z'),
            KeyCode::Space => Some(' '),
            KeyCode::Return => Some('\n'),
            KeyCode::Tab => Some('\t'),
            KeyCode::Comma => Some(','),
            KeyCode::Period => Some('.'),
            KeyCode::Apostrophe => Some('\''),
            KeyCode::Equals => Some('='),
            KeyCode::Minus => Some('-'),
            KeyCode::Slash => Some('/'),
            KeyCode::Backslash => Some('\\'),
            KeyCode::Grave => Some('`'),
            KeyCode::Semicolon => Some(';'),
            KeyCode::Colon => Some(':'),
            KeyCode::LBracket => Some('['),
            KeyCode::RBracket => Some(']'),
            _ => None,
        }
    } else {
        match keycode {
            KeyCode::Key1 => Some('!'),
            KeyCode::Key2 => Some('@'),
            KeyCode::Key3 => Some('#'),
            KeyCode::Key4 => Some('$'),
            KeyCode::Key5 => Some('%'),
            KeyCode::Key6 => Some('^'),
            KeyCode::Key7 => Some('&'),
            KeyCode::Key8 => Some('*'),
            KeyCode::Key9 => Some('('),
            KeyCode::Key0 => Some(')'),
            KeyCode::A => Some('A'),
            KeyCode::B => Some('B'),
            KeyCode::C => Some('C'),
            KeyCode::D => Some('D'),
            KeyCode::E => Some('E'),
            KeyCode::F => Some('F'),
            KeyCode::G => Some('G'),
            KeyCode::H => Some('H'),
            KeyCode::I => Some('I'),
            KeyCode::J => Some('J'),
            KeyCode::K => Some('K'),
            KeyCode::L => Some('L'),
            KeyCode::M => Some('M'),
            KeyCode::N => Some('N'),
            KeyCode::O => Some('O'),
            KeyCode::P => Some('P'),
            KeyCode::Q => Some('Q'),
            KeyCode::R => Some('R'),
            KeyCode::S => Some('S'),
            KeyCode::T => Some('T'),
            KeyCode::U => Some('U'),
            KeyCode::V => Some('V'),
            KeyCode::W => Some('W'),
            KeyCode::X => Some('X'),
            KeyCode::Y => Some('Y'),
            KeyCode::Z => Some('Z'),
            KeyCode::Space => Some(' '),
            KeyCode::Return => Some('\n'),
            KeyCode::Comma => Some('<'),
            KeyCode::Period => Some('>'),
            KeyCode::Apostrophe => Some('"'),
            KeyCode::Equals => Some('+'),
            KeyCode::Minus => Some('_'),
            KeyCode::Slash => Some('?'),
            KeyCode::Backslash => Some('|'),
            KeyCode::Grave => Some('~'),
            KeyCode::Semicolon => Some(':'),
            KeyCode::Colon => Some(':'),
            KeyCode::LBracket => Some('{'),
            KeyCode::RBracket => Some('}'),
            _ => None,
        }
    }
}
