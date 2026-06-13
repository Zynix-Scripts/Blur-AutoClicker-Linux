use super::cycle::{execute_click_cycle, ClickCycleKind, ClickCyclePlan};
use super::worker::{sleep_interruptible, RunControl};
use std::time::Duration;
use std::time::Instant;

use super::rng::SmallRng;
use super::worker::{sleep_interruptible, RunControl};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VirtualScreenRect {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

impl VirtualScreenRect {
    #[inline]
    pub fn new(left: i32, top: i32, width: i32, height: i32) -> Self {
        Self { left, top, width, height }
    }

    #[inline]
    pub fn right(self) -> i32 { self.left + self.width }

    #[inline]
    pub fn bottom(self) -> i32 { self.top + self.height }

    #[inline]
    pub fn contains(self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.right() && y >= self.top && y < self.bottom()
    }

    fn normalize_x(&self, pixel_x: i32) -> i32 {
        let relative_x = pixel_x as f64 - self.left as f64;
        let ratio = relative_x / self.width as f64;
        (ratio * 65535.0).round() as i32
    }
    fn normalize_y(&self, pixel_y: i32) -> i32 {
        let relative_y = pixel_y as f64 - self.top as f64;
        let ratio = relative_y / self.height as f64;
        (ratio * 65535.0).round() as i32
    }

    #[inline]
    pub fn offset_from(self, origin: VirtualScreenRect) -> Self {
        Self::new(self.left - origin.left, self.top - origin.top, self.width, self.height)
    }
}

use std::sync::Mutex;

static CACHED_MONITOR_RECTS: Mutex<Option<Vec<VirtualScreenRect>>> = Mutex::new(None);
static CACHED_VIRTUAL_SCREEN_RECT: Mutex<Option<VirtualScreenRect>> = Mutex::new(None);

pub fn set_cached_monitor_rects(rects: Vec<VirtualScreenRect>) {
    let mut guard = CACHED_MONITOR_RECTS.lock().unwrap();
    *guard = Some(rects);
}

pub fn set_cached_virtual_screen_rect(rect: VirtualScreenRect) {
    let mut guard = CACHED_VIRTUAL_SCREEN_RECT.lock().unwrap();
    *guard = Some(rect);
}


#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
    MOUSEINPUT,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SetCursorPos, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
    SM_YVIRTUALSCREEN,
};

#[cfg(target_os = "windows")]
pub fn current_cursor_position() -> Option<(i32, i32)> {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;
    let mut point = POINT { x: 0, y: 0 };
    let ok = unsafe { GetCursorPos(&mut point) };
    if ok == 0 { None } else { Some((point.x, point.y)) }
}

#[cfg(target_os = "windows")]
pub fn current_virtual_screen_rect() -> Option<VirtualScreenRect> {
    let left = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
    let top = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
    let width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
    let height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };
    if width <= 0 || height <= 0 { return None; }
    Some(VirtualScreenRect::new(left, top, width, height))
}

#[cfg(target_os = "windows")]
pub fn current_monitor_rects() -> Option<Vec<VirtualScreenRect>> {
    use std::ptr;
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, MONITORINFO};

    unsafe extern "system" fn enum_monitor_proc(
        monitor: isize,
        _hdc: isize,
        _clip_rect: *mut RECT,
        user_data: isize,
    ) -> i32 {
        let monitors = &mut *(user_data as *mut Vec<VirtualScreenRect>);
        let mut info = std::mem::zeroed::<MONITORINFO>();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(monitor, &mut info as *mut MONITORINFO as *mut _) == 0 {
            return 1;
        }
        let rect = info.rcMonitor;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if width > 0 && height > 0 {
            monitors.push(VirtualScreenRect::new(rect.left, rect.top, width, height));
        }
        1
    }

    let mut monitors = Vec::new();
    let ok = unsafe {
        EnumDisplayMonitors(
            0,
            ptr::null(),
            Some(enum_monitor_proc),
            &mut monitors as *mut Vec<VirtualScreenRect> as isize,
        )
    };

    if ok == 0 || monitors.is_empty() {
        return current_virtual_screen_rect().map(|screen| vec![screen]);
    }

    monitors.sort_by_key(|m: &VirtualScreenRect| (m.top, m.left));
    Some(monitors)
}

