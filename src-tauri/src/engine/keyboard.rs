use std::time::Duration;

use super::worker::{sleep_interruptible, RunControl};

#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, MapVirtualKeyW, SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, MAPVK_VK_TO_VSC_EX,
    VK_CAPITAL, VK_SHIFT,
};

#[inline]
pub fn is_alphabetic_vk(vk: u16) -> bool {
    (b'A' as u16..=b'Z' as u16).contains(&vk)
}

#[cfg(target_os = "windows")]
#[inline]
fn vk_to_scan(vk: u16) -> (u16, bool) {
    let raw = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC_EX) };
    ((raw & 0xFF) as u16, (raw >> 8) != 0)
}

#[cfg(target_os = "windows")]
#[inline]
pub fn make_keyboard_input(vk: u16, flags: u32) -> INPUT {
    let (scan, extended) = vk_to_scan(vk);
    let ext_flag = if extended { KEYEVENTF_EXTENDEDKEY } else { 0 };
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: windows_sys::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: flags | KEYEVENTF_SCANCODE | ext_flag,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

#[cfg(target_os = "windows")]
#[inline]
pub fn send_key_event(vk: u16, flags: u32) {
    let input = make_keyboard_input(vk, flags);
    unsafe { SendInput(1, &input, std::mem::size_of::<INPUT>() as i32) };
}

#[cfg(target_os = "windows")]
fn caps_lock_enabled() -> bool {
    unsafe { (GetKeyState(VK_CAPITAL as i32) & 1) != 0 }
}

#[cfg(target_os = "windows")]
fn should_hold_shift_for_case(vk: u16, uppercase: bool) -> bool {
    is_alphabetic_vk(vk) && (caps_lock_enabled() != uppercase)
}

#[cfg(target_os = "windows")]
fn send_key_down(vk: u16, use_shift: bool) {
    if use_shift {
        send_key_event(VK_SHIFT as u16, 0);
    }
    send_key_event(vk, 0);
}

#[cfg(target_os = "windows")]
fn send_key_up(vk: u16, use_shift: bool) {
    send_key_event(vk, KEYEVENTF_KEYUP);
    if use_shift {
        send_key_event(VK_SHIFT as u16, KEYEVENTF_KEYUP);
    }
}

#[cfg(target_os = "windows")]
pub fn send_key_presses(
    vk: u16,
    _key_token: &str,
    count: usize,
    hold_ms: u32,
    use_double_click_gap: bool,
    double_click_delay_ms: u32,
    uppercase: bool,
    control: &RunControl,
) {
    if count == 0 {
        return;
    }

    let use_shift = should_hold_shift_for_case(vk, uppercase);

    for index in 0..count {
        if !control.is_active() {
            return;
        }

        send_key_down(vk, use_shift);
        if hold_ms > 0 {
            sleep_interruptible(Duration::from_millis(hold_ms as u64), control);
            if !control.is_active() {
                send_key_up(vk, use_shift);
                return;
            }
        }
        send_key_up(vk, use_shift);

        if index + 1 < count && use_double_click_gap && double_click_delay_ms > 0 {
            sleep_interruptible(Duration::from_millis(double_click_delay_ms as u64), control);
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use evdev::{AttributeSet, EventType, InputEvent, Key};
    use std::sync::{Mutex, OnceLock};

    static DEVICE: OnceLock<Option<Mutex<evdev::uinput::VirtualDevice>>> = OnceLock::new();

    pub fn get() -> Option<&'static Mutex<evdev::uinput::VirtualDevice>> {
        DEVICE.get_or_init(|| {
            let builder = match evdev::uinput::VirtualDeviceBuilder::new() {
                Ok(b) => b,
                Err(e) => {
                    log::error!("[uinput-keyboard] Failed to open /dev/uinput: {e}");
                    return None;
                }
            };

            let keys: Vec<Key> = (evdev::Key::KEY_ESC.code()..=evdev::Key::KEY_MICMUTE.code())
                .filter_map(|code| {
                    let key = Key::new(code);
                    // evdev::Key enum only contains real key codes; skip unknown.
                    Some(key)
                })
                .collect();

            let dev = match builder
                .name("blur-autoclicker-keyboard")
                .with_keys(&AttributeSet::from_iter(keys))
                .and_then(|b| b.build())
            {
                Ok(d) => d,
                Err(e) => {
                    log::error!("[uinput-keyboard] Failed to build virtual device: {e}");
                    return None;
                }
            };
            Some(Mutex::new(dev))
        }).as_ref()
    }

    pub fn available() -> bool {
        get().is_some()
    }

    pub fn token_to_evdev_key(token: &str) -> Option<Key> {
        if token.len() == 1 {
            let ch = token.chars().next().unwrap();
            return match ch.to_ascii_lowercase() {
                'a' => Some(Key::KEY_A), 'b' => Some(Key::KEY_B),
                'c' => Some(Key::KEY_C), 'd' => Some(Key::KEY_D),
                'e' => Some(Key::KEY_E), 'f' => Some(Key::KEY_F),
                'g' => Some(Key::KEY_G), 'h' => Some(Key::KEY_H),
                'i' => Some(Key::KEY_I), 'j' => Some(Key::KEY_J),
                'k' => Some(Key::KEY_K), 'l' => Some(Key::KEY_L),
                'm' => Some(Key::KEY_M), 'n' => Some(Key::KEY_N),
                'o' => Some(Key::KEY_O), 'p' => Some(Key::KEY_P),
                'q' => Some(Key::KEY_Q), 'r' => Some(Key::KEY_R),
                's' => Some(Key::KEY_S), 't' => Some(Key::KEY_T),
                'u' => Some(Key::KEY_U), 'v' => Some(Key::KEY_V),
                'w' => Some(Key::KEY_W), 'x' => Some(Key::KEY_X),
                'y' => Some(Key::KEY_Y), 'z' => Some(Key::KEY_Z),
                '0' => Some(Key::KEY_0), '1' => Some(Key::KEY_1),
                '2' => Some(Key::KEY_2), '3' => Some(Key::KEY_3),
                '4' => Some(Key::KEY_4), '5' => Some(Key::KEY_5),
                '6' => Some(Key::KEY_6), '7' => Some(Key::KEY_7),
                '8' => Some(Key::KEY_8), '9' => Some(Key::KEY_9),
                '/' => Some(Key::KEY_SLASH),
                '\\' => Some(Key::KEY_BACKSLASH),
                ';' => Some(Key::KEY_SEMICOLON),
                '\'' => Some(Key::KEY_APOSTROPHE),
                '[' => Some(Key::KEY_LEFTBRACE),
                ']' => Some(Key::KEY_RIGHTBRACE),
                '-' => Some(Key::KEY_MINUS),
                '=' => Some(Key::KEY_EQUAL),
                '`' => Some(Key::KEY_GRAVE),
                ',' => Some(Key::KEY_COMMA),
                '.' => Some(Key::KEY_DOT),
                ' ' => Some(Key::KEY_SPACE),
                _ => None,
            };
        }

        match token {
            "space" => Some(Key::KEY_SPACE),
            "tab" => Some(Key::KEY_TAB),
            "enter" => Some(Key::KEY_ENTER),
            "return" => Some(Key::KEY_ENTER),
            "backspace" => Some(Key::KEY_BACKSPACE),
            "delete" => Some(Key::KEY_DELETE),
            "insert" => Some(Key::KEY_INSERT),
            "home" => Some(Key::KEY_HOME),
            "end" => Some(Key::KEY_END),
            "pageup" => Some(Key::KEY_PAGEUP),
            "pagedown" => Some(Key::KEY_PAGEDOWN),
            "up" => Some(Key::KEY_UP),
            "down" => Some(Key::KEY_DOWN),
            "left" => Some(Key::KEY_LEFT),
            "right" => Some(Key::KEY_RIGHT),
            "escape" => Some(Key::KEY_ESC),
            "esc" => Some(Key::KEY_ESC),
            "numpad0" => Some(Key::KEY_KP0),
            "numpad1" => Some(Key::KEY_KP1),
            "numpad2" => Some(Key::KEY_KP2),
            "numpad3" => Some(Key::KEY_KP3),
            "numpad4" => Some(Key::KEY_KP4),
            "numpad5" => Some(Key::KEY_KP5),
            "numpad6" => Some(Key::KEY_KP6),
            "numpad7" => Some(Key::KEY_KP7),
            "numpad8" => Some(Key::KEY_KP8),
            "numpad9" => Some(Key::KEY_KP9),
            "numpadadd" => Some(Key::KEY_KPPLUS),
            "numpadsubtract" => Some(Key::KEY_KPMINUS),
            "numpadmultiply" => Some(Key::KEY_KPASTERISK),
            "numpaddivide" => Some(Key::KEY_KPSLASH),
            "numpaddecimal" => Some(Key::KEY_KPDOT),
            "numpadenter" => Some(Key::KEY_KPENTER),
            "f1" => Some(Key::KEY_F1),
            "f2" => Some(Key::KEY_F2),
            "f3" => Some(Key::KEY_F3),
            "f4" => Some(Key::KEY_F4),
            "f5" => Some(Key::KEY_F5),
            "f6" => Some(Key::KEY_F6),
            "f7" => Some(Key::KEY_F7),
            "f8" => Some(Key::KEY_F8),
            "f9" => Some(Key::KEY_F9),
            "f10" => Some(Key::KEY_F10),
            "f11" => Some(Key::KEY_F11),
            "f12" => Some(Key::KEY_F12),
            "capslock" => Some(Key::KEY_CAPSLOCK),
            "printscreen" => Some(Key::KEY_SYSRQ),
            "scrolllock" => Some(Key::KEY_SCROLLLOCK),
            "pause" => Some(Key::KEY_PAUSE),
            "numlock" => Some(Key::KEY_NUMLOCK),
            "intlbackslash" => Some(Key::KEY_102ND),
            _ => None,
        }
    }

    pub fn emit_key(key: Key, value: i32) {
        if let Some(dev_lock) = get() {
            if let Ok(mut dev) = dev_lock.lock() {
                let _ = dev.emit(&[
                    InputEvent::new(EventType::KEY, key.code(), value),
                    InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                ]);
            }
        }
    }

    pub fn shift_down() {
        emit_key(Key::KEY_LEFTSHIFT, 1);
    }

    pub fn shift_up() {
        emit_key(Key::KEY_LEFTSHIFT, 0);
    }
}

#[cfg(target_os = "linux")]
pub fn linux_key_available() -> bool {
    linux::available()
}

#[cfg(target_os = "linux")]
fn token_is_alphabetic(token: &str) -> bool {
    token.len() == 1
        && token
            .chars()
            .next()
            .map(|c| c.is_ascii_alphabetic())
            .unwrap_or(false)
}

#[cfg(target_os = "linux")]
pub fn send_key_presses(
    key_token: &str,
    count: usize,
    hold_ms: u32,
    use_double_click_gap: bool,
    double_click_delay_ms: u32,
    uppercase: bool,
    control: &RunControl,
) {
    if count == 0 {
        return;
    }

    let key = match linux::token_to_evdev_key(key_token) {
        Some(k) => k,
        None => {
            log::error!("[keyboard] No evdev mapping for key '{key_token}'");
            return;
        }
    };

    let use_shift = token_is_alphabetic(key_token) && uppercase;

    for index in 0..count {
        if !control.is_active() {
            return;
        }

        if use_shift {
            linux::shift_down();
        }
        linux::emit_key(key, 1);
        if hold_ms > 0 {
            sleep_interruptible(Duration::from_millis(hold_ms as u64), control);
            if !control.is_active() {
                linux::emit_key(key, 0);
                if use_shift {
                    linux::shift_up();
                }
                return;
            }
        }
        linux::emit_key(key, 0);
        if use_shift {
            linux::shift_up();
        }

        if index + 1 < count && use_double_click_gap && double_click_delay_ms > 0 {
            sleep_interruptible(Duration::from_millis(double_click_delay_ms as u64), control);
        }
    }
}

#[inline]
pub fn send_key_presses_cross_platform(
    vk: u16,
    key_token: &str,
    count: usize,
    hold_ms: u32,
    use_double_click_gap: bool,
    double_click_delay_ms: u32,
    uppercase: bool,
    control: &RunControl,
) {
    #[cfg(target_os = "windows")]
    {
        send_key_presses(
            vk,
            key_token,
            count,
            hold_ms,
            use_double_click_gap,
            double_click_delay_ms,
            uppercase,
            control,
        );
    }
    #[cfg(target_os = "linux")]
    {
        send_key_presses(
            key_token,
            count,
            hold_ms,
            use_double_click_gap,
            double_click_delay_ms,
            uppercase,
            control,
        );
    }
}

#[cfg(target_os = "linux")]
pub fn keyboard_diagnostic() -> String {
    if linux::available() {
        String::from("Wayland uinput keyboard backend - available")
    } else {
        String::from("Wayland uinput keyboard backend unavailable")
    }
}
