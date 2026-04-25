use std::time::Duration;

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

    #[inline]
    pub fn offset_from(self, origin: VirtualScreenRect) -> Self {
        Self::new(self.left - origin.left, self.top - origin.top, self.width, self.height)
    }
}

// ─── Windows ─────────────────────────────────────────────────────────────────

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
pub fn move_mouse(x: i32, y: i32) {
    unsafe { SetCursorPos(x, y) };
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

// ─── Linux ───────────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
mod linux {
    pub fn use_x11() -> bool {
        std::env::var_os("DISPLAY").is_some()
    }

    // ── X11 backend (XTest + WarpPointer + RandR) ──────────────────────────

    pub mod x11 {
        use std::sync::OnceLock;
        use x11rb::connection::Connection;
        use x11rb::protocol::xproto::ConnectionExt as XprotoExt;
        use x11rb::protocol::xtest::ConnectionExt as XTestExt;
        use x11rb::rust_connection::RustConnection;

        struct State {
            conn: RustConnection,
            root: u32,
        }

        static STATE: OnceLock<Option<State>> = OnceLock::new();

        fn get() -> Option<&'static State> {
            STATE.get_or_init(|| {
                let (conn, snum) = x11rb::connect(None).ok()?;
                let root = conn.setup().roots[snum].root;
                Some(State { conn, root })
            }).as_ref()
        }

        pub fn cursor_pos() -> Option<(i32, i32)> {
            let s = get()?;
            let r = s.conn.query_pointer(s.root).ok()?.reply().ok()?;
            Some((r.root_x as i32, r.root_y as i32))
        }

        pub fn virtual_screen() -> Option<super::super::VirtualScreenRect> {
            let s = get()?;
            let screen = &s.conn.setup().roots[0];
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
            let Some(s) = get() else { return };
            let _ = s.conn.warp_pointer(0u32, s.root, 0, 0, 0, 0, x as i16, y as i16);
            let _ = s.conn.flush();
        }

        pub fn send_button(flags: u32) {
            let Some(s) = get() else { return };
            let (button, is_down) = super::decode_linux_flag(flags);
            let event_type: u8 = if is_down { 4 } else { 5 };
            let x11_btn: u8 = match button {
                2 => 3,
                3 => 2,
                _ => 1,
            };
            let _ = s.conn.xtest_fake_input(event_type, x11_btn, 0, s.root, 0, 0, 0);
            let _ = s.conn.flush();
        }
    }

    // ── uinput backend (Wayland) ───────────────────────────────────────────

    pub mod uinput {
        use std::sync::{Mutex, OnceLock};
        use evdev::uinput::VirtualDevice;
        use evdev::{AttributeSet, EventType, InputEvent, Key, RelativeAxisType};

        static DEVICE: OnceLock<Option<Mutex<VirtualDevice>>> = OnceLock::new();

        fn get() -> Option<&'static Mutex<VirtualDevice>> {
            DEVICE.get_or_init(|| {
                let dev = evdev::uinput::VirtualDeviceBuilder::new().ok()?
                    .name("blur-autoclicker-mouse")
                    .with_keys(&AttributeSet::from_iter([
                        Key::BTN_LEFT,
                        Key::BTN_RIGHT,
                        Key::BTN_MIDDLE,
                    ])).ok()?
                    .with_relative_axes(&AttributeSet::from_iter([
                        RelativeAxisType::REL_X,
                        RelativeAxisType::REL_Y,
                    ])).ok()?
                    .build().ok()?;
                Some(Mutex::new(dev))
            }).as_ref()
        }

        pub fn available() -> bool {
            get().is_some()
        }

        pub fn send_button(flags: u32) {
            let Some(dev_lock) = get() else {
                log::error!("[uinput] device unavailable — is uinput module loaded and user in 'input' group?");
                return;
            };
            let Ok(mut dev) = dev_lock.lock() else { return };
            let (button, is_down) = super::decode_linux_flag(flags);
            let key = match button {
                2 => Key::BTN_RIGHT,
                3 => Key::BTN_MIDDLE,
                _ => Key::BTN_LEFT,
            };
            let value: i32 = if is_down { 1 } else { 0 };
            let _ = dev.emit(&[
                InputEvent::new(EventType::KEY, key.code(), value),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
            ]);
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

    // flag encoding: high nibble = button (1=left,2=right,3=middle), low bit = is_down
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
}

#[cfg(target_os = "linux")]
pub fn current_monitor_rects() -> Option<Vec<VirtualScreenRect>> {
    if linux::use_x11() {
        linux::x11::monitor_rects()
    } else {
        current_virtual_screen_rect().map(|r| vec![r])
    }
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

// ─── Shared (all platforms) ───────────────────────────────────────────────────

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

    if !use_double_click_gap && count > 1 && hold_ms == 0 {
        send_batch(down, up, count, hold_ms);
        return;
    }

    for index in 0..count {
        if !control.is_active() {
            return;
        }

        send_mouse_event(down);
        if hold_ms > 0 {
            sleep_interruptible(Duration::from_millis(hold_ms as u64), control);
            if !control.is_active() {
                return;
            }
        }
        send_mouse_event(up);

        if index + 1 < count && use_double_click_gap && double_click_delay_ms > 0 {
            sleep_interruptible(Duration::from_millis(double_click_delay_ms as u64), control);
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

pub fn smooth_move(
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    duration_ms: u64,
    rng: &mut SmallRng,
) {
    if duration_ms < 5 {
        move_mouse(end_x, end_y);
        return;
    }

    let (sx, sy) = (start_x as f64, start_y as f64);
    let (ex, ey) = (end_x as f64, end_y as f64);
    let (dx, dy) = (ex - sx, ey - sy);
    let distance = (dx * dx + dy * dy).sqrt();
    if distance < 1.0 {
        return;
    }

    let (perp_x, perp_y) = (-dy / distance, dx / distance);
    let sign = |b: bool| if b { 1.0f64 } else { -1.0 };
    let o1 = (rng.next_f64() * 0.3 + 0.15) * distance * sign(rng.next_f64() >= 0.5);
    let o2 = (rng.next_f64() * 0.3 + 0.15) * distance * sign(rng.next_f64() >= 0.5);
    let cp1x = sx + dx * 0.33 + perp_x * o1;
    let cp1y = sy + dy * 0.33 + perp_y * o1;
    let cp2x = sx + dx * 0.66 + perp_x * o2;
    let cp2y = sy + dy * 0.66 + perp_y * o2;

    let steps = (duration_ms as usize).clamp(10, 200);
    let step_dur = Duration::from_millis(duration_ms / steps as u64);

    for i in 0..=steps {
        let t = ease_in_out_quad(i as f64 / steps as f64);
        move_mouse(
            cubic_bezier(t, sx, cp1x, cp2x, ex) as i32,
            cubic_bezier(t, sy, cp1y, cp2y, ey) as i32,
        );
        if i < steps {
            std::thread::sleep(step_dur);
        }
    }
}