#[cfg(target_os = "windows")]
#[inline]
pub fn move_mouse(target_x: i32, target_y: i32) {
    if let Some(screen_rect) = current_virtual_screen_rect() {
        let end_x = screen_rect.normalize_x(target_x);
        let end_y = screen_rect.normalize_y(target_y);

        let movement = make_movement(end_x, end_y);
        unsafe { SendInput(1, &movement, std::mem::size_of::<INPUT>() as i32) };
        log::debug!("moved cursor x:{end_x}, y:{end_y}")
    }
}

#[inline]
pub fn make_movement(end_x: i32, end_y: i32) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: windows_sys::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
            mi: MOUSEINPUT {
                dx: end_x,
                dy: end_y,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE | MOUSEEVENTF_VIRTUALDESK,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

#[cfg(target_os = "windows")]
#[inline]
pub fn make_input(flags: u32, time: u32) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: windows_sys::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: flags,
                time,
                dwExtraInfo: 0,
            },
        },
    }
}

#[cfg(target_os = "windows")]
#[inline]
pub fn send_mouse_event(flags: u32) {
    let input = make_input(flags, 0);
    unsafe { SendInput(1, &input, std::mem::size_of::<INPUT>() as i32) };
}

#[cfg(target_os = "windows")]
pub fn send_batch(down: u32, up: u32, n: usize, _hold_ms: u32) {
    let mut inputs: Vec<INPUT> = Vec::with_capacity(n * 2);
    for _ in 0..n {
        inputs.push(make_input(down, 0));
        inputs.push(make_input(up, 0));
    }
    unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        )
    };
}

#[cfg(target_os = "windows")]
#[inline]
pub fn get_button_flags(button: i32) -> (u32, u32) {
    match button {
        2 => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
        3 => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
        _ => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
    }
}


#[cfg(target_os = "linux")]
mod linux {
    pub fn use_x11() -> bool {
        std::env::var_os("DISPLAY").is_some()
    }



    pub mod x11 {
        use std::sync::OnceLock;
        use x11rb::connection::Connection;
        use x11rb::protocol::xproto::ConnectionExt as XprotoExt;
        use x11rb::protocol::xtest::ConnectionExt as XTestExt;
        use x11rb::rust_connection::RustConnection;

        struct State {
            conn: RustConnection,
            root: u32,
            screen_num: usize,
        }

        static STATE: OnceLock<Option<State>> = OnceLock::new();

        fn get() -> Option<&'static State> {
            STATE.get_or_init(|| {
                match x11rb::connect(None) {
                    Ok((conn, snum)) => {
                        let root = conn.setup().roots[snum].root;
                        Some(State { conn, root, screen_num: snum })
                    }
                    Err(e) => {
                        log::error!("[x11] Failed to connect to X server (DISPLAY={:?}): {e}", std::env::var_os("DISPLAY"));
                        None
                    }
                }
            }).as_ref()
        }

        pub fn cursor_pos() -> Option<(i32, i32)> {
            let s = get()?;
            match s.conn.query_pointer(s.root) {
                Ok(cookie) => match cookie.reply() {
                    Ok(r) => Some((r.root_x as i32, r.root_y as i32)),
                    Err(e) => {
                        log::error!("[x11] query_pointer reply failed: {e:?}");
                        None
                    }
                },
                Err(e) => {
                    log::error!("[x11] query_pointer request failed: {e:?}");
                    None
                }
            }
        }

        pub fn virtual_screen() -> Option<super::super::VirtualScreenRect> {
            let s = get()?;
            let screen = &s.conn.setup().roots[s.screen_num];
            Some(super::super::VirtualScreenRect::new(
                0, 0,
                screen.width_in_pixels as i32,
                screen.height_in_pixels as i32,
            ))
        }

        pub fn monitor_rects() -> Option<Vec<super::super::VirtualScreenRect>> {
            use x11rb::protocol::randr::ConnectionExt as RandrExt;
            let s = get()?;

            let randr = s.conn.randr_get_monitors(s.root, true)
                .ok()
                .and_then(|c| c.reply().ok())
                .filter(|r| !r.monitors.is_empty())
                .map(|reply| {
                    let mut rects: Vec<_> = reply.monitors.iter().map(|m| {
                        super::super::VirtualScreenRect::new(
                            m.x as i32, m.y as i32, m.width as i32, m.height as i32,
                        )
                    }).collect();
                    rects.sort_by_key(|r| (r.top, r.left));
                    rects
                });

            randr.or_else(|| virtual_screen().map(|r| vec![r]))
        }

