// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api_models;
mod app;
mod config;
mod cycle_orchestrator;
mod database;
mod domain;
mod errors;
mod handlers;
mod infra;
pub mod notification_service;
mod onboarding;
mod pkce;
mod services;
mod state;
pub mod strict_mode;
pub mod window_manager;

use app::run;

fn main() {
    run().expect("error while running tauri application");
}
