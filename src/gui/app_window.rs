mod app_window;
mod preview;
mod text_input;
mod widgets;
pub mod timeline;

use fltk::{app::App, prelude::*};

pub fn run_app() {
    let app = App::default();
    let mut wind = app_window::create_main_window();
    wind.show();
    app.run().unwrap();
}