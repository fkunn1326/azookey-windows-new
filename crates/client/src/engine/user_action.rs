use crate::extension::VKeyExt;
use anyhow::{Context, Result};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyboardState, ToUnicode, VK_SHIFT};

#[derive(Debug)]
pub enum UserAction {
    Input(char),
    Backspace,
    Enter,
    Space,
    Tab,
    Escape,
    Unknown,
    Navigation(Navigation),
    Function(Function),
    Number(i8),
    ToggleInputMode,
}

#[derive(Debug)]
pub enum Navigation {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
enum Function {
    Six,
    Seven,
    Eight,
}

impl TryFrom<usize> for UserAction {
    type Error = anyhow::Error;
    fn try_from(key_code: usize) -> Result<UserAction> {
        let action = match key_code {
            0x08 => UserAction::Backspace, // VK_BACK
            0x09 => UserAction::Tab,       // VK_TAB
            0x0D => UserAction::Enter,     // VK_RETURN
            0x20 => UserAction::Space,     // VK_SPACE
            0x1B => UserAction::Escape,    // VK_ESCAPE

            0x25 => UserAction::Navigation(Navigation::Left), // VK_LEFT
            0x26 => UserAction::Navigation(Navigation::Up),   // VK_UP
            0x27 => UserAction::Navigation(Navigation::Right), // VK_RIGHT
            0x28 => UserAction::Navigation(Navigation::Down), // VK_DOWN

            0x30..=0x39 | 0x60..=0x69 if !VK_SHIFT.is_pressed() => {
                match key_code {
                    0x30 | 0x60 => UserAction::Number(0), // VK_0, VK_NUMPAD0
                    0x31 | 0x61 => UserAction::Number(1), // VK_1, VK_NUMPAD1
                    0x32 | 0x62 => UserAction::Number(2), // VK_2, VK_NUMPAD2
                    0x33 | 0x63 => UserAction::Number(3), // VK_3, VK_NUMPAD3
                    0x34 | 0x64 => UserAction::Number(4), // VK_4, VK_NUMPAD4
                    0x35 | 0x65 => UserAction::Number(5), // VK_5, VK_NUMPAD5
                    0x36 | 0x66 => UserAction::Number(6), // VK_6, VK_NUMPAD6
                    0x37 | 0x67 => UserAction::Number(7), // VK_7, VK_NUMPAD7
                    0x38 | 0x68 => UserAction::Number(8), // VK_8, VK_NUMPAD8
                    0x39 | 0x69 => UserAction::Number(9), // VK_9, VK_NUMPAD9
                    _ => UserAction::Unknown,
                }
            }

            0x75 => UserAction::Function(Function::Six), // VK_F6
            0x76 => UserAction::Function(Function::Seven), // VK_F7
            0x77 => UserAction::Function(Function::Eight), // VK_F8

            0xF3 | 0xF4 => UserAction::ToggleInputMode, // Zenkaku/Hankaku

            _ => {
                let key_state = {
                    let mut key_state = [0u8; 256];
                    unsafe {
                        GetKeyboardState(&mut key_state)?;
                    }
                    key_state
                };
                let unicode = {
                    let mut unicode = [0u16; 1];
                    unsafe { ToUnicode(key_code as u32, 0, Some(&key_state), &mut unicode, 0) };
                    unicode[0]
                };

                if unicode != 0 {
                    UserAction::Input(char::from_u32(unicode as u32).context("Invalid char")?)
                } else {
                    UserAction::Unknown
                }
            }
        };

        Ok(action)
    }
}
