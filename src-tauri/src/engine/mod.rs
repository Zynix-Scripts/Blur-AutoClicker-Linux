pub mod failsafe;
pub mod keyboard;
pub mod mouse;
pub mod rng;
pub mod stats;
pub mod worker;
use crate::engine::mouse::VirtualScreenRect;
use std::sync::atomic::AtomicI64;
pub use worker::start_clicker;
pub const AUTOCLICKER_EXTRA_INFO: usize = 0x800D_A5A5; //Just a random Identifier

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SequenceTarget {
    pub x: i32,
    pub y: i32,
    pub clicks: usize,
}

#[derive(Clone, Debug)]
pub struct ClickerConfig {
    pub interval: f64,
    pub variation: f64,
    pub limit: i32,
    pub duty: f64,
    pub time_limit: f64,
    pub button: i32,
    pub double_click_enabled: bool,
    pub double_click_delay_ms: u32,
    pub position_enabled: bool,
    pub pos_x: i32,
    pub pos_y: i32,
    pub custom_stop_zone_enabled: bool,
    pub custom_stop_zone: VirtualScreenRect,
    pub offset: f64,
    pub offset_chance: f64,
    pub smoothing: i32,
    pub corner_stop_enabled: bool,
    pub corner_stop_tl: i32,
    pub corner_stop_tr: i32,
    pub corner_stop_bl: i32,
    pub corner_stop_br: i32,
    pub edge_stop_enabled: bool,
    pub edge_stop_top: i32,
    pub edge_stop_right: i32,
    pub edge_stop_bottom: i32,
    pub edge_stop_left: i32,
    pub high_cps_mode: bool,
    pub input_type: i32,
    pub key_code: u16,
    pub key_token: String,
    pub keyboard_uppercase: bool,
    pub sequence_enabled: bool,
    pub sequence_points: Vec<SequenceTarget>,
}

impl ClickerConfig {
    pub fn use_sequence(&self) -> bool {
        self.sequence_enabled && !self.sequence_points.is_empty()
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct RunOutcome {
    pub stop_reason: String,
    pub click_count: i64,
    pub elapsed_secs: f64,
    pub avg_cpu: f64,
}

static CLICK_COUNT: AtomicI64 = AtomicI64::new(0);

#[cfg(target_os = "windows")]
#[link(name = "ntdll")]
extern "system" {
    pub fn NtSetTimerResolution(
        DesiredResolution: u32,
        SetResolution: u8,
        CurrentResolution: *mut u32,
    ) -> u32;
}
