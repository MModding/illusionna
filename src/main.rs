use http;
use iced;
use iced::{Application, Settings};

mod gui;
mod workspace;
mod wrapper;
mod osc; // OSC => Outside Source Control

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    gui::IllusionnaApp::run(Settings::default()).expect("IllusionnaApp execution failed");
    Ok(())
}
