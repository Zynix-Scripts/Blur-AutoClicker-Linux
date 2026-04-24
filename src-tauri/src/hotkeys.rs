use crate::engine::worker::now_epoch_ms;
use crate::engine::worker::start_clicker_inner;
use crate::engine::worker::stop_clicker_inner;
use crate::engine::worker::toggle_clicker_inner;
use crate::AppHandle;
use crate::ClickerState;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use tauri::Manager;

#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, KBDLLHOOKSTRUCT, LLKHF_EXTENDED, MSG,
    WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP, WM_MOUSEWHEEL, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

#[cfg(target_os = "linux")]
#[allow(dead_code)]
mod vk_compat {
    pub const VK_LBUTTON: i32 = 0x01;
    pub const VK_RBUTTON: i32 = 0x02;
    pub const VK_MBUTTON: i32 = 0x04;
    pub const VK_XBUTTON1: i32 = 0x05;
    pub const VK_XBUTTON2: i32 = 0x06;
    pub const VK_BACK: i32 = 0x08;
    pub const VK_TAB: i32 = 0x09;
    pub const VK_RETURN: i32 = 0x0D;
    pub const VK_ESCAPE: i32 = 0x1B;
    pub const VK_SPACE: i32 = 0x20;
    pub const VK_PRIOR: i32 = 0x21;
    pub const VK_NEXT: i32 = 0x22;
    pub const VK_END: i32 = 0x23;
    pub const VK_HOME: i32 = 0x24;
    pub const VK_LEFT: i32 = 0x25;
    pub const VK_UP: i32 = 0x26;
    pub const VK_RIGHT: i32 = 0x27;
    pub const VK_DOWN: i32 = 0x28;
    pub const VK_INSERT: i32 = 0x2D;
    pub const VK_DELETE: i32 = 0x2E;
    pub const VK_LWIN: i32 = 0x5B;
    pub const VK_RWIN: i32 = 0x5C;
    pub const VK_NUMPAD0: i32 = 0x60;
    pub const VK_NUMPAD1: i32 = 0x61;
    pub const VK_NUMPAD2: i32 = 0x62;
    pub const VK_NUMPAD3: i32 = 0x63;
    pub const VK_NUMPAD4: i32 = 0x64;
    pub const VK_NUMPAD5: i32 = 0x65;
    pub const VK_NUMPAD6: i32 = 0x66;
    pub const VK_NUMPAD7: i32 = 0x67;
    pub const VK_NUMPAD8: i32 = 0x68;
    pub const VK_NUMPAD9: i32 = 0x69;
    pub const VK_MULTIPLY: i32 = 0x6A;
    pub const VK_ADD: i32 = 0x6B;
    pub const VK_SUBTRACT: i32 = 0x6D;
    pub const VK_DECIMAL: i32 = 0x6E;
    pub const VK_DIVIDE: i32 = 0x6F;
    pub const VK_F1: i32 = 0x70;
    pub const VK_OEM_1: i32 = 0xBA;
    pub const VK_OEM_PLUS: i32 = 0xBB;
    pub const VK_OEM_COMMA: i32 = 0xBC;
    pub const VK_OEM_MINUS: i32 = 0xBD;
    pub const VK_OEM_PERIOD: i32 = 0xBE;
    pub const VK_OEM_2: i32 = 0xBF;
    pub const VK_OEM_3: i32 = 0xC0;
    pub const VK_OEM_4: i32 = 0xDB;
    pub const VK_OEM_5: i32 = 0xDC;
    pub const VK_OEM_6: i32 = 0xDD;
    pub const VK_OEM_7: i32 = 0xDE;
    pub const VK_OEM_102: i32 = 0xE2;
}

#[cfg(target_os = "linux")]
use vk_compat::*;

pub const VK_SCROLL_UP_PSEUDO: i32 = -1;
pub const VK_SCROLL_DOWN_PSEUDO: i32 = -2;
pub const VK_NUMPAD_ENTER_PSEUDO: i32 = -3;

static SCROLL_UP_AT: AtomicU64 = AtomicU64::new(0);
static SCROLL_DOWN_AT: AtomicU64 = AtomicU64::new(0);
#[allow(dead_code)]
static NUMPAD_ENTER_DOWN: AtomicBool = AtomicBool::new(false);