        pub fn move_cursor(x: i32, y: i32) {
            let Some(s) = get() else {
                log::error!("[x11] Cannot move cursor: no X11 connection available");
                return;
            };
            if let Err(e) = s.conn.warp_pointer(0u32, s.root, 0, 0, 0, 0, x as i16, y as i16) {
                log::error!("[x11] warp_pointer request failed: {e:?}");
            }
            if let Err(e) = s.conn.flush() {
                log::error!("[x11] flush failed: {e:?}");
            }
        }

        pub fn send_button(flags: u32) {
            let Some(s) = get() else {
                log::error!("[x11] Cannot send button: no X11 connection available");
                return;
            };
            let (button, is_down) = super::decode_linux_flag(flags);
            let event_type: u8 = if is_down { 4 } else { 5 };
            let x11_btn: u8 = match button {
                2 => 3,
                3 => 2,
                _ => 1,
            };
            if let Err(e) = s.conn.xtest_fake_input(event_type, x11_btn, 0, s.root, 0, 0, 0) {
                log::error!("[x11] xtest_fake_input request failed: {e:?}");
            }
            if let Err(e) = s.conn.flush() {
                log::error!("[x11] Connection flush failed: {e:?}");
            }
        }
    }



    pub mod uinput {
        use std::sync::{Mutex, OnceLock};
        use evdev::uinput::VirtualDevice;
        use evdev::{AttributeSet, EventType, InputEvent, Key, RelativeAxisType};

        static DEVICE: OnceLock<Option<Mutex<VirtualDevice>>> = OnceLock::new();

        fn get() -> Option<&'static Mutex<VirtualDevice>> {
            DEVICE.get_or_init(|| {
                let builder = match evdev::uinput::VirtualDeviceBuilder::new() {
                    Ok(b) => b,
                    Err(e) => {
                        log::error!("[uinput] Failed to open /dev/uinput: {e}. Make sure the uinput module is loaded and your user is in the 'input' group.");
                        return None;
                    }
                };
                let dev = match builder
                    .name("blur-autoclicker-mouse")
                    .with_keys(&AttributeSet::from_iter([
                        Key::BTN_LEFT,
                        Key::BTN_RIGHT,
                        Key::BTN_MIDDLE,
                    ]))
                    .and_then(|b| b.with_relative_axes(&AttributeSet::from_iter([
                        RelativeAxisType::REL_X,
                        RelativeAxisType::REL_Y,
                    ])))
                    .and_then(|b| b.build())
                {
                    Ok(d) => d,
                    Err(e) => {
                        log::error!("[uinput] Failed to build virtual device: {e}");
                        return None;
                    }
                };
                Some(Mutex::new(dev))
            }).as_ref()
        }

        pub fn available() -> bool {
            get().is_some()
        }

        pub fn send_button(flags: u32) {
            let Some(dev_lock) = get() else {
                log::error!("[uinput] Cannot send button: virtual device unavailable - is uinput module loaded and user in 'input' group?");
                return;
            };
            let Ok(mut dev) = dev_lock.lock() else {
                log::error!("[uinput] Failed to lock virtual device mutex");
                return;
            };
            let (button, is_down) = super::decode_linux_flag(flags);
            let key = match button {
                2 => Key::BTN_RIGHT,
                3 => Key::BTN_MIDDLE,
                _ => Key::BTN_LEFT,
            };
            let value: i32 = if is_down { 1 } else { 0 };
            if let Err(e) = dev.emit(&[
                InputEvent::new(EventType::KEY, key.code(), value),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
            ]) {
                log::error!("[uinput] emit failed: {e}");
            }
        }

        #[allow(dead_code)]
        pub fn move_relative(dx: i32, dy: i32) {
            let Some(dev_lock) = get() else { return };
            let Ok(mut dev) = dev_lock.lock() else { return };
            let _ = dev.emit(&[
                InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_X.0, dx),
                InputEvent::new(EventType::RELATIVE, RelativeAxisType::REL_Y.0, dy),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
            ]);
        }
    }


    pub const LEFT_DOWN: u32 = 0x11;
    pub const LEFT_UP: u32 = 0x10;
    pub const RIGHT_DOWN: u32 = 0x21;
    pub const RIGHT_UP: u32 = 0x20;
    pub const MIDDLE_DOWN: u32 = 0x31;
    pub const MIDDLE_UP: u32 = 0x30;

    pub fn decode_linux_flag(flags: u32) -> (u8, bool) {
        ((flags >> 4) as u8, (flags & 1) == 1)
    }
}

#[cfg(target_os = "linux")]
pub fn linux_use_x11() -> bool {
    linux::use_x11()
}

#[cfg(target_os = "linux")]
pub fn uinput_available() -> bool {
    linux::uinput::available()
}

