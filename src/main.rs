// Hide console window in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod indexer;
mod platform;
mod search;
mod ui;

use config::Config;
use iced::Application;
use ui::App;

fn main() -> iced::Result {
    let config = Config::load();

    // Spawn background thread to add WS_THICKFRAME for resize borders
    // after iced/winit creates the frameless window
    platform::setup_frameless_resize();
    
    App::run(iced::Settings {
        window: iced::window::Settings {
            size: iced::Size::new(config.window_width, config.window_height),
            min_size: Some(iced::Size::new(400.0, 300.0)),
            decorations: true,
            transparent: false,
            resizable: true,
            ..Default::default()
        },
        default_font: iced::Font::DEFAULT,
        default_text_size: iced::Pixels(14.0),
        flags: config,
        ..Default::default()
    })
}
