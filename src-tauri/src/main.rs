// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod config;
mod errors;
mod state;
mod domain;
mod services;
mod infra;
mod handlers;
mod pkce;


fn main() {

    app::run().expect("error while running Pausa");
    // pausa_lib::run()
}
