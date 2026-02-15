mod global;
mod palette;
use crate::Window;

pub fn setup_shortcuts(window: &Window) {
    global::setup(window);
    palette::setup(window);
}