const SCROLL_WINDOW_MS: u64 = 200;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HotkeyBinding {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_key: bool,
    pub main_vk: i32,
    pub key_token: String,
}

pub fn register_hotkey_inner(app: &AppHandle, hotkey: String) -> Result<String, String> {
    let binding = parse_hotkey_binding(&hotkey)?;
    let state = app.state::<ClickerState>();
    state
        .suppress_hotkey_until_ms
        .store(now_epoch_ms().saturating_add(250), Ordering::SeqCst);
    state
        .suppress_hotkey_until_release
        .store(true, Ordering::SeqCst);
    *state.registered_hotkey.lock().unwrap() = Some(binding.clone());
    Ok(format_hotkey_binding(&binding))
}

pub fn normalize_hotkey(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .replace("control", "ctrl")
        .replace("command", "super")
        .replace("meta", "super")
        .replace("win", "super")
}

pub fn parse_hotkey_binding(hotkey: &str) -> Result<HotkeyBinding, String> {
    let normalized = normalize_hotkey(hotkey);
    let mut ctrl = false;
    let mut alt = false;
    let mut shift = false;
    let mut super_key = false;
    let mut main_key: Option<(i32, String)> = None;

    for token in normalized.split('+').map(str::trim) {
        if token.is_empty() {
            return Err(format!("Invalid hotkey '{hotkey}': found empty key token"));
        }

        match token {
            "alt" | "option" => alt = true,
            "ctrl" | "control" => ctrl = true,
            "shift" => shift = true,
            "super" | "command" | "cmd" | "meta" | "win" => super_key = true,
            _ => {
                if main_key
                    .replace(parse_hotkey_main_key(token, hotkey)?)
                    .is_some()
                {
                    return Err(format!(
                        "Invalid hotkey '{hotkey}': use modifiers first and only one main key"
                    ));
                }
            }
        }
    }

    let (main_vk, key_token) =
        main_key.ok_or_else(|| format!("Invalid hotkey '{hotkey}': missing main key"))?;

    Ok(HotkeyBinding { ctrl, alt, shift, super_key, main_vk, key_token })
}

