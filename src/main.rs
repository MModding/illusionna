use iced;
use iced::Size;
use crate::app::IllusionnaApp;

mod app;
mod workspace;
mod wrapper;
mod osc; // OSC => Outside Source Control

fn main() -> octocrab::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    iced::application("Illusionna", IllusionnaApp::update, IllusionnaApp::view)
        .window_size(Size::new(1280f32, 720f32))
        .resizable(false)
        .run_with(IllusionnaApp::new)
        .unwrap();
    Ok(())
}
