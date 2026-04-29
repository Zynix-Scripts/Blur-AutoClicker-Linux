#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use app_lib::run;

fn main() {



    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("DISPLAY").is_some()
            && std::env::var_os("WAYLAND_DISPLAY").is_some()
            && std::env::var_os("GDK_BACKEND").is_none()
        {
            std::env::set_var("GDK_BACKEND", "x11");
        }
    }
    run();
}