pub fn parse_hotkey_main_key(token: &str, original_hotkey: &str) -> Result<(i32, String), String> {
    let lower = token.trim().to_lowercase();

    let mapped = match lower.as_str() {
        "mouseleft" | "mouse1" => Some((VK_LBUTTON as i32, String::from("mouseleft"))),
        "mouseright" | "mouse2" => Some((VK_RBUTTON as i32, String::from("mouseright"))),
        "mousemiddle" | "mouse3" | "scrollbutton" | "middleclick" => {
            Some((VK_MBUTTON as i32, String::from("mousemiddle")))
        }
        "mouse4" | "mouseback" | "xbutton1" => Some((VK_XBUTTON1 as i32, String::from("mouse4"))),
        "mouse5" | "mouseforward" | "xbutton2" => {
            Some((VK_XBUTTON2 as i32, String::from("mouse5")))
        }
        "scrollup" | "wheelup" => Some((VK_SCROLL_UP_PSEUDO, String::from("scrollup"))),
        "scrolldown" | "wheeldown" => Some((VK_SCROLL_DOWN_PSEUDO, String::from("scrolldown"))),
        "numpad0" => Some((VK_NUMPAD0 as i32, String::from("numpad0"))),
        "numpad1" => Some((VK_NUMPAD1 as i32, String::from("numpad1"))),
        "numpad2" => Some((VK_NUMPAD2 as i32, String::from("numpad2"))),
        "numpad3" => Some((VK_NUMPAD3 as i32, String::from("numpad3"))),
        "numpad4" => Some((VK_NUMPAD4 as i32, String::from("numpad4"))),
        "numpad5" => Some((VK_NUMPAD5 as i32, String::from("numpad5"))),
        "numpad6" => Some((VK_NUMPAD6 as i32, String::from("numpad6"))),
        "numpad7" => Some((VK_NUMPAD7 as i32, String::from("numpad7"))),
        "numpad8" => Some((VK_NUMPAD8 as i32, String::from("numpad8"))),
        "numpad9" => Some((VK_NUMPAD9 as i32, String::from("numpad9"))),
        "numpadadd" => Some((VK_ADD as i32, String::from("numpadadd"))),
        "numpadsubtract" => Some((VK_SUBTRACT as i32, String::from("numpadsubtract"))),
        "numpadmultiply" => Some((VK_MULTIPLY as i32, String::from("numpadmultiply"))),
        "numpaddivide" => Some((VK_DIVIDE as i32, String::from("numpaddivide"))),
        "numpaddecimal" => Some((VK_DECIMAL as i32, String::from("numpaddecimal"))),
        "numpadenter" => Some((VK_NUMPAD_ENTER_PSEUDO, String::from("numpadenter"))),
        "<" | ">" | "intlbackslash" | "oem102" | "nonusbackslash" => {
            Some((VK_OEM_102 as i32, String::from("IntlBackslash")))
        }
        "space" | "spacebar" => Some((VK_SPACE as i32, String::from("space"))),
        "tab" => Some((VK_TAB as i32, String::from("tab"))),
        "enter" => Some((VK_RETURN as i32, String::from("enter"))),
        "backspace" => Some((VK_BACK as i32, String::from("backspace"))),
        "delete" => Some((VK_DELETE as i32, String::from("delete"))),
        "insert" => Some((VK_INSERT as i32, String::from("insert"))),
        "home" => Some((VK_HOME as i32, String::from("home"))),
        "end" => Some((VK_END as i32, String::from("end"))),
        "pageup" => Some((VK_PRIOR as i32, String::from("pageup"))),
        "pagedown" => Some((VK_NEXT as i32, String::from("pagedown"))),
        "up" => Some((VK_UP as i32, String::from("up"))),
        "down" => Some((VK_DOWN as i32, String::from("down"))),
        "left" => Some((VK_LEFT as i32, String::from("left"))),
        "right" => Some((VK_RIGHT as i32, String::from("right"))),
        "esc" | "escape" => Some((VK_ESCAPE as i32, String::from("escape"))),
        "/" | "slash" => Some((VK_OEM_2 as i32, String::from("/"))),
        "\\" | "backslash" => Some((VK_OEM_5 as i32, String::from("\\"))),
        ";" | "semicolon" => Some((VK_OEM_1 as i32, String::from(";"))),
        "'" | "quote" => Some((VK_OEM_7 as i32, String::from("'"))),
        "[" | "bracketleft" => Some((VK_OEM_4 as i32, String::from("["))),
        "]" | "bracketright" => Some((VK_OEM_6 as i32, String::from("]"))),
        "-" | "minus" => Some((VK_OEM_MINUS as i32, String::from("-"))),
        "=" | "equal" => Some((VK_OEM_PLUS as i32, String::from("="))),
        "`" | "backquote" => Some((VK_OEM_3 as i32, String::from("`"))),
        "," | "comma" => Some((VK_OEM_COMMA as i32, String::from(","))),
        "." | "period" => Some((VK_OEM_PERIOD as i32, String::from("."))),
        _ => None,
    };

    if let Some(binding) = mapped {
        return Ok(binding);
    }

    if lower.starts_with('f') && lower.len() <= 3 {
        if let Ok(number) = lower[1..].parse::<i32>() {
            let vk = match number {
                1..=24 => VK_F1 as i32 + (number - 1),
                _ => -1,
            };
            if vk >= 0 {
                return Ok((vk, lower));
            }
        }
    }

    if let Some(letter) = lower.strip_prefix("key") {
        if letter.len() == 1 {
            return parse_hotkey_main_key(letter, original_hotkey);
        }
    }

    if let Some(digit) = lower.strip_prefix("digit") {
        if digit.len() == 1 {
            return parse_hotkey_main_key(digit, original_hotkey);
        }
    }

    if lower.len() == 1 {
        let ch = lower.as_bytes()[0];
        if ch.is_ascii_lowercase() {
            return Ok((ch.to_ascii_uppercase() as i32, lower));
        }
        if ch.is_ascii_digit() {
            return Ok((ch as i32, lower));
        }
    }

    Err(format!(
        "Couldn't recognize '{token}' as a valid key in '{original_hotkey}'"
    ))
}

