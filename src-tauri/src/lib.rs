// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod domain;
mod errors;
mod handlers;
mod infra;
mod onboarding;
mod pkce;
mod services;
mod state;

use app::run;

fn main() {
    run().expect("error while running tauri application");
}
