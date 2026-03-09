// Suppress the extra console window in Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    fast_disk_cleaner_lib::run();
}
