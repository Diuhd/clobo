// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clobo_lib::app;

#[cfg(target_os = "windows")]
fn main() {
    app::run()
}
