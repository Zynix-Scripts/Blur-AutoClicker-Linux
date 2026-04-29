use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDepsInfo {
    pub display_server: String,
    pub uinput_accessible: bool,
    pub in_input_group: bool,
    pub uinput_module_loaded: bool,
    pub is_root: bool,
    pub mouse_backend: String,
    pub warnings: Vec<String>,
}

#[tauri::command]
pub fn check_system_deps() -> SystemDepsInfo {
    #[cfg(not(target_os = "linux"))]
    {
        SystemDepsInfo {
            display_server: "n/a".to_string(),
            uinput_accessible: true,
            in_input_group: true,
            uinput_module_loaded: true,
            is_root: false,
            mouse_backend: "n/a".to_string(),
            warnings: vec![],
        }
    }

    #[cfg(target_os = "linux")]
    check_linux()
}

#[cfg(target_os = "linux")]
fn check_linux() -> SystemDepsInfo {
    let has_x11 = std::env::var_os("DISPLAY").is_some();
    let has_wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();

    let display_server = if has_x11 {
        "x11".to_string()
    } else if has_wayland {
        "wayland".to_string()
    } else {
        "unknown".to_string()
    };

    let is_root = unsafe { libc::getuid() } == 0;
    let uinput_module_loaded = std::path::Path::new("/dev/uinput").exists();
    let uinput_accessible = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/uinput")
        .is_ok();
    let in_input_group = is_in_input_group();

    let mouse_backend = crate::engine::mouse::linux_mouse_diagnostic();

    let mut warnings: Vec<String> = Vec::new();


    if has_x11 {
        log::info!("[SystemCheck] X11 detected - autoclicker uses XTEST (full feature set)");
    }
    if has_wayland && !has_x11 {
        log::info!("[SystemCheck] Pure Wayland detected - autoclicker uses uinput (clicking works; position pick and always-on-top are limited)");
    }

    if !has_x11 {
        if !uinput_module_loaded {
            warnings.push(
                "uinput module not loaded.\nFix: sudo modprobe uinput\nTo persist: add 'uinput' to /etc/modules".to_string(),
            );
        } else if !uinput_accessible {
            if !in_input_group {
                warnings.push(
                    "User is not in the 'input' group - mouse clicking won't work on Wayland.\nFix: sudo usermod -aG input $USER\nThen log out and back in.".to_string(),
                );
            } else {
                warnings.push(
                    "/dev/uinput exists but is not accessible.\nFix: sudo chmod 660 /dev/uinput".to_string(),
                );
            }
        }
    }


    if has_x11 && has_wayland {
        warnings.push(
            "XWayland detected: the X11 click backend only works on X11 windows. Native Wayland applications will not receive clicks.".to_string(),
        );
    }


    if has_x11 && mouse_backend.contains("CONNECTION FAILED") {
        warnings.push(
            format!("X11 connection failed. Check your DISPLAY environment variable and ensure X11 authentication is configured."),
        );
    }

    SystemDepsInfo {
        display_server,
        uinput_accessible,
        in_input_group,
        uinput_module_loaded,
        is_root,
        mouse_backend,
        warnings,
    }
}

#[cfg(target_os = "linux")]
fn is_in_input_group() -> bool {
    let Some(input_gid) = get_group_gid("input") else {
        return false;
    };
    unsafe {
        let mut groups = [0u32; 64];
        let n = libc::getgroups(groups.len() as libc::c_int, groups.as_mut_ptr());
        if n < 0 {
            return false;
        }
        groups[..n as usize].contains(&input_gid)
    }
}

#[cfg(target_os = "linux")]
fn get_group_gid(name: &str) -> Option<u32> {
    let content = std::fs::read_to_string("/etc/group").ok()?;
    for line in content.lines() {
        let mut parts = line.splitn(4, ':');
        let grp_name = parts.next()?;
        parts.next();
        let gid_str = parts.next()?;
        if grp_name == name {
            return gid_str.parse().ok();
        }
    }
    None
}
