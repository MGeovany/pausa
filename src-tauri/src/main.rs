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
mod notification_service;
mod onboarding;
mod pkce;
mod services;
mod state;
mod strict_mode;
mod window_manager;

fn main() {
    app::run().expect("error while running Pausa");
    // pausa_lib::run()
}
