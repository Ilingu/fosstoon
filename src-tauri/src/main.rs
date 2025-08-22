// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    fosstoon_lib::run();

    Ok(())
}
