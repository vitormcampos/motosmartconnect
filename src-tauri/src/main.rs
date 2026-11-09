// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "macos")]
    smartconnect_mac_lib::adopt_bundle();
    smartconnect_mac_lib::run()
}
