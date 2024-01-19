use druid::{AppLauncher, WindowDesc};

mod controllers;
mod double_click;
mod data;
mod delegate;
mod view;

use data::AppState;
use delegate::Delegate;
use view::build_ui;

pub fn main() {
    let main_window = WindowDesc::new(build_ui())
        .title("Druid app")
        .window_size((400.0, 400.0));

    AppLauncher::with_window(main_window)
        .delegate(Delegate {})
        .launch(AppState::new())
        .expect("Failed to launch application");
}
