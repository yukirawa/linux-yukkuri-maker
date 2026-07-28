pub mod app_window;
pub mod dialogs;
pub mod media_browser;
pub mod menu;
pub mod preview;
pub mod properties;
pub mod serif_input;
pub mod state;
pub mod status_bar;
pub mod toolbar;
pub mod timeline;
pub mod widgets;

use fltk::{app::App, prelude::*};

pub fn run_app() {
    let app = App::default();
    let (mut wind, _state) = app_window::create_main_window();
    wind.show();
    app.run().unwrap();
}