#[cfg(target_os = "linux")]
pub fn linux_mouse_diagnostic() -> String {
    if linux::use_x11() {
        match x11rb::connect(None) {
            Ok((conn, _)) => {
                use x11rb::connection::RequestConnection;
                let xtest_ok = conn.extension_information(x11rb::protocol::xtest::X11_EXTENSION_NAME).ok().flatten().is_some();
                let mut msg = format!("X11 backend (DISPLAY={:?})", std::env::var_os("DISPLAY"));
                if xtest_ok {
                    msg.push_str(" - XTEST extension available");
                } else {
                    msg.push_str(" - XTEST extension MISSING");
                }

                if std::env::var_os("WAYLAND_DISPLAY").is_some() {
                    msg.push_str(" | WARNING: running under XWayland - clicks only affect X11 windows, not native Wayland apps");
                }
                msg
            }
            Err(e) => {
                format!("X11 backend (DISPLAY={:?}) - CONNECTION FAILED: {e}", std::env::var_os("DISPLAY"))
            }
        }
    } else {
        let uinput_path = std::path::Path::new("/dev/uinput");
        let exists = uinput_path.exists();
        let writable = std::fs::OpenOptions::new().write(true).open(uinput_path).is_ok();
        if exists && writable {
            format!("Wayland uinput backend - /dev/uinput is accessible")
        } else if exists {
            format!("Wayland uinput backend - /dev/uinput exists but is NOT writable (user not in 'input' group?)")
        } else {
            format!("Wayland uinput backend - /dev/uinput does not exist (uinput module not loaded?)")
        }
    }
}

#[cfg(target_os = "linux")]
pub fn current_cursor_position() -> Option<(i32, i32)> {
    if linux::use_x11() {
        linux::x11::cursor_pos()
    } else {
        log::warn!("[mouse] cursor position unavailable on pure Wayland without XWayland");
        None
    }
}

#[cfg(target_os = "linux")]
pub fn current_virtual_screen_rect() -> Option<VirtualScreenRect> {
    if linux::use_x11() {
        linux::x11::virtual_screen()
    } else {
        None
    }
    .or_else(|| {
        let guard = CACHED_VIRTUAL_SCREEN_RECT.lock().unwrap();
        guard.clone()
    })
}

#[cfg(target_os = "linux")]
pub fn current_monitor_rects() -> Option<Vec<VirtualScreenRect>> {
    if linux::use_x11() {
        linux::x11::monitor_rects()
    } else {
        current_virtual_screen_rect().map(|r| vec![r])
    }
    .or_else(|| {
        let guard = CACHED_MONITOR_RECTS.lock().unwrap();
        guard.clone()
    })
}

#[cfg(target_os = "linux")]
#[inline]
pub fn move_mouse(x: i32, y: i32) {
    if linux::use_x11() {
        linux::x11::move_cursor(x, y);
    } else {
        log::debug!("[mouse] move_mouse: Wayland abs positioning not supported");
    }
}

#[cfg(target_os = "linux")]
#[inline]
pub fn send_mouse_event(flags: u32) {
    if linux::use_x11() {
        linux::x11::send_button(flags);
    } else {
        linux::uinput::send_button(flags);
    }
}

#[cfg(target_os = "linux")]
pub fn send_batch(down: u32, up: u32, n: usize, _hold_ms: u32) {
    for _ in 0..n {
        send_mouse_event(down);
        send_mouse_event(up);
    }
}

#[cfg(target_os = "linux")]
#[inline]
pub fn get_button_flags(button: i32) -> (u32, u32) {
    match button {
        2 => (linux::RIGHT_DOWN, linux::RIGHT_UP),
        3 => (linux::MIDDLE_DOWN, linux::MIDDLE_UP),
        _ => (linux::LEFT_DOWN, linux::LEFT_UP),
    }
}


#[inline]
pub fn get_cursor_pos() -> (i32, i32) {
    current_cursor_position().unwrap_or((0, 0))
}

pub fn send_clicks(
    down: u32,
    up: u32,
    count: usize,
    hold_ms: u32,
    use_double_click_gap: bool,
    double_click_delay_ms: u32,
    control: &RunControl,
) {
    if count == 0 {
        return;
    }

    if plan.kind == ClickCycleKind::Single && count > 1 && plan.first_hold_ms == 0 {
        send_batch(down, up, count);
        return;
    }

    let is_active = || control.is_active();
    let mut sleep_for = |duration| sleep_interruptible(duration, control);

    for _ in 0..count {
        if !execute_click_cycle(
            plan,
            &mut || send_mouse_event(down),
            &mut || send_mouse_event(up),
            &mut sleep_for,
            &is_active,
        ) {
            return;
        }
    }
}

