#![windows_subsystem = "windows"]
use iced;
use iced::{Font, Size};
use crate::app::IllusionnaApp;

mod app;
mod workspace;
mod wrapper;
mod osc; // OSC => Outside Source Control

fn main() -> octocrab::Result<()> {
    iced::application("Illusionna", IllusionnaApp::update, IllusionnaApp::view)
        .window_size(Size::new(854f32, 480f32))
        .resizable(false)
        .font(include_bytes!("../resources/inter.ttf").as_slice())
        .default_font(Font::with_name("Inter 24pt"))
        .run_with(IllusionnaApp::new)
        .unwrap();
    Ok(())
}