pub fn format_hotkey_binding(binding: &HotkeyBinding) -> String {
    let mut parts: Vec<String> = Vec::new();
    if binding.ctrl { parts.push(String::from("ctrl")); }
    if binding.alt { parts.push(String::from("alt")); }
    if binding.shift { parts.push(String::from("shift")); }
    if binding.super_key { parts.push(String::from("super")); }
    parts.push(binding.key_token.clone());
    parts.join("+")
}

pub fn start_hotkey_listener(app: AppHandle) {
    std::thread::spawn(move || {
        let mut was_pressed = false;

        loop {
            let (binding, strict) = {
                let state = app.state::<ClickerState>();
                let binding = state.registered_hotkey.lock().unwrap().clone();
                let strict = state.settings.lock().unwrap().strict_hotkey_modifiers;
                (binding, strict)
            };

            let currently_pressed = binding
                .as_ref()
                .map(|b| is_hotkey_binding_pressed(b, strict))
                .unwrap_or(false);

            let suppress_until = app
                .state::<ClickerState>()
                .suppress_hotkey_until_ms
                .load(Ordering::SeqCst);
            let suppress_until_release = app
                .state::<ClickerState>()
                .suppress_hotkey_until_release
                .load(Ordering::SeqCst);
            let hotkey_capture_active = app
                .state::<ClickerState>()
                .hotkey_capture_active
                .load(Ordering::SeqCst);

            if hotkey_capture_active {
                was_pressed = currently_pressed;
                std::thread::sleep(Duration::from_millis(12));
                continue;
            }

            if suppress_until_release {
                if currently_pressed {
                    was_pressed = true;
                    std::thread::sleep(Duration::from_millis(12));
                    continue;
                }
                app.state::<ClickerState>()
                    .suppress_hotkey_until_release
                    .store(false, Ordering::SeqCst);
                was_pressed = false;
                std::thread::sleep(Duration::from_millis(12));
                continue;
            }

            if now_epoch_ms() < suppress_until {
                was_pressed = currently_pressed;
                std::thread::sleep(Duration::from_millis(12));
                continue;
            }

            if currently_pressed && !was_pressed {
                handle_hotkey_pressed(&app);
            } else if !currently_pressed && was_pressed {
                handle_hotkey_released(&app);
            }

            was_pressed = currently_pressed;
            std::thread::sleep(Duration::from_millis(12));
        }
    });
}

pub fn handle_hotkey_pressed(app: &AppHandle) {
    let mode = {
        let state = app.state::<ClickerState>();
        let x = state.settings.lock().unwrap().mode.clone();
        x
    };
    if mode == "Toggle" {
        let _ = toggle_clicker_inner(app);
    } else if mode == "Hold" {
        let _ = start_clicker_inner(app);
    }
}

pub fn handle_hotkey_released(app: &AppHandle) {
    let mode = {
        let state = app.state::<ClickerState>();
        let x = state.settings.lock().unwrap().mode.clone();
        x
    };
    if mode == "Hold" {
        let _ = stop_clicker_inner(app, Some(String::from("Stopped from hold hotkey")));
    }
}

pub fn is_hotkey_binding_pressed(binding: &HotkeyBinding, strict: bool) -> bool {
    #[cfg(target_os = "windows")]
    {
        let ctrl_down = is_vk_down(VK_CONTROL as i32);
        let alt_down = is_vk_down(VK_MENU as i32);
        let shift_down = is_vk_down(VK_SHIFT as i32);
        let super_down = is_vk_down(VK_LWIN as i32) || is_vk_down(VK_RWIN as i32);
        if !modifiers_match(binding, ctrl_down, alt_down, shift_down, super_down, strict) {
            return false;
        }
        is_main_key_active_windows(binding.main_vk)
    }
    #[cfg(target_os = "linux")]
    {
        linux_hotkeys::is_binding_pressed(binding, strict)
    }
}

#[allow(dead_code)]
fn modifiers_match(
    binding: &HotkeyBinding,
    ctrl_down: bool,
    alt_down: bool,
    shift_down: bool,
    super_down: bool,
    strict: bool,
) -> bool {
    if binding.ctrl && !ctrl_down { return false; }
    if binding.alt && !alt_down { return false; }
    if binding.shift && !shift_down { return false; }
    if binding.super_key && !super_down { return false; }

    if strict {
        if ctrl_down && !binding.ctrl { return false; }
        if alt_down && !binding.alt { return false; }
        if shift_down && !binding.shift { return false; }
        if super_down && !binding.super_key { return false; }
    }

    true
}