#[inline]
pub fn ease_in_out_quad(t: f64) -> f64 {
    if t < 0.5 { 2.0 * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(2) / 2.0 }
}

#[inline]
pub fn cubic_bezier(t: f64, p0: f64, p1: f64, p2: f64, p3: f64) -> f64 {
    let u = 1.0 - t;
    u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3
}

fn smooth_move_inner(
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    duration_ms: u64,
    rng: &mut crate::engine::rng::SmallRng,
    allow_overshoot: bool,
) {
    if duration_ms < 3 || (start_x == end_x && start_y == end_y) {
        move_mouse(end_x, end_y);
        return;
    }

    let (start_x, start_y) = (start_x as f64, start_y as f64);
    let (target_x, target_y) = (end_x as f64, end_y as f64);
    let delta_x = target_x - start_x;
    let delta_y = target_y - start_y;
    let distance = delta_x.hypot(delta_y);

    if distance < 3.0 {
        move_mouse(end_x, end_y);
        return;
    }

    let steps = if duration_ms <= 12 {
        (duration_ms / 3).clamp(1, 4) as usize
    } else {
        ((duration_ms / 8) as usize).clamp(4, 75)
    };

    let tick_duration = Duration::from_millis(duration_ms) / steps as u32;
    let start_time = Instant::now();

    let cp1_ratio = rng.next_f64() * 0.28 + 0.20;
    let cp2_ratio = rng.next_f64() * 0.24 + 0.55;

    let max_perp_offset = (distance * 0.29).min(76.0);

    let perp_x = -delta_y / distance;
    let perp_y = delta_x / distance;

    let offset_1 = (rng.next_f64() * 0.41 + 0.07)
        * max_perp_offset
        * (if rng.next_f64() >= 0.5 { 1.0 } else { -1.0 });
    let offset_2 = (rng.next_f64() * 0.41 + 0.07)
        * max_perp_offset
        * (if rng.next_f64() >= 0.5 { 1.0 } else { -1.0 });

    let control_1x = start_x + delta_x * cp1_ratio + perp_x * offset_1;
    let control_1y = start_y + delta_y * cp1_ratio + perp_y * offset_1;
    let control_2x = start_x + delta_x * cp2_ratio + perp_x * offset_2;
    let control_2y = start_y + delta_y * cp2_ratio + perp_y * offset_2;

    let mid_wobble = rng.next_f64() < 0.37 && duration_ms > 22;
    let wobble_step = if mid_wobble { steps / 2 } else { 0 };

    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let ease = ease_in_out_quad(t);

        let mut current_x = cubic_bezier(ease, start_x, control_1x, control_2x, target_x);
        let mut current_y = cubic_bezier(ease, start_y, control_1y, control_2y, target_y);

        if mid_wobble && i == wobble_step {
            let wobble = rng.next_f64() * 1.7 + 0.7;
            let sign = if rng.next_f64() >= 0.5 { 1.0 } else { -1.0 };
            current_x += perp_x * wobble * sign;
            current_y += perp_y * wobble * sign;
        }

        move_mouse(current_x as i32, current_y as i32);

        if i < steps {
            let elapsed = start_time.elapsed();
            let expected = tick_duration * (i + 1) as u32;

            if expected > elapsed {
                std::thread::sleep(expected - elapsed);
            }
        }
    }

    if allow_overshoot && duration_ms > 16 && rng.next_f64() < 0.47 {
        let overshoot_amount = rng.next_f64() * 6.3 + 2.2;
        let dir_x = delta_x / distance;
        let dir_y = delta_y / distance;

        let over_x = (target_x + dir_x * overshoot_amount) as i32;
        let over_y = (target_y + dir_y * overshoot_amount) as i32;

        let correction_ms = (duration_ms as f64 * 0.19).max(4.0) as u64;

        smooth_move_inner(end_x, end_y, over_x, over_y, correction_ms, rng, false);
        smooth_move_inner(
            over_x,
            over_y,
            end_x,
            end_y,
            (correction_ms * 2 / 3).max(3),
            rng,
            false,
        );
    }
}

pub fn smooth_move(
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    duration_ms: u64,
    rng: &mut crate::engine::rng::SmallRng,
) {
    smooth_move_inner(start_x, start_y, end_x, end_y, duration_ms, rng, true);
}
