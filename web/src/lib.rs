/**
 * @file lib.rs
 * @author Nguyen Le Duy
 * @date 31/03/2025
 * @brief WebAssembly module for the simulator
 */
mod api;
mod app;
mod tracker;
mod widgets;

pub use app::SimulatorApp;
pub use tracker::Tracker;

mod simulator;