// ─── Windows hotkey implementation ───────────────────────────────────────────

#[cfg(target_os = "windows")]
fn is_main_key_active_windows(vk: i32) -> bool {
    match vk {
        VK_SCROLL_UP_PSEUDO => {
            let ts = SCROLL_UP_AT.load(Ordering::SeqCst);
            if ts == 0 { return false; }
            now_epoch_ms().saturating_sub(ts) < SCROLL_WINDOW_MS
        }
        VK_SCROLL_DOWN_PSEUDO => {
            let ts = SCROLL_DOWN_AT.load(Ordering::SeqCst);
            if ts == 0 { return false; }
            now_epoch_ms().saturating_sub(ts) < SCROLL_WINDOW_MS
        }
        VK_NUMPAD_ENTER_PSEUDO => NUMPAD_ENTER_DOWN.load(Ordering::SeqCst),
        _ => is_vk_down(vk),
    }
}

#[cfg(target_os = "windows")]
pub fn is_vk_down(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 }
}

#[cfg(target_os = "windows")]
pub fn start_scroll_hook() {
    std::thread::spawn(|| unsafe {
        let mouse_hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), 0, 0);
        if mouse_hook == 0 {
            log::error!("[Hotkeys] Failed to install WH_MOUSE_LL hook");
        }

        let keyboard_hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), 0, 0);
        if keyboard_hook == 0 {
            log::error!("[Hotkeys] Failed to install WH_KEYBOARD_LL hook");
        }

        if mouse_hook == 0 && keyboard_hook == 0 {
            return;
        }

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {}
    });
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn keyboard_hook_proc(code: i32, w_param: usize, l_param: isize) -> isize {
    if code >= 0 {
        let info = &*(l_param as *const KBDLLHOOKSTRUCT);
        if info.vkCode as i32 == VK_RETURN as i32 && (info.flags & LLKHF_EXTENDED) != 0 {
            match w_param as u32 {
                WM_KEYDOWN | WM_SYSKEYDOWN => {
                    NUMPAD_ENTER_DOWN.store(true, Ordering::SeqCst);
                }
                WM_KEYUP | WM_SYSKEYUP => {
                    NUMPAD_ENTER_DOWN.store(false, Ordering::SeqCst);
                }
                _ => {}
            }
        }
    }
    CallNextHookEx(0, code, w_param, l_param)
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn mouse_hook_proc(code: i32, w_param: usize, l_param: isize) -> isize {
    if code >= 0 && w_param == WM_MOUSEWHEEL as usize {
        #[repr(C)]
        struct MsllHookStruct {
            pt_x: i32,
            pt_y: i32,
            mouse_data: u32,
            flags: u32,
            time: u32,
            extra_info: usize,
        }
        let info = &*(l_param as *const MsllHookStruct);
        let delta = (info.mouse_data >> 16) as i16;
        let now = now_epoch_ms();
        if delta > 0 {
            SCROLL_UP_AT.store(now, Ordering::SeqCst);
        } else if delta < 0 {
            SCROLL_DOWN_AT.store(now, Ordering::SeqCst);
        }
    }
    CallNextHookEx(0, code, w_param, l_param)
}

// ─── Linux hotkey implementation ──────────────────────────────────────────────

#[cfg(target_os = "linux")]
pub fn start_scroll_hook() {
    linux_hotkeys::start_evdev_scroll_thread();
}

#[cfg(target_os = "linux")]
mod linux_hotkeys {
    use super::{HotkeyBinding, SCROLL_DOWN_AT, SCROLL_UP_AT, SCROLL_WINDOW_MS};
    use crate::engine::worker::now_epoch_ms;
    use evdev::{Key, RelativeAxisType};
    use std::sync::{OnceLock, RwLock};
    use std::time::Duration;

    // ── Shared pressed-keys state (evdev background thread) ──────────────

    static PRESSED_KEYS: OnceLock<RwLock<std::collections::HashSet<Key>>> = OnceLock::new();

    fn pressed_keys() -> &'static RwLock<std::collections::HashSet<Key>> {
        PRESSED_KEYS.get_or_init(|| RwLock::new(std::collections::HashSet::new()))
    }

    pub fn start_evdev_scroll_thread() {
        std::thread::spawn(|| {
            let set_nonblock = |dev: &evdev::Device| {
                use std::os::unix::io::AsRawFd;
                unsafe {
                    libc::fcntl(dev.as_raw_fd(), libc::F_SETFL, libc::O_NONBLOCK);
                }
            };

            let mut kbd_devs: Vec<evdev::Device> = evdev::enumerate()
                .filter_map(|(path, _)| {
                    let dev = evdev::Device::open(&path).ok()?;
                    let has_keys = dev
                        .supported_keys()
                        .map(|k| k.contains(Key::KEY_A))
                        .unwrap_or(false);
                    if has_keys {
                        set_nonblock(&dev);
                        Some(dev)
                    } else {
                        None
                    }
                })
                .collect();

            let mut mouse_devs: Vec<evdev::Device> = evdev::enumerate()
                .filter_map(|(path, _)| {
                    let dev = evdev::Device::open(&path).ok()?;
                    let has_wheel = dev
                        .supported_relative_axes()
                        .map(|a| a.contains(RelativeAxisType::REL_WHEEL))
                        .unwrap_or(false);
                    if has_wheel {
                        set_nonblock(&dev);
                        Some(dev)
                    } else {
                        None
                    }
                })
                .collect();

            loop {
                for dev in &mut kbd_devs {
                    match dev.fetch_events() {
                        Ok(events) => {
                            if let Ok(mut keys) = pressed_keys().write() {
                                for ev in events {
                                    match ev.kind() {
                                        evdev::InputEventKind::Key(key) => {
                                            match ev.value() {
                                                1 | 2 => { keys.insert(key); }
                                                0 => { keys.remove(&key); }
                                                _ => {}
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                        Err(_) => {}
                    }
                }

                for dev in &mut mouse_devs {
                    match dev.fetch_events() {
                        Ok(events) => {
                            for ev in events {
                                match ev.kind() {
                                    evdev::InputEventKind::RelAxis(rel)
                                        if rel == RelativeAxisType::REL_WHEEL =>
                                    {
                                        let now = now_epoch_ms();
                                        if ev.value() > 0 {
                                            SCROLL_UP_AT.store(now, std::sync::atomic::Ordering::SeqCst);
                                        } else if ev.value() < 0 {
                                            SCROLL_DOWN_AT.store(now, std::sync::atomic::Ordering::SeqCst);
                                        }
                                    }
                                    evdev::InputEventKind::Key(key) => {
                                        if let Ok(mut keys) = pressed_keys().write() {
                                            match ev.value() {
                                                1 | 2 => { keys.insert(key); }
                                                0 => { keys.remove(&key); }
                                                _ => {}
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                        Err(_) => {}
                    }
                }

                std::thread::sleep(Duration::from_millis(4));
            }
        });
    }

    fn token_to_evdev_key(token: &str) -> Option<Key> {
        if token.len() == 1 {
            let ch = token.chars().next().unwrap();
            return match ch {
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
                _ => None,
            };
        }

        match token {
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
            "f13" => Some(Key::KEY_F13),
            "f14" => Some(Key::KEY_F14),
            "f15" => Some(Key::KEY_F15),
            "f16" => Some(Key::KEY_F16),
            "f17" => Some(Key::KEY_F17),
            "f18" => Some(Key::KEY_F18),
            "f19" => Some(Key::KEY_F19),
            "f20" => Some(Key::KEY_F20),
            "f21" => Some(Key::KEY_F21),
            "f22" => Some(Key::KEY_F22),
            "f23" => Some(Key::KEY_F23),
            "f24" => Some(Key::KEY_F24),
            "space" => Some(Key::KEY_SPACE),
            "tab" => Some(Key::KEY_TAB),
            "enter" => Some(Key::KEY_ENTER),
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
            "intlbackslash" => Some(Key::KEY_102ND),
            _ => None,
        }
    }

    fn is_evdev_key_pressed(key: Key) -> bool {
        pressed_keys()
            .read()
            .map(|keys| keys.contains(&key))
            .unwrap_or(false)
    }

    fn modifiers_ok(binding: &HotkeyBinding, strict: bool) -> bool {
        let keys = match pressed_keys().read() {
            Ok(k) => k,
            Err(_) => return false,
        };
        let ctrl_down = keys.contains(&Key::KEY_LEFTCTRL) || keys.contains(&Key::KEY_RIGHTCTRL);
        let alt_down = keys.contains(&Key::KEY_LEFTALT) || keys.contains(&Key::KEY_RIGHTALT);
        let shift_down = keys.contains(&Key::KEY_LEFTSHIFT) || keys.contains(&Key::KEY_RIGHTSHIFT);
        let super_down = keys.contains(&Key::KEY_LEFTMETA) || keys.contains(&Key::KEY_RIGHTMETA);

        if binding.ctrl && !ctrl_down { return false; }
        if binding.alt && !alt_down { return false; }
        if binding.shift && !shift_down { return false; }
        if binding.super_key && !super_down { return false; }

        if strict {
            if ctrl_down && !binding.ctrl { return false; }
            if alt_down && !binding.alt { return false; }
            if shift_down && !binding.shift { return false; }
            if super_down && !binding.super_key { return false; }
        }

        true
    }

    pub fn is_binding_pressed(binding: &HotkeyBinding, strict: bool) -> bool {
        if !modifiers_ok(binding, strict) {
            return false;
        }

        let token = binding.key_token.as_str();

        match binding.main_vk {
            super::VK_SCROLL_UP_PSEUDO => {
                let ts = SCROLL_UP_AT.load(std::sync::atomic::Ordering::SeqCst);
                return ts != 0 && now_epoch_ms().saturating_sub(ts) < SCROLL_WINDOW_MS;
            }
            super::VK_SCROLL_DOWN_PSEUDO => {
                let ts = SCROLL_DOWN_AT.load(std::sync::atomic::Ordering::SeqCst);
                return ts != 0 && now_epoch_ms().saturating_sub(ts) < SCROLL_WINDOW_MS;
            }
            super::VK_NUMPAD_ENTER_PSEUDO => {
                return is_evdev_key_pressed(Key::KEY_KPENTER);
            }
            _ => {}
        }

        // Mouse button hotkeys
        match token {
            "mouseleft" => return is_evdev_key_pressed(Key::BTN_LEFT),
            "mouseright" => return is_evdev_key_pressed(Key::BTN_RIGHT),
            "mousemiddle" => return is_evdev_key_pressed(Key::BTN_MIDDLE),
            "mouse4" => return is_evdev_key_pressed(Key::BTN_SIDE),
            "mouse5" => return is_evdev_key_pressed(Key::BTN_EXTRA),
            _ => {}
        }

        if let Some(key) = token_to_evdev_key(token) {
            return is_evdev_key_pressed(key);
        }

        false
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{format_hotkey_binding, modifiers_match, parse_hotkey_binding};

    #[test]
    fn numpad_tokens_round_trip() {
        for token in [
            "numpad0", "numpad1", "numpad2", "numpad3", "numpad4",
            "numpad5", "numpad6", "numpad7", "numpad8", "numpad9",
            "numpadadd", "numpadsubtract", "numpadmultiply",
            "numpaddivide", "numpaddecimal", "numpadenter",
        ] {
            let hotkey = format!("ctrl+shift+{token}");
            let binding = parse_hotkey_binding(&hotkey).expect("token should parse");
            assert_eq!(binding.key_token, token);
            assert_eq!(format_hotkey_binding(&binding), hotkey);
        }
    }

    #[test]
    fn empty_hotkeys_are_rejected() {
        assert!(parse_hotkey_binding("").is_err());
        assert!(parse_hotkey_binding("ctrl+").is_err());
    }

    #[test]
    fn extra_modifiers_do_not_block_hotkeys_in_relaxed_mode() {
        let binding = parse_hotkey_binding("f11").expect("hotkey should parse");
        assert!(modifiers_match(&binding, false, false, true, false, false));
        assert!(modifiers_match(&binding, true, true, true, true, false));
    }

    #[test]
    fn extra_modifiers_block_hotkeys_in_strict_mode() {
        let binding = parse_hotkey_binding("f11").expect("hotkey should parse");
        assert!(!modifiers_match(&binding, false, false, true, false, true));
        assert!(!modifiers_match(&binding, true, true, true, true, true));
        assert!(modifiers_match(&binding, false, false, false, false, true));
    }
}
